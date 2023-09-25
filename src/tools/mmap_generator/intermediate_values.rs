use std::{io, io::Write, path::Path, slice};

use byteorder::{NativeEndian, WriteBytesExt};
use recastnavigation_sys::{rcCompactHeightfield, rcContourSet, rcHeightfield, rcPolyMesh, rcPolyMeshDetail};
use tracing::info;

use crate::{buffered_file_create, tools::mmap_generator::common::MeshData, AzResult};

// this class gathers all debug info holding and output
pub struct IntermediateValues<'a> {
    pub heightfield:         Option<&'a rcHeightfield>,
    pub compact_heightfield: Option<&'a rcCompactHeightfield>,
    pub contours:            Option<&'a rcContourSet>,
    pub poly_mesh:           Option<&'a rcPolyMesh>,
    pub poly_mesh_detail:    Option<&'a rcPolyMeshDetail>,
}

impl<'a> IntermediateValues<'a> {
    pub fn write_iv<P: AsRef<Path>>(&self, meshes_base_dir: P, map_id: u32, tile_x: u16, tile_y: u16) {
        macro_rules! debug_write {
            ( $path_tmpl:expr, $file_extension:expr, $v:expr, $debug_write_func_name:ident ) => {{
                use tracing::{error, info};

                let file_name = $path_tmpl.with_extension($file_extension);
                match buffered_file_create(&file_name) {
                    Err(e) => {
                        error!("Failed to open {} for writing! err {e}", file_name.display());
                    },
                    Ok(mut f) => {
                        info!("Writing debug output {}...", file_name.display());
                        if let Err(e) = $debug_write_func_name(&mut f, $v) {
                            error!("Failed to write debug info into {}; err {e}", file_name.display());
                        }
                    },
                };
            }};
        }

        let dest_filename_tmpl = meshes_base_dir.as_ref().join(format!("{map_id:04}{tile_x:02}{tile_y:02}.xxx"));

        // if let Some(v) = &self.heightfield {
        //     debug_write!(dest_filename_tmpl, "hf", v, debug_write_recast_heightfield);
        // }
        if let Some(v) = &self.compact_heightfield {
            debug_write!(dest_filename_tmpl, "chf", v, debug_write_recast_compact_heightfield);
        }
        if let Some(v) = &self.contours {
            debug_write!(dest_filename_tmpl, "cs", v, debug_write_recast_contour_set);
        }
        if let Some(v) = &self.poly_mesh {
            debug_write!(dest_filename_tmpl, "pmesh", v, debug_write_recast_poly_mesh);
        }
        if let Some(v) = &self.poly_mesh_detail {
            debug_write!(dest_filename_tmpl, "dmesh", v, debug_write_recast_poly_mesh_detail);
        }
    }

    pub fn generate_obj_file<P: AsRef<Path>>(&self, meshes_base_dir: P, map_id: u32, tile_x: u16, tile_y: u16, mesh_data: &MeshData) {
        let obj_file_name = meshes_base_dir.as_ref().join(format!("map{map_id:04}{tile_y:02}{tile_x:2}.obj"));
        info!("writing debug output to {}", obj_file_name.display());
        let mut obj_file = match buffered_file_create(&obj_file_name) {
            Err(e) => {
                tracing::error!("Failed to open for writing: {}; e {e}", obj_file_name.display());
                return;
            },
            Ok(f) => f,
        };

        let mut all_verts = Vec::with_capacity(mesh_data.liquid_verts.len() + mesh_data.solid_verts.len());
        let mut all_tris = Vec::with_capacity(mesh_data.liquid_tris.len() + mesh_data.solid_tris.len());

        for v in mesh_data.liquid_verts.iter() {
            all_verts.push(*v);
        }
        for t in mesh_data.liquid_tris.iter() {
            all_tris.push(*t);
        }
        let curr_verts_count = i32::try_from(all_verts.len() / 3).unwrap();
        for v in mesh_data.solid_verts.iter() {
            all_verts.push(*v);
        }
        for t in mesh_data.solid_tris.iter() {
            all_tris.push(*t + curr_verts_count);
        }

        for v in all_verts.chunks_exact(3) {
            _ = writeln!(obj_file, "v {:?} {:?} {:?}", v[0], v[1], v[2]);
        }
        for t in all_tris.chunks_exact(3) {
            _ = writeln!(obj_file, "f {:?} {:?} {:?}", t[0] + 1, t[1] + 1, t[2] + 1);
        }
        _ = obj_file.flush();

        info!("writing debug output");
        let obj_file_name = meshes_base_dir.as_ref().join(format!("map{map_id:04}.map"));
        let mut obj_file = match buffered_file_create(&obj_file_name) {
            Err(e) => {
                tracing::error!("Failed to open for writing: {}; e {e}", obj_file_name.display());
                return;
            },
            Ok(f) => f,
        };
        _ = obj_file.write_all(&[0]);

        let obj_file_name = meshes_base_dir.as_ref().join(format!("{map_id:04}{tile_y:02}{tile_x:2}.mesh"));
        let mut obj_file = match buffered_file_create(&obj_file_name) {
            Err(e) => {
                tracing::error!("Failed to open for writing: {}; e {e}", obj_file_name.display());
                return;
            },
            Ok(f) => f,
        };

        _ = obj_file.write_all(&(all_verts.len() as u32).to_ne_bytes());
        _ = obj_file.write_all(&all_verts.iter().flat_map(|v| v.to_ne_bytes()).collect::<Vec<_>>());
        _ = obj_file.flush();

        _ = obj_file.write_all(&(all_tris.len() as u32).to_ne_bytes());
        _ = obj_file.write_all(&all_tris.iter().flat_map(|v| v.to_ne_bytes()).collect::<Vec<_>>());
        _ = obj_file.flush();
    }
}

