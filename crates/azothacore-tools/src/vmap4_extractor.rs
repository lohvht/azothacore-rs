use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io::{self, Read, Seek, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use azothacore_common::{
    az_error,
    cmp_or_return,
    collision::{
        models::model_instance::{ModelFlags, VmapModelSpawnWithMapId},
        vmap_definitions::RAW_VMAP_MAGIC,
    },
    utils::{bincode_deserialise, bincode_serialise, buffered_file_create, buffered_file_open},
    AzResult,
    Locale,
};
use azothacore_server::{
    game::grid::grid_defines::SIZE_OF_GRIDS,
    shared::data_stores::db2_structure::{GameObjectDisplayInfo, Map},
};
use nalgebra::{Quaternion, Rotation, UnitQuaternion, Vector3};
use tracing::{error, info, instrument, warn};
use wow_db2::wdc1;

use crate::{
    adt::{ADTFile, AdtDoodadDef, AdtMapObjectDefs},
    extractor_common::{
        casc_handles::{CascLocale, CascStorageHandle},
        get_fixed_plain_name,
        ExtractorConfig,
        VmapExtractAndGenerate,
    },
    vmap4_assembler::tile_assembler::WorldModel_Raw,
    vmap4_extractor::{
        model::Model,
        wmo::{WmoDoodadData, WmoMods, WmoRoot},
    },
    wdt::{WDTFile, WDT_MAP_SIZE},
};

pub mod model;
pub mod wmo;

pub struct VmapExtractor {
    pub temp_vmap_dir:         PathBuf,
    pub model_spawns_tmp:      PathBuf,
    pub gameobject_models_tmp: PathBuf,
    pub precise_vector_data:   bool,
}

pub struct FileIterator<T> {
    f:    io::BufReader<fs::File>,
    size: u64,
    t:    PhantomData<T>,
}

impl<T> FileIterator<T> {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let f = fs::File::open(path)?;
        let size = f.metadata()?.len();
        let mut f = io::BufReader::new(f);
        cmp_or_return!(f, RAW_VMAP_MAGIC)?;

        Ok(Self { f, size, t: PhantomData })
    }
}

impl<T: for<'a> serde::Deserialize<'a>> Iterator for FileIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.f.stream_position().ok()? < self.size {
            bincode_deserialise(&mut self.f).ok()
        } else {
            None
        }
    }
}

