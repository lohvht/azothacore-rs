mod world_impl;
mod world_trait;

use std::sync::OnceLock;

use thiserror::Error;
use tokio::sync::RwLock as AsyncRwLock;
pub use world_impl::*;
pub use world_trait::*;

use crate::shared::realms::Realm;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("World had trouble stopping")]
    StopFailed,
    #[error("DB execution error")]
    DBError(#[from] sqlx::Error),
}

pub struct CurrentRealm;

impl CurrentRealm {
    pub fn get() -> &'static Realm {
        REALM.get().expect("attempting to retrieve current realm when its not set, panicking")
    }

    pub fn set(realm: Realm) {
        REALM.set(realm).expect("attempting to set a realm when one already exists");
    }

    #[cfg(test)]
    /// Only used to set the global current realm during tests.
    pub fn setup_test() -> &'static Realm {
        use std::net::{Ipv4Addr, SocketAddr};

        use azothacore_common::AccountTypes;
        use ipnet::IpNet;

        use crate::shared::realms::{BnetRealmHandle, RealmFlags, RealmType};

        REALM.get_or_init(|| Realm {
            id:                     BnetRealmHandle {
                realm:  123,
                region: 2,
                site:   1,
            },
            build:                  456,
            external_address:       SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8085),
            local_address:          SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8085),
            local_network:          IpNet::with_netmask(
                std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                std::net::IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0)),
            )
            .unwrap(),
            port:                   8085,
            realm_type:             RealmType::Normal,
            name:                   "TEST_CLIENT".to_string(),
            flag:                   RealmFlags::None.into(),
            timezone:               0,
            allowed_security_level: AccountTypes::SecPlayer,
            population_level:       0.0,
        })
    }
}

static REALM: OnceLock<Realm> = OnceLock::new();

pub static WORLD: AsyncRwLock<World> = AsyncRwLock::const_new(World::new());
