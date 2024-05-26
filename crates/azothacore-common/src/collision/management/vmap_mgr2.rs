use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use flagset::FlagSet;
use tracing::{debug, error, instrument};

use crate::{
    collision::{
        management::VMapMgrTrait,
        maps::map_tree::StaticMapTree,
        models::{model_instance::ModelInstance, world_model::WorldModel},
    },
    deref_boilerplate,
    utils::buffered_file_open,
    MapLiquidTypeFlag,
};

pub type VmapInstanceMapTrees = HashMap<u32, Option<Arc<RwLock<StaticMapTree>>>>;

pub struct VMapMgr2<'liq, 'vd> {
    enable_line_of_sight_calc: bool,
    enable_height_calc:        bool,
    pub get_liquid_flags:      Arc<dyn Fn(u32) -> FlagSet<MapLiquidTypeFlag> + Send + Sync + 'liq>,
    pub is_vmap_disabled_for:  Arc<dyn Fn(u32, u8) -> bool + Send + Sync + 'vd>,

    /// the caller must pass the list of all mapIds that will be used in the VMapManager2 lifetime

    /// Tree to check collision
    model_store:        Arc<Mutex<VMapModelStore>>,
    /// Child map data, containings map_ids to their children IDs.
    child_map_data:     HashMap<u32, Vec<u32>>,
    /// Parent map data, containings map_ids to their parent ID.
    parent_map_data:    Arc<HashMap<u32, u32>>,
    instance_map_trees: Arc<RwLock<VmapInstanceMapTrees>>,
}

impl<'liq, 'vd> Default for VMapMgr2<'liq, 'vd> {
    fn default() -> Self {
        let mut s = Self {
            enable_line_of_sight_calc: true,
            enable_height_calc:        true,
            get_liquid_flags:          Arc::new(|_| None.into()),
            is_vmap_disabled_for:      Arc::new(|_, _| false),
            child_map_data:            HashMap::new(),
            parent_map_data:           Arc::new(HashMap::new()),
            instance_map_trees:        Arc::new(RwLock::new(HashMap::new())),
            model_store:               Default::default(),
        };
        s.init_new();
        s
    }
}

#[derive(Default)]
pub struct VMapModelStore {
    /// Tree to check collision
    loaded_model_files: HashMap<String, Arc<WorldModel>>,
}

deref_boilerplate!(VMapModelStore, HashMap<String, Arc<WorldModel>>, loaded_model_files);

/// pushes an extension to the path, making `ext` the new extension
fn push_extension<P: AsRef<Path>, E: AsRef<OsStr>>(path: P, ext: E) -> PathBuf {
    let path = path.as_ref();
    match path.extension() {
        None => path.with_extension(ext),
        Some(existing_ext) => {
            let mut existing_ext = existing_ext.to_os_string();
            existing_ext.push(".");
            existing_ext.push(ext);
            path.with_extension(existing_ext)
        },
    }
}

impl VMapModelStore {
    pub fn acquire_model_instance<P: AsRef<Path>>(&mut self, base_path: P, filename: &str) -> Option<Arc<WorldModel>> {
        if let Some(model) = self.loaded_model_files.get(filename) {
            Some(model.clone())
        } else {
            let path = push_extension(base_path.as_ref().join(filename), "vmo");
            match buffered_file_open(&path) {
                Err(e) => {
                    error!("misc: VMapMgr2: could not load {}; err {e}", path.display());
                    None
                },
                Ok(mut f) => match WorldModel::read_file(&mut f) {
                    Ok(m) => {
                        let r = self.loaded_model_files.entry(filename.to_string()).or_insert(Arc::new(m));
                        Some(r.clone())
                    },
                    Err(e) => {
                        debug!("error trying to open world model file: {filename}; e: {e}");
                        None
                    },
                },
            }
        }
    }

    pub fn release_model_instance(&mut self, filename: &str) {
        let should_remove = if let Some(m) = self.loaded_model_files.get(filename) {
            Arc::strong_count(m) <= 1
        } else {
            error!("misc: VMapMgr2: trying to unload non-loaded file {filename}");
            return;
        };

        if should_remove {
            debug!("misc: VMapMgr2: unloading file {filename}");
            self.loaded_model_files.remove(filename);
        }
    }
}

impl<'liq, 'vd> VMapMgr2<'liq, 'vd> {
    pub fn set_callbacks(
        &mut self,
        liq_cb: Arc<dyn Fn(u32) -> FlagSet<MapLiquidTypeFlag> + Send + Sync + 'liq>,
        disable_cb: Arc<dyn Fn(u32, u8) -> bool + Send + Sync + 'vd>,
    ) {
        self.get_liquid_flags = liq_cb;
        self.is_vmap_disabled_for = disable_cb;
    }

    /// named InitializeThreadUnsafe in TC.
    /// Initialises the maps that should be loaded / unloaded.
    pub fn set_map_data(&mut self, map_data: &HashMap<u32, Vec<u32>>) {
        self.child_map_data = map_data.clone();
        let mut parent_map_data = HashMap::new();
        for (map_id, children_map_ids) in self.child_map_data.iter() {
            self.instance_map_trees.write().unwrap().entry(*map_id).or_insert(None);
            for child_map_id in children_map_ids.iter() {
                parent_map_data.entry(*child_map_id).or_insert(*map_id);
            }
        }
        self.parent_map_data = Arc::new(parent_map_data);
    }

