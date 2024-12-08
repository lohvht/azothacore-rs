pub mod account_script;
pub mod database_script;
pub mod world_script;

#[macro_export]
macro_rules! input_script_non {
    ( $input_ty:ty ) => {{
        use bevy::prelude::IntoSystem;
        let mut _sys = Some(IntoSystem::into_system(|_inp: $input_ty| {}));
        _sys = None;
        _sys
    }};
}
