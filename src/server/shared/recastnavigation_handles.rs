use std::{
    io,
    ops::{Deref, DerefMut},
    slice,
};

use flagset::FlagSet;
use num_derive::FromPrimitive;
use recastnavigation_sys::*;
pub use recastnavigation_sys::{DT_NAVMESH_VERSION, DT_POLY_BITS, DT_VERTS_PER_POLYGON, RC_SPAN_HEIGHT_BITS, RC_WALKABLE_AREA};
use thiserror::Error;

macro_rules! safe_newtype_deref_boilerplate {
    ( $newtype:ident, $deref_type:ident ) => {
        impl Deref for $newtype {
            type Target = $deref_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $newtype {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

// ==========
// NOTE: RECAST SECTION
// ==========

#[derive(Debug, Clone)]
pub struct RecastConfig(rcConfig);

safe_newtype_deref_boilerplate!(RecastConfig, rcConfig);

impl Default for RecastConfig {
    fn default() -> Self {
        Self(rcConfig {
            width:                  0,
            height:                 0,
            tileSize:               0,
            borderSize:             0,
            cs:                     0.0,
            ch:                     0.0,
            bmin:                   [0.0, 0.0, 0.0],
            bmax:                   [0.0, 0.0, 0.0],
            walkableSlopeAngle:     0.0,
            walkableHeight:         0,
            walkableClimb:          0,
            walkableRadius:         0,
            maxEdgeLen:             0,
            maxSimplificationError: 0.0,
            minRegionArea:          0,
            mergeRegionArea:        0,
            maxVertsPerPoly:        0,
            detailSampleDist:       0.0,
            detailSampleMaxError:   0.0,
        })
    }
}

// ==========
// NOTE: DETOUR SECTION
// ==========

#[derive(Debug, Clone, Error)]
#[error("DetourStatus: status {status:?}, details {details:?}")]
pub struct DetourStatus {
    pub status:  FlagSet<DetourHighLevelStatus>,
    pub details: FlagSet<DetourStatusDetails>,
}

impl DetourStatus {
    pub fn from_raw_status(status: dtStatus) -> Self {
        let details = status & DT_STATUS_DETAIL_MASK;
        let details = FlagSet::new_truncated(details);
        let status = FlagSet::new_truncated(status);
        Self { details, status }
    }

    pub fn wrap_result<T>(self, t: T) -> Result<(T, Self), Self> {
        if self.status.contains(DetourHighLevelStatus::Success) {
            Ok((t, self))
        } else {
            Err(self)
        }
    }
}

flagset::flags! {
    #[derive(FromPrimitive)]
    #[repr(u32)]
    pub enum DetourHighLevelStatus: u32 {
        Failure = DT_FAILURE,
        Success = DT_SUCCESS,
        InProgress = DT_IN_PROGRESS,
    }
}

flagset::flags! {
    #[derive(FromPrimitive)]
    #[repr(u32)]
    pub enum DetourStatusDetails: u32 {
        /// Reason: 'Input data is not recognized'
        WrongMagic = DT_WRONG_MAGIC,
        /// Reason: 'Input data is in wrong version'
        WrongVersion = DT_WRONG_VERSION,
        /// Reason: 'Operation ran out of memory'
        OutOfMemory = DT_OUT_OF_MEMORY,
        /// Reason: 'An input parameter was invalid'
        InvalidParam = DT_INVALID_PARAM,
        /// Reason: 'Result buffer for the query was too small to store all results'
        BufferTooSmall = DT_BUFFER_TOO_SMALL,
        /// Reason: 'Query ran out of nodes during search'
        OutOfNodes = DT_OUT_OF_NODES,
        /// Reason: 'Query did not reach the end location, returning best guess'
        PartialResult = DT_PARTIAL_RESULT,
        /// Reason: 'A tile has already been assigned to the given x,y coordinate'
        AlreadyOccupied = DT_ALREADY_OCCUPIED,
    }
}

pub struct DetourNavMesh {
    handle: *mut dtNavMesh,
}

impl Drop for DetourNavMesh {
    fn drop(&mut self) {
        unsafe { dtFreeNavMesh(self.handle) };
    }
}

impl DetourNavMesh {
    pub fn init(params: &DetourNavMeshParams) -> Result<(Self, DetourStatus), DetourStatus> {
        let handle = unsafe { dtAllocNavMesh() };
        if handle.is_null() {
            // If allocate doesnt work, its fair to assume that we cant allocate anymore (i.e. OOM)
            return Err(DetourStatus {
                status:  DetourHighLevelStatus::Failure.into(),
                details: DetourStatusDetails::OutOfMemory.into(),
            });
        }

        let params_ptr = &params.0 as *const _;

        let res = unsafe { dtNavMesh_init(handle, params_ptr) };
        DetourStatus::from_raw_status(res).wrap_result(Self { handle })
    }

    pub fn get_params(&self) -> &dtNavMeshParams {
        unsafe { (*self.handle).getParams().as_ref().unwrap() }
    }

    pub fn add_tile(&mut self, mut input_data: Vec<u8>) -> Result<(dtTileRef, DetourStatus), DetourStatus> {
        input_data.shrink_to_fit();
        let mut boxed_slice = input_data.into_boxed_slice();
        let data = boxed_slice.as_mut_ptr();
        let data_size = boxed_slice.len();

        let res = unsafe {
            let mut tile_ref = 0;
            let status = dtNavMesh_addTile(
                self.handle,
                data,
                data_size as _,
                dtTileFlags_DT_TILE_FREE_DATA as _,
                0,
                &mut tile_ref,
            );

            DetourStatus::from_raw_status(status).wrap_result(tile_ref)
        };
        if res.is_ok() {
            std::mem::forget(boxed_slice);
        }
        res
    }

    pub fn remove_tile(&mut self, tile_ref: dtTileRef) -> Result<((), DetourStatus), DetourStatus> {
        let status = unsafe { dtNavMesh_removeTile(self.handle, tile_ref, std::ptr::null_mut(), std::ptr::null_mut()) };
        DetourStatus::from_raw_status(status).wrap_result(())
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DetourNavMeshParams(#[serde(with = "DtNavMeshParamsDef")] dtNavMeshParams);

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(remote = "dtNavMeshParams")]
#[allow(non_snake_case)]
struct DtNavMeshParamsDef {
    pub orig:       [f32; 3usize],
    pub tileWidth:  f32,
    pub tileHeight: f32,
    pub maxTiles:   ::std::os::raw::c_int,
    pub maxPolys:   ::std::os::raw::c_int,
}

safe_newtype_deref_boilerplate!(DetourNavMeshParams, dtNavMeshParams);

impl From<dtNavMeshParams> for DetourNavMeshParams {
    fn from(value: dtNavMeshParams) -> Self {
        Self(value)
    }
}

impl DetourNavMeshParams {
    pub fn new(orig: &[f32; 3], tile_width: f32, tile_height: f32, max_tiles: i32, max_polys: i32) -> Self {
        Self(dtNavMeshParams {
            orig:       *orig,
            tileWidth:  tile_width,
            tileHeight: tile_height,
            maxTiles:   max_tiles,
            maxPolys:   max_polys,
        })
    }
}

pub fn detour_create_nav_mesh_data<W: io::Write>(
    params: &mut dtNavMeshCreateParams,
    w: &mut W,
) -> Result<usize, Box<dyn std::error::Error>> {
    unsafe {
        let mut data: *mut u8 = std::ptr::null_mut();
        let mut n = 0;

        let res = dtCreateNavMeshData(params, &mut data, &mut n);
        if res {
            let out = slice::from_raw_parts_mut(data, n as usize);
            w.write_all(out)?;
            return Ok(n as usize);
        }
    }
    // Error handling
    // these values are checked within dtCreateNavMeshData - handle them here
    // so we have a clear error message
    if params.nvp > DT_VERTS_PER_POLYGON {
        Err(format!(
            "Invalid verts-per-polygon value! params.nvp was {} which is greater than DT_VERTS_PER_POLYGON {}",
            params.nvp, DT_VERTS_PER_POLYGON
        )
        .into())
    } else if params.vertCount >= 0xffff {
        Err(format!(
            "Too many vertices! params.nvp was {} which is greater than DT_VERTS_PER_POLYGON {}",
            params.vertCount, DT_VERTS_PER_POLYGON
        )
        .into())
    } else if params.vertCount == 0 || params.verts.is_null() {
        // occurs mostly when adjacent tiles have models
        // loaded but those models don't span into this tile
        // message is an annoyance
        Err(format!(
            "No vertices to build tile! params.vertCount={}, params.verts.is_null()={}",
            params.vertCount,
            params.verts.is_null()
        )
        .into())
    } else if params.detailMeshes.is_null() || params.detailVerts.is_null() || params.detailTris.is_null() {
        Err(format!(
            "No detail mesh to build tile! params.detailMeshes.is_null()={}, params.detailVerts.is_null()={}, params.detailTris.is_null()={}",
            params.detailMeshes.is_null(),
            params.detailVerts.is_null(),
            params.detailTris.is_null(),
        ).into())
    } else if params.polyCount == 0 || params.polys.is_null()
    // || TILES_PER_MAP * TILES_PER_MAP == (params.polyCount as usize)
    {
        // we have flat tiles with no actual geometry - don't build those, its useless
        // keep in mind that we do output those into debug info
        // drop tiles with only exact count - some tiles may have geometry while having less tiles
        // TILES_PER_MAP_sq = TILES_PER_MAP * TILES_PER_MAP,
        Err(format!(
            "No polygons to build on tile. params.polyCount={}, params.polys.is_null() = {}",
            params.polyCount,
            params.polys.is_null()
        )
        .into())
    } else {
        Err("Failed building navmesh tile!".into())
    }
}
