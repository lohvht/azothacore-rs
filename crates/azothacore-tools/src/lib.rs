#![feature(lint_reasons)]

use azothacore_common::Locale;
use extractor_common::casc_handles::CascLocale;
use flagset::FlagSet;

pub mod adt;
pub mod basic_extractor;
pub mod extractor_common;
pub mod mmap_generator;
pub mod vmap4_assembler;
pub mod vmap4_extractor;
pub mod wdt;

pub fn to_casc_locales(locale: &Locale) -> FlagSet<CascLocale> {
    match *locale {
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
