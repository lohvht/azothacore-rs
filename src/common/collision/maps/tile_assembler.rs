use std::{
    io,
    mem::{size_of, size_of_val},
};

use nalgebra::Vector3;

use crate::{cmp_or_return, common::collision::vmap_definitions::RAW_VMAP_MAGIC, read_le, tools::vmap4_extractor::wmo::WMOLiquidHeader};

#[allow(non_camel_case_types)]
pub struct WorldModel_Raw {
    pub n_vectors:   usize,
    pub root_wmo_id: u32,
    pub groups:      Vec<GroupModel_Raw>,
}

impl WorldModel_Raw {
    pub fn write<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        out.write_all(RAW_VMAP_MAGIC)?;
        out.write_all(&(self.n_vectors as u32).to_le_bytes())?;
        out.write_all(&(self.groups.len() as u32).to_le_bytes())?;
        out.write_all(&self.root_wmo_id.to_le_bytes())?;
        for g in self.groups.iter() {
            g.write(out)?;
        }
        Ok(())
    }

    pub fn read_world_model_raw_header<R: io::Read>(input: &mut R) -> io::Result<(usize, usize, u32)> {
        cmp_or_return!(input, RAW_VMAP_MAGIC)?;
        let n_vectors = read_le!(input, u32) as usize;
        let n_groups = read_le!(input, u32) as usize;
        let root_wmo_id = read_le!(input, u32);
        Ok((n_vectors, n_groups, root_wmo_id))
    }

    pub fn read<R: io::Read>(input: &mut R) -> io::Result<WorldModel_Raw> {
        let (n_vectors, n_groups, root_wmo_id) = Self::read_world_model_raw_header(input)?;
        let mut groups = Vec::with_capacity(n_groups);
        for _ in 0..n_groups {
            groups.push(GroupModel_Raw::read(input)?);
        }

        let mut buf_remain = vec![];
        input.read_to_end(&mut buf_remain)?;
        if !buf_remain.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "SANITY_CHECK: somehow file isn't fully consumed. please check again! {} bytes left",
                    buf_remain.len(),
                ),
            ));
        }

        Ok(Self {
            root_wmo_id,
            n_vectors,
            groups,
        })
    }
}

#[allow(non_camel_case_types)]
pub struct GroupModel_Raw {
    pub mogp_flags:            u32,
    pub group_wmo_id:          u32,
    pub bbcorn1:               Vector3<f32>,
    pub bbcorn2:               Vector3<f32>,
    pub liquidflags:           u32,
    /// Either from MOBA's MOVI indices count or from M2 collisionIndices size
    /// 1 group have at least 1 of these (group models can have >= 1 MOBAs)
    pub n_bounding_triangles:  Vec<u16>,
    /// either indices from MOVI or from M2 Model triangle indices
    pub mesh_triangle_indices: Vec<Vector3<u16>>,
    /// Either from MOVT or  in (X,Z,-Y) order
    pub vertices_chunks:       Vec<Vector3<f32>>,
    pub liquid_type:           u32,
    pub liquid:                Option<GroupModel_Liquid_Raw>,
}

macro_rules! vec_block_size {
    ( $collection:expr ) => {{
        if $collection.len() > 0 {
            $collection.len() * size_of_val(&$collection[0])
        } else {
            0
        }
    }};
}

macro_rules! vec_block_write {
    ( $out:expr, $collection:expr ) => {{
        for i in $collection.iter() {
            $out.write_all(&i.to_le_bytes())?;
        }
    }};
}

