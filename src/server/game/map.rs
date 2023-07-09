use std::{io, iter};

use byteorder::{LittleEndian, ReadBytesExt};
use flagset::{flags, FlagSet};
use nalgebra::Matrix3;

use crate::{
    sanity_check_read_all_bytes_from_reader,
    tools::adt::{ADT_CELLS_PER_GRID, ADT_GRID_SIZE},
    GenericResult,
};

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct MapFile {
    /// Map general
    /// Magic value
    map_magic: [u8; 4],
    map_version_magic: [u8; 4],
    pub map_build_magic: u32,
    map_area_map_offset: u32,
    map_area_map_size: u32,
    map_height_map_offset: u32,
    map_height_map_size: u32,
    map_liquid_map_offset: u32,
    map_liquid_map_size: u32,
    map_holes_offset: u32,
    map_holes_size: u32,
    /// Map Area
    map_area_header_fourcc: [u8; 4],
    map_area_header_flags: FlagSet<MapAreaFlag>,
    map_area_header_grid_area: u16,
    pub map_area_ids: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    /// Map height
    map_height_header_fourcc: [u8; 4],
    map_height_header_flags: FlagSet<MapHeightFlag>,
    map_height_header_grid_height: f32,
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
    pub map_height_V9: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
    pub map_height_V8: [[f32; ADT_GRID_SIZE]; ADT_GRID_SIZE],
    #[allow(clippy::type_complexity)]
    pub map_height_flight_box_max_min: Option<(Matrix3<i16>, Matrix3<i16>)>,
    // Map Liquid
    map_liquid_header_fourcc: [u8; 4],
    map_liquid_header_flags: FlagSet<MapLiquidHeaderFlag>,
    map_liquid_header_liquid_flags: FlagSet<MapLiquidTypeFlag>,
    map_liquid_header_liquid_type: u16,
    map_liquid_header_offset_x: u8,
    map_liquid_header_offset_y: u8,
    map_liquid_header_width: u8,
    map_liquid_header_height: u8,
    map_liquid_header_liquid_level: f32,
    pub map_liquid_entry: [[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub map_liquid_flags: [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub map_liquid_height_map: Vec<f32>,
    // holes
    pub map_holes: Option<[[[u8; 8]; 16]; 16]>,
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

impl Default for MapFile {
    fn default() -> Self {
        Self {
            map_magic: *b"MAPS",
            map_version_magic: *b"v1.9",
            map_build_magic: 0,
            map_area_map_offset: 0,
            map_area_map_size: 0,
            map_height_map_offset: 0,
            map_height_map_size: 0,
            map_liquid_map_offset: 0,
            map_liquid_map_size: 0,
            map_holes_offset: 0,
            map_holes_size: 0,
            map_area_header_fourcc: *b"AREA",
            map_area_header_flags: None.into(),
            map_area_header_grid_area: 0,
            map_area_ids: [[0; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
            map_height_header_fourcc: *b"MHGT",
            map_height_header_flags: None.into(),
            map_height_header_grid_height: 0f32,
            map_height_header_grid_max_height: 0f32,
            map_height_V9: [[0f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
            map_height_V8: [[0f32; ADT_GRID_SIZE]; ADT_GRID_SIZE],
            map_height_flight_box_max_min: None,
            map_liquid_header_fourcc: *b"MLIQ",
            map_liquid_header_flags: None.into(),
            map_liquid_header_liquid_flags: None.into(),
            map_liquid_header_liquid_type: 0,
            map_liquid_header_offset_x: 0,
            map_liquid_header_offset_y: 0,
            map_liquid_header_width: 0,
            map_liquid_header_height: 0,
            map_liquid_header_liquid_level: 0f32,
            map_liquid_entry: [[0; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
            map_liquid_flags: [[None.into(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
            map_liquid_height_map: vec![],
            map_holes: None,
        }
    }
}

macro_rules! twod_little_endian_write {
    ( $twod:expr, $out:expr ) => {{
        for row in $twod.iter() {
            for v in row.iter() {
                $out.write_all(&v.to_le_bytes())?;
            }
        }
    }};
}

macro_rules! matrix_little_endian_write {
    ( $twod:expr, $out:expr ) => {{
        for row in $twod.row_iter() {
            for v in row.iter() {
                $out.write_all(&v.to_le_bytes())?;
            }
        }
    }};
}

macro_rules! twod_little_endian_read {
    ( $twod:expr, $rdr:expr, $method:ident ) => {{
        for row in $twod.iter_mut() {
            for v in row.iter_mut() {
                *v = $rdr.$method::<LittleEndian>()?;
            }
        }
    }};
}

macro_rules! matrix_little_endian_read {
    ( $twod:expr, $rdr:expr, $method:ident ) => {{
        for mut row in $twod.row_iter_mut() {
            for v in row.iter_mut() {
                *v = $rdr.$method::<LittleEndian>()?;
            }
        }
    }};
}

macro_rules! twod_flags_little_endian_write {
    ( $twod:expr, $out:expr ) => {{
        for row in $twod.iter() {
            for v in row.iter() {
                $out.write_all(&v.bits().to_le_bytes())?;
            }
        }
    }};
}

macro_rules! twod_flags_bytes_write {
    ( $twod:expr, $out:expr ) => {{
        for row in $twod.iter() {
            for v in row.iter() {
                $out.write_all(v)?;
            }
        }
    }};
}

// Methods to load from MapFile
impl MapFile {
    pub fn unpack<R: io::Read + io::Seek>(rdr: &mut R) -> GenericResult<Self> {
        let default = Self::default();
        let mut map_magic = [0u8; 4];
        rdr.read_exact(&mut map_magic)?;
        if default.map_magic != map_magic {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("MAGIC MAP CHECK FAILED: wrong magic? expect {:?}, got {:?}", default.map_magic, map_magic),
            )));
        }
        let mut map_version_magic = [0u8; 4];
        rdr.read_exact(&mut map_version_magic)?;
        if default.map_version_magic != map_version_magic {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("MAGIC MAP CHECK FAILED: wrong map version? expect {:?}, got {:?}", default.map_magic, map_magic),
            )));
        }
        let map_build_magic = rdr.read_u32::<LittleEndian>()?;
        let map_area_map_offset = rdr.read_u32::<LittleEndian>()?;
        let map_area_map_size = rdr.read_u32::<LittleEndian>()?;
        let map_height_map_offset = rdr.read_u32::<LittleEndian>()?;
        let map_height_map_size = rdr.read_u32::<LittleEndian>()?;
        let map_liquid_map_offset = rdr.read_u32::<LittleEndian>()?;
        let map_liquid_map_size = rdr.read_u32::<LittleEndian>()?;
        let map_holes_offset = rdr.read_u32::<LittleEndian>()?;
        let map_holes_size = rdr.read_u32::<LittleEndian>()?;

        let curr = rdr.stream_position()?;
        if curr != map_area_map_offset as u64 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "SANITY CHECK: wrong offset? at map_area_map_offset, expect {:?}, got {:?}",
                    map_area_map_offset, curr
                ),
            )));
        }
        let mut map_area_header_fourcc = [0u8; 4];
        rdr.read_exact(&mut map_area_header_fourcc)?;
        if default.map_area_header_fourcc != map_area_header_fourcc {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "MAP_AREA_HEADER_FOURCC FAILED: wrong magic? expect {:?}, got {:?}",
                    default.map_area_header_fourcc, map_area_header_fourcc
                ),
            )));
        }
        let f = rdr.read_u16::<LittleEndian>()?;
        let map_area_header_flags = FlagSet::new(f).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: got {:?}, err was {}", f, e)))?;
        let map_area_header_grid_area = rdr.read_u16::<LittleEndian>()?;
        let mut map_area_ids = [[0; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        if !map_area_header_flags.contains(MapAreaFlag::NoArea) {
            twod_little_endian_read!(map_area_ids, rdr, read_u16);
        }
        let curr = rdr.stream_position()?;
        if curr != map_height_map_offset as u64 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "SANITY CHECK: wrong offset? at map_height_map_offset, expect {:?}, got {:?}",
                    map_height_map_offset, curr
                ),
            )));
        }
        let mut map_height_header_fourcc = [0; 4];
        rdr.read_exact(&mut map_height_header_fourcc)?;
        if default.map_height_header_fourcc != map_height_header_fourcc {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "MAP_HEIGHT_HEADER_FOURCC FAILED: wrong magic? expect {:?}, got {:?}",
                    default.map_height_header_fourcc, map_height_header_fourcc
                ),
            )));
        }
        let f = rdr.read_u32::<LittleEndian>()?;
        let map_height_header_flags =
            FlagSet::new(f).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: got {:?}, err was {}", f, e)))?;
        let map_height_header_grid_height = rdr.read_f32::<LittleEndian>()?;
        let map_height_header_grid_max_height = rdr.read_f32::<LittleEndian>()?;
        #[allow(non_snake_case)]
        let mut map_height_V9 = [[0f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1];
        #[allow(non_snake_case)]
        let mut map_height_V8 = [[0f32; ADT_GRID_SIZE]; ADT_GRID_SIZE];
        if !map_height_header_flags.contains(MapHeightFlag::NoHeight) {
            twod_little_endian_read!(map_height_V9, rdr, read_f32);
            twod_little_endian_read!(map_height_V8, rdr, read_f32);
        }
        let mut map_height_flight_box_max_min = None;
        if map_height_header_flags.contains(MapHeightFlag::HasFlightBounds) {
            let mut fb_max = Matrix3::zeros();
            let mut fb_min = Matrix3::zeros();
            matrix_little_endian_read!(fb_max, rdr, read_i16);
            matrix_little_endian_read!(fb_min, rdr, read_i16);
            map_height_flight_box_max_min = Some((fb_max, fb_min))
        }
        let mut map_liquid_header_fourcc = [0u8; 4];
        let mut map_liquid_header_flags = None.into();
        let mut map_liquid_header_liquid_flags = None.into();
        let mut map_liquid_header_liquid_type = 0;
        let mut map_liquid_header_offset_x = 0;
        let mut map_liquid_header_offset_y = 0;
        let mut map_liquid_header_width = 0;
        let mut map_liquid_header_height = 0;
        let mut map_liquid_header_liquid_level = 0f32;
        let mut map_liquid_entry = [[0u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        let mut map_liquid_flags = [[None.into(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        let mut map_liquid_height_map = vec![];
        if map_liquid_map_offset != 0 {
            let curr = rdr.stream_position()?;
            if curr != map_liquid_map_offset as u64 {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "SANITY CHECK: wrong offset? at map_liquid_map_offset, expect {:?}, got {:?}",
                        map_liquid_map_offset, curr
                    ),
                )));
            }
            rdr.read_exact(&mut map_liquid_header_fourcc)?;
            if default.map_liquid_header_fourcc != map_liquid_header_fourcc {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "MAP_LIQUID_HEADER_FOURCC FAILED: wrong magic? expect {:?}, got {:?}",
                        default.map_liquid_header_fourcc, map_liquid_header_fourcc
                    ),
                )));
            }
            let f = rdr.read_u8()?;
            map_liquid_header_flags =
                FlagSet::new(f).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: got {:?}, err was {}", f, e)))?;
            let f = rdr.read_u8()?;
            map_liquid_header_liquid_flags =
                FlagSet::new(f).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: got {:?}, err was {}", f, e)))?;
            map_liquid_header_liquid_type = rdr.read_u16::<LittleEndian>()?;
            map_liquid_header_offset_x = rdr.read_u8()?;
            map_liquid_header_offset_y = rdr.read_u8()?;
            map_liquid_header_width = rdr.read_u8()?;
            map_liquid_header_height = rdr.read_u8()?;
            map_liquid_header_liquid_level = rdr.read_f32::<LittleEndian>()?;

            if !map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoType) {
                twod_little_endian_read!(map_liquid_entry, rdr, read_u16);
                for row in map_liquid_flags.iter_mut() {
                    for v in row.iter_mut() {
                        let f = rdr.read_u8()?;
                        *v = FlagSet::new(f).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: got {:?}, err was {}", f, e)))?;
                    }
                }
            }
            if !map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoHeight) {
                let liq_height_map_len = map_liquid_header_width as usize * map_liquid_header_height as usize;
                for _ in 0..liq_height_map_len {
                    map_liquid_height_map.push(rdr.read_f32::<LittleEndian>()?);
                }
            }
        }
        let mut map_holes = None;
        if map_holes_offset != 0 {
            let curr = rdr.stream_position()?;
            if curr != map_holes_offset as u64 {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!("SANITY CHECK: wrong offset? at map_holes_offset, expect {:?}, got {:?}", map_holes_offset, curr),
                )));
            }
            let mut holes = [[[0u8; 8]; 16]; 16];
            for row in holes.iter_mut() {
                for v in row.iter_mut() {
                    rdr.read_exact(v)?;
                }
            }

            map_holes = Some(holes);
        }
        sanity_check_read_all_bytes_from_reader!(rdr)?;
        Ok(Self {
            map_magic,
            map_version_magic,
            map_build_magic,
            map_area_map_offset,
            map_area_map_size,
            map_height_map_offset,
            map_height_map_size,
            map_liquid_map_offset,
            map_liquid_map_size,
            map_holes_offset,
            map_holes_size,
            map_area_header_fourcc,
            map_area_header_flags,
            map_area_header_grid_area,
            map_area_ids,
            map_height_header_fourcc,
            map_height_header_flags,
            map_height_header_grid_height,
            map_height_header_grid_max_height,
            map_height_V9,
            map_height_V8,
            map_height_flight_box_max_min,
            map_liquid_header_fourcc,
            map_liquid_header_flags,
            map_liquid_header_liquid_flags,
            map_liquid_header_liquid_type,
            map_liquid_header_offset_x,
            map_liquid_header_offset_y,
            map_liquid_header_width,
            map_liquid_header_height,
            map_liquid_header_liquid_level,
            map_liquid_entry,
            map_liquid_flags,
            map_liquid_height_map,
            map_holes,
        })
        // TODO: Compare w/ the unpacked
        // default.map_area_header_fourcc
        // default.map_height_header_fourcc
        // default.map_liquid_header_fourcc
    }
}

