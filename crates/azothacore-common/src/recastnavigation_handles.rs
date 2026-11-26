use std::{io, slice};

use flagset::FlagSet;
use num_derive::FromPrimitive;
use recastnavigation_sys::*;
pub use recastnavigation_sys::{DT_NAVMESH_VERSION, DT_POLY_BITS, DT_VERTS_PER_POLYGON, RC_SPAN_HEIGHT_BITS, RC_WALKABLE_AREA};
use thiserror::Error;

use crate::deref_boilerplate;

// ==========
// NOTE: RECAST SECTION
// ==========

#[derive(Debug, Clone)]
pub struct RecastConfig(rcConfig);

deref_boilerplate!(RecastConfig, rcConfig, 0);

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
            let status = dtNavMesh_addTile(self.handle, data, data_size as _, dtTileFlags_DT_TILE_FREE_DATA as _, 0, &mut tile_ref);

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

    pub fn tiles(&self) -> impl Iterator<Item = DetourMeshTile<'_>> {
        let nav = unsafe { &*self.handle };
        (0..unsafe { nav.getMaxTiles() })
            .filter_map(move |i| unsafe { nav.getTile(i).as_ref() })
            .filter(|t| !t.header.is_null())
            .map(DetourMeshTile)
    }

    pub fn poly_vertices(&self, poly_ref: dtPolyRef) -> Option<Vec<[f32; 3]>> {
        let mut tile = std::ptr::null();
        let mut poly = std::ptr::null();
        let status = unsafe { (*self.handle).getTileAndPolyByRef(poly_ref, &mut tile, &mut poly) };
        DetourStatus::from_raw_status(status).wrap_result(()).ok()?;
        let (tile, poly) = unsafe { (DetourMeshTile(&*tile), &*poly) };
        Some(tile.poly_vertices(poly).collect())
    }
}

#[derive(Clone, Copy)]
pub struct DetourMeshTile<'a>(&'a dtMeshTile);

impl<'a> DetourMeshTile<'a> {
    pub fn header(&self) -> &'a dtMeshHeader {
        unsafe { &*self.0.header }
    }

    pub fn polys(&self) -> &'a [dtPoly] {
        unsafe { slice::from_raw_parts(self.0.polys, self.header().polyCount as usize) }
    }

    fn vertex(&self, i: u16) -> [f32; 3] {
        unsafe { *self.0.verts.add(usize::from(i) * 3).cast::<[f32; 3]>() }
    }

    pub fn poly_vertices(&self, poly: &dtPoly) -> impl Iterator<Item = [f32; 3]> {
        poly.verts[..usize::from(poly.vertCount)].iter().map(|&v| self.vertex(v))
    }

    /// The detail triangles of each ground polygon, as drawn by Recast's `duDebugDrawNavMesh`.
    pub fn for_each_detail_triangle(&self, mut f: impl FnMut(&dtPoly, [[f32; 3]; 3])) {
        for (i, poly) in self.polys().iter().enumerate() {
            if poly_type(poly) == dtPolyTypes_DT_POLYTYPE_OFFMESH_CONNECTION {
                continue;
            }
            let detail = unsafe { &*self.0.detailMeshes.add(i) };
            for t in 0..u32::from(detail.triCount) {
                let tri = unsafe { slice::from_raw_parts(self.0.detailTris.add(((detail.triBase + t) * 4) as usize), 3) };
                // Indices below vertCount are the polygon's own vertices, the rest index detailVerts.
                let corner = |k: u8| {
                    if k < poly.vertCount {
                        self.vertex(poly.verts[usize::from(k)])
                    } else {
                        let idx = detail.vertBase as usize + usize::from(k - poly.vertCount);
                        unsafe { *self.0.detailVerts.add(idx * 3).cast::<[f32; 3]>() }
                    }
                };
                f(poly, [corner(tri[0]), corner(tri[1]), corner(tri[2])]);
            }
        }
    }
}

pub fn poly_area(poly: &dtPoly) -> u8 {
    poly.areaAndtype & 0x3f
}

pub fn poly_type(poly: &dtPoly) -> dtPolyTypes {
    dtPolyTypes::from(poly.areaAndtype >> 6)
}

pub struct DetourNavMeshQuery<'a> {
    handle: *mut dtNavMeshQuery,
    _nav:   std::marker::PhantomData<&'a DetourNavMesh>,
}

