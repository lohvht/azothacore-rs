use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use azothacore_common::{
    az_error,
    bevy_app::AzStartupFailedEvent,
    collision::{
        management::vmap_mgr2::{LiquidFlagsGetter, VMapMgr2, VmapDisabledChecker},
        maps::map_defines::MmapTileFile,
    },
    configuration::ConfigMgr,
    deref_boilerplate,
    recastnavigation_handles::{DetourNavMesh, DetourNavMeshParams, DT_POLY_BITS},
    utils::{bincode_serialise, buffered_file_create},
    AzResult,
    MapLiquidTypeFlag,
};
use bevy::{
    app::{App, AppExit, PostUpdate, Startup, Update},
    ecs::system::SystemParam,
    prelude::{Commands, Component, Entity, EventWriter, In, IntoSystem, IntoSystemConfigs, Local, Query, Res, Resource, SystemSet, With, Without},
};
use flagset::FlagSet;
use parry3d::bounding_volume::Aabb;
use tracing::{error, info, warn};

use crate::{
    extractor_common::{get_dir_contents, ExtractorConfig, MapIdTileXY},
    mmap_generator::{
        common::{get_tile_bounds, MeshData, TileInfo, GRID_SIZE},
        terrain_builder::TerrainBuilder,
        tile_builder::TileBuilder,
    },
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MmapGenerationSets {
    DiscoverAndGenerateTilesToWork,
}

#[derive(Resource)]
struct MmapGeneratorStarted(Instant);

pub fn mmap_generator_plugin(app: &mut App) {
    app.insert_resource(MmapGeneratorStarted(Instant::now()))
        .add_systems(
            Startup,
            ((
                MapBuilder::discover_tiles.pipe(handle_discover_tiles_error),
                MapBuilder::populate_tiles_to_build,
            )
                .chain())
            .in_set(MmapGenerationSets::DiscoverAndGenerateTilesToWork),
        )
        .add_systems(
            Update,
            (
                load_tile_meshdata_work.run_if(any_tile_mesh_not_attempted),
                build_mmap_tile_work.run_if(any_tile_mmap_not_attempted),
            ),
        )
        .add_systems(PostUpdate, exit_when_all_tiles_attempted_generated);
}

pub type MmapBuilderVmapMgr<'w> = VMapMgr2<'w, ExtractorConfig, LiquidTypes, VmapNotDisabled>;
#[derive(Resource)]
pub struct VmapNotDisabled;

impl VmapDisabledChecker for VmapNotDisabled {
    fn is_vmap_disabled_for(&self, _entry: u32, _flags: u8) -> bool {
        false
    }
}

#[derive(Resource)]
pub struct LiquidTypes(pub HashMap<u32, FlagSet<MapLiquidTypeFlag>>);

impl LiquidFlagsGetter for LiquidTypes {
    fn get_liquid_flags(&self, liquid_type_id: u32) -> FlagSet<MapLiquidTypeFlag> {
        self.0.get(&liquid_type_id).map_or_else(|| None.into(), |liq_sound_bank| *liq_sound_bank)
    }
}

#[derive(Resource)]
struct AvailableTiles(HashMap<u32, HashSet<(u16, u16)>>);

deref_boilerplate!(AvailableTiles, HashMap<u32, HashSet<(u16, u16)>>, 0);

#[derive(Resource)]
struct NumberOfTilesToWork(u32);

deref_boilerplate!(NumberOfTilesToWork, u32, 0);

#[derive(SystemParam)]
struct MapBuilder<'w> {
    tiles:    Res<'w, AvailableTiles>,
    args:     Res<'w, ConfigMgr<ExtractorConfig>>,
    vmap_mgr: MmapBuilderVmapMgr<'w>,
}

fn handle_discover_tiles_error(In(res): In<AzResult<()>>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>) {
    if let Err(e) = res {
        error!(cause=%e, "error when discovering tiles");
        ev_startup_failed.send_default();
    }
}

