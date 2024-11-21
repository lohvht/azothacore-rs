use azothacore_common::configuration::ConfigMgr;
use azothacore_server::game::{
    scripting::{script_defines::world_script::WorldScript, script_mgr::ScriptMgr, script_object::Script},
    world::WorldConfig,
};
use bevy::prelude::{Commands, In, IntoSystem, Res, System};
use serde::{Deserialize, Serialize};
use tracing::info;
#[derive(Deserialize, Serialize, Clone, Debug)]
struct MyConfig {
    enabled: bool,
}

// static MY_CONFIG: AsyncMutex<MyConfig> = AsyncMutex::const_new(MyConfig { enabled: false });

#[derive(Debug)]
struct MyWorld;

impl Script for MyWorld {}

impl WorldScript for MyWorld {
    fn on_after_config_load(&self) -> Option<impl System<In = bool, Out = ()>> {
        Some(IntoSystem::into_system(|In(_reload), _cfg: Res<ConfigMgr<WorldConfig>>| {
            info!("SKELETON PRINT");
        }))
    }
}

pub fn init(mut commands: Commands) {
    let script = MyWorld {};
    ScriptMgr::register_world_script(&mut commands, script);
}
