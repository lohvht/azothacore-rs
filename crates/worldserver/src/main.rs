use std::{path::Path, sync::Arc};

use azothacore_common::{
    az_error,
    banner,
    configuration::{
        ConfigMgr,
        DatabaseType::{Character as DBFlagCharacter, Hotfix as DBFlagHotfix, Login as DBFlagLogin, World as DBFlagWorld},
        DbUpdates,
    },
    get_g,
    log::init_logging,
    mut_g,
    utils::create_pid_file,
    AzResult,
    AZOTHA_CORE_CONFIG,
    CONF_DIR,
    GIT_HASH,
    GIT_VERSION,
};
use azothacore_database::{
    database_env::{CharacterDatabase, HotfixDatabase, LoginDatabase, WorldDatabase},
    database_loader::DatabaseLoader,
    params,
    query_with,
};
use azothacore_modules::SCRIPTS as MODULES_LIST;
use azothacore_server::{
    game::{
        scripting::script_mgr::SCRIPT_MGR,
        scripts,
        world::{WorldTrait, S_WORLD},
    },
    receive_signal_and_run_expr,
    shared::{
        dropper_wrapper_fn,
        realms::{realm_list::RealmList, RealmFlags},
        shared_defines::{ServerProcessType, ThisServerProcess},
    },
    short_curcuit_unix_signal_unwrap,
};
use clap::Parser;
use flagset::FlagSet;
use rand::{rngs::OsRng, Rng};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, info_span, instrument, Instrument};

