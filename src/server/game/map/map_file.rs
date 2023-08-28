use std::{io, iter};

use flagset::FlagSet;
use nalgebra::{DMatrix, Matrix3, SMatrix};

use crate::{
    bincode_deserialise,
    bincode_serialise,
    cmp_or_return,
    sanity_check_read_all_bytes_from_reader,
    server::game::map::{
        MapLiquidData,
        MapLiquidDataEntryFlags,
        MapLiquidDataGlobalEntryFlags,
        MapLiquidTypeFlag,
        ADT_CELLS_PER_GRID,
        ADT_GRID_SIZE,
        ADT_GRID_SIZE_PLUS_ONE,
    },
    AzResult,
};

// Bunch of map stuff
/// Max accuracy = val/256
const V9V8_HEIGHT_FLOAT_TO_INT8_LIMIT: f32 = 2.0;
/// Max accuracy = val/65536
const V9V8_HEIGHT_FLOAT_TO_INT16_LIMIT: f32 = 2048.0;
/// If max - min less this value - surface is flat
const FLAT_HEIGHT_DELTA_LIMIT: f32 = 0.005;
/// If max - min less this value - liquid surface is flat
const FLAT_LIQUID_DELTA_LIMIT: f32 = 0.001;

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

/// Height
///
/// Height values for triangles stored in order:
///
/// ````
///  1     2     3     4     5     6     7     8     9
///
///     10    11    12    13    14    15    16    17
///
///  18    19    20    21    22    23    24    25    26
///
///     27    28    29    30    31    32    33    34
///
///  . . . . . . . .
/// ````
///
///  For better get height values merge it to V9 and V8 map
///
///  V9 height map:
///
/// ````
///  1     2     3     4     5     6     7     8     9
///
///  18    19    20    21    22    23    24    25    26
///
///  . . . . . . . .
/// ````
///
///  V8 height map:
///
/// ````
///     10    11    12    13    14    15    16    17
///
///     27    28    29    30    31    32    33    34
///
///  . . . . . . . .
/// ````
///
#[expect(clippy::large_enum_variant)]
#[derive(Debug, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapHeightFlightBox {
    pub max: Matrix3<i16>,
    pub min: Matrix3<i16>,
}

const MAP_MAGIC: &[u8] = b"MAPSv2.1";

pub struct MapFileParams {
    pub allow_float_to_int: bool,
    pub allow_height_limit: bool,
    pub use_min_height:     f32,
}

