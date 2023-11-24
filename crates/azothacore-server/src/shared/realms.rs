pub mod realm_list;

use std::net::IpAddr;

use azothacore_common::AccountTypes;
use flagset::{flags, FlagSet};
use num::FromPrimitive;
use num_derive::FromPrimitive;
use thiserror::Error;

/// Type of server, this is values from second column of Cfg_Configs.dbc
#[derive(Debug, FromPrimitive)]
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
    pub external_address:       IpAddr,
    pub local_address:          IpAddr,
    pub local_subnet_mask:      IpAddr,
    pub port:                   u16,
    pub realm_type:             RealmType,
    pub name:                   String,
    pub flag:                   FlagSet<RealmFlags>,
    pub timezone:               u8,
    pub allowed_security_level: AccountTypes,
    pub population_level:       f64,
}

impl Realm {
    pub fn get_address_for_client(&self) {
        todo!("NOT IMPL, check Realm::GetAddressForClient");
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
