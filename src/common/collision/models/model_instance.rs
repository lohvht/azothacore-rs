use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
};
use flagset::{flags, FlagSet};
use nalgebra::Vector3;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct VmapModelSpawn {
    pub map_num: u32,
    pub flags:   FlagSet<ModelFlags>,
    pub adt_id:  u16,
    pub id:      u32,
    pub i_pos:   Vector3<f32>,
    pub i_rot:   Vector3<f32>,
    pub i_scale: f32,
    pub bound:   Option<[Vector3<f32>; 2]>,
    pub name:    String,
    node_index:  usize,
}

#[allow(clippy::too_many_arguments)]
impl VmapModelSpawn {
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
            flags,
            adt_id,
            id,
            i_pos,
            i_rot,
            i_scale,
            bound,
            name,
            node_index: 0,
        }
    }
}

impl Bounded for VmapModelSpawn {
    fn aabb(&self) -> AABB {
        let b = self.bound.expect("bound for vmap spawn should have been set at this point, panicking");
        AABB::with_bounds(b[0].into(), b[1].into())
    }
}

impl BHShape for VmapModelSpawn {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

flags! {
    pub enum ModelFlags: u32 {
        ModM2           = 0b001,
        ModHasBound     = 0b010,
        ModParentSpawn  = 0b100,
    }
}
