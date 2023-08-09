use std::{collections::HashMap, fs, path::Path};

use bvh::aabb::AABB;
use nalgebra::{Vector3, Vector6};

use crate::{
    common::collision::maps::map_defines::MmapNavTerrainFlag,
    server::shared::recastnavigation_handles::{DetourNavMesh, DetourNavMeshParams},
    tools::adt::{ADT_GRID_SIZE, ADT_GRID_SIZE_PLUS_ONE},
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

pub fn get_tile_bounds(tile_x: u16, tile_y: u16, verts: &Vec<Vector3<f32>>) -> AABB {
    // this is for elevation
    let (min_elevation, max_elevation) = if verts.is_empty() {
        verts
            .iter()
            .fold((f32::MAX, f32::MIN), |(min, max), v| (min.min(v.y), max.max(v.y)))
    } else {
        (0.0, f32::MAX)
    };
    // this is for width and depth
    let bmax = Vector3::new((32 - tile_x) as f32 * GRID_SIZE, max_elevation, (32 - tile_y) as f32 * GRID_SIZE);
    let bmin = Vector3::new(bmax.x - GRID_SIZE, min_elevation, bmax.z - GRID_SIZE);
    AABB {
        min: bmax.into(),
        max: bmin.into(),
    }
}

pub struct TileInfo {
    pub map_id:          u32,
    pub tile_x:          u16,
    pub tile_y:          u16,
    pub nav_mesh_params: DetourNavMeshParams,
}

pub fn load_off_mesh_connections<P: AsRef<Path>>(map_id: u32, tile_x: u16, tile_y: u16, offmesh_path: Option<P>) -> AzResult<MeshData> {
    // no meshfile input given?
    let offmesh_path = match offmesh_path {
        None => return Ok(MeshData::default()),
        Some(f) => f,
    };
    let buf = fs::read_to_string(&offmesh_path)?;

    let mut offset_mesh_connections = vec![];
    let mut offset_mesh_connection_rads = vec![];
    let mut offset_mesh_connection_dirs = vec![];
    let mut offset_mesh_connections_areas = vec![];
    let mut offset_mesh_connections_flags = vec![];

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
            offset_mesh_connections.push(Vector6::new(p0y, p0z, p0x, p1y, p1z, p1x));
            offset_mesh_connection_dirs.push(1); // 1 - both direction, 0 - one sided
            offset_mesh_connection_rads.push(size); // agent size equivalent
                                                    // can be used same way as polygon flags
            offset_mesh_connections_areas.push(0xFF);
            offset_mesh_connections_flags.push(0xFF); // all movement masks can make this path
        }
    }

    Ok(MeshData {
        offset_mesh_connections,
        offset_mesh_connection_rads,
        offset_mesh_connection_dirs,
        offset_mesh_connections_areas,
        offset_mesh_connections_flags,
        ..Default::default()
    })
}

// see following files:
// contrib/extractor/system.cpp
// src/game/Map.cpp
#[derive(Default)]
pub struct MeshData {
    pub solid_verts: Vec<Vector3<f32>>,
    pub solid_tris:  Vec<Vector3<u16>>,

    pub liquid_verts: Vec<Vector3<f32>>,
    pub liquid_tris:  Vec<Vector3<u16>>,
    pub liquid_types: Vec<Option<MmapNavTerrainFlag>>,

    // offmesh connection data
    /// [p0y,p0z,p0x,p1y,p1z,p1x] - per connection
    pub offset_mesh_connections:       Vec<Vector6<f32>>,
    pub offset_mesh_connection_rads:   Vec<f32>,
    pub offset_mesh_connection_dirs:   Vec<u8>,
    pub offset_mesh_connections_areas: Vec<u8>,
    pub offset_mesh_connections_flags: Vec<u16>,
}

impl MeshData {
    pub fn merge_mesh_data<M>(&mut self, other_mesh_datas: M)
    where
        M: IntoIterator<Item = MeshData>,
    {
        for MeshData {
            solid_verts: mut other_solid_verts,
            solid_tris: other_solid_tris,
            liquid_verts: mut other_liquid_verts,
            liquid_tris: other_liquid_tris,
            liquid_types: mut other_liquid_types,
            offset_mesh_connections: mut other_offset_mesh_connections,
            offset_mesh_connection_rads: mut other_offset_mesh_connection_rads,
            offset_mesh_connection_dirs: mut other_offset_mesh_connection_dirs,
            offset_mesh_connections_areas: mut other_offset_mesh_connections_areas,
            offset_mesh_connections_flags: mut other_offset_mesh_connections_flags,
        } in other_mesh_datas
        {
            // Get the vertices counts first, this is mainly to ensure that the  triangles that reference
            // these vertices later have the correct indices.
            let solid_verts_counts = self.solid_verts.len();
            let liquid_verts_counts = self.liquid_verts.len();

            self.solid_verts.append(&mut other_solid_verts);
            self.liquid_verts.append(&mut other_liquid_verts);
            self.liquid_types.append(&mut other_liquid_types);
            self.offset_mesh_connections.append(&mut other_offset_mesh_connections);
            self.offset_mesh_connection_rads.append(&mut other_offset_mesh_connection_rads);
            self.offset_mesh_connection_dirs.append(&mut other_offset_mesh_connection_dirs);
            self.offset_mesh_connections_areas.append(&mut other_offset_mesh_connections_areas);
            self.offset_mesh_connections_flags.append(&mut other_offset_mesh_connections_flags);

            // Handle triangle indices separately. these are indices to the triangle
            self.solid_tris
                .extend(other_solid_tris.into_iter().map(|t| t.add_scalar(solid_verts_counts as u16)));
            self.liquid_tris
                .extend(other_liquid_tris.into_iter().map(|t| t.add_scalar(liquid_verts_counts as u16)));
        }
    }
}

pub fn clean_vertices(verts: &mut Vec<Vector3<f32>>, tris: &mut [Vector3<u16>]) {
    // collect all the vertex indices from triangle
    let mut cleaned_verts = vec![];
    let mut vert_map = HashMap::new();
    let mut count = 0u16;
    for tri in tris.iter() {
        for t in tri {
            if vert_map.get(t).is_some() {
                continue;
            }
            vert_map.insert(*t, count);
            cleaned_verts.push(verts[usize::from(*t)]);
            count += 1;
        }
    }
    verts.clear();
    verts.append(&mut cleaned_verts);
    // update triangles to use new indices
    for tri in tris.iter_mut() {
        for t in tri {
            if let Some(new_t) = vert_map.get(t) {
                *t = *new_t;
            }
        }
    }
}
