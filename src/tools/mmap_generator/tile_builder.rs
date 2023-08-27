use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use parry3d::bounding_volume::Aabb;
use tracing::{error, info, instrument, warn};

use crate::{
    az_error,
    common::collision::{
        management::vmap_mgr2::VMapMgr2,
        maps::map_defines::{MmapNavTerrainFlag, MmapTileFile},
    },
    server::shared::recastnavigation_handles::{
        recast_calc_grid_size,
        DetourNavMesh,
        DetourNavMeshCreateParams,
        RecastConfig,
        RecastContext,
        RecastPolyMesh,
        RecastPolyMeshDetail,
        DT_VERTS_PER_POLYGON,
        RC_WALKABLE_AREA,
    },
    tools::mmap_generator::{
        common::{clean_vertices, get_tile_bounds, load_off_mesh_connections, MeshData, GRID_SIZE},
        intermediate_values::IntermediateValues,
        terrain_builder::TerrainBuilder,
    },
    AzResult,
    AzothaError,
};

#[derive(Clone)]
pub struct TileBuilderParams<'vm> {
    pub mmap_output_path:       PathBuf,
    pub skip_liquid:            bool,
    pub debug_mesh_output_path: Option<PathBuf>,
    pub off_mesh_file_path:     Option<PathBuf>,
    pub vmap_mgr:               Arc<RwLock<VMapMgr2<'vm, 'vm>>>,
    pub vmaps_path:             PathBuf,
    pub maps_path:              PathBuf,
    pub use_min_height:         f32,
    pub big_base_unit:          bool,
    pub max_walkable_angle:     f32,
}

impl<'vm> TileBuilderParams<'vm> {
    pub fn try_to_builder(self) -> AzResult<TileBuilder<'vm>> {
        self.try_into()
    }
}

impl<'vm> TryInto<TileBuilder<'vm>> for TileBuilderParams<'vm> {
    type Error = AzothaError;

    fn try_into(self) -> Result<TileBuilder<'vm>, Self::Error> {
        let TileBuilderParams {
            mmap_output_path,
            skip_liquid,
            debug_mesh_output_path,
            vmap_mgr,
            vmaps_path,
            maps_path,
            use_min_height,
            off_mesh_file_path,
            big_base_unit,
            max_walkable_angle,
        } = self;

        let rc_context = RecastContext::alloc_new(true).map_err(|e| az_error!(e))?;
        Ok(TileBuilder {
            rc_context,
            terrain_builder: TerrainBuilder {
                vmap_mgr,
                vmaps_path,
                maps_path,
                use_min_height,
            },
            skip_liquid,
            off_mesh_file_path,
            mmap_output_path,
            debug_mesh_output_path,
            big_base_unit,
            max_walkable_angle,
        })
    }
}

pub struct TileBuilder<'tb> {
    rc_context:             RecastContext,
    terrain_builder:        TerrainBuilder<'tb>,
    skip_liquid:            bool,
    off_mesh_file_path:     Option<PathBuf>,
    big_base_unit:          bool,
    max_walkable_angle:     f32,
    mmap_output_path:       PathBuf,
    debug_mesh_output_path: Option<PathBuf>,
}

