use std::sync::atomic::AtomicI32;

use flagset::FlagSet;
use tracing::info;

use crate::{common::configuration::DatabaseTypeFlags, modules, server::game::scripts, GenericResult};

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
    fn check_validity(&self) -> GenericResult {
        Ok(())
    }
}

pub trait WorldScript: ScriptObject {
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

pub trait DatabaseScript: ScriptObject {
    fn on_after_databases_loaded(&mut self, _update_flags: FlagSet<DatabaseTypeFlags>) {}
}

pub struct ScriptMgr {
    scheduled_scripts: AtomicI32,
}

impl ScriptMgr {
    pub async fn initialise() -> GenericResult {
        info!("initialising scripts...");
        scripts::register().await?;
        modules::register().await?;

        Ok(())
    }

    pub async fn unload() -> GenericResult {
        WorldScriptRegistry::unload().await;
        DatabaseScriptRegistry::unload().await;

        Ok(())
    }

    pub async fn get_script_id_count() -> i64 {
        let mut i: i64 = 0;
        i += WorldScriptRegistry::get_script_id_counter().await;
        i += DatabaseScriptRegistry::get_script_id_counter().await;

        i
    }
}

macro_rules! script_register_method {
    (  $register_fn_name:tt, $script_trait:tt, $script_registry_name:tt ) => {
        pub async fn $register_fn_name(script: std::sync::Arc<tokio::sync::Mutex<dyn $script_trait>>) {
            $script_registry_name::add_script(Box::new(script)).await;
        }
    };
}

/// WorldScript functions
impl ScriptMgr {
    script_register_method!(register_world_script, WorldScript, WorldScriptRegistry);

    pub async fn on_load_module_config(is_reload: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut script_configs: Vec<String> = Vec::new();
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            let res = script.lock().await.on_load_module_config(is_reload)?;
            script_configs.extend(res.into_iter());
        }
        Ok(script_configs)
    }

    pub async fn on_load_custom_database_table(&mut self) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_load_custom_database_table();
        }
    }

    pub async fn on_open_state_change(open: bool) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_open_state_change(open);
        }
    }

    pub async fn on_before_config_load(reload: bool) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_before_config_load(reload);
        }
    }

    pub async fn on_after_config_load(reload: bool) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_after_config_load(reload);
        }
    }

    pub async fn on_before_finalize_player_world_session(cache_version: u32) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_before_finalize_player_world_session(cache_version);
        }
    }

    pub async fn on_motd_change(new_motd: &str) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_motd_change(new_motd);
        }
    }

    pub async fn on_shutdown_initiate(shutdown_exit_code: u32, shutdown_mask: u64) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_shutdown_initiate(shutdown_exit_code, shutdown_mask);
        }
    }

    pub async fn on_shutdown_cancel() {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_shutdown_cancel();
        }
    }

    pub async fn on_update(diff: u32) {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_update(diff);
        }
    }

    pub async fn on_startup() {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_startup();
        }
    }

    pub async fn on_shutdown() {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_shutdown();
        }
    }

    pub async fn on_before_world_initialized() {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_before_world_initialized();
        }
    }

    pub async fn on_after_unload_all_maps() {
        for (_script_id, script) in WORLD_SCRIPTS.read().await.iter() {
            script.lock().await.on_after_unload_all_maps();
        }
    }
}

/// WorldScript functions
impl ScriptMgr {
    script_register_method!(register_database_script, DatabaseScript, DatabaseScriptRegistry);

