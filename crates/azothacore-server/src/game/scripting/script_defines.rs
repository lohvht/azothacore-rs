pub mod account_script;
pub mod database_script;
pub mod world_script;

#[macro_export]
macro_rules! input_script_non {
    ( $input_ty:ty ) => { $crate::input_script_non!($input_ty => ()) };
    ( $input_ty:ty => $out:ty ) => {{
        use bevy::prelude::*;
        let mut _sys = Some(IntoSystem::into_system(|_inp: $input_ty| -> $out { unreachable!() }));
        _sys = None;
        _sys
    }};
}
