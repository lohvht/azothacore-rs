use std::{collections::BTreeMap, sync::atomic::AtomicU32};

use azothacore_common::{configuration::DatabaseType, AzResult};
use bevy::{
    ecs::system::{FunctionSystem, SystemId},
    prelude::*,
};
use flagset::FlagSet;

use crate::game::globals::object_mgr::OBJECT_MGR;

pub trait ScriptObjectSystems {
    fn name(&self) -> String {
        let original = std::any::type_name::<Self>();
        match original.rsplit_once(':') {
            None => original.to_string(),
            Some((_suffix, postfix)) => postfix.to_string(),
        }
    }
    fn is_database_bound(&self) -> impl System<In = (), Out = bool> {
        IntoSystem::into_system(|| false)
    }
    fn is_afterload_script(&self) -> impl System<In = (), Out = bool> {
        self.is_database_bound()
    }

    fn check_validity(&self) -> impl System<In = (), Out = AzResult<()>> {
        IntoSystem::into_system(|| Ok(()))
    }
}

#[derive(Clone)]
pub struct ScriptObject {
    pub name:                String,
    pub is_database_bound:   SystemId<(), bool>,
    pub is_afterload_script: SystemId<(), bool>,
    pub check_validity:      SystemId<(), AzResult<()>>,
}

pub trait ScriptObjectTrait {
    fn script_object(&self) -> &ScriptObject;
}

impl ScriptObjectTrait for ScriptObject {
    fn script_object(&self) -> &Self {
        self
    }
}

impl<S: ScriptObjectSystems> IntoScript<S, Self> for ScriptObject {
    fn create_from_systems(bevy_world: &mut World, s: S) -> Self {
        Self {
            name:                s.name(),
            is_database_bound:   bevy_world.register_system(s.is_database_bound()),
            is_afterload_script: bevy_world.register_system(s.is_afterload_script()),
            check_validity:      bevy_world.register_system(s.check_validity()),
        }
    }
}

pub trait IntoScript<Sys, Sc>
where
    Sys: ScriptObjectSystems,
    Sc: ScriptObjectTrait + Clone,
{
    fn create_from_systems(bevy_world: &mut World, s: Sys) -> Sc;
}

pub trait WorldScriptSystems: ScriptObjectSystems {
    /// Called when the open/closed state of the world changes.
    fn on_open_state_change(&self) -> Option<impl System<In = bool, Out = ()>> {
        None::<FunctionSystem<fn(In<bool>), fn(In<bool>)>>
    }

    /// Called after the world configuration is (re)loaded.
    fn on_after_config_load(&self) -> Option<impl System<In = bool, Out = ()>> {
        None::<FunctionSystem<fn(In<bool>), fn(In<bool>)>>
    }

    /// Called when loading custom database tables
    fn on_load_custom_database_table(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }

    /// Called before the world configuration is (re)loaded.
    fn on_before_config_load(&self) -> Option<impl System<In = bool, Out = ()>> {
        None::<FunctionSystem<fn(In<bool>), fn(In<bool>)>>
    }

    /// Called before the message of the day is changed.
    fn on_motd_change(&self) -> Option<impl System<In = String, Out = ()>> {
        None::<FunctionSystem<fn(In<String>), fn(In<String>)>>
    }

    /// Called when a world shutdown is initiated.
    fn on_shutdown_initiate(&self) -> Option<impl System<In = (u32, u64), Out = ()>> {
        None::<FunctionSystem<fn(In<(u32, u64)>), fn(In<(u32, u64)>)>>
    }

    /// Called when a world shutdown is cancelled.
    fn on_shutdown_cancel(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }

    /// Called on every world tick (don't execute too heavy code here).
    fn on_update(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }

    /// Called when the world is started.
    fn on_startup(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }

    /// Called when the world is actually shut down.
    fn on_shutdown(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }

    /// Called after all maps are unloaded from core
    fn on_after_unload_all_maps(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }

    ///
    /// @brief This hook runs before finalizing the player world session. Can be also used to mutate the cache version of the Client.
    ///
    /// @param version The cache version that we will be sending to the Client.
    ///
    fn on_before_finalize_player_world_session(&self) -> Option<impl System<In = u32, Out = ()>> {
        None::<FunctionSystem<fn(In<u32>), fn(In<u32>)>>
    }

    ///
    /// @brief This hook runs after all scripts loading and before itialized
    ///
    fn on_before_world_initialized(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }
}

