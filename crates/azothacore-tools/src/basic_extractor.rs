mod wow7_3_5_26972;

use std::{
    collections::BTreeMap,
    fs,
    io,
    path::{Path, PathBuf},
};

use flagset::FlagSet;
use nalgebra::{DMatrix, SMatrix};
use tracing::{error, info, warn};
pub use wow7_3_5_26972::*;
use wow_db2::wdc1;

// Bunch of map stuff
/// Max accuracy = val/256
const V9V8_HEIGHT_FLOAT_TO_INT8_LIMIT: f32 = 2.0;
/// Max accuracy = val/65536
const V9V8_HEIGHT_FLOAT_TO_INT16_LIMIT: f32 = 2048.0;
/// If max - min less this value - surface is flat
const FLAT_HEIGHT_DELTA_LIMIT: f32 = 0.005;
/// If max - min less this value - liquid surface is flat
const FLAT_LIQUID_DELTA_LIMIT: f32 = 0.001;

use azothacore_common::{az_error, utils::buffered_file_create, AzResult, Locale};
use azothacore_server::{
    game::{
        grid::grid_defines::{ADT_CELLS_PER_GRID, ADT_CELL_SIZE, ADT_GRID_SIZE, ADT_GRID_SIZE_PLUS_ONE},
        map::{
            map_file::{MapFile, MapFilev9v8, MapHeightData, MapHeightFlightBox},
            GridMap,
            MapLiquidData,
            MapLiquidDataEntryFlags,
            MapLiquidDataGlobalEntryFlags,
            MapLiquidTypeFlag,
        },
    },
    shared::data_stores::db2_structure::{CinematicCamera, LiquidMaterial, LiquidType, Map as MapDb2},
};

use crate::{
    adt::{AdtChunkMcnk, AdtChunkMfbo, AdtChunkMh2o, ADT_LIQUID_TYPE_MAGMA, ADT_LIQUID_TYPE_OCEAN, ADT_LIQUID_TYPE_SLIME, ADT_LIQUID_TYPE_WATER},
    extractor_common::{
        casc_handles::{CascFileHandle, CascLocale, CascStorageHandle},
        ChunkedFile,
        DB2AndMapExtractFlags,
        ExtractorConfig,
    },
    to_casc_locales,
    wdt::{WdtChunkMain, WDT_MAP_SIZE},
};

pub fn main_db2_and_map_extract(args: &ExtractorConfig, first_installed_locale: Locale, build: u32) -> AzResult<()> {
    let installed_locales_mask = args.get_installed_locales_mask()?;

    for l in args.locales.into_iter() {
        if let Locale::none = l {
            continue;
        }

        if (installed_locales_mask & to_casc_locales(&l)).bits() == 0 {
            info!(
                "Locale {l:?} ({:?}) is not part of the installed locales {installed_locales_mask:?}",
                to_casc_locales(&l)
            );
            continue;
        }
        let storage = match args.get_casc_storage_handler(l) {
            Err(e) => {
                error!(
                    "error opening casc storage '{}' locale {}, err was {}",
                    args.input_storage_data_dir().display(),
                    l,
                    e,
                );
                continue;
            },
            Ok(r) => r,
        };

        if !args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::Dbc) {
            info!("Detected client build: {}", build);
            break;
        }
        // Extract DBC files
        info!("Detected client build: {} for locale {}", build, l);
        extract_db_files_client(&storage, args, l)?;
    }

    if args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::Camera) {
        extract_camera_files(args, first_installed_locale)?;
    }
    if args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::GameTables) {
        extract_game_tables(args, first_installed_locale)?;
    }
    if args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::Map) {
        extract_maps(args, first_installed_locale, build)?;
    }
    Ok(())
}

fn get_casc_filename_part<P: AsRef<Path>>(casc_path: P) -> PathBuf {
    if let Some(last_sep) = casc_path.as_ref().file_name() {
        Path::new(last_sep).to_path_buf()
    } else {
        casc_path.as_ref().to_path_buf()
    }
}

