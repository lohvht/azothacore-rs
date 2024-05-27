use std::{path::Path, sync::Arc, time::Duration};

use authserver::{config::AuthserverConfig, rest::LoginRESTService, ssl_context::SslContext, BnetSessionManager};
use azothacore_common::{
    banner,
    configuration::{DatabaseInfo, DatabaseType, DbUpdates, CONFIG_MGR},
    log,
    r#async::Context,
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
    panic_handler,
    realms::realm_list::RealmList,
    shared_defines::{ServerProcessType, ThisServerProcess},
    signal_handler,
};
use clap::Parser;
use rand::{rngs::OsRng, Rng};
use tokio::sync::oneshot;
use tracing::{error, info};

/// Launch the auth server
fn main() -> AzResult<()> {
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build()?);
    let root_ctx = Context::new(rt.handle());
    panic_handler(root_ctx.clone());
    ThisServerProcess::set(ServerProcessType::Authserver);
    let vm = ConsoleArgs::parse();
    let cfg: AuthserverConfig = {
        let mut cfg_mgr_w = CONFIG_MGR.blocking_write();
        cfg_mgr_w.configure(&vm.config, vm.dry_run, Box::new(|_| Box::pin(async move { Ok(vec![]) })));
        cfg_mgr_w.load_app_configs()?
    };
    let _wg = {
        // TODO: Setup DB logging. Original code below
        // // Init all logs
        // sLog->RegisterAppender<AppenderDB>();
        log::init(&cfg.LogsDir, &cfg.Appender, &cfg.Logger)
    };

    banner::azotha_banner_show("authserver-daemon", || {
        info!(
            target:"server::authserver",
            "> Using configuration file       {}",
            CONFIG_MGR.blocking_read().get_filename().display()
        )
    });

    // Seed the OsRng here.
    // That way it won't auto-seed when calling OsRng and slow down the first world login
    OsRng.gen::<u64>();

    // worldserver PID file creation
    if let Some(pid_file) = cfg.PidFile {
        let pid = create_pid_file(pid_file)?;
        error!(target:"server", "Daemon PID: {pid}");
    }

    SslContext::initialise(cfg.CertificatesFile, cfg.PrivateKeyFile)?;
    // // TODO: Impl me? Init Secret Manager
    // sSecretMgr->Initialize();

    let (db_started_send, db_started_recv) = oneshot::channel();
    let ctx = root_ctx.clone();
    root_ctx.spawn(async move {
        if let Err(e) = start_db(ctx.clone(), cfg.Updates, cfg.LoginDatabaseInfo, db_started_send).await {
            error!(target:"server::authserver", cause=%e, "error starting/stopping DB");
            ctx.cancel();
        }
    });
    // Enforce DB to be up first
    db_started_recv.blocking_recv().unwrap();

    let ctx = root_ctx.clone();
    root_ctx.spawn(async move {
        if let Err(e) = LoginRESTService::start(ctx.clone(), cfg.BindIP, cfg.LoginREST, cfg.WrongPass).await {
            error!(target:"server::authserver", cause=%e, "error starting/stopping LoginRESTService");
            ctx.cancel();
        }
    });

    // Get the list of realms for the server
    let ctx = root_ctx.clone();
    root_ctx.spawn(RealmList::init(ctx, *cfg.RealmsStateUpdateDelay));

    // Stop auth server if dry run
    if CONFIG_MGR.blocking_read().is_dry_run() {
        info!(target:"server::authserver", "Dry run completed, terminating.");
        root_ctx.cancel();
        root_ctx.tt.close();
        rt.block_on(root_ctx.tt.wait());
        return Ok(());
    }

    let ctx = root_ctx.clone();
    root_ctx.spawn(BnetSessionManager::start_network(ctx, (cfg.BindIP, cfg.BattlenetPort)));

    // Set signal handlers
    let ctx = root_ctx.clone();
    root_ctx.spawn(signal_handler());

    let ctx = root_ctx.clone();
    root_ctx.spawn(ban_expiry_task(ctx, *cfg.BanExpiryCheckInterval));

    root_ctx.tt.close();
    rt.block_on(root_ctx.tt.wait());
    info!(target:"server::authserver", "Halting process...");

    Ok(())
}

async fn ban_expiry_task(ctx: Context, ban_expiry_check_interval: Duration) {
    let mut interval = tokio::time::interval(ban_expiry_check_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = ctx.cancelled() => {
                break;
            }
            i = interval.tick() => i,
        };
        let login_db = &LoginDatabase::get();
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
    info!(target:"server::authserver", "Closed ban expiry handler");
}

/// Initialize connection to the database
async fn start_db(ctx: Context, updates: DbUpdates, auth_cfg: DatabaseInfo, db_started_send: oneshot::Sender<()>) -> AzResult<()> {
    let login_db_loader = DatabaseLoader::new(DatabaseType::Login, auth_cfg, updates, vec![]);
    let auth_db = login_db_loader.load(ctx.clone()).await?;
    LoginDatabase::set(auth_db);
    info!("Started auth database connection pool.");
    db_started_send.send(()).unwrap();

    // Wait for cancellation
    ctx.cancelled().await;
    info!("Stopping auth database connection.");
    LoginDatabase::close().await;

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
