use std::time::Instant;

use azothacore_common::{
    bevy_app::TokioRuntime,
    collision::management::vmap_mgr2::{vmap_mgr2_plugin, VMapManager2InitSet, VmapConfig},
    configuration::{ConfigMgr, DataDirConfig},
    AccountTypes,
};
use azothacore_database::{
    args,
    database_env::{LoginDatabase, LoginPreparedStmts},
};
use bevy::{
    app::PreUpdate,
    ecs::system::SystemId,
    prelude::{App, Commands, In, IntoSystem, IntoSystemConfigs, IntoSystemSetConfigs, Res, ResMut, Resource, Startup},
    time::{Timer, TimerMode},
};
use rand::{rngs::OsRng, Rng, TryRngCore};
use tracing::{error, info};

use crate::{
    game::{
        conditions::disable_mgr::{disable_mgr_plugin, DisableMgr, DisableMgrInitialLoadSet},
        entities::unit::{PlayerBaseMoveSpeed, BASE_MOVE_SPEED},
        globals::object_mgr::{handle_set_highest_guids_error, set_highest_guids},
        map::map_mgr::{GridCleanupTimer, MapUpdateTimer},
        scripting::script_mgr::ScriptMgr,
        time::WorldUpdateTime,
        world::{AllowedSecurityLevel, CurrentRealm, WorldSets, WorldTrait},
    },
    shared::data_stores::{db2_mgr_plugin, db2_structure::LiquidType, DB2Storage, InitDB2MgrSet},
};

#[allow(non_camel_case_types, non_snake_case)]
mod config;
pub use config::*;

#[derive(Resource)]
pub struct World {
    sysid_load_config_settings:      SystemId<In<bool>>,
    sysid_set_player_security_limit: SystemId<In<AccountTypes>>,
}

pub fn world_plugin(app: &mut App) {
    let world = World {
        sysid_load_config_settings:      app.world_mut().register_system(World::load_config_settings),
        sysid_set_player_security_limit: app.world_mut().register_system(World::set_player_security_limit),
    };
    app.insert_resource(world).insert_resource(AllowedSecurityLevel(AccountTypes::SecPlayer));
    add_set_initial_world_settings_system(app);
}

#[derive(Resource)]
struct StartupTime(Instant);

/// WUPDATE_UPTIME in TC/AC
#[derive(Resource)]
struct WorldUpdateUptimeTable(Timer);

/// WUPDATE_AUTOBROADCAST in TC/AC
#[derive(Resource)]
struct WorldUpdateAutoBroadcast(Timer);

/// Initialize the World
/// World::SetInitialWorldSettings
fn add_set_initial_world_settings_system(app: &mut App) {
    //- Server startup begin
    let startup_begin = StartupTime(Instant::now());
    app.insert_resource(startup_begin);

    //- Initialize the random number generator
    _ = OsRng.unwrap_err().random::<u8>();

    // ///- Initialize detour memory management
    // dtAllocSetCustom(dtCustomAlloc, dtCustomFree);

    //     // TODO: hirogoro 24jun2024: Implement the histories from here FOR PoolMgr:
    //     // https://github.com/TrinityCore/TrinityCore/commits/0b8eed2d547acc0ba115198cb306c4f9127af807/src/server/game/Pools/PoolMgr.cpp
    //     //
    //     // Seems like poolmgr in this revision (7.3.5) starts from here:
    //     // Core/Entities: Created factory methods to create new areatriggers, creatures and gameobjects - https://github.com/TrinityCore/TrinityCore/commit/6226189a1687e1a2b4fb5a490031c22b5f334dc6
    //     //
    //     // Notable commits:
    //     //  Core/Maps: Use FindMap instead of CreateBaseMap in places where the intent was to check for a existing map (and a loaded grid on that map) - https://github.com/TrinityCore/TrinityCore/commit/4c173e4b7b35161fcaaa4917da8fde2e4f3cbdd8
    //     //  NOTE: hirogoro (Optional, not in Azerothcore as of 24jun2024): Dynamic Creature/Go spawning: https://github.com/TrinityCore/TrinityCore/commit/03b125e6d1947258316c931499746696a95aded2
    //     //  Core/Pooling: Fixed less and less objects from pools being spawned: https://github.com/TrinityCore/TrinityCore/commit/00991543167cd15d9ef52a50f19f0b242dfdbe00
    //     //  Core/Pools: Fixed spawning in pools with both explicitly and equally: https://github.com/TrinityCore/TrinityCore/commit/5e774fc7f1941484ec86c1c5abe5bbb14f4e4090
    //     //  Core/Spawns: Exterminate CONFIG_SAVE_RESPAWN_TIME_IMMEDIATELY: https://github.com/TrinityCore/TrinityCore/commit/d5e58cef694d3db65f0a27b93099ae4e517685a4
    //     //
    //     // Commits after seems to transition to a different pool system. Keep track after
    app.add_plugins((
        db2_mgr_plugin,
        //- Initialize VMapManager function pointers (to untangle game/collision circular deps)
        disable_mgr_plugin,
        vmap_mgr2_plugin::<WorldConfig, DB2Storage<LiquidType>, DisableMgr>,
    ))
    .add_systems(
        Startup,
        ((
            // Initialize config settings
            World::load_initial_config.before(VMapManager2InitSet),
            // Initialize Allowed Security Level
            World::load_db_allowed_security_level,
            // Init highest guids before any table loading to prevent using not initialized guids in some code.
            set_highest_guids.pipe(handle_set_highest_guids_error),
        )
            .in_set(WorldSets::SetInitialWorldSettings),),
    )
    .add_systems(
        PreUpdate,
        // Record update if recording set in log and diff is greater then minimum set in log
        WorldUpdateTime::record_update,
    )
    .configure_sets(
        Startup,
        (InitDB2MgrSet, DisableMgrInitialLoadSet, VMapManager2InitSet).in_set(WorldSets::SetInitialWorldSettings),
    )
    .configure_sets(Startup, (VMapManager2InitSet.after(DisableMgrInitialLoadSet),))
    .configure_sets(Startup, (VMapManager2InitSet.after(InitDB2MgrSet),));
}

