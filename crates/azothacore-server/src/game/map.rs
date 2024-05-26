use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

pub use azothacore_common::{AzResult, MapLiquidTypeFlag};
use flagset::FlagSet;
use nalgebra::DMatrix;
use num::Num;

use super::grid::grid_defines::ADT_CELLS_PER_GRID;

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
