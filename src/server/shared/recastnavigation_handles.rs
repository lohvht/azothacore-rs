use std::{
    collections::HashMap,
    io,
    ops::{Deref, DerefMut},
    ptr,
    slice,
};

use flagset::FlagSet;
use num_derive::FromPrimitive;
use recastnavigation_sys::*;
pub use recastnavigation_sys::{DT_NAVMESH_VERSION, DT_POLY_BITS, DT_VERTS_PER_POLYGON, RC_SPAN_HEIGHT_BITS, RC_WALKABLE_AREA};
use thiserror::Error;

macro_rules! alloc_body {
    ( $alloc_expr:expr ) => {{
        let result = unsafe {
            let n = $alloc_expr;
            if n.is_null() {
                Err(format!("{} failed to allocate", stringify!($alloc_expr)).into())
            } else {
                Ok(n)
            }
        };
        result.map(|d| Self(d))
    }};
    ( RAW, $alloc_expr:expr ) => {{
        let result = unsafe {
            let n = $alloc_expr;
            if n.is_null() {
                Err(format!("{} failed to allocate", stringify!($alloc_expr)).into())
            } else {
                Ok(n)
            }
        };
        result
    }};
}

macro_rules! unsafe_newtype_deref_drop_boilerplate {
    ( $newtype:ident, $deref_type:ident, $free_fn:ident ) => {
        impl Deref for $newtype {
            type Target = $deref_type;

            fn deref(&self) -> &Self::Target {
                unsafe { &*self.0 }
            }
        }

        impl DerefMut for $newtype {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *self.0 }
            }
        }

        impl Drop for $newtype {
            fn drop(&mut self) {
                unsafe { $free_fn(self.0) };
            }
        }
    };
}

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

macro_rules! alloc_boilerplate {
    ( $vis:vis, $newtype:ident, $alloc_fn:ident, $alloc_method_name:ident, [$($alloc_method_params:ident : $alloc_method_ty:ty),*], ($($alloc_fn_args:expr),*) ) => {
        impl $newtype {
            $vis fn $alloc_method_name(
                $(
                    $alloc_method_params: $alloc_method_ty,
                )*
            ) -> Result<Self, Box<dyn std::error::Error>> {
                alloc_body!($alloc_fn(
                    $(
                        $alloc_method_params,
                    )*
                ))
            }
        }
    };
}

macro_rules! unsafe_newtype_boilerplate {
    ( $vis:vis, $newtype:ident, $deref_type:ident, $free_fn:ident, $alloc_fn:ident ) => {
        $vis struct $newtype(*mut $deref_type);

        alloc_boilerplate!($vis, $newtype, $alloc_fn, alloc, [], ());

        unsafe_newtype_deref_drop_boilerplate!($newtype, $deref_type, $free_fn);
    };
    ( $vis:vis, $newtype:ident, $deref_type:ident, $free_fn:ident, $alloc_fn:ident, $alloc_method_name:ident, [$($alloc_method_params:ident : $alloc_method_ty:ty),*], ($($alloc_fn_args:expr),+) ) => {
        $vis struct $newtype(*mut $deref_type);

        alloc_boilerplate!($vis, $newtype, $alloc_fn, $alloc_method_name, [$($alloc_method_params: $alloc_method_ty),*], ($($alloc_method_params),*));

        unsafe_newtype_deref_drop_boilerplate!($newtype, $deref_type, $free_fn);

    };
    // ( $vis:vis, $newtype:ident, $deref_type:ident, $free_fn:ident, $alloc_fn:ident, $alloc_method_name:ident, [$($alloc_method_params:ident : $alloc_method_ty:ty),*], ($($alloc_fn_args:expr),+) ) => {
    //     $vis struct $newtype(*mut $deref_type);

    //     alloc_boilerplate!($vis, $newtype, $alloc_fn, $alloc_method_name, [$($alloc_method_params: $alloc_method_ty),*], ($($alloc_method_params),*));

    //     unsafe_newtype_deref_drop_boilerplate!($newtype, $deref_type, $free_fn);

    // };
}