// Methods to pack into MapFile
impl MapFile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        map_build_magic: u32,
        map_area_ids: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_height_v9: SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        map_height_v8: SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
        map_liquid_flags: [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_liquid_entry: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_holes: Option<[[[u8; 8]; 16]; 16]>,
        map_height_flight_box_max_min: Option<(Matrix3<i16>, Matrix3<i16>)>,
        liquid_show: [[bool; ADT_GRID_SIZE]; ADT_GRID_SIZE],
        map_liquid_height: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
        params: &MapFileParams,
    ) -> Self {
        let map_area_data = Self::pack_area_data(map_area_ids);
        let map_height_data = Self::pack_height_data(params, map_height_v9, map_height_v8, map_height_flight_box_max_min);
        let map_liquid_data = Self::pack_liquid_data(params, map_liquid_entry, map_liquid_flags, liquid_show, map_liquid_height);
        Self {
            map_build_magic,
            map_area_data,
            map_holes,
            map_height_data,
            map_liquid_data,
        }
    }

    fn pack_area_data(
        map_area_ids: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    ) -> Result<[[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID], u16> {
        //============================================
        // Try pack area data
        //============================================
        let area_id = map_area_ids[0][0];
        let full_area_data = map_area_ids.iter().any(|row| row.iter().any(|map_area_id| area_id != *map_area_id));

        if full_area_data {
            Ok(map_area_ids)
        } else {
            Err(area_id)
        }
    }

    fn pack_height_data(
        params: &MapFileParams,
        mut map_height_v9: SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        mut map_height_v8: SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
        flight_box_max_min: Option<(Matrix3<i16>, Matrix3<i16>)>,
    ) -> MapHeightData {
        //============================================
        // Try pack height data
        //============================================

        let mut max_height = (-20000.0f32).max(map_height_v8.max().max(map_height_v9.max()));
        let mut min_height = (20000.0f32).min(map_height_v8.min().min(map_height_v9.min()));

        // Check for allow limit minimum height (not store height in deep ochean - allow save some memory)
        if params.allow_height_limit && min_height < params.use_min_height {
            map_height_v8.iter_mut().for_each(|v| *v = v.max(params.use_min_height));
            map_height_v9.iter_mut().for_each(|v| *v = v.max(params.use_min_height));

            max_height = max_height.max(params.use_min_height);
            min_height = min_height.max(params.use_min_height);
        }

        let mut should_include_height = true;
        if max_height == min_height {
            should_include_height = false;
        }
        // Not need store if flat surface
        if params.allow_float_to_int && (max_height - min_height) < FLAT_HEIGHT_DELTA_LIMIT {
            should_include_height = false;
        }

        let flight_box = flight_box_max_min.map(|(max, min)| MapHeightFlightBox { max, min });

        // Try store as packed in uint16 or uint8 values
        let map_heights = if !should_include_height {
            None
        } else {
            let diff = max_height - min_height;
            if params.allow_float_to_int && diff < V9V8_HEIGHT_FLOAT_TO_INT8_LIMIT {
                // As uint8 (max accuracy = CONF_float_to_int8_limit/256)
                let step = 255.0 / diff;
                let map_height_v9 = map_height_v9.map(|v| ((v - min_height) * step + 0.5) as u8);
                let map_height_v8 = map_height_v8.map(|v| ((v - min_height) * step + 0.5) as u8);
                Some(MapFilev9v8::U8 {
                    v9: map_height_v9,
                    v8: map_height_v8,
                })
            } else if params.allow_float_to_int && diff < V9V8_HEIGHT_FLOAT_TO_INT16_LIMIT {
                // As uint16 (max accuracy = CONF_float_to_int16_limit/65536)
                let step = 65535.0 / diff;
                let map_height_v9 = map_height_v9.map(|v| ((v - min_height) * step + 0.5) as u16);
                let map_height_v8 = map_height_v8.map(|v| ((v - min_height) * step + 0.5) as u16);
                Some(MapFilev9v8::U16 {
                    v9: map_height_v9,
                    v8: map_height_v8,
                })
            } else {
                Some(MapFilev9v8::F32 {
                    v9: map_height_v9,
                    v8: map_height_v8,
                })
            }
        };
        MapHeightData {
            grid_height: min_height,
            grid_max_height: max_height,
            map_heights,
            flight_box,
        }
    }

    /// Returns a triple denoting the first liquid type, the first liquid flag, as well as whether or not
    /// its a full type
    fn global_liquid_info(
        map_liquid_entry: &[[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_liquid_flags: &[[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    ) -> Option<(u16, FlagSet<MapLiquidTypeFlag>)> {
        let first_liquid_type = map_liquid_entry[0][0];
        let first_liquid_flag = map_liquid_flags[0][0];

        let full_type = iter::zip(map_liquid_entry, map_liquid_flags).any(|(liq_entries, flag_entries)| {
            iter::zip(liq_entries, flag_entries)
                .any(|(liq_entry, flag_entry)| *liq_entry != first_liquid_type || *flag_entry != first_liquid_flag)
        });

        if full_type {
            Some((first_liquid_type, first_liquid_flag))
        } else {
            None
        }
    }

    fn pack_liquid_data(
        params: &MapFileParams,
        map_liquid_entry: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_liquid_flags: [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        liquid_show: [[bool; ADT_GRID_SIZE]; ADT_GRID_SIZE],
        map_liquid_height: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
    ) -> Option<MapLiquidData> {
        //============================================
        // Pack liquid data
        //============================================
        let global_liq_info = Self::global_liquid_info(&map_liquid_entry, &map_liquid_flags);
        // no water data (if all grid have 0 liquid type)
        match global_liq_info {
            Some((_, first_liquid_flag)) if first_liquid_flag.bits() == 0 => {
                // No liquid data
                return None;
            },
            // No liquid data
            None => return None,
            _ => {},
        };
        // has liquid data
        let (mut min_x, mut min_y) = (255, 255);
        let (mut max_x, mut max_y) = (0, 0);
        let mut max_height = -20000f32;
        let mut min_height = 20000f32;
        for (y, yarr) in liquid_show.iter().enumerate() {
            for (x, liq_show) in yarr.iter().enumerate() {
                if *liq_show {
                    min_x = min_x.min(x as u8);
                    max_x = max_x.max(x as u8);
                    min_y = min_y.min(y as u8);
                    max_y = max_y.max(y as u8);
                    let h = map_liquid_height[y][x];
                    max_height = max_height.max(h);
                    min_height = min_height.min(h);
                }
            }
        }
        let offset_x = min_x;
        let offset_y = min_y;
        let width = max_x - min_x + 1 + 1;
        let height = max_y - min_y + 1 + 1;
        let liquid_level = min_height;

        // // Not need store if flat surface
        let mut should_include_height = true;
        if max_height == min_height {
            should_include_height = false;
        }
        if params.allow_float_to_int && (max_height - min_height) < FLAT_LIQUID_DELTA_LIMIT {
            should_include_height = false;
        }

        let map_liquid_entry_flags = if let Some((first_liquid_type, first_liquid_flag)) = global_liq_info {
            Err(MapLiquidDataGlobalEntryFlags {
                liquid_flags: first_liquid_flag,
                liquid_type:  first_liquid_type,
            })
        } else {
            Ok(MapLiquidDataEntryFlags {
                liquid_entry: map_liquid_entry,
                liquid_flags: map_liquid_flags,
            })
        };

        // _liquidMap = new float[uint32(_liquidWidth) * uint32(_liquidHeight)];

        let liquid_height_details = if !should_include_height {
            Err(liquid_level)
        } else {
            // map.liquidMapSize += sizeof(float) * liquidHeader.width * liquidHeader.height;
            let offset_y = offset_y as usize;
            let offset_x = offset_x as usize;
            Ok(DMatrix::from_fn(height as usize, width as usize, |y, x| {
                map_liquid_height[y + offset_y][x + offset_x]
            }))
        };
        Some(MapLiquidData {
            offset_x,
            offset_y,
            map_liquid_entry_flags,
            liquid_height_details,
        })
    }

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
