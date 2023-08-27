use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use nalgebra::Vector3;
use parry3d::bounding_volume::Aabb;
use rayon::prelude::*;
use tracing::{error, info, info_span, instrument, warn};

use crate::{
    az_error,
    common::collision::{management::vmap_mgr2::VMapMgr2, maps::map_tree::StaticMapTree},
    server::shared::recastnavigation_handles::{DetourNavMesh, DetourNavMeshParams, DT_POLY_BITS},
    tools::{
        extractor_common::{bincode_deserialise, bincode_serialise, get_dir_contents, ExtractorConfig},
        mmap_generator::{
            common::{clean_vertices, get_tile_bounds, MeshData, TileInfo, GRID_SIZE},
            terrain_builder::TerrainBuilder,
            tile_builder::TileBuilderParams,
        },
    },
    AzResult,
};

pub struct MapBuilder<'tb> {
    tiles:               HashMap<u32, HashSet<u32>>,
    skip_continents:     bool,
    skip_junk_maps:      bool,
    skip_battlegrounds:  bool,
    tile_builder_params: TileBuilderParams<'tb>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct MapBuilderFile {
    map_id:   u32,
    tile_x:   u16,
    tile_y:   u16,
    vertices: Vec<Vector3<f32>>,
    indices:  Vec<Vector3<u16>>,
}

impl<'tb> MapBuilder<'tb> {
    fn discover_tiles(args: &ExtractorConfig, tb: &TerrainBuilder<'_>) -> AzResult<HashMap<u32, HashSet<u32>>> {
        info!("Discovering maps... ");

        let mut tiles = HashMap::new();

        for f in get_dir_contents(args.output_map_path(), "*")? {
            let map_id = match f.file_stem().and_then(|file_stem| file_stem.to_str()).and_then(|f| {
                let (map_id_str, _rest) = f.split_once('_')?;
                map_id_str.parse::<u32>().ok()
            }) {
                None => {
                    warn!("cannot take map_id from maps file: {}", f.display());
                    continue;
                },
                Some(i) => i,
            };
            tiles.entry(map_id).or_insert(HashSet::new());
        }

        for f in get_dir_contents(args.output_vmap_output_path(), "*.vmtree")? {
            let map_id = match f
                .file_stem()
                .and_then(|file_stem| file_stem.to_str())
                .and_then(|map_id_str| map_id_str.parse::<u32>().ok())
            {
                None => {
                    warn!("cannot take map_id from vmap tree file: {}", f.display());
                    continue;
                },
                Some(i) => i,
            };
            tiles.entry(map_id).or_default();
        }
        info!("found {} maps", tiles.len());

        let mut count = 0;
        for (map_id, map_tiles) in tiles.iter_mut() {
            for f in get_dir_contents(args.output_vmap_output_path(), &format!("{map_id:04}_*.vmtile"))? {
                let tile_id = match f.file_stem().and_then(|file_stem| file_stem.to_str()).map(|f| {
                    let splitted = f.splitn(3, '_').collect::<Vec<_>>();
                    let (_map_id, first, second) = (splitted[0], splitted[1], splitted[2]);

                    let first = first.parse::<u16>().ok().unwrap(); // tileY
                    let second = second.parse::<u16>().ok().unwrap(); // tileX

                    StaticMapTree::pack_tile_id(first, second)
                }) {
                    None => {
                        warn!("cannot take tileID from vmap tree tile file: {}", f.display());
                        continue;
                    },
                    Some(i) => i,
                };
                map_tiles.insert(tile_id);
            }
            for f in get_dir_contents(args.output_map_path(), &format!("{map_id:04}*"))? {
                let tile_id = match f.file_stem().and_then(|file_stem| file_stem.to_str()).map(|f| {
                    let splitted = f.splitn(3, '_').collect::<Vec<_>>();
                    let (_map_id, first, second) = (splitted[0], splitted[1], splitted[2]);

                    let first = first.parse::<u16>().ok().unwrap(); // tileY
                    let second = second.parse::<u16>().ok().unwrap(); // tileX

                    StaticMapTree::pack_tile_id(second, first)
                }) {
                    None => {
                        warn!("cannot take tileID from vmap tree tile file: {}", f.display());
                        continue;
                    },
                    Some(i) => i,
                };
                map_tiles.insert(tile_id);
            }
            // make sure we process maps which don't have tiles
            if map_tiles.is_empty() {
                info!("No map data found so far, try getting grid bounds: {map_id}");
                // convert coord bounds to grid bounds
                // FIXME: This function call to get_grid_bounds will always fail as its trying to get from 64_64, for now we assume
                //that we dont do anything (like how it silently fails for AC / TC)
                if let Ok((min_x, min_y, max_x, max_y)) = get_grid_bounds(tb, *map_id) {
                    // add all tiles within bounds to tile list.
                    for i in min_x..max_x {
                        for j in min_y..max_y {
                            map_tiles.insert(StaticMapTree::pack_tile_id(i, j));
                        }
                    }
                }
            }
            count += map_tiles.len();
        }
        info!("found {count} tiles.");

        Ok(tiles)
    }

