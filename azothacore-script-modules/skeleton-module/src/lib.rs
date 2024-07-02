use std::sync::Arc;

use azothacore_common::{configuration::ConfigMgr, AzResult};
use azothacore_server::game::{
    scripting::script_mgr::{ScriptMgr, ScriptObject, ScriptObjectSystems, WorldScript, WorldScriptSystems},
    world::WorldConfig,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
#[derive(Deserialize, Serialize, Clone, Debug)]
struct MyConfig {
    enabled: bool,
}

// static MY_CONFIG: AsyncMutex<MyConfig> = AsyncMutex::const_new(MyConfig { enabled: false });

#[derive(Debug)]
struct MyWorld;

impl ScriptObjectSystems for MyWorld {}

impl WorldScriptSystems for MyWorld {
    fn on_after_config_load(&self) -> Option<impl System<In = bool, Out = ()>> {
        Some(IntoSystem::into_system(|In(reload), cfg: Res<ConfigMgr<WorldConfig>>| {}))
    }
}

pub fn init(bevy_world: &mut bevy::prelude::World, script_mgr: &mut ScriptMgr) {
    let script = MyWorld {};

    script_mgr.register_world_script(bevy_world, script);
}
