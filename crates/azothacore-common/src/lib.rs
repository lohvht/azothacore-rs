pub mod banner;
pub mod bevy_app;
pub mod bounded_nums;
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
use std::{collections::HashMap, fmt::Debug};

pub use anyhow::{anyhow as az_error, Context as AzContext};
use bevy::prelude::Resource;
pub use compile_options::*;
use flagset::{flags, FlagSet};
pub use hex_fmt::HexFmt;
use num::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;
use tracing::warn;

#[derive(Copy, Clone, serde::Deserialize, serde::Serialize, Debug, ToPrimitive, FromPrimitive, PartialEq, PartialOrd, Ord, Eq)]
pub enum AccountTypes {
    SecPlayer = 0,
    SecModerator = 1,
    SecGamemaster = 2,
    SecAdministrator = 3,
    /// must be always last in list, accounts must have less security level always also
    SecConsole = 4,
}

impl AccountTypes {
    pub fn to_num(&self) -> u8 {
        self.to_u8()
            .unwrap_or_else(|| panic!("account type should never fail to become primitive {self:?}"))
    }

    pub fn is_player_account(&self) -> bool {
        matches!(self, AccountTypes::SecPlayer)
    }

    pub fn is_admin_account(&self) -> bool {
        matches!(self, AccountTypes::SecAdministrator) && matches!(self, AccountTypes::SecConsole)
    }

    pub fn is_console_account(&self) -> bool {
        matches!(self, AccountTypes::SecConsole)
    }
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

pub use wow_db2::Locale;

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

#[derive(Resource)]
pub struct ChildMapData(pub HashMap<u32, Vec<u32>>);

deref_boilerplate!(ChildMapData, HashMap<u32, Vec<u32>>, 0);

#[derive(Resource)]
pub struct ParentMapData(pub HashMap<u32, u32>);
deref_boilerplate!(ParentMapData, HashMap<u32, u32>, 0);
