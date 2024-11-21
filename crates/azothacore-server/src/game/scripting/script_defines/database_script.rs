use azothacore_common::configuration::DatabaseType;
use bevy::{
    ecs::system::{FunctionSystem, SystemId},
    prelude::{Component, In, System, World},
};
use flagset::FlagSet;

use crate::game::scripting::script_object::{IntoScriptObject, Script, ScriptObjectTrait};

pub trait DatabaseScript: Script {
    fn on_after_databases_loaded(&self) -> Option<impl System<In = FlagSet<DatabaseType>, Out = ()>> {
        None::<FunctionSystem<fn(In<FlagSet<DatabaseType>>), fn(In<FlagSet<DatabaseType>>)>>
    }
}

#[derive(Component, Clone)]
pub struct DatabaseScriptObject {
    pub on_after_databases_loaded: Option<SystemId<FlagSet<DatabaseType>>>,
}

impl<S: DatabaseScript> IntoScriptObject<S, DatabaseScriptObject> for S {
    fn create_from_systems(bevy_world: &mut World, s: &S) -> DatabaseScriptObject {
        DatabaseScriptObject {
            on_after_databases_loaded: s.on_after_databases_loaded().map(|sys| bevy_world.register_system(sys)),
        }
    }
}

impl ScriptObjectTrait for DatabaseScriptObject {
    fn remove_systems_from_bevy(&self, bevy_world: &mut World) {
        _ = self.on_after_databases_loaded.map(|s| bevy_world.remove_system(s));
    }
}
