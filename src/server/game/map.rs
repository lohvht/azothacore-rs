use std::{
    fmt::Display,
    io,
    iter,
    path::{Path, PathBuf},
};

use flagset::{flags, FlagSet};
use nalgebra::{DMatrix, Matrix3, SMatrix};
use num_traits::Num;
use tracing::warn;

use crate::{
    bincode_deserialise,
    bincode_serialise,
    cmp_or_return,
    sanity_check_read_all_bytes_from_reader,
    tools::adt::{ADT_CELLS_PER_GRID, ADT_GRID_SIZE, ADT_GRID_SIZE_PLUS_ONE},
    AzResult,
};

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
    pub map_heights:     Option<MapHeightV9V8>,
    pub flight_box:      Option<MapHeightFlightBox>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapHeightV9V8 {
    // Height
    // Height values for triangles stored in order:
    // 1     2     3     4     5     6     7     8     9
    //    10    11    12    13    14    15    16    17
    // 18    19    20    21    22    23    24    25    26
    //    27    28    29    30    31    32    33    34
    // . . . . . . . .
    // For better get height values merge it to V9 and V8 map
    // V9 height map:
    // 1     2     3     4     5     6     7     8     9
    // 18    19    20    21    22    23    24    25    26
    // . . . . . . . .
    // V8 height map:
    //    10    11    12    13    14    15    16    17
    //    27    28    29    30    31    32    33    34
    // . . . . . . . .
    pub v9: SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
    pub v8: SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapHeightFlightBox {
    pub max: Matrix3<i16>,
    pub min: Matrix3<i16>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapLiquidData {
    /// header flags are different from liquid_flags
    pub map_liquid_entry_flags: Result<MapLiquidDataEntryFlags, MapLiquidDataGlobalEntryFlags>,
    pub offset_x:               u8,
    pub offset_y:               u8,
    /// height is nrows, width is ncols
    pub liquid_height_details:  Result<DMatrix<f32>, f32>,
}

impl MapLiquidData {
    pub fn get_liquid_entry_flags(&self, x: usize, y: usize) -> (u16, FlagSet<MapLiquidTypeFlag>) {
        match &self.map_liquid_entry_flags {
            Ok(lf) => (lf.liquid_entry[x][y], lf.liquid_flags[x][y]),
            Err(global_lf) => (global_lf.liquid_type, global_lf.liquid_flags),
        }
    }

    pub fn get_liquid_level(&self, x: usize, y: usize) -> f32 {
        match &self.liquid_height_details {
            Ok(liquid_map) => liquid_map[(x, y)],
            Err(global_level) => *global_level,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapLiquidDataGlobalEntryFlags {
    pub liquid_flags: FlagSet<MapLiquidTypeFlag>,
    pub liquid_type:  u16,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapLiquidDataEntryFlags {
    pub liquid_entry: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub liquid_flags: [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
}

flags! {
    pub enum MapLiquidTypeFlag: u8 {
        // NoWater =    0x00,
        #[allow(clippy::identity_op)]
        Water =       1 << 0,
        Ocean =       1 << 1,
        Magma =       1 << 2,
        Slime =       1 << 3,
        DarkWater =   1 << 4,
        AllLiquids = (MapLiquidTypeFlag::Water | MapLiquidTypeFlag::Ocean | MapLiquidTypeFlag::Magma | MapLiquidTypeFlag::Slime).bits(),
      }

    //   pub enum MapAreaFlag: u16 {
    //     NoArea = 0x0001,
    //   }

    //   pub enum MapHeightFlag: u32 {
    //     NoHeight        = 0x0001,
    //     // AsInt16         = 0x0002,
    //     // AsInt8          = 0x0004,
    //     HasFlightBounds = 0x0008,
    //   }

    //   enum MapLiquidHeaderFlag: u8 {
    //     NoType      =  0x0001,
    //     NoHeight    =  0x0002,
    //   }
}

impl MapLiquidTypeFlag {
    pub fn from_liquid_type_sound_bank_unchecked(sound_bank: u8) -> FlagSet<Self> {
        Self::from_liquid_type_sound_bank(sound_bank)
            .inspect_err(|e| {
                warn!("{e}: sound_bank value was: {sound_bank}");
            })
            .unwrap_or_default()
    }

    pub fn from_liquid_type_sound_bank(sound_bank: u8) -> AzResult<FlagSet<Self>> {
        Ok(FlagSet::new(1u8 << sound_bank)?)
    }
}

const MAP_MAGIC: &[u8] = b"MAPS";
const MAP_VERSION_MAGIC: &[u8] = b"v2.0";

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
        allow_height_limit: bool,
        liquid_show: [[bool; ADT_GRID_SIZE]; ADT_GRID_SIZE],
        map_liquid_height: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
        use_min_height: f32,
    ) -> Self {
        let map_area_data = Self::pack_area_data(map_area_ids);
        let map_height_data = Self::pack_height_data(
            map_height_v9,
            map_height_v8,
            map_height_flight_box_max_min,
            allow_height_limit,
            use_min_height,
        );
        let map_liquid_data = Self::pack_liquid_data(map_liquid_entry, map_liquid_flags, liquid_show, map_liquid_height);
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
        mut map_height_v9: SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        mut map_height_v8: SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
        flight_box_max_min: Option<(Matrix3<i16>, Matrix3<i16>)>,
        allow_height_limit: bool,
        use_min_height: f32,
    ) -> MapHeightData {
        //============================================
        // Try pack height data
        //============================================

        let mut max_height = map_height_v8.max().max(map_height_v9.max());
        let mut min_height = map_height_v8.min().min(map_height_v9.min());

        // Check for allow limit minimum height (not store height in deep ochean - allow save some memory)
        if allow_height_limit && min_height < use_min_height {
            map_height_v8.iter_mut().for_each(|v| *v = v.max(use_min_height));
            map_height_v9.iter_mut().for_each(|v| *v = v.max(use_min_height));

            max_height = max_height.max(use_min_height);
            min_height = min_height.max(use_min_height);
        }

        //     // Not need store if flat surface
        //     if (CONF_allow_float_to_int && (maxHeight - minHeight) < CONF_flat_height_delta_limit)
        //         heightHeader.flags |= MAP_HEIGHT_NO_HEIGHT;

        let flight_box = flight_box_max_min.map(|(max, min)| MapHeightFlightBox { max, min });

        //     // Try store as packed in uint16 or uint8 values
        //     if (!(heightHeader.flags & MAP_HEIGHT_NO_HEIGHT))
        //     {
        //         float step = 0;
        //         // Try Store as uint values
        //         if (CONF_allow_float_to_int)
        //         {
        //             float diff = maxHeight - minHeight;
        //             if (diff < CONF_float_to_int8_limit)      // As uint8 (max accuracy = CONF_float_to_int8_limit/256)
        //             {
        //                 heightHeader.flags|=MAP_HEIGHT_AS_INT8;
        //                 step = selectUInt8StepStore(diff);
        //             }
        //             else if (diff<CONF_float_to_int16_limit)  // As uint16 (max accuracy = CONF_float_to_int16_limit/65536)
        //             {
        //                 heightHeader.flags|=MAP_HEIGHT_AS_INT16;
        //                 step = selectUInt16StepStore(diff);
        //             }
        //         }

        //         // Pack it to int values if need
        //         if (heightHeader.flags&MAP_HEIGHT_AS_INT8)
        //         {
        //             for (int y=0; y<ADT_GRID_SIZE; y++)
        //                 for(int x=0;x<ADT_GRID_SIZE;x++)
        //                     uint8_V8[y][x] = uint8((V8[y][x] - minHeight) * step + 0.5f);
        //             for (int y=0; y<=ADT_GRID_SIZE; y++)
        //                 for(int x=0;x<=ADT_GRID_SIZE;x++)
        //                     uint8_V9[y][x] = uint8((V9[y][x] - minHeight) * step + 0.5f);
        //             map.heightMapSize+= sizeof(uint8_V9) + sizeof(uint8_V8);
        //         }
        //         else if (heightHeader.flags&MAP_HEIGHT_AS_INT16)
        //         {
        //             for (int y=0; y<ADT_GRID_SIZE; y++)
        //                 for(int x=0;x<ADT_GRID_SIZE;x++)
        //                     uint16_V8[y][x] = uint16((V8[y][x] - minHeight) * step + 0.5f);
        //             for (int y=0; y<=ADT_GRID_SIZE; y++)
        //                 for(int x=0;x<=ADT_GRID_SIZE;x++)
        //                     uint16_V9[y][x] = uint16((V9[y][x] - minHeight) * step + 0.5f);
        //             map.heightMapSize+= sizeof(uint16_V9) + sizeof(uint16_V8);
        //         }
        //         else
        //             map.heightMapSize+= sizeof(V9) + sizeof(V8);
        //     }

        let map_heights = if max_height == min_height {
            None
        } else {
            Some(MapHeightV9V8 {
                v9: map_height_v9,
                v8: map_height_v8,
            })
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
        // if (CONF_allow_float_to_int && (maxHeight - minHeight) < CONF_flat_liquid_delta_limit)
        //     liquidHeader_flags |= MapLiquidHeaderFlag::NO_HEIGHT;

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

        let liquid_height_details = if max_height == min_height {
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
        cmp_or_return!(
            rdr,
            MAP_VERSION_MAGIC,
            "Mapfile is the wrong version, please extract new .map files. got {}, want {}"
        )?;
        let ret = bincode_deserialise(&mut rdr)?;

        sanity_check_read_all_bytes_from_reader!(rdr)?;
        Ok(ret)
    }

    pub fn write<W: io::Write + io::Seek>(&self, mut out: &mut W) -> AzResult<()> {
        // everything is packed, now proceed to writing
        out.write_all(MAP_MAGIC)?;
        out.write_all(MAP_VERSION_MAGIC)?;

        bincode_serialise(&mut out, self)?;

        Ok(())
    }
}

pub struct Map {}

impl Map {
    pub fn map_file_name<P, M, X, Y>(maps_dir: P, map_id: M, y: Y, x: X) -> PathBuf
    where
        P: AsRef<Path>,
        M: Num + Display,
        X: Num + Display,
        Y: Num + Display,
    {
        maps_dir.as_ref().join(format!("{map_id:04}_{y:02}_{x:02}.map"))
    }
}
