use std::path::Path;

use azothacore_common::{
    az_error,
    banner,
    bevy_app::{bevy_app, AzStartupFailedEvent, TokioRuntime},
    configuration::{config_mgr_plugin, ConfigMgr, ConfigMgrSet, DatabaseType},
    log::{logging_plugin, LoggingSetupSet},
    AzResult,
    AZOTHA_CORE_CONFIG,
    CONF_DIR,
    GIT_HASH,
    GIT_VERSION,
};
use azothacore_database::{
    args,
    database_env::{CharacterDatabase, HotfixDatabase, LoginDatabase, WorldDatabase},
    database_loader::DatabaseLoader,
    database_loader_utils::DatabaseLoaderError,
    query_with,
};
use azothacore_modules::SCRIPTS as MODULES_LIST;
use azothacore_server::{
    game::{
        scripting::script_mgr::ScriptMgr,
        scripts,
        world::{world_plugin, CurrentRealm, WorldConfig, WorldDbVersion, WorldSets},
    },
    shared::{
        realms::{
            realm_list::{realm_list_plugin, RealmList, RealmListStartSet},
            RealmFlags,
        },
        shared_defines::{set_server_process, ServerProcessType},
        tokio_signal_handling_bevy_plugin,
        SignalReceiver,
    },
};
use bevy::{
    app::AppExit,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{
        App,
        Commands,
        EventReader,
        EventWriter,
        In,
        IntoSystem,
        IntoSystemConfigs,
        IntoSystemSetConfigs,
        PostUpdate,
        PreStartup,
        Res,
        ResMut,
        Startup,
        SystemSet,
    },
};
use clap::Parser;
use flagset::FlagSet;
use tracing::{error, info, info_span};

