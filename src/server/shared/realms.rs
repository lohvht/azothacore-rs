use std::net::IpAddr;

use flagset::{flags, FlagSet};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;

use crate::common::AccountTypes;

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
    pub id:                     u32,
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
