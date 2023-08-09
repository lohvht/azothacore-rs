use std::{
    collections::{BTreeSet, HashMap, HashSet},
    io::{self, Read},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use flagset::{flags, FlagSet};
use nalgebra::{Vector3, Vector4};
use tracing::{error, info};

use crate::{
    az_error,
    common::collision::maps::tile_assembler::{GroupModel_Liquid_Raw, GroupModel_Raw, WorldModel_Raw},
    tools::extractor_common::{casc_handles::CascStorageHandle, cstr_bytes_to_string, ChunkedFile},
    AzResult,
};

#[derive(Clone, Default, Debug)]
pub struct WmoMods {
    pub name:        [u8; 20],
    /// index of first doodad instance in this set
    pub start_index: u32,
    /// number of doodad instances in this set
    pub count:       u32,
    pub _pad:        [u8; 4],
}

#[derive(Clone, Default, Debug)]
pub struct WmoModd {
    // 24
    pub name_index: u32,
    pub position:   Vector3<f32>,
    pub rotation:   Vector4<f32>,
    pub scale:      f32,
    pub color:      u32,
}

#[derive(Clone, Default, Debug)]
pub struct WmoDoodadData {
    pub sets:       Vec<WmoMods>,
    pub paths:      HashMap<usize, String>,
    pub spawns:     Vec<WmoModd>,
    pub references: BTreeSet<u16>,
}

#[derive(Default)]
pub struct WmoRoot {
    pub color:               u32,
    pub n_textures:          u32,
    pub n_groups:            u32,
    pub n_portals:           u32,
    pub n_lights:            u32,
    pub n_doodad_names:      u32,
    pub n_doodad_defs:       u32,
    pub n_doodad_sets:       u32,
    pub root_wmoid:          u32,
    pub bbcorn1:             Vector3<f32>,
    pub bbcorn2:             Vector3<f32>,
    pub flags:               u16,
    pub num_lod:             u16,
    pub doodad_data:         WmoDoodadData,
    pub group_file_data_ids: Vec<u32>,

    // pub wmo_n_vertices:      usize,
    wmo_groups: Vec<WmoGroup>,
    filename:   String,
}

impl WmoRoot {
    pub fn build<P: AsRef<Path>>(storage: &CascStorageHandle, filename: P) -> AzResult<Self> {
        let cf = ChunkedFile::build(storage, &filename)?;
        let mut s = Self {
            filename: filename.as_ref().to_string_lossy().to_string(),
            ..Self::default()
        };
        for (fourcc, chunk) in cf.chunks {
            match &fourcc {
                b"MOHD" => {
                    let mut f = io::Cursor::new(&chunk.data);

                    s.n_textures = f.read_u32::<LittleEndian>()?; // , 4
                    s.n_groups = f.read_u32::<LittleEndian>()?; // , 4
                    s.n_portals = f.read_u32::<LittleEndian>()?; // , 4
                    s.n_lights = f.read_u32::<LittleEndian>()?; // , 4
                    s.n_doodad_names = f.read_u32::<LittleEndian>()?; // , 4
                    s.n_doodad_defs = f.read_u32::<LittleEndian>()?; // , 4
                    s.n_doodad_sets = f.read_u32::<LittleEndian>()?; // , 4
                    s.color = f.read_u32::<LittleEndian>()?; // , 4
                    s.root_wmoid = f.read_u32::<LittleEndian>()?; // , 4
                    s.bbcorn1 = Vector3::new(
                        f.read_f32::<LittleEndian>()?,
                        f.read_f32::<LittleEndian>()?,
                        f.read_f32::<LittleEndian>()?,
                    ); // , 12
                    s.bbcorn2 = Vector3::new(
                        f.read_f32::<LittleEndian>()?,
                        f.read_f32::<LittleEndian>()?,
                        f.read_f32::<LittleEndian>()?,
                    ); // , 12
                    s.flags = f.read_u16::<LittleEndian>()?; // , 2
                    s.num_lod = f.read_u16::<LittleEndian>()?; // , 2
                },
                b"MODS" => {
                    let mut f = io::Cursor::new(&chunk.data);
                    while !f.is_empty() {
                        let mut name = [0u8; 20];
                        f.read_exact(&mut name)?;
                        let start_index = f.read_u32::<LittleEndian>()?;
                        let count = f.read_u32::<LittleEndian>()?;
                        let mut _pad = [0u8; 4];
                        f.read_exact(&mut _pad)?;
                        s.doodad_data.sets.push(WmoMods {
                            name,
                            start_index,
                            count,
                            _pad,
                        });
                    }
                    // else if (!strcmp(fourcc, "MODS"))
                    // {
                    //     DoodadData.Sets.resize(size / sizeof(WMO::MODS));
                    //     f.read(DoodadData.Sets.data(), size);
                    // }
                    // else if (!strcmp(fourcc,"MODN"))
                    // {
                    //     char* ptr = f.getPointer();
                    //     char* end = ptr + size;
                    //     DoodadData.Paths = std::make_unique<char[]>(size);
                    //     memcpy(DoodadData.Paths.get(), ptr, size);
                    //     while (ptr < end)
                    //     {
                    //         std::string path = ptr;

                    //         char* s = GetPlainName(ptr);
                    //         FixNameCase(s, strlen(s));
                    //         FixNameSpaces(s, strlen(s));

                    //         uint32 doodadNameIndex = ptr - f.getPointer();
                    //         ptr += path.length() + 1;

                    //         if (ExtractSingleModel(path))
                    //             ValidDoodadNames.insert(doodadNameIndex);
                    //     }
                    // }
                    // else if (!strcmp(fourcc,"MODD"))
                    // {
                    //     DoodadData.Spawns.resize(size / sizeof(WMO::MODD));
                    //     f.read(DoodadData.Spawns.data(), size);
                    // }
                },
                b"MODN" => {
                    let mut offset = 0;

                    s.doodad_data.paths = chunk
                        .data
                        .split_inclusive(|b| *b == 0)
                        .map(|raw| {
                            // We dont anticipate a panic here as the strings will always be nul-terminated
                            let s = cstr_bytes_to_string(raw).unwrap();
                            offset += 1; // raw.len();
                            (offset, s)
                        })
                        .collect::<HashMap<_, _>>();
                },
                b"MODD" => {
                    let mut f = io::Cursor::new(&chunk.data);
                    while !f.is_empty() {
                        s.doodad_data.spawns.push(WmoModd {
                            name_index: f.read_u32::<LittleEndian>()?,
                            position:   Vector3::new(
                                f.read_f32::<LittleEndian>()?,
                                f.read_f32::<LittleEndian>()?,
                                f.read_f32::<LittleEndian>()?,
                            ),
                            rotation:   Vector4::new(
                                f.read_f32::<LittleEndian>()?,
                                f.read_f32::<LittleEndian>()?,
                                f.read_f32::<LittleEndian>()?,
                                f.read_f32::<LittleEndian>()?,
                            ),
                            scale:      f.read_f32::<LittleEndian>()?,
                            color:      f.read_u32::<LittleEndian>()?,
                        })
                    }
                },
                b"GFID" => {
                    let mut f: io::Cursor<&Vec<u8>> = io::Cursor::new(&chunk.data);
                    // full LOD reading code for reference
                    // commented out as we are not interested in any of them beyond first, most detailed

                    //uint16 lodCount = 1;
                    //if (flags & 0x10)
                    //{
                    //    if (numLod)
                    //        lodCount = numLod;
                    //    else
                    //        lodCount = 3;
                    //}

                    //for (uint32 lod = 0; lod < lodCount; ++lod)
                    //{
                    for _ in 0..s.n_groups {
                        let file_data_id = f.read_u32::<LittleEndian>()?;
                        if file_data_id > 0 {
                            s.group_file_data_ids.push(file_data_id);
                        }
                    }
                    //}
                },
                b"MOTX" => {},
                b"MOMT" => {},
                b"MOGN" => {},
                b"MOGI" => {},
                b"MOLT" => {},
                b"MOSB" => {},
                b"MOPV" => {},
                b"MOPT" => {},
                b"MOPR" => {},
                b"MFOG" => {},
                _ => {},
            }
        }

        Ok(s)
    }

    pub fn init_wmo_groups(&mut self, storage: &CascStorageHandle, valid_doodad_name_indices: HashSet<usize>) -> AzResult<()> {
        for group_file_data_id in self.group_file_data_ids.iter() {
            let s = format!("FILE{group_file_data_id:08X}.xxx");
            let fgroup = WmoGroup::build(self, storage, s).inspect_err(|e| {
                error!("could not open all group files for {}, err was {e}", self.filename);
            })?;
            for group_reference in fgroup.doodad_references.iter() {
                if *group_reference as usize >= self.doodad_data.spawns.len() {
                    continue;
                }
                let doodad_name_index = self.doodad_data.spawns[*group_reference as usize].name_index;
                if valid_doodad_name_indices.get(&(doodad_name_index as usize)).is_none() {
                    continue;
                }
                self.doodad_data.references.insert(*group_reference);
            }
            self.wmo_groups.push(fgroup);
        }
        Ok(())
    }

    pub fn convert_to_vmap(&self, precise_vector_data: bool) -> WorldModel_Raw {
        info!("Converting to vmap: {}", self.filename);
        let mut wmo_n_vertices = 0;
        let mut groups = Vec::with_capacity(self.wmo_groups.len());
        for grp in self.wmo_groups.iter() {
            let (n_verts, g) = grp.convert_to_vmap_group_model_raw(precise_vector_data);
            wmo_n_vertices += n_verts;
            groups.push(g);
        }
        WorldModel_Raw {
            n_vectors: wmo_n_vertices,
            root_wmo_id: self.root_wmoid,
            groups,
        }
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct WMOLiquidHeader {
    pub xverts:   i32,
    pub yverts:   i32,
    pub xtiles:   i32,
    pub ytiles:   i32,
    pub pos_x:    f32,
    pub pos_y:    f32,
    pub pos_z:    f32,
    pub material: i16,
}

impl WMOLiquidHeader {
    /// raw_size_of, is required due to alignment
    pub fn raw_size_of(&self) -> usize {
        use std::mem::size_of_val;
        size_of_val(&self.xverts)
            + size_of_val(&self.yverts)
            + size_of_val(&self.xtiles)
            + size_of_val(&self.ytiles)
            + size_of_val(&self.pos_x)
            + size_of_val(&self.pos_y)
            + size_of_val(&self.pos_z)
            + size_of_val(&self.material)
    }
}

#[allow(dead_code)]
#[derive(Default)]
struct WMOLiquidVert {
    unk1:   u16,
    unk2:   u16,
    height: f32,
}

#[allow(dead_code)]
struct WmoGroupMOBA {
    unknown_box:       [u8; 10],
    material_id_large: u16,
    /// first face index used in MOVI
    start_index_movi:  u32,
    /// number of MOVI indices used. (i.e. number of faces => nFaces)
    index_count_movi:  u16,
    /// first vertex index used in MOVT - minIndex
    start_index_movt:  u16,
    /// last vertex index used in MOVT. Batch includes this one - maxIndex
    end_index_movt:    u16,
    /// Flags. right now only 1 flag is used => flag_use_material_id_large = 1
    flags:             u8,
    /// index in momt
    material_id:       u8,
}

#[derive(Default)]
struct WmoGroup {
    mopy:            Vec<WmoGroupMOPY>,
    movi:            Vec<Vector3<u16>>,
    // movi_ex:         Vec<u16>,
    movt:            Vec<Vector3<f32>>,
    moba:            Vec<WmoGroupMOBA>,
    // moba_ex:         Vec<i32>,
    hlq:             WMOLiquidHeader,
    liqu_ex:         Vec<WMOLiquidVert>,
    liqu_bytes:      Vec<u8>,
    group_name:      u32,
    desc_group_name: u32,
    mogp_flags:      u32,
    bbcorn1:         Vector3<f32>,
    bbcorn2:         Vector3<f32>,
    mopr_idx:        u16,
    mopr_n_items:    u16,
    n_batch_a:       u16,
    n_batch_b:       u16,
    n_batch_c:       u32,
    fog_idx:         u32,
    group_liquid:    u32,
    group_wmoid:     u32,

    // n_triangles:       usize,
    // n_vertices:        usize,
    liquflags:         u32,
    doodad_references: Vec<u16>,
}

flags! {
    /// MOPY flags
    enum MopyFlags: u8
    {
        WmoMaterialUnk01            = 0x01,
        WmoMaterialNocamcollide     = 0x02,
        WmoMaterialDetail           = 0x04,
        WmoMaterialCollision        = 0x08,
        WmoMaterialHint             = 0x10,
        WmoMaterialRender           = 0x20,
        WmoMaterialWallSurface     = 0x40, // Guessed
        WmoMaterialCollideHit      = 0x80,
    }
}

#[allow(dead_code)]
struct WmoGroupMOPY {
    flag:        FlagSet<MopyFlags>,
    material_id: u8,
}

impl WmoGroup {
    fn build<P: AsRef<Path>>(root_wmo: &WmoRoot, storage: &CascStorageHandle, filename: P) -> AzResult<Self> {
        let f = ChunkedFile::build(storage, &filename)?;
        let mut s = Self::default();

        // MOGP should exist
        let mogp = f.chunks.get(b"MOGP").unwrap();
        let mut mogp_data = io::Cursor::new(&mogp.data);
        s.group_name = mogp_data.read_u32::<LittleEndian>()?; // , 4
        s.desc_group_name = mogp_data.read_u32::<LittleEndian>()?; // , 4
        s.mogp_flags = mogp_data.read_u32::<LittleEndian>()?; // , 4
        s.bbcorn1 = Vector3::new(
            mogp_data.read_f32::<LittleEndian>()?,
            mogp_data.read_f32::<LittleEndian>()?,
            mogp_data.read_f32::<LittleEndian>()?,
        ); // , 12
        s.bbcorn2 = Vector3::new(
            mogp_data.read_f32::<LittleEndian>()?,
            mogp_data.read_f32::<LittleEndian>()?,
            mogp_data.read_f32::<LittleEndian>()?,
        ); // , 12
        s.mopr_idx = mogp_data.read_u16::<LittleEndian>()?; // , 2
        s.mopr_n_items = mogp_data.read_u16::<LittleEndian>()?; // , 2
        s.n_batch_a = mogp_data.read_u16::<LittleEndian>()?; // , 2
        s.n_batch_b = mogp_data.read_u16::<LittleEndian>()?; // , 2
        s.n_batch_c = mogp_data.read_u32::<LittleEndian>()?; // , 4
        s.fog_idx = mogp_data.read_u32::<LittleEndian>()?; // , 4
        s.group_liquid = mogp_data.read_u32::<LittleEndian>()?; // , 4
        s.group_wmoid = mogp_data.read_u32::<LittleEndian>()?; // ,4

        // according to WoW.Dev Wiki => https://wowdev.wiki/WMO#MOGP_chunk
        // uint16_t flag_use_liquid_type_dbc_id : 1;
        // use real liquid type ID from DBCs instead of local one. See MLIQ for further reference.
        if (root_wmo.flags & 4) != 0 {
            s.group_liquid = get_liquid_type_id(s.group_liquid, s.mogp_flags);
        } else if s.group_liquid == 15 {
            s.group_liquid = 0;
        } else {
            s.group_liquid = get_liquid_type_id(s.group_liquid + 1, s.mogp_flags);
        }
        if s.group_liquid > 0 {
            s.liquflags |= 2;
        }
        let mut mopy_data_size = 0;
        let mut movt_data_size = 0;
        for (fourcc, subchunk) in mogp.sub_chunks.iter() {
            #[allow(clippy::if_same_then_else)]
            if fourcc == b"MOPY" {
                let mut data = io::Cursor::new(&subchunk.data);
                while !data.is_empty() {
                    s.mopy.push(WmoGroupMOPY {
                        flag:        FlagSet::new(data.read_u8()?)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: err was {}", e)))?,
                        material_id: data.read_u8()?,
                    });
                }
                // s.n_triangles = s.mopy.len();
                mopy_data_size = subchunk.size;
            } else if fourcc == b"MOVI" {
                let mut data = io::Cursor::new(&subchunk.data);
                while !data.is_empty() {
                    s.movi.push(Vector3::new(
                        data.read_u16::<LittleEndian>()?,
                        data.read_u16::<LittleEndian>()?,
                        data.read_u16::<LittleEndian>()?,
                    ));
                }
            } else if fourcc == b"MOVT" {
                let mut data = io::Cursor::new(&subchunk.data);
                while !data.is_empty() {
                    s.movt.push(Vector3::new(
                        data.read_f32::<LittleEndian>()?,
                        data.read_f32::<LittleEndian>()?,
                        data.read_f32::<LittleEndian>()?,
                    ));
                }
                // s.n_vertices = s.movt.len();
                movt_data_size = subchunk.size;
            } else if fourcc == b"MONR" {
                //
            } else if fourcc == b"MOTV" {
                //
            } else if fourcc == b"MOBA" {
                let mut data = io::Cursor::new(&subchunk.data);
                while !data.is_empty() {
                    let mut unknown_box = [0u8; 10];
                    data.read_exact(&mut unknown_box)?;
                    s.moba.push(WmoGroupMOBA {
                        unknown_box,
                        material_id_large: data.read_u16::<LittleEndian>()?,
                        start_index_movi: data.read_u32::<LittleEndian>()?,
                        index_count_movi: data.read_u16::<LittleEndian>()?,
                        start_index_movt: data.read_u16::<LittleEndian>()?,
                        end_index_movt: data.read_u16::<LittleEndian>()?,
                        flags: data.read_u8()?,
                        material_id: data.read_u8()?,
                    });
                }
            } else if fourcc == b"MODR" {
                let mut data = io::Cursor::new(&subchunk.data);
                while !data.is_empty() {
                    s.doodad_references.push(data.read_u16::<LittleEndian>()?);
                }
            } else if fourcc == b"MLIQ" {
                let mut data = io::Cursor::new(&subchunk.data);
                s.liquflags |= 1;
                s.hlq = WMOLiquidHeader {
                    xverts:   data.read_i32::<LittleEndian>()?,
                    yverts:   data.read_i32::<LittleEndian>()?,
                    xtiles:   data.read_i32::<LittleEndian>()?,
                    ytiles:   data.read_i32::<LittleEndian>()?,
                    pos_x:    data.read_f32::<LittleEndian>()?,
                    pos_y:    data.read_f32::<LittleEndian>()?,
                    pos_z:    data.read_f32::<LittleEndian>()?,
                    material: data.read_i16::<LittleEndian>()?,
                };
                // LiquEx_size = sizeof(WMOLiquidVert) * hlq->xverts * hlq->yverts;
                for _ in 0..(s.hlq.xverts * s.hlq.yverts) {
                    s.liqu_ex.push(WMOLiquidVert {
                        unk1:   data.read_u16::<LittleEndian>()?,
                        unk2:   data.read_u16::<LittleEndian>()?,
                        height: data.read_f32::<LittleEndian>()?,
                    })
                }
                s.liqu_bytes = vec![0; (s.hlq.xtiles * s.hlq.ytiles) as usize];
                data.read_exact(&mut s.liqu_bytes)?;
                // int nLiquBytes = hlq->xtiles * hlq->ytiles;
                // LiquBytes = new char[nLiquBytes];
                // f.read(LiquBytes, nLiquBytes);

                // Determine legacy liquid type
                if s.group_liquid > 0 {
                    for b in s.liqu_bytes.iter() {
                        if (*b & 0xF) != 15 {
                            let liquid_type_id = ((*b & 0xF) + 1).into();
                            s.group_liquid = get_liquid_type_id(liquid_type_id, s.mogp_flags);
                            break;
                        }
                    }
                }
                /* std::ofstream llog("Buildings/liquid.log", ios_base::out | ios_base::app);
                llog << filename;
                llog << "\nbbox: " << bbcorn1[0] << ", " << bbcorn1[1] << ", " << bbcorn1[2] << " | " << bbcorn2[0] << ", " << bbcorn2[1] << ", " << bbcorn2[2];
                llog << "\nlpos: " << hlq->pos_x << ", " << hlq->pos_y << ", " << hlq->pos_z;
                llog << "\nx-/yvert: " << hlq->xverts << "/" << hlq->yverts << " size: " << size << " expected size: " << 30 + hlq->xverts*hlq->yverts*8 + hlq->xtiles*hlq->ytiles << std::endl;
                llog.close(); */
            }
        }
        // SANITY CHECK
        if s.mopy.len() == s.movi.len() {
            Ok(s)
        } else if s.hlq.xverts != s.hlq.xtiles + 1 {
            Err(az_error!(
                "SANITY CHECK, xverts {} must be 1 more than xtiles {}",
                s.hlq.xverts,
                s.hlq.xtiles
            ))
        } else if s.hlq.yverts != s.hlq.ytiles + 1 {
            Err(az_error!(
                "SANITY CHECK, yverts {} must be 1 more than ytiles {}",
                s.hlq.yverts,
                s.hlq.ytiles
            ))
        } else {
            Err(az_error!(
                "SANITY CHECK FAILED: MOPY and MOVI should be the same, mopy_data_size={}, movt_data_size={}, s.mopy.len()={}, movi.len()={}",
                mopy_data_size,
                movt_data_size,
                s.mopy.len(),
                s.movi.len(),
            ))
        }
    }

    /// copies this wmo group info into GroupModel_Raw
    /// returns the calculated nVertices for this group
    fn convert_to_vmap_group_model_raw(&self, precise_vector_data: bool) -> (usize, GroupModel_Raw) {
        let mogp_flags = self.mogp_flags;
        let group_wmo_id = self.group_wmoid;
        // group bound
        let bbcorn1 = self.bbcorn1;
        let bbcorn2 = self.bbcorn2;
        let liquidflags = self.liquflags;
        let n_bounding_triangles = self.moba.iter().map(|m| m.index_count_movi).collect();
        #[allow(unused_assignments)]
        let mut mesh_triangle_indices = Vec::new();
        #[allow(unused_assignments)]
        let mut vertices_chunks = Vec::new();
        let mut liquid = None;
        let mut n_col_triangles = 0;
        let mut liquid_type = 0;
        if precise_vector_data {
            mesh_triangle_indices = self.movi.clone();
            vertices_chunks = self.movt.clone();
            n_col_triangles = self.mopy.len();
        } else {
            //-------INDX------------------------------------
            //-------MOPY--------
            let mut movi_ex = vec![0; self.movi.len() * 3]; // "worst case" size...
            let mut index_renum = vec![None; self.movt.len()];
            for (i, mopy) in self.mopy.iter().enumerate() {
                use MopyFlags::*;
                // Skip no collision triangles
                let is_render_face = (mopy.flag & WmoMaterialRender).bits() > 0 && (mopy.flag & WmoMaterialDetail).bits() == 0;
                let is_detail = (mopy.flag & WmoMaterialDetail).bits() > 0;
                let is_collision = (mopy.flag & WmoMaterialCollision).bits() > 0;
                if !is_render_face && !is_detail && !is_collision {
                    continue;
                }
                // Use this triangle
                index_renum[usize::from(self.movi[i].x)] = Some(None);
                index_renum[usize::from(self.movi[i].y)] = Some(None);
                index_renum[usize::from(self.movi[i].z)] = Some(None);
                movi_ex[3 * n_col_triangles] = self.movi[i].x;
                movi_ex[3 * n_col_triangles + 1] = self.movi[i].y;
                movi_ex[3 * n_col_triangles + 2] = self.movi[i].z;
                n_col_triangles += 1;
            }
            // assign new vertex index numbers
            let mut n_col_vertices = 0;
            for idx in index_renum.iter_mut().flatten() {
                // if (IndexRenum[i] == 1)
                // {
                //     IndexRenum[i] = nColVertices;
                //     ++nColVertices;
                // }
                *idx = Some(n_col_vertices);
                n_col_vertices += 1;
            }

            // translate triangle indices to new numbers
            for i in 0..3 * n_col_triangles {
                if usize::from(movi_ex[i]) >= index_renum.len() {
                    // ASSERT(movi_ex[i] < nVertices);
                    panic!(
                        "the original movi_ex[{i}] = {} should not be greater than {}",
                        movi_ex[i],
                        index_renum.len()
                    );
                }
                movi_ex[i] = if let Some(movi_idx) = index_renum[usize::from(movi_ex[i])].flatten() {
                    movi_idx
                } else {
                    0xff
                };
            }
            // write triangle indices
            for (i, m) in movi_ex.chunks_exact(3).enumerate() {
                if i < n_col_triangles {
                    // write up to n_col_triangles triangles
                    mesh_triangle_indices.push(Vector3::new(m[0], m[1], m[2]));
                }
            }
            // write vertices
            let mut check = 0;
            for (i, idx_re) in index_renum.into_iter().enumerate() {
                if idx_re.flatten().is_some() {
                    vertices_chunks.push(self.movt[i]);
                    check += 1;
                }
            }
            if n_col_vertices != check {
                // ASSERT(check==0);
                panic!("n_col_vertices is not equals to the checked movt amount. n_col_vertices was {n_col_vertices}, check was {check}");
            }
        }
        if (self.liquflags & 3) > 0 {
            liquid_type = self.group_liquid;
            if (self.liquflags & 1) > 0 {
                let liq = GroupModel_Liquid_Raw {
                    header:         self.hlq.clone(),
                    // only need height values, the other values are unknown anyway
                    liquid_heights: self.liqu_ex.iter().map(|l| l.height).collect(),
                    liquid_flags:   self.liqu_bytes.clone(),
                };
                liquid = Some(liq);
            }
        }
        let raw = GroupModel_Raw {
            mogp_flags,
            group_wmo_id,
            bbcorn1,
            bbcorn2,
            liquidflags,
            n_bounding_triangles,
            mesh_triangle_indices,
            vertices_chunks,
            liquid_type,
            liquid,
        };
        (n_col_triangles, raw)
    }
}

fn get_liquid_type_id(liquid_type_id: u32, mogp_flags: u32) -> u32 {
    if liquid_type_id <= 20 && liquid_type_id > 0 {
        match (liquid_type_id as u8 - 1) & 3 {
            0 => (if (mogp_flags & 0x80000) != 0 { 1 } else { 0 }) + 13,
            1 => 14,
            2 => 19,
            3 => 20,
            _ => liquid_type_id,
        }
    } else {
        liquid_type_id
    }
}