fn extract_db_files_client(storage: &CascStorageHandle, args: &ExtractorConfig, locale: Locale) -> AzResult<()> {
    info!("Extracting dbc/db2 files for {}...", locale);
    let locale_path = args.output_dbc_path(locale);

    fs::create_dir_all(&locale_path)?;

    info!("locale {} output path {}", locale, locale_path.display());
    let mut count = 0;

    for file_name in DB_FILES_CLIENT_LIST {
        let mut dbc_file = match storage.open_file(file_name, FlagSet::from(CascLocale::None)) {
            Err(e) => {
                warn!("Unable to open file {} in the archive for locale {}: {}", file_name, locale, e);
                continue;
            },
            Ok(r) => r,
        };
        let file_path = locale_path.join(get_casc_filename_part(file_name));
        if file_path.exists() {
            continue;
        }
        if extract_file(&mut dbc_file, file_path).is_err() {
            continue;
        }
        count += 1;
    }
    info!("Extracted {} files!", count);
    Ok(())
}

fn extract_file(file_in_archive: &mut CascFileHandle, out_path: PathBuf) -> AzResult<()> {
    let file_size = file_in_archive.get_file_size()?;

    let mut output = buffered_file_create(&out_path).map_err(|e| {
        error!("can't create the output file '{}', err was: {}", out_path.display(), e);
        e
    })?;

    let res = io::copy(file_in_archive, &mut output)?;

    // Sanity check here! just verifying that file_size detected is the same as the result
    if file_size != res {
        let e = az_error!(
            "Extracted file sizes don't match somehow for {}. expected {}, got {}",
            out_path.display(),
            file_size,
            res
        );
        error!("extract_file has failed somehow: {e}");
        Err(e)
    } else {
        Ok(())
    }
}

fn extract_camera_files(args: &ExtractorConfig, locale: Locale) -> AzResult<()> {
    info!("Extracting camera files...");

    let storage = args.get_casc_storage_handler(locale)?;
    let camera_file_names = read_cinematic_camera_dbc(&storage, locale)?;

    let output_path = args.output_camera_path();

    fs::create_dir_all(&output_path)?;

    info!("output camera path is {}", output_path.display());

    // extract M2s
    let mut count = 0;
    for camera_file_name in camera_file_names {
        let mut dbc_file = storage.open_file(&camera_file_name, CascLocale::None.into())?;
        let file_path = output_path.join(get_casc_filename_part(&camera_file_name));
        if file_path.exists() {
            continue;
        }
        if extract_file(&mut dbc_file, file_path).is_err() {
            continue;
        }
        count += 1;
    }
    info!("Extracted {count} camera files");

    Ok(())
}

fn read_cinematic_camera_dbc(storage: &CascStorageHandle, locale: Locale) -> AzResult<Vec<String>> {
    info!("Read CinematicCamera.db2 file...");
    let source = storage.open_file("DBFilesClient/CinematicCamera.db2", CascLocale::None.into())?;
    let fl = wdc1::FileLoader::<CinematicCamera>::from_reader(source, locale)?;
    let data = fl.produce_data()?;

    let res = data
        .map(|d| {
            let fid = d.file_data_id;
            format!("FILE{fid:08X}.xxx")
        })
        .collect::<Vec<_>>();

    info!("Done! ({} CinematicCameras loaded)", res.len());
    Ok(res)
}

fn extract_game_tables(args: &ExtractorConfig, locale: Locale) -> AzResult<()> {
    info!("Extracting game tables...");
    let storage = args.get_casc_storage_handler(locale)?;
    let output_path = args.output_gametable_path();

    fs::create_dir_all(&output_path)?;

    info!("output game table path is {}", output_path.display());

    let game_tables = [
        "GameTables/ArmorMitigationByLvl.txt",
        "GameTables/ArtifactKnowledgeMultiplier.txt",
        "GameTables/ArtifactLevelXP.txt",
        "GameTables/BarberShopCostBase.txt",
        "GameTables/BaseMp.txt",
        "GameTables/BattlePetTypeDamageMod.txt",
        "GameTables/BattlePetXP.txt",
        "GameTables/ChallengeModeDamage.txt",
        "GameTables/ChallengeModeHealth.txt",
        "GameTables/CombatRatings.txt",
        "GameTables/CombatRatingsMultByILvl.txt",
        "GameTables/HonorLevel.txt",
        "GameTables/HpPerSta.txt",
        "GameTables/ItemSocketCostPerLevel.txt",
        "GameTables/NpcDamageByClass.txt",
        "GameTables/NpcDamageByClassExp1.txt",
        "GameTables/NpcDamageByClassExp2.txt",
        "GameTables/NpcDamageByClassExp3.txt",
        "GameTables/NpcDamageByClassExp4.txt",
        "GameTables/NpcDamageByClassExp5.txt",
        "GameTables/NpcDamageByClassExp6.txt",
        "GameTables/NPCManaCostScaler.txt",
        "GameTables/NpcTotalHp.txt",
        "GameTables/NpcTotalHpExp1.txt",
        "GameTables/NpcTotalHpExp2.txt",
        "GameTables/NpcTotalHpExp3.txt",
        "GameTables/NpcTotalHpExp4.txt",
        "GameTables/NpcTotalHpExp5.txt",
        "GameTables/NpcTotalHpExp6.txt",
        "GameTables/SandboxScaling.txt",
        "GameTables/SpellScaling.txt",
        "GameTables/xp.txt",
    ];

    let mut count = 0;
    for file_name in game_tables {
        let mut dbc_file = storage.open_file(file_name, CascLocale::None.into())?;
        let file_path = output_path.join(get_casc_filename_part(file_name));
        if file_path.exists() {
            continue;
        }
        if extract_file(&mut dbc_file, file_path).is_err() {
            continue;
        }
        count += 1;
    }
    info!("Extracted {count} game table files");
    Ok(())
}