// ==========
// NOTE: RECAST SECTION
// ==========

unsafe_newtype_boilerplate!(pub, RecastContext, rcContext, DeleteContext, CreateContext, alloc_new, [state: bool], (state));

impl RecastContext {
    pub fn create_height_field(
        &self,
        width: i32,
        height: i32,
        bmin: &[f32; 3],
        bmax: &[f32; 3],
        cs: f32,
        ch: f32,
    ) -> Result<RecastHeightField, Box<dyn std::error::Error>> {
        let hf = RecastHeightField::alloc()?;

        let create_height_field_result = unsafe { rcCreateHeightfield(self.0, hf.0, width, height, bmin.as_ptr(), bmax.as_ptr(), cs, ch) };
        if create_height_field_result {
            Ok(hf)
        } else {
            Err("Failed building heightfield!".into())
        }
    }

    pub fn build_compact_height_field(
        &self,
        walkable_height: i32,
        walkable_climb: i32,
        heightfield: &RecastHeightField,
    ) -> Result<RecastCompactHeightfield, Box<dyn std::error::Error>> {
        let chf = RecastCompactHeightfield::alloc()?;
        let create_res = unsafe { rcBuildCompactHeightfield(self.0, walkable_height, walkable_climb, heightfield.0, chf.0) };
        if create_res {
            Ok(chf)
        } else {
            Err("Failed compacting heightfield!".into())
        }
    }

    pub fn clear_unwalkable_triangles(
        &self,
        walkable_slope_angle: f32,
        verts_flattened: &[f32],
        tris_flattened: &[i32],
        tri_area_ids: &mut [u8],
    ) {
        unsafe {
            rcClearUnwalkableTriangles(
                self.0,
                walkable_slope_angle,
                verts_flattened.as_ptr(),
                (verts_flattened.len() / 3) as i32,
                tris_flattened.as_ptr(),
                (tris_flattened.len() / 3) as i32,
                tri_area_ids.as_mut_ptr(),
            )
        }
    }

    pub fn rasterize_triangles(
        &self,
        verts_flattened: &[f32],
        tris_flattened: &[i32],
        tri_area_ids: &[u8],
        target_heightfield: &RecastHeightField,
        flag_merge_threshold: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let res = unsafe {
            rcRasterizeTriangles(
                self.0,
                verts_flattened.as_ptr(),
                (verts_flattened.len() / 3) as i32,
                tris_flattened.as_ptr(),
                tri_area_ids.as_ptr(),
                (tris_flattened.len() / 3) as i32,
                target_heightfield.0,
                flag_merge_threshold,
            )
        };
        if res {
            Ok(())
        } else {
            Err("rasterize triangles failed!".into())
        }
    }

    pub fn filter_low_hanging_walkable_obstacles(&self, walkable_climb: i32, target_heightfield: &RecastHeightField) {
        unsafe { rcFilterLowHangingWalkableObstacles(self.0, walkable_climb, target_heightfield.0) }
    }

    pub fn filter_ledge_spans(&self, walkable_height: i32, walkable_climb: i32, target_heightfield: &RecastHeightField) {
        unsafe { rcFilterLedgeSpans(self.0, walkable_height, walkable_climb, target_heightfield.0) }
    }

    pub fn filter_walkable_low_height_spans(&self, walkable_height: i32, target_heightfield: &RecastHeightField) {
        unsafe { rcFilterWalkableLowHeightSpans(self.0, walkable_height, target_heightfield.0) }
    }

    pub fn erode_walkable_area(&self, radius: i32, target_chf: &RecastCompactHeightfield) -> Result<(), Box<dyn std::error::Error>> {
        if unsafe { rcErodeWalkableArea(self.0, radius, target_chf.0) } {
            Ok(())
        } else {
            Err("erode walkable area failed".into())
        }
    }

    pub fn build_distance_field(&self, chf: &RecastCompactHeightfield) -> Result<(), Box<dyn std::error::Error>> {
        if unsafe { rcBuildDistanceField(self.0, chf.0) } {
            Ok(())
        } else {
            Err("build distance field failed".into())
        }
    }

