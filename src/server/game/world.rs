mod world_impl;
mod world_trait;

use once_cell::sync::OnceCell;
use thiserror::Error;
use tokio::sync::RwLock;
pub use world_impl::*;
pub use world_trait::*;

use crate::server::shared::realms::Realm;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("World had trouble stopping")]
    StopFailed,
    #[error("DB execution error")]
    DBError(#[from] sqlx::Error),
}

pub struct WorldRealm;

impl WorldRealm {
    pub fn get() -> &'static Realm {
        WORLD_REALM.get().expect("Realm is not initialised yet")
    }

    pub fn set(realm: Realm) {
        WORLD_REALM.set(realm).expect("Realm has already been set")
    }
}

pub static S_WORLD: RwLock<World> = RwLock::const_new(World::new());
static WORLD_REALM: OnceCell<Realm> = OnceCell::new();
