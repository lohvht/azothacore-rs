use std::{path::Path, sync::Arc};

use azothacore_common::{
    banner,
    configuration::{
        ConfigMgr,
        DatabaseType::{Character as DBFlagCharacter, Hotfix as DBFlagHotfix, Login as DBFlagLogin, World as DBFlagWorld},
        DbUpdates,
    },
    log,
    AzResult,
    AZOTHA_DB_IMPORT_CONFIG,
    CONF_DIR,
};
use azothacore_database::{
    database_env::{CharacterDatabase, HotfixDatabase, LoginDatabase, WorldDatabase},
    database_loader::DatabaseLoader,
};
use azothacore_modules::SCRIPTS as MODULES_LIST;
use azothacore_server::shared::dropper_wrapper_fn;
use clap::Parser;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[cfg(target_os = "windows")]
fn signal_handler(rt: &tokio::runtime::Runtime, expression: impl Future<Output = AzResult<T>>) -> JoinHandle<Result<(), std::io::Error>> {
    rt.spawn(
        async {
            use tokio::signal::windows::ctrl_break;
            let mut sig_break = ctrl_break()?;
            receive_signal_and_run_expr!(
                S_WORLD.write().await.stop_now(1),
                "SIGBREAK" => sig_break
            );
            Ok(())
        }
        .instrument(info_span!("signal_handler")),
    )
}

#[cfg(target_os = "linux")]
fn signal_handler(rt: &tokio::runtime::Runtime, cancel_token: CancellationToken) -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    use azothacore_server::{receive_signal_and_run_expr, short_curcuit_unix_signal_unwrap};
    use tokio::signal::unix::SignalKind;
    use tracing::{info_span, Instrument};

    fn cancel_func(cancel_token: CancellationToken) -> AzResult<()> {
        cancel_token.cancel();
        Ok(())
    }

    rt.spawn(
        async move {
            let mut sig_interrupt = short_curcuit_unix_signal_unwrap!(SignalKind::interrupt());
            let mut sig_terminate = short_curcuit_unix_signal_unwrap!(SignalKind::terminate());
            let mut sig_quit = short_curcuit_unix_signal_unwrap!(SignalKind::quit());
            receive_signal_and_run_expr!(
                cancel_func(cancel_token.clone()),
                cancel_token,
                "SIGINT" => sig_interrupt
                "SIGTERM" => sig_terminate
                "SIGQUIT" => sig_quit
            );
            Ok(())
        }
        .instrument(info_span!("signal_handler")),
    )
}

fn main() -> AzResult<()> {
    let vm = ConsoleArgs::parse();
    {
        let mut cfg_mgr_w = ConfigMgr::m();
        cfg_mgr_w.configure(&vm.config, vm.dry_run);
        cfg_mgr_w.load_app_configs()?;
    };
    let _wg = {
        let cfg_mgr_r = ConfigMgr::r();
        // TODO: Setup DB logging. Original code below
        // // Init all logs
        // sLog->RegisterAppender<AppenderDB>();
        log::init(
            cfg_mgr_r.get_option::<String>("LogsDir")?,
            &cfg_mgr_r.get_option::<Vec<_>>("Appender")?,
            &cfg_mgr_r.get_option::<Vec<_>>("Logger")?,
        )
    };
    let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build()?);
    banner::azotha_banner_show("dbimport", || {
        info!(
            target:"dbimport",
            "> Using configuration file       {}",
            ConfigMgr::r().get_filename().display()
        )
    });

    let cancel_token = CancellationToken::new();

    let rt = runtime.clone();
    let ct = cancel_token.clone();
    let jh = signal_handler(&rt, ct.clone());
    let _signal_handle = dropper_wrapper_fn(runtime.handle(), ct, async move {
        jh.await??;
        Ok(())
    });

    runtime.block_on(start_db(cancel_token.clone()))?;
    let _db_handle = dropper_wrapper_fn(runtime.handle(), cancel_token.clone(), stop_db());

    info!(target:"dbimport", "Halting process...");

    Ok(())
}

/// Initialize connection to the database
async fn start_db(cancel_token: CancellationToken) -> AzResult<()> {
    let updates;
    let auth_cfg;
    let world_cfg;
    let character_cfg;
    let hotfix_cfg;
    {
        let config_mgr_r = ConfigMgr::r();
        updates = config_mgr_r.get_option::<DbUpdates>("Updates")?;
        auth_cfg = config_mgr_r.get_option("LoginDatabaseInfo")?;
        world_cfg = config_mgr_r.get_option("WorldDatabaseInfo")?;
        character_cfg = config_mgr_r.get_option("CharacterDatabaseInfo")?;
        hotfix_cfg = config_mgr_r.get_option("HotfixDatabaseInfo")?;
    }
    let login_db_loader = DatabaseLoader::new(cancel_token.clone(), DBFlagLogin, &auth_cfg, &updates, MODULES_LIST);
    let world_db_loader = DatabaseLoader::new(cancel_token.clone(), DBFlagWorld, &world_cfg, &updates, MODULES_LIST);
    let chars_db_loader = DatabaseLoader::new(cancel_token.clone(), DBFlagCharacter, &character_cfg, &updates, MODULES_LIST);
    let hotfixes_db_loader = DatabaseLoader::new(cancel_token.clone(), DBFlagHotfix, &hotfix_cfg, &updates, MODULES_LIST);

    LoginDatabase::set(login_db_loader.load().await?);
    WorldDatabase::set(world_db_loader.load().await?);
    CharacterDatabase::set(chars_db_loader.load().await?);
    HotfixDatabase::set(hotfixes_db_loader.load().await?);

    info!(target:"dbimport", "Started database connection pool.");
    Ok(())
}

async fn stop_db() -> AzResult<()> {
    info!(target:"dbimport", "Stopping database connection pool.");
    LoginDatabase::close().await;
    WorldDatabase::close().await;
    CharacterDatabase::close().await;
    HotfixDatabase::close().await;
    info!(target:"dbimport", "Stopped database connection pool.");

    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ConsoleArgs {
    /// Dry run
    #[arg(short, long = "dry-run")]
    dry_run: bool,
    /// use <arg> as configuration file
    #[arg(short, long, default_value_t = Path::new(CONF_DIR).join(AZOTHA_DB_IMPORT_CONFIG).to_str().unwrap().to_string())]
    config:  String,
    #[arg(short, long, default_value_t = String::new())]
    service: String,
}