impl WorldTrait<WorldConfig> for World {
    // fn is_stopped(&self) -> bool {
    //     self.exit_code.is_some()
    // }

    /// LoadDBAllowedSecurityLevel in Ac/TC
    fn load_db_allowed_security_level(
        this: Res<Self>,
        mut commands: Commands,
        rt: Res<TokioRuntime>,
        login_db: Res<LoginDatabase>,
        current_realm: Res<CurrentRealm>,
    ) {
        let res = rt.block_on(async { LoginDatabase::sel_realmlist_security_level::<_, (u8,)>(&**login_db, args!(current_realm.id.realm)?).await });
        if let Ok(Some((account_type,))) = res {
            let account_type = AccountTypes::try_from(account_type).unwrap_or(AccountTypes::SecPlayer);
            commands.run_system_with_input(this.sysid_set_player_security_limit, account_type);
        }
    }

    /// SetPlayerSecurityLimit in Ac/TC
    fn set_player_security_limit(In(sec): In<AccountTypes>, mut allowed: ResMut<AllowedSecurityLevel>) {
        let update = sec > allowed.0;
        allowed.0 = sec;
        if update {
            //     // TODO: Implement me: KickAllLess
            //     self.kick_all_less(self.allowed_security_level);

            //c NOTE: Snippet from TC/AC here:
            // ```
            // /// Kick (and save) all players with security level less `sec`
            // void World::KickAllLess(AccountTypes sec)
            // {
            //     // session not removed at kick and will removed in next update tick
            //     for (SessionMap::const_iterator itr = m_sessions.begin(); itr != m_sessions.end(); ++itr)
            //         if (itr->second->GetSecurity() < sec)
            //             itr->second->KickPlayer("KickAllLess");
            // }
            // ```
        }
    }