    pub fn build(args: &ExtractorConfig, vmap_mgr: VMapMgr2<'tb, 'tb>) -> AzResult<Self> {
        let vmap_mgr = Arc::new(RwLock::new(vmap_mgr));
        // Do the tile discovery first
        let tiles = Self::discover_tiles(
            args,
            &TerrainBuilder {
                vmap_mgr:       vmap_mgr.clone(),
                vmaps_path:     args.output_vmap_output_path(),
                maps_path:      args.output_map_path(),
                use_min_height: args.db2_and_map_extract.use_min_height,
            },
        )?;

        let debug_mesh_output_path = if args.mmap_path_generator.debug_output {
            Some(args.output_meshes_debug_path())
        } else {
            None
        };

        Ok(Self {
            tiles,
            skip_battlegrounds: args.mmap_path_generator.skip_battlegrounds,
            skip_continents: args.mmap_path_generator.skip_continents,
            skip_junk_maps: args.mmap_path_generator.skip_junk_maps,
            tile_builder_params: TileBuilderParams {
                mmap_output_path: args.output_mmap_path().clone(),
                skip_liquid: args.mmap_path_generator.skip_liquid,
                vmap_mgr,
                debug_mesh_output_path,
                off_mesh_file_path: args.mmap_path_generator.off_mesh_file_path.clone().map(PathBuf::from),
                vmaps_path: args.output_vmap_output_path(),
                maps_path: args.output_map_path(),
                use_min_height: args.db2_and_map_extract.use_min_height,
                big_base_unit: args.mmap_path_generator.big_base_unit,
                max_walkable_angle: args.mmap_path_generator.max_angle,
            },
        })
    }

    pub fn build_mesh_from_file<P: AsRef<Path>>(&self, file_path: P) -> AzResult<()> {
        let mut file = fs::File::open(file_path.as_ref())?;
        info!("Building mesh from file: {}", file_path.as_ref().display());
        let MapBuilderFile {
            map_id,
            tile_x,
            tile_y,
            mut vertices,
            mut indices,
        } = bincode_deserialise(&mut file)?;

        let mut nav_mesh = self.build_nav_mesh(map_id)?;

        clean_vertices(&mut vertices, &mut indices);
        // get bounds of current tile
        let b_max_min = get_tile_bounds(tile_x, tile_y, &vertices);

        let data = MeshData {
            solid_verts: vertices,
            solid_tris: indices,
            ..Default::default()
        };

        // build navmesh tile
        self.tile_builder_params
            .clone()
            .try_to_builder()?
            .build_move_map_tile(map_id, tile_x, tile_y, &data, &b_max_min, &mut nav_mesh)?;

        Ok(())
    }

    pub fn build_single_tile(&self, map_id: u32, tile_x: u16, tile_y: u16) -> AzResult<()> {
        let mut nav_mesh = self.build_nav_mesh(map_id)?;

        self.tile_builder_params
            .clone()
            .try_to_builder()?
            .build_tile(map_id, tile_x, tile_y, &mut nav_mesh)?;

        Ok(())
    }

