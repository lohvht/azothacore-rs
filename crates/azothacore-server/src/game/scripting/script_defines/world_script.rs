use bevy::{
    ecs::system::{FunctionSystem, SystemId},
    prelude::{Component, In, System, World},
};

use crate::{
    game::scripting::script_object::{IntoScriptObject, Script, ScriptObjectTrait},
    input_script_non,
};

pub trait WorldScript: Script {
    /// Called when the open/closed state of the world changes.
    fn on_open_state_change(&self) -> Option<impl System<In = In<bool>, Out = ()>> {
        input_script_non!(In<bool>)
    }

    /// Called after the world configuration is (re)loaded.
    fn on_after_config_load(&self) -> Option<impl System<In = In<bool>, Out = ()>> {
        input_script_non!(In<bool>)
    }

    /// Called when loading custom database tables
    fn on_load_custom_database_table(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }

    /// Called before the world configuration is (re)loaded.
    fn on_before_config_load(&self) -> Option<impl System<In = In<bool>, Out = ()>> {
        input_script_non!(In<bool>)
    }

    /// Called before the message of the day is changed.
    fn on_motd_change(&self) -> Option<impl System<In = In<String>, Out = ()>> {
        input_script_non!(In<String>)
    }

    /// Called when a world shutdown is initiated.
    fn on_shutdown_initiate(&self) -> Option<impl System<In = In<(u32, u64)>, Out = ()>> {
        input_script_non!(In<(u32, u64)>)
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
    fn on_before_finalize_player_world_session(&self) -> Option<impl System<In = In<u32>, Out = ()>> {
        input_script_non!(In<u32>)
    }

    ///
    /// @brief This hook runs after all scripts loading and before itialized
    ///
    fn on_before_world_initialized(&self) -> Option<impl System<In = (), Out = ()>> {
        None::<FunctionSystem<fn(), fn()>>
    }
}

#[derive(Component, Clone)]
pub struct WorldScriptObject {
    pub on_open_state_change: Option<SystemId<In<bool>>>,
    pub on_after_config_load: Option<SystemId<In<bool>>>,
    pub on_load_custom_database_table: Option<SystemId>,
    pub on_before_config_load: Option<SystemId<In<bool>>>,
    pub on_motd_change: Option<SystemId<In<String>>>,
    pub on_shutdown_initiate: Option<SystemId<In<(u32, u64)>>>,
    pub on_shutdown_cancel: Option<SystemId>,
    pub on_update: Option<SystemId>,
    pub on_startup: Option<SystemId>,
    pub on_shutdown: Option<SystemId>,
    pub on_after_unload_all_maps: Option<SystemId>,
    pub on_before_finalize_player_world_session: Option<SystemId<In<u32>>>,
    pub on_before_world_initialized: Option<SystemId>,
}

impl<S: WorldScript> IntoScriptObject<S, WorldScriptObject> for S {
    fn create_from_systems(bevy_world: &mut World, s: &S) -> WorldScriptObject {
        WorldScriptObject {
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
        }
    }
}

impl ScriptObjectTrait for WorldScriptObject {
    fn remove_systems_from_bevy(&self, bevy_world: &mut World) {
        _ = self.on_open_state_change.map(|s| bevy_world.unregister_system(s));
        _ = self.on_after_config_load.map(|s| bevy_world.unregister_system(s));
        _ = self.on_load_custom_database_table.map(|s| bevy_world.unregister_system(s));
        _ = self.on_before_config_load.map(|s| bevy_world.unregister_system(s));
        _ = self.on_motd_change.map(|s| bevy_world.unregister_system(s));
        _ = self.on_shutdown_initiate.map(|s| bevy_world.unregister_system(s));
        _ = self.on_shutdown_cancel.map(|s| bevy_world.unregister_system(s));
        _ = self.on_update.map(|s| bevy_world.unregister_system(s));
        _ = self.on_startup.map(|s| bevy_world.unregister_system(s));
        _ = self.on_shutdown.map(|s| bevy_world.unregister_system(s));
        _ = self.on_after_unload_all_maps.map(|s| bevy_world.unregister_system(s));
        _ = self.on_before_finalize_player_world_session.map(|s| bevy_world.unregister_system(s));
        _ = self.on_before_world_initialized.map(|s| bevy_world.unregister_system(s));
    }
}