impl Drop for DetourNavMeshQuery<'_> {
    fn drop(&mut self) {
        unsafe { dtFreeNavMeshQuery(self.handle) };
    }
}

impl<'a> DetourNavMeshQuery<'a> {
    pub fn init(nav: &'a DetourNavMesh, max_nodes: i32) -> Result<(Self, DetourStatus), DetourStatus> {
        let handle = unsafe { dtAllocNavMeshQuery() };
        if handle.is_null() {
            return Err(DetourStatus {
                status:  DetourHighLevelStatus::Failure.into(),
                details: DetourStatusDetails::OutOfMemory.into(),
            });
        }
        let query = Self {
            handle,
            _nav: std::marker::PhantomData,
        };
        let res = unsafe { (*handle).init(nav.handle, max_nodes) };
        DetourStatus::from_raw_status(res).wrap_result(query)
    }

    /// Returns a zero poly ref when no polygon lies within `half_extents` of `center`.
    pub fn find_nearest_poly(
        &self,
        center: &[f32; 3],
        half_extents: &[f32; 3],
        filter: &dtQueryFilter,
    ) -> Result<((dtPolyRef, [f32; 3]), DetourStatus), DetourStatus> {
        let mut poly_ref = 0;
        let mut nearest = [0.0; 3];
        let res = unsafe { (*self.handle).findNearestPoly(center.as_ptr(), half_extents.as_ptr(), filter, &mut poly_ref, nearest.as_mut_ptr()) };
        DetourStatus::from_raw_status(res).wrap_result((poly_ref, nearest))
    }

    pub fn find_path(
        &self,
        start_ref: dtPolyRef,
        end_ref: dtPolyRef,
        start_pos: &[f32; 3],
        end_pos: &[f32; 3],
        filter: &dtQueryFilter,
        max_path: usize,
    ) -> Result<(Vec<dtPolyRef>, DetourStatus), DetourStatus> {
        let mut path = vec![0; max_path];
        let mut count = 0;
        let res = unsafe {
            (*self.handle).findPath(
                start_ref,
                end_ref,
                start_pos.as_ptr(),
                end_pos.as_ptr(),
                filter,
                path.as_mut_ptr(),
                &mut count,
                max_path as i32,
            )
        };
        path.truncate(count as usize);
        DetourStatus::from_raw_status(res).wrap_result(path)
    }

    pub fn find_straight_path(
        &self,
        start_pos: &[f32; 3],
        end_pos: &[f32; 3],
        path: &[dtPolyRef],
        max_points: usize,
    ) -> Result<(Vec<[f32; 3]>, DetourStatus), DetourStatus> {
        let mut points = vec![[0.0; 3]; max_points];
        let mut count = 0;
        let res = unsafe {
            (*self.handle).findStraightPath(
                start_pos.as_ptr(),
                end_pos.as_ptr(),
                path.as_ptr(),
                path.len() as i32,
                points.as_mut_ptr().cast(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut count,
                max_points as i32,
                0,
            )
        };
        points.truncate(count as usize);
        DetourStatus::from_raw_status(res).wrap_result(points)
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

deref_boilerplate!(DetourNavMeshParams, dtNavMeshParams, 0);

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
    tiles_per_map: usize,
    params: &mut dtNavMeshCreateParams,
    w: &mut W,
) -> Result<usize, Box<dyn std::error::Error>> {
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
        )
        .into())
    } else if params.polyCount == 0 || params.polys.is_null() || tiles_per_map * tiles_per_map == (params.polyCount as usize) {
        // we have flat tiles with no actual geometry - don't build those, its useless
        // keep in mind that we do output those into debug info
        // drop tiles with only exact count - some tiles may have geometry while having less tiles
        Err(format!(
            "No polygons to build on tile. params.polyCount={}, params.polys.is_null() = {}",
            params.polyCount,
            params.polys.is_null()
        )
        .into())
    } else {
        unsafe {
            let mut data: *mut u8 = std::ptr::null_mut();
            let mut n = 0;

            let res = dtCreateNavMeshData(params, &mut data, &mut n);
            if res {
                let out = slice::from_raw_parts_mut(data, n as usize);
                w.write_all(out)?;
                Ok(n as usize)
            } else {
                Err("Failed building navmesh tile!".into())
            }
        }
    }
}
