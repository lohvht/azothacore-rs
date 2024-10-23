use azothacore_common::{
    az_error,
    collision::{management::vmap_mgr2::LiquidFlagsGetter, maps::map_defines::MmapNavTerrainFlag, models::model_instance::ModelFlags},
    configuration::ConfigMgr,
    g3dlite_copied::matrix3_from_euler_angles_xyz,
    row_vector_to_matrix_index,
    utils::buffered_file_open,
    AzResult,
};
use azothacore_server::game::map::{map_file::MapFile, GridMap, MapLiquidTypeFlag};
use bevy::{ecs::system::SystemParam, prelude::Res};
use flagset::FlagSet;
use nalgebra::{DMatrix, SMatrix, Vector3};
use tracing::debug;

use crate::{
    extractor_common::ExtractorConfig,
    mmap_generator::{
        common::{MeshData, GRID_PART_SIZE, GRID_SIZE, INVALID_MAP_LIQ_HEIGHT_MAX, V8_SIZE, V8_SIZE_SQ, V9_SIZE, V9_SIZE_SQ},
        map_builder::MmapBuilderVmapMgr,
    },
};

#[derive(Debug)]
enum Spot {
    Top = 1,
    Right = 2,
    Left = 3,
    Bottom = 4,
    Entire = 5,
}

impl Spot {
    fn get_loop_vars(&self) -> impl Iterator<Item = usize> {
        let (loop_start, loop_end, loop_incre) = match *self {
            Spot::Entire => (0, V8_SIZE_SQ, 1),
            Spot::Top => (0, V8_SIZE, 1),
            Spot::Left => (0, V8_SIZE_SQ - V8_SIZE + 1, V8_SIZE),
            Spot::Right => (V8_SIZE - 1, V8_SIZE_SQ, V8_SIZE),
            Spot::Bottom => (V8_SIZE_SQ - V8_SIZE, V8_SIZE_SQ, 1),
        };
        (loop_start..loop_end).step_by(loop_incre)
    }

    fn get_height_triangle(&self, square: usize, liquid: bool, out_indices: &mut [i32; 3]) {
        let row_offset = square / V8_SIZE;
        if !liquid {
            //  0-----1 .... 128
            //  |\ T /|
            //  | \ / |
            //  |L 0 R| .. 127
            //  | / \ |
            //  |/ B \|
            // 129---130 ... 386
            //  |\   /|
            //  | \ / |
            //  | 128 | .. 255
            //  | / \ |
            //  |/   \|
            // 258---259 ... 515
            match *self {
                Spot::Top => {
                    out_indices[0] = (square + row_offset) as _;
                    out_indices[1] = (square + 1 + row_offset) as _;
                    out_indices[2] = (V9_SIZE_SQ + square) as _;
                },
                Spot::Left => {
                    out_indices[0] = (square + row_offset) as _;
                    out_indices[1] = (V9_SIZE_SQ + square) as _;
                    out_indices[2] = (square + V9_SIZE + row_offset) as _;
                },
                Spot::Right => {
                    out_indices[0] = (square + 1 + row_offset) as _;
                    out_indices[1] = (square + V9_SIZE + 1 + row_offset) as _;
                    out_indices[2] = (V9_SIZE_SQ + square) as _;
                },
                Spot::Bottom => {
                    out_indices[0] = (V9_SIZE_SQ + square) as _;
                    out_indices[1] = (square + V9_SIZE + 1 + row_offset) as _;
                    out_indices[2] = (square + V9_SIZE + row_offset) as _;
                },
                Spot::Entire => {},
            }
        } else {
            //  0-----1 .... 128
            //  |\    |
            //  | \ T |
            //  |  \  |
            //  | B \ |
            //  |    \|
            // 129---130 ... 386
            //  |\    |
            //  | \   |
            //  |  \  |
            //  |   \ |
            //  |    \|
            // 258---259 ... 515
            match *self {
                Spot::Top => {
                    out_indices[0] = (square + row_offset) as _;
                    out_indices[1] = (square + 1 + row_offset) as _;
                    out_indices[2] = (square + V9_SIZE + 1 + row_offset) as _;
                },
                Spot::Bottom => {
                    out_indices[0] = (square + row_offset) as _;
                    out_indices[1] = (square + V9_SIZE + 1 + row_offset) as _;
                    out_indices[2] = (square + V9_SIZE + row_offset) as _;
                },
                Spot::Entire | Spot::Left | Spot::Right => {},
            }
        }
    }
}

