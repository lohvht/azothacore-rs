use std::path::Path;

use azothacore_common::{
    banner,
    bevy_app::{bevy_app, TokioRuntime},
    configuration::{config_mgr_plugin, ConfigMgr, ConfigMgrSet, DatabaseType},
    log::{logging_plugin, LoggingSetupSet},
    AZOTHA_DB_IMPORT_CONFIG,
    CONF_DIR,
};
use azothacore_database::{database_loader::DatabaseLoader, database_loader_utils::DatabaseLoaderError};
use azothacore_modules::SCRIPTS as MODULES_LIST;
use azothacore_server::shared::{tokio_signal_handling_bevy_plugin, SignalBroadcaster};
use bevy::prelude::*;
use clap::Parser;
use dbimport::DbImportConfig;
use tracing::{error, info};

fn main() {
    let vm = ConsoleArgs::parse();
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    let mut app = bevy_app();
    app.insert_resource(TokioRuntime(rt))
        .add_plugins((
            tokio_signal_handling_bevy_plugin,
            config_mgr_plugin::<DbImportConfig, _>(vm.config, false),
            logging_plugin::<DbImportConfig>,
        ))
        .add_systems(
            Startup,
            (
                show_banner.run_if(resource_exists::<ConfigMgr<DbImportConfig>>).in_set(DbImportSet::ShowBanner),
                start_db.run_if(resource_exists::<ConfigMgr<DbImportConfig>>).in_set(DbImportSet::StartDB),
            ),
        )
        // Init logging right after config management
        .configure_sets(PreStartup, ConfigMgrSet::<DbImportConfig>::load_initial().before(LoggingSetupSet))
        .update();
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DbImportSet {
    ShowBanner,
    StartDB,
}

fn show_banner(cfg: Res<ConfigMgr<DbImportConfig>>) {
    banner::azotha_banner_show("dbimport", || {
        info!(
            target:"dbimport",
            "> Using configuration file       {}",
            cfg.filename.display()
        )
    });
}

/// Initialize connection to the database
fn start_db(cfg: Res<ConfigMgr<DbImportConfig>>, rt: Res<TokioRuntime>, mut signal: ResMut<SignalBroadcaster>) {
    let modules: Vec<_> = MODULES_LIST.iter().map(|s| s.to_string()).collect();
    let login_db_loader = DatabaseLoader::new(DatabaseType::Character, cfg.CharacterDatabaseInfo.clone(), cfg.Updates.clone(), modules.clone());
    let world_db_loader = DatabaseLoader::new(DatabaseType::World, cfg.WorldDatabaseInfo.clone(), cfg.Updates.clone(), modules.clone());
    let chars_db_loader = DatabaseLoader::new(DatabaseType::Character, cfg.LoginDatabaseInfo.clone(), cfg.Updates.clone(), modules.clone());
    let hotfixes_db_loader = DatabaseLoader::new(DatabaseType::Hotfix, cfg.HotfixDatabaseInfo.clone(), cfg.Updates.clone(), modules.clone());

    let span = info_span!(target:"dbimport", "login_db", db=?cfg.LoginDatabaseInfo);
    let span_guard = span.enter();
    match rt.block_on(async {
        tokio::select! {
            d = login_db_loader.load() => d,
            _ = signal.0.recv() => {
                Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
            }
        }
    }) {
        Err(e) => {
            error!(cause=%e, "error starting / updating DB");
        },
        Ok(db) => {
            info!("connected to DB successfully and updated the DB (if configured). Stopping connection pool");
            rt.block_on(db.close());
        },
    }
    drop(span_guard);
    let span = info_span!(target:"dbimport", "world_db", db=?cfg.WorldDatabaseInfo);
    let span_guard = span.enter();
    match rt.block_on(async {
        tokio::select! {
            d = world_db_loader.load() => d,
            _ = signal.0.recv() => {
                Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
            }
        }
    }) {
        Err(DatabaseLoaderError::Generic { msg }) if msg.strip_prefix("signal termination detected").is_none() => {
            error!("signal termination detected, quitting start and update DB.");
            return;
        },
        Err(e) => {
            error!(cause=%e, "error starting / updating DB");
        },
        Ok(db) => {
            info!("connected to DB successfully and updated the DB (if configured). Stopping connection pool");
            rt.block_on(db.close());
        },
    }
    drop(span_guard);
    let span = info_span!(target:"dbimport", "characters_db", db=?cfg.CharacterDatabaseInfo);
    let span_guard = span.enter();
    match rt.block_on(async {
        tokio::select! {
            d = chars_db_loader.load() => d,
            _ = signal.0.recv() => {
                Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
            }
        }
    }) {
        Err(DatabaseLoaderError::Generic { msg }) if msg.strip_prefix("signal termination detected").is_some() => {
            error!("signal termination detected, quitting start and update DB.");
            return;
        },
        Err(e) => {
            error!(cause=%e, "error starting / updating DB");
        },
        Ok(db) => {
            info!("connected to DB successfully and updated the DB (if configured). Stopping connection pool");
            rt.block_on(db.close());
        },
    }
    drop(span_guard);
    let span = info_span!(target:"dbimport", "characters_db", db=?cfg.HotfixDatabaseInfo);
    let span_guard = span.enter();
    match rt.block_on(async {
        tokio::select! {
            d = hotfixes_db_loader.load() => d,
            _ = signal.0.recv() => {
                Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
            }
        }
    }) {
        Err(DatabaseLoaderError::Generic { msg }) if msg.strip_prefix("signal termination detected").is_none() => {
            error!("signal termination detected, quitting start and update DB.");
            return;
        },
        Err(e) => {
            error!(cause=%e, "error starting / updating DB");
        },
        Ok(db) => {
            info!("connected to DB successfully and updated the DB (if configured). Stopping connection pool");
            rt.block_on(db.close());
        },
    }
    drop(span_guard);
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ConsoleArgs {
    /// use <arg> as configuration file
    #[arg(short, long, default_value_t = Path::new(CONF_DIR).join(AZOTHA_DB_IMPORT_CONFIG).to_str().unwrap().to_string())]
    config:  String,
    #[arg(short, long, default_value_t = String::new())]
    service: String,
}
