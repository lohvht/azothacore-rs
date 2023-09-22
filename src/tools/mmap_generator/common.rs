use std::{collections::BTreeMap, fs, path::Path};

use nalgebra::Vector6;
use recastnavigation_sys::rcCalcBounds;

use crate::{
    server::{
        game::map::{ADT_GRID_SIZE, ADT_GRID_SIZE_PLUS_ONE},
        shared::recastnavigation_handles::DetourNavMeshParams,
    },
    AzResult,
};

pub const V9_SIZE: usize = ADT_GRID_SIZE_PLUS_ONE;
pub const V9_SIZE_SQ: usize = V9_SIZE * V9_SIZE;
pub const V8_SIZE: usize = ADT_GRID_SIZE;
pub const V8_SIZE_SQ: usize = V8_SIZE * V8_SIZE;
pub const GRID_SIZE: f32 = 1600.0 / 3.0;
pub const GRID_PART_SIZE: f32 = GRID_SIZE / V8_SIZE as f32;

// // see extractor.toml, db2_and_map_extract.use_min_height
// static const float self.use_min_height = -2000.f;
pub const INVALID_MAP_LIQ_HEIGHT_MAX: f32 = 5000.0;

pub fn get_tile_bounds(tile_x: u16, tile_y: u16, verts: &Vec<f32>, bmin: &mut [f32; 3], bmax: &mut [f32; 3]) {
    // this is for elevation
    if !verts.is_empty() {
        unsafe { rcCalcBounds(verts.as_ptr(), verts.len() as i32 / 3, bmin.as_mut_ptr(), bmax.as_mut_ptr()) };
    } else {
        bmin[1] = f32::EPSILON;
        bmax[1] = f32::MAX;
    }

    // this is for width and depth
    bmax[0] = (32 - tile_x as i32) as f32 * GRID_SIZE;
    bmax[2] = (32 - tile_y as i32) as f32 * GRID_SIZE;
    bmin[0] = bmax[0] - GRID_SIZE;
    bmin[2] = bmax[2] - GRID_SIZE;
}

pub struct TileInfo {
    pub map_id:          u32,
    pub tile_x:          u16,
    pub tile_y:          u16,
    pub nav_mesh_params: DetourNavMeshParams,
}

pub fn load_off_mesh_connections<P: AsRef<Path>>(
    map_id: u32,
    tile_x: u16,
    tile_y: u16,
    offmesh_path: Option<P>,
    mesh_data: &mut MeshData,
) -> AzResult<()> {
    // no meshfile input given?
    let offmesh_path = match offmesh_path {
        None => return Ok(()),
        Some(f) => f,
    };
    let buf = fs::read_to_string(&offmesh_path)?;

    // pretty silly thing, as we parse entire file and load only the tile we need
    // but we don't expect this file to be too large
    for l in buf.split('\n') {
        let scanned = match sscanf::sscanf!(
            l,
            "{} {},{} ({} {} {}) ({} {} {}) {}",
            u32,
            u16,
            u16,
            f32,
            f32,
            f32,
            f32,
            f32,
            f32,
            f32
        ) {
            Ok(scanned) => scanned,
            Err(_) => continue,
        };
        let (mid, tx, ty, p0x, p0y, p0z, p1x, p1y, p1z, size) = scanned;
        if map_id == mid && tile_x == tx && tile_y == ty {
            mesh_data.offset_mesh_connections.push(Vector6::new(p0y, p0z, p0x, p1y, p1z, p1x));
            mesh_data.offset_mesh_connection_dirs.push(1); // 1 - both direction, 0 - one sided
            mesh_data.offset_mesh_connection_rads.push(size); // agent size equivalent
                                                              // can be used same way as polygon flags
            mesh_data.offset_mesh_connections_areas.push(0xFF);
            mesh_data.offset_mesh_connections_flags.push(0xFF); // all movement masks can make this path
        }
    }

    Ok(())
}

// see following files:
// contrib/extractor/system.cpp
// src/game/Map.cpp
#[derive(Default)]
pub struct MeshData {
    pub solid_verts: Vec<f32>,
    pub solid_tris:  Vec<i32>,

    pub liquid_verts: Vec<f32>,
    pub liquid_tris:  Vec<i32>,
    pub liquid_types: Vec<u8>,

    // offmesh connection data
    /// [p0y,p0z,p0x,p1y,p1z,p1x] - per connection
    pub offset_mesh_connections:       Vec<Vector6<f32>>,
    pub offset_mesh_connection_rads:   Vec<f32>,
    pub offset_mesh_connection_dirs:   Vec<u8>,
    pub offset_mesh_connections_areas: Vec<u8>,
    pub offset_mesh_connections_flags: Vec<u16>,
}

pub fn clean_vertices(verts: &mut Vec<f32>, tris: &mut [i32]) {
    // collect all the vertex indices from triangle
    let mut vert_map = BTreeMap::new();
    let mut cleaned_verts = vec![];
    let mut count = 0i32;
    for t in tris.iter() {
        if vert_map.get(t).is_some() {
            continue;
        }
        vert_map.insert(*t, count);

        let index: usize = (*t).try_into().unwrap();
        cleaned_verts.push(verts[index * 3]);
        cleaned_verts.push(verts[index * 3 + 1]);
        cleaned_verts.push(verts[index * 3 + 2]);
        count += 1;
    }
    verts.clear();
    verts.append(&mut cleaned_verts);
    // update triangles to use new indices
    for tri in tris.iter_mut() {
        if let Some(new_t) = vert_map.get(tri) {
            *tri = *new_t;
        }
    }
}
