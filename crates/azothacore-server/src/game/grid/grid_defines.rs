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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct CoordPair<const LIMIT: usize> {
    pub x_coord: usize,
    pub y_coord: usize,
}

impl<const LIMIT: usize> CoordPair<LIMIT> {
    pub fn dec_x(&mut self, val: usize) {
        if self.x_coord > val {
            self.x_coord -= val;
        } else {
            self.x_coord = 0;
        }
    }

    pub fn inc_x(&mut self, val: usize) {
        self.x_coord = (self.x_coord + val).min(LIMIT - 1);
    }

    pub fn dec_y(&mut self, val: usize) {
        if self.y_coord > val {
            self.y_coord -= val;
        } else {
            self.y_coord = 0;
        }
    }

    pub fn inc_y(&mut self, val: usize) {
        self.y_coord = (self.y_coord + val).min(LIMIT - 1);
    }

    pub fn is_coord_valid(&self) -> bool {
        self.x_coord < LIMIT && self.y_coord < LIMIT
    }

    pub fn normalize(mut self) -> Self {
        self.x_coord = self.x_coord.min(LIMIT - 1);
        self.y_coord = self.y_coord.min(LIMIT - 1);
        self
    }

    pub fn get_id(&self) -> usize {
        self.y_coord * LIMIT + self.x_coord
    }
}

pub type GridCoord = CoordPair<MAX_NUMBER_OF_GRIDS>;
pub type CellCoord = CoordPair<TOTAL_NUMBER_OF_CELLS_PER_MAP>;

fn compute<const LIMIT: usize, const CENTER_VAL: usize>(x: f32, y: f32, center_offset: f32, size: f32) -> CoordPair<LIMIT> {
    // calculate and store temporary values in double format for having same result as same mySQL calculations
    let x_offset = (f64::from(x) - f64::from(center_offset)) / f64::from(size);
    let y_offset = (f64::from(y) - f64::from(center_offset)) / f64::from(size);

    let x_coord = (x_offset + 0.5) as usize + CENTER_VAL;
    let y_coord = (y_offset + 0.5) as usize + CENTER_VAL;
    CoordPair::<LIMIT> { x_coord, y_coord }
}

/// ComputeGridCoord
pub fn compute_grid_coord(x: f32, y: f32) -> GridCoord {
    compute::<MAX_NUMBER_OF_GRIDS, CENTER_GRID_ID>(x, y, CENTER_GRID_OFFSET, SIZE_OF_GRIDS)
}

/// ComputeCellCoord
pub fn compute_cell_coord(x: f32, y: f32) -> CellCoord {
    compute::<TOTAL_NUMBER_OF_CELLS_PER_MAP, CENTER_GRID_CELL_ID>(x, y, CENTER_GRID_CELL_OFFSET, SIZE_OF_GRID_CELL)
}

pub fn compute_cell_coord2(x: f32, y: f32, x_off: &mut f32, y_off: &mut f32) -> CellCoord {
    let x_offset = (f64::from(x) - f64::from(CENTER_GRID_CELL_OFFSET)) / f64::from(SIZE_OF_GRID_CELL);
    let y_offset = (f64::from(y) - f64::from(CENTER_GRID_CELL_OFFSET)) / f64::from(SIZE_OF_GRID_CELL);

    let x_coord = (x_offset + 0.5) as usize + CENTER_GRID_CELL_ID;
    let y_coord = (y_offset + 0.5) as usize + CENTER_GRID_CELL_ID;
    *x_off = (x_offset as f32 - x_coord as f32 + CENTER_GRID_CELL_ID as f32) * SIZE_OF_GRID_CELL;
    *y_off = (y_offset as f32 - y_coord as f32 + CENTER_GRID_CELL_ID as f32) * SIZE_OF_GRID_CELL;
    CellCoord { x_coord, y_coord }
}

pub fn normalize_map_coord(c: &mut f32) {
    if *c > MAP_HALFSIZE - 0.5 {
        *c = MAP_HALFSIZE - 0.5;
    } else if *c < -(MAP_HALFSIZE - 0.5) {
        *c = -(MAP_HALFSIZE - 0.5);
    }
}

pub fn is_valid_map_coord1(c: f32) -> bool {
    c.is_finite() && c.abs() <= (MAP_HALFSIZE - 0.5)
}

pub fn is_valid_map_coord2(x: f32, y: f32) -> bool {
    is_valid_map_coord1(x) && is_valid_map_coord1(y)
}

pub fn is_valid_map_coord3(x: f32, y: f32, z: f32) -> bool {
    is_valid_map_coord2(x, y) && is_valid_map_coord1(z)
}

pub fn is_valid_map_coord4(x: f32, y: f32, z: f32, o: f32) -> bool {
    is_valid_map_coord3(x, y, z) && is_valid_map_coord1(o)
}

// x:  17066.66656 compute_grid_coord => 63; Map::GetGrid => 0
// y: -17066.66656 compute_grid_coord => 0 ; Map::GetGrid => 63

// x: 0 compute_grid_coord => 32; Map::GetGrid => 32
// y: 0 compute_grid_coord => 32; Map::GetGrid => 32

// x: 7000 compute_grid_coord => 45.12500080156255; Map::GetGrid => 18.87499917968745
// y: -150 compute_grid_coord => 31.71874996367187; Map::GetGrid => 32.28125001757813

// x: 15999 compute_grid_coord => 61.99812685613293; Map::GetGrid => 2.001873125117070
// y: -11222 compute_grid_coord => 10.95874866617179; Map::GetGrid => 53.04125131507821

// (17066.66656 - 266.66666) / 533.3333 + 0.5 + 32
// 32 - (17066.66656 / 533.3333)