    pub async fn on_after_databases_loaded(update_flags: FlagSet<DatabaseTypeFlags>) {
        for (_script_id, script) in DATABASE_SCRIPTS.read().await.iter() {
            script.lock().await.on_after_databases_loaded(update_flags);
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
            pub async fn unload() {
                *$al_scripts.lock().await = Vec::new();
                *$script_pointer_list.write().await = std::collections::BTreeMap::new();
            }

            pub async fn get_script_id_counter() -> i64 {
                $script_id_counter.load(std::sync::atomic::Ordering::SeqCst)
            }

            pub async fn add_script(script: Box<std::sync::Arc<tokio::sync::Mutex<dyn $script_trait>>>) {
                if script.lock().await.is_database_bound() {
                    $al_scripts.lock().await.push(script);
                    return;
                }
                _ = script.lock().await.check_validity();
                // We're dealing with a code-only script; just add it.
                $script_pointer_list
                    .write()
                    .await
                    .insert($script_id_counter.load(std::sync::atomic::Ordering::SeqCst), script);
                $script_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }

            /// Adds the database-bound (i.e. after load) scripts to script management
            #[tracing::instrument]
            pub async fn add_al_scripts() {
                let mut locked = $al_scripts.lock().await;
                while let Some(script) = locked.pop() {
                    _ = script.lock().await.check_validity();
                    if !script.lock().await.is_database_bound() {
                        // We're dealing with a code-only script; just add it.
                        $script_pointer_list
                            .write()
                            .await
                            .insert($script_id_counter.load(std::sync::atomic::Ordering::SeqCst), script);
                        $script_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        continue;
                    }
                    let script_name = script.lock().await.name();
                    // Get an ID for the script. An ID only exists if it's a script that is assigned in the database
                    // through a script name (or similar).
                    let script_id: i64 = match $crate::server::game::globals::object_mgr::S_OBJECT_MGR
                        .read()
                        .await
                        .get_script_id(&script_name)
                    {
                        Err(e) => {
                            tracing::error!("Scripted named '{}' is not assigned in the database. error was: {}", script_name, e);
                            continue;
                        },
                        Ok(i) => i,
                    };
                    // Drop / delete existing scripts that have names that match the incoming script
                    let mut guard = $script_pointer_list.write().await;

                    let mut all_script_id_to_name = Vec::new();
                    for (k, v) in guard.iter() {
                        all_script_id_to_name.push((*k, v.lock().await.name().clone()));
                    }
                    // Take out old scripts if it matches the same script name.
                    let mut old_scripts_count: i64 = 0;
                    for (script_id, old_script_name) in all_script_id_to_name {
                        if script_name == old_script_name {
                            old_scripts_count += 1;
                            guard.remove_entry(&script_id);
                        }
                    }

                    // Assign new script!
                    guard.insert(script_id, script);

                    // Increment the script count only with new scripts.
                    $script_id_counter.fetch_sub(old_scripts_count - 1, std::sync::atomic::Ordering::SeqCst);
                }
            }

            pub async fn get_script_by_id(script_id: i64) -> Option<Box<std::sync::Arc<tokio::sync::Mutex<dyn $script_trait>>>> {
                let guard = $script_pointer_list.read().await;
                let res = guard.get(&script_id)?;
                Some(res.to_owned())
            }
        }

        /// The actual list of $script_trait scripts. This will be accessed concurrently, so it must not be modified
        /// after server startup.
        static $script_pointer_list: tokio::sync::RwLock<std::collections::BTreeMap<i64, Box<std::sync::Arc<tokio::sync::Mutex<dyn $script_trait>>>>> =
            tokio::sync::RwLock::const_new(std::collections::BTreeMap::new());
        /// After database load scripts
        static $al_scripts: tokio::sync::Mutex<Vec<Box<std::sync::Arc<tokio::sync::Mutex<dyn $script_trait>>>>> = tokio::sync::Mutex::const_new(Vec::new());
        /// Counter used for code-only scripts.
        static $script_id_counter: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    };
}

script_registry! {WORLD_SCRIPTS, PENDING_AL_WORLD_SCRIPTS, WORLD_SCRIPTS_ID_COUNTER, WorldScript => WorldScriptRegistry}
script_registry! {DATABASE_SCRIPTS, PENDING_AL_DATABASE_SCRIPTS, DATABASE_SCRIPTS_ID_COUNTER, DatabaseScript => DatabaseScriptRegistry}
