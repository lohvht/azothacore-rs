use std::{io, path::Path};

use flagset::FlagSet;
use thiserror::Error;

use crate::common::collision::models::ModelIgnoreFlags;
// #[derive(Error, Debug, Clone)]
// #[error("locale string error: got {got}")]
// pub struct LocaleParseError {
//     got: String,
// }

// #[derive(Error, Debug)]
// pub enum ConfigError {
//     #[error("encountered unexpected error accessing file")]
//     UnexpectedOSError(#[from] io::Error),
//     #[error("generic error: {msg}")]
//     Generic { msg: String },
// }

pub mod mmap_mgr;
pub mod vmap_mgr2;

#[derive(Error, Debug)]
pub enum VmapFactoryLoadError {
    #[error("encountered unexpected error when loading from vmap factory: {0}")]
    General(String),
    #[error("vmap is ignored, can be skipped")]
    Ignored,
}

#[derive(Error, Debug)]
pub enum VmapLoadError {
    #[error("VmapLoadError: file not found or error loading it in")]
    FileNotFound(#[from] io::Error),
    #[error("VmapLoadError: version mismatch, got: {got} but expected {expected}")]
    VersionMismatched { got: String, expected: String },
}

pub type VmapFactoryLoadResult<T> = Result<T, VmapFactoryLoadError>;

pub type VmapLoadResult<T> = Result<T, VmapLoadError>;

/// for check
pub const VMAP_INVALID_HEIGHT_CHECK: f32 = -100000.0;
/// real assigned value in unknown height case
pub const VMAP_INVALID_HEIGHT_VALUE: f32 = -200000.0;

/// This is the minimum interface to the VMapMgr.
/// Its the equivalent to IVMapManager / IVMapMgr
pub trait VMapMgrTrait {
    // /// The trick to make downcasting back to concrete type possible
    // fn as_any(&self) -> &dyn Any;

    // /// The trick to make downcasting back to mutable concrete type possible
    // fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Ensures that checks are being passed in
    ///
    /// Enable/disable LOS calculation and model height calculation
    ///
    /// `enable_line_of_sight_calc` should be enabled by default.
    /// If it is enabled in mid game the maps have to loaded manualy
    ///
    /// `enable_height_calc` should be enabled by default.
    /// If it is enabled in mid game the maps have to loaded manually
    fn init_new(&mut self) {
        self.init_new_with_options(true, true)
    }

    fn init_new_with_options(&mut self, enable_line_of_sight_calc: bool, enable_height_calc: bool);
    /// loadMap in TC / ACore
    fn load_map_tile(&self, p_base_path: &Path, p_map_id: u32, x: u16, y: u16) -> VmapFactoryLoadResult<()>;
    /// existsMap in TC / ACore
    fn exists_map_tile(&self, p_base_path: &Path, p_map_id: u32, x: u16, y: u16) -> VmapLoadResult<()>;
    /// unloadMap in TC / ACore
    fn unload_map_tile(&self, p_map_id: u32, x: u16, y: u16);
    fn unload_map(&self, p_map_id: u32);

    #[allow(clippy::too_many_arguments, non_snake_case)]
    // TODO: refactor this to return instead of taking in multiple mutable parmas?
    fn is_in_line_of_sight(
        &self,
        p_map_id: u32,
        x1: f32,
        y1: f32,
        z1: f32,
        x2: f32,
        y2: f32,
        z2: f32,
        ignore_flags: FlagSet<ModelIgnoreFlags>,
    ) -> bool;
    fn get_height(&self, p_map_id: u32, x: f32, y: f32, z: f32, max_search_dist: f32) -> f32;
    /// test if we hit an object. return true if we hit one. rx, ry, rz will hold the hit position or the dest position, if no intersection was found
    /// return a position, that is pReduceDist closer to the origin
    // TODO: refactor this to return instead of taking in multiple mutable parmas?
    #[allow(clippy::too_many_arguments, non_snake_case)]
    fn get_object_hit_pos(
        &self,
        p_map_id: u32,
        x1: f32,
        y1: f32,
        z1: f32,
        x2: f32,
        y2: f32,
        z2: f32,
        rx: &mut f32,
        ry: &mut f32,
        rz: &mut f32,
        p_modify_dist: f32,
    ) -> bool;
    // /**
    // send debug commands
    // */
    // fn process_command(char *pCommand)= 0; -> bool

    fn is_line_of_sight_calc_enabled(&self) -> bool;
    fn is_height_calc_enabled(&self) -> bool;
    fn is_map_loading_enabled(&self) -> bool {
        self.is_line_of_sight_calc_enabled() || self.is_height_calc_enabled()
    }
    // fn getDirFileName(unsigned int pMapId, int x, int y) const =0; -> string
    /// Query world model area info.
    ///
    /// \param z gets adjusted to the ground height for which this are info is valid
    #[allow(clippy::too_many_arguments, non_snake_case)]
    // TODO: refactor this to return instead of taking in multiple mutable parmas?
    fn get_area_info(
        &self,
        p_map_id: u32,
        x: f32,
        y: f32,
        z: &mut f32,
        flags: &mut u32,
        adt_id: &mut u16,
        root_id: &mut u32,
        group_id: &mut u32,
    ) -> bool;

    #[allow(clippy::too_many_arguments, non_snake_case)]
    // TODO: refactor this to return instead of taking in multiple mutable parmas?
    fn get_liquid_level(
        &self,
        p_map_id: u32,
        x: f32,
        y: f32,
        z: f32,
        req_liquid_type: u8,
        level: &mut f32,
        floor: &mut f32,
        typ: &mut u32,
    ) -> bool;
}

//  /**
//  */
//  namespace VMAP
//  {

//      #define

//      //===========================================================
//      class TC_COMMON_API IVMapManager
//      {

//              virtual fn void) { } -> IVMapManager(

//              fn  = 0; -> LoadResult

//      };

//  }
//  #endif