fn extract_maps(args: &ExtractorConfig, locale: Locale, build_no: u32) -> AzResult<()> {
    let storage = args.get_casc_storage_handler(locale)?;

    info!("Extracting maps...");
    info!("Read Map.db2 file...");
    let source = storage.open_file("DBFilesClient/Map.db2", CascLocale::None.into())?;
    let db2 = wdc1::FileLoader::<MapDb2>::from_reader(source, locale)?;
    let maps = db2.produce_data()?;
    let (num_maps, _) = maps.size_hint();
    info!("Done! ({} maps loaded)", num_maps);

    info!("Read LiquidMaterial.db2 file...");
    let source = storage.open_file("DBFilesClient/LiquidMaterial.db2", CascLocale::None.into())?;
    let db2 = wdc1::FileLoader::<LiquidMaterial>::from_reader(source, locale)?;
    let liquid_materials = db2.produce_data()?.map(|r| (r.id, r)).collect::<BTreeMap<_, _>>();
    info!("Done! ({} LiquidMaterials loaded)", liquid_materials.len());

    // info!("Read LiquidObject.db2 file...");
    // let source = storage.open_file("DBFilesClient/LiquidObject.db2", CascLocale::None.into())?;
    // let db2 = wdc1::FileLoader::<LiquidObject>::from_reader(source, locale)?;
    // let liquid_objects = db2.produce_data()?.map(|r| (r.id, r)).collect::<BTreeMap<_, _>>();
    // info!("Done! ({} LiquidObjects loaded)", liquid_objects.len());

    info!("Read LiquidType.db2 file...");
    let source = storage.open_file("DBFilesClient/LiquidType.db2", CascLocale::None.into())?;
    let db2 = wdc1::FileLoader::<LiquidType>::from_reader(source, locale)?;
    let liquid_types = db2.produce_data()?.map(|r| (r.id, r)).collect::<BTreeMap<_, _>>();
    info!("Done! ({} LiquidTypes loaded)", liquid_types.len());

    let output_path = args.output_map_path();
    fs::create_dir_all(&output_path)?;

    info!("Convert map files");

    for (z, map) in maps.enumerate() {
        let map_id = map.id;
        let map_name = &map.directory;
        let storage_path = format!("World/Maps/{map_name}/{map_name}.wdt");
        let wdt = match ChunkedFile::build(&storage, &storage_path) {
            Err(_e) => {
                // error!("Error opening wdt file at {storage_path}, err was {e}");
                continue;
            },
            Ok(f) => f,
        };
        info!("Extract {} ({}/{})", map_name, z + 1, num_maps);
        // We expect MAIN chunk to always exist

        let chunk = wdt
            .chunks()
            .find_map(|(fcc, data)| if fcc == b"MAIN" { Some(WdtChunkMain::from((fcc, data))) } else { None })
            .unwrap();
        // Loadup map grid data
        for y in 0..WDT_MAP_SIZE {
            for x in 0..WDT_MAP_SIZE {
                if chunk.adt_list[y][x].flag & 0x1 == 0 {
                    continue;
                }
                let storage_path = format!("World/Maps/{map_name}/{map_name}_{x}_{y}.adt");
                let output_file_name = GridMap::file_name(&output_path, map_id, y, x);
                // TODO: Verify if the indices are correct? seems to be reversed here
                let ignore_deep_water = MapDb2::is_deep_water_ignored(map.id, y, x);
                if output_file_name.exists() {
                    continue;
                }
                let map_file = match convert_adt(
                    &storage,
                    &storage_path,
                    args.db2_and_map_extract.allow_height_limit,
                    args.db2_and_map_extract.use_min_height,
                    args.db2_and_map_extract.allow_float_to_int,
                    build_no,
                    ignore_deep_water,
                    &liquid_types,
                    &liquid_materials,
                ) {
                    Err(e) => {
                        error!("error converting {storage_path} ADT to mapfile due to err: {e}");
                        continue;
                    },
                    Ok(f) => f,
                };
                let mut f = buffered_file_create(&output_file_name)?;
                if let Err(e) = map_file.write(&mut f) {
                    let output_file_name_display = output_file_name.display();
                    error!("error saving mapfile to {output_file_name_display} due to err: {e}");
                    continue;
                };
            }
            // // draw progress bar
            // info!("Processing........................{}%\r", (100 * (y + 1)) / WDT_MAP_SIZE);
        }
    }
    Ok(())
}