// Methods to pack into MapFile
impl MapFile {
    fn pack_area_data(&mut self) {
        //============================================
        // Try pack area data
        //============================================
        let area_id = self.map_area_ids[0][0];
        let full_area_data = self.map_area_ids.iter().any(|row| row.iter().any(|map_area_id| area_id != *map_area_id));

        if full_area_data {
            self.map_area_header_grid_area = 0;
        } else {
            self.map_area_header_grid_area = area_id;
            self.map_area_header_flags |= MapAreaFlag::NoArea;
        }
    }

    fn pack_height_data(&mut self, allow_height_limit: bool, use_min_height: f32) {
        //============================================
        // Try pack height data
        //============================================
        let max_min = self.map_height_V8.iter().fold((-20000f32, 20000f32), |t, row| {
            row.iter().fold(t, |(max_h, min_h), v| (max_h.max(*v), min_h.max(*v)))
        });
        let (mut max_height, mut min_height) = self
            .map_height_V9
            .iter()
            .fold(max_min, |t, row| row.iter().fold(t, |(max_h, min_h), v| (max_h.max(*v), min_h.max(*v))));

        // Check for allow limit minimum height (not store height in deep ochean - allow save some memory)
        if allow_height_limit && min_height < use_min_height {
            self.map_height_V8
                .iter_mut()
                .for_each(|row| row.iter_mut().for_each(|v| *v = v.max(use_min_height)));
            self.map_height_V9
                .iter_mut()
                .for_each(|row| row.iter_mut().for_each(|v| *v = v.max(use_min_height)));

            max_height = max_height.max(use_min_height);
            min_height = min_height.max(use_min_height);
        }

        //     map.heightMapOffset = map.areaMapOffset + map.areaMapSize;
        //     map.heightMapSize = sizeof(map_heightHeader);

        //     map_heightHeader heightHeader;
        self.map_height_header_grid_height = min_height;
        self.map_height_header_grid_max_height = max_height;

        if max_height == min_height {
            self.map_height_header_flags |= MapHeightFlag::NoHeight;
        }

        //     // Not need store if flat surface
        //     if (CONF_allow_float_to_int && (maxHeight - minHeight) < CONF_flat_height_delta_limit)
        //         heightHeader.flags |= MAP_HEIGHT_NO_HEIGHT;

        if self.map_height_flight_box_max_min.is_some() {
            self.map_height_header_flags |= MapHeightFlag::HasFlightBounds;
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
    }

    /// Returns a triple denoting the first liquid type, the first liquid flag, as well as whether or not
    /// its a full type
    fn overall_liquid_info(&self) -> (u16, FlagSet<MapLiquidTypeFlag>, bool) {
        let first_liquid_type = self.map_liquid_entry[0][0];
        let first_liquid_flag = self.map_liquid_flags[0][0];

        let mut it = iter::zip(self.map_liquid_entry, self.map_liquid_flags);

        let full_type = it.any(|(tyit, flit)| {
            let mut it = iter::zip(tyit, flit);
            it.any(|(ty, fl)| ty != first_liquid_type || fl != first_liquid_flag)
        });

        (first_liquid_type, first_liquid_flag, full_type)
    }

    fn pack_liquid_data(&mut self, liquid_show: [[bool; ADT_GRID_SIZE]; ADT_GRID_SIZE], map_liquid_height: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1]) {
        //============================================
        // Pack liquid data
        //============================================
        let (first_liquid_type, first_liquid_flag, full_type) = self.overall_liquid_info();
        // no water data (if all grid have 0 liquid type)
        if first_liquid_flag.bits() == 0 && !full_type {
            // No liquid data
            return;
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
        self.map_liquid_header_offset_x = min_x;
        self.map_liquid_header_offset_y = min_y;
        self.map_liquid_header_width = max_x - min_x + 1 + 1;
        self.map_liquid_header_height = max_y - min_y + 1 + 1;
        self.map_liquid_header_liquid_level = min_height;

        if max_height == min_height {
            self.map_liquid_header_flags |= MapLiquidHeaderFlag::NoHeight;
        }

        // // Not need store if flat surface
        // if (CONF_allow_float_to_int && (maxHeight - minHeight) < CONF_flat_liquid_delta_limit)
        //     self.liquidHeader_flags |= MapLiquidHeaderFlag::NO_HEIGHT;

        if !full_type {
            self.map_liquid_header_flags |= MapLiquidHeaderFlag::NoType;
        }

        if self.map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoType) {
            self.map_liquid_header_liquid_flags = first_liquid_flag;
            self.map_liquid_header_liquid_type = first_liquid_type;
        }

        // _liquidMap = new float[uint32(_liquidWidth) * uint32(_liquidHeight)];

        if !self.map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoHeight) {
            // map.liquidMapSize += sizeof(float) * liquidHeader.width * liquidHeader.height;
            for y in 0..self.map_liquid_header_height {
                let y_off: usize = (y + self.map_liquid_header_offset_y).try_into().unwrap();
                let x_off: usize = self.map_liquid_header_offset_x.try_into().unwrap();
                let liq_header_width: usize = self.map_liquid_header_width.try_into().unwrap();
                for h in map_liquid_height[y_off][x_off..x_off + liq_header_width].iter() {
                    self.map_liquid_height_map.push(*h);
                }
            }
            let liq_height_map_len = self.map_liquid_header_width as usize * self.map_liquid_header_height as usize;
            if self.map_liquid_height_map.len() != liq_height_map_len {
                panic!(
                    "MAP LIQUID HEIGHT MAP LEN DOES NOT MATCH CALCULATED LEN: calc was {liq_height_map_len}, data from header was: {}",
                    self.map_liquid_height_map.len()
                );
            }
        }
    }

    // write map file contents to the start of the file. The position is reset to 0 first and then
    // the end position of the output write handler just after the header's data
    fn write_mapfile_header<W: io::Write + io::Seek>(&self, out: &mut W) -> GenericResult<()> {
        // go to start
        out.seek(io::SeekFrom::Start(0))?;
        out.write_all(&self.map_magic)?;
        out.write_all(&self.map_version_magic)?;
        out.write_all(&self.map_build_magic.to_le_bytes())?;
        out.write_all(&self.map_area_map_offset.to_le_bytes())?;
        out.write_all(&self.map_area_map_size.to_le_bytes())?;
        out.write_all(&self.map_height_map_offset.to_le_bytes())?;
        out.write_all(&self.map_height_map_size.to_le_bytes())?;
        out.write_all(&self.map_liquid_map_offset.to_le_bytes())?;
        out.write_all(&self.map_liquid_map_size.to_le_bytes())?;
        out.write_all(&self.map_holes_offset.to_le_bytes())?;
        out.write_all(&self.map_holes_size.to_le_bytes())?;
        Ok(())
    }

    pub fn pack<W: io::Write + io::Seek>(
        &mut self,
        allow_height_limit: bool,
        liquid_show: [[bool; ADT_GRID_SIZE]; ADT_GRID_SIZE],
        map_liquid_height: [[f32; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1],
        use_min_height: f32,
        out: &mut W,
    ) -> GenericResult<()> {
        self.pack_area_data();
        self.pack_height_data(allow_height_limit, use_min_height);
        self.pack_liquid_data(liquid_show, map_liquid_height);
        // everything is packed, now begin handling offsets and sizes
        // prepopulate map_file header first
        self.write_mapfile_header(out)?;
        // write map area and set its offset
        let current: u32 = out.stream_position()?.try_into()?;
        self.map_area_map_offset = current;
        out.write_all(&self.map_area_header_fourcc)?;
        out.write_all(&self.map_area_header_flags.bits().to_le_bytes())?;
        out.write_all(&self.map_area_header_grid_area.to_le_bytes())?;
        if !self.map_area_header_flags.contains(MapAreaFlag::NoArea) {
            twod_little_endian_write!(self.map_area_ids, out);
        }
        let current: u32 = out.stream_position()?.try_into()?;
        self.map_area_map_size = current - self.map_area_map_offset;
        // Store height data
        self.map_height_map_offset = current;
        out.write_all(&self.map_height_header_fourcc)?;
        out.write_all(&self.map_height_header_flags.bits().to_le_bytes())?;
        out.write_all(&self.map_height_header_grid_height.to_le_bytes())?;
        out.write_all(&self.map_height_header_grid_max_height.to_le_bytes())?;
        if !self.map_height_header_flags.contains(MapHeightFlag::NoHeight) {
            twod_little_endian_write!(self.map_height_V9, out);
            twod_little_endian_write!(self.map_height_V8, out);
        }
        if let Some((fb_max, fb_min)) = self.map_height_flight_box_max_min {
            matrix_little_endian_write!(fb_max, out);
            matrix_little_endian_write!(fb_min, out);
        }
        let current: u32 = out.stream_position()?.try_into()?;
        self.map_height_map_size = current - self.map_height_map_offset;

        // Store liquid data if needed
        let (_, first_liquid_flag, full_type) = self.overall_liquid_info();
        if first_liquid_flag.bits() != 0 || full_type {
            self.map_liquid_map_offset = current;
            out.write_all(&self.map_liquid_header_fourcc)?;
            out.write_all(&self.map_liquid_header_flags.bits().to_le_bytes())?;
            out.write_all(&self.map_liquid_header_liquid_flags.bits().to_le_bytes())?;
            out.write_all(&self.map_liquid_header_liquid_type.to_le_bytes())?;
            out.write_all(&self.map_liquid_header_offset_x.to_le_bytes())?;
            out.write_all(&self.map_liquid_header_offset_y.to_le_bytes())?;
            out.write_all(&self.map_liquid_header_width.to_le_bytes())?;
            out.write_all(&self.map_liquid_header_height.to_le_bytes())?;
            out.write_all(&self.map_liquid_header_liquid_level.to_le_bytes())?;
            if !self.map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoType) {
                twod_little_endian_write!(self.map_liquid_entry, out);
                twod_flags_little_endian_write!(self.map_liquid_flags, out);
            }
            if !self.map_liquid_header_flags.contains(MapLiquidHeaderFlag::NoHeight) {
                for h in self.map_liquid_height_map.iter() {
                    out.write_all(&h.to_le_bytes())?;
                }
            }
            let current: u32 = out.stream_position()?.try_into()?;
            self.map_liquid_map_size = current - self.map_liquid_map_offset;
        }
        if let Some(h) = self.map_holes {
            let current: u32 = out.stream_position()?.try_into()?;
            self.map_holes_offset = current;
            twod_flags_bytes_write!(h, out);
            let current: u32 = out.stream_position()?.try_into()?;
            self.map_holes_size = current - self.map_holes_offset;
        }
        // go back and re-write the offsets
        self.write_mapfile_header(out)?;
        Ok(())
    }
}