    pub fn build_maps(&self, map_id_opt: Option<u32>) -> AzResult<()> {
        info!("generating mmap tiles to build");

        // for (unsigned int i = 0; i < m_threads; ++i)
        // {
        //     m_tileBuilders.push_back(new TileBuilder(this, m_skipLiquid, m_bigBaseUnit, m_debugOutput));
        // }

        let tiles_to_build = if let Some(map_id) = map_id_opt {
            self.gather_map_tiles(map_id)?
        } else {
            // Build all maps if no map id has been specified
            let mut res = vec![];
            for map_id in self.tiles.keys() {
                if !self.should_skip_map(*map_id) {
                    res.extend(self.gather_map_tiles(*map_id)?);
                }
            }
            res
        };

        let num_tiles = tiles_to_build.len();
        let header_span = info_span!("build_maps");
        let _header_span_enter = header_span.enter();
        let result: AzResult<Vec<_>> = tiles_to_build
            .into_par_iter()
            .enumerate()
            .map(|(count, (ti, tp))| {
                info!(
                    "{}/{} building for Map {:04} - {:02},{:02}",
                    count, num_tiles, ti.map_id, ti.tile_x, ti.tile_y
                );
                // Span::current().pb_inc(1);
                build_tile_work(ti, tp)
            })
            .collect();

        result?;

        Ok(())
    }

    /// buildMap in TC/AC
    #[instrument(skip_all, fields(map = format!("[Map {map_id:04}]")))]
    fn gather_map_tiles(&self, map_id: u32) -> AzResult<Vec<(TileInfo, TileBuilderParams<'tb>)>> {
        let empty = Default::default();
        let tiles = self.tiles.get(&map_id).unwrap_or(&empty);
        if tiles.is_empty() {
            return Ok(vec![]);
        }
        let nav_mesh = self.build_nav_mesh(map_id).map_err(|e| {
            error!("Failed creating navmesh!");
            e
        })?;
        let mut tile_infos = Vec::with_capacity(tiles.len());
        // now start building mmtiles for each tile
        info!("we have {} tiles", tiles.len());
        for tile in tiles {
            // unpack tile coords
            let (tile_x, tile_y) = StaticMapTree::unpack_tile_id(*tile);

            let nav_mesh_params = *nav_mesh.get_params();
            tile_infos.push((
                TileInfo {
                    map_id,
                    tile_x,
                    tile_y,
                    nav_mesh_params: nav_mesh_params.into(),
                },
                self.tile_builder_params.clone(),
            ))
        }

        Ok(tile_infos)
    }

    fn build_nav_mesh(&self, map_id: u32) -> AzResult<DetourNavMesh> {
        // if map has a parent we use that to generate dtNavMeshParams - worldserver will load all missing tiles from that map
        let nav_mesh_params_map_id = self.tile_builder_params.vmap_mgr.read().unwrap().get_parent_map_id(map_id);

        let empty = Default::default();
        let tiles = self.tiles.get(&nav_mesh_params_map_id).unwrap_or(&empty);

        // old code for non-statically assigned bitmask sizes:
        // /*** calculate number of bits needed to store tiles & polys ***/
        //int tileBits = dtIlog2(dtNextPow2(tiles->size()));
        //if (tileBits < 1) tileBits = 1;                                     // need at least one bit!
        //int polyBits = sizeof(dtPolyRef)*8 - SALT_MIN_BITS - tileBits;

        let poly_bits = DT_POLY_BITS;

        let max_tiles = tiles.len();
        let max_polys_per_tile = 1 << poly_bits;

        /***          calculate bounds of map         ***/
        let (_tile_x_min, _tile_y_min, tile_x_max, tile_y_max) =
            tiles.iter().fold((64, 64, 0, 0), |(x_min, y_min, x_max, y_max), tile_id| {
                let (x, y) = StaticMapTree::unpack_tile_id(*tile_id);
                (x.min(x_min), y.min(y_min), x.max(x_max), y.max(y_max))
            });

        // use Max because '32 - tile_x' is negative for values over 32
        let b_min_max = get_tile_bounds(tile_x_max, tile_y_max, &vec![]);

        /***       now create the navmesh       ***/
        // navmesh creation params
        let nav_mesh_params = DetourNavMeshParams::new(&b_min_max.mins.into(), GRID_SIZE, GRID_SIZE, max_tiles as i32, max_polys_per_tile);
        info!("Creating nav_mesh...");
        let (nav_mesh, _) = DetourNavMesh::init(&nav_mesh_params)?;

        let file_name = self.tile_builder_params.mmap_output_path.join(format!("{map_id:04}.mmap"));
        let mut file = fs::File::create(file_name)?;
        // now that we know nav_mesh params are valid, we can write them to file
        // TODO: Do the dedup logic here, if a navmesh file exists, we load the navmesh from there.
        bincode_serialise(&mut file, &nav_mesh_params)?;
        Ok(nav_mesh)
    }