#[derive(Clone)]
pub struct WorldScript {
    pub base: ScriptObject,
    pub on_open_state_change: Option<SystemId<bool>>,
    pub on_after_config_load: Option<SystemId<bool>>,
    pub on_load_custom_database_table: Option<SystemId>,
    pub on_before_config_load: Option<SystemId<bool>>,
    pub on_motd_change: Option<SystemId<String>>,
    pub on_shutdown_initiate: Option<SystemId<(u32, u64)>>,
    pub on_shutdown_cancel: Option<SystemId>,
    pub on_update: Option<SystemId>,
    pub on_startup: Option<SystemId>,
    pub on_shutdown: Option<SystemId>,
    pub on_after_unload_all_maps: Option<SystemId>,
    pub on_before_finalize_player_world_session: Option<SystemId<u32>>,
    pub on_before_world_initialized: Option<SystemId>,
}

impl ScriptObjectTrait for WorldScript {
    fn script_object(&self) -> &ScriptObject {
        &self.base
    }
}

impl<S: WorldScriptSystems> IntoScript<S, Self> for WorldScript {
    fn create_from_systems(bevy_world: &mut World, s: S) -> Self {
        Self {
            on_open_state_change: s.on_open_state_change().map(|sys| bevy_world.register_system(sys)),
            on_after_config_load: s.on_after_config_load().map(|sys| bevy_world.register_system(sys)),
            on_load_custom_database_table: s.on_load_custom_database_table().map(|sys| bevy_world.register_system(sys)),
            on_before_config_load: s.on_before_config_load().map(|sys| bevy_world.register_system(sys)),
            on_motd_change: s.on_motd_change().map(|sys| bevy_world.register_system(sys)),
            on_shutdown_initiate: s.on_shutdown_initiate().map(|sys| bevy_world.register_system(sys)),
            on_shutdown_cancel: s.on_shutdown_cancel().map(|sys| bevy_world.register_system(sys)),
            on_update: s.on_update().map(|sys| bevy_world.register_system(sys)),
            on_startup: s.on_startup().map(|sys| bevy_world.register_system(sys)),
            on_shutdown: s.on_shutdown().map(|sys| bevy_world.register_system(sys)),
            on_after_unload_all_maps: s.on_after_unload_all_maps().map(|sys| bevy_world.register_system(sys)),
            on_before_finalize_player_world_session: s.on_before_finalize_player_world_session().map(|sys| bevy_world.register_system(sys)),
            on_before_world_initialized: s.on_before_world_initialized().map(|sys| bevy_world.register_system(sys)),
            base: ScriptObject::create_from_systems(bevy_world, s),
        }
    }
}

pub trait DatabaseScriptSystems: ScriptObjectSystems {
    fn on_after_databases_loaded(&self) -> Option<impl System<In = FlagSet<DatabaseType>, Out = ()>> {
        None::<FunctionSystem<fn(In<FlagSet<DatabaseType>>), fn(In<FlagSet<DatabaseType>>)>>
    }
}

#[derive(Clone)]
pub struct DatabaseScript {
    pub base: ScriptObject,
    pub on_after_databases_loaded: Option<SystemId<FlagSet<DatabaseType>>>,
}

impl ScriptObjectTrait for DatabaseScript {
    fn script_object(&self) -> &ScriptObject {
        &self.base
    }
}

impl<S: DatabaseScriptSystems> IntoScript<S, Self> for DatabaseScript {
    fn create_from_systems(bevy_world: &mut World, s: S) -> Self {
        Self {
            on_after_databases_loaded: s.on_after_databases_loaded().map(|sys| bevy_world.register_system(sys)),
            base: ScriptObject::create_from_systems(bevy_world, s),
        }
    }
}

// pub trait AccountScript: ScriptObject {
//     fn on_account_login(&self, _account_id: u32) {}
//     fn on_last_ip_update(&self, _account_id: u32, _ip: &str) {}
//     fn on_failed_account_login(&self, _account_id: u32) {}
//     fn on_email_change(&self, _account_id: u32) {}
//     fn on_failed_email_change(&self, _account_id: u32) {}
//     fn on_password_change(&self, _account_id: u32) {}
//     fn on_failed_password_change(&self, _account_id: u32) {}
//     fn can_account_create_character(&self, _account_id: u32, _char_race: u8, _char_class: u8) {}
// }

