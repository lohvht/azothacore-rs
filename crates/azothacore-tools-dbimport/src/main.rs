use std::path::Path;

use azothacore_common::{
    banner,
    configuration::{
        ConfigMgr,
        DatabaseType::{Character as DBFlagCharacter, Hotfix as DBFlagHotfix, Login as DBFlagLogin, World as DBFlagWorld},
        DbUpdates,
    },
    log::init_logging,
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
use tracing::info;

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
        init_logging(
            cfg_mgr_r.get_option::<String>("LogsDir")?,
            &cfg_mgr_r.get_option::<Vec<_>>("Appender")?,
            &cfg_mgr_r.get_option::<Vec<_>>("Logger")?,
        )
    };
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    banner::azotha_banner_show("dbimport", || {
        info!(
            target:"dbimport",
            "> Using configuration file       {}",
            ConfigMgr::r().get_filename().display()
        )
    });

    rt.block_on(start_db())?;
    let _db_handle = dropper_wrapper_fn(rt.handle(), stop_db);

    info!(target:"dbimport", "Halting process...");

    Ok(())
}

/// Initialize connection to the database
async fn start_db() -> AzResult<()> {
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
    let login_db_loader = DatabaseLoader::new(DBFlagLogin, &auth_cfg, &updates, MODULES_LIST);
    let world_db_loader = DatabaseLoader::new(DBFlagWorld, &world_cfg, &updates, MODULES_LIST);
    let chars_db_loader = DatabaseLoader::new(DBFlagCharacter, &character_cfg, &updates, MODULES_LIST);
    let hotfixes_db_loader = DatabaseLoader::new(DBFlagHotfix, &hotfix_cfg, &updates, MODULES_LIST);

    LoginDatabase::set(login_db_loader.load().await?);
    WorldDatabase::set(world_db_loader.load().await?);
    CharacterDatabase::set(chars_db_loader.load().await?);
    HotfixDatabase::set(hotfixes_db_loader.load().await?);

    info!(target:"dbimport", "Started database connection pool.");
    Ok(())
}

async fn stop_db() -> AzResult<()> {
    info!(target:"dbimport", "Stopping database connection pool.");
    LoginDatabase::get().close().await;
    WorldDatabase::get().close().await;
    CharacterDatabase::get().close().await;
    HotfixDatabase::get().close().await;
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
