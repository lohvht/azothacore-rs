#![feature(result_option_inspect)]

use std::{
    collections::BTreeMap,
    fs,
    io,
    path::{Path, PathBuf},
};

use azothacore_rs::{
    common::{banner, Locale},
    logging::init_logging,
    server::{
        game::map::{MapFile, MapLiquidTypeFlag},
        shared::data_stores::db2_structure::{CinematicCamera, LiquidMaterial, LiquidObject, LiquidType, Map},
    },
    tools::{
        adt::{
            AdtChunkMcnk,
            AdtChunkMcnkSubchunkMclq,
            AdtChunkMcnkSubchunkMcvt,
            AdtChunkMfbo,
            AdtChunkMh2o,
            AdtChunkMh2oLiquidInstance,
            AdtLiquidType,
            LiquidVertexFormatType,
            ADT_CELLS_PER_GRID,
            ADT_CELL_SIZE,
            ADT_GRID_SIZE,
        },
        basic_extractor::{ChunkedFile, DB_FILES_CLIENT_LIST},
        extractor_common::{
            casc_handles::{CascFileHandle, CascHandlerError, CascLocale, CascStorageHandle},
            DB2AndMapExtractFlags,
            Db2AndMapExtract,
            ExtractorConfig,
        },
        wdt::{WdtChunkMain, WDT_MAP_SIZE},
    },
    GenericResult,
};
use byteorder::{LittleEndian, ReadBytesExt};
use flagset::FlagSet;
use tracing::{error, info, warn};
use walkdir::WalkDir;
use wow_db2::wdc1;

fn main() -> GenericResult<()> {
    init_logging();
    let mut f = fs::File::open("env/dist/etc/extractor.toml")?;
    let args = ExtractorConfig::from_toml(&mut f)?;

    banner::azotha_banner_show("DBC, Maps, VMaps & MMaps Extractor", || {
        info!("Client directory: {}", args.input_path);
        info!("Data directory:   {}", args.output_path);
        info!("rest of config: {:?}", args);
    });

    old_client_check(&args)?;

    // MAP & DB2 EXTRACTOR
    let installed_locales_mask = get_installed_locales_mask(&args)?;
    let mut first_installed_locale: Option<Locale> = None;
    let mut build = 0;

    for l in args.locales.into_iter() {
        if let Locale::none = l {
            continue;
        }
        if (installed_locales_mask & l.to_casc_locales()).bits() == 0 {
            continue;
        }
        let storage = match get_casc_storage_handler(&args, l) {
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
        let product_info = match storage.get_product_info() {
            Err(_) => continue,
            Ok(r) => r,
        };
        if first_installed_locale.is_none() {
            build = product_info.build_number;
            first_installed_locale = Some(l);
        }

        if !args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::Dbc) {
            info!("Detected client build: {}", build);
            break;
        }
        // Extract DBC files
        info!("Detected client build: {} for locale {}", build, l);
        extract_db_files_client(&storage, &args, l)?;
    }

    let first_installed_locale = if let Some(l) = first_installed_locale {
        l
    } else {
        info!("No locales detected!");
        return Ok(());
    };

    if args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::Camera) {
        extract_camera_files(&args, first_installed_locale)?;
    }
    if args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::GameTables) {
        extract_game_tables(&args, first_installed_locale)?;
    }
    if args.db2_and_map_extract.should_extract(DB2AndMapExtractFlags::Map) {
        extract_maps(&args, first_installed_locale, build)?;
    }

    // VMAP EXTRACTOR AND ASSEMBLER

    Ok(())
}

fn get_casc_storage_handler(args: &ExtractorConfig, locale: Locale) -> Result<CascStorageHandle, CascHandlerError> {
    CascStorageHandle::build(args.input_storage_data_dir(), locale.to_casc_locales())
}

fn get_casc_filename_part<P: AsRef<Path>>(casc_path: P) -> PathBuf {
    if let Some(last_sep) = casc_path.as_ref().file_name() {
        Path::new(last_sep).to_path_buf()
    } else {
        casc_path.as_ref().to_path_buf()
    }
}