#[cfg(target_os = "windows")]
fn signal_handler(rt: &tokio::runtime::Runtime) -> JoinHandle<Result<(), std::io::Error>> {
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
    rt.spawn(
        async move {
            use tokio::signal::unix::SignalKind;
            let mut sig_interrupt = short_curcuit_unix_signal_unwrap!(SignalKind::interrupt());
            let mut sig_terminate = short_curcuit_unix_signal_unwrap!(SignalKind::terminate());
            let mut sig_quit = short_curcuit_unix_signal_unwrap!(SignalKind::quit());
            receive_signal_and_run_expr!(
                mut_g!(S_WORLD).stop_now(1),
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
    ThisServerProcess::set(ServerProcessType::Worldserver);
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

    let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build()?);
    let token = tokio_util::sync::CancellationToken::new();
    banner::azotha_banner_show("worldserver-daemon", || {
        info!(
            target:"server::worldserver",
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
        error!(target:"server", "Daemon PID: {}", pid);
    }
    let rt = runtime.clone();
    let ct = token.clone();
    let signal_handler = signal_handler(&rt, ct);

    // // TODO: Follow thread pool based model? from the original core code
    // // Start the Boost based thread pool
    // int numThreads = sConfigMgr->GetOption<int32>("ThreadPool", 1);
    // std::shared_ptr<std::vector<std::thread>> threadPool(new std::vector<std::thread>(), [ioContext](std::vector<std::thread>* del)
    // {
    //     ioContext->stop();
    //     for (std::thread& thr : *del)
    //         thr.join();
    //     delete del;
    // });
    // if (numThreads < 1)
    // {
    //     numThreads = 1;
    // }
    // for (int i = 0; i < numThreads; ++i)
    // {
    //     threadPool->push_back(std::thread([ioContext]()
    //     {
    //         ioContext->run();
    //     }));
    // }

    // // TODO: Implement process priority?
    // // Set process priority according to configuration settings
    // SetProcessPriority("server.worldserver", sConfigMgr->GetOption<int32>(CONFIG_PROCESSOR_AFFINITY, 0), sConfigMgr->GetOption<bool>(CONFIG_HIGH_PRIORITY, false));

    info!(target:"server::loading", "Initializing Scripts...");
    // Loading modules configs before scripts
    ConfigMgr::m().load_modules_configs(false, true, |reload| get_g!(SCRIPT_MGR).on_load_module_config(reload))?;
    let _s_script_mgr_handle = dropper_wrapper_fn(runtime.handle(), token.clone(), async { mut_g!(SCRIPT_MGR).unload() });
    scripts::add_scripts()?;
    azothacore_modules::add_scripts()?;

    let realm_id = ConfigMgr::r().get_option::<u32>("RealmID")?;
    runtime.block_on(start_db(token.clone(), realm_id))?;
    let _db_handle = dropper_wrapper_fn(runtime.handle(), token.clone(), stop_db());

    // set server offline (not connectable)

    runtime.block_on(async {
        query_with(
            "UPDATE realmlist SET flag = (flag & ~?) | ? WHERE id = ?",
            params!(
                FlagSet::from(RealmFlags::Offline).bits(),
                FlagSet::from(RealmFlags::VersionMismatch).bits(),
                realm_id
            ),
        )
        .execute(LoginDatabase::get())
        .await
    })?;

    RealmList::init(
        runtime.handle(),
        token.clone(),
        ConfigMgr::r().get_option("RealmsStateUpdateDelay").unwrap_or(10),
    );

    // // TODO: Implement metrics?
    // sMetric->Initialize(realm.Name, *ioContext, []()
    // {
    //     METRIC_VALUE("online_players", sWorld->GetPlayerCount());
    //     METRIC_VALUE("db_queue_login", uint64(LoginDatabase.QueueSize()));
    //     METRIC_VALUE("db_queue_character", uint64(CharacterDatabase.QueueSize()));
    //     METRIC_VALUE("db_queue_world", uint64(WorldDatabase.QueueSize()));
    // });
    // METRIC_EVENT("events", "Worldserver started", "");
    // std::shared_ptr<void> sMetricHandle(nullptr, [](void*)
    // {
    //     METRIC_EVENT("events", "Worldserver shutdown", "");
    //     sMetric->Unload();
    // });

    // TODO: hirogoro@29/03/2023: implement secrets mgr?
    // //- Initialize the World
    // sSecretMgr->Initialize();

    // // // TODO: hirogoro@29/03/2023: Implement set initial world settings
    // // sWorld->SetInitialWorldSettings();
    // S_WORLD.write().await.set_initial_world_settings().await?;

    // Begin shutdown, waiting for signal handler first. Then unload everything else.
    rt.block_on(async { signal_handler.await? })?;

    info!("TERMINATING!");
    Ok(())
}

async fn start_db(cancel_token: CancellationToken, realm_id: u32) -> AzResult<()> {
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

    //- Get the realm Id from the configuration file
    if realm_id > 255 {
        /*
         * Due to the client only being able to read a realm.Id.Realm
         * with a size of uint8 we can "only" store up to 255 realms
         * anything further the client will behave anormaly
         */
        error!("Realm ID must range from 1 to 255");
        return Err(az_error!("Realm ID must range from 1 to 255"));
    }

    info!("Loading World Information...");
    info!("> RealmID:              {}", realm_id);

    //- Clean the database before starting
    clear_online_accounts(realm_id).await?;

    // Insert version info into DB
    query_with("UPDATE version SET core_version = ?, core_revision = ?", params!(GIT_VERSION, GIT_HASH))
        .execute(WorldDatabase::get())
        .await?;

    mut_g!(S_WORLD).load_db_version()?;

    info!("> Version DB world:     {}", get_g!(S_WORLD).get_db_version());

    get_g!(SCRIPT_MGR).on_after_databases_loaded(updates.EnableDatabases);

    Ok(())
}

async fn stop_db() -> AzResult<()> {
    LoginDatabase::close().await;
    WorldDatabase::close().await;
    CharacterDatabase::close().await;
    HotfixDatabase::close().await;

    Ok(())
}

/// Clear 'online' status for all accounts with characters in this realm
#[instrument]
async fn clear_online_accounts(realm_id: u32) -> AzResult<()> {
    // Reset online status for all accounts with characters on the current realm
    // pussywizard: tc query would set online=0 even if logged in on another realm >_>
    query_with("UPDATE account SET online = ? WHERE online = ?", params!(false, realm_id))
        .execute(LoginDatabase::get())
        .await?;
    // Reset online status for all characters
    query_with("UPDATE characters SET online = ? WHERE online <> ?", params!(false, false))
        .execute(LoginDatabase::get())
        .await?;

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