macro_rules! buf_slice_from_ptr {
    ( $ptr:expr, $ptr_len:expr ) => {{
        #[allow(unused_unsafe)]
        &unsafe { slice::from_raw_parts($ptr, $ptr_len as usize) }
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect::<Vec<_>>()
    }};
}

macro_rules! buf_slice_from_slice {
    ( $slice:expr ) => {{
        &$slice.iter().flat_map(|v| v.to_ne_bytes()).collect::<Vec<_>>()
    }};
}

// fn debug_write_recast_heightfield<W: io::Write>(file: &mut W, hf: &RecastHeightField) -> AzResult<()> {
// file.write_all(&hf.cs.to_ne_bytes())?;
// file.write_all(&hf.ch.to_ne_bytes())?;
// file.write_all(&hf.width.to_ne_bytes())?;
// file.write_all(&hf.height.to_ne_bytes())?;
// file.write_all(buf_slice_from_slice!(hf.bmin))?;
// file.write_all(buf_slice_from_slice!(hf.bmax))?;

// for y in 0..hf.height as usize {
//     for x in 0..hf.width as usize {
//         let mut span = unsafe { *hf.spans.wrapping_add(x + y * hf.width as usize) };

//         // first, count the number of spans
//         let mut span_count: i32 = 0;
//         while !span.is_null() {
//             span_count += 1;
//             unsafe {
//                 span = (*span).next;
//             }
//         }

//         // write the span count
//         file.write_all(&span_count.to_ne_bytes())?;

//         // write the spans
//         let mut span = unsafe { *hf.spans.wrapping_add(x + y * hf.width as usize) };
//         while !span.is_null() {
//             unsafe {
//                 // smin - originally bitpacked as u16 (i.e. RC_SPAN_HEIGHT_BITS=16)
//                 file.write_all(&((*span).smin() as u16).to_ne_bytes())?;
//                 // smax - originally bitpacked as u16 (i.e. RC_SPAN_HEIGHT_BITS=16)
//                 file.write_all(&((*span).smax() as u16).to_ne_bytes())?;
//                 file.write_all(&(*span).area().to_ne_bytes())?;

//                 (*span)._bitfield_1.

//                 // unsigned int smin : RC_SPAN_HEIGHT_BITS; ///< The lower limit of the span. [Limit: < #smax]
//                 // unsigned int smax : RC_SPAN_HEIGHT_BITS; ///< The upper limit of the span. [Limit: <= #RC_SPAN_MAX_HEIGHT]
//                 // unsigned char area;                      ///< The area id assigned to the span.
//                 // rcSpan* next;                            ///< The next span higher up in column.

//                 span = (*span).next;
//             }
//         }
//     }
// }
// }