// pub trait CommandScript: ScriptObject {
//     /// Should return a pointer to a valid command table (ChatCommand array) to be used by ChatHandler.
//     fn commands(&self) -> Vec<ChatCommand>;
// }

#[derive(Resource)]
pub struct ScriptMgr {
    // scheduled_scripts: AtomicI32,
    world:    ScriptRegistry<WorldScript>,
    database: ScriptRegistry<DatabaseScript>,
    // account:  ScriptRegistry<dyn AccountScript>,
    // command:  ScriptRegistry<dyn CommandScript>,
}

impl Default for ScriptMgr {
    fn default() -> Self {
        Self {
            // scheduled_scripts: AtomicI32::new(0),
            world:    ScriptRegistry::new(),
            database: ScriptRegistry::new(),
            // account:  ScriptRegistry::new(),
            // command:  ScriptRegistry::new(),
        }
    }
}

/// WorldScript functions
impl ScriptMgr {
    pub fn register_world_script<S: WorldScriptSystems>(&mut self, bevy_world: &mut World, script_sys: S) {
        registry_add_script(&mut self.world, bevy_world, script_sys);
    }

    pub fn on_load_custom_database_table(&self, commands: &mut Commands) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_load_custom_database_table {
                commands.run_system(i)
            }
        }
    }

    pub fn on_open_state_change(&self, commands: &mut Commands, open: bool) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_open_state_change {
                commands.run_system_with_input(i, open)
            }
        }
    }

    pub fn on_before_config_load(&self, commands: &mut Commands, reload: bool) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_before_config_load {
                commands.run_system_with_input(i, reload)
            }
        }
    }

    pub fn on_after_config_load(&self, commands: &mut Commands, reload: bool) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_after_config_load {
                commands.run_system_with_input(i, reload)
            }
        }
    }

    pub fn on_before_finalize_player_world_session(&self, commands: &mut Commands, cache_version: u32) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_before_finalize_player_world_session {
                commands.run_system_with_input(i, cache_version)
            }
        }
    }

    pub fn on_motd_change(&self, commands: &mut Commands, new_motd: &str) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_motd_change {
                commands.run_system_with_input(i, new_motd.to_string())
            }
        }
    }

    pub fn on_shutdown_initiate(&self, commands: &mut Commands, shutdown_exit_code: u32, shutdown_mask: u64) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_shutdown_initiate {
                commands.run_system_with_input(i, (shutdown_exit_code, shutdown_mask))
            }
        }
    }

    pub fn on_shutdown_cancel(&self, commands: &mut Commands) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_shutdown_cancel {
                commands.run_system(i)
            }
        }
    }

    pub fn on_update(&self, commands: &mut Commands) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_update {
                commands.run_system(i)
            }
        }
    }

    pub fn on_startup(&self, commands: &mut Commands) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_startup {
                commands.run_system(i)
            }
        }
    }

    pub fn on_shutdown(&self, commands: &mut Commands) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_shutdown {
                commands.run_system(i)
            }
        }
    }

    pub fn on_before_world_initialized(&self, commands: &mut Commands) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_before_world_initialized {
                commands.run_system(i)
            }
        }
    }

    pub fn on_after_unload_all_maps(&self, commands: &mut Commands) {
        for script in self.world.script_pointer_list.values() {
            if let Some(i) = script.on_after_unload_all_maps {
                commands.run_system(i)
            }
        }
    }
}

/// DatabaseScript functions
impl ScriptMgr {
    pub fn register_database_script<S: DatabaseScriptSystems>(&mut self, bevy_world: &mut World, script_sys: S) {
        registry_add_script(&mut self.database, bevy_world, script_sys);
    }

    pub fn on_after_databases_loaded(&self, commands: &mut Commands, update_flags: FlagSet<DatabaseType>) {
        for script in self.database.script_pointer_list.values() {
            if let Some(i) = script.on_after_databases_loaded {
                commands.run_system_with_input(i, update_flags)
            }
        }
    }
}

// /// AccountScript functions
// impl ScriptMgr {
//     pub fn register_account_script(&mut self, script: Arc<dyn AccountScript>) {
//         self.account.add_script(script);
//     }

//     pub fn on_account_login(&self, account_id: u32) {
//         for script in self.account.script_pointer_list.values() {
//             script.on_account_login(account_id);
//         }
//     }

//     // TODO: Impl this azerothcore hook (Not in TC)
//     pub fn on_last_ip_update(&self, account_id: u32, ip: String) {
//         for script in self.account.script_pointer_list.values() {
//             script.on_last_ip_update(account_id, &ip);
//         }
//     }

