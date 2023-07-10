use std::{io, iter};

use flagset::{flags, FlagSet};
use nalgebra::{Matrix3, SMatrix};

use crate::{
    cmp_or_return,
    sanity_check_read_all_bytes_from_reader,
    tools::{
        adt::{ADT_CELLS_PER_GRID, ADT_GRID_SIZE, ADT_GRID_SIZE_PLUS_ONE},
        extractor_common::{bincode_deserialise, bincode_serialise},
    },
    GenericResult,
};

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MapFile {
    pub map_build_magic:               u32,
    /// Map Area
    map_area_header_flags:             FlagSet<MapAreaFlag>,
    map_area_header_grid_area:         u16,
    pub map_area_ids:                  [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    /// Map height
    map_height_header_flags:           FlagSet<MapHeightFlag>,
    map_height_header_grid_height:     f32,
    map_height_header_grid_max_height: f32,
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
    pub map_height_V9:                 SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
    pub map_height_V8:                 SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
    #[allow(clippy::type_complexity)]
    pub map_height_flight_box_max_min: Option<(Matrix3<i16>, Matrix3<i16>)>,
    // Map Liquid
    map_liquid_header_flags:           FlagSet<MapLiquidHeaderFlag>,
    map_liquid_header_liquid_flags:    FlagSet<MapLiquidTypeFlag>,
    map_liquid_header_liquid_type:     u16,
    map_liquid_header_offset_x:        u8,
    map_liquid_header_offset_y:        u8,
    map_liquid_header_width:           u8,
    map_liquid_header_height:          u8,
    map_liquid_header_liquid_level:    f32,
    pub map_liquid_entry:              [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub map_liquid_flags:              [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub map_liquid_height_map:         Vec<f32>,
    // holes
    pub map_holes:                     Option<[[[u8; 8]; 16]; 16]>,
}

flags! {
    pub enum MapLiquidTypeFlag: u8 {
        // NoWater =    0x00,
        #[allow(clippy::identity_op)]
        Water =       0x01,
        Ocean =       0x02,
        Magma =       0x04,
        Slime =       0x08,
        DarkWater =  0x10,
        AllLiquids = (MapLiquidTypeFlag::Water | MapLiquidTypeFlag::Ocean | MapLiquidTypeFlag::Magma | MapLiquidTypeFlag::Slime).bits(),
      }

      pub enum MapAreaFlag: u16 {
        NoArea = 0x0001,
      }

      pub enum MapHeightFlag: u32 {
        NoHeight        = 0x0001,
        // AsInt16         = 0x0002,
        // AsInt8          = 0x0004,
        HasFlightBounds = 0x0008,
      }

      pub enum MapLiquidHeaderFlag: u8 {
        NoType      =  0x0001,
        NoHeight    =  0x0002,
      }
}

// Methods to pack into MapFile
impl MapFile {
    #[allow(clippy::too_many_arguments, non_snake_case)]
    pub fn new(
        map_build_magic: u32,
        map_area_ids: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        mut map_height_V9: SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        mut map_height_V8: SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
        map_liquid_flags: [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_liquid_entry: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_holes: Option<[[[u8; 8]; 16]; 16]>,
        map_height_flight_box_max_min: Option<(Matrix3<i16>, Matrix3<i16>)>,
        allow_height_limit: bool,
        liquid_show: [[bool; ADT_GRID_SIZE]; ADT_GRID_SIZE],
        map_liquid_height: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
        use_min_height: f32,
    ) -> Self {
        let (map_area_header_grid_area, map_area_header_flags) = Self::pack_area_data(&map_area_ids);
        let (map_height_header_grid_height, map_height_header_grid_max_height, map_height_header_flags) = Self::pack_height_data(
            &mut map_height_V9,
            &mut map_height_V8,
            map_height_flight_box_max_min.as_ref(),
            allow_height_limit,
            use_min_height,
        );
        let (
            map_liquid_header_offset_x,
            map_liquid_header_offset_y,
            map_liquid_header_width,
            map_liquid_header_height,
            map_liquid_header_liquid_level,
            map_liquid_header_flags,
            map_liquid_header_liquid_flags,
            map_liquid_header_liquid_type,
            map_liquid_height_map,
        ) = Self::pack_liquid_data(&map_liquid_entry, &map_liquid_flags, liquid_show, map_liquid_height);
        Self {
            map_build_magic,
            map_area_ids,
            map_height_V9,
            map_height_V8,
            map_liquid_flags,
            map_liquid_entry,
            map_holes,
            map_height_flight_box_max_min,
            map_area_header_grid_area,
            map_area_header_flags,
            map_height_header_grid_height,
            map_height_header_grid_max_height,
            map_height_header_flags,
            map_liquid_header_offset_x,
            map_liquid_header_offset_y,
            map_liquid_header_width,
            map_liquid_header_height,
            map_liquid_header_liquid_level,
            map_liquid_header_flags,
            map_liquid_header_liquid_flags,
            map_liquid_header_liquid_type,
            map_liquid_height_map,
        }
    }

    fn pack_area_data(map_area_ids: &[[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID]) -> (u16, FlagSet<MapAreaFlag>) {
        //============================================
        // Try pack area data
        //============================================
        let area_id = map_area_ids[0][0];
        let full_area_data = map_area_ids.iter().any(|row| row.iter().any(|map_area_id| area_id != *map_area_id));

        if full_area_data {
            (0, None.into())
        } else {
            (area_id, MapAreaFlag::NoArea.into())
        }
    }

    #[allow(non_snake_case)]
    fn pack_height_data(
        map_height_V9: &mut SMatrix<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>,
        map_height_V8: &mut SMatrix<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>,
        map_height_flight_box_max_min: Option<&(Matrix3<i16>, Matrix3<i16>)>,
        allow_height_limit: bool,
        use_min_height: f32,
    ) -> (f32, f32, FlagSet<MapHeightFlag>) {
        //============================================
        // Try pack height data
        //============================================

        let (mut max_height, mut min_height) = (map_height_V8.max().max(map_height_V9.max()), map_height_V8.min().min(map_height_V9.min()));

        map_height_V8.iter_mut().for_each(|v| *v = v.max(use_min_height));

        // Check for allow limit minimum height (not store height in deep ochean - allow save some memory)
        if allow_height_limit && min_height < use_min_height {
            map_height_V8.iter_mut().for_each(|v| *v = v.max(use_min_height));
            map_height_V9.iter_mut().for_each(|v| *v = v.max(use_min_height));

            max_height = max_height.max(use_min_height);
            min_height = min_height.max(use_min_height);
        }

        //     map.heightMapOffset = map.areaMapOffset + map.areaMapSize;
        //     map.heightMapSize = sizeof(map_heightHeader);

        //     map_heightHeader heightHeader;
        let map_height_header_grid_height = min_height;
        let map_height_header_grid_max_height = max_height;

        let mut map_height_header_flags = None.into();
        if max_height == min_height {
            map_height_header_flags |= MapHeightFlag::NoHeight;
        }

        //     // Not need store if flat surface
        //     if (CONF_allow_float_to_int && (maxHeight - minHeight) < CONF_flat_height_delta_limit)
        //         heightHeader.flags |= MAP_HEIGHT_NO_HEIGHT;

        if map_height_flight_box_max_min.is_some() {
            map_height_header_flags |= MapHeightFlag::HasFlightBounds;
            // map.heightMapSize += sizeof(flight_box_max) + sizeof(flight_box_min);
        }

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
        (map_height_header_grid_height, map_height_header_grid_max_height, map_height_header_flags)
    }

    /// Returns a triple denoting the first liquid type, the first liquid flag, as well as whether or not
    /// its a full type
    fn overall_liquid_info(
        map_liquid_entry: &[[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_liquid_flags: &[[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    ) -> (u16, FlagSet<MapLiquidTypeFlag>, bool) {
        let first_liquid_type = map_liquid_entry[0][0];
        let first_liquid_flag = map_liquid_flags[0][0];

        let mut it = iter::zip(map_liquid_entry, map_liquid_flags);

        let full_type = it.any(|(tyit, flit)| {
            let mut it = iter::zip(tyit, flit);
            it.any(|(ty, fl)| *ty != first_liquid_type || *fl != first_liquid_flag)
        });

        (first_liquid_type, first_liquid_flag, full_type)
    }

    #[allow(clippy::type_complexity)]
    fn pack_liquid_data(
        map_liquid_entry: &[[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        map_liquid_flags: &[[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
        liquid_show: [[bool; ADT_GRID_SIZE]; ADT_GRID_SIZE],
        map_liquid_height: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
    ) -> (u8, u8, u8, u8, f32, FlagSet<MapLiquidHeaderFlag>, FlagSet<MapLiquidTypeFlag>, u16, Vec<f32>) {
        //============================================
        // Pack liquid data
        //============================================
        let (first_liquid_type, first_liquid_flag, full_type) = Self::overall_liquid_info(map_liquid_entry, map_liquid_flags);
        // no water data (if all grid have 0 liquid type)
        if first_liquid_flag.bits() == 0 && !full_type {
            // No liquid data
            return (0, 0, 0, 0, 0.0, None.into(), None.into(), 0, vec![]);
        }
        // has liquid data
        let mut min_x = 255;
        let mut min_y = 255;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut max_height = -20000f32;
        let mut min_height = 20000f32;
        for (y, yarr) in liquid_show.iter().enumerate() {
            for (x, liq_show) in yarr.iter().enumerate() {
                if *liq_show {
                    min_x = min_x.min(x as u8);
                    min_y = min_y.min(y as u8);
                    max_x = max_x.max(x as u8);
                    max_y = max_y.max(y as u8);
                    let h = map_liquid_height[y][x];
                    max_height = max_height.max(h);
                    min_height = min_height.min(h);
                }
            }
        }
        let map_liquid_header_offset_x = min_x;
        let map_liquid_header_offset_y = min_y;
        let map_liquid_header_width = max_x - min_x + 1 + 1;
        let map_liquid_header_height = max_y - min_y + 1 + 1;
        let map_liquid_header_liquid_level = min_height;
        let mut map_liquid_header_flags: FlagSet<_> = None.into();
        let mut map_liquid_header_liquid_flags = None.into();
        let mut map_liquid_header_liquid_type = 0;
        let mut map_liquid_height_map = vec![];

        if max_height == min_height {
            map_liquid_header_flags |= MapLiquidHeaderFlag::NoHeight;
        }

        // // Not need store if flat surface
        // if (CONF_allow_float_to_int && (maxHeight - minHeight) < CONF_flat_liquid_delta_limit)
        //     liquidHeader_flags |= MapLiquidHeaderFlag::NO_HEIGHT;

        if !full_type {
            map_liquid_header_flags |= MapLiquidHeaderFlag::NoType;
        }

        if map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoType) {
            map_liquid_header_liquid_flags = first_liquid_flag;
            map_liquid_header_liquid_type = first_liquid_type;
        }

        // _liquidMap = new float[uint32(_liquidWidth) * uint32(_liquidHeight)];

        if !map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoHeight) {
            // map.liquidMapSize += sizeof(float) * liquidHeader.width * liquidHeader.height;
            for y in 0..map_liquid_header_height {
                let y_off: usize = (y + map_liquid_header_offset_y).try_into().unwrap();
                let x_off: usize = map_liquid_header_offset_x.try_into().unwrap();
                let liq_header_width: usize = map_liquid_header_width.try_into().unwrap();
                for h in map_liquid_height[y_off][x_off..x_off + liq_header_width].iter() {
                    map_liquid_height_map.push(*h);
                }
            }
            let liq_height_map_len = map_liquid_header_width as usize * map_liquid_header_height as usize;
            if map_liquid_height_map.len() != liq_height_map_len {
                panic!(
                    "MAP LIQUID HEIGHT MAP LEN DOES NOT MATCH CALCULATED LEN: calc was {liq_height_map_len}, data from header was: {}",
                    map_liquid_height_map.len()
                );
            }
        }
        (
            map_liquid_header_offset_x,
            map_liquid_header_offset_y,
            map_liquid_header_width,
            map_liquid_header_height,
            map_liquid_header_liquid_level,
            map_liquid_header_flags,
            map_liquid_header_liquid_flags,
            map_liquid_header_liquid_type,
            map_liquid_height_map,
        )
    }

    pub fn read<R: io::Read + io::Seek>(rdr: &mut R) -> GenericResult<Self> {
        let mut rdr = rdr;
        cmp_or_return!(rdr, b"MAPS")?;
        cmp_or_return!(rdr, b"v1.9")?;
        let ret = bincode_deserialise(&mut rdr)?;

        sanity_check_read_all_bytes_from_reader!(rdr)?;
        Ok(ret)
    }

    pub fn write<W: io::Write + io::Seek>(&self, out: &mut W) -> GenericResult<()> {
        let mut out = out;
        // everything is packed, now proceed to writing
        out.write_all(b"MAPS")?;
        out.write_all(b"v1.9")?;

        bincode_serialise(&mut out, self)?;

        Ok(())
    }
}
