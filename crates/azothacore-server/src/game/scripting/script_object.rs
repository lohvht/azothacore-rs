use azothacore_common::AzResult;
use bevy::{
    ecs::system::SystemId,
    prelude::{Component, IntoSystem, System, World},
};

/// Component to keep track of afterload scripts
#[derive(Component)]
pub struct AfterLoadScriptObject<O>(pub Option<O>);

pub trait Script {
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

pub trait IntoScriptObject<S, O> {
    fn create_from_systems(bevy_world: &mut World, s: &S) -> O;
}

#[derive(Component, Clone)]
pub struct ScriptObject {
    pub name:                String,
    pub is_database_bound:   SystemId<(), bool>,
    pub is_afterload_script: SystemId<(), bool>,
    pub check_validity:      SystemId<(), AzResult<()>>,
}

impl<S: Script> IntoScriptObject<S, Self> for ScriptObject {
    fn create_from_systems(bevy_world: &mut World, s: &S) -> Self {
        Self {
            name:                s.name(),
            is_database_bound:   bevy_world.register_system(s.is_database_bound()),
            is_afterload_script: bevy_world.register_system(s.is_afterload_script()),
            check_validity:      bevy_world.register_system(s.check_validity()),
        }
    }
}

pub trait ScriptObjectTrait: Component + Clone {
    fn remove_systems_from_bevy(&self, bevy_world: &mut World);
}

impl ScriptObjectTrait for ScriptObject {
    fn remove_systems_from_bevy(&self, bevy_world: &mut World) {
        _ = bevy_world.unregister_system(self.is_database_bound);
        _ = bevy_world.unregister_system(self.is_afterload_script);
        _ = bevy_world.unregister_system(self.check_validity);
    }
}
