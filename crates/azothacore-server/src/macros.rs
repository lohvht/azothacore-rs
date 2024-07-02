/// Adds the sript modules via two different
#[macro_export]
macro_rules! add_script_modules {
    ( INCLUDE_MOD; $($module_name:tt);* ) => {
        $(
            mod $module_name;
        )*

        add_script_modules!($($module_name);*);
    };
    ( $($module_name:tt);* ) => {
        #[doc = "Runs through a run of init functions, returning early if at the first script that fails to register"]
        pub fn add_scripts(bevy_world: &mut bevy::prelude::World, script_mgr: &mut $crate::game::scripting::script_mgr::ScriptMgr) {
            $(
                $module_name::init(bevy_world, script_mgr);
            )*
        }

        pub static SCRIPTS: &[&str] = &[
            $(
                stringify!($module_name),
            )*
        ];
    };
}
