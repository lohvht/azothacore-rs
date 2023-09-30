use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use flagset::{flags, FlagSet};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;

use crate::tools::extractor_common::casc_handles::CascLocale;

pub mod banner;
pub mod collision;
pub mod configuration;
pub mod utils;

#[derive(Debug, FromPrimitive)]
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
    pub fn to_casc_locales(&self) -> FlagSet<CascLocale> {
        match *self {
            Locale::enUS => CascLocale::Enus | CascLocale::Engb,
            Locale::koKR => CascLocale::Kokr.into(),
            Locale::frFR => CascLocale::Frfr.into(),
            Locale::deDE => CascLocale::Dede.into(),
            Locale::zhCN => CascLocale::Zhcn.into(),
            Locale::zhTW => CascLocale::Zhtw.into(),
            Locale::esES => CascLocale::Eses.into(),
            Locale::esMX => CascLocale::Esmx.into(),
            Locale::ruRU => CascLocale::Ruru.into(),
            Locale::none => CascLocale::None.into(),
            Locale::ptBR => CascLocale::Ptbr | CascLocale::Ptpt,
            Locale::itIT => CascLocale::Itit.into(),
        }
    }

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

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
