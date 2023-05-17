use std::io::{self, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::tools::basic_extractor::FileChunk;

pub const ADT_CELLS_PER_GRID: usize = 16;
pub const ADT_CELL_SIZE: usize = 8;
pub const ADT_GRID_SIZE: usize = ADT_CELLS_PER_GRID * ADT_CELL_SIZE;

pub struct AdtChunkMcnkSubchunkMcvt {
    pub fcc:        [u8; 4],
    pub size:       u32,
    pub height_map: [f32; (ADT_CELL_SIZE + 1) * (ADT_CELL_SIZE + 1) + ADT_CELL_SIZE * ADT_CELL_SIZE],
}

impl From<FileChunk> for AdtChunkMcnkSubchunkMcvt {
    fn from(value: FileChunk) -> Self {
        if value.fcc != *b"MCVT" {
            panic!("value.fcc must be MCVT, got {}", std::str::from_utf8(&value.fcc[..]).unwrap());
        }
        let mut height_map = [0f32; (ADT_CELL_SIZE + 1) * (ADT_CELL_SIZE + 1) + ADT_CELL_SIZE * ADT_CELL_SIZE];
        let mut cursor = io::Cursor::new(value.data);
        for h in height_map.iter_mut() {
            *h = cursor.read_f32::<LittleEndian>().unwrap();
        }
        Self {
            fcc: value.fcc,
            size: value.size,
            height_map,
        }
    }
}

pub enum LiquidVertexFormatType {
    HeightDepth = 0,
    HeightTextureCoord = 1,
    Depth = 2,
    HeightDepthTextureCoord = 3,
    Unk4 = 4,
    Unk5 = 5,
}

pub enum AdtLiquidType {
    Water = 0,
    Ocean = 1,
    Magma = 2,
    Slime = 3,
}

#[derive(Default, Clone, Copy)]
pub struct AdtChunkMcnkSubchunkMclqLiquidData {
    pub light:  u32,
    pub height: f32,
}

///
/// Adt file liquid map chunk (old)
///
pub struct AdtChunkMcnkSubchunkMclq {
    pub fcc:     [u8; 4],
    pub size:    u32,
    pub height1: f32,
    pub height2: f32,
    pub liquid:  [[AdtChunkMcnkSubchunkMclqLiquidData; ADT_CELL_SIZE + 1]; ADT_CELL_SIZE + 1],

    /// ```
    /// 1<<0 - ocean
    /// 1<<1 - lava/slime
    /// 1<<2 - water
    /// 1<<6 - all water
    /// 1<<7 - dark water
    /// == 0x0F - not show liquid
    /// ```
    pub flags: [[u8; ADT_CELL_SIZE]; ADT_CELL_SIZE],
    pub data:  [u8; 84],
}

impl From<FileChunk> for AdtChunkMcnkSubchunkMclq {
    fn from(value: FileChunk) -> Self {
        if value.fcc != *b"MCLQ" {
            panic!("value.fcc must be MCLQ, got {}", std::str::from_utf8(&value.fcc[..]).unwrap());
        }
        let mut liquid = [[AdtChunkMcnkSubchunkMclqLiquidData::default(); ADT_CELL_SIZE + 1]; ADT_CELL_SIZE + 1];
        let mut flags = [[0u8; ADT_CELL_SIZE]; ADT_CELL_SIZE];
        let mut data = [0u8; 84];

        // start reading
        let mut cursor = io::Cursor::new(value.data);
        let height1 = cursor.read_f32::<LittleEndian>().unwrap();
        let height2 = cursor.read_f32::<LittleEndian>().unwrap();
        for liqy in liquid.iter_mut() {
            for liquid_data in liqy.iter_mut() {
                liquid_data.light = cursor.read_u32::<LittleEndian>().unwrap();
                liquid_data.height = cursor.read_f32::<LittleEndian>().unwrap();
            }
        }
        for flags_row in flags.iter_mut() {
            cursor.read_exact(&mut flags_row[..]).unwrap()
        }
        cursor.read_exact(&mut data[..]).unwrap();
        Self {
            fcc: value.fcc,
            size: value.size,
            height1,
            height2,
            liquid,
            flags,
            data,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AdtChunkMh2oSmLiquidChunk {
    pub offset_instances: u32,
    pub used:             u32,
    offset_attributes:    u32,
}

#[derive(Default, Clone, Copy)]
pub struct AdtChunkMh2oLiquidInstance {
    // Index from LiquidType.db2
    pub liquid_type:          u16,
    // Id from LiquidObject.db2 if >= 42
    pub liquid_vertex_format: u16,
    pub min_height_level:     f32,
    pub max_height_level:     f32,
    pub offset_x:             u8,
    pub offset_y:             u8,
    pub width:                u8,
    pub height:               u8,
    offset_exists_bitmap:     u32,
    pub offset_vertex_data:   u32,
}

impl AdtChunkMh2oLiquidInstance {
    pub fn get_offset_x(&self) -> usize {
        if self.liquid_vertex_format < 42 {
            self.offset_x as usize
        } else {
            0
        }
    }

    pub fn get_offset_y(&self) -> usize {
        if self.liquid_vertex_format < 42 {
            self.offset_y as usize
        } else {
            0
        }
    }

    pub fn get_width(&self) -> usize {
        if self.liquid_vertex_format < 42 {
            self.width as usize
        } else {
            8
        }
    }

    pub fn get_height(&self) -> usize {
        if self.liquid_vertex_format < 42 {
            self.height as usize
        } else {
            8
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct AdtChunkMh2oLiquidAttributes {
    pub fishable: u64,
    pub deep:     u64,
}

//
// Adt file liquid data chunk (new)
//
pub struct AdtChunkMh2o {
    pub fcc:               [u8; 4],
    pub size:              u32,
    pub liquid:            [[AdtChunkMh2oSmLiquidChunk; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub liquid_instance:   [[AdtChunkMh2oLiquidInstance; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub liquid_attributes: [[AdtChunkMh2oLiquidAttributes; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub raw_data:          io::Cursor<Vec<u8>>,
}

impl From<FileChunk> for AdtChunkMh2o {
    fn from(value: FileChunk) -> Self {
        if value.fcc != *b"MH2O" {
            panic!("value.fcc must be MH2O, got {}", std::str::from_utf8(&value.fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(value.data);
        let mut liquid = [[AdtChunkMh2oSmLiquidChunk::default(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        let mut liquid_instance = [[AdtChunkMh2oLiquidInstance::default(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        let mut liquid_attributes = [[AdtChunkMh2oLiquidAttributes::default(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        for (_i, liq_row) in liquid.iter_mut().enumerate() {
            for (_j, liq) in liq_row.iter_mut().enumerate() {
                liq.offset_instances = cursor.read_u32::<LittleEndian>().unwrap();
                liq.used = cursor.read_u32::<LittleEndian>().unwrap();
                liq.offset_attributes = cursor.read_u32::<LittleEndian>().unwrap();
            }
        }

        for i in 0..liquid.len() {
            for j in 0..liquid[i].len() {
                if liquid[i][j].used == 0 {
                    continue;
                }
                if liquid[i][j].offset_instances > 0 {
                    cursor.set_position(liquid[i][j].offset_instances as u64);
                    liquid_instance[i][j].liquid_type = cursor.read_u16::<LittleEndian>().unwrap();
                    liquid_instance[i][j].liquid_vertex_format = cursor.read_u16::<LittleEndian>().unwrap();
                    liquid_instance[i][j].min_height_level = cursor.read_f32::<LittleEndian>().unwrap();
                    liquid_instance[i][j].max_height_level = cursor.read_f32::<LittleEndian>().unwrap();
                    liquid_instance[i][j].offset_x = cursor.read_u8().unwrap();
                    liquid_instance[i][j].offset_y = cursor.read_u8().unwrap();
                    liquid_instance[i][j].width = cursor.read_u8().unwrap();
                    liquid_instance[i][j].height = cursor.read_u8().unwrap();
                    liquid_instance[i][j].offset_exists_bitmap = cursor.read_u32::<LittleEndian>().unwrap();
                    liquid_instance[i][j].offset_vertex_data = cursor.read_u32::<LittleEndian>().unwrap();
                }
                if liquid[i][j].offset_attributes > 0 {
                    cursor.set_position(liquid[i][j].offset_attributes as u64);
                    liquid_attributes[i][j].fishable = cursor.read_u64::<LittleEndian>().unwrap();
                    liquid_attributes[i][j].deep = cursor.read_u64::<LittleEndian>().unwrap();
                } else {
                    liquid_attributes[i][j].fishable = 0xFFFFFFFFFFFFFFFF;
                    liquid_attributes[i][j].deep = 0xFFFFFFFFFFFFFFFF;
                }
            }
        }
        cursor.set_position(0);
        Self {
            fcc: value.fcc,
            size: value.size,
            liquid,
            liquid_instance,
            liquid_attributes,
            raw_data: cursor,
        }
    }
}

impl AdtChunkMh2o {
    pub fn get_exists_bitmap(&mut self, i: usize, j: usize) -> u64 {
        let offset = self.liquid_instance[i][j].offset_exists_bitmap as u64;
        if offset > 0 {
            self.raw_data.set_position(self.liquid_instance[i][j].offset_exists_bitmap as u64);
            self.raw_data.read_u64::<LittleEndian>().unwrap()
        } else {
            u64::MAX
        }
    }
}

pub struct AdtChunkMfbo {
    pub fcc:  [u8; 4],
    pub size: u32,
    pub max:  [[i16; 3]; 3],
    pub min:  [[i16; 3]; 3],
}

impl From<FileChunk> for AdtChunkMfbo {
    fn from(value: FileChunk) -> Self {
        if value.fcc != *b"MFBO" {
            panic!("value.fcc must be MFBO, got {}", std::str::from_utf8(&value.fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(value.data);
        let mut max = [[0; 3]; 3];
        for row in max.iter_mut() {
            for v in row.iter_mut() {
                *v = cursor.read_i16::<LittleEndian>().unwrap();
            }
        }
        let mut min = [[0; 3]; 3];
        for row in min.iter_mut() {
            for v in row.iter_mut() {
                *v = cursor.read_i16::<LittleEndian>().unwrap();
            }
        }
        Self {
            fcc: value.fcc,
            size: value.size,
            max,
            min,
        }
    }
}

#[derive(Default)]
pub struct AdtChunkMcnk {
    pub fcc:              [u8; 4],
    pub size:             u32,
    pub flags:            u32,
    ix:                   u32,
    iy:                   u32,
    pub n_layers:         u32,
    pub n_doodad_refs:    u32,
    pub high_res_holes:   [u8; 8],
    /// Texture layer definitions
    /// offsMCLY in trinitycore, ofsLayer
    pub offs_mcly:        u32,
    /// A list of indices into the parent file's MDDF chunk
    /// offsMCRF in trinitycore, ofsRefs
    pub offs_mcrf:        u32,
    /// Alpha maps for additional texture layers
    /// offsMCAL in trinitycore, ofsAlpha
    pub offs_mcal:        u32,
    /// sizeMCAL in trinitycore, sizeAlpha
    pub size_mcal:        u32,
    /// Shadow map for static shadows on the terrain
    /// offsMCSH in trinitycore, ofsShadow
    pub offs_mcsh:        u32,
    /// sizeMCSH in trinitycore, sizeShadow
    pub size_mcsh:        u32,
    pub areaid:           u32,
    pub n_map_obj_refs:   u32,
    /// holes in trinitycore
    pub holes_low_res:    u16,
    /// unknown_but_used in documentation, in alpha: padding
    pub unknown_but_used: u16,
    /// aka ReallyLowQualityTextureingMap It is used to determine which detail doodads to show. Values are an array of two bit
    pub pred_tex:         [u8; 16],
    /// doodads disabled if 1; WoD: may be an explicit MCDD chunk. values are 8x8 arrays of 1 bit
    pub no_effect_doodad: [u8; 8],
    pub offs_mcse:        u32,
    pub n_snd_emitters:   u32,
    /// Liquid level (old)
    pub offs_mclq:        u32,
    pub size_mclq:        u32,
    pub zpos:             f32,
    pub xpos:             f32,
    pub ypos:             f32,
    /// offsColorValues in WotLK
    pub offs_mccv:        u32,
    pub props:            u32,
    pub effect_id:        u32,
}

impl From<FileChunk> for AdtChunkMcnk {
    // allow field reassing with default as cursor reading order matters
    #[allow(clippy::field_reassign_with_default)]
    fn from(value: FileChunk) -> Self {
        if value.fcc != *b"MCNK" {
            panic!("value.fcc must be MCNK, got {}", std::str::from_utf8(&value.fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(value.data);
        let mut data = Self::default();
        data.fcc = value.fcc;
        data.size = value.size;

        data.flags = cursor.read_u32::<LittleEndian>().unwrap();
        data.ix = cursor.read_u32::<LittleEndian>().unwrap();
        data.iy = cursor.read_u32::<LittleEndian>().unwrap();
        data.n_layers = cursor.read_u32::<LittleEndian>().unwrap();
        data.n_doodad_refs = cursor.read_u32::<LittleEndian>().unwrap();
        cursor.read_exact(&mut data.high_res_holes[..]).unwrap();
        data.offs_mcly = cursor.read_u32::<LittleEndian>().unwrap();
        data.offs_mcrf = cursor.read_u32::<LittleEndian>().unwrap();
        data.offs_mcal = cursor.read_u32::<LittleEndian>().unwrap();
        data.size_mcal = cursor.read_u32::<LittleEndian>().unwrap();
        data.offs_mcsh = cursor.read_u32::<LittleEndian>().unwrap();
        data.size_mcsh = cursor.read_u32::<LittleEndian>().unwrap();
        data.areaid = cursor.read_u32::<LittleEndian>().unwrap();
        data.n_map_obj_refs = cursor.read_u32::<LittleEndian>().unwrap();
        data.holes_low_res = cursor.read_u16::<LittleEndian>().unwrap();
        data.unknown_but_used = cursor.read_u16::<LittleEndian>().unwrap();
        cursor.read_exact(&mut data.pred_tex[..]).unwrap();
        cursor.read_exact(&mut data.no_effect_doodad[..]).unwrap();
        data.offs_mcse = cursor.read_u32::<LittleEndian>().unwrap();
        data.n_snd_emitters = cursor.read_u32::<LittleEndian>().unwrap();
        data.offs_mclq = cursor.read_u32::<LittleEndian>().unwrap();
        data.size_mclq = cursor.read_u32::<LittleEndian>().unwrap();
        data.zpos = cursor.read_f32::<LittleEndian>().unwrap();
        data.xpos = cursor.read_f32::<LittleEndian>().unwrap();
        data.ypos = cursor.read_f32::<LittleEndian>().unwrap();
        data.offs_mccv = cursor.read_u32::<LittleEndian>().unwrap();
        data.props = cursor.read_u32::<LittleEndian>().unwrap();
        data.effect_id = cursor.read_u32::<LittleEndian>().unwrap();

        data
    }
}

impl AdtChunkMcnk {
    pub fn ix(&self) -> usize {
        self.ix as usize
    }

    pub fn iy(&self) -> usize {
        self.iy as usize
    }
}