fn transform_to_high_res(low_res_holes: u16) -> [u8; 8] {
    let mut hi_res_holes = [0u8; 8];
    for (i, hole) in hi_res_holes.iter_mut().enumerate() {
        for j in 0..8 {
            let hole_idx_l = (i / 2) * 4 + (j / 2);
            if ((low_res_holes >> hole_idx_l) & 1) == 1 {
                *hole |= 1 << j;
            }
        }
    }
    hi_res_holes
}

#[expect(clippy::too_many_arguments)]
pub fn convert_adt(
    storage: &CascStorageHandle,
    storage_path: &str,
    allow_height_limit: bool,
    use_min_height: f32,
    allow_float_to_int: bool,
    build_no: u32,
    ignore_deep_water: bool,
    liquid_types_db2: &BTreeMap<u32, LiquidType>,
    liquid_materials_db2: &BTreeMap<u32, LiquidMaterial>,
) -> AzResult<MapFile> {
    let adt = ChunkedFile::build(storage, storage_path)?;

    // Prepare map header
    let map_build_magic = build_no;
    let mut map_area_ids = [[0; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
    let mut map_height_v9 = SMatrix::<f32, ADT_GRID_SIZE_PLUS_ONE, ADT_GRID_SIZE_PLUS_ONE>::zeros();
    let mut map_height_v8 = SMatrix::<f32, ADT_GRID_SIZE, ADT_GRID_SIZE>::zeros();
    let mut map_liquid_flags: [[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID] = [[None.into(); ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
    let mut map_liquid_entry = [[0; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID];
    let mut map_holes = [[[0u8; 8]; 16]; 16];
    let mut has_holes = false;
    let mut map_height_flight_box_max_min = None;

    let mut liquid_show = [[false; ADT_GRID_SIZE]; ADT_GRID_SIZE];
    let mut map_liquid_height = [[0.0; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1];
    // Get area flags data
    for (fcc, chunk) in adt.chunks().filter(|(fcc, _)| *fcc == b"MCNK") {
        let mcnk = AdtChunkMcnk::from((fcc, chunk));
        // Area data
        map_area_ids[mcnk.iy()][mcnk.ix()] = mcnk.areaid.try_into().unwrap();
        // Set map height as grid height
        for y in 0..ADT_CELL_SIZE + 1 {
            let cy = mcnk.iy() * ADT_CELL_SIZE + y;
            for x in 0..ADT_CELL_SIZE + 1 {
                let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                map_height_v9[(cy, cx)] = mcnk.ypos;
            }
        }
        for y in 0..ADT_CELL_SIZE {
            let cy = mcnk.iy() * ADT_CELL_SIZE + y;
            for x in 0..ADT_CELL_SIZE {
                let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                map_height_v8[(cy, cx)] = mcnk.ypos;
            }
        }
        // Get custom height
        if let Some(mcvt) = &mcnk.mcvt {
            // get V9 height map
            for y in 0..ADT_CELL_SIZE + 1 {
                let cy = mcnk.iy() * ADT_CELL_SIZE + y;
                for x in 0..ADT_CELL_SIZE + 1 {
                    let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                    map_height_v9[(cy, cx)] += mcvt.height_map[y * (ADT_CELL_SIZE * 2 + 1) + x];
                }
            }
            // get V8 height map
            for y in 0..ADT_CELL_SIZE {
                let cy = mcnk.iy() * ADT_CELL_SIZE + y;
                for x in 0..ADT_CELL_SIZE {
                    let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                    map_height_v8[(cy, cx)] += mcvt.height_map[y * (ADT_CELL_SIZE * 2 + 1) + ADT_CELL_SIZE + 1 + x];
                }
            }
        }

        // Liquid data
        if mcnk.size_mclq > 8 {
            if let Some(liquid) = &mcnk.mclq {
                let mut count = 0usize;
                for y in 0..ADT_CELL_SIZE {
                    let cy = mcnk.iy() * ADT_CELL_SIZE + y;
                    for x in 0..ADT_CELL_SIZE {
                        let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                        if liquid.flags[y][x] != 0x0F {
                            liquid_show[cy][cx] = true;
                            if !ignore_deep_water && (liquid.flags[y][x] & (1 << 7) > 0) {
                                map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::DarkWater;
                            }
                            count += 1;
                        }
                    }
                }

                let c_flag = mcnk.flags;
                if c_flag & (1 << 2) == (1 << 2) {
                    // water
                    map_liquid_entry[mcnk.iy()][mcnk.ix()] = 1;
                    map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::Water;
                }
                if c_flag & (1 << 3) == (1 << 3) {
                    // ocean
                    map_liquid_entry[mcnk.iy()][mcnk.ix()] = 2;
                    map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::Ocean;
                }
                if c_flag & (1 << 4) == (1 << 4) {
                    // magma/slime
                    map_liquid_entry[mcnk.iy()][mcnk.ix()] = 3;
                    map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::Magma;
                }

                if count == 0 && !map_liquid_flags[mcnk.iy()][mcnk.ix()].is_empty() {
                    error!("Wrong liquid detect in MCLQ chunk");
                }
                for y in 0..ADT_CELL_SIZE + 1 {
                    let cy = mcnk.iy() * ADT_CELL_SIZE + y;
                    for x in 0..ADT_CELL_SIZE + 1 {
                        let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                        map_liquid_height[cy][cx] = liquid.liquid[y][x].height;
                    }
                }
            }
        }

        // Hole data // https://wowdev.wiki/ADT/v18#MCNK_chunk
        if mcnk.flags & 0x10000 != 0x10000 {
            if mcnk.holes_low_res > 0 {
                // transform the block to high res if possible
                let hole = transform_to_high_res(mcnk.holes_low_res);
                if u64::from_le_bytes(hole) != 0 {
                    map_holes[mcnk.iy()][mcnk.ix()] = hole;
                    has_holes = true;
                }
            }
        } else {
            map_holes[mcnk.iy()][mcnk.ix()] = mcnk.high_res_holes;
            if u64::from_le_bytes(mcnk.high_res_holes) != 0 {
                has_holes = true;
            }
        }
    }

    // Get liquid map for grid (in WOTLK used MH2O chunk)
    if let Some((fcc, data)) = adt.chunks().find(|(fcc, _)| *fcc == b"MH2O") {
        let h2o = AdtChunkMh2o::from((fcc, data));
        for i in 0..ADT_CELLS_PER_GRID {
            for j in 0..ADT_CELLS_PER_GRID {
                let h = match h2o.get_liquid_instance(i, j) {
                    None => continue,
                    Some(h) => h,
                };
                let attrs = &h2o.get_liquid_attributes(i, j);

                let mut count = 0;
                let mut exists_mask = h2o.get_exists_bitmap(&h);
                for y in 0..h.get_height() {
                    let cy = i * ADT_CELL_SIZE + y + h.get_offset_y();
                    for x in 0..h.get_width() {
                        let cx = j * ADT_CELL_SIZE + x + h.get_offset_x();
                        if exists_mask & 1 > 0 {
                            liquid_show[cy][cx] = true;
                            count += 1;
                        }
                        exists_mask >>= 1;
                    }
                }
                map_liquid_entry[i][j] = h2o.get_liquid_type(&h, liquid_types_db2, liquid_materials_db2);
                match *liquid_types_db2.get(&(map_liquid_entry[i][j] as u32)).unwrap() {
                    LiquidType { sound_bank, .. } if sound_bank == ADT_LIQUID_TYPE_WATER => {
                        map_liquid_flags[i][j] |= MapLiquidTypeFlag::Water;
                    },
                    LiquidType { sound_bank, .. } if sound_bank == ADT_LIQUID_TYPE_OCEAN => {
                        map_liquid_flags[i][j] |= MapLiquidTypeFlag::Ocean;
                        if !ignore_deep_water && attrs.deep.get() > 0 {
                            map_liquid_flags[i][j] |= MapLiquidTypeFlag::DarkWater;
                        }
                    },
                    LiquidType { sound_bank, .. } if sound_bank == ADT_LIQUID_TYPE_MAGMA => {
                        map_liquid_flags[i][j] |= MapLiquidTypeFlag::Magma;
                    },
                    LiquidType { sound_bank, .. } if sound_bank == ADT_LIQUID_TYPE_SLIME => {
                        map_liquid_flags[i][j] |= MapLiquidTypeFlag::Slime;
                    },
                    _ => {
                        warn!("Can't find Liquid type {} for map {}. chunk {},{}", h.liquid_type.get(), storage_path, i, j);
                    },
                };

                if count == 0 && map_liquid_flags[i][j].bits() > 0 {
                    warn!(
                        "Wrong liquid detect in MH2O chunk, count was {count}, liq flags were: {:?}",
                        map_liquid_flags[i][j],
                    );
                }

                let mut pos = 0;
                for y in 0..=h.get_height() {
                    let cy = i * ADT_CELL_SIZE + y + h.get_offset_y();
                    for x in 0..=h.get_width() {
                        let cx = j * ADT_CELL_SIZE + x + h.get_offset_x();
                        let height = h2o.get_liquid_height(&h, pos, liquid_types_db2, liquid_materials_db2);
                        map_liquid_height[cy][cx] = height;
                        pos += 1;
                    }
                }
            }
        }
    }

    if let Some(mfbo) = adt
        .chunks()
        .find_map(|(fcc, data)| if fcc == b"MFBO" { Some(AdtChunkMfbo::from((fcc, data))) } else { None })
    {
        map_height_flight_box_max_min = Some(MapHeightFlightBox { max: mfbo.max, min: mfbo.min });
    }

    //============================================
    // Try pack area data
    //============================================
    let area_id = map_area_ids[0][0];
    let full_area_data = map_area_ids.iter().any(|row| row.iter().any(|map_area_id| area_id != *map_area_id));

    let map_area_data = if full_area_data { Ok(map_area_ids) } else { Err(area_id) };
    //============================================
    // Try pack height data
    //============================================

    let mut max_height = (-20000.0f32).max(map_height_v8.max().max(map_height_v9.max()));
    let mut min_height = (20000.0f32).min(map_height_v8.min().min(map_height_v9.min()));

    // Check for allow limit minimum height (not store height in deep ochean - allow save some memory)
    if allow_height_limit && min_height < use_min_height {
        map_height_v8.iter_mut().for_each(|v| *v = v.max(use_min_height));
        map_height_v9.iter_mut().for_each(|v| *v = v.max(use_min_height));

        max_height = max_height.max(use_min_height);
        min_height = min_height.max(use_min_height);
    }

    let mut should_include_height = true;
    if max_height == min_height {
        should_include_height = false;
    }
    // Not need store if flat surface
    if allow_float_to_int && (max_height - min_height) < FLAT_HEIGHT_DELTA_LIMIT {
        should_include_height = false;
    }

    let flight_box = map_height_flight_box_max_min;

    // Try store as packed in uint16 or uint8 values
    let map_heights = if !should_include_height {
        None
    } else {
        let diff = max_height - min_height;
        if allow_float_to_int && diff < V9V8_HEIGHT_FLOAT_TO_INT8_LIMIT {
            // As uint8 (max accuracy = CONF_float_to_int8_limit/256)
            let step = 255.0 / diff;
            let map_height_v9 = map_height_v9.map(|v| ((v - min_height) * step + 0.5) as u8);
            let map_height_v8 = map_height_v8.map(|v| ((v - min_height) * step + 0.5) as u8);
            Some(MapFilev9v8::U8 {
                v9: map_height_v9,
                v8: map_height_v8,
            })
        } else if allow_float_to_int && diff < V9V8_HEIGHT_FLOAT_TO_INT16_LIMIT {
            // As uint16 (max accuracy = CONF_float_to_int16_limit/65536)
            let step = 65535.0 / diff;
            let map_height_v9 = map_height_v9.map(|v| ((v - min_height) * step + 0.5) as u16);
            let map_height_v8 = map_height_v8.map(|v| ((v - min_height) * step + 0.5) as u16);
            Some(MapFilev9v8::U16 {
                v9: map_height_v9,
                v8: map_height_v8,
            })
        } else {
            Some(MapFilev9v8::F32 {
                v9: map_height_v9,
                v8: map_height_v8,
            })
        }
    };
    let map_height_data = MapHeightData {
        grid_height: min_height,
        grid_max_height: max_height,
        map_heights,
        flight_box,
    };
    //============================================
    // Pack liquid data
    //============================================
    #[expect(clippy::never_loop)]
    let map_liquid_data = loop {
        let global_liq_info = global_liquid_info(&map_liquid_entry, &map_liquid_flags);
        match global_liq_info {
            // no water data (if all grid have 0 liquid type)
            Some((_, first_liquid_flag)) if first_liquid_flag.is_empty() => {
                // No liquid data
                break None;
            },
            _ => {},
        };
        // has liquid data
        let (mut min_x, mut min_y) = (255, 255);
        let (mut max_x, mut max_y) = (0, 0);
        let mut max_height = -20000f32;
        let mut min_height = 20000f32;
        for y in 0..ADT_GRID_SIZE {
            for x in 0..ADT_GRID_SIZE {
                if liquid_show[y][x] {
                    min_x = min_x.min(x as u8);
                    max_x = max_x.max(x as u8);
                    min_y = min_y.min(y as u8);
                    max_y = max_y.max(y as u8);
                    let h = map_liquid_height[y][x];
                    max_height = max_height.max(h);
                    min_height = min_height.min(h);
                } else {
                    map_liquid_height[y][x] = use_min_height;
                }
            }
        }
        let offset_x = min_x;
        let offset_y = min_y;
        let width = max_x - min_x + 1 + 1;
        let height = max_y - min_y + 1 + 1;
        let liquid_level = min_height;

        // // Not need store if flat surface
        let mut should_include_height = true;
        if max_height == min_height {
            should_include_height = false;
        }
        if allow_float_to_int && (max_height - min_height) < FLAT_LIQUID_DELTA_LIMIT {
            should_include_height = false;
        }

        let map_liquid_entry_flags = if let Some((first_liquid_type, first_liquid_flag)) = global_liq_info {
            Err(MapLiquidDataGlobalEntryFlags {
                liquid_flags: first_liquid_flag,
                liquid_type:  first_liquid_type,
            })
        } else {
            Ok(MapLiquidDataEntryFlags {
                liquid_entry: map_liquid_entry,
                liquid_flags: map_liquid_flags,
            })
        };

        // _liquidMap = new float[uint32(_liquidWidth) * uint32(_liquidHeight)];

        let liquid_height_details = if !should_include_height {
            Err(liquid_level)
        } else {
            // map.liquidMapSize += sizeof(float) * liquidHeader.width * liquidHeader.height;
            let offset_y = offset_y as usize;
            let offset_x = offset_x as usize;
            let mut heights = DMatrix::zeros(height as usize, width as usize);
            for y in 0..height as usize {
                for x in 0..width as usize {
                    heights[(y, x)] = map_liquid_height[y + offset_y][x + offset_x];
                }
            }
            Ok(heights)
        };
        break Some(MapLiquidData {
            offset_x,
            offset_y,
            map_liquid_entry_flags,
            liquid_height_details,
        });
    };
    let map_holes = if has_holes { Some(map_holes) } else { None };
    Ok(MapFile {
        map_build_magic,
        map_area_data,
        map_height_data,
        map_liquid_data,
        map_holes,
    })
}

/// Returns a triple denoting the first liquid type, the first liquid flag, as well as whether or not
/// its a full type
fn global_liquid_info(
    map_liquid_entry: &[[u16; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
    map_liquid_flags: &[[FlagSet<MapLiquidTypeFlag>; ADT_CELLS_PER_GRID]; ADT_CELLS_PER_GRID],
) -> Option<(u16, FlagSet<MapLiquidTypeFlag>)> {
    let first_liquid_type = map_liquid_entry[0][0];
    let first_liquid_flag = map_liquid_flags[0][0];
    for y in 0..map_liquid_entry.len() {
        for x in 0..map_liquid_entry[y].len() {
            if map_liquid_entry[y][x] != first_liquid_type || map_liquid_flags[y][x] != first_liquid_flag {
                // Is full type, returns none
                return None;
            }
        }
    }

    Some((first_liquid_type, first_liquid_flag))
}
