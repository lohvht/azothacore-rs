use std::{path::Path, sync::Arc, time::Duration};

use azothacore_common::{
    az_error,
    banner,
    configuration::{
        DatabaseType::{Character as DBFlagCharacter, Hotfix as DBFlagHotfix, Login as DBFlagLogin, World as DBFlagWorld},
        DbUpdates,
        CONFIG_MGR,
    },
    log,
    r#async::Context,
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
        world::{SWorld, WorldTrait},
    },
    shared::{
        panic_handler,
        realms::{realm_list::RealmList, Realm, RealmFlags},
        shared_defines::{ServerProcessType, ThisServerProcess},
        signal_handler,
    },
};
use clap::Parser;
use flagset::FlagSet;
use rand::{rngs::OsRng, Rng};
use tokio::sync::oneshot;
use tracing::{error, info, instrument};

fn main() -> AzResult<()> {
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build()?);
    let root_ctx = Context::new(rt.handle());
    panic_handler(root_ctx.clone());
    ThisServerProcess::set(ServerProcessType::Worldserver);
    let vm = ConsoleArgs::parse();
    {
        let mut cfg_mgr_w = CONFIG_MGR.blocking_write();
        cfg_mgr_w.configure(
            &vm.config,
            vm.dry_run,
            Box::new(|reload| Box::pin(async move { SCRIPT_MGR.write().await.on_load_module_config(reload) })),
        );
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

    banner::azotha_banner_show("worldserver-daemon", || {
        info!(
            target:"server::worldserver",
            "> Using configuration file       {}",
            CONFIG_MGR.blocking_read().get_filename().display()
        )
    });
    // Seed the OsRng here.
    // That way it won't auto-seed when calling OsRng and slow down the first world login
    OsRng.gen::<u64>();

    // worldserver PID file creation
    if let Ok(pid_file) = &CONFIG_MGR.blocking_read().get_option::<String>("PidFile") {
        let pid = create_pid_file(pid_file)?;
        error!(target:"server", "Daemon PID: {}", pid);
    }
    let ctx = root_ctx.clone();
    root_ctx.spawn(signal_handler(ctx));

    let ctx = root_ctx.clone();
    root_ctx.spawn(async move {
        if let Err(e) = load_scripts(ctx.clone()).await {
            error!(target:"server::loading", cause=%e, "error starting load scripts, terminating!");
            ctx.cancel();
        }
    });

    let realm_id = CONFIG_MGR.blocking_read().get_option::<u32>("RealmID")?;
    let (db_started_send, db_started_recv) = oneshot::channel();
    let ctx = root_ctx.clone();
    root_ctx.spawn(async move {
        if let Err(e) = start_db(ctx.clone(), db_started_send, realm_id).await {
            error!(target:"server::loading", cause=%e, "error starting DB");
            ctx.cancel();
        }
    });
    // Enforce DB to be up before everything else
    db_started_recv.blocking_recv().unwrap();

    // set server offline (not connectable)
    let ctx = root_ctx.clone();
    let (send, recv) = oneshot::channel();
    root_ctx.spawn(async move {
        let res = query_with(
            "UPDATE realmlist SET flag = flag | ? WHERE id = ?",
            params!(FlagSet::from(RealmFlags::Offline).bits(), realm_id),
        )
        .execute(&LoginDatabase::get())
        .await;
        if let Err(e) = res {
            error!(target:"server::loading", cause=%e, "error flipping realmlist offline while world is loading, terminating");
            ctx.cancel();
        }
        send.send(()).unwrap();
    });
    // Enforce realm status to be offline before everything else
    recv.blocking_recv().unwrap();

    let ctx = root_ctx.clone();
    root_ctx.spawn(RealmList::init(ctx, CONFIG_MGR.blocking_read().get("RealmsStateUpdateDelay", || 10)));

    let Some(realm) = load_realm_info(realm_id) else {
        error!(target:"server::loading", "Unable to find realm with ID: {realm_id}");
        root_ctx.cancel();
        root_ctx.tt.close();
        rt.block_on(root_ctx.tt.wait());
        return Err(az_error!("Unable to find realm with ID: {realm_id}"));
    };

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

    // // TODO: hirogoro@29/03/2023: Implement set initial world settings
    // // sWorld->SetInitialWorldSettings();
    // SWorld::get().write().await.set_initial_world_settings().await?;

    // Wait for shutdown by seeing if the tasks above have all run its course.
    // Tasks must perform graceful shutdown by using the Context above otherwise this
    // program will not terminate properly.
    root_ctx.tt.close();
    rt.block_on(root_ctx.tt.wait());

    info!("TERMINATING!");
    Ok(())
}

async fn load_scripts(ctx: Context) -> AzResult<()> {
    info!(target:"server::loading", "Initializing Scripts...");
    // Adding scripts first, then they can load modules
    scripts::add_scripts()?;
    azothacore_modules::add_scripts()?;

    CONFIG_MGR.write().await.load_modules_configs(false, true).await?;

    // Wait for cancellation
    ctx.cancelled().await;
    info!("Unloading scripts.");
    SCRIPT_MGR.write().await.unload()?;
    Ok(())
}

async fn start_db(ctx: Context, db_started_send: oneshot::Sender<()>, realm_id: u32) -> AzResult<()> {
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
        .execute(&WorldDatabase::get())
        .await?;

    let mut w = SWorld::write().await;
    w.load_db_version().await?;

    info!("> Version DB world:     {}", w.get_db_version());
    db_started_send.send(()).unwrap();

    SCRIPT_MGR.read().await.on_after_databases_loaded(updates.EnableDatabases);

    // Wait for cancellation
    ctx.cancelled().await;

    info!(target:"server", "Stopping database connection pools.");

    LoginDatabase::close().await;
    WorldDatabase::close().await;
    CharacterDatabase::close().await;
    HotfixDatabase::close().await;
    Ok(())
}

/// Clear 'online' status for all accounts with characters in this realm
#[instrument]
async fn clear_online_accounts(realm_id: u32) -> AzResult<()> {
    let login_db = LoginDatabase::get();
    let char_db = CharacterDatabase::get();

    // Reset online status for all accounts with characters on the current realm
    query_with(
        "UPDATE account SET online = 0 WHERE online > 0 AND id IN (SELECT acctid FROM realmcharacters WHERE realmid = ?)",
        params!(realm_id),
    )
    .execute(&login_db)
    .await?;

    // Reset online status for all characters
    query_with("UPDATE characters SET online = ? WHERE online <> ?", params!(false, false))
        .execute(&char_db)
        .await?;

    // Battleground instance ids reset at server restart
    query_with("UPDATE character_battleground_data SET instanceId = ?", params!(false))
        .execute(&char_db)
        .await?;
    Ok(())
}

fn load_realm_info(realm_id: u32) -> Option<Realm> {
    RealmList::get().realms().iter().find(|(r, _)| r.realm == realm_id).map(|(_, r)| r.to_owned())
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