impl TileBuilder<'_> {
    #[instrument(skip_all, fields(tile = format!("[Map {map_id:04}] [{tile_x:02},{tile_y:02}]")))]
    pub fn build_tile(&self, map_id: u32, tile_x: u16, tile_y: u16, nav_mesh: &mut DetourNavMesh) -> AzResult<()> {
        if self.should_skip_tile(map_id, tile_x, tile_y) {
            return Ok(());
        }
        info!("Start building tile");
        // get heightmap data
        let mut mesh_data = MeshData::default();
        self.terrain_builder
            .load_map(map_id, tile_x, tile_y, self.skip_liquid, &mut mesh_data)?;

        // get model data
        self.terrain_builder.load_vmap(map_id, tile_y, tile_x, &mut mesh_data)?;

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
        // float bmin[3], bmax[3];
        let b_max_min = get_tile_bounds(tile_x, tile_y, &all_verts);

        load_off_mesh_connections(map_id, tile_x, tile_y, self.off_mesh_file_path.as_ref(), &mut mesh_data)?;

        // build navmesh tile
        self.build_move_map_tile(map_id, tile_x, tile_y, &mesh_data, &b_max_min, nav_mesh)?;
        Ok(())
    }

    #[instrument(skip_all, fields(tile = format!("[Map {map_id:04}] [{tile_x:02},{tile_y:02}]")))]
    pub fn build_move_map_tile(
        &self,
        map_id: u32,
        tile_x: u16,
        tile_y: u16,
        mesh_data: &MeshData,
        b_max_min: &Aabb,
        nav_mesh: &mut DetourNavMesh,
    ) -> AzResult<()> {
        // console output
        info!("Building movemap tiles...");

        let Aabb { mins: bmin, maxs: bmax } = b_max_min;
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

        let t_verts = solid_verts.iter().flatten().copied().collect::<Vec<_>>();
        let t_tris = solid_tris.iter().flatten().map(|v| *v as i32).collect::<Vec<_>>();
        let l_verts = liquid_verts.iter().flatten().copied().collect::<Vec<_>>();
        let l_tris = liquid_tris.iter().flatten().map(|v| *v as i32).collect::<Vec<_>>();
        let l_tri_flags = liquid_types
            .iter()
            .map(|f| if let Some(f) = f { f.area_id() } else { 0 })
            .collect::<Vec<_>>();

        // these are WORLD UNIT based metrics
        // this are basic unit dimentions
        // value have to divide GRID_SIZE(533.3333f) ( aka: 0.5333, 0.2666, 0.3333, 0.1333, etc )
        let base_unit_dim = if self.big_base_unit {
            GRID_SIZE / 1000.0
        } else {
            GRID_SIZE / 1000.0 / 2.0
        };

        // All are in UNIT metrics!
        let vertex_per_map = ((GRID_SIZE / base_unit_dim + 0.5).floor()) as usize;
        // must divide vertex_per_map
        let vertex_per_tile = if self.big_base_unit { 40 } else { 80 };
        let tiles_per_map = vertex_per_map / vertex_per_tile;

        let mut config = RecastConfig::default();
        config.bmin = b_max_min.mins.into();
        config.bmax = b_max_min.maxs.into();

        config.maxVertsPerPoly = DT_VERTS_PER_POLYGON;
        config.cs = base_unit_dim;
        config.ch = base_unit_dim;
        config.walkableSlopeAngle = self.max_walkable_angle;
        config.tileSize = vertex_per_tile as _;
        config.walkableRadius = if self.big_base_unit { 1 } else { 2 };
        config.borderSize = config.walkableRadius + 3;
        // anything bigger than tileSize
        config.maxEdgeLen = (vertex_per_tile + 1) as _;
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
        let mut width = 0;
        let mut height = 0;
        recast_calc_grid_size(&config.bmin, &config.bmax, config.cs, &mut width, &mut height);
        config.width = width;
        config.height = height;
        // // allocate subregions : tiles
        // let mut tiles = (0..tiles_per_map * tiles_per_map).map(|_| None).collect::<Vec<Option<_>>>();

        // Initialize per tile config.
        let mut tile_cfg = config.clone();
        tile_cfg.width = config.tileSize + config.borderSize * 2;
        tile_cfg.height = config.tileSize + config.borderSize * 2;

        // // merge per tile poly and detail meshes
        let mut pmmerge = Vec::with_capacity(tiles_per_map * tiles_per_map);
        let mut dmmerge = Vec::with_capacity(tiles_per_map * tiles_per_map);
        // build all tiles
        for y in 0..tiles_per_map {
            for x in 0..tiles_per_map {
                //         Tile& tile = tiles[x + y * tiles_per_map];

                // Calculate the per tile bounding box.
                tile_cfg.bmin[0] = config.bmin[0] + (x as i32 * config.tileSize - config.borderSize) as f32 * config.cs;
                tile_cfg.bmin[2] = config.bmin[2] + (y as i32 * config.tileSize - config.borderSize) as f32 * config.cs;
                tile_cfg.bmax[0] = config.bmin[0] + ((x + 1) as i32 * config.tileSize + config.borderSize) as f32 * config.cs;
                tile_cfg.bmax[2] = config.bmin[2] + ((y + 1) as i32 * config.tileSize + config.borderSize) as f32 * config.cs;

                // build heightfield
                let tile_solid = match self.rc_context.create_height_field(
                    tile_cfg.width,
                    tile_cfg.height,
                    &tile_cfg.bmin,
                    &tile_cfg.bmax,
                    tile_cfg.cs,
                    tile_cfg.ch,
                ) {
                    Err(e) => {
                        warn!(e);
                        continue;
                    },
                    Ok(hf) => hf,
                };
                // mark all walkable tiles, both liquids and solids
                let ground_area_id = MmapNavTerrainFlag::Ground.area_id();
                let mut tri_flags = vec![ground_area_id; t_tris.len() / 3];
                self.rc_context
                    .clear_unwalkable_triangles(tile_cfg.walkableSlopeAngle, &t_verts, &t_tris, &mut tri_flags);
                _ = self
                    .rc_context
                    .rasterize_triangles(&t_verts, &t_tris, &tri_flags, &tile_solid, config.walkableClimb);

                self.rc_context
                    .filter_low_hanging_walkable_obstacles(config.walkableClimb, &tile_solid);
                self.rc_context
                    .filter_ledge_spans(tile_cfg.walkableHeight, tile_cfg.walkableClimb, &tile_solid);
                self.rc_context
                    .filter_walkable_low_height_spans(tile_cfg.walkableHeight, &tile_solid);

                _ = self
                    .rc_context
                    .rasterize_triangles(&l_verts, &l_tris, &l_tri_flags, &tile_solid, config.walkableClimb);

                // compact heightfield spans
                let tile_chf =
                    match self
                        .rc_context
                        .build_compact_height_field(tile_cfg.walkableHeight, tile_cfg.walkableClimb, &tile_solid)
                    {
                        Err(e) => {
                            warn!(e);
                            continue;
                        },
                        Ok(hf) => hf,
                    };

                // build polymesh intermediates
                if let Err(e) = self.rc_context.erode_walkable_area(config.walkableRadius, &tile_chf) {
                    warn!(e);
                    continue;
                }

                if let Err(e) = self.rc_context.build_distance_field(&tile_chf) {
                    warn!(e);
                    continue;
                }

                if let Err(e) =
                    self.rc_context
                        .build_regions(&tile_chf, tile_cfg.borderSize, tile_cfg.minRegionArea, tile_cfg.mergeRegionArea)
                {
                    warn!(e);
                    continue;
                }

                let tile_cset = match self
                    .rc_context
                    .build_contours(&tile_chf, tile_cfg.maxSimplificationError, tile_cfg.maxEdgeLen, 1)
                {
                    Err(e) => {
                        warn!(e);
                        continue;
                    },
                    Ok(hf) => hf,
                };

                // build polymesh
                let tile_pmesh = match self.rc_context.build_poly_mesh(&tile_cset, tile_cfg.maxVertsPerPoly) {
                    Err(e) => {
                        warn!(e);
                        continue;
                    },
                    Ok(v) => v,
                };

                let tile_dmesh = match self.rc_context.build_poly_mesh_detail(
                    &tile_pmesh,
                    &tile_chf,
                    tile_cfg.detailSampleDist,
                    tile_cfg.detailSampleMaxError,
                ) {
                    Err(e) => {
                        warn!(e);
                        continue;
                    },
                    Ok(v) => v,
                };
                //         // free those up
                //         // we may want to keep them in the future for debug
                //         // but right now, we don't have the code to merge them
                //         rcFreeHeightField(tile_solid);
                //         tile_solid = NULL;
                //         rcFreeCompactHeightfield(tile_chf);
                //         tile_chf = NULL;
                //         rcFreeContourSet(tile_cset);
                //         tile_cset = NULL;

                pmmerge.push(tile_pmesh);
                dmmerge.push(tile_dmesh);
            }
        }

        let mut iv_poly_mesh = RecastPolyMesh::alloc().map_err(|e| {
            error!("alloc iv_poly_mesh FAILED! {e}");
            az_error!(e)
        })?;
        _ = self.rc_context.merge_poly_meshes(pmmerge, &iv_poly_mesh);

        let iv_poly_mesh_detail = RecastPolyMeshDetail::alloc().map_err(|e| {
            error!("alloc m_dmesh FAILED! {e}");
            az_error!(e)
        })?;
        _ = self.rc_context.merge_poly_mesh_details(dmmerge, &iv_poly_mesh_detail);

        // // free things up
        // delete[] pmmerge;
        // delete[] dmmerge;
        // delete[] tiles;

        // set polygons as walkable
        // TODO: special flags for DYNAMIC polygons, ie surfaces that can be turned on and off
        for i in 0..iv_poly_mesh.npolys as usize {
            let raw_area_id = iv_poly_mesh.get_area_id(i) & RC_WALKABLE_AREA;
            let area = MmapNavTerrainFlag::from_area_id(raw_area_id);

            if raw_area_id > 0 {
                let flag = if area.is_empty() {
                    // Flags that arent known yet, so empty even though raw_area_id > 0
                    // TODO: these will be dynamic in future
                    MmapNavTerrainFlag::Ground.flags().bits()
                } else {
                    area.bits()
                };
                iv_poly_mesh.set_flag(i, flag);
            };
        }

        // setup mesh parameters
        let mut params = DetourNavMeshCreateParams::new_from_recast(
            &iv_poly_mesh,
            &iv_poly_mesh_detail,
            &offset_mesh_connections,
            offset_mesh_connection_rads,
            offset_mesh_connection_dirs,
            offset_mesh_connections_areas,
            offset_mesh_connections_flags,
            base_unit_dim * config.walkableHeight as f32, // agent height
            base_unit_dim * config.walkableRadius as f32, // agent radius
            base_unit_dim * config.walkableClimb as f32,  // keep less that walkableHeight (aka agent height)!
            ((((bmin[0] + bmax[0]) / 2.0) - nav_mesh.get_params().orig[0]) / GRID_SIZE) as i32,
            ((((bmin[2] + bmax[2]) / 2.0) - nav_mesh.get_params().orig[2]) / GRID_SIZE) as i32,
            &(*bmin).into(),
            &(*bmax).into(),
            config.cs,
            config.ch,
            0,
            true,
        );

        #[expect(clippy::never_loop, reason = "loop mechanism is used as a goto")]
        loop {
            if params.polyCount == 0 || params.polys.is_null() || tiles_per_map * tiles_per_map == (params.polyCount as usize) {
                // we have flat tiles with no actual geometry - don't build those, its useless
                // keep in mind that we do output those into debug info
                // drop tiles with only exact count - some tiles may have geometry while having less tiles
                warn!(
                    poly_count = params.polyCount,
                    poly_is_null = params.polys.is_null(),
                    tiles_per_map_sq = tiles_per_map * tiles_per_map,
                    "No polygons to build on tile.",
                );
                break;
            }

            // will hold final navmesh
            let mut nav_data = vec![];

            info!("Building navmesh tile...");
            if let Err(e) = params.create_nav_mesh_data(&mut nav_data.as_mut_slice()) {
                warn!(e);
                break;
            }

            info!("Adding tile to navmesh...");

            let tile_ref = match nav_mesh.add_tile(nav_data, 0) {
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

            // file output
            info!("Writing to file...");

            // write header
            // write data
            let mmtilefile = MmapTileFile::new(!self.skip_liquid, nav_mesh.get_tile_data(tile_ref).clone());
            if let Err(e) = mmtilefile.write_to_mmtile(&self.mmap_output_path, map_id, tile_y, tile_x) {
                error!("error writing to mmtile file: {e}");
                break;
            }
            // now that tile is written to disk, we can unload it
            if let Err(e2) = nav_mesh.remove_tile(tile_ref) {
                warn!("[Map {map_id:04}] unable to free tile on cleanup! err: {e2}");
            }

            if true {
                break;
            }
        }

        if let Some(p) = &self.debug_mesh_output_path {
            // restore padding so that the debug visualization is correct
            for i in 0..iv_poly_mesh.nverts as usize {
                let v = iv_poly_mesh.verts.wrapping_add(i * 3);
                unsafe {
                    *v.wrapping_add(0) += config.borderSize as u16;
                    *v.wrapping_add(2) += config.borderSize as u16;
                }
            }

            let iv = IntermediateValues {
                heightfield:         None,
                compact_heightfield: None,
                contours:            None,
                poly_mesh:           Some(iv_poly_mesh),
                poly_mesh_detail:    Some(iv_poly_mesh_detail),
            };
            iv.generate_obj_file(p, map_id, tile_x, tile_y, mesh_data);
            iv.write_iv(p, map_id, tile_x, tile_y);
        }
        Ok(())
    }

    pub fn should_skip_tile(&self, map_id: u32, tile_x: u16, tile_y: u16) -> bool {
        let header = match MmapTileFile::read_header_from_mmtile(&self.mmap_output_path, map_id, tile_y, tile_x) {
            Err(_) => {
                return false;
            },
            Ok(h) => h,
        };
        header.verify().is_ok()
    }
}
