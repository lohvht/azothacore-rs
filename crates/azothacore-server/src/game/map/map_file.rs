use std::io;

use azothacore_common::{
    cmp_or_return,
    sanity_check_read_all_bytes_from_reader,
    utils::{bincode_deserialise, bincode_serialise},
    AzResult,
};
use nalgebra::{Matrix3, SMatrix};

use crate::game::map::{MapLiquidData, ADT_CELLS_PER_GRID, ADT_GRID_SIZE, ADT_GRID_SIZE_PLUS_ONE};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapFile {
    pub map_build_magic: u32,
    /// Map Area
    pub map_area_data:   Result<[[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID], u16>,
    /// Map height
    pub map_height_data: MapHeightData,
    // Map Liquid
    pub map_liquid_data: Option<MapLiquidData>,
    // holes
    pub map_holes:       Option<[[[u8; 8]; 16]; 16]>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapHeightData {
    pub grid_height:     f32,
    pub grid_max_height: f32,
    // m_V9[(x_int) * 129 + y_int]; => v9[(x_int, y_int)]
    pub map_heights:     Option<MapFilev9v8>,
    pub flight_box:      Option<MapHeightFlightBox>,
}

// Height
//
// Height values for triangles stored in order:
//
// ````
//  1     2     3     4     5     6     7     8     9
//
//     10    11    12    13    14    15    16    17
//
//  18    19    20    21    22    23    24    25    26
//
//     27    28    29    30    31    32    33    34
//
//  . . . . . . . .
// ````
//
//  For better get height values merge it to V9 and V8 map
//
//  V9 height map:
//
// ````
//  1     2     3     4     5     6     7     8     9
//
//  18    19    20    21    22    23    24    25    26
//
//  . . . . . . . .
// ````
//
//  V8 height map:
//
// ````
//     10    11    12    13    14    15    16    17
//
//     27    28    29    30    31    32    33    34
//
//  . . . . . . . .
// ````
//
#[expect(clippy::large_enum_variant)]
#[derive(PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum MapFilev9v8 {
    U8 {
        v9: SMatrix<u8, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        v8: SMatrix<u8, ADT_GRID_SIZE, ADT_GRID_SIZE>,
    },
    U16 {
        v9: SMatrix<u16, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        v8: SMatrix<u16, ADT_GRID_SIZE, ADT_GRID_SIZE>,
    },
    F32 {
        v9: SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        v8: SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
    },
}

impl MapFilev9v8 {
    pub fn to_v9v8(
        &self,
        grid_height: f32,
        grid_max_height: f32,
    ) -> (
        SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
    ) {
        match self {
            MapFilev9v8::F32 { v9, v8 } => (*v9, *v8),
            MapFilev9v8::U16 { v9, v8 } => (
                v9.map(|v| v as f32 * (grid_max_height - grid_height) / 65535.0 + grid_height),
                v8.map(|v| v as f32 * (grid_max_height - grid_height) / 65535.0 + grid_height),
            ),
            MapFilev9v8::U8 { v9, v8 } => (
                v9.map(|v| v as f32 * (grid_max_height - grid_height) / 255.0 + grid_height),
                v8.map(|v| v as f32 * (grid_max_height - grid_height) / 255.0 + grid_height),
            ),
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MapHeightFlightBox {
    pub max: Matrix3<i16>,
    pub min: Matrix3<i16>,
}

const MAP_MAGIC: &[u8] = b"MAPSv2.1";

// Methods to pack into MapFile
impl MapFile {
    pub fn read<R: io::Read + io::Seek>(mut rdr: &mut R) -> AzResult<Self> {
        cmp_or_return!(
            rdr,
            MAP_MAGIC,
            "Mapfile magic does not match, please ensure that this is the right file. got {}, want {}"
        )?;
        let ret = bincode_deserialise(&mut rdr)?;

        sanity_check_read_all_bytes_from_reader!(rdr)?;
        Ok(ret)
    }

    pub fn write<W: io::Write + io::Seek>(&self, mut out: &mut W) -> AzResult<()> {
        // everything is packed, now proceed to writing
        out.write_all(MAP_MAGIC)?;

        bincode_serialise(&mut out, self)?;

        Ok(())
    }
}