#[derive(SystemParam)]
pub struct TerrainBuilder<'w> {
    pub cfg:      Res<'w, ConfigMgr<ExtractorConfig>>,
    pub vmap_mgr: MmapBuilderVmapMgr<'w>,
}

impl TerrainBuilder<'_> {
    pub fn load_map(&self, map_id: u32, tile_x: u16, tile_y: u16, mesh_data: &mut MeshData) -> AzResult<()> {
        if let Err(e) = self.load_map_spot(map_id, tile_x, tile_y, Spot::Entire, mesh_data) {
            tracing::trace!("error loading entire map spot for the map ID {map_id} [{tile_x}:{tile_y}]: err was {e}");
            return Ok(());
        };

        _ = self.load_map_spot(map_id, tile_x + 1, tile_y, Spot::Left, mesh_data);
        _ = self.load_map_spot(map_id, tile_x - 1, tile_y, Spot::Right, mesh_data);
        _ = self.load_map_spot(map_id, tile_x, tile_y + 1, Spot::Top, mesh_data);
        _ = self.load_map_spot(map_id, tile_x, tile_y - 1, Spot::Bottom, mesh_data);

        Ok(())
    }

    fn load_map_spot(&self, map_id: u32, tile_x: u16, tile_y: u16, portion: Spot, mesh_data: &mut MeshData) -> AzResult<()> {
        let mut tried_map_ids = vec![];
        let mut used_map_id = Some(map_id);
        let map_file = loop {
            let map_id = match used_map_id {
                None => {
                    break Err(az_error!(
                        "error retrieving map spot for the map {map_id:04}[{tile_x},{tile_y}], tried finding from these maps: {tried_map_ids:?}"
                    ))
                },
                Some(i) => i,
            };
            let map_file_name = GridMap::file_name(self.cfg.output_map_path(), map_id, tile_y, tile_x);
            let f = match buffered_file_open(&map_file_name).map_err(|e| e.into()).and_then(|mut f| MapFile::read(&mut f)) {
                Err(_) => {
                    tried_map_ids.push(map_id);
                    used_map_id = self.vmap_mgr.helper.parent_map_data.get(&map_id).cloned();
                    continue;
                },
                Ok(m) => m,
            };
            break Ok(f);
        }?;

        // i.e. Has height
        let have_terrain = map_file.map_height_data.map_heights.is_some();
        let have_liquid = !self.cfg.mmap_path_generator.skip_liquid && map_file.map_liquid_data.is_some();

        if !have_terrain && !have_liquid {
            return Err(az_error!("no data in this map file"));
        }

        // Temporary
        let mut ltriangles = vec![];
        let mut ttriangles = vec![];

        // terrain data
        if let Some(v9v8) = &map_file.map_height_data.map_heights {
            let (v9, v8) = v9v8.to_v9v8(map_file.map_height_data.grid_height, map_file.map_height_data.grid_max_height);
            let count = mesh_data.solid_verts.len() / 3;
            let x_offset = (tile_x as f32 - 32.0) * GRID_SIZE;
            let y_offset = (tile_y as f32 - 32.0) * GRID_SIZE;

            let mut coord = [0.0; 3];
            for i in 0..V9_SIZE_SQ {
                terrain_builder_get_height_coord(i, x_offset, y_offset, true, &mut coord, &v9);
                mesh_data.solid_verts.push(coord[0]);
                mesh_data.solid_verts.push(coord[2]);
                mesh_data.solid_verts.push(coord[1]);
            }
            for i in 0..V8_SIZE_SQ {
                terrain_builder_get_height_coord(i, x_offset, y_offset, false, &mut coord, &v8);
                mesh_data.solid_verts.push(coord[0]);
                mesh_data.solid_verts.push(coord[2]);
                mesh_data.solid_verts.push(coord[1]);
            }

            let mut indices = [0; 3];
            for i in portion.get_loop_vars() {
                for j in [Spot::Top, Spot::Right, Spot::Left, Spot::Bottom] {
                    j.get_height_triangle(i, false, &mut indices);
                    ttriangles.push(indices[2] + i32::try_from(count).unwrap());
                    ttriangles.push(indices[1] + i32::try_from(count).unwrap());
                    ttriangles.push(indices[0] + i32::try_from(count).unwrap());
                }
            }
        }

        // liquid data
        if let Some(liq_data) = &map_file.map_liquid_data {
            let count = mesh_data.liquid_verts.len() / 3;
            let x_offset = (tile_x as f32 - 32.0) * GRID_SIZE;
            let y_offset = (tile_y as f32 - 32.0) * GRID_SIZE;

            let mut coord = [0.0; 3];
            // generate coordinates
            match &liq_data.liquid_height_details {
                Ok(liquid_map) => {
                    // j keeps track of the current index of liquid_map
                    let mut j = 0;
                    for i in 0..V9_SIZE_SQ {
                        let (row, col) = row_vector_to_matrix_index!(S: (V9_SIZE, V9_SIZE), i);

                        // liquid_map height and width can be smaller than V9
                        // The ones that arent inside are pushed as dummy verts
                        let (height, width) = liquid_map.shape();
                        if row < liq_data.offset_y as usize
                            || row >= liq_data.offset_y as usize + height
                            || col < liq_data.offset_x as usize
                            || col >= liq_data.offset_x as usize + width
                        {
                            // dummy vert using invalid height
                            mesh_data.liquid_verts.push((x_offset + col as f32 * GRID_PART_SIZE) * -1.0);
                            mesh_data.liquid_verts.push(self.cfg.db2_and_map_extract.use_min_height);
                            mesh_data.liquid_verts.push((y_offset + row as f32 * GRID_PART_SIZE) * -1.0);
                            continue;
                        }
                        terrain_builder_get_liquid_coord(i, j, x_offset, y_offset, &mut coord, liquid_map);
                        mesh_data.liquid_verts.push(coord[0]);
                        mesh_data.liquid_verts.push(coord[2]);
                        mesh_data.liquid_verts.push(coord[1]);
                        j += 1;
                    }
                },
                Err(liquid_level) => {
                    for i in 0..V9_SIZE_SQ {
                        let (row, col) = row_vector_to_matrix_index!(S: (V9_SIZE, V9_SIZE), i);
                        mesh_data.liquid_verts.push((x_offset + col as f32 * GRID_PART_SIZE) * -1.0);
                        mesh_data.liquid_verts.push(*liquid_level);
                        mesh_data.liquid_verts.push((y_offset + row as f32 * GRID_PART_SIZE) * -1.0);
                    }
                },
            }

            let mut indices = [0; 3];
            // generate triangles
            for i in portion.get_loop_vars() {
                for j in [Spot::Top, Spot::Bottom] {
                    j.get_height_triangle(i, true, &mut indices);
                    ltriangles.push(indices[2] + i32::try_from(count).unwrap());
                    ltriangles.push(indices[1] + i32::try_from(count).unwrap());
                    ltriangles.push(indices[0] + i32::try_from(count).unwrap());
                }
            }
        }

        // now that we have gathered the data, we can figure out which parts to keep:
        // liquid above ground, ground above liquid
        let ltris = &ltriangles;
        let ttris = &ttriangles;

        if ltriangles.is_empty() && ttriangles.is_empty() {
            return Err(az_error!("No triangle indices found in map"));
        }

        // make a copy of liquid vertices
        // used to pad right-bottom frame due to lost vertex data at extraction
        let lverts_copy = mesh_data.liquid_verts.clone();

        let mut l_idx = 0;
        let mut t_idx = 0;
        for i in portion.get_loop_vars() {
            for _ in 0..2 {
                // default is true, will change to false if needed
                let mut use_terrain = true;
                let mut use_liquid = true;
                let mut liquid_type = 0;

                // if there is no liquid, don't use liquid
                if mesh_data.liquid_verts.is_empty() || ltriangles.is_empty() {
                    use_liquid = false;
                } else {
                    let map_liq_flag = terrain_builder_get_liquid_type(i, &map_file);
                    if (map_liq_flag & MapLiquidTypeFlag::DarkWater).bits() > 0 {
                        // players should not be here, so logically neither should creatures
                        use_terrain = false;
                        use_liquid = false;
                    } else if (map_liq_flag & (MapLiquidTypeFlag::Water | MapLiquidTypeFlag::Ocean)).bits() > 0 {
                        liquid_type = MmapNavTerrainFlag::Water.area_id();
                    } else if (map_liq_flag & (MapLiquidTypeFlag::Magma | MapLiquidTypeFlag::Slime)).bits() > 0 {
                        liquid_type = MmapNavTerrainFlag::MagmaSlime.area_id();
                    } else {
                        use_liquid = false;
                    }
                }

                // if there is no terrain, don't use terrain
                if ttriangles.is_empty() {
                    use_terrain = false;
                }

                // while extracting ADT data we are losing right-bottom vertices
                // this code adds fair approximation of lost data
                if use_liquid {
                    let mut quad_height = 0.0;
                    let mut valid_count = 0;
                    for idx in 0..3 {
                        let h = lverts_copy[ltris[l_idx + idx] as usize * 3 + 1];
                        if h != self.cfg.db2_and_map_extract.use_min_height && h < INVALID_MAP_LIQ_HEIGHT_MAX {
                            quad_height += h;
                            valid_count += 1;
                        }
                    }

                    // update vertex height data
                    if valid_count > 0 && valid_count < 3 {
                        quad_height /= valid_count as f32;
                        for idx in 0..3 {
                            let h = mesh_data.liquid_verts[ltris[l_idx + idx] as usize * 3 + 1];
                            if h == self.cfg.db2_and_map_extract.use_min_height || h > INVALID_MAP_LIQ_HEIGHT_MAX {
                                mesh_data.liquid_verts[ltris[l_idx + idx] as usize * 3 + 1] = quad_height;
                            }
                        }
                    }

                    // no valid vertexes - don't use this poly at all
                    if valid_count == 0 {
                        use_liquid = false;
                    }
                }

                // if there is a hole here, don't use the terrain
                if use_terrain {
                    use_terrain = !terrain_builder_is_hole(i, &map_file);
                }

                // we use only one terrain kind per quad - pick higher one
                if use_terrain && use_liquid {
                    let mut min_l_level = INVALID_MAP_LIQ_HEIGHT_MAX;
                    let mut max_l_level = self.cfg.db2_and_map_extract.use_min_height;
                    for x in 0..3 {
                        let h = mesh_data.liquid_verts[ltris[l_idx + x] as usize * 3 + 1];
                        if min_l_level > h {
                            min_l_level = h;
                        }

                        if max_l_level < h {
                            max_l_level = h;
                        }
                    }

                    let mut max_t_level = self.cfg.db2_and_map_extract.use_min_height;
                    let mut min_t_level = INVALID_MAP_LIQ_HEIGHT_MAX;
                    for x in 0..6 {
                        let h = mesh_data.solid_verts[ttris[t_idx + x] as usize * 3 + 1];
                        if max_t_level < h {
                            max_t_level = h;
                        }

                        if min_t_level > h {
                            min_t_level = h;
                        }
                    }

                    // terrain under the liquid?
                    if min_l_level > max_t_level {
                        use_terrain = false;
                    }

                    //liquid under the terrain?
                    if min_t_level > max_l_level {
                        use_liquid = false;
                    }
                }

                // store the result
                if use_liquid {
                    mesh_data.liquid_types.push(liquid_type);
                    for k in 0..3 {
                        mesh_data.liquid_tris.push(ltris[l_idx + k]);
                    }
                }

                if use_terrain {
                    for k in 0..6 {
                        mesh_data.solid_tris.push(ttris[t_idx + k]);
                    }
                }

                // advance to next set of triangles
                l_idx += 3;
                t_idx += 6;
            }
        }
        if mesh_data.solid_tris.is_empty() && mesh_data.liquid_tris.is_empty() {
            Err(az_error!("No mesh triangle data found when loading map spot"))
        } else {
            Ok(())
        }
    }

    pub fn load_vmap(&mut self, map_id: u32, tile_x: u16, tile_y: u16, mesh_data: &mut MeshData) -> AzResult<()> {
        if let Err(e) = self.vmap_mgr.load_single_map_tile(map_id, tile_x, tile_y, true) {
            debug!("Unable to load vmap tile. Tile reference may have been from Map instead; err was {e}");
            return Ok(());
        };

        let tree = self.vmap_mgr.instance_map_tree(map_id).unwrap();
        for instance in tree.tile_model_instances(tile_x, tile_y) {
            let Some(world_model) = self.vmap_mgr.model_store.loaded_model_files.get(&instance.model) else {
                continue;
            };
            // model instances exist in tree even though there are instances of that model in this tile
            let group_models = world_model.group_models.as_slice();

            // all M2s need to have triangle indices reversed
            let is_m2 = instance.flags.contains(ModelFlags::ModM2);

            // transform data
            let scale = instance.i_scale;
            let rotation = matrix3_from_euler_angles_xyz(-instance.i_rot.z.to_radians(), -instance.i_rot.x.to_radians(), -instance.i_rot.y.to_radians());
            let mut position = instance.i_pos;
            position.x -= 32.0 * GRID_SIZE;
            position.y -= 32.0 * GRID_SIZE;

            for g in group_models {
                // first handle collision mesh
                // Similar to TerrainBuilder::transform
                let transformed_vertices = g.mesh.iter().flat_map(|mesh| {
                    mesh.vertices().iter().map(|v| {
                        // apply tranform, then mirror along the horizontal axes
                        let mut v = rotation.transpose() * v * scale + position;
                        v.x *= -1.0;
                        v.y *= -1.0;
                        v
                    })
                });

                let offset = i32::try_from(mesh_data.solid_verts.len() / 3).unwrap();

                // Similar to TerrainBuilder::copyVertices
                for v in transformed_vertices {
                    mesh_data.solid_verts.push(v.y);
                    mesh_data.solid_verts.push(v.z);
                    mesh_data.solid_verts.push(v.x);
                }
                // Similar to TerrainBuilder::copyIndices
                for tri in g.mesh.iter().flat_map(|mesh| mesh.indices().iter()) {
                    // Flip if its an M2
                    let tri = tri.map(|f| i32::try_from(f).unwrap());
                    if is_m2 {
                        mesh_data.solid_tris.push(tri[2] + offset);
                        mesh_data.solid_tris.push(tri[1] + offset);
                        mesh_data.solid_tris.push(tri[0] + offset);
                    } else {
                        mesh_data.solid_tris.push(tri[0] + offset);
                        mesh_data.solid_tris.push(tri[1] + offset);
                        mesh_data.solid_tris.push(tri[2] + offset);
                    }
                }

                // now handle liquid data
                if let Some(liq) = g.i_liquid.as_ref() {
                    let (data, tile_flags, corner) = match &liq.heights {
                        Err(_) => continue,
                        Ok(f) => (&f.i_height, &f.i_flags, &f.i_corner),
                    };
                    // convert liquid type to NavTerrain
                    let liquid_flags = self.vmap_mgr.helper.liquid_flags_getter.get_liquid_flags(liq.i_type);
                    let typ = if (liquid_flags & (MapLiquidTypeFlag::Water | MapLiquidTypeFlag::Ocean)).bits() > 0 {
                        MmapNavTerrainFlag::Water.area_id()
                    } else if (liquid_flags & (MapLiquidTypeFlag::Magma | MapLiquidTypeFlag::Slime)).bits() > 0 {
                        MmapNavTerrainFlag::MagmaSlime.area_id()
                    } else {
                        0
                    };

                    // indexing is weird...
                    // after a lot of trial and error, this is what works:
                    // vertex = y*vertsX+x
                    // tile   = x*tilesY+y
                    // flag   = y*tilesY+x

                    let mut liq_verts = vec![];
                    let mut liq_tris = vec![];
                    let (verts_y, verts_x) = data.shape();
                    for x in 0..verts_x {
                        for y in 0..verts_y {
                            let vert = Vector3::new(corner.x + x as f32 * GRID_PART_SIZE, corner.y + y as f32 * GRID_PART_SIZE, data[(y, x)]);
                            let mut vert = rotation.transpose() * vert * scale + position;
                            vert.x *= -1.0;
                            vert.y *= -1.0;
                            liq_verts.push(vert);
                        }
                    }

                    let (tiles_y, tiles_x) = tile_flags.shape();
                    for x in 0..tiles_x {
                        for y in 0..tiles_y {
                            if (tile_flags[(y, x)] & 0x0f) == 0x0f {
                                // Should be related to https://wowdev.wiki/WMO#MLIQ_chunk `liquidTileList`
                                // which takes it documentation from https://wowdev.wiki/ADT/v18#MH2O_chunk_.28WotLK.2B.29
                                //
                                // 0x0f or 0x8 mean don't render (?, TC: 0xF)
                                // &0xf: liquid type (1: ocean, 3: slime, 4: river, 6: magma)
                                // 0x10:
                                // 0x20:
                                // 0x40: not low depth (forced swimming ?)
                                // 0x80: fatigue (?, TC: yes)
                                continue;
                            }
                            let square = x * tiles_y + y;
                            let idx1 = square + x;
                            let idx2 = square + 1 + x;
                            let idx3 = square + tiles_y + 1 + 1 + x;
                            let idx4 = square + tiles_y + 1 + x;

                            // top triangle
                            liq_tris.push(idx3 as u16);
                            liq_tris.push(idx2 as u16);
                            liq_tris.push(idx1 as u16);
                            // bottom triangle
                            liq_tris.push(idx4 as u16);
                            liq_tris.push(idx3 as u16);
                            liq_tris.push(idx1 as u16);
                        }
                    }
                    let liq_offset = i32::try_from(mesh_data.liquid_verts.len() / 3).unwrap();
                    for liq_vert in liq_verts {
                        mesh_data.liquid_verts.push(liq_vert.y);
                        mesh_data.liquid_verts.push(liq_vert.z);
                        mesh_data.liquid_verts.push(liq_vert.x);
                    }
                    for i in 0..(liq_tris.len() / 3) {
                        mesh_data.liquid_tris.push(i32::from(liq_tris[i * 3 + 1]) + liq_offset);
                        mesh_data.liquid_tris.push(i32::from(liq_tris[i * 3 + 2]) + liq_offset);
                        mesh_data.liquid_tris.push(i32::from(liq_tris[i * 3]) + liq_offset);
                        mesh_data.liquid_types.push(typ);
                    }
                }
            }
        }

        self.vmap_mgr.unload_single_map_tile(map_id, tile_x, tile_y);

        Ok(())
    }
}

