use std::{io, path::Path};

use nalgebra::Vector3;
use parry3d::bounding_volume::Aabb;

use crate::{
    az_error,
    read_buf,
    read_le,
    tools::{
        extractor_common::{casc_handles::CascStorageHandle, ChunkedFile},
        vmap4_assembler::tile_assembler::{GroupModel_Raw, WorldModel_Raw},
    },
    AzResult,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Model {
    /// HEADER VALUES
    id: [u8; 4],
    version: [u8; 4],
    names: M2Array,               // uint32 nameLength; uint32 nameOfs;
    type_: u32,                   // type
    global_sequences: M2Array,    // nGlobalSequences, ofsGlobalSequences
    animations: M2Array,          // nAnimations, ofsAnimations
    animation_lookup: M2Array,    // nAnimationLookup, ofsAnimationLookup
    bones: M2Array,               // nBones, ofsBones
    key_bone_lookup: M2Array,     // nKeyBoneLookup, ofsKeyBoneLookup
    vertices: M2Array,            // nVertices, ofsVertices
    n_views: u32,                 // nViews
    colors: M2Array,              // nColors, ofsColors
    textures: M2Array,            // nTextures, ofsTextures
    transparency: M2Array,        // nTransparency, ofsTransparency
    textureanimations: M2Array,   // nTextureanimations, ofsTextureanimations
    tex_replace: M2Array,         // nTexReplace, ofsTexReplace
    render_flags: M2Array,        // nRenderFlags, ofsRenderFlags
    bone_lookup_table: M2Array,   // nBoneLookupTable, ofsBoneLookupTable
    tex_lookup: M2Array,          // nTexLookup, ofsTexLookup
    tex_units: M2Array,           // nTexUnits, ofsTexUnits
    trans_lookup: M2Array,        // nTransLookup, ofsTransLookup
    tex_anim_lookup: M2Array,     // nTexAnimLookup, ofsTexAnimLookup
    bounding_box: Aabb,           // boundingBox
    bounding_sphere_radius: f32,  // boundingSphereRadius
    collision_box: Aabb,          // collisionBox
    collision_sphere_radius: f32, // collisionSphereRadius
    bounding_triangles: M2Array,  // nBoundingTriangles, ofsBoundingTriangles
    bounding_vertices: M2Array,   // nBoundingVertices, ofsBoundingVertices
    bounding_normals: M2Array,    // nBoundingNormals, ofsBoundingNormals
    attachments: M2Array,         // nAttachments, ofsAttachments
    attach_lookup: M2Array,       // nAttachLookup, ofsAttachLookup
    attachments_2: M2Array,       // nAttachments_2, ofsAttachments_2
    lights: M2Array,              // nLights, ofsLights
    cameras: M2Array,             // nCameras, ofsCameras
    camera_lookup: M2Array,       // nCameraLookup, ofsCameraLookup
    ribbon_emitters: M2Array,     // nRibbonEmitters, ofsRibbonEmitters
    particle_emitters: M2Array,   // nParticleEmitters, ofsParticleEmitters

    /// Values
    val_vertices: Vec<Vector3<f32>>,
    val_indices:  Vec<u16>,
}
#[derive(Debug)]
struct M2Array {
    number:          u32,
    offset_elements: u32,
}

impl M2Array {
    fn read<R: io::Read>(input: &mut R) -> io::Result<Self> {
        Ok(Self {
            number:          read_le!(input, u32)?,
            offset_elements: read_le!(input, u32)?,
        })
    }
}

fn fix_coord_system(v: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(v.x, v.z, -v.y)
}

impl Model {
    pub fn build<P: AsRef<Path>>(storage: &CascStorageHandle, filename: P) -> AzResult<Self> {
        let f = ChunkedFile::build(storage, &filename)?;
        let md21 = f
            .chunks()
            .find_map(|(fcc, data)| if fcc == b"12DM" { Some(data) } else { None })
            .expect("MD21 chunk should exist in this version of WoW");
        let mut md20data = io::Cursor::new(md21);

        let id = read_buf!(md20data, 4)?;
        if &id != b"MD20" {
            return Err(az_error!(
                "SANITY CHECK: wrong magic number?, expect {}, got {}",
                String::from_utf8_lossy(b"MD20"),
                String::from_utf8_lossy(&id),
            ));
        }
        let version = read_buf!(md20data, 4)?;
        let names = M2Array::read(&mut md20data)?;
        let type_ = read_le!(md20data, u32)?;
        let global_sequences = M2Array::read(&mut md20data)?;
        let animations = M2Array::read(&mut md20data)?;
        let animation_lookup = M2Array::read(&mut md20data)?;
        let bones = M2Array::read(&mut md20data)?;
        let key_bone_lookup = M2Array::read(&mut md20data)?;
        let vertices = M2Array::read(&mut md20data)?;
        let n_views = read_le!(md20data, u32)?;
        let colors = M2Array::read(&mut md20data)?;
        let textures = M2Array::read(&mut md20data)?;
        let transparency = M2Array::read(&mut md20data)?;
        let textureanimations = M2Array::read(&mut md20data)?;
        let tex_replace = M2Array::read(&mut md20data)?;
        let render_flags = M2Array::read(&mut md20data)?;
        let bone_lookup_table = M2Array::read(&mut md20data)?;
        let tex_lookup = M2Array::read(&mut md20data)?;
        let tex_units = M2Array::read(&mut md20data)?;
        let trans_lookup = M2Array::read(&mut md20data)?;
        let tex_anim_lookup = M2Array::read(&mut md20data)?;

        let bounding_box = Aabb::new(
            Vector3::new(read_le!(md20data, f32)?, read_le!(md20data, f32)?, read_le!(md20data, f32)?).into(),
            Vector3::new(read_le!(md20data, f32)?, read_le!(md20data, f32)?, read_le!(md20data, f32)?).into(),
        );
        let bounding_sphere_radius = read_le!(md20data, f32)?;
        let collision_box = Aabb::new(
            Vector3::new(read_le!(md20data, f32)?, read_le!(md20data, f32)?, read_le!(md20data, f32)?).into(),
            Vector3::new(read_le!(md20data, f32)?, read_le!(md20data, f32)?, read_le!(md20data, f32)?).into(),
        );
        let collision_sphere_radius = read_le!(md20data, f32)?; // end 212
        let bounding_triangles = M2Array::read(&mut md20data)?;
        let bounding_vertices = M2Array::read(&mut md20data)?;
        let bounding_normals = M2Array::read(&mut md20data)?;
        let attachments = M2Array::read(&mut md20data)?;
        let attach_lookup = M2Array::read(&mut md20data)?;
        let attachments_2 = M2Array::read(&mut md20data)?;
        let lights = M2Array::read(&mut md20data)?;
        let cameras = M2Array::read(&mut md20data)?;
        let camera_lookup = M2Array::read(&mut md20data)?;
        let ribbon_emitters = M2Array::read(&mut md20data)?;
        let particle_emitters = M2Array::read(&mut md20data)?;

        if bounding_triangles.number == 0 {
            return Err(az_error!("Model {} has no bounding triangles", filename.as_ref().display()));
        }

        let mut val_vertices = Vec::with_capacity(bounding_vertices.number as usize);
        md20data.set_position(bounding_vertices.offset_elements.into());
        for _ in 0..bounding_vertices.number {
            val_vertices.push(fix_coord_system(Vector3::new(
                read_le!(md20data, f32)?,
                read_le!(md20data, f32)?,
                read_le!(md20data, f32)?,
            )));
        }

        let mut val_indices = Vec::with_capacity(bounding_triangles.number as usize);
        md20data.set_position(bounding_triangles.offset_elements.into());
        for _ in 0..bounding_triangles.number {
            val_indices.push(read_le!(md20data, u16)?);
        }
        Ok(Self {
            id,
            version,
            names,
            type_,
            global_sequences,
            animations,
            animation_lookup,
            bones,
            key_bone_lookup,
            vertices,
            n_views,
            colors,
            textures,
            transparency,
            textureanimations,
            tex_replace,
            render_flags,
            bone_lookup_table,
            tex_lookup,
            tex_units,
            trans_lookup,
            tex_anim_lookup,
            bounding_box,
            bounding_sphere_radius,
            collision_box,
            collision_sphere_radius,
            bounding_triangles,
            bounding_vertices,
            bounding_normals,
            attachments,
            attach_lookup,
            attachments_2,
            lights,
            cameras,
            camera_lookup,
            ribbon_emitters,
            particle_emitters,
            val_vertices,
            val_indices,
        })
    }

    pub fn convert_to_vmap(&self) -> WorldModel_Raw {
        // https://github.com/249CAAFE40/mangos-wotlk/commit/2f8b8e55d99122d34be2a08cbdbd2d2b1ae172d1
        //
        // Related to the above link, in case if it goes down heres the commit message explaining the change:
        //      ```
        //      # Fix vmap geometry.
        //      This patch fixes 2 issues with the vmap extractor. 1) Incorrectly converts vertex coordinates. 2) Forgets to convert coordinates on triangle indices.
        //
        //      The effects of [1] causes models to be flipped. This isn't very noticeable on most trees as flipping a cylinder results in the same cylinder, but it's very noticeable on any non-symmetrical geometry (which, even includes trees, it's just harder to notice). [2] Didn't seem to be a problem when the coordinates were incorrectly converted, but when applying the correct conversion caused some triangles to be flipped (i.e. facing inwards).
        //
        //      You will need to re-extract vmaps and re-generate mmaps for these changes to take effect.
        //      ```
        // Swaps the middle and last values.
        let mesh_triangle_indices = self.val_indices.chunks_exact(3).map(|r| Vector3::new(r[0], r[2], r[1])).collect();
        let vertices_chunks = self.val_vertices.iter().map(|r| Vector3::new(r.x, -r.z, r.y)).collect();
        WorldModel_Raw {
            n_vectors:   self.bounding_vertices.number as usize,
            root_wmo_id: 0,
            groups:      vec![GroupModel_Raw {
                mogp_flags: 0,
                group_wmo_id: 0,
                bbcorn: self.collision_box,
                n_bounding_triangles: vec![self.bounding_triangles.number as u16],
                mesh_triangle_indices, // INDX
                vertices_chunks,       // VERT
                liquid: None,
            }],
        }
    }
}
