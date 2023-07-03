use flagset::{flags, FlagSet};
use nalgebra::Vector3;

#[derive(Debug, Clone, Default)]
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
}

flags! {
    pub enum ModelFlags: u32 {
        ModM2           = 0b001,
        ModHasBound     = 0b010,
        ModParentSpawn  = 0b100,
    }
}
