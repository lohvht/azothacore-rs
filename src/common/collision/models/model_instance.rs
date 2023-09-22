use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use flagset::{flags, FlagSet};
use nalgebra::{Rotation, Rotation3, Vector3};
use parry3d::bounding_volume::Aabb;

use crate::common::collision::models::world_model::WorldModel;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct VmapModelSpawnWithMapId {
    pub map_num: u32,
    pub spawn:   VmapModelSpawn,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct VmapModelSpawn {
    pub flags:   FlagSet<ModelFlags>,
    pub adt_id:  u16,
    pub id:      u32,
    pub i_pos:   Vector3<f32>,
    pub i_rot:   Vector3<f32>,
    pub i_scale: f32,
    pub bound:   Option<Aabb>,
    pub name:    String,
}

#[allow(clippy::too_many_arguments)]
impl VmapModelSpawnWithMapId {
    pub fn new(
        map_num: u32,
        flags: FlagSet<ModelFlags>,
        adt_id: u16,
        id: u32,
        i_pos: Vector3<f32>,
        i_rot: Vector3<f32>,
        i_scale: f32,
        bound: Option<[Vector3<f32>; 2]>,
        name: String,
    ) -> Self {
        Self {
            map_num,
            spawn: VmapModelSpawn {
                flags,
                adt_id,
                id,
                i_pos,
                i_rot,
                i_scale,
                bound: bound.map(|[min, max]| Aabb::new(min.into(), max.into())),
                name,
            },
        }
    }
}

impl Deref for VmapModelSpawnWithMapId {
    type Target = VmapModelSpawn;

    fn deref(&self) -> &Self::Target {
        &self.spawn
    }
}

impl DerefMut for VmapModelSpawnWithMapId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.spawn
    }
}

flags! {
    pub enum ModelFlags: u32 {
        ModM2           = 0b001,
        ModParentSpawn  = 0b100,
    }
}

impl ModelFlags {
    pub fn flags_from_u32(value: u32) -> FlagSet<Self> {
        FlagSet::new_truncated(value)
    }
}

#[derive(Clone)]
pub struct ModelInstance {
    pub spawn:     VmapModelSpawn,
    pub inv_scale: f32,
    pub inv_rot:   Rotation3<f32>,
    pub model:     Arc<WorldModel>,
}

impl Deref for ModelInstance {
    type Target = VmapModelSpawn;

    fn deref(&self) -> &Self::Target {
        &self.spawn
    }
}

impl ModelInstance {
    pub fn new(spawn: VmapModelSpawn, model: Arc<WorldModel>) -> Self {
        let inv_rot = Rotation::from_euler_angles(spawn.i_rot.z.to_radians(), spawn.i_rot.x.to_radians(), spawn.i_rot.y.to_radians());
        // iInvRot = G3D::Matrix3::fromEulerAnglesZYX(G3D::pif()*iRot.y/180.f, G3D::pif()*iRot.x/180.f, G3D::pif()*iRot.z/180.f).inverse();

        let inv_scale = 1.0 / spawn.i_scale;
        Self {
            spawn,
            inv_scale,
            inv_rot,
            model,
        }
    }
}
