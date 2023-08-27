use std::{collections::BTreeMap, fs, io, path::Path};

use parry3d::bounding_volume::Aabb;

use crate::{
    cmp_or_return,
    common::collision::vmap_definitions::{GAMEOBJECT_MODELS, VMAP_MAGIC},
    sanity_check_read_all_bytes_from_reader,
    tools::extractor_common::{bincode_deserialise, bincode_serialise},
    AzResult,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GameObjectModelData {
    pub display_id: u32,
    pub is_wmo:     bool,
    pub name:       String,
    pub bounds:     Aabb,
}

impl GameObjectModelData {
    pub fn write_to_file<P: AsRef<Path>>(dir: P, model_list: &BTreeMap<u32, Self>) -> AzResult<()> {
        let mut model_list_copy = fs::File::create(dir.as_ref().join(GAMEOBJECT_MODELS))?;
        Self::write(&mut model_list_copy, model_list)
    }

    fn write<W: io::Write>(w: &mut W, model_list: &BTreeMap<u32, Self>) -> AzResult<()> {
        let mut w = w;

        w.write_all(VMAP_MAGIC)?;
        bincode_serialise(&mut w, &model_list)?;
        Ok(())
    }

    pub fn read<R: io::Read>(r: &mut R) -> AzResult<BTreeMap<u32, Self>> {
        let mut r = r;

        cmp_or_return!(r, VMAP_MAGIC)?;
        let res = bincode_deserialise(&mut r)?;

        sanity_check_read_all_bytes_from_reader!(r)?;

        Ok(res)
    }
}
