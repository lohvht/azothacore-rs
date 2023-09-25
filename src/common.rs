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
    pub enum Locale: u32 {
        EnUs = 0,
        KoKr = 1,
        FrFr = 2,
        DeDe = 3,
        ZhCn = 4,
        ZhTw = 5,
        EsEs = 6,
        EsMx = 7,
        RuRu = 8,
        None = 9,
        PtBr = 10,
        ItIt = 11,
    }
}

impl Locale {
    pub fn to_casc_locales(&self) -> FlagSet<CascLocale> {
        match *self {
            Locale::EnUs => CascLocale::Enus | CascLocale::Engb,
            Locale::KoKr => CascLocale::Kokr.into(),
            Locale::FrFr => CascLocale::Frfr.into(),
            Locale::DeDe => CascLocale::Dede.into(),
            Locale::ZhCn => CascLocale::Zhcn.into(),
            Locale::ZhTw => CascLocale::Zhtw.into(),
            Locale::EsEs => CascLocale::Eses.into(),
            Locale::EsMx => CascLocale::Esmx.into(),
            Locale::RuRu => CascLocale::Ruru.into(),
            Locale::None => CascLocale::None.into(),
            Locale::PtBr => CascLocale::Ptbr | CascLocale::Ptpt,
            Locale::ItIt => CascLocale::Itit.into(),
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
            "enUS" => Ok(Locale::EnUs),
            "koKR" => Ok(Locale::KoKr),
            "frFR" => Ok(Locale::FrFr),
            "deDE" => Ok(Locale::DeDe),
            "zhCN" => Ok(Locale::ZhCn),
            "zhTW" => Ok(Locale::ZhTw),
            "esES" => Ok(Locale::EsEs),
            "esMX" => Ok(Locale::EsMx),
            "ruRU" => Ok(Locale::RuRu),
            "none" => Ok(Locale::None),
            "ptBR" => Ok(Locale::PtBr),
            "itIT" => Ok(Locale::ItIt),
            _ => Err(LocaleParseError { got: s.to_string() }),
        }
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
