use std::{collections::HashMap, fs, io, sync::Arc, time::Instant};

use tracing::{info, warn};
use wow_db2::wdc1;

use crate::{
    common::{collision::management::vmap_mgr2::VMapMgr2, Locale},
    server::{
        game::map::MapLiquidTypeFlag,
        shared::data_stores::db2_structure::{LiquidType, Map},
    },
    tools::{
        extractor_common::{get_dir_contents, ExtractorConfig, MapIdTileXY},
        mmap_generator::map_builder::MapBuilder,
    },
    AzResult,
};

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

pub fn main_path_generator(args: &ExtractorConfig, first_installed_locale: Locale) -> AzResult<()> {
    if args.mmap_path_generator.map_id_tile_x_y.is_none() && args.mmap_path_generator.debug_output {
        warn!("You have specifed debug output, but didn't specify a map to generate. This will generate debug output for ALL maps.");
    }
    check_directories(args, first_installed_locale)?;

    let liquid_source = fs::File::open(args.output_dbc_path(first_installed_locale).join("LiquidType.db2"))?;
    let db2 = wdc1::FileLoader::<LiquidType>::from_reader(liquid_source, first_installed_locale as u32)?;
    let liquid_types = db2.produce_data()?;

    let map_source = fs::File::open(args.output_dbc_path(first_installed_locale).join("Map.db2"))?;
    let db2 = wdc1::FileLoader::<Map>::from_reader(map_source, first_installed_locale as u32)?;
    let map_data = db2.produce_data()?;
    let mut map_id_to_child_map_ids = HashMap::new();
    for m in map_data {
        map_id_to_child_map_ids.entry(m.id).or_insert(vec![]);
        if m.parent_map_id >= 0 {
            let child_entries = map_id_to_child_map_ids.entry(m.parent_map_id.try_into().unwrap()).or_default();
            child_entries.push(m.id);
        }
    }

    let liquid_types = liquid_types
        .map(|t| (t.id, MapLiquidTypeFlag::from_liquid_type_sound_bank_unchecked(t.sound_bank)))
        .collect::<HashMap<_, _>>();

    let mut vmgr2 = VMapMgr2::default();
    vmgr2.set_map_data(&map_id_to_child_map_ids);
    vmgr2.set_callbacks(
        Arc::new(move |liq_id| {
            let ret = liquid_types
                .get(&liq_id)
                .map_or_else(|| None.into(), |liq_sound_bank| *liq_sound_bank);

            ret
        }),
        Arc::new(|_, _| false),
    );

    let builder = MapBuilder::build(args, vmgr2)?;

    let start = Instant::now();
    if let Some(file) = &args.mmap_path_generator.file {
        builder.build_mesh_from_file(file)?;
    } else if let Some(MapIdTileXY {
        map_id,
        tile_x_y: Some((tile_x, tile_y)),
    }) = args.mmap_path_generator.map_id_tile_x_y
    {
        builder.build_single_tile(map_id, tile_x, tile_y)?;
    } else if let Some(MapIdTileXY { map_id, .. }) = args.mmap_path_generator.map_id_tile_x_y {
        builder.build_maps(Some(map_id))?;
    } else {
        builder.build_maps(None)?;
    }
    let time_run = start.elapsed();

    info!("Finished. MMAPS were built in {}s", time_run.as_secs());

    Ok(())
}
