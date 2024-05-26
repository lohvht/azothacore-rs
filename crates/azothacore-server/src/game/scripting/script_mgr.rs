use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicI64, Arc},
};

use azothacore_common::{configuration::DatabaseType, AzResult};
use flagset::FlagSet;
use tokio::sync::RwLock as AsyncRwLock;

use crate::game::globals::object_mgr::OBJECT_MGR;

pub trait ScriptObject: Sync + Send {
    fn name(&self) -> String {
        let original = std::any::type_name::<Self>();
        match original.rsplit_once(':') {
            None => original.to_string(),
            Some((_suffix, postfix)) => postfix.to_string(),
        }
    }
    fn is_database_bound(&self) -> bool {
        false
    }
    fn check_validity(&self) -> AzResult<()> {
        Ok(())
    }
}

impl<W: ScriptObject + ?Sized> ScriptObject for Box<W> {
    #[inline]
    fn name(&self) -> String {
        (**self).name()
    }

    #[inline]
    fn check_validity(&self) -> AzResult<()> {
        (**self).check_validity()
    }

    #[inline]
    fn is_database_bound(&self) -> bool {
        (**self).is_database_bound()
    }
}

pub trait WorldScript: ScriptObject {
    /// Called when the open/closed state of the world changes.
    fn on_open_state_change(&self, _open: bool) {}

    /// Called after the world configuration is (re)loaded.
    fn on_after_config_load(&self, _reload: bool) {}

    /// Called when loading custom database tables
    fn on_load_custom_database_table(&self) {}

    /// Called when loading module configuration. Returns Ok(Vec(String)) containing the paths
    /// used for a given module if successfully loaded, else return an error.
    fn on_load_module_config(&self, _reload: bool) -> AzResult<Vec<String>> {
        Ok(Vec::new())
    }

    /// Called before the world configuration is (re)loaded.
    fn on_before_config_load(&self, _reload: bool) {}

    /// Called before the message of the day is changed.
    fn on_motd_change(&self, _new_motd: &str) {}

    /// Called when a world shutdown is initiated.
    fn on_shutdown_initiate(&self, _shutdown_exit_code: u32 /**/, _shutdown_mask: u64) {}

    /// Called when a world shutdown is cancelled.
    fn on_shutdown_cancel(&self) {}

    /// Called on every world tick (don't execute too heavy code here).
    fn on_update(&self, _diff: u32) {}

    /// Called when the world is started.
    fn on_startup(&self) {}

    /// Called when the world is actually shut down.
    fn on_shutdown(&self) {}

    /// Called after all maps are unloaded from core
    fn on_after_unload_all_maps(&self) {}

    ///
    /// @brief This hook runs before finalizing the player world session. Can be also used to mutate the cache version of the Client.
    ///
    /// @param version The cache version that we will be sending to the Client.
    ///
    fn on_before_finalize_player_world_session(&self, _cache_version: u32) {}

    ///
    /// @brief This hook runs after all scripts loading and before itialized
    ///
    fn on_before_world_initialized(&self) {}
}

pub trait DatabaseScript: ScriptObject {
    fn on_after_databases_loaded(&self, _update_flags: FlagSet<DatabaseType>) {}
}

pub trait AccountScript: ScriptObject {
    fn on_account_login(&self, _account_id: u32) {}
    fn on_last_ip_update(&self, _account_id: u32, _ip: &str) {}
    fn on_failed_account_login(&self, _account_id: u32) {}
    fn on_email_change(&self, _account_id: u32) {}
    fn on_failed_email_change(&self, _account_id: u32) {}
    fn on_password_change(&self, _account_id: u32) {}
    fn on_failed_password_change(&self, _account_id: u32) {}
    fn can_account_create_character(&self, _account_id: u32, _char_race: u8, _char_class: u8) {}
}

pub struct ScriptMgr {
    // scheduled_scripts: AtomicI32,
    world:    ScriptRegistry<dyn WorldScript>,
    database: ScriptRegistry<dyn DatabaseScript>,
    account:  ScriptRegistry<dyn AccountScript>,
    command:  ScriptRegistry<dyn CommandScript>,
}