    fn load_config_settings(In(reload): In<bool>, mut commands: Commands, mut cfg: ResMut<ConfigMgr<WorldConfig>>, script_mgr: ScriptMgr) {
        // Happens earlier than AC b/c config reload is built into Config object directly
        script_mgr.on_before_config_load(&mut commands, reload);
        if reload {
            if let Err(e) = cfg.reload_from_path() {
                error!(target:"server.loading", cause=?e, "World settings reload fail: can't read settings.");
                return;
            }
            // TODO: Implement me! => Right now appenders / loggers logic is buggy and also coupled together such that
            //       we cannot do reloads easily.
            // Use our log::init()
            // sLog->LoadFromConfig();
            // sMetric->LoadFromConfigs();
        }
        // TODO: Implement me!
        // // Set realm id and enable db logging
        // sLog->SetRealmId(realm.Id.Realm);

        info!(target:"server.loading", "Using {dbc:?} DBC Locale", dbc=cfg.DBCLocale);
        // load update time related configs
        commands.insert_resource(WorldUpdateTime::from(&**cfg));

        // // TODO: Implement me! either TC World::SetMotd OR MotdMgr::SetMotd
        // SetMotd(sConfigMgr->GetStringDefault("Motd", "Welcome to a Trinity Core Server."));

        // TODO: IMPLEMENT ME! Support mgr //- Read support system setting from the config file
        // SUPPORT_MGR.write().await.set_config(config_mgr_r.get("Support", SupportConfig::default));
        //
        // NOTE: TC IMPL:
        // ///- Read support system setting from the config file
        // m_bool_configs[CONFIG_SUPPORT_ENABLED] = sConfigMgr->GetBoolDefault("Support.Enabled", true);
        // m_bool_configs[CONFIG_SUPPORT_TICKETS_ENABLED] = sConfigMgr->GetBoolDefault("Support.TicketsEnabled", false);
        // m_bool_configs[CONFIG_SUPPORT_BUGS_ENABLED] = sConfigMgr->GetBoolDefault("Support.BugsEnabled", false);
        // m_bool_configs[CONFIG_SUPPORT_COMPLAINTS_ENABLED] = sConfigMgr->GetBoolDefault("Support.ComplaintsEnabled", false);
        // m_bool_configs[CONFIG_SUPPORT_SUGGESTIONS_ENABLED] = sConfigMgr->GetBoolDefault("Support.SuggestionsEnabled", false);
        // if (reload)
        // {
        //     sSupportMgr->SetSupportSystemStatus(m_bool_configs[CONFIG_SUPPORT_ENABLED]); // _bool_configs[CONFIG_ALLOW_TICKETS] in ACore
        //     sSupportMgr->SetTicketS  ystemStatus(m_bool_configs[CONFIG_SUPPORT_TICKETS_ENABLED]);
        //     sSupportMgr->SetBugSystemStatus(m_bool_configs[CONFIG_SUPPORT_BUGS_ENABLED]);
        //     sSupportMgr->SetComplaintSystemStatus(m_bool_configs[CONFIG_SUPPORT_COMPLAINTS_ENABLED]);
        //     sSupportMgr->SetSuggestionSystemStatus(m_bool_configs[CONFIG_SUPPORT_SUGGESTIONS_ENABLED]);
        // }
        // NOTE: AC IMPL:
        // ///- Read ticket system setting from the config file
        // _bool_configs[CONFIG_ALLOW_TICKETS] = sConfigMgr->GetOption<bool>("AllowTickets", true);
        // _bool_configs[CONFIG_DELETE_CHARACTER_TICKET_TRACE] = sConfigMgr->GetOption<bool>("DeletedCharacterTicketTrace", false);

        let new_player_base_move_speed = PlayerBaseMoveSpeed(BASE_MOVE_SPEED * *cfg.Rate.MoveSpeed);
        commands.insert_resource(new_player_base_move_speed);

        commands.insert_resource(GridCleanupTimer::from(&**cfg));
        commands.insert_resource(MapUpdateTimer::from(&**cfg));

        commands.insert_resource(WorldUpdateUptimeTable(Timer::new(*cfg.UpdateUptimeInterval, TimerMode::Repeating)));

        info!(target:"server.loading", "Using DataDir {dir:?}", dir=cfg.DataDir.display());
        info!(target:"server.loading", "WORLD: MMap data directory is: {}", cfg.mmaps_dir().display());
        if !cfg.vmap.enableHeight {
            error!(target:"server.loading", "VMap height checking disabled! Creatures movements and other various things WILL be broken! Expect no support.");
        }
        info!(target:"server.loading", "VMap support included. LineOfSight: {}, getHeight: {}, indoorCheck: {}", cfg.vmap.enableLOS, cfg.vmap.enableHeight, cfg.vmap.enableIndoorCheck);
        info!(target:"server.loading", "VMap data directory is: {}", cfg.vmaps_dir().display());

        commands.insert_resource(WorldUpdateAutoBroadcast(Timer::new(*cfg.AutoBroadcast.Timer, TimerMode::Repeating)));

        // TODO: Implement me if needed? else remove after impl DisableMgr::IsPathfindingEnabled
        // MMAP::MMapFactory::InitializeDisabledMaps();

        // call ScriptMgr if we're reloading the configuration
        script_mgr.on_after_config_load(&mut commands, reload);
    }

    // fn stop_now(&mut self, exit_code: i32) -> Result<i32, WorldError> {
    //     if self.is_stopped() {
    //         return Ok(self.exit_code.unwrap());
    //     }
    //     info!("Turning world flag to stopped");
    //     // if let Some(ct) = &self.cancel_token {
    //     //     ct.cancel();
    //     // }
    //     // self.cancel_token = None;
    //     self.exit_code = Some(exit_code);
    //     Ok(exit_code)
    // }
}

impl World {
    fn load_initial_config(this: Res<Self>, mut commands: Commands) {
        commands.run_system_with_input(this.sysid_load_config_settings, false);
    }
}
