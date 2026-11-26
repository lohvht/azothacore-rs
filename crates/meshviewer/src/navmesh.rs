use std::{fs, path::Path};

use azothacore_common::{
    AzResult,
    az_error,
    collision::maps::map_defines::{MmapNavTerrainFlag, MmapTileFile},
    recastnavigation_handles::{DetourNavMesh, DetourNavMeshParams, DetourNavMeshQuery, DetourStatusDetails},
    utils::{bincode_deserialise, buffered_file_open},
};
use flagset::FlagSet;
use recastnavigation_sys::{dtPolyRef, dtQueryFilter, dtStraightPathFlags_DT_STRAIGHTPATH_END, dtStraightPathFlags_DT_STRAIGHTPATH_OFFMESH_CONNECTION};

/// Loads `{map_id}.mmap` and its `.mmtile` files. `tiles` holds (x, y) pairs; empty loads every tile.
pub fn load_navmesh(mmaps_dir: &Path, map_id: u32, tiles: &[(u16, u16)]) -> AzResult<(DetourNavMesh, usize)> {
    let params: DetourNavMeshParams = bincode_deserialise(&mut buffered_file_open(mmaps_dir.join(format!("{map_id:04}.mmap")))?)?;
    let (mut nav, _) = DetourNavMesh::init(&params)?;

    let prefix = format!("{map_id:04}");
    let mut loaded = 0;
    for entry in fs::read_dir(mmaps_dir)? {
        let name = entry?.file_name();
        let Some(stem) = name.to_str().and_then(|n| n.strip_suffix(".mmtile")) else {
            continue;
        };
        let Some(yx) = stem.strip_prefix(&prefix).filter(|s| s.len() == 4) else {
            continue;
        };
        let (Ok(tile_y), Ok(tile_x)) = (yx[..2].parse::<u16>(), yx[2..].parse::<u16>()) else {
            continue;
        };
        if !tiles.is_empty() && !tiles.contains(&(tile_x, tile_y)) {
            continue;
        }
        let file = MmapTileFile::read_from_mmtile(mmaps_dir, map_id, tile_y, tile_x)?;
        nav.add_tile(file.data.bytes).map_err(|e| az_error!("tile [{tile_x},{tile_y}]: {e}"))?;
        loaded += 1;
    }
    Ok((nav, loaded))
}

/// Same search boxes as TC's `PathGenerator::GetPolyByLocation`: close by first, then up to 50 yd vertically.
const NEAREST_POLY_EXTENTS: [[f32; 3]; 2] = [[3.0, 5.0, 3.0], [3.0, 50.0, 3.0]];
const MAX_PATH_POLYS: usize = 4096;
const MAX_PATH_POINTS: usize = 4096;
const SMOOTH_PATH_STEP_SIZE: f32 = 4.0;
const SMOOTH_PATH_SLOP: f32 = 0.3;
const MAX_VISIT_POLY: usize = 16;

pub struct PathResult {
    pub start:    [f32; 3],
    pub end:      [f32; 3],
    pub corridor: Vec<dtPolyRef>,
    pub points:   Vec<[f32; 3]>,
    pub partial:  bool,
}

pub fn find_path(nav: &DetourNavMesh, start: [f32; 3], end: [f32; 3]) -> Result<PathResult, String> {
    let (query, _) = DetourNavMeshQuery::init(nav, 65535).map_err(|e| format!("query init failed: {e}"))?;
    let filter = dtQueryFilter {
        m_areaCost:     [1.0; 64],
        m_includeFlags: FlagSet::<MmapNavTerrainFlag>::full().bits(),
        m_excludeFlags: 0,
    };
    let nearest = |pos: [f32; 3], what: &str| {
        for extents in &NEAREST_POLY_EXTENTS {
            if let Ok(((poly, point), _)) = query.find_nearest_poly(&pos, extents, &filter)
                && poly != 0
            {
                return Ok((poly, point));
            }
        }
        Err(format!("no navmesh polygon near the {what} point"))
    };
    let (start_ref, start) = nearest(start, "start")?;
    let (end_ref, end) = nearest(end, "end")?;

    let (corridor, status) = query
        .find_path(start_ref, end_ref, &start, &end, &filter, MAX_PATH_POLYS)
        .map_err(|e| format!("findPath failed: {e}"))?;
    let partial = status.details.contains(DetourStatusDetails::PartialResult);
    let points = find_smooth_path(nav, &query, &filter, &start, &end, &corridor)?;
    Ok(PathResult {
        start,
        end,
        corridor,
        points,
        partial,
    })
}

