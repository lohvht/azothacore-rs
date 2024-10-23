use std::{collections::HashMap, fs, io, path::Path};

use azothacore_common::{
    bevy_app::bevy_app,
    collision::management::vmap_mgr2::{vmap_mgr2_plugin, VMapManager2InitSet, VmapConfig},
    configuration::{config_mgr_plugin, Config},
    utils::buffered_file_open,
    AzResult,
    ChildMapData,
    Locale,
    ParentMapData,
};
use azothacore_server::{
    game::map::MapLiquidTypeFlag,
    shared::data_stores::db2_structure::{LiquidType, Map},
};
use bevy::{app::Startup, prelude::IntoSystemSetConfigs};
use map_builder::{mmap_generator_plugin, LiquidTypes, MmapGenerationSets, VmapNotDisabled};
use tracing::{info, warn};
use wow_db2::wdc1;

use crate::extractor_common::{get_dir_contents, ExtractorConfig};
mod common;
mod intermediate_values;
mod map_builder;
mod terrain_builder;
mod tile_builder;

fn check_directories(args: &ExtractorConfig, first_installed_locale: Locale) -> io::Result<()> {
    let dbc = args.output_dbc_path(first_installed_locale);
    if get_dir_contents(&dbc, "*")?.next().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("'{}' directory is empty or does not exist", dbc.display()),
        ));
    }

    let maps = args.output_map_path();
    if get_dir_contents(&maps, "*")?.next().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("'{}' directory is empty or does not exist", maps.display()),
        ));
    }

    let vmaps = args.output_vmap_output_path();
    if get_dir_contents(&vmaps, "*")?.next().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("'{}' directory is empty or does not exist", vmaps.display()),
        ));
    }

    let mmaps = args.output_mmap_path();
    fs::create_dir_all(mmaps)?;

    if args.mmap_path_generator.debug_output {
        let meshes_debug = args.output_meshes_debug_path();
        fs::create_dir_all(meshes_debug)?;
    }

    Ok(())
}

impl Config for ExtractorConfig {}

impl VmapConfig for ExtractorConfig {
    fn enable_height_calc(&self) -> bool {
        true
    }

    fn enable_line_of_sight_calc(&self) -> bool {
        true
    }

    fn vmaps_dir(&self) -> std::path::PathBuf {
        self.output_vmap_output_path()
    }
}

pub fn main_path_generator<P: AsRef<Path>>(cfg_file: P, args: &ExtractorConfig, first_installed_locale: Locale) -> AzResult<()> {
    if args.mmap_path_generator.map_id_tile_x_y.is_none() && args.mmap_path_generator.debug_output {
        warn!("You have specifed debug output, but didn't specify a map to generate. This will generate debug output for ALL maps.");
    }
    check_directories(args, first_installed_locale)?;

    let liquid_source = buffered_file_open(args.output_dbc_path(first_installed_locale).join("LiquidType.db2"))?;
    let db2 = wdc1::FileLoader::<LiquidType>::from_reader(liquid_source, first_installed_locale)?;
    let liquid_types = db2.produce_data()?;

    let map_source = buffered_file_open(args.output_dbc_path(first_installed_locale).join("Map.db2"))?;
    let db2 = wdc1::FileLoader::<Map>::from_reader(map_source, first_installed_locale)?;
    let map_data = db2.produce_data()?;
    let mut map_id_to_child_map_ids = HashMap::new();
    let mut parent_map_data = HashMap::new();
    for m in map_data {
        map_id_to_child_map_ids.entry(m.id).or_insert(vec![]);
        if m.parent_map_id >= 0 {
            let parent_id = u32::try_from(m.parent_map_id).unwrap();
            let child_entries = map_id_to_child_map_ids.entry(parent_id).or_default();
            child_entries.push(m.id);
            parent_map_data.entry(m.id).or_insert(parent_id);
        }
    }

    let liquid_types = liquid_types
        .map(|t| (t.id, MapLiquidTypeFlag::from_liquid_type_sound_bank_unchecked(t.sound_bank)))
        .collect::<HashMap<_, _>>();

    let mut app = bevy_app();
    app.insert_resource(ChildMapData(map_id_to_child_map_ids))
        .insert_resource(ParentMapData(parent_map_data))
        .insert_resource(VmapNotDisabled)
        .insert_resource(LiquidTypes(liquid_types))
        .add_plugins((
            config_mgr_plugin::<ExtractorConfig, _>(cfg_file.as_ref().to_path_buf(), false),
            vmap_mgr2_plugin::<ExtractorConfig, LiquidTypes, VmapNotDisabled>,
            mmap_generator_plugin,
        ))
        .configure_sets(Startup, VMapManager2InitSet.before(MmapGenerationSets::DiscoverAndGenerateTilesToWork));

    let exit = app.run();
    info!("Generate Mmap done: {exit:?}");

    Ok(())
}