//     pub fn on_failed_account_login(&self, account_id: u32) {
//         for script in self.account.script_pointer_list.values() {
//             script.on_failed_account_login(account_id);
//         }
//     }

//     pub fn on_email_change(&self, account_id: u32) {
//         for script in self.account.script_pointer_list.values() {
//             script.on_email_change(account_id);
//         }
//     }

//     pub fn on_failed_email_change(&self, account_id: u32) {
//         for script in self.account.script_pointer_list.values() {
//             script.on_failed_email_change(account_id);
//         }
//     }

//     pub fn on_password_change(&self, account_id: u32) {
//         for script in self.account.script_pointer_list.values() {
//             script.on_password_change(account_id);
//         }
//     }

//     pub fn on_failed_password_change(&self, account_id: u32) {
//         for script in self.account.script_pointer_list.values() {
//             script.on_failed_password_change(account_id);
//         }
//     }

//     // TODO: Impl this azerothcore hook (Not in TC)
//     pub fn can_account_create_character(&self, account_id: u32, char_race: u8, char_class: u8) {
//         for script in self.account.script_pointer_list.values() {
//             script.can_account_create_character(account_id, char_race, char_class);
//         }
//     }
// }

// /// CommandScript functions
// impl ScriptMgr {
//     pub fn register_command_script(&mut self, script: Arc<dyn CommandScript>) {
//         self.command.add_script(script);
//     }

//     pub fn chat_commands(&self) -> BTreeMap<String, ChatCommand> {
//         let mut commands = BTreeMap::new();
//         for script in self.command.script_pointer_list.values() {
//             for c in script.commands() {
//                 commands.entry(c.name.clone()).or_insert(c);
//             }
//         }
//         commands
//     }
// }

// template class AC_GAME_API ScriptRegistry<AchievementCriteriaScript>;
// template class AC_GAME_API ScriptRegistry<AchievementScript>;
// template class AC_GAME_API ScriptRegistry<AllCreatureScript>;
// template class AC_GAME_API ScriptRegistry<AllGameObjectScript>;
// template class AC_GAME_API ScriptRegistry<AllItemScript>;
// template class AC_GAME_API ScriptRegistry<AllMapScript>;
// template class AC_GAME_API ScriptRegistry<AreaTriggerScript>;
// template class AC_GAME_API ScriptRegistry<ArenaScript>;
// template class AC_GAME_API ScriptRegistry<ArenaTeamScript>;
// template class AC_GAME_API ScriptRegistry<AuctionHouseScript>;
// template class AC_GAME_API ScriptRegistry<BGScript>;
// template class AC_GAME_API ScriptRegistry<BattlegroundMapScript>;
// template class AC_GAME_API ScriptRegistry<BattlegroundScript>;
// template class AC_GAME_API ScriptRegistry<CommandSC>;
// template class AC_GAME_API ScriptRegistry<ConditionScript>;
// template class AC_GAME_API ScriptRegistry<CreatureScript>;
// template class AC_GAME_API ScriptRegistry<DynamicObjectScript>;
// template class AC_GAME_API ScriptRegistry<ElunaScript>;
// template class AC_GAME_API ScriptRegistry<FormulaScript>;
// template class AC_GAME_API ScriptRegistry<GameEventScript>;
// template class AC_GAME_API ScriptRegistry<GameObjectScript>;
// template class AC_GAME_API ScriptRegistry<GlobalScript>;
// template class AC_GAME_API ScriptRegistry<GroupScript>;
// template class AC_GAME_API ScriptRegistry<GuildScript>;
// template class AC_GAME_API ScriptRegistry<InstanceMapScript>;
// template class AC_GAME_API ScriptRegistry<ItemScript>;
// template class AC_GAME_API ScriptRegistry<LootScript>;
// template class AC_GAME_API ScriptRegistry<MailScript>;
// template class AC_GAME_API ScriptRegistry<MiscScript>;
// template class AC_GAME_API ScriptRegistry<MovementHandlerScript>;
// template class AC_GAME_API ScriptRegistry<OutdoorPvPScript>;
// template class AC_GAME_API ScriptRegistry<PetScript>;
// template class AC_GAME_API ScriptRegistry<PlayerScript>;
// template class AC_GAME_API ScriptRegistry<ServerScript>;
// template class AC_GAME_API ScriptRegistry<SpellSC>;
// template class AC_GAME_API ScriptRegistry<SpellScriptLoader>;
// template class AC_GAME_API ScriptRegistry<TransportScript>;
// template class AC_GAME_API ScriptRegistry<UnitScript>;
// template class AC_GAME_API ScriptRegistry<VehicleScript>;
// template class AC_GAME_API ScriptRegistry<WeatherScript>;
// template class AC_GAME_API ScriptRegistry<WorldMapScript>;
// template class AC_GAME_API ScriptRegistry<WorldObjectScript>;