fn extract_db_files_client(storage: &CascStorageHandle, args: &ExtractorConfig, locale: Locale) -> GenericResult<()> {
    info!("Extracting dbc/db2 files for {}...", locale);
    let locale_path = args.output_dbc_path(locale);

    fs::create_dir_all(&locale_path)?;

    info!("locale {} output path {}", locale, locale_path.display());
    let mut count = 0;

    for file_name in DB_FILES_CLIENT_LIST {
        let mut dbc_file = match storage.open_file(file_name, FlagSet::from(CascLocale::None)) {
            Err(e) => {
                error!("Unable to open file {} in the archive for locale {}: {}", file_name, locale, e);
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

fn extract_file(file_in_archive: &mut CascFileHandle, out_path: PathBuf) -> GenericResult<()> {
    let file_size = file_in_archive.get_file_size()?;

    let mut output = fs::File::create(&out_path).inspect_err(|e| {
        error!("can't create the output file '{}', err was: {}", out_path.display(), e);
    })?;

    let res = io::copy(file_in_archive, &mut output)?;

    // Sanity check here! just verifying that file_size detected is the same as the result
    if file_size != res {
        let e = Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Extracted file sizes don't match somehow for {}. expected {}, got {}",
                out_path.display(),
                file_size,
                res
            )
            .as_str(),
        ));
        error!("extract_file has failed somehow: {}", e);
        Err(e)
    } else {
        Ok(())
    }
}

fn extract_camera_files(args: &ExtractorConfig, locale: Locale) -> GenericResult<()> {
    info!("Extracting camera files...");

    let storage = get_casc_storage_handler(args, locale)?;
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

fn read_cinematic_camera_dbc(storage: &CascStorageHandle, locale: Locale) -> GenericResult<Vec<String>> {
    info!("Read CinematicCamera.db2 file...");
    let source = storage.open_file("DBFilesClient/CinematicCamera.db2", CascLocale::None.into())?;
    let fl = wdc1::FileLoader::<CinematicCamera>::from_reader(source, locale as u32)?;
    let data = fl.produce_data()?;

    let res = data
        .values()
        .map(|d| {
            let fid = d.file_data_id;
            format!("FILE{fid:08X}.xxx")
        })
        .collect::<Vec<_>>();

    info!("Done! ({} CinematicCameras loaded)", res.len());
    Ok(res)
}

fn extract_game_tables(args: &ExtractorConfig, locale: Locale) -> GenericResult<()> {
    info!("Extracting game tables...");
    let storage = get_casc_storage_handler(args, locale)?;
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

fn extract_maps(args: &ExtractorConfig, locale: Locale, build_no: u32) -> GenericResult<()> {
    let storage = get_casc_storage_handler(args, locale)?;

    info!("Extracting maps...");
    info!("Read Map.db2 file...");
    let source = storage.open_file("DBFilesClient/Map.db2", CascLocale::None.into())?;
    let db2 = wdc1::FileLoader::<Map>::from_reader(source, locale as u32)?;
    let maps = db2.produce_data()?;
    info!("Done! ({} maps loaded)", maps.len());

    info!("Read LiquidMaterial.db2 file...");
    let source = storage.open_file("DBFilesClient/LiquidMaterial.db2", CascLocale::None.into())?;
    let db2 = wdc1::FileLoader::<LiquidMaterial>::from_reader(source, locale as u32)?;
    let liquid_materials = db2.produce_data()?;
    info!("Done! ({} LiquidMaterials loaded)", liquid_materials.len());

    info!("Read LiquidObject.db2 file...");
    let source = storage.open_file("DBFilesClient/LiquidObject.db2", CascLocale::None.into())?;
    let db2 = wdc1::FileLoader::<LiquidObject>::from_reader(source, locale as u32)?;
    let liquid_objects = db2.produce_data()?;
    info!("Done! ({} LiquidObjects loaded)", liquid_objects.len());

    info!("Read LiquidType.db2 file...");
    let source = storage.open_file("DBFilesClient/LiquidType.db2", CascLocale::None.into())?;
    let db2 = wdc1::FileLoader::<LiquidType>::from_reader(source, locale as u32)?;
    let liquid_types = db2.produce_data()?;
    info!("Done! ({} LiquidTypes loaded)", liquid_types.len());

    let output_path = args.output_map_path();
    fs::create_dir_all(&output_path)?;

    info!("Convert map files");
    for (z, (map_id, map)) in maps.iter().enumerate() {
        let map_name = &map.directory[locale as usize];
        info!("Extract {} ({}/{})", map_name, z + 1, maps.len());
        let storage_path = format!("World/Maps/{map_name}/{map_name}.wdt");
        let wdt = match ChunkedFile::build(&storage, &storage_path) {
            Err(e) => {
                error!("Error opening wdt file at {storage_path}, err was {e}");
                continue;
            },
            Ok(f) => f,
        };
        // We expect MAIN chunk to always exist
        let chunk = wdt.chunks.get(b"MAIN").unwrap();
        let chunk = WdtChunkMain::from(chunk.clone());
        // Loadup map grid data
        for y in 0..WDT_MAP_SIZE {
            for x in 0..WDT_MAP_SIZE {
                let storage_path = format!("World/Maps/{map_name}/{map_name}_{x}_{y}.adt");
                if chunk.adt_list[y][x].flag & 0x1 == 0 {
                    continue;
                }
                let output_file_name = output_path.join(format!("{map_id:04}_{y:02}_{x:02}.map"));
                if output_file_name.exists() {
                    continue;
                }
                // TODO: Verify if the indices are correct? seems to be reversed here
                let ignore_deep_water = map.is_deep_water_ignored(y, x);
                let _ = convert_adt(
                    &args.db2_and_map_extract,
                    &storage,
                    storage_path.as_ref(),
                    output_file_name.as_ref(),
                    build_no,
                    ignore_deep_water,
                    &liquid_types,
                    &liquid_materials,
                )
                .inspect_err(|e| {
                    let output_file_name_display = output_file_name.display();
                    error!("error converting {storage_path} ADT to map {output_file_name_display} due to err: {e}");
                });
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

fn get_liquid_vertex_format(
    liquid_instance: &AdtChunkMh2oLiquidInstance,
    liquid_types_db2: &BTreeMap<u32, LiquidType>,
    liquid_materials_db2: &BTreeMap<u32, LiquidMaterial>,
) -> Option<u16> {
    if liquid_instance.liquid_vertex_format < 42 {
        return Some(liquid_instance.liquid_vertex_format);
    }
    if liquid_instance.liquid_type == LiquidVertexFormatType::Depth as u16 {
        return Some(liquid_instance.liquid_type);
    }

    if let Some(liquid_type) = liquid_types_db2.get(&(liquid_instance.liquid_type as u32)) {
        if let Some(liquid_material) = liquid_materials_db2.get(&(liquid_type.material_id as u32)) {
            return Some(liquid_material.lvf as u16);
        }
    }
    None
}

fn get_liquid_type(h: &AdtChunkMh2oLiquidInstance, liquid_types_db2: &BTreeMap<u32, LiquidType>, liquid_materials_db2: &BTreeMap<u32, LiquidMaterial>) -> u16 {
    match get_liquid_vertex_format(h, liquid_types_db2, liquid_materials_db2) {
        Some(t) if t == LiquidVertexFormatType::Depth as u16 => 2,
        _ => h.liquid_type,
    }
}

fn get_liquid_height(
    mh20_raw_data: &mut io::Cursor<Vec<u8>>,
    h: &AdtChunkMh2oLiquidInstance,
    pos: usize,
    liquid_types_db2: &BTreeMap<u32, LiquidType>,
    liquid_materials_db2: &BTreeMap<u32, LiquidMaterial>,
) -> f32 {
    if h.offset_vertex_data == 0 {
        return 0.0;
    }
    let lvf = match get_liquid_vertex_format(h, liquid_types_db2, liquid_materials_db2) {
        Some(t) if t != LiquidVertexFormatType::Depth as u16 => t,
        _ => return 0.0,
    };

    if lvf == LiquidVertexFormatType::HeightDepth as u16
        || lvf == LiquidVertexFormatType::HeightTextureCoord as u16
        || lvf == LiquidVertexFormatType::HeightDepthTextureCoord as u16
    {
        mh20_raw_data.set_position(h.offset_vertex_data as u64 + pos as u64);
        mh20_raw_data.read_f32::<LittleEndian>().unwrap()
    } else if lvf == LiquidVertexFormatType::Depth as u16 {
        0.0
    } else if lvf == LiquidVertexFormatType::Unk4 as u16 || lvf == LiquidVertexFormatType::Unk5 as u16 {
        mh20_raw_data.set_position(h.offset_vertex_data as u64 + 4 + pos as u64 * 2);
        mh20_raw_data.read_f32::<LittleEndian>().unwrap()
    } else {
        0.0
    }
}

#[allow(clippy::too_many_arguments)]
fn convert_adt(
    args: &Db2AndMapExtract,
    storage: &CascStorageHandle,
    input_path: &Path,
    output_path: &Path,
    build_no: u32,
    ignore_deep_water: bool,
    liquid_types_db2: &BTreeMap<u32, LiquidType>,
    liquid_materials_db2: &BTreeMap<u32, LiquidMaterial>,
) -> GenericResult<()> {
    let adt = ChunkedFile::build(storage, input_path)?;
    // Prepare map header
    let mut map_file = MapFile::default();
    map_file.map_build_magic = build_no;

    let mut liquid_show = [[false; ADT_GRID_SIZE]; ADT_GRID_SIZE];
    let mut map_liquid_height = [[args.use_min_height; ADT_GRID_SIZE + 1]; ADT_GRID_SIZE + 1];
    // Get area flags data
    for (_fcc, chunk) in adt.chunks.iter().filter(|(fcc, _)| *fcc == b"MCNK") {
        let mcnk = AdtChunkMcnk::from(chunk.clone());
        // Area data
        map_file.map_area_ids[mcnk.iy()][mcnk.ix()] = mcnk.areaid.try_into().unwrap();
        // Set map height as grid height
        for y in 0..ADT_CELL_SIZE + 1 {
            let cy = mcnk.iy() * ADT_CELL_SIZE + y;
            for x in 0..ADT_CELL_SIZE + 1 {
                let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                map_file.map_height_V9[cy][cx] = mcnk.ypos;
            }
        }
        for y in 0..ADT_CELL_SIZE {
            let cy = mcnk.iy() * ADT_CELL_SIZE + y;
            for x in 0..ADT_CELL_SIZE {
                let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                map_file.map_height_V8[cy][cx] = mcnk.ypos;
            }
        }
        // Get custom height
        if let Some(chunk) = chunk.sub_chunks.get(b"MCVT") {
            let mcvt = AdtChunkMcnkSubchunkMcvt::from(chunk.clone());
            // get V9 height map
            for y in 0..ADT_CELL_SIZE + 1 {
                let cy = mcnk.iy() * ADT_CELL_SIZE + y;
                for x in 0..ADT_CELL_SIZE + 1 {
                    let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                    map_file.map_height_V9[cy][cx] += mcvt.height_map[y * (ADT_CELL_SIZE * 2 + 1) + x];
                }
            }
            // get V8 height map
            for y in 0..ADT_CELL_SIZE {
                let cy = mcnk.iy() * ADT_CELL_SIZE + y;
                for x in 0..ADT_CELL_SIZE {
                    let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                    map_file.map_height_V8[cy][cx] += mcvt.height_map[y * (ADT_CELL_SIZE * 2 + 1) + ADT_CELL_SIZE + 1 + x];
                }
            }
        }

        // Liquid data
        if mcnk.size_mclq > 8 {
            if let Some(chunk) = chunk.sub_chunks.get(b"MCLQ") {
                let liquid = AdtChunkMcnkSubchunkMclq::from(chunk.clone());
                let mut count = 0usize;
                for y in 0..ADT_CELL_SIZE {
                    let cy = mcnk.iy() * ADT_CELL_SIZE + y;
                    for x in 0..ADT_CELL_SIZE {
                        let cx = mcnk.ix() * ADT_CELL_SIZE + x;
                        if liquid.flags[y][x] != 0x0F {
                            liquid_show[cy][cx] = true;
                            if !ignore_deep_water && (liquid.flags[y][x] & (1 << 7) == (1 << 7)) {
                                map_file.map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::DarkWater;
                            }
                            count += 1;
                        }
                    }
                }

                let c_flag = mcnk.flags;
                if c_flag & (1 << 2) == (1 << 2) {
                    // water
                    map_file.map_liquid_entry[mcnk.iy()][mcnk.ix()] = 1;
                    map_file.map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::Water;
                }
                if c_flag & (1 << 3) == (1 << 3) {
                    // ocean
                    map_file.map_liquid_entry[mcnk.iy()][mcnk.ix()] = 2;
                    map_file.map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::Ocean;
                }
                if c_flag & (1 << 4) == (1 << 4) {
                    // magma/slime
                    map_file.map_liquid_entry[mcnk.iy()][mcnk.ix()] = 3;
                    map_file.map_liquid_flags[mcnk.iy()][mcnk.ix()] |= MapLiquidTypeFlag::Magma;
                }

                if count == 0 && map_file.map_liquid_flags[mcnk.iy()][mcnk.ix()].bits() > 0 {
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
        // map_file.map_holes[mcnk.iy()][mcnk.ix()]
        // &mut map_file.map_holes[mcnk.iy()][mcnk.ix()]
        // u64::from_le_bytes() != 0;

        // [[[0u8; 8]; 16]; 16]

        // Hole data // https://wowdev.wiki/ADT/v18#MCNK_chunk

        // Does not have high_res_holes flag
        let chunk_holes = if (mcnk.flags & 0x10000) != 0x10000 {
            if mcnk.holes_low_res > 0 {
                // transform the block to high res if possible
                Some(transform_to_high_res(mcnk.holes_low_res))
            } else {
                None
            }
        } else {
            // Have high_res_holes flag
            Some(mcnk.high_res_holes)
        };

        match chunk_holes {
            Some(h) if u64::from_le_bytes(h) != 0 => {
                if map_file.map_holes.is_none() {
                    map_file.map_holes = Some([[[0u8; 8]; 16]; 16]);
                }
                let map_file_holes = &mut map_file.map_holes.unwrap();
                map_file_holes[mcnk.iy()][mcnk.ix()] = h;
            },
            _ => {},
        }
    }

    // Get liquid map for grid (in WOTLK used MH2O chunk)
    if let Some(chunk) = adt.chunks.get(b"MH2O") {
        let mut h2o = AdtChunkMh2o::from(chunk.clone());
        for i in 0..ADT_CELLS_PER_GRID {
            for j in 0..ADT_CELLS_PER_GRID {
                if h2o.liquid[i][j].used == 0 && h2o.liquid[i][j].offset_instances == 0 {
                    continue;
                }
                let mut count = 0;
                let mut exists_mask = h2o.get_exists_bitmap(i, j);
                let h = &h2o.liquid_instance[i][j];
                let attrs = &h2o.liquid_attributes[i][j];
                for y in 0..h.get_height() {
                    let cy = i * ADT_CELL_SIZE + y + h.get_offset_y();
                    for x in 0..h.get_width() {
                        let cx = j * ADT_CELL_SIZE + x + h.get_offset_x();
                        if exists_mask & 1 == 1 {
                            liquid_show[cy][cx] = true;
                            count += 1;
                        }
                        exists_mask >>= 1;
                    }
                }
                map_file.map_liquid_entry[i][j] = get_liquid_type(h, liquid_types_db2, liquid_materials_db2);
                match liquid_types_db2.get(&(map_file.map_liquid_entry[i][j] as u32)) {
                    None => {
                        warn!("can't find liquid_type of ID {}", &(map_file.map_liquid_entry[i][j] as u32));
                    },
                    Some(&LiquidType { sound_bank, .. }) if sound_bank == AdtLiquidType::Water as u8 => {
                        map_file.map_liquid_flags[i][j] |= MapLiquidTypeFlag::Water
                    },
                    Some(&LiquidType { sound_bank, .. }) if sound_bank == AdtLiquidType::Ocean as u8 => {
                        map_file.map_liquid_flags[i][j] |= MapLiquidTypeFlag::Ocean;
                        if !ignore_deep_water && attrs.deep > 0 {
                            map_file.map_liquid_flags[i][j] |= MapLiquidTypeFlag::DarkWater;
                        }
                    },
                    Some(&LiquidType { sound_bank, .. }) if sound_bank == AdtLiquidType::Magma as u8 => {
                        map_file.map_liquid_flags[i][j] |= MapLiquidTypeFlag::Magma
                    },
                    Some(&LiquidType { sound_bank, .. }) if sound_bank == AdtLiquidType::Slime as u8 => {
                        map_file.map_liquid_flags[i][j] |= MapLiquidTypeFlag::Slime
                    },
                    _ => {
                        warn!(
                            "\nCan't find Liquid type {} for map {}\nchunk {},{}\n",
                            h.liquid_type,
                            input_path.display(),
                            i,
                            j
                        );
                    },
                };

                if count == 0 && map_file.map_liquid_flags[i][j].bits() > 0 {
                    warn!(
                        "Wrong liquid detect in MH2O chunk, count was {count}, liq flags were: {:?}",
                        map_file.map_liquid_flags[i][j],
                    );
                }

                let mut pos = 0;
                for y in 0..h.get_height() {
                    let cy = i * ADT_CELL_SIZE + y + h.get_offset_y();
                    for x in 0..h.get_width() {
                        let cx = j * ADT_CELL_SIZE + x + h.get_offset_x();
                        map_liquid_height[cy][cx] = get_liquid_height(&mut h2o.raw_data, h, pos, liquid_types_db2, liquid_materials_db2);
                        pos += 1;
                    }
                }
            }
        }
    }

    if let Some(chunk) = adt.chunks.get(b"MFBO") {
        let mfbo = AdtChunkMfbo::from(chunk.clone());
        map_file.map_height_flight_box_max_min = Some((mfbo.max, mfbo.min));
    }
    let mut f = fs::File::create(output_path)?;

    map_file.pack(args.allow_height_limit, liquid_show, map_liquid_height, args.use_min_height, &mut f)?;

    Ok(())
}

fn get_installed_locales_mask(args: &ExtractorConfig) -> GenericResult<FlagSet<CascLocale>> {
    let storage = get_casc_storage_handler(args, Locale::none)?;

    Ok(storage.get_installed_locales_mask()?)
}

/// old_client_check checks if there are any MPQ files in the Data directory
/// If it does, returns check
fn old_client_check(args: &ExtractorConfig) -> io::Result<()> {
    let storage_dir = args.input_storage_data_dir();
    let has_mpq = WalkDir::new(storage_dir).into_iter().any(|direntry| {
        match direntry {
            Err(err) => {
                error!("Error checking client version due to directory walk error: {}", err.to_string());
                // skip over anyway
                false
            },
            Ok(de) => {
                let r = de.path().extension();
                if let Some(ex) = r {
                    let res = "MPQ" == ex;
                    if res {
                        error!(
                            r#"
                        MPQ files found in Data directory!
                        This tool works only with World of Warcraft: Legion

                        To extract maps for Wrath of the Lich King, rebuild tools using 3.3.5 branch!
                        "#
                        )
                    }
                    res
                } else {
                    // If directory has no extension we shouldnt really care
                    false
                }
            },
        }
    });
    if has_mpq {
        Err(io::Error::new(io::ErrorKind::Other, "HAS_MPQ"))
    } else {
        Ok(())
    }
}