impl Default for ScriptMgr {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptMgr {
    pub const fn new() -> Self {
        Self {
            // scheduled_scripts: AtomicI32::new(0),
            world:    ScriptRegistry::new(),
            database: ScriptRegistry::new(),
            account:  ScriptRegistry::new(),
            command:  ScriptRegistry::new(),
        }
    }

    pub fn unload(&mut self) -> AzResult<()> {
        self.world.unload();
        self.database.unload();
        self.account.unload();
        Ok(())
    }
}

/// WorldScript functions
impl ScriptMgr {
    pub fn register_world_script(&mut self, script: Arc<dyn WorldScript>) {
        self.world.add_script(script);
    }

    pub fn on_load_module_config(&self, is_reload: bool) -> AzResult<Vec<String>> {
        let mut script_configs: Vec<String> = Vec::new();
        for script in self.world.script_pointer_list.values() {
            let res = script.on_load_module_config(is_reload)?;
            script_configs.extend(res);
        }
        Ok(script_configs)
    }

    pub fn on_load_custom_database_table(&self) {
        for script in self.world.script_pointer_list.values() {
            script.on_load_custom_database_table();
        }
    }

    pub fn on_open_state_change(&self, open: bool) {
        for script in self.world.script_pointer_list.values() {
            script.on_open_state_change(open);
        }
    }

    pub fn on_before_config_load(&self, reload: bool) {
        for script in self.world.script_pointer_list.values() {
            script.on_before_config_load(reload);
        }
    }

    pub fn on_after_config_load(&self, reload: bool) {
        for script in self.world.script_pointer_list.values() {
            script.on_after_config_load(reload);
        }
    }

    pub fn on_before_finalize_player_world_session(&self, cache_version: u32) {
        for script in self.world.script_pointer_list.values() {
            script.on_before_finalize_player_world_session(cache_version);
        }
    }

    pub fn on_motd_change(&self, new_motd: &str) {
        for script in self.world.script_pointer_list.values() {
            script.on_motd_change(new_motd);
        }
    }

    pub fn on_shutdown_initiate(&self, shutdown_exit_code: u32, shutdown_mask: u64) {
        for script in self.world.script_pointer_list.values() {
            script.on_shutdown_initiate(shutdown_exit_code, shutdown_mask);
        }
    }

    pub fn on_shutdown_cancel(&self) {
        for script in self.world.script_pointer_list.values() {
            script.on_shutdown_cancel();
        }
    }

    pub fn on_update(&self, diff: u32) {
        for script in self.world.script_pointer_list.values() {
            script.on_update(diff);
        }
    }

    pub fn on_startup(&self) {
        for script in self.world.script_pointer_list.values() {
            script.on_startup();
        }
    }

    pub fn on_shutdown(&self) {
        for script in self.world.script_pointer_list.values() {
            script.on_shutdown();
        }
    }

    pub fn on_before_world_initialized(&self) {
        for script in self.world.script_pointer_list.values() {
            script.on_before_world_initialized();
        }
    }

    pub fn on_after_unload_all_maps(&self) {
        for script in self.world.script_pointer_list.values() {
            script.on_after_unload_all_maps();
        }
    }
}

/// DatabaseScript functions
impl ScriptMgr {
    pub fn register_database_script(&mut self, script: Arc<dyn DatabaseScript>) {
        self.database.add_script(script);
    }

    pub fn on_after_databases_loaded(&self, update_flags: FlagSet<DatabaseType>) {
        for script in self.database.script_pointer_list.values() {
            script.on_after_databases_loaded(update_flags);
        }
    }
}

/// AccountScript functions
impl ScriptMgr {
    pub fn register_account_script(&mut self, script: Arc<dyn AccountScript>) {
        self.account.add_script(script);
    }

    pub fn on_account_login(&self, account_id: u32) {
        for script in self.account.script_pointer_list.values() {
            script.on_account_login(account_id);
        }
    }

    // TODO: Impl this azerothcore hook (Not in TC)
    pub fn on_last_ip_update(&self, account_id: u32, ip: String) {
        for script in self.account.script_pointer_list.values() {
            script.on_last_ip_update(account_id, &ip);
        }
    }