    fn should_skip_map(&self, map_id: u32) -> bool {
        if self.skip_continents {
            match map_id {
                0 | 1 | 530 | 571 | 870 | 1116 | 1220 => return true,
                _ => {},
            }
        }

        if self.skip_junk_maps {
            match map_id
            {
                13 |    // test.wdt
                25 |    // ScottTest.wdt
                29 |    // Test.wdt
                42 |    // Colin.wdt
                169 |   // EmeraldDream.wdt (unused, and very large)
                451 |   // development.wdt
                573 |   // ExteriorTest.wdt
                597 |   // CraigTest.wdt
                605 |   // development_nonweighted.wdt
                606 |   // QA_DVD.wdt
                651 |   // ElevatorSpawnTest.wdt
                1060 |  // LevelDesignLand-DevOnly.wdt
                1181 |  // PattyMackTestGarrisonBldgMap.wdt
                1264 |  // Propland-DevOnly.wdt
                1270 |  // devland3.wdt
                1310 |  // Expansion5QAModelMap.wdt
                1407 |  // GorgrondFinaleScenarioMap.wdt (zzzOld)
                1427 |  // PattyMackTestGarrisonBldgMap2.wdt
                1451 |  // TanaanLegionTest.wdt
                1454 |  // ArtifactAshbringerOrigin.wdt
                1457 |  // FXlDesignLand-DevOnly.wdt
                1471 |  // 1466.wdt (Dungeon Test Map 1466)
                1499 |  // Artifact-Warrior Fury Acquisition.wdt (oldArtifact - Warrior Fury Acquisition)
                1537 |  // BoostExperience.wdt (zzOLD - Boost Experience)
                1538 |  // Karazhan Scenario.wdt (test)
                1549 |  // TechTestSeamlessWorldTransitionA.wdt
                1550 |  // TechTestSeamlessWorldTransitionB.wdt
                1555 |  // TransportBoostExperienceAllianceGunship.wdt
                1556 |  // TransportBoostExperienceHordeGunship.wdt
                1561 |  // TechTestCosmeticParentPerformance.wdt
                1582 |  // Artifact�DalaranVaultAcquisition.wdt // no, this weird symbol is not an encoding error.
                1584 |  // JulienTestLand-DevOnly.wdt
                1586 |  // AssualtOnStormwind.wdt (Assault on Stormwind - Dev Map)
                1588 |  // DevMapA.wdt
                1589 |  // DevMapB.wdt
                1590 |  // DevMapC.wdt
                1591 |  // DevMapD.wdt
                1592 |  // DevMapE.wdt
                1593 |  // DevMapF.wdt
                1594 |  // DevMapG.wdt
                1603 |  // AbyssalMaw_Interior_Scenario.wdt
                1670 =>  // BrokenshorePristine.wdt
                    {return true;},
                _ => {
                    if is_transport_map(map_id) {
                        return true
                    }
                }
            }
        }

        if self.skip_battlegrounds {
            match map_id
            {
                 30 |    // Alterac Valley
                 37 |    // ?
                 489 |   // Warsong Gulch
                 529 |   // Arathi Basin
                 566 |   // Eye of the Storm
                 607 |   // Strand of the Ancients
                 628 |   // Isle of Conquest
                 726 |   // Twin Peaks
                 727 |   // Silvershard Mines
                 761 |   // The Battle for Gilneas
                 968 |   // Rated Eye of the Storm
                 998 |   // Temple of Kotmogu
                 1010 |  // CTF3
                 1105 |  // Deepwind Gorge
                 1280 |  // Southshore vs. Tarren Mill
                 1681 |  // Arathi Basin Winter
                 1803 =>  // Seething Shore
                    {return true;}
                _ => {}
            }
        }

        false
    }
}

