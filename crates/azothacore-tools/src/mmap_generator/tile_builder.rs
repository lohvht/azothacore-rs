use std::{path::PathBuf, sync::Arc};

use azothacore_common::{
    az_error,
    collision::{
        management::vmap_mgr2::VMapMgr2,
        maps::map_defines::{MmapNavTerrainFlag, MmapTileFile},
    },
    recastnavigation_handles::{detour_create_nav_mesh_data, DetourNavMesh, DetourNavMeshParams, RecastConfig},
    AzResult,
};
use recastnavigation_sys::{
    dtNavMeshCreateParams,
    rcAllocCompactHeightfield,
    rcAllocContourSet,
    rcAllocHeightfield,
    rcAllocPolyMesh,
    rcAllocPolyMeshDetail,
    rcBuildCompactHeightfield,
    rcBuildContours,
    rcBuildDistanceField,
    rcBuildPolyMesh,
    rcBuildPolyMeshDetail,
    rcBuildRegions,
    rcCalcGridSize,
    rcClearUnwalkableTriangles,
    rcCreateHeightfield,
    rcErodeWalkableArea,
    rcFilterLedgeSpans,
    rcFilterLowHangingWalkableObstacles,
    rcFilterWalkableLowHeightSpans,
    rcFreeCompactHeightfield,
    rcFreeContourSet,
    rcFreeHeightField,
    rcFreePolyMesh,
    rcFreePolyMeshDetail,
    rcMergePolyMeshDetails,
    rcMergePolyMeshes,
    rcRasterizeTriangles,
    CreateContext,
    DT_VERTS_PER_POLYGON,
    RC_WALKABLE_AREA,
};
use tracing::{error, info, instrument, warn};

use crate::mmap_generator::{
    common::{clean_vertices, get_tile_bounds, load_off_mesh_connections, MeshData, GRID_SIZE},
    intermediate_values::IntermediateValues,
    terrain_builder::TerrainBuilder,
};

#[derive(Clone)]
pub struct TileBuilderParams {
    pub mmap_output_path:       PathBuf,
    pub skip_liquid:            bool,
    pub debug_mesh_output_path: Option<PathBuf>,
    pub off_mesh_file_path:     Option<PathBuf>,
    pub vmaps_path:             PathBuf,
    pub maps_path:              PathBuf,
    pub use_min_height:         f32,
    pub big_base_unit:          bool,
    pub max_walkable_angle:     f32,
}

impl TileBuilderParams {
    pub fn into_builder<'vm>(self, vmap_mgr: Arc<VMapMgr2<'vm, 'vm>>) -> (TileBuilder, TerrainBuilder<'vm>) {
        let TileBuilderParams {
            mmap_output_path,
            skip_liquid,
            debug_mesh_output_path,
            vmaps_path,
            maps_path,
            use_min_height,
            off_mesh_file_path,
            big_base_unit,
            max_walkable_angle,
        } = self;
        let terrain_builder = TerrainBuilder {
            vmap_mgr,
            skip_liquid,
            vmaps_path,
            maps_path,
            use_min_height,
        };
        let tile_builder = TileBuilder {
            skip_liquid,
            off_mesh_file_path,
            mmap_output_path,
            debug_mesh_output_path,
            big_base_unit,
            max_walkable_angle,
        };
        (tile_builder, terrain_builder)
    }
}

pub struct TileBuilder {
    pub skip_liquid:            bool,
    pub off_mesh_file_path:     Option<PathBuf>,
    pub big_base_unit:          bool,
    pub max_walkable_angle:     f32,
    pub mmap_output_path:       PathBuf,
    pub debug_mesh_output_path: Option<PathBuf>,
}

