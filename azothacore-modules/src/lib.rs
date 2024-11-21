use bevy::prelude::{IntoSystemConfigs, SystemSet};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulesInitSet;

include!(concat!(env!("OUT_DIR"), "/build_modules_link.rs"));