fn main() {
    let vm = ConsoleArgs::parse();
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    let mut app = bevy_app();
    app.insert_resource(TokioRuntime(rt))
        .add_plugins((
            FrameTimeDiagnosticsPlugin,
            tokio_signal_handling_bevy_plugin,
            config_mgr_plugin::<WorldConfig, _>(vm.config, vm.dry_run),
            logging_plugin::<WorldConfig>,
            // Get the list of realms for the server
            realm_list_plugin::<WorldConfig>,
            scripts_plugin,
            world_plugin,
            // socket_mgr_plugin::<WorldConfig, SessionInner>,
            // bnet_session_handling_plugin,
            // // TODO: Impl me? Init Secret Manager
            // sSecretMgr->Initialize();
        ))
        .add_systems(PreStartup, (show_banner.in_set(WorldserverMainSets::ShowBanner),))
        .add_systems(
            Startup,
            (
                (|mut commands: Commands| set_server_process(&mut commands, ServerProcessType::Worldserver)).in_set(WorldserverMainSets::SetProcessType),
                start_db.pipe(handle_startup_errors).in_set(WorldserverMainSets::StartDB),
                set_server_unconnectable
                    .pipe(handle_startup_errors)
                    .in_set(WorldserverMainSets::SetRealmNotConnectable),
                load_realm_info.pipe(handle_startup_errors).in_set(WorldserverMainSets::LoadCurrentRealm),
            ),
        )
        // Init logging right after config management
        .configure_sets(
            PreStartup,
            (ConfigMgrSet::<WorldConfig>::load_initial(), LoggingSetupSet, WorldserverMainSets::ShowBanner).chain(),
        )
        .configure_sets(
            Startup,
            ((
                WorldserverMainSets::StartDB,
                WorldserverMainSets::SetRealmNotConnectable,
                RealmListStartSet,
                WorldserverMainSets::LoadCurrentRealm,
                WorldSets::SetInitialWorldSettings,
            )
                .chain(),),
        )
        .add_systems(PostUpdate, stop_db)
        .run();

    // // // TODO: Implement metrics?
    // // sMetric->Initialize(realm.Name, *ioContext, []()
    // // {
    // //     METRIC_VALUE("online_players", sWorld->GetPlayerCount());
    // //     METRIC_VALUE("db_queue_login", uint64(LoginDatabase.QueueSize()));
    // //     METRIC_VALUE("db_queue_character", uint64(CharacterDatabase.QueueSize()));
    // //     METRIC_VALUE("db_queue_world", uint64(WorldDatabase.QueueSize()));
    // // });
    // // METRIC_EVENT("events", "Worldserver started", "");
    // // std::shared_ptr<void> sMetricHandle(nullptr, [](void*)
    // // {
    // //     METRIC_EVENT("events", "Worldserver shutdown", "");
    // //     sMetric->Unload();
    // // });

    // // TODO: hirogoro@29/03/2023: implement secrets mgr?
    // // //- Initialize the World
    // // sSecretMgr->Initialize();

    // // // TODO: hirogoro@29/03/2023: Implement set initial world settings
    // // // sWorld->SetInitialWorldSettings();
    // // SWorld::get().write().await.set_initial_world_settings().await?;

    // // Wait for shutdown by seeing if the tasks above have all run its course.
    // // Tasks must perform graceful shutdown by using the Context above otherwise this
    // // program will not terminate properly.
    // root_ctx.tt.close();
    // rt.block_on(root_ctx.tt.wait());
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldserverMainSets {
    ShowBanner,
    SetProcessType,
    StartDB,
    LoadScript,
    SetRealmNotConnectable,
    LoadCurrentRealm,
}

fn show_banner(cfg: Res<ConfigMgr<WorldConfig>>) {
    banner::azotha_banner_show("worldserver-daemon", || {
        info!(
            target:"server::worldserver",
            "> Using configuration file       {}",
            cfg.filename.display()
        )
    });
}

fn scripts_plugin(app: &mut App) {
    info!(target:"server::loading", "Initializing Scripts...");
    // Adding scripts first, then they can load modules
    let mut script_mgr = ScriptMgr::default();

    scripts::add_scripts(app.world_mut(), &mut script_mgr);
    azothacore_modules::add_scripts(app.world_mut(), &mut script_mgr);
    app.insert_resource(script_mgr);
}

/// Initialize connection to the database
fn start_db(
    mut commands: Commands,
    mut cfg: ResMut<ConfigMgr<WorldConfig>>,
    rt: Res<TokioRuntime>,
    script_mgr: Res<ScriptMgr>,
    mut signal: ResMut<SignalReceiver>,
) -> AzResult<()> {
    let top_span = info_span!(target:"server::worldserver", "start_db");
    let _top_span_guard = top_span.enter();
    let modules: Vec<_> = MODULES_LIST.iter().map(|s| s.to_string()).collect();
    let updates = cfg.Updates.clone();
    let span = info_span!(parent:&top_span, "login_db", db=?cfg.LoginDatabaseInfo);
    let span_guard = span.enter();
    let auth_db = rt
        .block_on(async {
            tokio::select! {
                d = DatabaseLoader::new(DatabaseType::Character, cfg.LoginDatabaseInfo.clone(), updates.clone(), modules.clone()).load() => d,
                _ = signal.0.recv() => {
                    Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
                }
            }
        })
        .map(LoginDatabase)?;
    drop(span_guard);

    let span = info_span!(parent:&top_span, "world_db", db=?cfg.WorldDatabaseInfo);
    let span_guard = span.enter();
    let world_db = rt
        .block_on(async {
            tokio::select! {
                d = DatabaseLoader::new(DatabaseType::World, cfg.WorldDatabaseInfo.clone(), updates.clone(), modules.clone()).load() => d,
                _ = signal.0.recv() => {
                    Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
                }
            }
        })
        .map(WorldDatabase)?;
    drop(span_guard);

    let span = info_span!(parent:&top_span, "characters_db", db=?cfg.CharacterDatabaseInfo);
    let span_guard = span.enter();
    let characters_db = rt
        .block_on(async {
            tokio::select! {
                d = DatabaseLoader::new(DatabaseType::Character, cfg.CharacterDatabaseInfo.clone(), updates.clone(), modules.clone()).load() => d,
                _ = signal.0.recv() => {
                    Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
                }
            }
        })
        .map(CharacterDatabase)?;
    drop(span_guard);

    let span = info_span!(parent:&top_span, "hotfix_db", db=?cfg.HotfixDatabaseInfo);
    let span_guard = span.enter();
    let hotfix_db = rt
        .block_on(async {
            tokio::select! {
                d = DatabaseLoader::new(DatabaseType::Hotfix, cfg.HotfixDatabaseInfo.clone(), updates.clone(), modules.clone()).load() => d,
                _ = signal.0.recv() => {
                    Err(DatabaseLoaderError::Generic { msg: "signal termination detected!".to_string() })
                }
            }
        })
        .map(HotfixDatabase)?;
    drop(span_guard);

    //- Get the realm Id from the configuration file
    if cfg.RealmID > 255 {
        /*
         * Due to the client only being able to read a realm.Id.Realm
         * with a size of uint8 we can "only" store up to 255 realms
         * anything further the client will behave anormaly
         */
        return Err(az_error!("Realm ID must range from 1 to 255, got {}", cfg.RealmID));
    }
    info!("Loading World Information...");
    info!("> RealmID:              {}", cfg.RealmID);

    //- Clean the database before starting
    rt.block_on(clear_online_accounts(&auth_db, &characters_db, cfg.RealmID))
        .map_err(|e| az_error!("error clearing online accounts: {e}"))?;

    // Insert version info into DB
    rt.block_on(async {
        query_with("UPDATE version SET core_version = ?, core_revision = ?", args!(GIT_VERSION, GIT_HASH)?)
            .execute(&*world_db)
            .await
    })
    .map_err(|e| az_error!("error inserting current version info: {e}"))?;

    let world_db_version = match rt.block_on(WorldDbVersion::load(&world_db))? {
        Some(c) => c,
        None => WorldDbVersion {
            db_version:      "Unknown world database".to_string(),
            cache_id:        0,
            hotfix_cache_id: 0,
        },
    };
    if cfg.ClientCacheVersion == 0 {
        cfg.ClientCacheVersion = world_db_version.cache_id
    }
    if cfg.HotfixCacheVersion == 0 {
        cfg.HotfixCacheVersion = world_db_version.hotfix_cache_id
    }
    info!("> Version DB world:     {}", world_db_version.db_version);
    commands.insert_resource(world_db_version);
    script_mgr.on_after_databases_loaded(&mut commands, updates.EnableDatabases);

    // Register DBs as resources
    commands.insert_resource(auth_db);
    commands.insert_resource(world_db);
    commands.insert_resource(characters_db);
    commands.insert_resource(hotfix_db);
    Ok(())
}

fn set_server_unconnectable(rt: Res<TokioRuntime>, login_db: Res<LoginDatabase>, cfg: Res<ConfigMgr<WorldConfig>>) -> AzResult<()> {
    info!("setting worldserver as unconnectable");
    // set server not not connectable
    rt.block_on(async {
        query_with(
            "UPDATE realmlist SET flag = (flag & ~?) | ? WHERE id = ?",
            args!(
                FlagSet::from(RealmFlags::Offline).bits(),
                FlagSet::from(RealmFlags::VersionMismatch).bits(),
                cfg.RealmID
            )?,
        )
        .execute(&**login_db)
        .await
    })?;
    Ok(())
}

fn stop_db(
    rt: Res<TokioRuntime>,
    login_db: Option<Res<LoginDatabase>>,
    world_db: Option<Res<WorldDatabase>>,
    characters_db: Option<Res<CharacterDatabase>>,
    hotfix_db: Option<Res<HotfixDatabase>>,
    mut app_exit_events: EventReader<AppExit>,
) {
    let mut stopped = false;
    for _ev in app_exit_events.read() {
        if stopped {
            continue;
        }
        // Deliberately read through all events. the login DB should be closed already
        stopped = true;
        if let Some(db) = &login_db {
            info!("Stopping auth database connection.");
            rt.block_on(db.close());
        }
        if let Some(db) = &world_db {
            info!("Stopping world database connection.");
            rt.block_on(db.close());
        }
        if let Some(db) = &characters_db {
            info!("Stopping characters database connection.");
            rt.block_on(db.close());
        }
        if let Some(db) = &hotfix_db {
            info!("Stopping hotfix database connection.");
            rt.block_on(db.close());
        }
    }
}

fn handle_startup_errors(In(result): In<AzResult<()>>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>) {
    if let Err(e) = result {
        error!(cause=%e, "Startup err");
        ev_startup_failed.send_default();
    }
}

/// Clear 'online' status for all accounts with characters in this realm
async fn clear_online_accounts(login_db: &LoginDatabase, char_db: &CharacterDatabase, realm_id: u32) -> AzResult<()> {
    // Reset online status for all accounts with characters on the current realm
    query_with(
        "UPDATE account SET online = 0 WHERE online > 0 AND id IN (SELECT acctid FROM realmcharacters WHERE realmid = ?)",
        args!(realm_id)?,
    )
    .execute(&**login_db)
    .await?;

    // Reset online status for all characters
    query_with("UPDATE characters SET online = ? WHERE online <> ?", args!(false, false)?)
        .execute(&**char_db)
        .await?;

    // Battleground instance ids reset at server restart
    query_with("UPDATE character_battleground_data SET instanceId = ?", args!(false)?)
        .execute(&**char_db)
        .await?;
    Ok(())
}

fn load_realm_info(mut commands: Commands, cfg: Res<ConfigMgr<WorldConfig>>, realm_list: Res<RealmList>) -> AzResult<()> {
    let current = realm_list
        .realms
        .iter()
        .find(|(r, _)| r.realm == cfg.RealmID)
        .map(|(_, r)| r.to_owned())
        .ok_or(az_error!("Unable to find realm with ID: {}", cfg.RealmID))?;

    commands.insert_resource(CurrentRealm(current));
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ConsoleArgs {
    /// Dry run
    #[arg(short, long = "dry-run")]
    dry_run: bool,
    /// use <arg> as configuration file
    #[arg(short, long, default_value_t = Path::new(CONF_DIR).join(AZOTHA_CORE_CONFIG).to_str().unwrap().to_string())]
    config:  String,
    #[arg(short, long, default_value_t = String::new())]
    service: String,
}
