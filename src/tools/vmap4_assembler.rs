use tracing::{error, info};

use crate::{
    common::collision::{maps::tile_assembler::tile_assembler_convert_world2, models::model_instance::VmapModelSpawn},
    tools::{extractor_common::ExtractorConfig, vmap4_extractor::TempGameObjectModel},
    AzResult,
};

pub fn main_vmap4_assemble(
    args: &ExtractorConfig,
    model_spawns_data: Vec<VmapModelSpawn>,
    temp_gameobject_models: Vec<TempGameObjectModel>,
) -> AzResult<()> {
    tile_assembler_convert_world2(args, model_spawns_data, temp_gameobject_models).inspect_err(|e| {
        error!("TileAssembler exit with errors: {e}");
    })?;
    info!("Ok, all done");
    Ok(())
}