impl TileBuilder {
    #[instrument(skip_all, fields(tile = format!("[Map {map_id:04}] [{tile_x:02},{tile_y:02}]")))]
    pub fn build_tile(
        &self,
        terrain_builder: &TerrainBuilder,
        map_id: u32,
        tile_x: u16,
        tile_y: u16,
        nav_mesh_params: &DetourNavMeshParams,
    ) -> AzResult<()> {
        info!("Start building tile");
        // get heightmap data
        let mut mesh_data = MeshData::default();
        terrain_builder.load_map(map_id, tile_x, tile_y, &mut mesh_data)?;

        // get model data
        terrain_builder.load_vmap(map_id, tile_y, tile_x, &mut mesh_data)?;

        // if there is no data, give up now
        if mesh_data.solid_verts.is_empty() && mesh_data.liquid_verts.is_empty() {
            warn!("No vertices found");
            return Ok(());
        }

        // remove unused vertices
        clean_vertices(&mut mesh_data.solid_verts, &mut mesh_data.solid_tris);
        clean_vertices(&mut mesh_data.liquid_verts, &mut mesh_data.liquid_tris);

        // gather all mesh data for final data check, and bounds calculation
        let mut all_verts = Vec::with_capacity(mesh_data.solid_verts.len() + mesh_data.liquid_verts.len());
        all_verts.extend_from_slice(&mesh_data.liquid_verts);
        all_verts.extend_from_slice(&mesh_data.solid_verts);

        // get bounds of current tile
        let mut bmin = [0.0; 3];
        let mut bmax = [0.0; 3];
        get_tile_bounds(tile_x, tile_y, &all_verts, &mut bmin, &mut bmax);

        load_off_mesh_connections(map_id, tile_x, tile_y, self.off_mesh_file_path.as_ref(), &mut mesh_data)?;

        // build navmesh tile
        self.build_move_map_tile(map_id, tile_x, tile_y, &mesh_data, &bmin, &bmax, nav_mesh_params)?;
        Ok(())
    }

