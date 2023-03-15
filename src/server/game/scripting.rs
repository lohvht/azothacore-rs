use std::sync::{atomic::AtomicI32, Arc};

use parking_lot::Mutex;
use tracing::info;

use crate::{modules, server::game::scripts};

pub trait ScriptObject {
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
    fn check_validity(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

macro_rules! script_register_method {
    (  $script_registry_name:tt ) => {
        fn register(script: Arc<Mutex<Self>>)
        where
            Self: 'static + Sized,
        {
            $script_registry_name::add_script(Box::new(script));
        }
    };
}

pub trait WorldScript: ScriptObject + Sync + Send {
    script_register_method!(WorldScriptRegistry);

    /// Called when the open/closed state of the world changes.
    fn on_open_state_change(&mut self, _open: bool) {}

    /// Called after the world configuration is (re)loaded.
    fn on_after_config_load(&mut self, _reload: bool) {}

    /// Called when loading custom database tables
    fn on_load_custom_database_table(&mut self) {}

    /// Called when loading module configuration. Returns Ok(Vec(String)) containing the paths
    /// used for a given module if successfully loaded, else return an error.
    fn on_load_module_config(&mut self, _reload: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }

    /// Called before the world configuration is (re)loaded.
    fn on_before_config_load(&mut self, _reload: bool) {}

    /// Called before the message of the day is changed.
    fn on_motd_change(&mut self, _new_motd: &str) {}

    /// Called when a world shutdown is initiated.
    fn on_shutdown_initiate(&mut self, _shutdown_exit_code: u32 /**/, _shutdown_mask: u64) {}

    /// Called when a world shutdown is cancelled.
    fn on_shutdown_cancel(&mut self) {}

    /// Called on every world tick (don't execute too heavy code here).
    fn on_update(&mut self, _diff: u32) {}

    /// Called when the world is started.
    fn on_startup(&mut self) {}

    /// Called when the world is actually shut down.
    fn on_shutdown(&mut self) {}

    /// Called after all maps are unloaded from core
    fn on_after_unload_all_maps(&mut self) {}

    ///
    /// @brief This hook runs before finalizing the player world session. Can be also used to mutate the cache version of the Client.
    ///
    /// @param version The cache version that we will be sending to the Client.
    ///
    fn on_before_finalize_player_world_session(&mut self, _cache_version: u32) {}

    ///
    /// @brief This hook runs after all scripts loading and before itialized
    ///
    fn on_before_world_initialized(&mut self) {}
}

pub struct ScriptMgr {
    scheduled_scripts: AtomicI32,
}

impl ScriptMgr {
    pub fn initialise() -> Result<(), Box<dyn std::error::Error>> {
        info!("initialising scripts...");
        scripts::register()?;
        modules::register()?;

        Ok(())
    }

    pub fn unload() {
        WorldScriptRegistry::unload();
    }

    pub fn get_script_id_count() -> i64 {
        let mut i: i64 = 0;
        i += WorldScriptRegistry::get_script_id_counter();

        i
    }
}

/// WorldScript functions
impl ScriptMgr {
    pub fn on_load_module_config(is_reload: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut script_configs: Vec<String> = Vec::new();
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            let res = script.lock().on_load_module_config(is_reload)?;
            script_configs.extend(res.into_iter());
        }
        Ok(script_configs)
    }

    pub fn on_load_custom_database_table(&mut self) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_load_custom_database_table();
        }
    }

    pub fn on_open_state_change(open: bool) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_open_state_change(open);
        }
    }

    pub fn on_before_config_load(reload: bool) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_before_config_load(reload);
        }
    }

    pub fn on_after_config_load(reload: bool) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_after_config_load(reload);
        }
    }

    pub fn on_before_finalize_player_world_session(cache_version: u32) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_before_finalize_player_world_session(cache_version);
        }
    }

    pub fn on_motd_change(new_motd: &str) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_motd_change(new_motd);
        }
    }

    pub fn on_shutdown_initiate(shutdown_exit_code: u32, shutdown_mask: u64) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_shutdown_initiate(shutdown_exit_code, shutdown_mask);
        }
    }

    pub fn on_shutdown_cancel() {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_shutdown_cancel();
        }
    }

    pub fn on_update(diff: u32) {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_update(diff);
        }
    }

    pub fn on_startup() {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_startup();
        }
    }

    pub fn on_shutdown() {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_shutdown();
        }
    }

    pub fn on_before_world_initialized() {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_before_world_initialized();
        }
    }

    pub fn on_after_unload_all_maps() {
        for (_script_id, script) in WORLD_SCRIPTS.read().iter() {
            script.lock().on_after_unload_all_maps();
        }
    }
}