    pub fn build_regions(
        &self,
        chf: &RecastCompactHeightfield,
        border_size: i32,
        min_region_area: i32,
        merge_region_area: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if unsafe { rcBuildRegions(self.0, chf.0, border_size, min_region_area, merge_region_area) } {
            Ok(())
        } else {
            Err("build regions failed".into())
        }
    }

    pub fn build_contours(
        &self,
        chf: &RecastCompactHeightfield,
        max_error: f32,
        max_edge_len: i32,
        build_flags: i32,
    ) -> Result<RecastContourSet, Box<dyn std::error::Error>> {
        let v = RecastContourSet::alloc()?;
        let res = unsafe { rcBuildContours(self.0, chf.0, max_error, max_edge_len, v.0, build_flags) };
        if res {
            Ok(v)
        } else {
            Err("build countours failed".into())
        }
    }

    pub fn build_poly_mesh(&self, csset: &RecastContourSet, nvp: i32) -> Result<RecastPolyMesh, Box<dyn std::error::Error>> {
        let v = RecastPolyMesh::alloc()?;
        let res = unsafe { rcBuildPolyMesh(self.0, csset.0, nvp, v.0) };
        if res {
            Ok(v)
        } else {
            Err("build poly mesh failed".into())
        }
    }

    pub fn build_poly_mesh_detail(
        &self,
        mesh: &RecastPolyMesh,
        chf: &RecastCompactHeightfield,
        sample_dist: f32,
        sample_max_error: f32,
    ) -> Result<RecastPolyMeshDetail, Box<dyn std::error::Error>> {
        let v = RecastPolyMeshDetail::alloc()?;
        let res = unsafe { rcBuildPolyMeshDetail(self.0, mesh.0, chf.0, sample_dist, sample_max_error, v.0) };
        if res {
            Ok(v)
        } else {
            Err("build poly mesh detail failed".into())
        }
    }

    pub fn merge_poly_meshes(&self, meshes: Vec<RecastPolyMesh>, target_mesh: &RecastPolyMesh) -> Result<(), Box<dyn std::error::Error>> {
        let mut meshes = meshes.into_iter().map(|m| m.0).collect::<Vec<_>>();
        if unsafe { rcMergePolyMeshes(self.0, meshes.as_mut_ptr(), meshes.len() as i32, target_mesh.0) } {
            Ok(())
        } else {
            Err("merge poly meshes failed".into())
        }
    }

    pub fn merge_poly_mesh_details(
        &self,
        dmeshes: Vec<RecastPolyMeshDetail>,
        target_dmesh: &RecastPolyMeshDetail,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut dmeshes = dmeshes.into_iter().map(|m| m.0).collect::<Vec<_>>();
        if unsafe { rcMergePolyMeshDetails(self.0, dmeshes.as_mut_ptr(), dmeshes.len() as i32, target_dmesh.0) } {
            Ok(())
        } else {
            Err("merge poly mesh details failed".into())
        }
    }
}

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

unsafe_newtype_boilerplate!(pub, RecastHeightField, rcHeightfield, rcFreeHeightField, rcAllocHeightfield);

pub fn recast_calc_grid_size(min_bounds: &[f32; 3], max_bounds: &[f32; 3], cell_size: f32, size_x: &mut i32, size_z: &mut i32) {
    unsafe { rcCalcGridSize(min_bounds.as_ptr(), max_bounds.as_ptr(), cell_size, size_x, size_z) }
}

unsafe_newtype_boilerplate!(
    pub,
    RecastCompactHeightfield,
    rcCompactHeightfield,
    rcFreeCompactHeightfield,
    rcAllocCompactHeightfield
);

unsafe_newtype_boilerplate!(pub, RecastContourSet, rcContourSet, rcFreeContourSet, rcAllocContourSet);

unsafe_newtype_boilerplate!(pub, RecastPolyMesh, rcPolyMesh, rcFreePolyMesh, rcAllocPolyMesh);

