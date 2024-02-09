use std::{path::Path, sync::Arc};

use azothacore_common::{
    banner,
    configuration::{
        DatabaseType::{Character as DBFlagCharacter, Hotfix as DBFlagHotfix, Login as DBFlagLogin, World as DBFlagWorld},
        DbUpdates,
        CONFIG_MGR,
    },
    log,
    r#async::Context,
    AzResult,
    AZOTHA_DB_IMPORT_CONFIG,
    CONF_DIR,
};
use azothacore_database::{
    database_env::{CharacterDatabase, HotfixDatabase, LoginDatabase, WorldDatabase},
    database_loader::DatabaseLoader,
};
use azothacore_modules::SCRIPTS as MODULES_LIST;
use azothacore_server::shared::{panic_handler, signal_handler};
use clap::Parser;
use tokio::sync::oneshot;
use tracing::{error, info};

fn main() -> AzResult<()> {
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build()?);
    let root_ctx = Context::new(rt.handle());
    panic_handler(root_ctx.clone());
    let vm = ConsoleArgs::parse();
    {
        let mut cfg_mgr_w = CONFIG_MGR.blocking_write();
        cfg_mgr_w.configure(&vm.config, vm.dry_run);
        cfg_mgr_w.load_app_configs()?;
    };
    let _wg = {
        let cfg_mgr_r = CONFIG_MGR.blocking_read();
        // TODO: Setup DB logging. Original code below
        // // Init all logs
        // sLog->RegisterAppender<AppenderDB>();
        log::init(
            cfg_mgr_r.get_option::<String>("LogsDir")?,
            &cfg_mgr_r.get_option::<Vec<_>>("Appender")?,
            &cfg_mgr_r.get_option::<Vec<_>>("Logger")?,
        )
    };
    banner::azotha_banner_show("dbimport", || {
        info!(
            target:"dbimport",
            "> Using configuration file       {}",
            CONFIG_MGR.blocking_read().get_filename().display()
        )
    });

    let ctx = root_ctx.clone();
    root_ctx.spawn(signal_handler(ctx));

    let (db_started_send, db_started_recv) = oneshot::channel();
    let ctx = root_ctx.clone();
    root_ctx.spawn(async move {
        if let Err(e) = start_db(ctx.clone(), db_started_send).await {
            error!(target:"server::authserver", cause=%e, "error starting/stopping DB");
            ctx.cancel();
        }
    });
    // Enforce DB to be up first
    db_started_recv.blocking_recv().unwrap();

    info!(target:"dbimport", "Halting process...");

    Ok(())
}

/// Initialize connection to the database
async fn start_db(ctx: Context, db_started_send: oneshot::Sender<()>) -> AzResult<()> {
    let updates;
    let auth_cfg;
    let world_cfg;
    let character_cfg;
    let hotfix_cfg;
    {
        let config_mgr_r = CONFIG_MGR.read().await;
        updates = config_mgr_r.get_option::<DbUpdates>("Updates")?;
        auth_cfg = config_mgr_r.get_option("LoginDatabaseInfo")?;
        world_cfg = config_mgr_r.get_option("WorldDatabaseInfo")?;
        character_cfg = config_mgr_r.get_option("CharacterDatabaseInfo")?;
        hotfix_cfg = config_mgr_r.get_option("HotfixDatabaseInfo")?;
    }
    let modules: Vec<_> = MODULES_LIST.iter().map(|s| s.to_string()).collect();
    let login_db_loader = DatabaseLoader::new(DBFlagLogin, auth_cfg, updates.clone(), modules.clone());
    let world_db_loader = DatabaseLoader::new(DBFlagWorld, world_cfg, updates.clone(), modules.clone());
    let chars_db_loader = DatabaseLoader::new(DBFlagCharacter, character_cfg, updates.clone(), modules.clone());
    let hotfixes_db_loader = DatabaseLoader::new(DBFlagHotfix, hotfix_cfg, updates.clone(), modules.clone());

    LoginDatabase::set(login_db_loader.load(ctx.clone()).await?);
    WorldDatabase::set(world_db_loader.load(ctx.clone()).await?);
    CharacterDatabase::set(chars_db_loader.load(ctx.clone()).await?);
    HotfixDatabase::set(hotfixes_db_loader.load(ctx.clone()).await?);

    info!(target:"dbimport", "Started database connection pool.");
    db_started_send.send(()).unwrap();

    // Wait for cancellation
    ctx.cancelled().await;
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