pub fn main_vmap4_extract(
    args: &ExtractorConfig,
    first_installed_locale: Locale,
) -> AzResult<(FileIterator<VmapModelSpawnWithMapId>, FileIterator<TempGameObjectModel>)> {
    // VMAP EXTRACTOR AND ASSEMBLER
    let version_string = VmapExtractAndGenerate::version_string();
    info!("Extract VMAP {version_string}. Beginning work .....\n\n");
    //xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    // Create the VMAP working directory
    fs::create_dir_all(args.output_vmap_sz_work_dir_wmo())?;

    let model_spawns_tmp = args.output_vmap_sz_work_dir_wmo_dir_bin();
    let gameobject_models_tmp = args.output_vmap_sz_work_dir_wmo_tmp_gameobject_models();
    if model_spawns_tmp.exists() && gameobject_models_tmp.exists() && !args.vmap_extract_and_generate.override_cached {
        // if not override cache, when these 2 files exist, we go ahead load from them instead. It is assumed that
        // if these 2 files are available, the rest of the map / vmap files + doodads etc are extracted too.
        let model_spawns_data = FileIterator::new(model_spawns_tmp)?;
        let temp_gameobject_models = FileIterator::new(gameobject_models_tmp)?;
        info!("Extract VMAP skipped due to no override cached!");
        return Ok((model_spawns_data, temp_gameobject_models));
    }

    let mut wmo_doodads = BTreeMap::new();

    let vmap_extract = VmapExtractor {
        temp_vmap_dir: args.output_vmap_sz_work_dir_wmo(),
        precise_vector_data: args.vmap_extract_and_generate.precise_vector_data,
        model_spawns_tmp,
        gameobject_models_tmp,
    };

    let storage = args.get_casc_storage_handler(first_installed_locale)?;
    {
        // Populate the magic number first
        let mut model_spawns_dir_bin = buffered_file_create(&vmap_extract.model_spawns_tmp)?;
        model_spawns_dir_bin.write_all(RAW_VMAP_MAGIC)?;
        let mut model_list = buffered_file_create(&vmap_extract.gameobject_models_tmp)?;
        model_list.write_all(RAW_VMAP_MAGIC)?;
    };
    vmap_extract.extract_game_object_models(&storage, first_installed_locale, &mut wmo_doodads)?;
    vmap_extract.parse_map_files(first_installed_locale, &storage, &mut wmo_doodads)?;

    let model_spawns_data = FileIterator::new(&vmap_extract.model_spawns_tmp)?;
    let temp_gameobject_models = FileIterator::new(&vmap_extract.gameobject_models_tmp)?;

    // TODO: hirogoro@04jul2023 - VMAP extraction caching (i.e. how not to do more work)
    // 1. save model_spawns_data and temp_gameobject_models to files (similar to
    // `dir_bin` and `temp_gameobject_models` resp in TC),
    // use them as indications that the files are extracted
    // 2. Open via existing map files if possible? - dont rely on CASC storage too much. can probably extract adt and wdt files
    info!("Extract VMAP done!");
    Ok((model_spawns_data, temp_gameobject_models))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TempGameObjectModel {
    pub id:        u32,
    pub is_wmo:    bool,
    pub file_name: String,
}

type AdtCache = Vec<Vec<Option<Arc<Mutex<AdtWithDirFileCache>>>>>;

struct WdtWithAdts {
    _wdt:      WDTFile,
    map_name:  String,
    adt_cache: Option<AdtCache>,
}

impl WdtWithAdts {
    fn get_map(&mut self, storage: &CascStorageHandle, x: usize, y: usize) -> Option<Arc<Mutex<AdtWithDirFileCache>>> {
        if let Some(cache) = &self.adt_cache {
            let cached = cache[x][y].clone();
            if cached.is_some() {
                return cached;
            }
        }
        let map_name = &self.map_name;
        let storage_path = format!("World/Maps/{map_name}/{map_name}_{x}_{y}_obj0.adt");
        let adt = match ADTFile::build(storage, storage_path) {
            Err(_e) => {
                // warn!("Unable to get ADT file {storage_path} with warning: {e}, moving on");
                return None;
            },
            Ok(f) => Arc::new(Mutex::new(AdtWithDirFileCache {
                f,
                cacheable: self.adt_cache.is_some(),
                dir_file_cache: None,
            })),
        };
        if let Some(cache) = self.adt_cache.as_mut() {
            cache[x][y] = Some(adt.clone());
        }
        Some(adt)
    }
}

struct AdtWithDirFileCache {
    f:              ADTFile,
    cacheable:      bool,
    dir_file_cache: Option<Vec<VmapModelSpawnWithMapId>>,
}

impl AdtWithDirFileCache {
    fn init(
        &mut self,
        storage: &CascStorageHandle,
        vm_ex: &VmapExtractor,
        wmo_doodads: &mut BTreeMap<String, WmoDoodadData>,
        map_id: u32,
        original_map_id: u32,
    ) -> AzResult<()> {
        if let Some(dfc) = &self.dir_file_cache {
            if dfc.is_empty() {
                return Ok(());
            }
            let mut dirfile = io::BufWriter::new(fs::File::options().append(true).open(&vm_ex.model_spawns_tmp)?);
            for cached in dfc {
                let mut cached = cached.clone();
                cached.map_num = map_id;
                if map_id != original_map_id {
                    cached.flags |= ModelFlags::ModParentSpawn;
                }
                bincode_serialise(&mut dirfile, &cached)?;
            }
            return Ok(());
        }
        if self.cacheable {
            self.dir_file_cache = Some(vec![]);
        }
        let mut p_dir_file = io::BufWriter::new(fs::File::options().append(true).open(&vm_ex.model_spawns_tmp)?);
        // Do some extraction here as well.
        for mddf in self.f.mddf.iter() {
            for doodad_def in mddf.doodad_defs.iter() {
                let (storage_path, name) = if doodad_def.flags & 0x40 == 0 {
                    let path = self
                        .f
                        .model_paths
                        .get(&(doodad_def.id as usize))
                        .unwrap_or_else(|| panic!("name_id {} should exist in {:?}", doodad_def.id, self.f.model_paths))
                        .clone();
                    let name = get_fixed_plain_name(&path);
                    (path, name)
                } else {
                    let fid = doodad_def.id;
                    let filename = format!("FILE{fid:08X}.xxx");
                    (filename.clone(), filename)
                };
                let ok = vm_ex
                    .extract_single_model(storage, &storage_path)
                    .map_err(|e| {
                        if !e.to_string().contains("has no bounding triangles") {
                            warn!("AdtWithDirFileCache::init extract_single_model err for path {storage_path} due to err: {e}");
                        }
                    })
                    .is_ok();
                if ok {
                    _ = vm_ex
                        .doodad_extract(doodad_def, &name, map_id, original_map_id, &mut p_dir_file, &mut self.dir_file_cache)
                        .map_err(|e| {
                            warn!("AdtWithDirFileCache::init doodad_extract err for {name} due to err: {e}");
                        });
                }
            }
        }
        for modf in self.f.modf.iter() {
            for map_obj_def in modf.map_object_defs.iter() {
                let (storage_path, name) = if map_obj_def.flags & 0x8 == 0 {
                    let path = self
                        .f
                        .wmo_paths
                        .get(&(map_obj_def.id as usize))
                        .unwrap_or_else(|| panic!("name_id {} should exist in {:?}", map_obj_def.id, self.f.wmo_paths));
                    let wmo_inst_name = get_fixed_plain_name(path);
                    (path.clone(), wmo_inst_name)
                } else {
                    let fid = map_obj_def.id;
                    let filename = format!("FILE{fid:08X}.xxx");
                    (filename.clone(), filename)
                };

                _ = vm_ex.extract_single_wmo(storage, &storage_path, wmo_doodads).map_err(|e| {
                    warn!("AdtWithDirFileCache::init extract_single_wmo err for path {storage_path} due to err: {e}");
                });

                _ = vm_ex
                    .mapobject_extract(map_obj_def, &name, false, map_id, original_map_id, &mut p_dir_file, &mut self.dir_file_cache)
                    .map_err(|e| {
                        warn!("AdtWithDirFileCache::init mapobject_extract err for name {name} due to err: {e}");
                    });
                _ = vm_ex
                    .doodad_extractset(
                        &name,
                        map_obj_def,
                        false,
                        map_id,
                        original_map_id,
                        wmo_doodads,
                        &mut p_dir_file,
                        &mut self.dir_file_cache,
                    )
                    .map_err(|e| {
                        warn!("AdtWithDirFileCache::init doodad_extractset err for name {name} due to err: {e}");
                    });
            }
        }
        Ok(())
    }
}

impl VmapExtractor {
    fn extract_game_object_models(&self, storage: &CascStorageHandle, locale: Locale, wmo_doodads: &mut BTreeMap<String, WmoDoodadData>) -> AzResult<()> {
        info!("Extracting GameObject models...");

        let source = storage.open_file("DBFilesClient/GameObjectDisplayInfo.db2", CascLocale::None.into())?;
        let db2 = wdc1::FileLoader::<GameObjectDisplayInfo>::from_reader(source, locale as u32)?;
        let recs = db2.produce_data()?;
        let mut model_list = io::BufWriter::new(fs::File::options().append(true).open(&self.gameobject_models_tmp)?);

        for rec in recs {
            let fid = rec.file_data_id;
            let file_name = format!("FILE{fid:08X}.xxx");
            let mut header_magic = [0u8; 4];
            {
                let mut h = match storage.open_file(&file_name, CascLocale::All.into()) {
                    Err(e) => {
                        warn!("ERROR OPENING GAME DISPLAY INFO: {file_name} - {e}");
                        continue;
                    },
                    Ok(h) => h,
                };
                h.read_exact(&mut header_magic)?;
            }
            let res = match &header_magic {
                // This an an MVER (i.e. a chunked file)
                // should a root WMO
                b"REVM" => self.extract_single_wmo(storage, &file_name, wmo_doodads).map(|_| true),
                b"MD21" => self.extract_single_model(storage, &file_name).map(|_| false),
                c => {
                    let magic = String::from_utf8_lossy(c);
                    return Err(az_error!("File name {file_name} has unexpected header {magic}"));
                },
            };
            let is_wmo = match res {
                Err(e) => {
                    if !e.to_string().contains("has no bounding triangles") {
                        warn!("ERROR Extracting single model/single wmo: {e}");
                    }
                    continue;
                },
                Ok(b) => b,
            };

            bincode_serialise(
                &mut model_list,
                &TempGameObjectModel {
                    id: rec.id,
                    is_wmo,
                    file_name: get_fixed_plain_name(&file_name),
                },
            )?;
        }
        info!("Done!");
        model_list.flush().unwrap();

        Ok(())
    }

    fn extract_single_wmo(&self, storage: &CascStorageHandle, file_name: &str, wmo_doodads: &mut BTreeMap<String, WmoDoodadData>) -> AzResult<()> {
        let plain_name = get_fixed_plain_name(file_name);
        let sz_local_file = self.temp_vmap_dir.join(&plain_name);
        if let Some((_prefix, rest)) = plain_name.rsplit_once('_') {
            if rest.len() == 3 && rest.chars().all(|c| c.is_ascii_digit()) {
                // i.e. the rest after '_' are all digits => not root wmo files, return
                return Ok(());
            }
        }
        if wmo_doodads.get(&plain_name).is_some() {
            // This is to catch when extract_single_wmo is called again but the doodad data is
            // already populated. We dont need to do the below
            // If it isnt therem it can mean 2 things:
            // 1. File does not exist yet => i.e. its not extracted yet, do the extraction.
            // 2. file exists => i.e. re-running the vmap extractor function. Re-retrieve the doodad
            //                  data from WMO Root. We will not save the WMO again, but the doodad
            //                  data will be re-populated as the extracted WMO data does not
            //                  contain doodad data that can be retrieved.
            return Ok(());
        }
        let file_exist = sz_local_file.exists();
        if !file_exist {
            info!("Extracting to vmap: {}", file_name);
        }
        let mut froot = WmoRoot::build(storage, file_name)?;
        froot.doodad_data.references.retain(|_k, s| self.extract_single_model(storage, s).is_ok());
        if !file_exist {
            // save only if not exist. The above code is also to ensure that the model spawns are always idempotent.
            let mut output = buffered_file_create(&sz_local_file).map_err(|e| {
                error!("can't create the output file '{}' for writing, err was: {}", sz_local_file.display(), e);
                e
            })?;
            let vmap = froot.convert_to_vmap(self.precise_vector_data);
            vmap.write(&mut output)?;
        }
        wmo_doodads.insert(plain_name, froot.doodad_data);
        Ok(())
    }

    fn extract_single_model(&self, storage: &CascStorageHandle, file_name: &str) -> AzResult<()> {
        if file_name.len() < 4 {
            return Err(az_error!("File name {file_name} has unexpected length"));
        }

        let file_name_path: &Path = file_name.as_ref();
        let file_name = if let Some(ext) = file_name_path.extension() {
            let mut f_n = file_name_path.to_owned();
            if ext == "mdx" || ext == "MDX" || ext == "mdl" || ext == "MDL" {
                f_n.set_extension("m2");
            }
            f_n
        } else {
            file_name_path.to_owned()
        };
        let plain_name = get_fixed_plain_name(&file_name.to_string_lossy());
        let sz_local_file = self.temp_vmap_dir.join(plain_name);
        if sz_local_file.exists() {
            return Ok(());
        }

        let mdl = Model::build(storage, file_name)?;
        let vmap = mdl.convert_to_vmap();
        let mut output = buffered_file_create(&sz_local_file).map_err(|e| {
            error!("can't create the output file '{}' for writing, err was: {}", sz_local_file.display(), e);
            e
        })?;
        vmap.write(&mut output)?;
        Ok(())
    }

    /// does the role of the insert portion of getWDT internal function in TC's vmap extract code
    ///
    /// returns true if already exists, else do the insert (+ extraction logic) and return false
    /// None is returned when the given WDT file does not exist.
    ///
    /// This function's error handling is explicitly ignored it attempts to follow the original TC code
    /// but we should really try to handle it - TODO: Handle errors properly instead. Really fatal errors
    /// such as invariants within the WoW client files that arent met are *PANICKED*.
    ///
    /// for the get part of getWDT itself, it should be done by just a normal `wdts.get_mut`
    #[instrument(skip_all, fields(map_id=map_id))]
    fn get_wdt<'a>(
        &self,
        map_id: u32,
        storage: &CascStorageHandle,
        maps: &BTreeMap<u32, Map>,
        maps_that_are_parents: &BTreeSet<u32>,
        wdts: &'a mut HashMap<u32, Arc<Mutex<WdtWithAdts>>>,
        wmo_doodads: &mut BTreeMap<String, WmoDoodadData>,
    ) -> Option<Arc<Mutex<WdtWithAdts>>> {
        if let Some(wdt) = wdts.get(&map_id) {
            return Some(wdt.clone());
        }
        let name = maps.get(&map_id).unwrap().directory.def_str();
        let storage_path = format!("World/Maps/{name}/{name}.wdt");
        let wdt = match WDTFile::build(storage, &storage_path) {
            Err(_e) => {
                warn!("unable to open WDT file {storage_path}: {_e}");
                return None;
            },
            Ok(wdt) => wdt,
        };
        // cache ADTs for maps that have parent maps
        let cache = maps_that_are_parents.get(&map_id).is_some();
        let adt_cache = if cache { Some(vec![vec![None; WDT_MAP_SIZE]; WDT_MAP_SIZE]) } else { None };
        let mut p_dir_file = io::BufWriter::new(fs::File::options().append(true).open(&self.model_spawns_tmp).ok()?);
        let mut dir_file_cache = None;
        // do some extraction also
        // global wmo instance data
        for modf in wdt.modf.iter() {
            for map_obj_def in &modf.map_object_defs {
                let (storage_path, name) = if map_obj_def.flags & 0x8 == 0 {
                    let path = wdt.wmo_paths[map_obj_def.id as usize].clone();
                    let name = get_fixed_plain_name(&path);
                    (path, name)
                } else {
                    let fid = map_obj_def.id;
                    let filename = format!("FILE{fid:08X}.xxx");
                    (filename.clone(), filename)
                };

                _ = self.extract_single_wmo(storage, &storage_path, wmo_doodads).map_err(|e| {
                    warn!("get_wdt extract_single_wmo err for path {storage_path} due to err: {e}");
                });
                _ = self
                    .mapobject_extract(map_obj_def, &name, true, map_id, map_id, &mut p_dir_file, &mut dir_file_cache)
                    .map_err(|e| {
                        warn!("get_wdt mapobject_extract err for name {name} due to err: {e}");
                    });
                _ = self
                    .doodad_extractset(&name, map_obj_def, true, map_id, map_id, wmo_doodads, &mut p_dir_file, &mut dir_file_cache)
                    .map_err(|e| {
                        warn!("get_wdt doodad_extractset err for name {name} due to err: {e}");
                    });
            }
        }
        let wdt = wdts.entry(map_id).or_insert(Arc::new(Mutex::new(WdtWithAdts {
            map_name: name,
            _wdt: wdt,
            adt_cache,
        })));
        Some(wdt.clone())
    }

    #[rustfmt::skip]
    pub fn parse_map_files(
        &self,
        locale: Locale,
        storage: &CascStorageHandle,
        wmo_doodads: &mut BTreeMap<String, WmoDoodadData>,
    ) -> AzResult<()> {
        //xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
        //map.dbc
        info!("Read Map.dbc file...");
        let source = storage.open_file("DBFilesClient/Map.db2", CascLocale::None.into())?;
        let db2 = wdc1::FileLoader::<Map>::from_reader(source, locale as u32)?;
        let maps = db2.produce_data()?.map(|r| (r.id, r)).collect::<BTreeMap<_, _>>();
        let maps_that_are_parents = maps.iter().filter_map(|(_, m)| if m.parent_map_id >= 0 { Some(m.parent_map_id.try_into().unwrap()) } else { None } ).collect();
        let maps_len = maps.len();
        info!("Done! ({maps_len} maps loaded)");

        let mut wdts = HashMap::new();
        let now = Instant::now();
        for (i, (map_id, map)) in maps.iter().enumerate() {
            if let Some(wdt) = self.get_wdt(*map_id, storage, &maps, &maps_that_are_parents, &mut wdts, wmo_doodads) {
                let parent_wdt = if map.parent_map_id >= 0 {
                    self.get_wdt(map.parent_map_id.try_into().unwrap(), storage, &maps, &maps_that_are_parents, &mut wdts, wmo_doodads)
                } else {
                    None
                };
                let map_id = map.id;
                let map_name = map.directory.def_str();
                // After populating, then process ADTs
                info!("[{i}/{maps_len}] - Processing Map file {map_id} - {map_name}");
                for x in 0..WDT_MAP_SIZE {
                    for y in 0..WDT_MAP_SIZE {
                        let mut success = false;
                        if let Some(adt) = wdt.lock().unwrap().get_map(storage, x, y) {
                            success = adt.lock().unwrap().init(storage, self, wmo_doodads, map_id, map_id).is_ok();
                        }
                        match &parent_wdt {
                            Some(p_wdt) if !success => {
                                let original_map_id = map.parent_map_id.try_into().unwrap();
                                if let Some(adt) = p_wdt.lock().unwrap().get_map(storage, x, y) {
                                    _ = adt.lock().unwrap().init(storage, self, wmo_doodads, map_id, original_map_id).is_ok();
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
        info!(
            "Done parsing map files and extracting spawns. total time taken was {}s",
            now.elapsed().as_secs()
        );
        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    #[instrument(skip_all, fields(map_id=map_id, original_map_id=original_map_id))]
    pub fn mapobject_extract<W: io::Write>(
        &self,
        map_obj_def: &AdtMapObjectDefs,
        wmo_inst_name: &str,
        is_global_wmo: bool,
        map_id: u32,
        original_map_id: u32,
        mut p_dir_file: &mut W,
        dir_file_cache: &mut Option<Vec<VmapModelSpawnWithMapId>>,
    ) -> AzResult<()> {
        // destructible wmo, do not dump. we can handle the vmap for these
        // in dynamic tree (gameobject vmaps)
        if (map_obj_def.flags & 0x1) != 0 {
            return Ok(());
        }

        //-----------add_in _dir_file----------------
        let tempname = self.temp_vmap_dir.join(wmo_inst_name);
        let mut input = buffered_file_open(tempname)?;
        let (n_vertices, _) = WorldModel_Raw::read_world_model_raw_header(&mut input)?;
        if n_vertices == 0 {
            return Ok(());
        }

        let mut position = fix_coords(&map_obj_def.position);
        let mut bounds = [fix_coords(&map_obj_def.bounds[0]), fix_coords(&map_obj_def.bounds[1])];
        if is_global_wmo {
            position += Vector3::new(SIZE_OF_GRIDS * 32.0, SIZE_OF_GRIDS * 32.0, 0.0);
            bounds[0] += Vector3::new(SIZE_OF_GRIDS * 32.0, SIZE_OF_GRIDS * 32.0, 0.0);
            bounds[1] += Vector3::new(SIZE_OF_GRIDS * 32.0, SIZE_OF_GRIDS * 32.0, 0.0);
        }

        let mut scale = 1.0;
        if map_obj_def.flags & 0x4 > 0 {
            scale = f32::from(map_obj_def.scale) / 1024.0;
        }
        let unique_id = generate_unique_object_id(map_obj_def.unique_id, 0);
        let mut flags = None.into();
        if map_id != original_map_id {
            flags |= ModelFlags::ModParentSpawn;
        }
        //write mapID, Flags, name_set, unique_id, Pos, Rot, Scale, Bound_lo, Bound_hi, name
        let m = VmapModelSpawnWithMapId::new(
            map_id,
            flags,
            map_obj_def.name_set,
            unique_id,
            position,
            map_obj_def.rotation,
            scale,
            Some(bounds),
            wmo_inst_name.to_string(),
        );
        bincode_serialise(&mut p_dir_file, &m)?;
        if let Some(dfc) = dir_file_cache {
            let mut m = m.clone();
            m.flags = None.into();
            dfc.push(m);
        }
        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    #[instrument(skip_all, fields(map_id=map_id, original_map_id=original_map_id))]
    pub fn doodad_extractset<W: io::Write>(
        &self,
        wmo_inst_name: &str,
        wmo: &AdtMapObjectDefs,
        is_global_wmo: bool,
        map_id: u32,
        original_map_id: u32,
        wmo_doodads: &BTreeMap<String, WmoDoodadData>,
        mut p_dir_file: &mut W,
        dir_file_cache: &mut Option<Vec<VmapModelSpawnWithMapId>>,
    ) -> AzResult<()> {
        let doodad_data = match wmo_doodads.get(wmo_inst_name) {
            None => {
                let keys = wmo_doodads.keys().collect::<Vec<_>>();
                return Err(az_error!("name {} should exist in collected wmo doodads {:?}", wmo_inst_name, keys));
            },
            Some(d) => d,
        };
        if doodad_data.sets.is_empty() {
            return Ok(());
        }

        let mut wmo_position = Vector3::new(wmo.position.z, wmo.position.x, wmo.position.y);
        let wmo_rotation = Rotation::from_euler_angles(wmo.rotation.z.to_radians(), wmo.rotation.x.to_radians(), wmo.rotation.y.to_radians());
        if is_global_wmo {
            wmo_position += Vector3::new(SIZE_OF_GRIDS * 32.0, SIZE_OF_GRIDS * 32.0, 0.0);
        }

        let mut doodad_id = 0u16;

        let mut extract_single_set = |doodad_set_data: &WmoMods| {
            for (doodad_index, path) in doodad_data.references.iter() {
                if *doodad_index < doodad_set_data.start_index || *doodad_index >= doodad_set_data.start_index + doodad_set_data.count {
                    continue;
                }

                let doodad = &doodad_data.spawns[*doodad_index as usize];

                let plain_name = get_fixed_plain_name(path);
                let file_name_path: &Path = plain_name.as_ref();
                let plain_name = if let Some(ext) = file_name_path.extension() {
                    let mut f_n = file_name_path.to_owned();
                    if ext == "mdx" || ext == "MDX" || ext == "mdl" || ext == "MDL" {
                        f_n.set_extension("m2");
                    }
                    f_n
                } else {
                    file_name_path.to_owned()
                };
                let model_inst_name = plain_name.to_string_lossy().to_string();
                let tempname = self.temp_vmap_dir.join(&model_inst_name);
                let mut input = match buffered_file_open(&tempname).map_err(|e| az_error!("READ_ERR: path={}, err={e}", tempname.display())) {
                    Err(e) => {
                        error!("Unable to open file at {} to read vertices: {e}", tempname.display());
                        continue;
                    },
                    Ok(f) => f,
                };
                let (n_vertices, _) = match WorldModel_Raw::read_world_model_raw_header(&mut input) {
                    Err(e) => {
                        error!("Unable to read world model header at {} to read vertices: {e}", tempname.display());
                        continue;
                    },
                    Ok(r) => r,
                };
                if n_vertices == 0 {
                    continue;
                }

                if doodad_id == u16::MAX {
                    panic!("doodad_id cannot exceed u16 maximum");
                }
                doodad_id += 1;

                let position = wmo_position + (wmo_rotation * Vector3::new(doodad.position.x, doodad.position.y, doodad.position.z));

                // Vec3D rotation;
                // (G3D::Quat(doodad.Rotation.X, doodad.Rotation.Y, doodad.Rotation.Z, doodad.Rotation.W)
                //     .toRotationMatrix() * wmoRotation)
                //     .toEulerAnglesXYZ(rotation.z, rotation.x, rotation.y);
                // X - roll, Y - pitch, Z - yaw
                let (z, x, y) = (UnitQuaternion::from_quaternion(Quaternion::from(doodad.rotation)).to_rotation_matrix() * wmo_rotation)
                    // .transpose() // Different from TC, have to transpose because G3D is in row-major, but nalgebra is in column major
                    .euler_angles();
                let rotation = Vector3::new(x.to_degrees(), y.to_degrees(), z.to_degrees());

                let name_set = 0; // not used for models
                let unique_id = generate_unique_object_id(wmo.unique_id, doodad_id);
                let mut tcflags = ModelFlags::ModM2.into();
                if map_id != original_map_id {
                    tcflags |= ModelFlags::ModParentSpawn;
                }

                //write mapID, Flags, name_set, unique_id, Pos, Rot, Scale, name
                let m = VmapModelSpawnWithMapId::new(
                    map_id,
                    tcflags,
                    name_set, // not used for models
                    unique_id,
                    position,
                    rotation,
                    doodad.scale,
                    None,
                    model_inst_name,
                );
                if let Err(e) = bincode_serialise(&mut p_dir_file, &m) {
                    warn!("Error saving extractset spawn: {e}");
                    continue;
                };
                if let Some(dfc) = dir_file_cache {
                    let mut m = m.clone();
                    m.flags = ModelFlags::ModM2.into();
                    dfc.push(m);
                }
            }
        };
        // first doodad set is always active
        extract_single_set(&doodad_data.sets[0]);
        if wmo.doodad_set != 0 && usize::from(wmo.doodad_set) < doodad_data.sets.len() {
            extract_single_set(&doodad_data.sets[usize::from(wmo.doodad_set)]);
        }

        Ok(())
    }

    #[instrument(skip_all, fields(map_id=map_id, original_map_id=original_map_id))]
    pub fn doodad_extract<W: io::Write>(
        &self,
        doodad_def: &AdtDoodadDef,
        model_inst_name: &str,
        map_id: u32,
        original_map_id: u32,
        mut p_dir_file: &mut W,
        dir_file_cache: &mut Option<Vec<VmapModelSpawnWithMapId>>,
    ) -> AzResult<()> {
        let tempname = self.temp_vmap_dir.join(model_inst_name);
        let mut input = buffered_file_open(tempname)?;
        let (n_vertices, _) = WorldModel_Raw::read_world_model_raw_header(&mut input)?;
        if n_vertices == 0 {
            return Ok(());
        }
        // scale factor - divide by 1024. blizzard devs must be on crack, why not just use a float?
        let sc = f32::from(doodad_def.scale) / 1024f32;

        let position = fix_coords(&doodad_def.position);

        let mut flags = ModelFlags::ModM2.into();
        if map_id != original_map_id {
            flags |= ModelFlags::ModParentSpawn;
        }
        //write mapID, Flags, name_set, unique_id, Pos, Rot, Scale, name
        let m = VmapModelSpawnWithMapId::new(
            map_id,
            flags,
            0, // not used for models
            generate_unique_object_id(doodad_def.unique_id, 0),
            position,
            doodad_def.rotation,
            sc,
            None,
            model_inst_name.to_string(),
        );
        bincode_serialise(&mut p_dir_file, &m)?;
        if let Some(dfc) = dir_file_cache {
            let mut m = m.clone();
            m.flags = ModelFlags::ModM2.into();
            dfc.push(m);
        }
        Ok(())
    }
}

fn fix_coords(v: &Vector3<f32>) -> Vector3<f32> {
    Vector3::new(v.z, v.x, v.y)
}

static UNIQUE_OBJECT_BANK: Mutex<BTreeMap<(u32, u16), u32>> = Mutex::new(BTreeMap::new());

fn generate_unique_object_id(client_id: u32, client_doodad_id: u16) -> u32 {
    let mut bank = UNIQUE_OBJECT_BANK.lock().unwrap();

    let next_id = (bank.len() + 1) as u32;
    *bank.entry((client_id, client_doodad_id)).or_insert(next_id)
}
