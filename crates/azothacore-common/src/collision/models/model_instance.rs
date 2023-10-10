use std::sync::Arc;

use flagset::{flags, FlagSet};
use nalgebra::{Matrix3, Vector3};
use parry3d::bounding_volume::Aabb;

use crate::{collision::models::world_model::WorldModel, deref_boilerplate, g3dlite_copied::matrix3_from_euler_angles_zyx};

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

#[expect(clippy::too_many_arguments)]
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

deref_boilerplate!(VmapModelSpawnWithMapId, VmapModelSpawn, spawn);

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
    pub inv_rot:   Matrix3<f32>,
    pub model:     Arc<WorldModel>,
}

deref_boilerplate!(ModelInstance, VmapModelSpawn, spawn);

impl ModelInstance {
    pub fn new(spawn: VmapModelSpawn, model: Arc<WorldModel>) -> Self {
        let inv_rot = matrix3_from_euler_angles_zyx(spawn.i_rot.y.to_radians(), spawn.i_rot.x.to_radians(), spawn.i_rot.z.to_radians())
            .try_inverse()
            .unwrap();

        let inv_scale = 1.0 / spawn.i_scale;
        Self {
            spawn,
            inv_scale,
            inv_rot,
            model,
        }
    }
}
