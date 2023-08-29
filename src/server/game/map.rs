use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use flagset::{flags, FlagSet};
use nalgebra::DMatrix;
use num_traits::Num;
use tracing::warn;

use crate::AzResult;

pub const ADT_CELLS_PER_GRID: usize = 16;
pub const ADT_CELL_SIZE: usize = 8;
pub const ADT_GRID_SIZE: usize = ADT_CELLS_PER_GRID * ADT_CELL_SIZE;
pub const ADT_GRID_SIZE_PLUS_ONE: usize = ADT_GRID_SIZE + 1;

pub mod map_file;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MapLiquidData {
    /// header flags are different from liquid_flags
    pub map_liquid_entry_flags: Result<MapLiquidDataEntryFlags, MapLiquidDataGlobalEntryFlags>,
    pub offset_x:               u8,
    pub offset_y:               u8,
    /// height is nrows, width is ncols
    pub liquid_height_details:  Result<DMatrix<f32>, f32>,
}

impl MapLiquidData {
    pub fn get_liquid_entry_flags(&self, x: usize, y: usize) -> (u16, FlagSet<MapLiquidTypeFlag>) {
        match &self.map_liquid_entry_flags {
            Ok(lf) => (lf.liquid_entry[x][y], lf.liquid_flags[x][y]),
            Err(global_lf) => (global_lf.liquid_type, global_lf.liquid_flags),
        }
    }

    pub fn get_liquid_level(&self, x: usize, y: usize) -> f32 {
        match &self.liquid_height_details {
            Ok(liquid_map) => liquid_map[(x, y)],
            Err(global_level) => *global_level,
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MapLiquidDataGlobalEntryFlags {
    pub liquid_flags: FlagSet<MapLiquidTypeFlag>,
    pub liquid_type:  u16,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MapLiquidDataEntryFlags {
    pub liquid_entry: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub liquid_flags: [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
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
            .inspect_err(|e| {
                warn!("{e}: sound_bank value was: {sound_bank}");
            })
            .unwrap_or_default()
    }

    pub fn from_liquid_type_sound_bank(sound_bank: u8) -> AzResult<FlagSet<Self>> {
        Ok(FlagSet::new(1u8 << sound_bank)?)
    }
}

pub struct GridMap {}

impl GridMap {
    pub fn file_name<P, M, X, Y>(maps_dir: P, map_id: M, y: Y, x: X) -> PathBuf
    where
        P: AsRef<Path>,
        M: Num + Display,
        X: Num + Display,
        Y: Num + Display,
    {
        maps_dir.as_ref().join(format!("{map_id:04}_{y:02}_{x:02}.map"))
    }
}