fn debug_write_recast_compact_heightfield<W: io::Write>(file: &mut W, chf: &rcCompactHeightfield) -> AzResult<()> {
    file.write_all(&chf.width.to_ne_bytes())?;
    file.write_all(&chf.height.to_ne_bytes())?;
    file.write_all(&chf.spanCount.to_ne_bytes())?;

    file.write_all(&chf.walkableHeight.to_ne_bytes())?;
    file.write_all(&chf.walkableClimb.to_ne_bytes())?;

    file.write_all(&chf.maxDistance.to_ne_bytes())?;
    file.write_all(&chf.maxRegions.to_ne_bytes())?;

    file.write_all(buf_slice_from_slice!(chf.bmin))?;
    file.write_all(buf_slice_from_slice!(chf.bmax))?;

    file.write_all(&chf.cs.to_ne_bytes())?;
    file.write_all(&chf.ch.to_ne_bytes())?;

    let mut tmp: i32 = 0;
    if !chf.cells.is_null() {
        tmp |= 1;
    }
    if !chf.spans.is_null() {
        tmp |= 2;
    }
    if !chf.dist.is_null() {
        tmp |= 4;
    }
    if !chf.areas.is_null() {
        tmp |= 8;
    }

    file.write_all(&tmp.to_ne_bytes())?;

    if !chf.cells.is_null() {
        // fwrite(chf.cells, sizeof(rcCompactCell), chf.width * chf.height, file);
        unsafe {
            // index is originally bitpacked as u24 (from rcCompactCell)
            file.write_u24::<NativeEndian>((*chf.cells).index())?;
            // count is originally bitpacked as u8 (from rcCompactCell)
            file.write_all(&((*chf.cells).count() as u8).to_ne_bytes())?;
        }
    }
    if !chf.spans.is_null() {
        // fwrite(chf.spans, sizeof(rcCompactSpan), chf.spanCount, file);
        for i in 0..chf.spanCount as usize {
            let chf_span = chf.spans.wrapping_add(i);
            unsafe {
                file.write_all(&(*chf_span).y.to_ne_bytes())?;
                file.write_all(&(*chf_span).reg.to_ne_bytes())?;
                // con is originally bitpacked, as u24 (from rcCompactSpan)
                file.write_u24::<NativeEndian>((*chf_span).con())?;
                // h is originally  bitpacked, as u8 (from rcCompactSpan)
                file.write_all(&((*chf_span).h() as u8).to_ne_bytes())?;
            };
        }
    }
    if !chf.dist.is_null() {
        file.write_all(buf_slice_from_ptr!(chf.dist, chf.spanCount))?;
    }
    if !chf.areas.is_null() {
        file.write_all(buf_slice_from_ptr!(chf.areas, chf.spanCount))?;
    }

    Ok(())
}

fn debug_write_recast_contour_set<W: io::Write>(file: &mut W, cs: &rcContourSet) -> AzResult<()> {
    file.write_all(&cs.cs.to_ne_bytes())?;
    file.write_all(&cs.ch.to_ne_bytes())?;
    file.write_all(buf_slice_from_slice!(cs.bmin))?;
    file.write_all(buf_slice_from_slice!(cs.bmax))?;
    file.write_all(&cs.nconts.to_ne_bytes())?;

    for i in 0..cs.nconts as usize {
        let cs_cont = cs.conts.wrapping_add(i);
        unsafe {
            file.write_all(&(*cs_cont).area.to_ne_bytes())?;
            file.write_all(&(*cs_cont).reg.to_ne_bytes())?;
            file.write_all(&(*cs_cont).nverts.to_ne_bytes())?;
            file.write_all(buf_slice_from_ptr!((*cs_cont).verts, (*cs_cont).nverts * 4))?;
            file.write_all(&(*cs_cont).nrverts.to_ne_bytes())?;
            file.write_all(buf_slice_from_ptr!((*cs_cont).rverts, (*cs_cont).nrverts * 4))?;
        };
    }
    Ok(())
}

fn debug_write_recast_poly_mesh<W: io::Write>(file: &mut W, mesh: &rcPolyMesh) -> AzResult<()> {
    file.write_all(&mesh.cs.to_ne_bytes())?;
    file.write_all(&mesh.ch.to_ne_bytes())?;
    file.write_all(&mesh.nvp.to_ne_bytes())?;
    file.write_all(buf_slice_from_slice!(mesh.bmin))?;
    file.write_all(buf_slice_from_slice!(mesh.bmax))?;
    file.write_all(&mesh.nverts.to_ne_bytes())?;
    file.write_all(buf_slice_from_ptr!(mesh.verts, mesh.nverts * 3))?;
    file.write_all(&mesh.npolys.to_ne_bytes())?;
    file.write_all(buf_slice_from_ptr!(mesh.polys, mesh.npolys * mesh.nvp * 2))?;
    file.write_all(buf_slice_from_ptr!(mesh.flags, mesh.npolys))?;
    file.write_all(buf_slice_from_ptr!(mesh.areas, mesh.npolys))?;
    file.write_all(buf_slice_from_ptr!(mesh.regs, mesh.npolys))?;
    Ok(())
}

fn debug_write_recast_poly_mesh_detail<W: io::Write>(file: &mut W, mesh: &rcPolyMeshDetail) -> AzResult<()> {
    file.write_all(&mesh.nverts.to_ne_bytes())?;
    file.write_all(buf_slice_from_ptr!(mesh.verts, mesh.nverts * 3))?;
    file.write_all(&mesh.ntris.to_ne_bytes())?;
    file.write_all(buf_slice_from_ptr!(mesh.tris, mesh.ntris * 4))?;
    file.write_all(&mesh.nmeshes.to_ne_bytes())?;
    file.write_all(buf_slice_from_ptr!(mesh.meshes, mesh.nmeshes * 4))?;
    Ok(())
}
