use std::{
    collections::HashMap,
    fs,
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, Mutex},
};

use flagset::FlagSet;
use tracing::{debug, error};

use crate::{
    common::collision::{management::VMapMgrTrait, maps::map_tree::StaticMapTree, models::world_model::WorldModel},
    server::game::map::MapLiquidTypeFlag,
};

pub struct VMapMgr2<'liq, 'vd> {
    enable_line_of_sight_calc: bool,
    enable_height_calc:        bool,
    pub get_liquid_flags:      Arc<dyn Fn(u32) -> FlagSet<MapLiquidTypeFlag> + Send + Sync + 'liq>,
    pub is_vmap_disabled_for:  Arc<dyn Fn(u32, u8) -> bool + Send + Sync + 'vd>,

    /// the caller must pass the list of all mapIds that will be used in the VMapManager2 lifetime

    /// Tree to check collision
    model_store:            Arc<Mutex<VMapModelStore>>,
    /// Child map data, containings map_ids to their children IDs.
    child_map_data:         HashMap<u32, Vec<u32>>,
    /// Parent map data, containings map_ids to their parent ID.
    parent_map_data:        Arc<HashMap<u32, u32>>,
    ///
    pub instance_map_trees: HashMap<u32, Option<StaticMapTree>>,
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
            instance_map_trees:        HashMap::new(),
            model_store:               Default::default(),
        };
        s.init_new();
        s
    }
}

#[derive(Default)]
pub struct VMapModelStore {
    /// Tree to check collision
    loaded_model_files: HashMap<String, (Arc<WorldModel>, usize)>,
}

impl Deref for VMapModelStore {
    type Target = HashMap<String, (Arc<WorldModel>, usize)>;

    fn deref(&self) -> &Self::Target {
        &self.loaded_model_files
    }
}

impl DerefMut for VMapModelStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.loaded_model_files
    }
}

impl VMapModelStore {
    pub fn acquire_model_instance<P: AsRef<Path>>(&mut self, base_path: P, filename: &str) -> Option<Arc<WorldModel>> {
        if let Some((model, ref_count)) = self.loaded_model_files.get_mut(filename) {
            *ref_count += 1;
            return Some(model.clone());
        };
        let path = base_path.as_ref().join(filename).with_extension("vmo");
        match fs::File::open(&path) {
            Err(e) => {
                error!("misc: VMapMgr2: could not load {}; err {e}", path.display());
                return None;
            },
            Ok(mut f) => match WorldModel::read_file(&mut f) {
                Ok(m) => {
                    self.loaded_model_files.entry(filename.to_string()).or_insert((Arc::new(m), 1));
                },
                Err(e) => {
                    debug!("error trying to open world model file: {filename}; e: {e}");
                },
            },
        };
        self.loaded_model_files.get(filename).map(|(m, _)| m.clone())
    }

    pub fn release_model_instance(&mut self, filename: &str) {
        let should_remove = if let Some((_, ref_count)) = self.loaded_model_files.get_mut(filename) {
            *ref_count -= 1;
            *ref_count == 0
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
            self.instance_map_trees.entry(*map_id).or_insert(None);
            for child_map_id in children_map_ids.iter() {
                parent_map_data.entry(*child_map_id).or_insert(*map_id);
            }
        }
        self.parent_map_data = Arc::new(parent_map_data);
    }

    fn get_or_load_map_tree<P: AsRef<Path>>(&mut self, map_id: u32, base_path: P) -> super::VmapFactoryLoadResult<&mut StaticMapTree> {
        let instance_tree = match self.instance_map_trees.get_mut(&map_id) {
            None => {
                //TODO: go ahead with the panic for now mimicking `!thread_safe_environment` in TC/ Acore
                // because Map Data map require reading from child trees too.
                panic!("Invalid map_id {map_id} passed to VMapMgr2 after startup");
            },
            Some(it) => {
                if it.is_none() {
                    let new_tree = StaticMapTree::init_from_file(&base_path, map_id)
                        .map_err(|e| super::VmapFactoryLoadError::General(e.to_string()))?;
                    *it = Some(new_tree)
                }
                it.as_mut().unwrap_or_else(|| {
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
    pub fn load_single_map_tile<P: AsRef<Path>>(
        &mut self,
        map_id: u32,
        base_path: P,
        tile_x: u16,
        tile_y: u16,
    ) -> super::VmapFactoryLoadResult<()> {
        let model_store = self.model_store.clone();
        let parent_map_data = self.parent_map_data.clone();
        let instance_tree = self.get_or_load_map_tree(map_id, base_path)?;

        instance_tree
            .load_map_tile(tile_x, tile_y, parent_map_data, model_store)
            .map_err(|e| super::VmapFactoryLoadError::General(e.to_string()))?;
        Ok(())
    }

    // unloadSingleMap in TC
    pub fn unload_single_map_tile(&mut self, map_id: u32, tile_x: u16, tile_y: u16) {
        let model_store = self.model_store.clone();
        if let Some(instance_tree) = self.instance_map_trees.get_mut(&map_id) {
            let remove_tree = if let Some(itree) = instance_tree {
                itree.unload_map_tile(tile_x, tile_y, model_store);
                itree.tree_values.is_empty()
            } else {
                true
            };
            if remove_tree {
                self.instance_map_trees.remove(&map_id);
            }
        }
    }

    /// Retieves the parent map ID, if it doesnt exist, return the current map_id
    pub fn get_parent_map_id(&self, map_id: u32) -> u32 {
        self.parent_map_data.get(&map_id).cloned().unwrap_or(map_id)
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

    fn load_map_tile(&self, p_base_path: &Path, p_map_id: u32, x: u16, y: u16) -> super::VmapFactoryLoadResult<()> {
        todo!()
    }

    fn exists_map_tile(&self, p_base_path: &Path, p_map_id: u32, x: u16, y: u16) -> super::VmapLoadResult<()> {
        todo!()
    }

    fn unload_map_tile(&self, p_map_id: u32, x: u16, y: u16) {
        todo!()
    }

    fn unload_map(&self, p_map_id: u32) {
        todo!()
    }

    fn is_in_line_of_sight(
        &self,
        p_map_id: u32,
        x1: f32,
        y1: f32,
        z1: f32,
        x2: f32,
        y2: f32,
        z2: f32,
        ignore_flags: flagset::FlagSet<crate::common::collision::models::ModelIgnoreFlags>,
    ) -> bool {
        todo!()
    }

    fn get_height(&self, p_map_id: u32, x: f32, y: f32, z: f32, max_search_dist: f32) -> f32 {
        todo!()
    }

    fn get_object_hit_pos(
        &self,
        p_map_id: u32,
        x1: f32,
        y1: f32,
        z1: f32,
        x2: f32,
        y2: f32,
        z2: f32,
        rx: &mut f32,
        ry: &mut f32,
        rz: &mut f32,
        p_modify_dist: f32,
    ) -> bool {
        let a = (self.get_liquid_flags)(3);
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
        p_map_id: u32,
        x: f32,
        y: f32,
        z: &mut f32,
        flags: &mut u32,
        adt_id: &mut u16,
        root_id: &mut u32,
        group_id: &mut u32,
    ) -> bool {
        todo!()
    }

    fn get_liquid_level(
        &self,
        p_map_id: u32,
        x: f32,
        y: f32,
        z: f32,
        req_liquid_type: u8,
        level: &mut f32,
        floor: &mut f32,
        typ: &mut u32,
    ) -> bool {
        todo!()
    }
}