impl MapBuilder<'_> {
    fn discover_tiles(mut commands: Commands, mut tb: TerrainBuilder) -> AzResult<()> {
        info!("Discovering maps... ");

        let mut tiles = HashMap::new();

        for f in get_dir_contents(tb.cfg.output_map_path(), "*")? {
            let map_id = match f
                .file_stem()
                .and_then(|file_stem| file_stem.to_str())
                .and_then(|f| f.split_once('_')?.0.parse::<u32>().ok())
            {
                None => {
                    warn!("cannot take map_id from maps file: {}", f.display());
                    continue;
                },
                Some(i) => i,
            };
            tiles.entry(map_id).or_insert(HashSet::new());
        }

        for f in get_dir_contents(tb.cfg.output_vmap_output_path(), "*.vmtree")? {
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
            for f in get_dir_contents(tb.cfg.output_vmap_output_path(), &format!("{map_id:04}_*.vmtile"))? {
                let tile_id = match f.file_stem().and_then(|file_stem| file_stem.to_str()) {
                    None => {
                        warn!("cannot take tileID from vmap tree tile file: {}", f.display());
                        continue;
                    },
                    Some(f) => {
                        let splitted = f.splitn(3, '_').collect::<Vec<_>>();
                        let (_map_id, first, second) = (splitted[0], splitted[1], splitted[2]);

                        let first = first.parse::<u16>().ok().unwrap(); // tileY
                        let second = second.parse::<u16>().ok().unwrap(); // tileX

                        (first, second)
                    },
                };
                map_tiles.insert(tile_id);
            }
            for f in get_dir_contents(tb.cfg.output_map_path(), &format!("{map_id:04}*"))? {
                let tile_id = match f.file_stem().and_then(|file_stem| file_stem.to_str()) {
                    None => {
                        warn!("cannot take tileID from vmap tree tile file: {}", f.display());
                        continue;
                    },
                    Some(f) => {
                        let splitted = f.splitn(3, '_').collect::<Vec<_>>();
                        let (_map_id, first, second) = (splitted[0], splitted[1], splitted[2]);

                        let first = first.parse::<u16>().ok().unwrap(); // tileY
                        let second = second.parse::<u16>().ok().unwrap(); // tileX

                        (second, first)
                    },
                };
                map_tiles.insert(tile_id);
            }
            // make sure we process maps which don't have tiles
            if map_tiles.is_empty() {
                info!("No map data found so far, try getting grid bounds: {map_id}");
                // convert coord bounds to grid bounds
                // FIXME: This function call to get_grid_bounds will always fail as its trying to get from 64_64, for now we assume
                //that we dont do anything (like how it silently fails for AC / TC)
                if let Ok((min_x, min_y, max_x, max_y)) = get_grid_bounds(&mut tb, *map_id) {
                    // add all tiles within bounds to tile list.
                    for i in min_x..max_x {
                        for j in min_y..max_y {
                            map_tiles.insert((i, j));
                        }
                    }
                }
            }
            count += map_tiles.len();
        }
        info!("found {count} tiles.");
        commands.insert_resource(AvailableTiles(tiles));

        Ok(())
    }

    fn populate_tiles_to_build(this: MapBuilder, mut commands: Commands, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>) {
        info!("Begin populating tiles to build");
        if let Err(e) = this.build_maps(&mut commands) {
            error!(cause=%e, "error when populating tiles to build");
            ev_startup_failed.send_default();
        }
    }

    fn build_maps(&self, commands: &mut Commands) -> AzResult<()> {
        let mut tiles_to_build_count = 0;
        let map_id_opt = match &self.args.mmap_path_generator.map_id_tile_x_y {
            Some(MapIdTileXY {
                map_id,
                tile_x_y: Some((tile_x, tile_y)),
            }) if !self.should_skip_map(*map_id) && !self.should_skip_tile(*map_id, *tile_x, *tile_y) => {
                let nav_mesh = self.build_nav_mesh(*map_id)?;
                let nav_mesh_params = *nav_mesh.get_params();
                commands.spawn(TileInfo {
                    map_id:          *map_id,
                    tile_x:          *tile_x,
                    tile_y:          *tile_y,
                    nav_mesh_params: nav_mesh_params.into(),
                });
                tiles_to_build_count += 1;
                commands.insert_resource(NumberOfTilesToWork(tiles_to_build_count));
                return Ok(());
            },
            opt => opt.as_ref().map(|v| v.map_id),
        };

        info!("generating mmap tiles to build");

        // for (unsigned int i = 0; i < m_threads; ++i)
        // {
        //     m_tileBuilders.push_back(new TileBuilder(this, m_skipLiquid, m_bigBaseUnit, m_debugOutput));
        // }

        let map_ids = if let Some(map_id) = map_id_opt {
            vec![map_id]
        } else {
            // Build all maps if no map id has been specified
            self.tiles.keys().cloned().collect()
        };

        for map_id in map_ids {
            if self.should_skip_map(map_id) {
                continue;
            }
            let Some(tiles) = self.tiles.get(&map_id) else { continue };
            if tiles.is_empty() {
                continue;
            }
            let nav_mesh = self.build_nav_mesh(map_id).inspect_err(|e| {
                error!(cause=%e, "Failed creating navmesh!");
            })?;

            for tile in tiles {
                // unpack tile coords
                let (tile_x, tile_y) = *tile;
                if self.should_skip_tile(map_id, tile_x, tile_y) {
                    continue;
                }
                let nav_mesh_params = *nav_mesh.get_params();
                commands.spawn(TileInfo {
                    map_id,
                    tile_x,
                    tile_y,
                    nav_mesh_params: nav_mesh_params.into(),
                });
                tiles_to_build_count += 1;
            }
        }
        info!("we have {tiles_to_build_count} tiles to build");
        commands.insert_resource(NumberOfTilesToWork(tiles_to_build_count));
        Ok(())
    }

    fn build_nav_mesh(&self, map_id: u32) -> AzResult<DetourNavMesh> {
        // if map has a parent we use that to generate dtNavMeshParams - worldserver will load all missing tiles from that map
        let nav_mesh_params_map_id = self.vmap_mgr.helper.parent_map_data.get(&map_id).unwrap_or(&map_id);

        let empty = Default::default();
        let tiles = self.tiles.get(nav_mesh_params_map_id).unwrap_or(&empty);

        // old code for non-statically assigned bitmask sizes:
        // /*** calculate number of bits needed to store tiles & polys ***/
        //int tileBits = dtIlog2(dtNextPow2(tiles->size()));
        //if (tileBits < 1) tileBits = 1;                                     // need at least one bit!
        //int polyBits = sizeof(dtPolyRef)*8 - SALT_MIN_BITS - tileBits;

        let poly_bits = DT_POLY_BITS;

        let max_tiles = tiles.len();
        let max_polys_per_tile = 1 << poly_bits;

        /***          calculate bounds of map         ***/
        let mut tile_x_min = 64;
        let mut tile_y_min = 64;
        let mut tile_x_max = 0;
        let mut tile_y_max = 0;
        for tile_id in tiles {
            let (tile_x, tile_y) = *tile_id;
            tile_x_max = tile_x_max.max(tile_x);
            tile_x_min = tile_x_min.min(tile_x);
            tile_y_max = tile_y_max.max(tile_y);
            tile_y_min = tile_y_min.min(tile_y);
        }
        let mut bmin = [0.0; 3];
        let mut bmax = [0.0; 3];
        // use Max because '32 - tile_x' is negative for values over 32
        get_tile_bounds(tile_x_max, tile_y_max, &[], &mut bmin, &mut bmax);

        /***       now create the navmesh       ***/
        // navmesh creation params
        let nav_mesh_params = DetourNavMeshParams::new(&bmin, GRID_SIZE, GRID_SIZE, max_tiles as i32, max_polys_per_tile);
        info!("Creating nav_mesh...");
        let (nav_mesh, _) = DetourNavMesh::init(&nav_mesh_params)?;

        let file_name = self.args.output_mmap_path().join(format!("{map_id:04}.mmap"));
        let mut file = buffered_file_create(file_name)?;
        // now that we know nav_mesh params are valid, we can write them to file
        // TODO: Do the dedup logic here, if a navmesh file exists, we load the navmesh from there.
        bincode_serialise(&mut file, &nav_mesh_params)?;
        Ok(nav_mesh)
    }

    fn should_skip_tile(&self, map_id: u32, tile_x: u16, tile_y: u16) -> bool {
        let header = match MmapTileFile::read_header_from_mmtile(self.args.output_mmap_path(), map_id, tile_y, tile_x) {
            Err(_) => {
                return false;
            },
            Ok(h) => h,
        };
        header.verify().is_ok()
    }

    fn should_skip_map(&self, map_id: u32) -> bool {
        if self.args.mmap_path_generator.skip_continents {
            match map_id {
                0 | 1 | 530 | 571 | 870 | 1116 | 1220 => return true,
                _ => {},
            }
        }

        if self.args.mmap_path_generator.skip_junk_maps {
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

        if self.args.mmap_path_generator.skip_battlegrounds {
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

#[derive(Component)]
struct AttemptedLoadMesh;

#[allow(clippy::type_complexity)]
fn load_tile_meshdata_work(
    mut commands: Commands,
    mut teb: TerrainBuilder,
    tile_infos: Query<(Entity, &TileInfo), (Without<MeshData>, Without<AttemptedLoadMesh>)>,
) {
    let mut count = 0;
    for (e, ti) in &tile_infos {
        count += 1;
        commands.entity(e).insert(AttemptedLoadMesh);

        info!("loading mesh data for Map {:04} - {:02},{:02}", ti.map_id, ti.tile_x, ti.tile_y);
        let mesh_data = match TileBuilder::load_map_vertices(&mut teb, ti.map_id, ti.tile_x, ti.tile_y) {
            Err(e) => {
                error!(
                    map_id = ti.map_id,
                    tile_x = ti.tile_x,
                    tile_y = ti.tile_y,
                    "Load map vertices failed because of error: {e}"
                );
                continue;
            },
            Ok(m) => m,
        };
        commands.entity(e).insert(mesh_data);

        if count > 1000 {
            // break so that we give the other systems in the same stage a chance
            break;
        }
    }
}

fn any_tile_mesh_not_attempted(tile_infos: Query<Option<&AttemptedLoadMesh>, With<TileInfo>>) -> bool {
    tile_infos.iter().any(|i| i.is_none())
}

#[derive(Component)]
struct AttemptedBuildMmapTile;

fn build_mmap_tile_work(
    mut commands: Commands,
    cfg: Res<ConfigMgr<ExtractorConfig>>,
    tile_infos: Query<(Entity, &TileInfo, &MeshData), Without<AttemptedBuildMmapTile>>,
) {
    for e in tile_infos.iter().map(|v| v.0) {
        commands.entity(e).insert(AttemptedBuildMmapTile);
    }
    tile_infos.par_iter().for_each(|(_, ti, md)| {
        info!("building tile for Map {:04} - {:02},{:02}", ti.map_id, ti.tile_x, ti.tile_y);
        if let Err(e) = TileBuilder::build_mmap_tile(&cfg, ti.map_id, ti.tile_x, ti.tile_y, &ti.nav_mesh_params, md) {
            warn!(
                map_id = ti.map_id,
                tile_x = ti.tile_x,
                tile_y = ti.tile_y,
                "Build tile failed because of error: {e}"
            );
        }
    });
}

fn any_tile_mmap_not_attempted(tile_infos: Query<Option<&AttemptedBuildMmapTile>, (With<TileInfo>, With<MeshData>)>) -> bool {
    tile_infos.iter().any(|i| i.is_none())
}

#[allow(clippy::type_complexity)]
fn exit_when_all_tiles_attempted_generated(
    started: Res<MmapGeneratorStarted>,
    num_tiles: Res<NumberOfTilesToWork>,
    mut num_processed: Local<u32>,
    mut commands: Commands,
    tile_infos: Query<(
        Entity,
        &TileInfo,
        Option<&MeshData>,
        Option<&AttemptedLoadMesh>,
        Option<&AttemptedBuildMmapTile>,
    )>,
    mut ev_app_exit: EventWriter<AppExit>,
) {
    let mut all_attempted = true;
    for (e, ti, md, alm, abm) in &tile_infos {
        if alm.is_none() {
            all_attempted = false;
            continue;
        }
        let has_meshdata = md.is_some();
        let has_attempted_build_tile = abm.is_some();
        if has_meshdata && !has_attempted_build_tile {
            all_attempted = false;
            continue;
        }
        *num_processed += 1;
        info!(
            "{}/{} tiles processed - tile [Map {} {},{}]",
            *num_processed, num_tiles.0, ti.map_id, ti.tile_x, ti.tile_y
        );
        commands.entity(e).despawn();
    }
    if !all_attempted {
        return;
    }
    let time_run = started.0.elapsed();
    info!("Finished. MMAPS were built in {}s", time_run.as_secs());
    ev_app_exit.send(AppExit::Success);
}

fn get_grid_bounds(tb: &mut TerrainBuilder, map_id: u32) -> AzResult<(u16, u16, u16, u16)> {
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
    for vertex in mesh_data.solid_verts.chunks(3) {
        bounding.take_point([vertex[0], vertex[1], vertex[2]].into());
    }

    for vertex in mesh_data.liquid_verts.chunks(3) {
        bounding.take_point([vertex[0], vertex[1], vertex[2]].into());
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
        582 | 584 | 586 | 587 | 588 | 589 | 590 | 591 | 592 | 593 | 594 | 596 | 610 | 612 | 613 | 614 | 620 | 621 | 622 | 623 | 641 | 642 | 647 | 662 | 672
        | 673 | 674 | 712 | 713 | 718 | 738 | 739 | 740 | 741 | 742 | 743 | 747 | 748 | 749 | 750 | 762 | 763 | 765 | 766 | 767 | 1113 | 1132 | 1133 | 1172
        | 1173 | 1192 | 1231 | 1459 | 1476 | 1484 | 1555 | 1556 | 1559 | 1560 | 1628 | 1637 | 1638 | 1639 | 1649 | 1650 | 1711 | 1751 | 1752 | 1856 | 1857
        | 1902 | 1903 => true,
        _ => false,
    }
}
