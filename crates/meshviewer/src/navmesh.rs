use std::{fs, path::Path};

use azothacore_common::{
    AzResult,
    az_error,
    collision::maps::map_defines::{MmapNavTerrainFlag, MmapTileFile},
    recastnavigation_handles::{DetourNavMesh, DetourNavMeshParams, DetourNavMeshQuery, DetourStatusDetails},
    utils::{bincode_deserialise, buffered_file_open},
};
use flagset::FlagSet;
use recastnavigation_sys::{dtPolyRef, dtQueryFilter};

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
    // On a partial corridor Detour clamps `end` to the last polygon itself.
    let (points, _) = query
        .find_straight_path(&start, &end, &corridor, MAX_PATH_POINTS)
        .map_err(|e| format!("findStraightPath failed: {e}"))?;
    Ok(PathResult {
        start,
        end,
        corridor,
        points,
        partial,
    })
}
