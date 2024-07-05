use azothacore_common::configuration::ConfigMgr;
use azothacore_server::game::{
    scripting::script_mgr::{ScriptMgr, ScriptObjectSystems, WorldScriptSystems},
    world::WorldConfig,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
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
        Some(IntoSystem::into_system(|In(_reload), _cfg: Res<ConfigMgr<WorldConfig>>| {}))
    }
}

pub fn init(bevy_world: &mut bevy::prelude::World, script_mgr: &mut ScriptMgr) {
    let script = MyWorld {};

    script_mgr.register_world_script(bevy_world, script);
}