    #[instrument(skip_all, fields(base_path=format!("{}", base_path.as_ref().display()), map_id = map_id))]
    fn get_or_load_map_tree<P: AsRef<Path>>(&self, map_id: u32, base_path: P) -> super::VmapFactoryLoadResult<Arc<RwLock<StaticMapTree>>> {
        let instance_tree = match self.instance_map_trees.write().unwrap().get_mut(&map_id) {
            None => {
                //TODO: go ahead with the panic for now mimicking `!thread_safe_environment` in TC/ Acore
                // because Map Data map require reading from child trees too.
                panic!("Invalid map_id {map_id} passed to VMapMgr2 after startup");
            },
            Some(it) => {
                if it.is_none() {
                    let new_tree = StaticMapTree::init_from_file(&base_path, map_id)
                        .map_err(|e| super::VmapFactoryLoadError::General(format!("error loading map tree: {}", e)))?;
                    *it = Some(Arc::new(RwLock::new(new_tree)))
                }
                it.clone().unwrap_or_else(|| {
                    panic!(
                        "expect instance tree to be loaded by this point: map {map_id}; path: {}",
                        base_path.as_ref().display(),
                    )
                })
            },
        };
        Ok(instance_tree)
    }

    /// load one tile (internal use only)
    /// loadSingleMap in TC
    #[instrument(skip_all, fields(base_path=format!("{}", base_path.as_ref().display()), tile = format!("[Map {map_id:04}] [{tile_x:02},{tile_y:02}]")))]
    pub fn load_single_map_tile<P: AsRef<Path>>(
        &self,
        map_id: u32,
        base_path: P,
        tile_x: u16,
        tile_y: u16,
    ) -> super::VmapFactoryLoadResult<Vec<Arc<ModelInstance>>> {
        let model_store = self.model_store.clone();
        let parent_map_data = self.parent_map_data.clone();
        let instance_tree = self.get_or_load_map_tree(map_id, base_path)?;

        let mut i_w = instance_tree.write().unwrap();
        i_w.load_map_tile(tile_x, tile_y, parent_map_data, model_store)
            .map_err(|e| super::VmapFactoryLoadError::General(format!("error loading map tile: {}", e)))?;
        Ok(i_w.get_tile_model_instances(tile_x, tile_y))
    }

    // unloadSingleMap in TC
    pub fn unload_single_map_tile(&self, map_id: u32, tile_x: u16, tile_y: u16) {
        let model_store = self.model_store.clone();
        let parent_map_data = self.parent_map_data.clone();
        if let Some(instance_tree) = self.instance_map_trees.write().unwrap().get_mut(&map_id) {
            let remove_tree = if let Some(itree) = instance_tree {
                let mut itree_w = itree.write().unwrap();
                itree_w.unload_map_tile(tile_x, tile_y, model_store, parent_map_data);
                itree_w.has_no_loaded_tiles()
            } else {
                true
            };
            if remove_tree {
                *instance_tree = None;
            }
        }
    }

    pub fn get_parent_map_id(&self, map_id: u32) -> Option<u32> {
        self.parent_map_data.get(&map_id).cloned()
    }
}

impl<'liq, 'vd> VMapMgrTrait for VMapMgr2<'liq, 'vd> {
    // fn as_any(&self) -> &dyn Any {
    //     self
    // }

    // fn as_any_mut(&mut self) -> &mut dyn Any {
    //     self
    // }

    fn init_new_with_options(&mut self, enable_line_of_sight_calc: bool, enable_height_calc: bool) {
        self.enable_line_of_sight_calc = enable_line_of_sight_calc;
        self.enable_height_calc = enable_height_calc;
    }

    fn load_map_tile(&self, _p_base_path: &Path, _p_map_id: u32, _x: u16, _y: u16) -> super::VmapFactoryLoadResult<()> {
        todo!()
    }

    fn exists_map_tile(&self, _p_base_path: &Path, _p_map_id: u32, _x: u16, _y: u16) -> super::VmapLoadResult<()> {
        todo!()
    }

    fn unload_map_tile(&self, _p_map_id: u32, _x: u16, _y: u16) {
        todo!()
    }

    fn unload_map(&self, _p_map_id: u32) {
        todo!()
    }

    fn is_in_line_of_sight(
        &self,
        _p_map_id: u32,
        _x1: f32,
        _y1: f32,
        _z1: f32,
        _x2: f32,
        _y2: f32,
        _z2: f32,
        _ignore_flags: flagset::FlagSet<crate::collision::models::ModelIgnoreFlags>,
    ) -> bool {
        todo!()
    }

    fn get_height(&self, _p_map_id: u32, _x: f32, _y: f32, _z: f32, _max_search_dist: f32) -> f32 {
        todo!()
    }

    fn get_object_hit_pos(
        &self,
        _p_map_id: u32,
        _x1: f32,
        _y1: f32,
        _z1: f32,
        _x2: f32,
        _y2: f32,
        _z2: f32,
        _rx: &mut f32,
        _ry: &mut f32,
        _rz: &mut f32,
        _p_modify_dist: f32,
    ) -> bool {
        let _a = (self.get_liquid_flags)(3);
        todo!()
    }

    fn is_line_of_sight_calc_enabled(&self) -> bool {
        todo!()
    }

    fn is_height_calc_enabled(&self) -> bool {
        todo!()
    }

    fn get_area_info(
        &self,
        _p_map_id: u32,
        _x: f32,
        _y: f32,
        _z: &mut f32,
        _flags: &mut u32,
        _adt_id: &mut u16,
        _root_id: &mut u32,
        _group_id: &mut u32,
    ) -> bool {
        todo!()
    }

    fn get_liquid_level(&self, _p_map_id: u32, _x: f32, _y: f32, _z: f32, _req_liquid_type: u8, _level: &mut f32, _floor: &mut f32, _typ: &mut u32) -> bool {
        todo!()
    }
}
