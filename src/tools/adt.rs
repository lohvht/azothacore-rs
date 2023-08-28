use std::{
    collections::HashMap,
    io::{self, Read},
    path::Path,
};

use nalgebra::{Matrix3, Vector3};

use crate::{
    read_le_unwrap,
    server::game::map::{ADT_CELLS_PER_GRID, ADT_CELL_SIZE},
    tools::extractor_common::{casc_handles::CascStorageHandle, chunked_data_offsets, cstr_bytes_to_string, ChunkedFile},
    AzResult,
};

pub struct AdtChunkMcnkSubchunkMcvt {
    pub height_map: [f32; (ADT_CELL_SIZE + 1) * (ADT_CELL_SIZE + 1) + ADT_CELL_SIZE * ADT_CELL_SIZE],
}

impl From<(&[u8; 4], &[u8])> for AdtChunkMcnkSubchunkMcvt {
    fn from(value: (&[u8; 4], &[u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MCVT" {
            panic!("fcc must be MCVT, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let mut height_map = [0f32; (ADT_CELL_SIZE + 1) * (ADT_CELL_SIZE + 1) + ADT_CELL_SIZE * ADT_CELL_SIZE];
        let mut cursor = io::Cursor::new(data);
        for h in height_map.iter_mut() {
            *h = read_le_unwrap!(cursor, f32);
        }
        Self { height_map }
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

#[derive(Default, Clone, Copy)]
pub struct AdtChunkMcnkSubchunkMclqLiquidData {
    pub light:  u32,
    pub height: f32,
}

///
/// Adt file liquid map chunk (old)
///
pub struct AdtChunkMcnkSubchunkMclq {
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

impl From<(&[u8; 4], &[u8])> for AdtChunkMcnkSubchunkMclq {
    fn from(value: (&[u8; 4], &[u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MCLQ" {
            panic!("fcc must be MCLQ, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(data);
        let mut liquid = [[AdtChunkMcnkSubchunkMclqLiquidData::default(); ADT_CELL_SIZE + 1]; ADT_CELL_SIZE + 1];
        let mut flags = [[0u8; ADT_CELL_SIZE]; ADT_CELL_SIZE];
        let mut data = [0u8; 84];

        // start reading
        let height1 = read_le_unwrap!(cursor, f32);
        let height2 = read_le_unwrap!(cursor, f32);
        for liqy in liquid.iter_mut() {
            for liquid_data in liqy.iter_mut() {
                liquid_data.light = read_le_unwrap!(cursor, u32);
                liquid_data.height = read_le_unwrap!(cursor, f32);
            }
        }
        for flags_row in flags.iter_mut() {
            cursor.read_exact(&mut flags_row[..]).unwrap()
        }
        cursor.read_exact(&mut data[..]).unwrap();
        Self {
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
pub struct AdtChunkMh2o<'a> {
    pub liquid:            [[AdtChunkMh2oSmLiquidChunk; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub liquid_instance:   [[AdtChunkMh2oLiquidInstance; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub liquid_attributes: [[AdtChunkMh2oLiquidAttributes; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    pub raw_data:          io::Cursor<&'a [u8]>,
}

impl<'a> From<(&[u8; 4], &'a [u8])> for AdtChunkMh2o<'a> {
    fn from(value: (&[u8; 4], &'a [u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MH2O" {
            panic!("fcc must be MH2O, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(data);
        let mut liquid = [[AdtChunkMh2oSmLiquidChunk::default(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        let mut liquid_instance = [[AdtChunkMh2oLiquidInstance::default(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        let mut liquid_attributes = [[AdtChunkMh2oLiquidAttributes::default(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
        for (_i, liq_row) in liquid.iter_mut().enumerate() {
            for (_j, liq) in liq_row.iter_mut().enumerate() {
                liq.offset_instances = read_le_unwrap!(cursor, u32);
                liq.used = read_le_unwrap!(cursor, u32);
                liq.offset_attributes = read_le_unwrap!(cursor, u32);
            }
        }

        for i in 0..liquid.len() {
            for j in 0..liquid[i].len() {
                if liquid[i][j].used == 0 {
                    continue;
                }
                if liquid[i][j].offset_instances > 0 {
                    cursor.set_position(liquid[i][j].offset_instances as u64);
                    liquid_instance[i][j].liquid_type = read_le_unwrap!(cursor, u16);
                    liquid_instance[i][j].liquid_vertex_format = read_le_unwrap!(cursor, u16);
                    liquid_instance[i][j].min_height_level = read_le_unwrap!(cursor, f32);
                    liquid_instance[i][j].max_height_level = read_le_unwrap!(cursor, f32);
                    liquid_instance[i][j].offset_x = read_le_unwrap!(cursor, u8);
                    liquid_instance[i][j].offset_y = read_le_unwrap!(cursor, u8);
                    liquid_instance[i][j].width = read_le_unwrap!(cursor, u8);
                    liquid_instance[i][j].height = read_le_unwrap!(cursor, u8);
                    liquid_instance[i][j].offset_exists_bitmap = read_le_unwrap!(cursor, u32);
                    liquid_instance[i][j].offset_vertex_data = read_le_unwrap!(cursor, u32);
                }
                if liquid[i][j].offset_attributes > 0 {
                    cursor.set_position(liquid[i][j].offset_attributes as u64);
                    liquid_attributes[i][j].fishable = read_le_unwrap!(cursor, u64);
                    liquid_attributes[i][j].deep = read_le_unwrap!(cursor, u64);
                } else {
                    liquid_attributes[i][j].fishable = 0xFFFFFFFFFFFFFFFF;
                    liquid_attributes[i][j].deep = 0xFFFFFFFFFFFFFFFF;
                }
            }
        }
        cursor.set_position(0);
        Self {
            liquid,
            liquid_instance,
            liquid_attributes,
            raw_data: cursor,
        }
    }
}

impl<'a> AdtChunkMh2o<'a> {
    pub fn get_exists_bitmap(&mut self, i: usize, j: usize) -> u64 {
        let offset = self.liquid_instance[i][j].offset_exists_bitmap as u64;
        if offset > 0 {
            self.raw_data.set_position(self.liquid_instance[i][j].offset_exists_bitmap as u64);
            read_le_unwrap!(self.raw_data, u64)
        } else {
            u64::MAX
        }
    }
}

pub struct AdtChunkMfbo {
    pub max: Matrix3<i16>,
    pub min: Matrix3<i16>,
}

impl From<(&[u8; 4], &[u8])> for AdtChunkMfbo {
    fn from(value: (&[u8; 4], &[u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MFBO" {
            panic!("fcc must be MFBO, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(data);
        let mut max = Matrix3::zeros();
        for mut row in max.row_iter_mut() {
            for v in row.iter_mut() {
                *v = read_le_unwrap!(cursor, i16);
            }
        }
        let mut min = Matrix3::zeros();
        for mut row in min.row_iter_mut() {
            for v in row.iter_mut() {
                *v = read_le_unwrap!(cursor, i16);
            }
        }
        Self { max, min }
    }
}

#[derive(Default)]
pub struct AdtChunkMcnk {
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
    pub mcvt:             Option<AdtChunkMcnkSubchunkMcvt>,
    pub mclq:             Option<AdtChunkMcnkSubchunkMclq>,
}

impl From<(&[u8; 4], &[u8])> for AdtChunkMcnk {
    // allow field reassing with default as cursor reading order matters
    fn from(value: (&[u8; 4], &[u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MCNK" {
            panic!("value.fcc must be MCNK, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(data);

        let flags = read_le_unwrap!(cursor, u32);
        let ix = read_le_unwrap!(cursor, u32);
        let iy = read_le_unwrap!(cursor, u32);
        let n_layers = read_le_unwrap!(cursor, u32);
        let n_doodad_refs = read_le_unwrap!(cursor, u32);
        let mut high_res_holes = [0u8; 8];
        cursor.read_exact(&mut high_res_holes[..]).unwrap();
        let offs_mcly = read_le_unwrap!(cursor, u32);
        let offs_mcrf = read_le_unwrap!(cursor, u32);
        let offs_mcal = read_le_unwrap!(cursor, u32);
        let size_mcal = read_le_unwrap!(cursor, u32);
        let offs_mcsh = read_le_unwrap!(cursor, u32);
        let size_mcsh = read_le_unwrap!(cursor, u32);
        let areaid = read_le_unwrap!(cursor, u32);
        let n_map_obj_refs = read_le_unwrap!(cursor, u32);
        let holes_low_res = read_le_unwrap!(cursor, u16);
        let unknown_but_used = read_le_unwrap!(cursor, u16);
        let mut pred_tex = [0u8; 16];
        cursor.read_exact(&mut pred_tex[..]).unwrap();
        let mut no_effect_doodad = [0u8; 8];
        cursor.read_exact(&mut no_effect_doodad[..]).unwrap();
        let offs_mcse = read_le_unwrap!(cursor, u32);
        let n_snd_emitters = read_le_unwrap!(cursor, u32);
        let offs_mclq = read_le_unwrap!(cursor, u32);
        let size_mclq = read_le_unwrap!(cursor, u32);
        let zpos = read_le_unwrap!(cursor, f32);
        let xpos = read_le_unwrap!(cursor, f32);
        let ypos = read_le_unwrap!(cursor, f32);
        let offs_mccv = read_le_unwrap!(cursor, u32);
        let props = read_le_unwrap!(cursor, u32);
        let effect_id = read_le_unwrap!(cursor, u32);

        // Process the rest of the subchunks
        let chunk_data = cursor.remaining_slice();
        let mut mcvt = None;
        let mut mclq = None;
        for (fourcc, start, end) in chunked_data_offsets(chunk_data).unwrap().into_iter() {
            match &fourcc {
                b"MCVT" => {
                    if mcvt.is_some() {
                        panic!("MCVT IS ALREADY SET");
                    }
                    mcvt = Some(AdtChunkMcnkSubchunkMcvt::from((&fourcc, &chunk_data[start..end])));
                },
                b"MCLQ" => {
                    if mclq.is_some() {
                        panic!("MCLQ IS ALREADY SET");
                    }
                    mclq = Some(AdtChunkMcnkSubchunkMclq::from((&fourcc, &chunk_data[start..end])));
                },
                _ => {},
            }
        }
        Self {
            flags,
            ix,
            iy,
            n_layers,
            n_doodad_refs,
            high_res_holes,
            offs_mcly,
            offs_mcrf,
            offs_mcal,
            size_mcal,
            offs_mcsh,
            size_mcsh,
            areaid,
            n_map_obj_refs,
            holes_low_res,
            unknown_but_used,
            pred_tex,
            no_effect_doodad,
            offs_mcse,
            n_snd_emitters,
            offs_mclq,
            size_mclq,
            zpos,
            xpos,
            ypos,
            offs_mccv,
            props,
            effect_id,
            mcvt,
            mclq,
        }
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

#[derive(Debug)]
pub struct AdtChunkModf {
    pub map_object_defs: Vec<AdtMapObjectDefs>,
}

#[derive(Debug)]
pub struct AdtMapObjectDefs {
    pub id:         u32,
    pub unique_id:  u32,
    pub position:   Vector3<f32>,
    pub rotation:   Vector3<f32>,
    pub bounds:     [Vector3<f32>; 2],
    pub flags:      u16,
    /// can be larger than number of doodad sets in WMO
    pub doodad_set: u16,
    pub name_set:   u16,
    pub scale:      u16,
}

impl From<(&[u8; 4], &[u8])> for AdtChunkModf {
    fn from(value: (&[u8; 4], &[u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MODF" {
            panic!("fcc must be MODF, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let data_len: usize = data.len();
        let mut cursor = io::Cursor::new(data);
        let mut map_object_defs = Vec::new();

        while cursor.position() < data_len as u64 {
            let id = read_le_unwrap!(cursor, u32);
            let unique_id = read_le_unwrap!(cursor, u32);
            let position = Vector3::new(
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
            );
            let rotation = Vector3::new(
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
            );
            let bounds = [
                Vector3::new(
                    read_le_unwrap!(cursor, f32),
                    read_le_unwrap!(cursor, f32),
                    read_le_unwrap!(cursor, f32),
                ),
                Vector3::new(
                    read_le_unwrap!(cursor, f32),
                    read_le_unwrap!(cursor, f32),
                    read_le_unwrap!(cursor, f32),
                ),
            ];
            let flags = read_le_unwrap!(cursor, u16);
            let doodad_set = read_le_unwrap!(cursor, u16);
            let name_set = read_le_unwrap!(cursor, u16);
            let scale = read_le_unwrap!(cursor, u16);
            map_object_defs.push(AdtMapObjectDefs {
                id,
                unique_id,
                position,
                rotation,
                bounds,
                flags,
                doodad_set,
                name_set,
                scale,
            });
        }
        Self { map_object_defs }
    }
}

pub struct AdtChunkMddf {
    pub doodad_defs: Vec<AdtDoodadDef>,
}

#[derive(Debug)]
pub struct AdtDoodadDef {
    pub id:        u32,
    pub unique_id: u32,
    pub position:  Vector3<f32>,
    pub rotation:  Vector3<f32>,
    pub scale:     u16,
    pub flags:     u16,
}

impl From<(&[u8; 4], &[u8])> for AdtChunkMddf {
    fn from(value: (&[u8; 4], &[u8])) -> Self {
        let (fcc, data) = value;
        if fcc != b"MDDF" {
            panic!("fcc must be MDDF, got {}", std::str::from_utf8(&fcc[..]).unwrap());
        }
        let mut cursor = io::Cursor::new(data);
        let mut doodad_defs = Vec::new();
        while !cursor.is_empty() {
            let id = read_le_unwrap!(cursor, u32);
            let unique_id = read_le_unwrap!(cursor, u32);
            let position = Vector3::new(
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
            );
            let rotation = Vector3::new(
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
                read_le_unwrap!(cursor, f32),
            );
            let scale = read_le_unwrap!(cursor, u16);
            let flags = read_le_unwrap!(cursor, u16);
            doodad_defs.push(AdtDoodadDef {
                id,
                unique_id,
                position,
                rotation,
                scale,
                flags,
            });
        }
        Self { doodad_defs }
    }
}

pub struct ADTFile {
    pub mddf:        Vec<AdtChunkMddf>,
    pub modf:        Vec<AdtChunkModf>,
    pub model_paths: HashMap<usize, String>,
    pub wmo_paths:   HashMap<usize, String>,
}

impl ADTFile {
    pub fn build<P: AsRef<Path>>(storage: &CascStorageHandle, storage_path: P) -> AzResult<Self> {
        let file = ChunkedFile::build(storage, &storage_path)?;
        // .inspect_err(|e| {
        //     error!("Error opening adt file at {}, err was {e}", storage_path.as_ref().display());
        // })?;
        let mut mddf = vec![];
        let mut modf = vec![];
        let mut model_paths = HashMap::new();
        let mut wmo_paths = HashMap::new();

        for (fourcc, chunk) in file.chunks() {
            match fourcc {
                b"MCIN" => {},
                b"MTEX" => {},
                b"MMDX" => {
                    let mut offset = 0;
                    let paths = chunk
                        .split_inclusive(|b| *b == 0)
                        .map(|raw| {
                            // We dont anticipate a panic here as the strings will always be nul-terminated
                            let s = cstr_bytes_to_string(raw).unwrap();
                            let r = (offset, s);
                            offset += 1; // raw.len();
                            r
                        })
                        .collect::<HashMap<_, _>>();
                    model_paths.extend(paths);
                },
                b"MWMO" => {
                    let mut offset = 0;
                    let paths = chunk
                        .split_inclusive(|b| *b == 0)
                        .map(|raw| {
                            // We dont anticipate a panic here as the strings will always be nul-terminated
                            let s = cstr_bytes_to_string(raw).unwrap();
                            let r = (offset, s);
                            offset += 1; // raw.len();
                            r
                        })
                        .collect::<HashMap<_, _>>();
                    wmo_paths.extend(paths);
                },
                //======================
                b"MDDF" => {
                    mddf.push(AdtChunkMddf::from((fourcc, chunk)));
                },
                b"MODF" => {
                    modf.push(AdtChunkModf::from((fourcc, chunk)));
                },
                _ => {},
            }
        }
        Ok(Self {
            mddf,
            modf,
            model_paths,
            wmo_paths,
        })
    }
}