impl GroupModel_Raw {
    pub fn write<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        out.write_all(&self.mogp_flags.to_le_bytes())?;
        out.write_all(&self.group_wmo_id.to_le_bytes())?;
        vec_block_write!(out, self.bbcorn1);
        vec_block_write!(out, self.bbcorn2);
        out.write_all(&self.liquidflags.to_le_bytes())?;
        // GRP section
        out.write_all(b"GRP ")?;
        let block_size = size_of::<u32>() + vec_block_size!(self.n_bounding_triangles);
        out.write_all(&(block_size as u32).to_le_bytes())?;
        vec_block_write!(out, self.n_bounding_triangles);
        // INDX section
        out.write_all(b"INDX")?;
        let flat_triangle_indices = self.mesh_triangle_indices.iter().flat_map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>();
        let block_size = size_of::<u32>() + vec_block_size!(flat_triangle_indices);
        out.write_all(&(block_size as u32).to_le_bytes())?;
        vec_block_write!(out, flat_triangle_indices);
        // VERT section
        out.write_all(b"VERT")?;
        let flat_vert_chunks = self.vertices_chunks.iter().flat_map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>();
        let block_size = size_of::<u32>() + vec_block_size!(flat_vert_chunks);
        out.write_all(&(block_size as u32).to_le_bytes())?;
        vec_block_write!(out, flat_vert_chunks);
        // LIQU section
        if (self.liquidflags & 3) > 0 {
            let mut liqu_total_size = size_of::<u32>();
            if let Some(liq) = &self.liquid {
                liqu_total_size += liq.header.raw_size_of() + vec_block_size!(liq.liquid_heights) + vec_block_size!(liq.liquid_flags);
            }
            out.write_all(b"LIQU")?;
            out.write_all(&(liqu_total_size as u32).to_le_bytes())?;
            out.write_all(&self.liquid_type.to_le_bytes())?;
            if let Some(liq) = &self.liquid {
                out.write_all(&liq.header.xverts.to_le_bytes())?;
                out.write_all(&liq.header.yverts.to_le_bytes())?;
                out.write_all(&liq.header.xtiles.to_le_bytes())?;
                out.write_all(&liq.header.ytiles.to_le_bytes())?;
                out.write_all(&liq.header.pos_x.to_le_bytes())?;
                out.write_all(&liq.header.pos_y.to_le_bytes())?;
                out.write_all(&liq.header.pos_z.to_le_bytes())?;
                out.write_all(&liq.header.material.to_le_bytes())?;
                vec_block_write!(out, liq.liquid_heights);
                vec_block_write!(out, liq.liquid_flags);
            }
        }
        Ok(())
    }

    fn read<R: io::Read>(input: &mut R) -> io::Result<GroupModel_Raw> {
        let mogp_flags = read_le!(input, u32);
        let group_wmo_id = read_le!(input, u32);
        let bbcorn1 = Vector3::new(read_le!(input, f32), read_le!(input, f32), read_le!(input, f32));
        let bbcorn2 = Vector3::new(read_le!(input, f32), read_le!(input, f32), read_le!(input, f32));
        let liquidflags = read_le!(input, u32);

        // will this ever be used? what is it good for anyway??
        cmp_or_return!(input, b"GRP ")?;
        let _block_size = read_le!(input, u32);
        let branches = read_le!(input, u32);
        let mut n_bounding_triangles = Vec::with_capacity(branches as usize);
        for _ in 0..branches {
            n_bounding_triangles.push(read_le!(input, u16));
        }
        // ---- indexes
        cmp_or_return!(input, b"INDX")?;
        let _block_size = read_le!(input, u32);
        let mut nindexes = read_le!(input, u32);
        let mut mesh_triangle_indices = Vec::with_capacity((nindexes / 3) as usize);
        while nindexes > 0 {
            mesh_triangle_indices.push(Vector3::new(read_le!(input, u16), read_le!(input, u16), read_le!(input, u16)));
            nindexes -= 3;
        }
        // ---- vectors
        cmp_or_return!(input, b"VERT")?;
        let _block_size = read_le!(input, u32);
        let mut nvectors = read_le!(input, u32);
        let mut vertices_chunks = Vec::with_capacity((nvectors / 3) as usize);
        while nvectors > 0 {
            vertices_chunks.push(Vector3::new(read_le!(input, f32), read_le!(input, f32), read_le!(input, f32)));
            nvectors -= 3;
        }
        let mut liquid_type = 0;
        let mut liquid = None;
        if (liquidflags & 3) > 0 {
            cmp_or_return!(input, b"LIQU")?;
            let _block_size = read_le!(input, u32);
            liquid_type = read_le!(input, u32);
            if (liquidflags & 1) > 0 {
                let hlq = WMOLiquidHeader {
                    xverts:   read_le!(input, i32),
                    yverts:   read_le!(input, i32),
                    xtiles:   read_le!(input, i32),
                    ytiles:   read_le!(input, i32),
                    pos_x:    read_le!(input, f32),
                    pos_y:    read_le!(input, f32),
                    pos_z:    read_le!(input, f32),
                    material: read_le!(input, i16),
                };
                let size = (hlq.xverts * hlq.yverts) as usize;
                let mut liquid_heights = Vec::with_capacity(size);
                for _ in 0..size {
                    liquid_heights.push(read_le!(input, f32));
                }
                let size = (hlq.xtiles * hlq.ytiles) as usize;
                let mut liquid_flags = Vec::with_capacity(size);
                for _ in 0..size {
                    liquid_flags.push(read_le!(input, u8));
                }
                liquid = Some(GroupModel_Liquid_Raw {
                    header: hlq,
                    liquid_heights,
                    liquid_flags,
                })
            }
        }

        let mut buf_remain = vec![];
        input.read_to_end(&mut buf_remain)?;
        if !buf_remain.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "SANITY_CHECK: somehow file isn't fully consumed. please check again! {} bytes left",
                    buf_remain.len(),
                ),
            ));
        }

        Ok(Self {
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
        })
    }
}

#[allow(non_camel_case_types)]
pub struct GroupModel_Liquid_Raw {
    pub header:         WMOLiquidHeader,
    pub liquid_heights: Vec<f32>,
    pub liquid_flags:   Vec<u8>,
}