pub struct ScriptRegistry<Sc> {
    /// The actual list of scripts. This will be accessed concurrently, so it must not be modified
    /// after server startup.
    script_pointer_list: BTreeMap<u32, Sc>,
    /// After database load script systems to be registered
    al_scripts:          Vec<Sc>,
    /// Counter used for code-only scripts.
    script_id_counter:   AtomicU32,
}

impl<Sc> Default for ScriptRegistry<Sc> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Sc> ScriptRegistry<Sc> {
    pub const fn new() -> Self {
        Self {
            script_pointer_list: BTreeMap::new(),
            al_scripts:          Vec::new(),
            script_id_counter:   AtomicU32::new(0),
        }
    }

    /// Force unload all scripts registered
    pub fn unload(&mut self) {
        self.al_scripts = Vec::new();
        self.script_pointer_list = BTreeMap::new();
    }

    pub fn get_script_id_counter(&self) -> u32 {
        self.script_id_counter.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn atomic_add_increment_script_id(&mut self) -> u32 {
        self.script_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

pub fn registry_add_script<Sys, Sc>(registry: &mut ScriptRegistry<Sc>, bevy_world: &mut World, script_sys: Sys)
where
    Sys: ScriptObjectSystems,
    Sc: IntoScript<Sys, Sc> + ScriptObjectTrait + Clone,
{
    let script = Sc::create_from_systems(bevy_world, script_sys);
    let base = script.script_object();
    if bevy_world.run_system(base.is_afterload_script).unwrap() {
        registry.al_scripts.push(script);
        return;
    }
    _ = bevy_world.run_system(base.check_validity).unwrap();
    // We're dealing with a code-only script; just add it.
    let prev = registry.atomic_add_increment_script_id();
    registry.script_pointer_list.insert(prev, script);
}

/// Adds the database-bound (i.e. after load) scripts to script management
pub fn registry_add_al_scripts<Sys, Sc>(registry: &mut ScriptRegistry<Sc>, bevy_world: &mut World)
where
    Sys: ScriptObjectSystems,
    Sc: IntoScript<Sys, Sc> + ScriptObjectTrait + Clone,
{
    while let Some(script) = registry.al_scripts.pop() {
        let base = script.script_object();
        _ = bevy_world.run_system(base.check_validity).unwrap();
        if !bevy_world.run_system(base.is_database_bound).unwrap() {
            // We're dealing with a code-only script; just add it.
            let prev = registry.atomic_add_increment_script_id();
            registry.script_pointer_list.insert(prev, script);
            continue;
        }
        let script_name = &base.name;
        // Get an ID for the script. An ID only exists if it's a script that is assigned in the database
        // through a script name (or similar).
        let script_id = match OBJECT_MGR.blocking_read().get_script_id(script_name) {
            Err(e) => {
                if !script_name.contains("Smart") {
                    tracing::error!(
                        target:"sql::sql",
                        err = e,
                        "Script named '{}' is not assigned in the database.",
                        script_name,
                    );
                }
                continue;
            },
            Ok(i) => i,
        };
        // Drop / delete existing scripts that have names that match the incoming script
        let mut all_script_id_to_name = Vec::new();
        for (k, v) in registry.script_pointer_list.iter() {
            all_script_id_to_name.push((*k, v.script_object().name.clone()));
        }
        // Take out old scripts if it matches the same script name.
        for (script_id, old_script_name) in all_script_id_to_name {
            if script_name == &old_script_name {
                registry.script_pointer_list.remove_entry(&script_id);
            }
        }
        // Assign new script!
        registry.script_pointer_list.insert(script_id, script);
    }
}

impl<Sc> ScriptRegistry<Sc>
where
    Sc: Clone,
{
    pub fn get_script_by_id(&self, script_id: u32) -> Option<Sc> {
        self.script_pointer_list.get(&script_id).cloned()
    }
}
