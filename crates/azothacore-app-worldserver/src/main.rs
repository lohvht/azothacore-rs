use std::{io, net::ToSocketAddrs, path::Path};

use azothacore_common::{
    az_error,
    banner,
    configuration::{
        DatabaseTypeFlags::{Character as DBFlagCharacter, Login as DBFlagLogin, World as DBFlagWorld},
        CONFIG_MGR,
    },
    log::init_logging,
    utils::create_pid_file,
    AccountTypes,
    AzResult,
    AZOTHA_CORE_CONFIG,
    CONF_DIR,
    GIT_HASH,
    GIT_VERSION,
};
use azothacore_modules::scripts as module_scripts;
use azothacore_server::{
    database::{
        database_env::{CharacterDatabase, LoginDatabase, WorldDatabase},
        database_loader::DatabaseLoader,
        qargs,
        sql_w_args,
    },
    game::{
        scripting::script_mgr::SCRIPT_MGR,
        scripts,
        world::{WorldRealm, WorldTrait, S_WORLD},
    },
    receive_signal_and_run_expr,
    shared::{
        dropper_wrapper_fn,
        realms::{Realm, RealmFlags, RealmType},
        shared_defines::{ServerProcessType, ThisServerProcess},
    },
    short_curcuit_unix_signal_unwrap,
};
use clap::Parser;
use flagset::FlagSet;
use num_bigint::RandBigInt;
use rand::rngs::OsRng;
use tokio::task::JoinHandle;
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
fn signal_handler(rt: &tokio::runtime::Runtime) -> JoinHandle<Result<(), std::io::Error>> {
    rt.spawn(
        async {
            use tokio::signal::unix::SignalKind;
            let mut sig_interrupt = short_curcuit_unix_signal_unwrap!(SignalKind::interrupt());
            let mut sig_terminate = short_curcuit_unix_signal_unwrap!(SignalKind::terminate());
            let mut sig_quit = short_curcuit_unix_signal_unwrap!(SignalKind::quit());
            receive_signal_and_run_expr!(
                S_WORLD.write().unwrap().stop_now(1),
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
    let _wg = {
        let mut cfg_mgr_w = CONFIG_MGR.write().unwrap();
        cfg_mgr_w.configure(&vm.config, vm.dry_run);
        cfg_mgr_w.load_app_configs()?;
        // TODO: Setup DB logging. Original code below
        // // Init all logs
        // sLog->RegisterAppender<AppenderDB>();
        init_logging(&cfg_mgr_w.world().LogsDir, &cfg_mgr_w.world().Appender, &cfg_mgr_w.world().Logger)
    };

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;

    banner::azotha_banner_show("worldserver-daemon", || {
        info!(
            target = "server::worldserver",
            "> Using configuration file       {}",
            CONFIG_MGR.read().unwrap().get_filename().display()
        )
    });
    // Seed the OsRng here.
    // That way it won't auto-seed when calling OsRng and slow down the first world login
    OsRng.gen_bigint(16 * 8);

    // worldserver PID file creation
    if let Some(pid_file) = &CONFIG_MGR.read().unwrap().world().PidFile {
        let pid = create_pid_file(pid_file)?;
        error!(target = "server", "Daemon PID: {}", pid);
    }
    let signal_handler = signal_handler(&rt);

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

    info!(target = "server::loading", "Initializing Scripts...");
    // Loading modules configs before scripts
    CONFIG_MGR
        .write()
        .unwrap()
        .load_modules_configs(false, true, |reload| SCRIPT_MGR.read().unwrap().on_load_module_config(reload))?;
    let _s_script_mgr_handle = dropper_wrapper_fn(rt.handle(), || async { SCRIPT_MGR.write().unwrap().unload() });
    scripts::add_scripts()?;
    azothacore_modules::add_scripts()?;

    rt.block_on(async { start_db().await })?;
    let _db_handle = dropper_wrapper_fn(rt.handle(), || async { stop_db().await });

    // set server offline (not connectable)

    let realm_id = CONFIG_MGR.read().unwrap().world().RealmID;
    rt.block_on(async {
        sql_w_args(
            "UPDATE realmlist SET flag = (flag & ~?) | ? WHERE id = ?",
            qargs!(
                FlagSet::from(RealmFlags::Offline).bits(),
                FlagSet::from(RealmFlags::VersionMismatch).bits(),
                realm_id
            ),
        )
        .execute(LoginDatabase::get())
        .await
    })?;

    rt.block_on(async { load_realm_info().await })?;

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

async fn start_db() -> AzResult<()> {
    let (realm_id, updates, auth_cfg, world_cfg, character_cfg) = {
        let config_mgr_r = CONFIG_MGR.read().unwrap();
        let world_cfg = config_mgr_r.world();
        (
            world_cfg.RealmID,
            world_cfg.Updates.clone(),
            world_cfg.LoginDatabaseInfo.clone(),
            world_cfg.WorldDatabaseInfo.clone(),
            world_cfg.CharacterDatabaseInfo.clone(),
        )
    };
    let registered_modules = module_scripts();
    let login_db_loader = DatabaseLoader::new(DBFlagCharacter, registered_modules.iter().map(String::from), &auth_cfg, &updates);
    let world_db_loader = DatabaseLoader::new(DBFlagLogin, registered_modules.iter().map(String::from), &world_cfg, &updates);
    let chars_db_laoder = DatabaseLoader::new(DBFlagWorld, registered_modules.iter().map(String::from), &character_cfg, &updates);

    let (auth_db, world_db, chars_db) = tokio::try_join!(login_db_loader.load(), world_db_loader.load(), chars_db_laoder.load())?;
    LoginDatabase::set(auth_db);
    WorldDatabase::set(world_db);
    CharacterDatabase::set(chars_db);

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
    sql_w_args(
        "UPDATE version SET core_version = ?, core_revision = ?",
        qargs!(GIT_VERSION, GIT_HASH),
    )
    .execute(WorldDatabase::get())
    .await?;

    S_WORLD.write().unwrap().load_db_version()?;

    info!("> Version DB world:     {}", S_WORLD.read().unwrap().get_db_version());

    SCRIPT_MGR.read().unwrap().on_after_databases_loaded(updates.EnableDatabases);

    Ok(())
}

async fn stop_db() -> AzResult<()> {
    LoginDatabase::get().close().await;
    WorldDatabase::get().close().await;
    CharacterDatabase::get().close().await;

    Ok(())
}

/// Clear 'online' status for all accounts with characters in this realm
#[instrument]
async fn clear_online_accounts(realm_id: u32) -> AzResult<()> {
    // Reset online status for all accounts with characters on the current realm
    // pussywizard: tc query would set online=0 even if logged in on another realm >_>
    sql_w_args("UPDATE account SET online = ? WHERE online = ?", qargs!(false, realm_id))
        .execute(LoginDatabase::get())
        .await?;
    // Reset online status for all characters
    sql_w_args("UPDATE characters SET online = ? WHERE online <> ?", qargs!(false, false))
        .execute(LoginDatabase::get())
        .await?;

    Ok(())
}

#[instrument]
async fn load_realm_info() -> AzResult<()> {
    let realm_id = CONFIG_MGR.read().unwrap().world().RealmID;

    let realm = sql_w_args(
        "SELECT id, name, address, localAddress, localSubnetMask, port, icon, flag, timezone, allowedSecurityLevel, population, gamebuild FROM realmlist WHERE id = ?",
        qargs!(realm_id),
    ).try_map(|r| {
        use sqlx::Row;
        let id = r.try_get("id")?;
        let name = r.try_get("name")?;
        let external_address: String = r.try_get("address")?;
        let local_address: String = r.try_get("localAddress")?;
        let local_subnet_mask: String = r.try_get("localSubnetMask")?;
        let port = r.try_get("port")?;
        let realm_type: u8 = r.try_get("icon")?;
        let flag: u16 = r.try_get("flag")?;
        let timezone = r.try_get("timezone")?;
        let allowed_security_level: u8 = r.try_get("allowedSecurityLevel")?;
        let population_level = r.try_get("population")?;
        let build = r.try_get("gamebuild")?;
        let external_address = external_address.parse().map_err(|e| {
            sqlx::Error::ColumnDecode{index: "address".to_string(), source:  Box::new(e)}
        })?;
        let local_address = local_address.parse().map_err(|e| {
            sqlx::Error::ColumnDecode{index: "localAddress".to_string(), source:  Box::new(e)}
        })?;
        let local_subnet_mask = local_subnet_mask.parse().map_err(|e| {
            sqlx::Error::ColumnDecode{index: "localSubnetMask".to_string(), source:  Box::new(e)}
        })?;
        let realm_type = RealmType::try_from(realm_type).map_err(|e| {
            sqlx::Error::ColumnDecode{index: "icon".to_string(), source:  Box::new(e)}
        })?;
        let allowed_security_level = AccountTypes::try_from(allowed_security_level).map_err(|e| {
            sqlx::Error::ColumnDecode{index: "allowedSecurityLevel".to_string(), source:  Box::new(e)}
        })?;
        // let realm_type: RealmType = FromPrimitive::from_u32(realm_type).ok_or_else(|| )?;
        let flag = FlagSet::<RealmFlags>::new(flag).map_err(|e| {
            sqlx::Error::ColumnDecode{index: "flag".to_string(), source:  Box::new(io::Error::new(io::ErrorKind::Other, format!("invalid bits: {e}")))}
        })?;
        Ok(Realm{
            id,
            build,
            external_address,
            local_address,
            local_subnet_mask,
            port,
            realm_type,
            name,
            flag,
            timezone,
            allowed_security_level,
            population_level,
        })
    }).fetch_one(LoginDatabase::get()).await?;

    for x in &[realm.external_address, realm.local_address, realm.local_subnet_mask] {
        if (x.to_owned(), realm.port).to_socket_addrs()?.next().is_none() {
            return Err(az_error!("Could not resolve address {x}"));
        }
    }

    WorldRealm::set(realm);

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
