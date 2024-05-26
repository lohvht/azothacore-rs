mod world_impl;
mod world_trait;

use std::sync::OnceLock;

use flagset::flags;
use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;
use tokio::sync::{RwLock as AsyncRwLock, RwLockReadGuard as AsyncRwLockReadGuard, RwLockWriteGuard as AsyncRwLockWriteGuard};
pub use world_impl::*;
pub use world_trait::*;

use crate::shared::realms::Realm;

#[derive(PartialEq, serde::Deserialize, serde::Serialize, Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum RealmZone {
    // any language
    Unknown = 0,
    // any language
    Development = 1,
    // extended-Latin
    UnitedStates = 2,
    // extended-Latin
    Oceanic = 3,
    // extended-Latin
    LatinAmerica = 4,
    // basic-Latin at create, any at login
    Tournament5 = 5,
    // East-Asian
    Korea = 6,
    // basic-Latin at create, any at login
    Tournament7 = 7,
    // extended-Latin
    English = 8,
    // extended-Latin
    German = 9,
    // extended-Latin
    French = 10,
    // extended-Latin
    Spanish = 11,
    // Cyrillic
    Russian = 12,
    // basic-Latin at create, any at login
    Tournament13 = 13,
    // East-Asian
    Taiwan = 14,
    // basic-Latin at create, any at login
    Tournament15 = 15,
    // East-Asian
    China = 16,
    // basic-Latin at create, any at login
    Cn1 = 17,
    // basic-Latin at create, any at login
    Cn2 = 18,
    // basic-Latin at create, any at login
    Cn3 = 19,
    // basic-Latin at create, any at login
    Cn4 = 20,
    // basic-Latin at create, any at login
    Cn5 = 21,
    // basic-Latin at create, any at login
    Cn6 = 22,
    // basic-Latin at create, any at login
    Cn7 = 23,
    // basic-Latin at create, any at login
    Cn8 = 24,
    // basic-Latin at create, any at login
    Tournament25 = 25,
    // any language
    TestServer = 26,
    // basic-Latin at create, any at login
    Tournament27 = 27,
    // any language
    QaServer = 28,
    // basic-Latin at create, any at login
    Cn9 = 29,
    // any language
    TestServer2 = 30,
    // basic-Latin at create, any at login
    Cn10 = 31,
    Ctc = 32,
    Cnc = 33,
    // basic-Latin at create, any at login
    Cn1_4 = 34,
    // basic-Latin at create, any at login
    Cn2_6_9 = 35,
    // basic-Latin at create, any at login
    Cn3_7 = 36,
    // basic-Latin at create, any at login
    Cn5_8 = 37,
}

flags! {
    pub enum StrictName: u8 {
        Latin = 0b01,
        RealmSpecific = 0b10,
    }
}

flags! {
    pub enum CharacterCreateFactionDisabled: u8 {
        Alliance = 0b001,
        Horde    = 0b010,
        Neutral  = 0b100,
    }
}

flags! {
    pub enum CharacterCreateRaceDisabled: u32 {
        Human            = 1,
        Orc              = 2,
        Dwarf            = 4,
        NightElf         = 8,
        Undead           = 16,
        Tauren           = 32,
        Gnome            = 64,
        Troll            = 128,
        Goblin           = 256,
        BloodElf         = 512,
        Draenei          = 1024,
        Worgen           = 2097152,
        PandarenNeutral  = 8388608,
        PandarenAlliance = 16777216,
        PandarenHorde    = 33554432,
    }
}

flags! {
    pub enum CharacterCreateClassDisabled: u32 {
        Warrior     = 1,
        Paladin     = 2,
        Hunter      = 4,
        Rogue       = 8,
        Priest      = 16,
        DeathKnight = 32,
        Shaman      = 64,
        Mage        = 128,
        Warlock     = 256,
        Monk        = 512,
        Druid       = 1024,
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum SkipCinematics {
    #[default]
    Show,
    ShowFirstCharacterOfRace,
    Disable,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum GroupVisibilityMode {
    Party,
    #[default]
    Raid,
    Faction,
    None,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum TalentsInspectingMode {
    #[default]
    Disabled,
    SameFaction,
    All,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ChatStrictLinkCheckingSeverity {
    #[default]
    Disabled,
    EnabledValidPipe,
    EnabledValidPipeOrder,
    EnabledValidStrict,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ChatStrictLinkCheckingKick {
    #[default]
    Ignore,
    DisconnectMalformed,
}

flags! {
    pub enum ArenaQueueAnnouncerDetail: u8 {
        TeamName    = 0b01,
        TeamRatings = 0b10,
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum CharDeleteMethod {
    #[default]
    RemoveFromDB,
    UnlinkFromAccount,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ItemDeleteMethod {
    #[default]
    RemoveFromDB,
    SaveItemToDB,
}
#[derive(Default, serde::Deserialize, serde::Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum PvPTokenMapAllowType {
    #[default]
    All,
    Battlegrounds,
    FfaArea,
    BattlegroundsAndFfaAreas,
}

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("World had trouble stopping")]
    StopFailed,
    #[error("DB execution error: {0}")]
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

pub struct SWorld;

impl SWorld {
    pub fn get() -> &'static AsyncRwLock<impl WorldTrait> {
        &WORLD
    }

    pub async fn read() -> AsyncRwLockReadGuard<'static, impl WorldTrait> {
        WORLD.read().await
    }

    pub async fn write() -> AsyncRwLockWriteGuard<'static, impl WorldTrait> {
        WORLD.write().await
    }
}

static REALM: OnceLock<Realm> = OnceLock::new();

static WORLD: AsyncRwLock<World> = AsyncRwLock::const_new(World::new());
