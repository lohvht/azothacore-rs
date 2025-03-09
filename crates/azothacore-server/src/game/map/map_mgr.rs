use bevy::{
    prelude::Resource,
    time::{Timer, TimerMode},
};

use crate::game::{
    grid::grid_defines::{compute_grid_coord, MAX_NUMBER_OF_GRIDS},
    map::{GridMap, Map},
    world::WorldConfig,
};

pub struct MapMgr {
    grid_update_timer: Timer,
}

#[derive(Resource)]
pub struct GridCleanupTimer(Timer);

impl From<&WorldConfig> for GridCleanupTimer {
    fn from(cfg: &WorldConfig) -> Self {
        Self(Timer::new(*cfg.GridCleanUpDelay, TimerMode::Repeating))
    }
}

#[derive(Resource)]
pub struct MapUpdateTimer(Timer);

impl From<&WorldConfig> for MapUpdateTimer {
    fn from(cfg: &WorldConfig) -> Self {
        Self(Timer::new(*cfg.MapUpdateInterval, TimerMode::Repeating))
    }
}

impl MapMgr {
    /// ExistMapAndVMap
    fn exist_map_and_vmap(cfg: &WorldConfig, map_id: u32, x: f32, y: f32) -> bool {
        let p = compute_grid_coord(x, y);
        let grid_x = (MAX_NUMBER_OF_GRIDS - 1) - p.x_coord;
        let grid_y = (MAX_NUMBER_OF_GRIDS - 1) - p.y_coord;

        todo!()
        // NOTE: Maybe replace Map::ExistMap w/ the one below? its clearer that we're checking on this
        // GridMap::exists(cfg.maps_dir(), map_id, grid_x, grid_y)?;

        // return Map::exist_map(cfg, map_id, grid_x, grid_y) && Map::exist_v_map(cfg, map_id, grid_x, grid_y);

        // return Map::ExistMap(mapid, grid_x, grid_y) && Map::ExistVMap(mapid, grid_x, grid_y);
    }
}
