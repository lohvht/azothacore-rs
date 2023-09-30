use std::{
    collections::{BTreeMap, HashMap},
    io::{self, Read},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use flagset::{flags, FlagSet};
use nalgebra::{Vector3, Vector4};
use parry3d::bounding_volume::Aabb;
use tracing::{debug, error};

use crate::{
    az_error,
    common::collision::models::world_model::{WmoLiquid, WmoLiquidParams},
    tools::{
        extractor_common::{casc_handles::CascStorageHandle, chunked_data_offsets, cstr_bytes_to_string, ChunkedFile},
        vmap4_assembler::tile_assembler::{GroupModel_Raw, WorldModel_Raw},
    },
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
    // Bit flags - https://wowdev.wiki/WMO#MODD_chunk
    // first 4 bits are related to textures:
    // flag_AcceptProjTex, flag_0x2, flag_0x4, flag_0x8
    // Rest of the 4 bits are unused as of 7.0.1.20994
    pub flags:      u8,
    pub position:   Vector3<f32>,
    pub rotation:   Vector4<f32>,
    pub scale:      f32,
    pub color:      u32,
}

#[derive(Clone, Default, Debug)]
pub struct WmoDoodadData {
    pub sets:       Vec<WmoMods>,
    pub spawns:     Vec<WmoModd>,
    pub references: BTreeMap<u32, String>,
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
        let mut wmo_paths = HashMap::new();
        for (fourcc, chunk) in cf.chunks() {
            let chunk_data_len = u64::try_from(chunk.len()).unwrap();
            match fourcc {
                b"MOHD" => {
                    let mut f = io::Cursor::new(chunk);

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
                    let mut f = io::Cursor::new(chunk);
                    while f.position() < chunk_data_len {
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
                },
                b"MODN" => {
                    let mut offset = 0;
                    for raw in chunk.split_inclusive(|b| *b == 0) {
                        // We dont anticipate a panic here as the strings will always be nul-terminated
                        let p = cstr_bytes_to_string(raw).unwrap();
                        if p.len() >= 4 {
                            wmo_paths.insert(offset, p);
                        }
                        offset += raw.len();
                    }
                },
                b"MODD" => {
                    let mut f = io::Cursor::new(chunk);
                    while f.position() < chunk_data_len {
                        s.doodad_data.spawns.push(WmoModd {
                            name_index: f.read_u24::<LittleEndian>()?,
                            flags:      f.read_u8()?,
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
                    let mut f = io::Cursor::new(chunk);
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
        s.init_wmo_groups(storage, wmo_paths)?;

        Ok(s)
    }

    fn init_wmo_groups(&mut self, storage: &CascStorageHandle, wmo_paths: HashMap<usize, String>) -> AzResult<()> {
        for group_file_data_id in self.group_file_data_ids.iter() {
            let s = format!("FILE{group_file_data_id:08X}.xxx");
            let fgroup = WmoGroup::build(self, storage, s).map_err(|e| {
                error!("could not open all group files for {}, err was {e}", self.filename);
                e
            })?;
            for group_reference in fgroup.doodad_references.iter() {
                if *group_reference as usize >= self.doodad_data.spawns.len() {
                    continue;
                }
                let doodad = &self.doodad_data.spawns[*group_reference as usize];
                let path = match wmo_paths.get(&(doodad.name_index as usize)) {
                    None => {
                        debug!(
                            "doodad.name_index {} should exist in {:?} but it doesn't",
                            doodad.name_index, wmo_paths,
                        );
                        continue;
                    },
                    Some(s) => s,
                };
                self.doodad_data.references.insert(u32::from(*group_reference), path.clone());
            }
            self.wmo_groups.push(fgroup);
        }
        Ok(())
    }

    pub fn convert_to_vmap(&self, precise_vector_data: bool) -> WorldModel_Raw {
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

#[expect(dead_code)]
#[derive(Default, Debug)]
struct WMOLiquidVert {
    unk1:   u16,
    unk2:   u16,
    height: f32,
}

#[expect(dead_code)]
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
    liq_has_mogp_liq:  bool,
    liq_has_mliq:      bool,
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

#[expect(dead_code)]
struct WmoGroupMOPY {
    flag:        FlagSet<MopyFlags>,
    material_id: u8,
}

impl WmoGroup {
    fn build<P: AsRef<Path>>(root_wmo: &WmoRoot, storage: &CascStorageHandle, filename: P) -> AzResult<Self> {
        let f = ChunkedFile::build(storage, &filename)?;
        let mut s = Self::default();

        // MOGP should exist
        let mogp = f
            .chunks()
            .find_map(|(fcc, data)| if fcc == b"MOGP" { Some(data) } else { None })
            .unwrap();
        let mut mogp_data = io::Cursor::new(&mogp);
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
            s.liq_has_mogp_liq = true;
        }

        let mogp_subchunks = &mogp[mogp_data.position() as usize..];
        for (fourcc, start, end) in &chunked_data_offsets(mogp_subchunks)? {
            let subchunk_data = &mogp_subchunks[*start..*end];
            let subchunk_data_len = u64::try_from(subchunk_data.len()).unwrap();
            #[expect(clippy::if_same_then_else)]
            if fourcc == b"MOPY" {
                let mut data = io::Cursor::new(subchunk_data);
                while data.position() < subchunk_data_len {
                    s.mopy.push(WmoGroupMOPY {
                        flag:        FlagSet::new(data.read_u8()?)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("FLAGS INVALID?: err was {}", e)))?,
                        material_id: data.read_u8()?,
                    });
                }
                // s.n_triangles = s.mopy.len();
            } else if fourcc == b"MOVI" {
                let mut data = io::Cursor::new(subchunk_data);
                while data.position() < subchunk_data_len {
                    s.movi.push(Vector3::new(
                        data.read_u16::<LittleEndian>()?,
                        data.read_u16::<LittleEndian>()?,
                        data.read_u16::<LittleEndian>()?,
                    ));
                }
            } else if fourcc == b"MOVT" {
                let mut data = io::Cursor::new(subchunk_data);
                while data.position() < subchunk_data_len {
                    s.movt.push(Vector3::new(
                        data.read_f32::<LittleEndian>()?,
                        data.read_f32::<LittleEndian>()?,
                        data.read_f32::<LittleEndian>()?,
                    ));
                }
                // s.n_vertices = s.movt.len();
            } else if fourcc == b"MONR" {
                //
            } else if fourcc == b"MOTV" {
                //
            } else if fourcc == b"MOBA" {
                let mut data = io::Cursor::new(subchunk_data);
                while data.position() < subchunk_data_len {
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
                let mut data = io::Cursor::new(subchunk_data);
                while data.position() < subchunk_data_len {
                    s.doodad_references.push(data.read_u16::<LittleEndian>()?);
                }
            } else if fourcc == b"MLIQ" {
                let mut data = io::Cursor::new(subchunk_data);
                s.liq_has_mliq = true;
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
                if s.group_liquid == 0 {
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
                "SANITY CHECK FAILED: MOPY and MOVI should be the same, s.mopy.len()={}, movi.len()={}",
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
        let bbcorn = Aabb::new(self.bbcorn1.into(), self.bbcorn2.into());
        let n_bounding_triangles = self.moba.iter().map(|m| m.index_count_movi).collect::<Vec<_>>();
        #[expect(unused_assignments)]
        let mut mesh_triangle_indices = Vec::new();
        #[expect(unused_assignments)]
        let mut vertices_chunks = Vec::new();
        #[expect(unused_assignments)]
        let mut n_col_triangles = 0;
        if precise_vector_data {
            mesh_triangle_indices = self.movi.clone();
            vertices_chunks = self.movt.clone();
            n_col_triangles = self.mopy.len();
        } else {
            //-------INDX------------------------------------
            //-------MOPY--------
            let mut index_renum = vec![-1; self.movt.len()];
            let mut movi_ex = Vec::with_capacity(self.movi.len());
            for (i, mopy) in self.mopy.iter().enumerate() {
                use MopyFlags::*;
                // Skip no collision triangles
                let is_render_face = mopy.flag.contains(WmoMaterialRender) && !mopy.flag.contains(WmoMaterialDetail);
                let is_detail = mopy.flag.contains(WmoMaterialDetail);
                let is_collision = mopy.flag.contains(WmoMaterialCollision);
                if !is_render_face && !is_detail && !is_collision {
                    continue;
                }
                // Use this triangle
                // For now use a dummy number
                index_renum[self.movi[i].x as usize] = 1i32;
                index_renum[self.movi[i].y as usize] = 1i32;
                index_renum[self.movi[i].z as usize] = 1i32;
                movi_ex.push(self.movi[i]);
            }
            // assign new vertex index numbers
            let mut n_col_vertices = 0;
            for i in index_renum.iter_mut() {
                if *i == 1 {
                    *i = n_col_vertices;
                    n_col_vertices += 1;
                }
            }

            // translate triangle indices to new numbers
            for movi in movi_ex.iter_mut() {
                movi.x = index_renum[usize::try_from(movi.x).unwrap()] as u16;
                movi.y = index_renum[usize::try_from(movi.y).unwrap()] as u16;
                movi.z = index_renum[usize::try_from(movi.z).unwrap()] as u16;
            }
            let mut check = n_col_vertices;
            let mut vertices_to_use = Vec::with_capacity(self.movt.len());
            for idx in index_renum {
                if idx >= 0 {
                    vertices_to_use.push(self.movt[usize::try_from(idx).unwrap()]);
                    check -= 1;
                }
            }
            assert!(check == 0);
            mesh_triangle_indices = movi_ex;
            vertices_chunks = vertices_to_use;
            n_col_triangles = mesh_triangle_indices.len();
        }
        let liquid = if self.liq_has_mliq {
            Some(WmoLiquid::new(
                self.group_liquid,
                Ok(WmoLiquidParams {
                    // only need height values, the other values are unknown anyway
                    i_height: self.liqu_ex.iter().map(|l| l.height).collect(),
                    i_flags:  self.liqu_bytes.clone(),
                    width:    self.hlq.xtiles as usize,
                    height:   self.hlq.ytiles as usize,
                    corner:   Vector3::new(self.hlq.pos_x, self.hlq.pos_y, self.hlq.pos_z),
                }),
            ))
        } else if self.liq_has_mogp_liq {
            Some(WmoLiquid::new(self.group_liquid, Err(bbcorn.maxs.z)))
        } else {
            None
        };
        let raw = GroupModel_Raw {
            mogp_flags,
            group_wmo_id,
            bbcorn,
            n_bounding_triangles,
            mesh_triangle_indices,
            vertices_chunks,
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
