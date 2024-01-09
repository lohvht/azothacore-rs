pub mod realm_list;

use std::net::SocketAddr;

use azothacore_common::AccountTypes;
use flagset::{flags, FlagSet};
use ipnet::IpNet;
use num::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;

use super::networking::socket::AddressOrName;

/// Type of server, this is values from second column of Cfg_Configs.dbc
#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum RealmType {
    Normal = 0,
    Pvp = 1,
    Normal2 = 4,
    Rp = 6,
    RpPvp = 8,
    MaxClient = 14,
    /// custom, free for all pvp mode like arena PvP in all zones except rest activated places and sanctuaries
    /// replaced by REALM_PVP in realm list
    FfaPvp = 16,
}

const CONFIG_ID_BY_TYPE: [u32; 14] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];

#[derive(Error, Debug, Clone)]
#[error("parse realm type error: got {got}")]
pub struct RealmTypeParseError {
    got: u8,
}

impl TryFrom<u8> for RealmType {
    type Error = RealmTypeParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(RealmTypeParseError { got: value })
    }
}

flags! {
  pub enum RealmFlags: u16 {
    None             = 0x00,
    VersionMismatch  = 0x01,
    Offline          = 0x02,
    Specifybuild     = 0x04,
    Unk1             = 0x08,
    Unk2             = 0x10,
    Recommended      = 0x20,
    New              = 0x40,
    Full             = 0x80,
  }
}

#[derive(Debug)]
pub struct Realm {
    pub id:                     BnetRealmHandle,
    pub build:                  u32,
    pub external_address:       SocketAddr,
    pub local_address:          SocketAddr,
    pub local_network:          IpNet,
    pub port:                   u16,
    pub realm_type:             RealmType,
    pub name:                   String,
    pub flag:                   FlagSet<RealmFlags>,
    pub timezone:               u8,
    pub allowed_security_level: AccountTypes,
    pub population_level:       f32,
}

impl Realm {
    pub fn config_id(&self) -> u32 {
        CONFIG_ID_BY_TYPE[self.realm_type.to_usize().unwrap()]
    }

    pub fn address_for_client<'a>(&'a self, client_address: &'a AddressOrName) -> &'a SocketAddr {
        let client_address = match client_address {
            // If its a name, we use local address
            AddressOrName::Name(_) => return &self.local_address,
            AddressOrName::Addr(a) => a,
        };

        if client_address.ip().is_loopback() {
            // Try guessing if realm is also connected locally
            if self.local_address.ip().is_loopback() || self.external_address.ip().is_loopback() {
                client_address
            } else {
                // Assume that user connecting from the machine that bnetserver is located on
                // has all realms available in his local network
                &self.local_address
            }
        } else if self.local_network.contains(&client_address.ip()) {
            &self.local_address
        } else {
            &self.external_address
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BnetRealmHandle {
    region: u8,
    site:   u8,
    /// primary key in `realmlist` table
    realm:  u32,
}

impl BnetRealmHandle {
    pub const fn new(region: u8, battlegroup: u8, index: u32) -> Self {
        Self {
            region,
            site: battlegroup,
            realm: index,
        }
    }

    pub fn from_realm_address(realm_address: u32) -> Self {
        Self {
            region: u8::try_from((realm_address >> 24) & 0xFF).unwrap(),
            site:   u8::try_from((realm_address >> 16) & 0xFF).unwrap(),
            realm:  realm_address & 0xFFFF,
        }
    }

    pub fn get_address(&self) -> u32 {
        (u32::from(self.region) << 24) | (u32::from(self.site) << 16) | self.realm
    }

    pub fn get_address_string(&self) -> String {
        format!("{}-{}-{}", self.region, self.site, self.realm)
    }

    pub fn get_sub_region_address(&self) -> String {
        format!("{}-{}-0", self.region, self.site)
    }
}

impl PartialOrd for BnetRealmHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BnetRealmHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.realm.cmp(&other.realm)
    }
}

impl PartialEq for BnetRealmHandle {
    fn eq(&self, other: &Self) -> bool {
        self.realm == other.realm
    }
}

impl Eq for BnetRealmHandle {}
