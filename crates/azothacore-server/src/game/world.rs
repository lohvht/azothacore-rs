mod world_impl;
mod world_trait;

use std::sync::RwLock;

use thiserror::Error;
pub use world_impl::*;
pub use world_trait::*;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("World had trouble stopping")]
    StopFailed,
    #[error("DB execution error")]
    DBError(#[from] sqlx::Error),
}

pub static S_WORLD: RwLock<World> = RwLock::new(World::new());
