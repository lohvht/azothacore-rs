use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    io::{self, Read},
    path::Path,
};

use nalgebra::{Quaternion, Rotation, UnitQuaternion, Vector3};
use tracing::{error, info, warn};
use wow_db2::wdc1;

use crate::{
    common::{
        collision::{
            maps::tile_assembler::WorldModel_Raw,
            models::model_instance::{ModelFlags, VmapModelSpawn},
        },
        Locale,
    },
    server::shared::data_stores::db2_structure::{GameObjectDisplayInfo, Map},
    tools::{
        adt::{ADTFile, AdtDoodadDef, AdtMapObjectDefs},
        extractor_common::{
            casc_handles::{CascLocale, CascStorageHandle},
            get_fixed_plain_name,
            ExtractorConfig,
            VmapExtractAndGenerate,
        },
        vmap4_extractor::{
            model::Model,
            wmo::{WmoDoodadData, WmoRoot},
        },
        wdt::{WDTFile, WDT_MAP_SIZE},
    },
    GenericResult,
};

mod model;
pub mod wmo;

#[derive(Default)]
pub struct VmapExtractor {
    args: ExtractorConfig,
    wmo_doodads: BTreeMap<String, WmoDoodadData>,
    /// Mapping of map_id to spawn ID to its spawn.
    pub model_spawns_data: BTreeMap<u32, BTreeMap<u32, VmapModelSpawn>>,
    pub temp_gameobject_models: Vec<TempGameObjectModel>,
    unique_object_id: HashMap<(u32, u16), u32>,
}

