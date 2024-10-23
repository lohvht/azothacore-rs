use std::path::Path;

use authserver::{
    config::AuthserverConfig,
    rest::{login_rest_service_plugin, LoginRESTServiceSystemSets},
    session::{bnet_session_handling_plugin, SessionInner},
    ssl_context::{ssl_context_plugin, SetSslContextSet},
};
use azothacore_common::{
    banner,
    bevy_app::{az_startup_succeeded, bevy_app, AzStartupFailedEvent, TokioRuntime},
    configuration::{config_mgr_plugin, ConfigMgr, ConfigMgrSet, DatabaseType},
    log::{logging_plugin, LoggingSetupSet},
    AzResult,
    AZOTHA_REALM_CONFIG,
    CONF_DIR,
};
use azothacore_database::{
    database_env::{LoginDatabase, LoginPreparedStmts},
    database_loader::DatabaseLoader,
    params,
};
use azothacore_server::shared::{
    networking::socket_mgr::socket_mgr_plugin,
    realms::realm_list::realm_list_plugin,
    shared_defines::{set_server_process, ServerProcessType},
    tokio_signal_handling_bevy_plugin,
};
use bevy::{
    app::AppExit,
    prelude::{
        Commands,
        EventReader,
        EventWriter,
        FixedUpdate,
        IntoSystemConfigs,
        IntoSystemSetConfigs,
        PostUpdate,
        PreStartup,
        Real,
        Res,
        ResMut,
        Resource,
        Startup,
        SystemSet,
        Time,
        Timer,
        TimerMode,
    },
};
use clap::Parser;
use tracing::{error, info};

/// Launch the auth server
fn main() -> AzResult<()> {
    let vm = ConsoleArgs::parse();
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    let mut app = bevy_app();
    app.insert_resource(TokioRuntime(rt))
        .add_plugins((
            tokio_signal_handling_bevy_plugin,
            config_mgr_plugin::<AuthserverConfig, _>(vm.config, vm.dry_run),
            logging_plugin::<AuthserverConfig>,
            ssl_context_plugin::<AuthserverConfig>,
            login_rest_service_plugin,
            // Get the list of realms for the server
            realm_list_plugin::<AuthserverConfig>,
            socket_mgr_plugin::<AuthserverConfig, SessionInner>,
            bnet_session_handling_plugin,
            // // TODO: Impl me? Init Secret Manager
            // sSecretMgr->Initialize();
        ))
        .add_systems(
            Startup,
            (
                (|mut commands: Commands| set_server_process(&mut commands, ServerProcessType::Authserver)).in_set(AuthserverSet::SetProcessType),
                show_banner.in_set(AuthserverSet::ShowBanner),
                start_db.in_set(AuthserverSet::StartDB),
                insert_ban_expiry_timer.in_set(AuthserverSet::InsertBanExpiryTimer),
            ),
        )
        .add_systems(FixedUpdate, ban_expiry_task.run_if(az_startup_succeeded()).in_set(AuthserverSet::BanExpiryTask))
        // Init logging right after config management
        .configure_sets(PreStartup, ConfigMgrSet::<AuthserverConfig>::load_initial().before(LoggingSetupSet))
        .configure_sets(Startup, ((SetSslContextSet, AuthserverSet::StartDB).before(LoginRESTServiceSystemSets::Start),))
        .add_systems(PostUpdate, stop_db)
        .run();

    Ok(())
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuthserverSet {
    ShowBanner,
    SetProcessType,
    InsertBanExpiryTimer,
    BanExpiryTask,
    StartDB,
}

#[derive(Resource)]
struct BanExpiryTimer(Timer);

fn insert_ban_expiry_timer(mut commands: Commands, cfg: Res<ConfigMgr<AuthserverConfig>>) {
    commands.insert_resource(BanExpiryTimer(Timer::new(*cfg.BanExpiryCheckInterval, TimerMode::Repeating)));
}

fn ban_expiry_task(mut timer: ResMut<BanExpiryTimer>, time: Res<Time<Real>>, login_db: Res<LoginDatabase>, rt: Res<TokioRuntime>) {
    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }
    rt.block_on(async {
        if let Err(e) = LoginDatabase::del_expired_ip_bans(&**login_db, params!()).await {
            error!(target:"bnetserver", cause=%e, "del_expired_ip_bans err");
        };
        if let Err(e) = LoginDatabase::upd_expired_account_bans(&**login_db, params!()).await {
            error!(target:"bnetserver", cause=%e, "upd_expired_account_bans err");
        };
        if let Err(e) = LoginDatabase::del_bnet_expired_account_banned(&**login_db, params!()).await {
            error!(target:"bnetserver", cause=%e, "del_bnet_expired_account_banned err");
        };
    });
}

fn show_banner(cfg: Res<ConfigMgr<AuthserverConfig>>) {
    banner::azotha_banner_show("authserver-daemon", || {
        info!(
            target:"server::authserver",
            "> Using configuration file       {}",
            cfg.filename.display()
        )
    });
}

/// Initialize connection to the database
fn start_db(mut commands: Commands, cfg: Res<ConfigMgr<AuthserverConfig>>, rt: Res<TokioRuntime>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>) {
    let login_db_loader = DatabaseLoader::new(DatabaseType::Login, cfg.LoginDatabaseInfo.clone(), cfg.Updates.clone(), vec![]);
    let auth_db = match rt.block_on(login_db_loader.load()) {
        Err(e) => {
            error!(target:"server::authserver", cause=%e, "error starting/stopping DB");
            ev_startup_failed.send_default();
            return;
        },
        Ok(d) => d,
    };
    commands.insert_resource(LoginDatabase(auth_db));
}

fn stop_db(rt: Res<TokioRuntime>, login_db: Option<Res<LoginDatabase>>, mut app_exit_events: EventReader<AppExit>) {
    let mut stopped = false;
    for _ev in app_exit_events.read() {
        if stopped {
            continue;
        }
        // Deliberately read through all events. the login DB should be closed already
        stopped = true;
        if let Some(login_db) = &login_db {
            info!("Stopping auth database connection.");
            rt.block_on(login_db.close());
        }
    }
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ConsoleArgs {
    /// Dry run
    #[arg(short, long = "dry-run")]
    dry_run: bool,
    /// use <arg> as configuration file
    #[arg(short, long, default_value_t = Path::new(CONF_DIR).join(AZOTHA_REALM_CONFIG).to_str().unwrap().to_string())]
    config:  String,
    #[arg(short, long, default_value_t = String::new())]
    service: String,
}
