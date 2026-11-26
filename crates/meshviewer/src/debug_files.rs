//! Readers for the files the mmap generator writes to `meshes/` with `debug_output = true`.
//! All positions are in Recast space: (WoW y, WoW z, WoW x).

use std::io;

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.buf.len());
        let Some(end) = end else {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("need {n} bytes at offset {}", self.pos)));
        };
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        Ok(self.take(N)?.try_into().unwrap())
    }

    fn count(&mut self) -> io::Result<usize> {
        let n = i32::from_ne_bytes(self.array()?);
        usize::try_from(n).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("negative count {n}")))
    }

    fn f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_ne_bytes(self.array()?))
    }

    fn vec3s(&mut self, n: usize) -> io::Result<Vec<[f32; 3]>> {
        (0..n).map(|_| Ok([self.f32()?, self.f32()?, self.f32()?])).collect()
    }

    fn u16s(&mut self, n: usize) -> io::Result<Vec<u16>> {
        Ok(self.take(n * 2)?.as_chunks::<2>().0.iter().map(|b| u16::from_ne_bytes(*b)).collect())
    }

    fn u32s(&mut self, n: usize) -> io::Result<Vec<u32>> {
        Ok(self.take(n * 4)?.as_chunks::<4>().0.iter().map(|b| u32::from_ne_bytes(*b)).collect())
    }
}

/// `.mesh`: the input geometry (terrain, liquid and models) Recast rasterised.
pub struct InputMesh {
    pub triangles: Vec<[[f32; 3]; 3]>,
}

/// TC writes vertex and triangle counts. Files from older AZC builds hold float and index counts
/// instead, so the layout is picked by which one matches the file size.
pub fn read_input_mesh(buf: &[u8]) -> io::Result<InputMesh> {
    let layout_fits = |per_count: usize| {
        let mut r = Reader::new(buf);
        let verts = r.count().ok()?;
        r.take(verts * per_count * 4).ok()?;
        let tris = r.count().ok()?;
        (r.pos + tris * per_count * 4 == buf.len()).then_some(())
    };
    let per_count = if layout_fits(3).is_some() {
        3
    } else if layout_fits(1).is_some() {
        1
    } else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "size matches neither .mesh layout"));
    };

    let mut r = Reader::new(buf);
    let floats = r.count()? * per_count;
    let verts = r.vec3s(floats / 3)?;
    let indices = r.count()? * per_count;
    let tris = r.u32s(indices)?;
    let triangles = tris
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| {
            let corner = |i: u32| {
                verts
                    .get(i as usize)
                    .copied()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("vertex {i} out of range")))
            };
            Ok([corner(t[0])?, corner(t[1])?, corner(t[2])?])
        })
        .collect::<io::Result<_>>()?;
    Ok(InputMesh { triangles })
}

/// `.pmesh`: Recast's `rcPolyMesh`, the polygons before Detour packs them into a tile.
pub struct PolyMesh {
    /// Each polygon's vertices and area id.
    pub polys: Vec<(Vec<[f32; 3]>, u8)>,
}

const RC_MESH_NULL_IDX: u16 = 0xffff;

pub fn read_poly_mesh(buf: &[u8]) -> io::Result<PolyMesh> {
    let mut r = Reader::new(buf);
    let cs = r.f32()?;
    let ch = r.f32()?;
    let nvp = r.count()?;
    let bmin = r.vec3s(1)?[0];
    let _bmax = r.vec3s(1)?;
    let nverts = r.count()?;
    let verts = r.u16s(nverts * 3)?;
    let npolys = r.count()?;
    let polys = r.u16s(npolys * nvp * 2)?;
    let _flags = r.u16s(npolys)?;
    let areas = r.take(npolys)?;

    // Same placement as Recast's duDebugDrawPolyMesh, which lifts every vertex by one cell height.
    let vertex = |i: u16| {
        let v = &verts[usize::from(i) * 3..][..3];
        [
            bmin[0] + f32::from(v[0]) * cs,
            bmin[1] + (f32::from(v[1]) + 1.0) * ch,
            bmin[2] + f32::from(v[2]) * cs,
        ]
    };
    let polys = polys
        .chunks_exact(nvp * 2)
        .zip(areas)
        .map(|(p, &area)| {
            let corners = p[..nvp].iter().take_while(|&&i| i != RC_MESH_NULL_IDX);
            let corners = corners
                .map(|&i| {
                    if usize::from(i) < nverts {
                        Ok(vertex(i))
                    } else {
                        Err(io::Error::new(io::ErrorKind::InvalidData, format!("vertex {i} out of range")))
                    }
                })
                .collect::<io::Result<_>>()?;
            Ok((corners, area))
        })
        .collect::<io::Result<_>>()?;
    Ok(PolyMesh { polys })
}

/// `.dmesh`: Recast's `rcPolyMeshDetail`, one triangulated sub-mesh per polygon.
pub struct DetailMesh {
    /// Triangles grouped per sub-mesh.
    pub submeshes: Vec<Vec<[[f32; 3]; 3]>>,
}

pub fn read_detail_mesh(buf: &[u8]) -> io::Result<DetailMesh> {
    let mut r = Reader::new(buf);
    let nverts = r.count()?;
    let verts = r.vec3s(nverts)?;
    let ntris = r.count()?;
    let tris = r.take(ntris * 4)?;
    let nmeshes = r.count()?;
    let meshes = r.u32s(nmeshes * 4)?;

    let submeshes = meshes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|&[vert_base, _vert_count, tri_base, tri_count]| {
            (tri_base..tri_base + tri_count)
                .map(|t| {
                    let t = tris
                        .get(t as usize * 4..t as usize * 4 + 3)
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "triangle out of range"))?;
                    let corner = |k: u8| {
                        verts
                            .get((vert_base + u32::from(k)) as usize)
                            .copied()
                            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "vertex out of range"))
                    };
                    Ok([corner(t[0])?, corner(t[1])?, corner(t[2])?])
                })
                .collect()
        })
        .collect::<io::Result<_>>()?;
    Ok(DetailMesh { submeshes })
}