pub fn main_vmap4_extract(args: &ExtractorConfig, first_installed_locale: Locale) -> GenericResult<VmapExtractor> {
    // VMAP EXTRACTOR AND ASSEMBLER
    let version_string = VmapExtractAndGenerate::version_string();
    info!("Extract VMAP {version_string}. Beginning work .....\n\n");
    //xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    // Create the VMAP working directory
    fs::create_dir_all(args.output_vmap_sz_work_dir_wmo())?;

    let mut vmap_extract = VmapExtractor {
        args: args.clone(),
        ..VmapExtractor::default()
    };

    let model_spawns_tmp = args.output_vmap_sz_work_dir_wmo_dir_bin();
    let gameobject_models_tmp = args.output_vmap_sz_work_dir_wmo_tmp_gameobject_models();
    if model_spawns_tmp.exists() && gameobject_models_tmp.exists() && !args.vmap_extract_and_generate.override_cached {
        // if not override cache, when these 2 files exist, we go ahead load from them instead. It is assumed that
        // if these 2 files are available, the rest of the map / vmap files + doodads etc are extracted too.
        vmap_extract.model_spawns_data = bincode::deserialize_from(fs::File::open(model_spawns_tmp)?)?;
        vmap_extract.temp_gameobject_models = bincode::deserialize_from(fs::File::open(gameobject_models_tmp)?)?;
        info!("Extract VMAP skipped due to no override cached!");
        return Ok(vmap_extract);
    }

    let storage = args.get_casc_storage_handler(first_installed_locale)?;
    vmap_extract.extract_game_object_models(&storage, first_installed_locale)?;
    vmap_extract.parse_map_files(first_installed_locale, &storage)?;

    bincode::serialize_into(fs::File::create(model_spawns_tmp)?, &vmap_extract.model_spawns_data)?;
    bincode::serialize_into(fs::File::create(gameobject_models_tmp)?, &vmap_extract.temp_gameobject_models)?;

    // TODO: hirogoro@04jul2023 - VMAP extraction caching (i.e. how not to do more work)
    // 1. save model_spawns_data and temp_gameobject_models to files (similar to
    // `dir_bin` and `temp_gameobject_models` resp in TC),
    // use them as indications that the files are extracted
    // 2. Open via existing map files if possible? - dont rely on CASC storage too much. can probably extract adt and wdt files
    info!("Extract VMAP done!");
    Ok(vmap_extract)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TempGameObjectModel {
    pub id:        u32,
    pub is_wmo:    bool,
    pub file_name: String,
}

struct WdtWithAdts {
    _wdt:      WDTFile,
    adt_cache: Vec<Vec<Option<AdtWithDirFileCache>>>,
}

struct AdtWithDirFileCache {
    adt:            ADTFile,
    dir_file_cache: Vec<VmapModelSpawn>,
}

impl VmapExtractor {
    fn extract_game_object_models(&mut self, storage: &CascStorageHandle, locale: Locale) -> GenericResult<()> {
        info!("Extracting GameObject models...");

        let source = storage.open_file("DBFilesClient/GameObjectDisplayInfo.db2", CascLocale::None.into())?;
        let db2 = wdc1::FileLoader::<GameObjectDisplayInfo>::from_reader(source, locale as u32)?;
        let recs = db2.produce_data()?;

        for (_, rec) in recs {
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
                b"REVM" => self.extract_single_wmo(storage, &file_name).map(|_| true),
                b"MD21" => self.extract_single_model(storage, &file_name).map(|_| false),
                c => {
                    let magic = String::from_utf8_lossy(c);
                    let e = Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        format!("File name {file_name} has unexpected header {magic}",).as_str(),
                    ));
                    return Err(e);
                },
            };
            let is_wmo = match res {
                Err(e) => {
                    warn!("ERROR Extracting single model/single wmo: {e}");
                    continue;
                },
                Ok(b) => b,
            };
            self.temp_gameobject_models.push(TempGameObjectModel { id: rec.id, is_wmo, file_name });
        }
        info!("Done!");

        Ok(())
    }

    fn extract_single_wmo(&mut self, storage: &CascStorageHandle, file_name: &str) -> GenericResult<()> {
        let plain_name = get_fixed_plain_name(file_name);
        let sz_local_file = self.args.output_vmap_sz_work_dir_wmo().join(&plain_name);
        if let Some((_prefix, rest)) = plain_name.rsplit_once('_') {
            if rest.len() == 3 && rest.chars().all(|c| c.is_ascii_digit()) {
                // i.e. the rest after '_' are all digits => not root wmo files, return
                return Ok(());
            }
        }
        let file_exist = sz_local_file.exists();
        if !file_exist {
            info!("Extracting to vmap: {}", file_name);
        }
        let mut froot = WmoRoot::build(storage, file_name)?;
        let mut valid_doodad_names = HashSet::new();
        for (doodad_name_index, s) in froot.doodad_data.paths.iter() {
            if self.extract_single_model(storage, s).is_ok() {
                valid_doodad_names.insert(*doodad_name_index);
            }
        }
        froot.init_wmo_groups(storage, valid_doodad_names)?;
        if !file_exist {
            // save only if not exist. The above code is also to ensure that the model spawns are always idempotent.
            let mut output = fs::File::create(&sz_local_file).inspect_err(|e| {
                error!("can't create the output file '{}' for writing, err was: {}", sz_local_file.display(), e);
            })?;
            let vmap = froot.convert_to_vmap(self.args.vmap_extract_and_generate.precise_vector_data);
            vmap.write(&mut output)?;
        }
        self.wmo_doodads.insert(plain_name, froot.doodad_data);
        Ok(())
    }

    fn extract_single_model(&mut self, storage: &CascStorageHandle, file_name: &str) -> GenericResult<()> {
        if file_name.len() < 4 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("File name {file_name} has unexpected length",).as_str(),
            )));
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
        let sz_local_file = self.args.output_vmap_sz_work_dir_wmo().join(plain_name);
        if sz_local_file.exists() {
            return Ok(());
        }

        let mdl = Model::build(storage, file_name)?;
        let vmap = mdl.convert_to_vmap();
        let mut output = fs::File::create(&sz_local_file).inspect_err(|e| {
            error!("can't create the output file '{}' for writing, err was: {}", sz_local_file.display(), e);
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
    fn get_or_extract_wdt<'a>(&mut self, map: &Map, storage: &CascStorageHandle, wdts: &'a mut HashMap<u32, WdtWithAdts>) -> Option<&'a mut WdtWithAdts> {
        if wdts.contains_key(&map.id) {
            return wdts.get_mut(&map.id);
        }
        let map_name = map.directory.def_str();
        let storage_path = format!("World/Maps/{map_name}/{map_name}.wdt");
        let wdt = match WDTFile::build(storage, &storage_path) {
            Err(_e) => {
                warn!("unable to open WDT file {storage_path}: {_e}");
                return None;
            },
            Ok(wdt) => wdt,
        };
        // do some extraction also
        let mut wmo_names = HashMap::with_capacity(wdt.wmo_paths.len());
        for (wmid, path) in &wdt.wmo_paths {
            wmo_names.entry(*wmid).or_insert(get_fixed_plain_name(path));
            if self.extract_single_wmo(storage, path).is_err() {
                continue;
            }
        }
        let mut dir_file_cache = Vec::new();
        // global wmo instance data
        if let Some(modf) = &wdt.modf {
            for map_obj_def in &modf.map_object_defs {
                if map_obj_def.flags & 0x8 == 0 {
                    let name = wmo_names
                        .get(&(map_obj_def.id as usize))
                        .unwrap_or_else(|| panic!("name_id {} should exist in {:?}", map_obj_def.id, wmo_names));
                    _ = self
                        .mapobject_extract(map_obj_def, name, true, map.id, map.id, &mut dir_file_cache)
                        .inspect_err(|e| {
                            warn!("get_or_extract_wdt mapobject_extract err for name {name} due to err: {e}");
                        });
                    _ = self
                        .doodad_extractset(name, map_obj_def, true, map.id, map.id, &mut dir_file_cache)
                        .inspect_err(|e| {
                            warn!("get_or_extract_wdt doodad_extractset err for name {name} due to err: {e}");
                        });
                } else {
                    let fid = map_obj_def.id;
                    let filename = format!("FILE{fid:08X}.xxx");
                    _ = self.extract_single_wmo(storage, &filename).inspect_err(|e| {
                        warn!("get_or_extract_wdt extract_single_wmo err for fid {filename} due to err: {e}");
                    });
                    _ = self
                        .mapobject_extract(map_obj_def, &filename, true, map.id, map.id, &mut dir_file_cache)
                        .inspect_err(|e| {
                            warn!("get_or_extract_wdt doodad_extractset err for fid {filename} due to err: {e}");
                        });
                    _ = self
                        .doodad_extractset(&filename, map_obj_def, true, map.id, map.id, &mut dir_file_cache)
                        .inspect_err(|e| {
                            warn!("get_or_extract_wdt doodad_extractset err for fid {filename} due to err: {e}");
                        });
                }
            }
        };
        for m in dir_file_cache {
            self.model_spawns_data.entry(m.map_num).or_insert(BTreeMap::new()).entry(m.id).or_insert(m);
        }

        let mut adt_cache = Vec::new();
        adt_cache.resize_with(WDT_MAP_SIZE, || {
            let mut v = Vec::new();
            v.resize_with(WDT_MAP_SIZE, || None);
            v
        });
        Some(wdts.entry(map.id).or_insert(WdtWithAdts { _wdt: wdt, adt_cache }))
    }

    fn parse_map_files(&mut self, locale: Locale, storage: &CascStorageHandle) -> GenericResult<()> {
        //xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
        //map.dbc
        info!("Read Map.dbc file...");
        let source = storage.open_file("DBFilesClient/Map.db2", CascLocale::None.into())?;
        let db2 = wdc1::FileLoader::<Map>::from_reader(source, locale as u32)?;
        let maps = db2.produce_data()?;
        info!("Done! ({} maps loaded)", maps.len());

        let mut wdts = HashMap::new();
        for (_map_id, map) in maps.iter() {
            // Populate `wdts` with map_id and parent_map_id's WDT files first
            let map_id = if self.get_or_extract_wdt(map, storage, &mut wdts).is_some() {
                map.id
            } else {
                continue;
            };
            let parent_map_id = maps
                .get(&(map.parent_map_id as u32))
                .and_then(|m| self.get_or_extract_wdt(m, storage, &mut wdts).map(|_w| map.parent_map_id as u32));

            let map_name = map.directory.def_str();
            // After populating, then process ADTs
            info!("Processing Map file {map_id} - {map_name}");
            for x in 0..WDT_MAP_SIZE {
                for y in 0..WDT_MAP_SIZE {
                    let adt = if let Some(r) = self.store_adt_in_wdt(storage, &map_name, map_id, map_id, x, y, &mut wdts) {
                        r
                    } else if let Some(r) =
                        parent_map_id.and_then(|original_map_id| self.store_adt_in_wdt(storage, &map_name, map_id, original_map_id, x, y, &mut wdts))
                    {
                        r
                    } else {
                        continue;
                    };
                    for m in adt.dir_file_cache.iter() {
                        self.model_spawns_data
                            .entry(m.map_num)
                            .or_insert(BTreeMap::new())
                            .entry(m.id)
                            .or_insert(m.clone());
                    }
                }
            }
        }
        Ok(())
    }

    fn generate_unique_object_id(&mut self, client_id: u32, client_doodad_id: u16) -> u32 {
        let next_id = (self.unique_object_id.len() + 1) as u32;
        *self.unique_object_id.entry((client_id, client_doodad_id)).or_insert(next_id)
    }

    fn mapobject_extract(
        &mut self,
        map_obj_def: &AdtMapObjectDefs,
        wmo_inst_name: &str,
        is_global_wmo: bool,
        map_id: u32,
        original_map_id: u32,
        dir_file_cache: &mut Vec<VmapModelSpawn>,
    ) -> GenericResult<()> {
        // destructible wmo, do not dump. we can handle the vmap for these
        // in dynamic tree (gameobject vmaps)
        if (map_obj_def.flags & 0x1) != 0 {
            return Ok(());
        }

        //-----------add_in _dir_file----------------
        let tempname = self.args.output_vmap_sz_work_dir_wmo().join(wmo_inst_name);
        let mut input = fs::File::open(tempname)?;
        let (n_vertices, _, _) = WorldModel_Raw::read_world_model_raw_header(&mut input)?;
        if n_vertices == 0 {
            return Ok(());
        }

        let mut position = fix_coords(&map_obj_def.position);
        let mut bounds = [fix_coords(&map_obj_def.bounds[0]), fix_coords(&map_obj_def.bounds[1])];
        if is_global_wmo {
            position += Vector3::new((1600.0 / 3.0) * 32.0, (1600.0 / 3.0) * 32.0, 0.0);
            bounds[0] += Vector3::new((1600.0 / 3.0) * 32.0, (1600.0 / 3.0) * 32.0, 0.0);
            bounds[1] += Vector3::new((1600.0 / 3.0) * 32.0, (1600.0 / 3.0) * 32.0, 0.0);
        }

        let mut scale = 1.0;
        if map_obj_def.flags & 0x4 > 0 {
            scale = f32::from(map_obj_def.scale) / 1024.0;
        }
        let unique_id = self.generate_unique_object_id(map_obj_def.unique_id, 0);
        let mut flags = ModelFlags::ModHasBound.into();
        if map_id != original_map_id {
            flags |= ModelFlags::ModParentSpawn;
        }

        //write mapID, Flags, name_set, unique_id, Pos, Rot, Scale, Bound_lo, Bound_hi, name
        let m = VmapModelSpawn {
            map_num: map_id,
            flags,
            adt_id: map_obj_def.name_set,
            id: unique_id,
            i_pos: position,
            i_rot: map_obj_def.rotation,
            i_scale: scale,
            bound: Some(bounds),
            name: wmo_inst_name.to_string(),
        };
        dir_file_cache.push(m);
        Ok(())
    }

    fn doodad_extractset(
        &mut self,
        wmo_inst_name: &str,
        wmo: &AdtMapObjectDefs,
        is_global_wmo: bool,
        map_id: u32,
        original_map_id: u32,
        dir_file_cache: &mut Vec<VmapModelSpawn>,
    ) -> GenericResult<()> {
        // Hacky fix for now, we clone this data just for use here.
        //
        // In theory we should be able to get away without cloning but Rust's
        // safety guide-rails makes it hard for us to do so (Without unsafe)
        let doodad_data = self
            .wmo_doodads
            .get(wmo_inst_name)
            .unwrap_or_else(|| panic!("name {} should exist in {:?}", wmo_inst_name, self.wmo_doodads))
            .clone();
        if usize::from(wmo.doodad_set) >= doodad_data.sets.len() {
            return Ok(());
        }

        let mut wmo_position = Vector3::new(wmo.position.z, wmo.position.x, wmo.position.y);
        let wmo_rotation = Rotation::from_euler_angles(wmo.rotation.z.to_radians(), wmo.rotation.x.to_radians(), wmo.rotation.y.to_radians());
        // G3D::Matrix3 wmoRotation = G3D::Matrix3::fromEulerAnglesZYX(G3D::toRadians(wmo.Rotation.y), G3D::toRadians(wmo.Rotation.x), G3D::toRadians(wmo.Rotation.z));

        if is_global_wmo {
            wmo_position += Vector3::new((1600.0 / 3.0) * 32.0, (1600.0 / 3.0) * 32.0, 0.0);
        }

        let mut doodad_id = 0u16;
        let doodad_set_data = &doodad_data.sets[usize::from(wmo.doodad_set)];
        for doodad_index in doodad_data.references.iter() {
            if u32::from(*doodad_index) < doodad_set_data.start_index || u32::from(*doodad_index) >= doodad_set_data.start_index + doodad_set_data.count {
                continue;
            }

            let doodad = &doodad_data.spawns[usize::from(*doodad_index)];

            let model_inst_name = doodad_data
                .paths
                .get(&(doodad.name_index as usize))
                .map(|p| {
                    let plain_name = get_fixed_plain_name(p);
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
                    plain_name.to_string_lossy().to_string()
                })
                .unwrap_or_else(|| panic!("doodad.name_index {} should exist in {:?}", doodad.name_index, doodad_data.paths));
            let tempname = self.args.output_vmap_sz_work_dir_wmo().join(&model_inst_name);
            let mut input = fs::File::open(tempname)?;
            let (n_vertices, _, _) = WorldModel_Raw::read_world_model_raw_header(&mut input)?;
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
            let (z, x, y) = UnitQuaternion::from_quaternion(Quaternion::from(doodad.rotation))
                .to_rotation_matrix()
                .euler_angles();
            let rotation = Vector3::new(x.to_degrees(), y.to_degrees(), z.to_degrees());

            let name_set = 0; // not used for models
            let unique_id = self.generate_unique_object_id(wmo.unique_id, doodad_id);
            let mut tcflags = ModelFlags::ModM2.into();
            if map_id != original_map_id {
                tcflags |= ModelFlags::ModParentSpawn;
            }

            //write mapID, Flags, name_set, unique_id, Pos, Rot, Scale, name
            let m = VmapModelSpawn {
                map_num: map_id,
                flags:   tcflags,
                adt_id:  name_set, // not used for models
                id:      unique_id,
                i_pos:   position,
                i_rot:   rotation,
                i_scale: doodad.scale,
                bound:   None,
                name:    model_inst_name,
            };
            dir_file_cache.push(m);
        }
        Ok(())
    }

    fn doodad_extract(
        &mut self,
        doodad_def: &AdtDoodadDef,
        model_inst_name: &str,
        map_id: u32,
        original_map_id: u32,
        dir_file_cache: &mut Vec<VmapModelSpawn>,
    ) -> GenericResult<()> {
        let tempname = self.args.output_vmap_sz_work_dir_wmo().join(model_inst_name);
        let mut input = fs::File::open(tempname)?;
        let (n_vertices, _, _) = WorldModel_Raw::read_world_model_raw_header(&mut input)?;
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
        let m = VmapModelSpawn {
            map_num: map_id,
            flags,
            adt_id: 0, // not used for models
            id: self.generate_unique_object_id(doodad_def.unique_id, 0),
            i_pos: position,
            i_rot: doodad_def.rotation,
            i_scale: sc,
            bound: None,
            name: model_inst_name.to_string(),
        };
        dir_file_cache.push(m);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn store_adt_in_wdt<'a>(
        &mut self,
        storage: &CascStorageHandle,
        map_name: &str,
        map_id: u32,
        original_map_id: u32,
        x: usize,
        y: usize,
        wdts: &'a mut HashMap<u32, WdtWithAdts>,
    ) -> Option<&'a mut AdtWithDirFileCache> {
        let wdt = match wdts.get_mut(&original_map_id) {
            None => return None,
            Some(wdt) => wdt,
        };
        let mut is_first_set = false;
        let storage_path = format!("World/Maps/{map_name}/{map_name}_{x}_{y}_obj0.adt");
        if wdt.adt_cache[x][y].is_none() {
            wdt.adt_cache[x][y] = ADTFile::build(storage, &storage_path)
                // .inspect_err(|e| {
                //     warn!("Unable to get ADT file {storage_path} with warn: {e}, moving on");
                // })
                .ok()
                .map(|adt| AdtWithDirFileCache { adt, dir_file_cache: vec![] });
            // The first time see this ADT. Do some extraction
            is_first_set = true;
        }

        if is_first_set {
            if let Some(adt) = wdt.adt_cache[x][y].as_mut() {
                // Do some extraction here as well.
                let mut model_instance_names = HashMap::with_capacity(adt.adt.model_paths.len());
                let mut wmo_instance_names = HashMap::with_capacity(adt.adt.wmo_paths.len());
                for (off, path) in adt.adt.model_paths.iter() {
                    model_instance_names.entry(*off).or_insert(get_fixed_plain_name(path));
                    _ = self.extract_single_model(storage, path).inspect_err(|e| {
                        warn!("store_adt_in_wdt extract_single_model err for path {path} due to err: {e}");
                    });
                }
                for (off, path) in adt.adt.wmo_paths.iter() {
                    wmo_instance_names.entry(*off).or_insert(get_fixed_plain_name(path));
                    _ = self.extract_single_wmo(storage, path).inspect_err(|e| {
                        warn!("store_adt_in_wdt extract_single_wmo err for path {path} due to err: {e}");
                    });
                }

                if let Some(mddf) = &adt.adt.mddf {
                    for doodad_def in mddf.doodad_defs.iter() {
                        if doodad_def.flags & 0x40 == 0 {
                            let name = model_instance_names
                                .get(&(doodad_def.id as usize))
                                .unwrap_or_else(|| panic!("name_id {} should exist in {:?}", doodad_def.id, model_instance_names,));
                            _ = self
                                .doodad_extract(doodad_def, name, map_id, original_map_id, &mut adt.dir_file_cache)
                                .inspect_err(|e| {
                                    warn!("store_adt_in_wdt doodad_extract err for name {name} due to err: {e}");
                                });
                        } else {
                            let fid = doodad_def.id;
                            let filename = format!("FILE{fid:08X}.xxx");
                            _ = self.extract_single_model(storage, &filename).inspect_err(|e| {
                                warn!("store_adt_in_wdt extract_single_model err for fid {filename} due to err: {e}");
                            });
                            _ = self
                                .doodad_extract(doodad_def, &filename, map_id, original_map_id, &mut adt.dir_file_cache)
                                .inspect_err(|e| {
                                    warn!("store_adt_in_wdt doodad_extract err for fid {filename} due to err: {e}");
                                });
                        }
                    }
                }
                if let Some(modf) = &adt.adt.modf {
                    for map_obj_def in modf.map_object_defs.iter() {
                        if map_obj_def.flags & 0x8 == 0 {
                            let wmo_inst_name = wmo_instance_names
                                .get(&(map_obj_def.id as usize))
                                .unwrap_or_else(|| panic!("name_id {} should exist in {:?}", map_obj_def.id, model_instance_names));
                            _ = self
                                .mapobject_extract(map_obj_def, wmo_inst_name, false, map_id, original_map_id, &mut adt.dir_file_cache)
                                .inspect_err(|e| {
                                    warn!("store_adt_in_wdt mapobject_extract err for wmo_inst_name {wmo_inst_name} due to err: {e}");
                                });
                            _ = self
                                .doodad_extractset(wmo_inst_name, map_obj_def, false, map_id, original_map_id, &mut adt.dir_file_cache)
                                .inspect_err(|e| {
                                    warn!("store_adt_in_wdt doodad_extractset err for wmo_inst_name {wmo_inst_name} due to err: {e}");
                                });
                        } else {
                            let fid = map_obj_def.id;
                            let filename = format!("FILE{fid:08X}.xxx");
                            _ = self.extract_single_wmo(storage, &filename).inspect_err(|e| {
                                warn!("store_adt_in_wdt extract_single_wmo err for fid {filename} due to err: {e}");
                            });
                            _ = self
                                .mapobject_extract(map_obj_def, &filename, false, map_id, original_map_id, &mut adt.dir_file_cache)
                                .inspect_err(|e| {
                                    warn!("store_adt_in_wdt mapobject_extract err for fid {filename} due to err: {e}");
                                });
                            _ = self
                                .doodad_extractset(&filename, map_obj_def, false, map_id, original_map_id, &mut adt.dir_file_cache)
                                .inspect_err(|e| {
                                    warn!("store_adt_in_wdt doodad_extractset err for fid {filename} due to err: {e}");
                                });
                        }
                    }
                }
            }
        }

        wdt.adt_cache[x][y].as_mut()
    }
}

fn fix_coords(v: &Vector3<f32>) -> Vector3<f32> {
    Vector3::new(v.z, v.x, v.y)
}