/// TC's `PathGenerator::FindSmoothPath`, the path the server sends by default. Detour's straight
/// path only has the corners, so its segments cut through hills between them.
fn find_smooth_path(
    nav: &DetourNavMesh,
    query: &DetourNavMeshQuery,
    filter: &dtQueryFilter,
    start: &[f32; 3],
    end: &[f32; 3],
    corridor: &[dtPolyRef],
) -> Result<Vec<[f32; 3]>, String> {
    let mut polys = corridor.to_vec();
    let boundary = |poly, pos| {
        query
            .closest_point_on_poly_boundary(poly, pos)
            .map(|(p, _)| p)
            .map_err(|e| format!("closestPointOnPolyBoundary failed: {e}"))
    };
    let mut iter_pos = boundary(polys[0], start)?;
    let target_pos = boundary(polys[polys.len() - 1], end)?;
    let mut points = vec![iter_pos];

    while !polys.is_empty() && points.len() < MAX_PATH_POINTS {
        let Some((steer_pos, steer_flags, steer_ref)) = steer_target(query, &iter_pos, &target_pos, &polys) else {
            break;
        };
        let end_of_path = steer_flags & dtStraightPathFlags_DT_STRAIGHTPATH_END as u8 != 0;
        let off_mesh = steer_flags & dtStraightPathFlags_DT_STRAIGHTPATH_OFFMESH_CONNECTION as u8 != 0;

        let delta = sub(&steer_pos, &iter_pos);
        let len = dot(&delta, &delta).sqrt();
        // Don't step past the end of the path or an off-mesh link.
        let scale = if (end_of_path || off_mesh) && len < SMOOTH_PATH_STEP_SIZE {
            1.0
        } else {
            SMOOTH_PATH_STEP_SIZE / len
        };
        let move_target = [0, 1, 2].map(|i| iter_pos[i] + delta[i] * scale);

        let (mut result, visited) = match query.move_along_surface(polys[0], &iter_pos, &move_target, filter, MAX_VISIT_POLY) {
            Ok((m, _)) => (m.pos, m.visited),
            Err(_) => (iter_pos, vec![]),
        };
        fixup_corridor(&mut polys, &visited);
        if let Ok((h, _)) = query.get_poly_height(polys[0], &result) {
            result[1] = h;
        }
        result[1] += 0.5;
        iter_pos = result;

        if end_of_path && in_range_yzx(&iter_pos, &steer_pos, SMOOTH_PATH_SLOP, 1.0) {
            points.push(target_pos);
            break;
        } else if off_mesh && in_range_yzx(&iter_pos, &steer_pos, SMOOTH_PATH_SLOP, 1.0) {
            // Advance the corridor up to and over the off-mesh connection.
            let mut prev_ref = 0;
            let mut poly_ref = polys[0];
            let mut npos = 0;
            while npos < polys.len() && poly_ref != steer_ref {
                prev_ref = poly_ref;
                poly_ref = polys[npos];
                npos += 1;
            }
            polys.drain(..npos);
            if let Some((connection_start, connection_end)) = nav.off_mesh_connection_end_points(prev_ref, poly_ref) {
                points.push(connection_start);
                iter_pos = connection_end;
                if let Some(&poly) = polys.first()
                    && let Ok((h, _)) = query.get_poly_height(poly, &iter_pos)
                {
                    iter_pos[1] = h;
                }
                iter_pos[1] += 0.5;
            }
        }
        points.push(iter_pos);
    }
    Ok(points)
}

/// TC's `PathGenerator::GetSteerTarget`: the first straight path corner further than `SMOOTH_PATH_SLOP`
/// away, at the current height.
fn steer_target(query: &DetourNavMeshQuery, start: &[f32; 3], end: &[f32; 3], polys: &[dtPolyRef]) -> Option<([f32; 3], u8, dtPolyRef)> {
    let (steer_path, _) = query.find_straight_path(start, end, polys, 3).ok()?;
    let p = steer_path
        .iter()
        .find(|p| p.flags & dtStraightPathFlags_DT_STRAIGHTPATH_OFFMESH_CONNECTION as u8 != 0 || !in_range_yzx(&p.pos, start, SMOOTH_PATH_SLOP, 1000.0))?;
    let mut pos = p.pos;
    pos[1] = start[1];
    Some((pos, p.flags, p.poly))
}

/// TC's `PathGenerator::FixupCorridor`: replaces the corridor up to the furthest visited polygon with
/// the polygons `moveAlongSurface` walked through.
fn fixup_corridor(path: &mut Vec<dtPolyRef>, visited: &[dtPolyRef]) {
    let Some((furthest_path, furthest_visited)) = path
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, p)| visited.iter().position(|v| v == p).map(|j| (i, j)))
    else {
        return;
    };
    let rest = path.split_off(furthest_path + 1);
    path.clear();
    path.extend(visited[furthest_visited..].iter().rev());
    path.extend(rest);
    path.truncate(MAX_PATH_POLYS);
}

fn in_range_yzx(a: &[f32; 3], b: &[f32; 3], r: f32, h: f32) -> bool {
    let (dx, dy, dz) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
    dx * dx + dz * dz < r * r && dy.abs() < h
}

fn sub(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
