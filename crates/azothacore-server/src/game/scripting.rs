use bevy::{app::App, prelude::SystemSet};
use script_mgr::ScriptMgr;
use tracing::info;

pub mod script_defines;
pub mod script_mgr;
pub mod script_object;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScriptsInitSet;

/// TODO: Move this to its own crate as well, don't compile along with "azerothcore-server"
/// crate
pub fn scripts_plugin(_app: &mut App) {
    info!(target:"server::loading", "Initializing Scripts...");
    // Adding scripts first, then they can load modules
    // let mut script_mgr = ScriptMgr::default();

    // scripts::add_scripts(app.world_mut(), &mut script_mgr);
    // azothacore_module_scripts::add_scripts(app.world_mut(), &mut script_mgr);
    // app.insert_resource(script_mgr);
}

// pub mod mod_skeleton_example;
// pub fn modules_plugin(_app: &mut bevy::prelude::App) {
//     tracing::info!(modules = ? MODULES_LIST, "initialising modules!");
//     _app.add_systems(
//         bevy::prelude::Startup,
//         mod_skeleton_example::init.in_set(crate::ModulesInitSet),
//     );
// }
// pub static MODULES_LIST: &[&str] = &["mod_skeleton_example"];