    pub fn on_failed_account_login(&self, account_id: u32) {
        for script in self.account.script_pointer_list.values() {
            script.on_failed_account_login(account_id);
        }
    }

    pub fn on_email_change(&self, account_id: u32) {
        for script in self.account.script_pointer_list.values() {
            script.on_email_change(account_id);
        }
    }

    pub fn on_failed_email_change(&self, account_id: u32) {
        for script in self.account.script_pointer_list.values() {
            script.on_failed_email_change(account_id);
        }
    }

    pub fn on_password_change(&self, account_id: u32) {
        for script in self.account.script_pointer_list.values() {
            script.on_password_change(account_id);
        }
    }

    pub fn on_failed_password_change(&self, account_id: u32) {
        for script in self.account.script_pointer_list.values() {
            script.on_failed_password_change(account_id);
        }
    }

    // TODO: Impl this azerothcore hook (Not in TC)
    pub fn can_account_create_character(&self, account_id: u32, char_race: u8, char_class: u8) {
        for script in self.account.script_pointer_list.values() {
            script.can_account_create_character(account_id, char_race, char_class);
        }
    }
}

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
// template class AC_GAME_API ScriptRegistry<CommandScript>;
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

pub struct ScriptRegistry<T: ScriptObject + ?Sized> {
    /// The actual list of scripts. This will be accessed concurrently, so it must not be modified
    /// after server startup.
    script_pointer_list: BTreeMap<u32, Arc<T>>,
    /// After database load scripts
    al_scripts:          Vec<Arc<T>>,
    /// Counter used for code-only scripts.
    script_id_counter:   AtomicI64,
}

impl<T: ScriptObject + ?Sized> Default for ScriptRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ScriptObject + ?Sized> ScriptRegistry<T> {
    pub const fn new() -> Self {
        Self {
            script_pointer_list: BTreeMap::new(),
            al_scripts:          Vec::new(),
            script_id_counter:   AtomicI64::new(0),
        }
    }

    /// Force unload all scripts registered
    pub fn unload(&mut self) {
        self.al_scripts = Vec::new();
        self.script_pointer_list = BTreeMap::new();
    }

    pub fn get_script_id_counter(&self) -> i64 {
        self.script_id_counter.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn atomic_add_increment_script_id(&mut self) -> i64 {
        self.script_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn add_script(&mut self, script: Arc<T>) {
        if script.is_database_bound() {
            self.al_scripts.push(script);
            return;
        }
        _ = script.check_validity();
        // We're dealing with a code-only script; just add it.
        let prev = self.atomic_add_increment_script_id();
        self.script_pointer_list.insert(prev.try_into().unwrap(), script);
    }

    /// Adds the database-bound (i.e. after load) scripts to script management
    pub fn add_al_scripts(&mut self) {
        while let Some(script) = self.al_scripts.pop() {
            _ = script.check_validity();
            if !script.is_database_bound() {
                // We're dealing with a code-only script; just add it.
                let prev = self.atomic_add_increment_script_id();
                self.script_pointer_list.insert(prev.try_into().unwrap(), script);
                continue;
            }
            let script_name = script.name();
            // Get an ID for the script. An ID only exists if it's a script that is assigned in the database
            // through a script name (or similar).
            let script_id = match OBJECT_MGR.blocking_read().get_script_id(&script_name) {
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
            for (k, v) in self.script_pointer_list.iter() {
                all_script_id_to_name.push((*k, v.name().clone()));
            }
            // Take out old scripts if it matches the same script name.
            for (script_id, old_script_name) in all_script_id_to_name {
                if script_name == old_script_name {
                    self.script_pointer_list.remove_entry(&script_id);
                }
            }
            // Assign new script!
            self.script_pointer_list.insert(script_id, script);
        }
    }

    pub fn get_script_by_id(&self, script_id: u32) -> Option<Arc<T>> {
        self.script_pointer_list.get(&script_id).cloned()
    }
}

pub static SCRIPT_MGR: AsyncRwLock<ScriptMgr> = AsyncRwLock::const_new(ScriptMgr::new());
