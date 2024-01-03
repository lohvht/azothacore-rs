use std::{path::Path, time::Duration};

use authserver::{rest::LoginRESTService, ssl_context::SslContext, BnetSessionManager};
use azothacore_common::{
    banner,
    configuration::{ConfigMgr, DatabaseType, DbUpdates},
    log::init_logging,
    utils::create_pid_file,
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
    dropper_wrapper_fn,
    realms::realm_list::RealmList,
    shared_defines::{ServerProcessType, ThisServerProcess},
};
use clap::Parser;
use rand::{rngs::OsRng, Rng};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

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
fn signal_handler(rt: &tokio::runtime::Runtime, cancel_token: CancellationToken) -> JoinHandle<Result<(), std::io::Error>> {
    use azothacore_server::{receive_signal_and_run_expr, short_curcuit_unix_signal_unwrap};
    use tokio::signal::unix::SignalKind;
    use tracing::{info_span, Instrument};

    fn cancel_func(cancel_token: CancellationToken) -> AzResult<()> {
        cancel_token.cancel();
        Ok(())
    }

    rt.spawn(
        async {
            let mut sig_interrupt = short_curcuit_unix_signal_unwrap!(SignalKind::interrupt());
            let mut sig_terminate = short_curcuit_unix_signal_unwrap!(SignalKind::terminate());
            let mut sig_quit = short_curcuit_unix_signal_unwrap!(SignalKind::quit());
            receive_signal_and_run_expr!(
                cancel_func(cancel_token),
                "SIGINT" => sig_interrupt
                "SIGTERM" => sig_terminate
                "SIGQUIT" => sig_quit
            );
            Ok(())
        }
        .instrument(info_span!("signal_handler")),
    )
}

/// Launch the auth server
fn main() -> AzResult<()> {
    ThisServerProcess::set(ServerProcessType::Authserver);
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
    let token = tokio_util::sync::CancellationToken::new();
    banner::azotha_banner_show("authserver-daemon", || {
        info!(
            target:"server::authserver",
            "> Using configuration file       {}",
            ConfigMgr::r().get_filename().display()
        )
    });
    // Seed the OsRng here.
    // That way it won't auto-seed when calling OsRng and slow down the first world login
    OsRng.gen::<u64>();

    // worldserver PID file creation
    if let Ok(pid_file) = &ConfigMgr::r().get_option::<String>("PidFile") {
        let pid = create_pid_file(pid_file)?;
        error!(target:"server", "Daemon PID: {pid}");
    }

    SslContext::initialise()?;

    rt.block_on(start_db())?;

    // // TODO: Impl me? Init Secret Manager
    // sSecretMgr->Initialize();

    let _db_handle = dropper_wrapper_fn(rt.handle(), stop_db);

    LoginRESTService::start(rt.handle(), token.clone())?;
    let _login_service_handle = dropper_wrapper_fn(rt.handle(), LoginRESTService::stop);

    // Get the list of realms for the server
    RealmList::init(rt.handle(), token.clone(), ConfigMgr::r().get_option("RealmsStateUpdateDelay").unwrap_or(10));
    // let _realm_list_handle = dropper_wrapper_fn(rt.handle(), || async { RealmList::close().await });

    // Stop auth server if dry run
    if ConfigMgr::r().is_dry_run() {
        info!(target:"server::authserver", "Dry run completed, terminating.");
        return Ok(());
    }

    let bind_ip = ConfigMgr::r().get_option("BindIP").unwrap_or("0.0.0.0".to_string());
    let bnport = ConfigMgr::r().get_option("BattlenetPort").unwrap_or(1119u16);

    BnetSessionManager::start_network(rt.handle(), token.clone(), (bind_ip, bnport))?;
    let _session_mgr_handle = dropper_wrapper_fn(rt.handle(), || async { BnetSessionManager::stop_network().await.map_err(|e| e.into()) });

    // Set signal handlers
    let ct = token.clone();
    let _signal_handler = signal_handler(&rt, ct.clone());

    // // TODO: Implement process priority?
    // // Set process priority according to configuration settings
    // SetProcessPriority("server.bnetserver", sConfigMgr->GetIntDefault(CONFIG_PROCESSOR_AFFINITY, 0), sConfigMgr->GetBoolDefault(CONFIG_HIGH_PRIORITY, false));

    let ban_expiry_check_interval = Duration::from_secs(ConfigMgr::r().get_option("BanExpiryCheckInterval").unwrap_or(60));
    let _ban_expiry_handler = rt.spawn(ban_expiry_task(token.clone(), ban_expiry_check_interval));

    // TODO: Impl me? Windows service status watcher
    // #if TRINITY_PLATFORM == TRINITY_PLATFORM_WINDOWS
    //     std::shared_ptr<boost::asio::deadline_timer> serviceStatusWatchTimer;
    //     if (m_ServiceStatus != -1)
    //     {
    //         serviceStatusWatchTimer = std::make_shared<boost::asio::deadline_timer>(*ioContext);
    //         serviceStatusWatchTimer->expires_from_now(boost::posix_time::seconds(1));
    //         serviceStatusWatchTimer->async_wait(std::bind(&ServiceStatusWatcher,
    //             std::weak_ptr<boost::asio::deadline_timer>(serviceStatusWatchTimer),
    //             std::weak_ptr<Trinity::Asio::IoContext>(ioContext),
    //             std::placeholders::_1));
    //     }
    // #endif

    rt.block_on(async {
        token.cancelled().await;
        _ = _ban_expiry_handler.await;
        _ = _signal_handler.await;
    });
    info!(target = "server::bnetserver", "Halting process...");

    Ok(())
}

async fn ban_expiry_task(cancel_token: CancellationToken, ban_expiry_check_interval: Duration) {
    let mut interval = tokio::time::interval(ban_expiry_check_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                break;
            }
            i = interval.tick() => i,
        };
        let login_db = LoginDatabase::get();
        if let Err(e) = LoginDatabase::del_expired_ip_bans(login_db, params!()).await {
            error!(target:"bnetserver", cause=%e, "del_expired_ip_bans err");
        };
        if let Err(e) = LoginDatabase::upd_expired_account_bans(login_db, params!()).await {
            error!(target:"bnetserver", cause=%e, "upd_expired_account_bans err");
        };
        if let Err(e) = LoginDatabase::del_bnet_expired_account_banned(login_db, params!()).await {
            error!(target:"bnetserver", cause=%e, "del_bnet_expired_account_banned err");
        };
    }
}

/// Initialize connection to the database
async fn start_db() -> AzResult<()> {
    let (updates, auth_cfg) = {
        let config_mgr_r = ConfigMgr::r();
        (config_mgr_r.get_option::<DbUpdates>("Updates")?, config_mgr_r.get_option("LoginDatabaseInfo")?)
    };

    let login_db_loader = DatabaseLoader::new(DatabaseType::Login, &auth_cfg, &updates, &[]);
    let auth_db = login_db_loader.load().await?;
    LoginDatabase::set(auth_db);
    info!("Started auth database connection pool.");
    Ok(())
}

async fn stop_db() -> AzResult<()> {
    LoginDatabase::get().close().await;

    Ok(())
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