// template class AC_GAME_API ScriptRegistry<AccountScript>;
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
// template class AC_GAME_API ScriptRegistry<DatabaseScript>;
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

macro_rules! script_registry {
    (  $script_pointer_list:tt, $al_scripts:tt, $script_id_counter:tt, $script_trait:tt => $script_registry_name:tt ) => {
        pub struct $script_registry_name {}

        impl $crate::server::game::scripting::$script_registry_name {
            /// Force unload all scripts registered
            pub fn unload() {
                *$al_scripts.lock() = Vec::new();
                *$script_pointer_list.write() = std::collections::BTreeMap::new();
            }

            pub fn get_script_id_counter() -> i64 {
                $script_id_counter.load(std::sync::atomic::Ordering::SeqCst)
            }

            pub fn add_script(script: Box<std::sync::Arc<parking_lot::Mutex<dyn $script_trait>>>) {
                if script.lock().is_database_bound() {
                    $al_scripts.lock().push(script);
                    return;
                }
                _ = script.lock().check_validity();
                // We're dealing with a code-only script; just add it.
                $script_pointer_list
                    .write()
                    .insert($script_id_counter.load(std::sync::atomic::Ordering::SeqCst), script);
                $script_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }

            /// Adds the database-bound (i.e. after load) scripts to script management
            #[tracing::instrument]
            pub fn add_al_scripts() {
                let mut locked = $al_scripts.lock();
                while let Some(script) = locked.pop() {
                    _ = script.lock().check_validity();
                    if !script.lock().is_database_bound() {
                        // We're dealing with a code-only script; just add it.
                        $script_pointer_list
                            .write()
                            .insert($script_id_counter.load(std::sync::atomic::Ordering::SeqCst), script);
                        $script_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        continue;
                    }
                    // Get an ID for the script. An ID only exists if it's a script that is assigned in the database
                    // through a script name (or similar).
                    let script_id: i64 = match $crate::server::game::globals::object_mgr::S_OBJECT_MGR
                        .read()
                        .get_script_id(&script.lock().name())
                    {
                        Err(e) => {
                            tracing::error!(
                                "Scripted named '{}' is not assigned in the database. error was: {}",
                                script.lock().name(),
                                e
                            );
                            continue;
                        },
                        Ok(i) => i,
                    };
                    // Drop / delete existing scripts that have names that match the incoming script
                    let mut guard = $script_pointer_list.write();
                    let old_scripts = guard.drain_filter(|_k, v| v.lock().name() == script.lock().name());
                    // should not panic here
                    let old_scripts_count: i64 = old_scripts.count().try_into().unwrap();

                    // Assign new script!
                    guard.insert(script_id, script);

                    // Increment the script count only with new scripts.
                    $script_id_counter.fetch_sub(old_scripts_count - 1, std::sync::atomic::Ordering::SeqCst);
                }
            }

            pub fn get_script_by_id(script_id: i64) -> Option<Box<std::sync::Arc<parking_lot::Mutex<dyn $script_trait>>>> {
                let guard = $script_pointer_list.read();
                let res = guard.get(&script_id)?;
                Some(res.to_owned())
            }
        }

        /// The actual list of $script_trait scripts. This will be accessed concurrently, so it must not be modified
        /// after server startup.
        static $script_pointer_list: parking_lot::RwLock<
            std::collections::BTreeMap<i64, Box<std::sync::Arc<parking_lot::Mutex<dyn $script_trait>>>>,
        > = parking_lot::RwLock::new(std::collections::BTreeMap::new());
        /// After database load scripts
        static $al_scripts: parking_lot::Mutex<Vec<Box<std::sync::Arc<parking_lot::Mutex<dyn $script_trait>>>>> =
            parking_lot::Mutex::new(Vec::new());
        /// Counter used for code-only scripts.
        static $script_id_counter: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    };
}

script_registry! {WORLD_SCRIPTS, PENDING_AL_WORLD_SCRIPTS, WORLD_SCRIPTS_ID_COUNTER, WorldScript => WorldScriptRegistry}
