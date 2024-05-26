use std::time::Duration;

// Grid update timer constants

pub const MIN_GRID_DELAY: Duration = Duration::from_secs(60);
pub const MIN_MAP_UPDATE_DELAY: Duration = Duration::from_millis(50);

// NOTE: hirogoro@20feb2024: referring to https://wowdev.wiki/ADT/v18#MCVT_sub-chunk
// Maps are 64x64 blocks = 4096 (some missing) => In TC/AC terms, maps are 64x64 grids
// Each block is 16x16 chunks = 256
// Each chunk has a size of 8x8 = 64
//
// Here we'll refer to the cell's size as a "unit"
//
// Here TC/AC and other emulator codes (along w/ their extractors) seem to differ.
// It appears that it doesn't use the ADT's definition much w/ the exception for things
// based on ADT values like height/liquid values (such as via Map::GetGridHeight or GridMap::GetLiquidData).
//
// i.e. the block w/ 16x16 chunks + chunk w/ 8x8 units.
//
// Instead, NGrid uses a grid w/ 8x8 cells to contain objects from what i can understand.
// This seems to handle the all the map related events as I understand.

/// Number of cells AXIS-WISE per grid
pub const ADT_CELLS_PER_GRID: usize = 16;
/// Number of units AXIS-WISE per cell
pub const ADT_CELL_SIZE: usize = 8;
/// Total Grid size AXIS-WISE by units
pub const ADT_GRID_SIZE: usize = ADT_CELLS_PER_GRID * ADT_CELL_SIZE;
/// Used in V9 height map
pub const ADT_GRID_SIZE_PLUS_ONE: usize = ADT_GRID_SIZE + 1;

/// Used by Grid/NGrid for objects
pub const MAX_NUMBER_OF_CELLS: usize = 8;
/// Number of grids axis-wise. this is the map length in terms of grids.
pub const MAX_NUMBER_OF_GRIDS: usize = 64;

/// Size of a single grid. Same as the ADT values
pub const SIZE_OF_GRIDS: f32 = 1600.0 / 3.0;
/// Grid ID of the center of the map AXIS-WISE. i.e. half map size in terms of grids
pub const CENTER_GRID_ID: usize = MAX_NUMBER_OF_GRIDS / 2;

/// size in units of a single cell on a Grid/NGrid
pub const SIZE_OF_GRID_CELL: f32 = SIZE_OF_GRIDS / MAX_NUMBER_OF_CELLS as f32;

/// Grid Cell ID of the center of the map AXIS-WISE. i.e. half map size in terms of cells
pub const CENTER_GRID_CELL_ID: usize = MAX_NUMBER_OF_CELLS * MAX_NUMBER_OF_GRIDS / 2;

// have not uncovered what these are specifically -> but all of them seem to be just half sizes of the respective
// grid / grid cell.
pub const CENTER_GRID_OFFSET: f32 = SIZE_OF_GRIDS / 2.0;
pub const CENTER_GRID_CELL_OFFSET: f32 = SIZE_OF_GRID_CELL / 2.0;

/// Number of grid cells axis-wise. this is the map length in terms of cells.
pub const TOTAL_NUMBER_OF_CELLS_PER_MAP: usize = MAX_NUMBER_OF_GRIDS * MAX_NUMBER_OF_CELLS;

/// The ADT grid size, primarily used in ADT related map resolution i.e. involving v8v8 / liquid values for example.
pub const MAP_RESOLUTION: usize = ADT_GRID_SIZE;

pub const MAP_SIZE: f32 = SIZE_OF_GRIDS * MAX_NUMBER_OF_GRIDS as f32;
pub const MAP_HALFSIZE: f32 = MAP_SIZE / 2.0;
