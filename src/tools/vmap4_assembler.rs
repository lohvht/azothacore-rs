use std::{collections::BTreeMap, fs};

use tracing::{error, info};

use crate::{
    common::collision::{maps::tile_assembler::tile_assembler_convert_world2, models::model_instance::VmapModelSpawn},
    tools::{extractor_common::ExtractorConfig, vmap4_extractor::TempGameObjectModel},
    GenericResult,
};

pub fn main_vmap4_assemble(
    args: &ExtractorConfig,
    model_spawns_data: BTreeMap<u32, BTreeMap<u32, VmapModelSpawn>>,
    temp_gameobject_models: Vec<TempGameObjectModel>,
) -> GenericResult<()> {
    let src = args.output_vmap_sz_work_dir_wmo();
    let dst = args.output_vmap_output_path();

    let src_display = src.display();
    let dst_display = dst.display();
    info!("using {src_display} as source directory and writing output to {dst_display}");

    fs::create_dir_all(&dst)?;

    tile_assembler_convert_world2(dst, src, model_spawns_data, temp_gameobject_models).inspect_err(|e| {
        error!("TileAssembler exit with errors: {e}");
    })?;
    info!("Ok, all done");
    Ok(())
}