impl RecastPolyMesh {
    /// Safe wrapper to retrieve the area ID assigned to polygon at n, where n is within 0..self.npolys
    pub fn get_area_id(&self, n: usize) -> u8 {
        unsafe { *self.areas.add(n) }
    }

    /// Safe wrapper to set the flag assigned to polygon at n, where n is within 0..self.npolys
    pub fn set_flag(&mut self, n: usize, f: u16) {
        unsafe { *self.flags.add(n) = f }
    }
}

unsafe_newtype_boilerplate!(
    pub,
    RecastPolyMeshDetail,
    rcPolyMeshDetail,
    rcFreePolyMeshDetail,
    rcAllocPolyMeshDetail
);

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
    fn from_raw_status(status: dtStatus) -> Self {
        let details = status & DT_STATUS_DETAIL_MASK;
        let details = FlagSet::new_truncated(details);
        let status = FlagSet::new_truncated(status);
        Self { details, status }
    }

    fn wrap_result<T>(self, t: T) -> Result<(T, Self), Self> {
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

pub struct DetourNavMesh(*mut dtNavMesh, HashMap<dtTileRef, Vec<u8>>);

unsafe_newtype_deref_drop_boilerplate!(DetourNavMesh, dtNavMesh, dtFreeNavMesh);

impl DetourNavMesh {
    pub fn alloc() -> Result<Self, Box<dyn std::error::Error>> {
        alloc_body!(RAW, dtAllocNavMesh()).map(|raw_mesh| Self(raw_mesh, HashMap::new()))
    }

    pub fn init(params: &DetourNavMeshParams) -> Result<(Self, DetourStatus), DetourStatus> {
        Self::alloc()
            .map(|d| {
                let mut d = d;
                let status = unsafe { DetourStatus::from_raw_status(d.init(&**params)) };
                status.wrap_result(d)
            })
            .unwrap_or_else(|_| {
                // If allocate doesnt work, its fair to assume that we cant allocate anymore (i.e. OOM)
                Err(DetourStatus {
                    status:  DetourHighLevelStatus::Failure.into(),
                    details: DetourStatusDetails::OutOfMemory.into(),
                })
            })
    }

    pub fn get_params(&self) -> &dtNavMeshParams {
        unsafe { &*self.getParams() }
    }

    pub fn add_tile(&mut self, mut data: Vec<u8>, last_ref: dtTileRef) -> Result<(u64, DetourStatus), DetourStatus> {
        data.shrink_to_fit();
        let data_size = data.len();
        unsafe {
            // DT_TILE_FREE_DATA tells detour to unallocate memory when the tile
            // is removed via removeTile()
            // In most cases we will not want this because we'll manage memory
            // in this very same newtype wrapper.
            let flags = 0; // dtTileFlags_DT_TILE_FREE_DATA
            let mut tile_ref = 0;
            let status = DetourStatus::from_raw_status(self.addTile(data.as_mut_ptr(), data_size as _, flags, last_ref, &mut tile_ref));

            let r = status.wrap_result(tile_ref);
            if r.is_ok() {
                // insert, for the case of last_ref == tile_ref, this would mean that its popping out lazily too.
                // We just allow this to happen if this is the case as we can manage the old memory lazily this way
                // The function forces any `data` coming in to be owned by this function (and by `self` subsequently if successful)
                self.1.insert(tile_ref, data);
            }
            r
        }
    }

    pub fn remove_tile(&mut self, tile_ref: dtTileRef) -> Result<((), DetourStatus), DetourStatus> {
        unsafe {
            let status = DetourStatus::from_raw_status(self.removeTile(tile_ref, std::ptr::null_mut(), std::ptr::null_mut()));
            let r = status.wrap_result(());
            if r.is_ok() {
                self.1.remove(&tile_ref);
            }
            r
        }
    }

    pub fn get_tile_data(&self, tile_ref: dtTileRef) -> &Vec<u8> {
        self.1
            .get(&tile_ref)
            .expect("expect tile data when retrieved to be available as it was added before on a successful add_tile")
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

#[derive(Debug, Clone)]
pub struct DetourNavMeshCreateParams(dtNavMeshCreateParams);

safe_newtype_deref_boilerplate!(DetourNavMeshCreateParams, dtNavMeshCreateParams);

impl DetourNavMeshCreateParams {
    #[allow(clippy::too_many_arguments)]
    pub fn new_from_recast(
        recast_mesh: &RecastPolyMesh,
        recast_mesh_details: &RecastPolyMeshDetail,
        offset_mesh_connections: &[f32],
        offset_mesh_connection_rads: &[f32],
        offset_mesh_connection_dirs: &[u8],
        offset_mesh_connections_areas: &[u8],
        offset_mesh_connections_flags: &[u16],
        walkable_height: f32,
        walkable_radius: f32,
        walkable_climb: f32,
        tile_x: i32,
        tile_y: i32,
        bmin: &[f32; 3],
        bmax: &[f32; 3],
        cs: f32,
        ch: f32,
        tile_layer: i32,
        build_bv_tree: bool,
    ) -> Self {
        Self(dtNavMeshCreateParams {
            verts: recast_mesh.verts,
            vertCount: recast_mesh.nverts,
            polys: recast_mesh.polys,
            polyAreas: recast_mesh.areas,
            polyFlags: recast_mesh.flags,
            polyCount: recast_mesh.npolys,
            nvp: recast_mesh.nvp,
            detailMeshes: recast_mesh_details.meshes,
            detailVerts: recast_mesh_details.verts,
            detailVertsCount: recast_mesh_details.nverts,
            detailTris: recast_mesh_details.tris,
            detailTriCount: recast_mesh_details.ntris,
            offMeshConVerts: offset_mesh_connections.as_ptr(),
            offMeshConCount: offset_mesh_connections.len() as i32 / 6,
            offMeshConRad: offset_mesh_connection_rads.as_ptr(),
            offMeshConDir: offset_mesh_connection_dirs.as_ptr(),
            offMeshConAreas: offset_mesh_connections_areas.as_ptr(),
            offMeshConFlags: offset_mesh_connections_flags.as_ptr(),
            walkableHeight: walkable_height,
            walkableRadius: walkable_radius,
            walkableClimb: walkable_climb,
            tileX: tile_x,
            tileY: tile_y,
            bmin: *bmin,
            bmax: *bmax,
            cs,
            ch,
            tileLayer: tile_layer,
            buildBvTree: build_bv_tree,
            offMeshConUserID: ptr::null(),
            userId: 0,
        })
    }

    pub fn create_nav_mesh_data<W: io::Write>(&mut self, w: &mut W) -> Result<usize, Box<dyn std::error::Error>> {
        unsafe {
            let data = std::ptr::null_mut();
            let mut n = 0;

            let res = dtCreateNavMeshData(&mut self.0, data, &mut n);
            if res {
                let out = slice::from_raw_parts_mut(*data, n as usize);
                w.write_all(out)?;
                return Ok(n as usize);
            }
        };
        // Error handling
        // these values are checked within dtCreateNavMeshData - handle them here
        // so we have a clear error message
        if self.nvp > DT_VERTS_PER_POLYGON {
            Err(format!(
                "Invalid verts-per-polygon value! self.nvp was {} which is greater than DT_VERTS_PER_POLYGON {}",
                self.nvp, DT_VERTS_PER_POLYGON
            )
            .into())
        } else if self.vertCount >= 0xffff {
            Err(format!(
                "Too many vertices! self.nvp was {} which is greater than DT_VERTS_PER_POLYGON {}",
                self.vertCount, DT_VERTS_PER_POLYGON
            )
            .into())
        } else if self.vertCount == 0 || self.verts.is_null() {
            // occurs mostly when adjacent tiles have models
            // loaded but those models don't span into this tile
            // message is an annoyance
            Err("No vertices to build tile!".into())
        } else if self.detailMeshes.is_null() || self.detailVerts.is_null() || self.detailTris.is_null() {
            Err("No detail mesh to build tile!".into())
        } else {
            Err("Failed building navmesh tile!".into())
        }
    }
}