    #[allow(non_snake_case)]
    #[expect(clippy::too_many_arguments)]
    #[instrument(skip_all, fields(tile = format!("[Map {map_id:04}] [{tile_x:02},{tile_y:02}]")))]
    pub fn build_move_map_tile(
        &self,
        map_id: u32,
        tile_x: u16,
        tile_y: u16,
        mesh_data: &MeshData,
        bmin: &[f32; 3],
        bmax: &[f32; 3],
        nav_mesh_params: &DetourNavMeshParams,
    ) -> AzResult<()> {
        let (mut nav_mesh, _) = match DetourNavMesh::init(nav_mesh_params) {
            Err(e) => {
                error!("[Map {map_id:04}] Failed creating navmesh for tile {tile_x:02},{tile_y:02}! err = {e}");
                // ignore for now.
                return Ok(());
            },
            Ok(n) => n,
        };
        let rc_context = unsafe { CreateContext(true) };
        if rc_context.is_null() {
            return Err(az_error!("error initialising recast context"));
        }
        // console output
        info!("Building movemap tiles...");

        let MeshData {
            solid_verts,
            solid_tris,
            liquid_verts,
            liquid_tris,
            liquid_types,
            offset_mesh_connections,
            offset_mesh_connection_rads,
            offset_mesh_connection_dirs,
            offset_mesh_connections_areas,
            offset_mesh_connections_flags,
        } = mesh_data;

        let offset_mesh_connections = offset_mesh_connections.iter().flatten().copied().collect::<Vec<_>>();
        let t_verts = solid_verts;
        let t_vert_count = solid_verts.len() / 3;
        let t_tris = solid_tris;
        let t_tri_count = solid_tris.len() / 3;
        let l_verts = liquid_verts;
        let l_vert_count = liquid_verts.len() / 3;
        let l_tris = liquid_tris;
        let l_tri_count = liquid_tris.len() / 3;
        let l_tri_flags = liquid_types;

        // these are WORLD UNIT based metrics
        // this are basic unit dimentions
        // value have to divide GRID_SIZE(533.3333f) ( aka: 0.5333, 0.2666, 0.3333, 0.1333, etc )
        let BASE_UNIT_DIM: f32 = if self.big_base_unit {
            GRID_SIZE / 1000.0
        } else {
            GRID_SIZE / 1000.0 / 2.0
        };

        // All are in UNIT metrics!
        let VERTEX_PER_MAP: usize = (GRID_SIZE / BASE_UNIT_DIM + 0.5) as usize;
        let VERTEX_PER_TILE: usize = if self.big_base_unit { 40 } else { 80 }; // must divide VERTEX_PER_MAP
        let TILES_PER_MAP: usize = VERTEX_PER_MAP / VERTEX_PER_TILE;

        let mut config = RecastConfig::default();
        config.bmin = *bmin;
        config.bmax = *bmax;

        config.maxVertsPerPoly = DT_VERTS_PER_POLYGON;
        config.cs = BASE_UNIT_DIM;
        config.ch = BASE_UNIT_DIM;
        config.walkableSlopeAngle = self.max_walkable_angle;
        config.tileSize = VERTEX_PER_TILE as _;
        config.walkableRadius = if self.big_base_unit { 1 } else { 2 };
        config.borderSize = config.walkableRadius + 3;
        config.maxEdgeLen = (VERTEX_PER_TILE + 1) as _; // anything bigger than tileSize
        config.walkableHeight = if self.big_base_unit { 3 } else { 6 };
        // a value >= 3|6 allows npcs to walk over some fences
        // a value >= 4|8 allows npcs to walk over all fences
        config.walkableClimb = if self.big_base_unit { 4 } else { 8 };
        config.minRegionArea = 60i32.pow(2);
        config.mergeRegionArea = 50i32.pow(2);
        // eliminates most jagged edges (tiny polygons)
        config.maxSimplificationError = 1.8;
        config.detailSampleDist = config.cs * 64.0;
        config.detailSampleMaxError = config.ch * 2.0;

        // this sets the dimensions of the heightfield - should maybe happen before border padding
        unsafe {
            rcCalcGridSize(
                config.bmin.as_ptr(),
                config.bmax.as_ptr(),
                config.cs,
                &mut config.width,
                &mut config.height,
            )
        }

        // // allocate subregions : tiles
        // let mut tiles = (0..TILES_PER_MAP * TILES_PER_MAP).map(|_| None).collect::<Vec<Option<_>>>();

        // Initialize per tile config.
        let mut tile_cfg = config.clone();
        tile_cfg.width = config.tileSize + config.borderSize * 2;
        tile_cfg.height = config.tileSize + config.borderSize * 2;

        // // merge per tile poly and detail meshes
        let mut pmmerge = Vec::with_capacity(TILES_PER_MAP * TILES_PER_MAP);
        let mut dmmerge = Vec::with_capacity(TILES_PER_MAP * TILES_PER_MAP);
        // build all tiles
        for y in 0..TILES_PER_MAP as i32 {
            for x in 0..TILES_PER_MAP as i32 {
                //         Tile& tile = tiles[x + y * TILES_PER_MAP];

                // Calculate the per tile bounding box.
                tile_cfg.bmin[0] = config.bmin[0] + (x * config.tileSize - config.borderSize) as f32 * config.cs;
                tile_cfg.bmin[2] = config.bmin[2] + (y * config.tileSize - config.borderSize) as f32 * config.cs;
                tile_cfg.bmax[0] = config.bmin[0] + ((x + 1) * config.tileSize + config.borderSize) as f32 * config.cs;
                tile_cfg.bmax[2] = config.bmin[2] + ((y + 1) * config.tileSize + config.borderSize) as f32 * config.cs;

                // build heightfield
                let tile_solid = unsafe { rcAllocHeightfield() };
                if tile_solid.is_null()
                    || !unsafe {
                        rcCreateHeightfield(
                            rc_context,
                            tile_solid,
                            tile_cfg.width,
                            tile_cfg.height,
                            tile_cfg.bmin.as_ptr(),
                            tile_cfg.bmax.as_ptr(),
                            tile_cfg.cs,
                            tile_cfg.ch,
                        )
                    }
                {
                    warn!("%s Failed building heightfield!");
                    continue;
                }

                // mark all walkable tiles, both liquids and solids
                let mut tri_flags = vec![MmapNavTerrainFlag::Ground.area_id(); t_tri_count];
                unsafe {
                    rcClearUnwalkableTriangles(
                        rc_context,
                        tile_cfg.walkableSlopeAngle,
                        t_verts.as_ptr(),
                        t_vert_count as i32,
                        t_tris.as_ptr(),
                        t_tri_count as i32,
                        tri_flags.as_mut_ptr(),
                    )
                };
                unsafe {
                    rcRasterizeTriangles(
                        rc_context,
                        t_verts.as_ptr(),
                        t_vert_count as i32,
                        t_tris.as_ptr(),
                        tri_flags.as_ptr(),
                        t_tri_count as i32,
                        tile_solid,
                        config.walkableClimb,
                    )
                };

                unsafe { rcFilterLowHangingWalkableObstacles(rc_context, config.walkableClimb, tile_solid) };
                unsafe { rcFilterLedgeSpans(rc_context, tile_cfg.walkableHeight, tile_cfg.walkableClimb, tile_solid) };
                unsafe { rcFilterWalkableLowHeightSpans(rc_context, tile_cfg.walkableHeight, tile_solid) };

                unsafe {
                    rcRasterizeTriangles(
                        rc_context,
                        l_verts.as_ptr(),
                        l_vert_count as i32,
                        l_tris.as_ptr(),
                        l_tri_flags.as_ptr(),
                        l_tri_count as i32,
                        tile_solid,
                        config.walkableClimb,
                    )
                };

                // compact heightfield spans
                let tile_chf = unsafe { rcAllocCompactHeightfield() };
                if tile_chf.is_null()
                    || !unsafe {
                        rcBuildCompactHeightfield(rc_context, tile_cfg.walkableHeight, tile_cfg.walkableClimb, tile_solid, tile_chf)
                    }
                {
                    warn!("Failed compacting heightfield!");
                    continue;
                }

                // build polymesh intermediates
                if !unsafe { rcErodeWalkableArea(rc_context, config.walkableRadius, tile_chf) } {
                    warn!("Failed eroding area!");
                    continue;
                }

                if !unsafe { rcBuildDistanceField(rc_context, tile_chf) } {
                    warn!("Failed building distance field!");
                    continue;
                }

                if !unsafe {
                    rcBuildRegions(
                        rc_context,
                        tile_chf,
                        tile_cfg.borderSize,
                        tile_cfg.minRegionArea,
                        tile_cfg.mergeRegionArea,
                    )
                } {
                    warn!("Failed building regions!");
                    continue;
                }

                let tile_cset = unsafe { rcAllocContourSet() };
                if tile_cset.is_null()
                    || !unsafe {
                        rcBuildContours(
                            rc_context,
                            tile_chf,
                            tile_cfg.maxSimplificationError,
                            tile_cfg.maxEdgeLen,
                            tile_cset,
                            1,
                        )
                    }
                {
                    warn!("Failed building contours!");
                    continue;
                };

                // build polymesh
                let tile_pmesh = unsafe { rcAllocPolyMesh() };
                if tile_pmesh.is_null() || !unsafe { rcBuildPolyMesh(rc_context, tile_cset, tile_cfg.maxVertsPerPoly, tile_pmesh) } {
                    warn!("Failed building polymesh!");
                    continue;
                }

                let tile_dmesh = unsafe { rcAllocPolyMeshDetail() };
                if tile_dmesh.is_null()
                    || !unsafe {
                        rcBuildPolyMeshDetail(
                            rc_context,
                            tile_pmesh,
                            tile_chf,
                            tile_cfg.detailSampleDist,
                            tile_cfg.detailSampleMaxError,
                            tile_dmesh,
                        )
                    }
                {
                    warn!("Failed building polymesh detail");
                    continue;
                }
                // free those up
                // we may want to keep them in the future for debug
                // but right now, we don't have the code to merge them
                unsafe { rcFreeHeightField(tile_solid) };
                unsafe { rcFreeCompactHeightfield(tile_chf) };
                unsafe { rcFreeContourSet(tile_cset) };

                pmmerge.push(tile_pmesh);
                dmmerge.push(tile_dmesh);
            }
        }
        let iv_poly_mesh = unsafe { rcAllocPolyMesh() };
        if iv_poly_mesh.is_null() {
            return Err(az_error!("alloc iv.polyMesh FAILED!"));
        }
        unsafe { rcMergePolyMeshes(rc_context, pmmerge.as_mut_ptr(), pmmerge.len() as i32, iv_poly_mesh) };

        let iv_poly_mesh_detail = unsafe { rcAllocPolyMeshDetail() };
        if iv_poly_mesh_detail.is_null() {
            return Err(az_error!("alloc m_dmesh FAILED!"));
        }
        unsafe { rcMergePolyMeshDetails(rc_context, dmmerge.as_mut_ptr(), dmmerge.len() as i32, iv_poly_mesh_detail) };

        // free things up
        for m in pmmerge {
            unsafe {
                rcFreePolyMesh(m);
            }
        }
        for m in dmmerge {
            unsafe {
                rcFreePolyMeshDetail(m);
            }
        }
        // delete[] tiles;

        // set polygons as walkable
        // TODO: special flags for DYNAMIC polygons, ie surfaces that can be turned on and off
        let iv_poly_mesh_ref = unsafe { &mut *iv_poly_mesh };
        let iv_poly_mesh_detail_ref = unsafe { &mut *iv_poly_mesh_detail };

        for i in 0..iv_poly_mesh_ref.npolys as usize {
            let raw_area_id = unsafe { *iv_poly_mesh_ref.areas.add(i) } & RC_WALKABLE_AREA;
            let area = MmapNavTerrainFlag::from_area_id(raw_area_id);

            if raw_area_id > 0 {
                let flag = if area.is_empty() {
                    // Flags that arent known yet, so empty even though raw_area_id > 0
                    // TODO: these will be dynamic in future
                    MmapNavTerrainFlag::Ground.flags().bits()
                } else {
                    area.bits()
                };
                unsafe { *iv_poly_mesh_ref.flags.add(i) = flag };
            };
        }

        // setup mesh parameters
        let mut params = dtNavMeshCreateParams {
            verts:            iv_poly_mesh_ref.verts,
            vertCount:        iv_poly_mesh_ref.nverts,
            polys:            iv_poly_mesh_ref.polys,
            polyAreas:        iv_poly_mesh_ref.areas,
            polyFlags:        iv_poly_mesh_ref.flags,
            polyCount:        iv_poly_mesh_ref.npolys,
            nvp:              iv_poly_mesh_ref.nvp,
            detailMeshes:     iv_poly_mesh_detail_ref.meshes,
            detailVerts:      iv_poly_mesh_detail_ref.verts,
            detailVertsCount: iv_poly_mesh_detail_ref.nverts,
            detailTris:       iv_poly_mesh_detail_ref.tris,
            detailTriCount:   iv_poly_mesh_detail_ref.ntris,
            offMeshConVerts:  offset_mesh_connections.as_ptr(),
            offMeshConCount:  offset_mesh_connections.len() as i32 / 6,
            offMeshConRad:    offset_mesh_connection_rads.as_ptr(),
            offMeshConDir:    offset_mesh_connection_dirs.as_ptr(),
            offMeshConAreas:  offset_mesh_connections_areas.as_ptr(),
            offMeshConFlags:  offset_mesh_connections_flags.as_ptr(),
            walkableHeight:   BASE_UNIT_DIM * config.walkableHeight as f32, // agent height
            walkableRadius:   BASE_UNIT_DIM * config.walkableRadius as f32, // agent radius
            walkableClimb:    BASE_UNIT_DIM * config.walkableClimb as f32,  // keep less that walkableHeight (aka agent height)!
            tileX:            ((((bmin[0] + bmax[0]) / 2.0) - nav_mesh.get_params().orig[0]) / GRID_SIZE) as i32,
            tileY:            ((((bmin[2] + bmax[2]) / 2.0) - nav_mesh.get_params().orig[2]) / GRID_SIZE) as i32,
            bmin:             *bmin,
            bmax:             *bmax,
            cs:               config.cs,
            ch:               config.ch,
            tileLayer:        0,
            buildBvTree:      true,
            offMeshConUserID: std::ptr::null(),
            userId:           0,
        };

        #[expect(clippy::never_loop, reason = "loop mechanism is used as a goto")]
        loop {
            // will hold final navmesh
            let mut nav_data = vec![];
            info!("Building navmesh tile...");
            if let Err(e) = detour_create_nav_mesh_data(&mut params, &mut nav_data) {
                warn!(e);
                break;
            }
            let nav_data_copy = nav_data.clone();

            info!("Adding tile to navmesh...");

            let tile_ref = match nav_mesh.add_tile(nav_data) {
                Err(e) => {
                    warn!("failed to add tile to navmesh! {e}");
                    break;
                },
                Ok((r, ..)) => {
                    if r == 0 {
                        warn!("navmesh add tile potential error as resultant tile_ref is zero");
                        break;
                    }
                    r
                },
            };

            // write header
            // write data
            let skip_liquid = self.skip_liquid;
            let mmtilefile = MmapTileFile::new(!skip_liquid, nav_data_copy);
            if let Err(e) = mmtilefile.write_to_mmtile(self.mmap_output_path.clone(), map_id, tile_y, tile_x) {
                error!("error writing to mmtile file: {e}");
            }

            // now that tile is written to disk, we can unload it
            if let Err(e2) = nav_mesh.remove_tile(tile_ref) {
                warn!("[Map {map_id:04}] unable to free tile on cleanup! err: {e2}");
            }

            break;
        }

        if let Some(p) = &self.debug_mesh_output_path {
            // restore padding so that the debug visualization is correct
            for i in 0..iv_poly_mesh_ref.nverts as usize {
                let v = iv_poly_mesh_ref.verts.wrapping_add(i * 3);
                unsafe {
                    *v.wrapping_add(0) += config.borderSize as u16;
                    *v.wrapping_add(2) += config.borderSize as u16;
                }
            }

            let iv = IntermediateValues {
                heightfield:         None,
                compact_heightfield: None,
                contours:            None,
                poly_mesh:           Some(iv_poly_mesh_ref),
                poly_mesh_detail:    Some(iv_poly_mesh_detail_ref),
            };
            iv.generate_obj_file(p, map_id, tile_x, tile_y, mesh_data);
            iv.write_iv(p, map_id, tile_x, tile_y);
        }
        unsafe {
            rcFreePolyMesh(iv_poly_mesh);
            rcFreePolyMeshDetail(iv_poly_mesh_detail);
            libc::malloc_trim(0);
        }
        Ok(())
    }
}
