#![feature(lint_reasons)]

pub mod banner;
pub mod collision;
pub mod compile_options;
pub mod configuration;
pub mod g3dlite_copied;
pub mod log;
pub mod macros;
pub mod recastnavigation_handles;
pub mod utils;

pub type AzResult<T> = anyhow::Result<T>;
pub type AzError = anyhow::Error;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

pub use anyhow::{anyhow as az_error, Context as AzContext};
pub use compile_options::*;
use flagset::{flags, FlagSet};
use num::FromPrimitive;
use num_derive::FromPrimitive;
use thiserror::Error;
use tracing::warn;

#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum AccountTypes {
    SecPlayer = 0,
    SecModerator = 1,
    SecGamemaster = 2,
    SecAdministrator = 3,
    /// must be always last in list, accounts must have less security level always also
    SecConsole = 4,
}

#[derive(Error, Debug, Clone)]
#[error("parse account types error: got {got}")]
pub struct AccountTypesParseError {
    got: u8,
}

impl TryFrom<u8> for AccountTypes {
    type Error = AccountTypesParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(AccountTypesParseError { got: value })
    }
}

flags! {
    #[allow(non_camel_case_types)]
    pub enum Locale: u32 {
        enUS = 0,
        koKR = 1,
        frFR = 2,
        deDE = 3,
        zhCN = 4,
        zhTW = 5,
        esES = 6,
        esMX = 7,
        ruRU = 8,
        none = 9,
        ptBR = 10,
        itIT = 11,
    }
}

impl Locale {
    pub fn to_flagset(self) -> FlagSet<Locale> {
        self.into()
    }
}

#[derive(Error, Debug, Clone)]
#[error("locale string error: got {got}")]
pub struct LocaleParseError {
    got: String,
}

impl FromStr for Locale {
    type Err = LocaleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enUS" => Ok(Locale::enUS),
            "koKR" => Ok(Locale::koKR),
            "frFR" => Ok(Locale::frFR),
            "deDE" => Ok(Locale::deDE),
            "zhCN" => Ok(Locale::zhCN),
            "zhTW" => Ok(Locale::zhTW),
            "esES" => Ok(Locale::esES),
            "esMX" => Ok(Locale::esMX),
            "ruRU" => Ok(Locale::ruRU),
            "none" => Ok(Locale::none),
            "ptBR" => Ok(Locale::ptBR),
            "itIT" => Ok(Locale::itIT),
            _ => Err(LocaleParseError { got: s.to_string() }),
        }
    }
}

impl Locale {
    /// GetLocaleByName
    pub fn from_name(name: &str) -> Self {
        // including enGB case
        Self::from_str(name).unwrap_or(Locale::enUS)
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

flags! {
    pub enum MapLiquidTypeFlag: u8 {
        // NoWater =    0x00,
        #[allow(clippy::identity_op)]
        Water =       1 << 0,
        Ocean =       1 << 1,
        Magma =       1 << 2,
        Slime =       1 << 3,
        DarkWater =   1 << 4,
        AllLiquids = (MapLiquidTypeFlag::Water | MapLiquidTypeFlag::Ocean | MapLiquidTypeFlag::Magma | MapLiquidTypeFlag::Slime).bits(),
      }
}

impl MapLiquidTypeFlag {
    pub fn from_liquid_type_sound_bank_unchecked(sound_bank: u8) -> FlagSet<Self> {
        Self::from_liquid_type_sound_bank(sound_bank)
            .map_err(|e| {
                warn!("{e}: sound_bank value was: {sound_bank}");
                e
            })
            .unwrap_or_default()
    }

    pub fn from_liquid_type_sound_bank(sound_bank: u8) -> AzResult<FlagSet<Self>> {
        FlagSet::new(1u8 << sound_bank).map_err(|e| az_error!("invalid bits: {}", e))
    }
}