fn terrain_builder_get_height_coord<const R: usize, const C: usize>(
    index: usize,
    x_offset: f32,
    y_offset: f32,
    query_v9: bool,
    coord: &mut [f32; 3],
    v: &SMatrix<f32, R, C>,
) {
    let (row_idx, col_idx) = row_vector_to_matrix_index!(v, index);
    // wow coords: x, y, height
    // coord is mirroed about the horizontal axes
    if query_v9 {
        // index%(V9_SIZE)
        coord[0] = (x_offset + col_idx as f32 * GRID_PART_SIZE) * -1.0;
        // (int)(index/(V9_SIZE))
        coord[1] = (y_offset + row_idx as f32 * GRID_PART_SIZE) * -1.0;
        coord[2] = v[(row_idx, col_idx)];
    } else {
        // index%(V8_SIZE)
        coord[0] = (x_offset + col_idx as f32 * GRID_PART_SIZE + GRID_PART_SIZE / 2.0) * -1.0;
        // (int)(index/(V8_SIZE))
        coord[1] = (y_offset + row_idx as f32 * GRID_PART_SIZE + GRID_PART_SIZE / 2.0) * -1.0;
        coord[2] = v[(row_idx, col_idx)];
    }
}

fn terrain_builder_get_liquid_coord(index: usize, index2: usize, x_offset: f32, y_offset: f32, coord: &mut [f32; 3], v: &DMatrix<f32>) {
    // wow coords: x, y, height
    // coord is mirroed about the horizontal axes
    let (row, col) = row_vector_to_matrix_index!(S: (V9_SIZE, V9_SIZE), index);

    coord[0] = (x_offset + col as f32 * GRID_PART_SIZE) * -1.0;
    coord[1] = (y_offset + row as f32 * GRID_PART_SIZE) * -1.0;
    coord[2] = v[row_vector_to_matrix_index!(v, index2)];
}

/**************************************************************************/
fn terrain_builder_is_hole(square: usize, map_file: &MapFile) -> bool {
    if let Some(holes) = map_file.map_holes {
        let (row, col) = row_vector_to_matrix_index!(S: (V8_SIZE, V8_SIZE), square);
        // 8 squares per cell
        let cell_row = row / 8;
        let cell_col = col / 8;
        let hole_row = row % 8;
        let hole_col = square - (row * 128 + cell_col * 8);

        (holes[cell_row][cell_col][hole_row] & (1 << hole_col)) != 0
    } else {
        false
    }
}

/**************************************************************************/
fn terrain_builder_get_liquid_type(square: usize, map_file: &MapFile) -> FlagSet<MapLiquidTypeFlag> {
    if let Some(liq_data) = &map_file.map_liquid_data {
        let (row, col) = row_vector_to_matrix_index!(S: (V8_SIZE, V8_SIZE), square);
        // 8 squares per cell
        let (cell_row, cell_col) = (row / 8, col / 8);
        let (_, map_liq_flag) = liq_data.get_liquid_entry_flags(cell_row, cell_col);
        map_liq_flag
    } else {
        None.into()
    }
}