#[instrument(skip_all, fields(tile = format!("[Map {:04}] [{:02},{:02}]", ti.map_id, ti.tile_x, ti.tile_y)))]
fn build_tile_work(ti: TileInfo, tp: TileBuilderParams) -> AzResult<()> {
    let tb = match tp.try_to_builder() {
        Err(e) => {
            error!(
                "[Map {:04}] Failed creating tile builder for tile {:02},{:02}! err = {e}",
                ti.map_id, ti.tile_x, ti.tile_y
            );
            // ignore for now.
            return Ok(());
        },
        Ok(b) => b,
    };
    let (mut nav_mesh, _) = match DetourNavMesh::init(&ti.nav_mesh_params) {
        Err(e) => {
            error!(
                "[Map {:04}] Failed creating navmesh for tile {:02},{:02}! err = {e}",
                ti.map_id, ti.tile_x, ti.tile_y
            );
            // ignore for now.
            return Ok(());
        },
        Ok(n) => n,
    };
    tb.build_tile(ti.map_id, ti.tile_x, ti.tile_y, &mut nav_mesh).map_err(|e| {
        error!("Build tile failed because of error: {e}");
        e
    })?;
    Ok(())
}

fn get_grid_bounds(tb: &TerrainBuilder<'_>, map_id: u32) -> AzResult<(u16, u16, u16, u16)> {
    // make sure we process maps which don't have tiles
    // initialize the static tree, which loads WDT models
    //
    // TODO: Fix me! now this fails because tile_x and tile_y cannot be > 63
    let mut mesh_data = MeshData::default();
    tb.load_vmap(map_id, 64, 64, &mut mesh_data)?;

    if mesh_data.solid_verts.is_empty() && mesh_data.liquid_verts.is_empty() {
        return Err(az_error!("no mesh verticals found for map_id {map_id}"));
    }

    let mut bounding = Aabb::new_invalid();
    for vertex in mesh_data.solid_verts {
        bounding.take_point(vertex.into());
    }

    for vertex in mesh_data.liquid_verts {
        bounding.take_point(vertex.into());
    }

    // convert coord bounds to grid bounds
    // Axes are flipped somehow here.
    let min_x = 32 - (bounding.mins.x / GRID_SIZE).floor() as u16;
    let max_x = 32 - (bounding.maxs.x / GRID_SIZE).floor() as u16;

    let min_y = 32 - (bounding.mins.z / GRID_SIZE).floor() as u16;
    let max_y = 32 - (bounding.maxs.z / GRID_SIZE).floor() as u16;

    Ok((min_x, min_y, max_x, max_y))
}

fn is_transport_map(map_id: u32) -> bool {
    match map_id {
        // transport maps
        582 | 584 | 586 | 587 | 588 | 589 | 590 | 591 | 592 | 593 | 594 | 596 | 610 | 612 | 613 | 614 | 620 | 621 | 622 | 623 | 641
        | 642 | 647 | 662 | 672 | 673 | 674 | 712 | 713 | 718 | 738 | 739 | 740 | 741 | 742 | 743 | 747 | 748 | 749 | 750 | 762 | 763
        | 765 | 766 | 767 | 1113 | 1132 | 1133 | 1172 | 1173 | 1192 | 1231 | 1459 | 1476 | 1484 | 1555 | 1556 | 1559 | 1560 | 1628
        | 1637 | 1638 | 1639 | 1649 | 1650 | 1711 | 1751 | 1752 | 1856 | 1857 | 1902 | 1903 => true,
        _ => false,
    }
}
