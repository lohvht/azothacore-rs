use std::{
    collections::HashMap,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use bevy::{
    app::{App, Startup},
    asset::{io::Reader as BevyAssetIoReader, AssetApp, AssetLoader, AssetServer, Assets, AsyncReadExt, LoadContext},
    ecs::system::SystemParam,
    prelude::{EventWriter, Handle, IntoSystemConfigs, Res, ResMut, Resource, SystemSet},
};
use flagset::FlagSet;
use tracing::{debug, error, instrument};

use crate::{
    bevy_app::{az_startup_succeeded, AzStartupFailedEvent},
    collision::{management::VMapMgr, maps::map_tree::StaticMapTree, models::world_model::WorldModel},
    configuration::ConfigMgr,
    deref_boilerplate,
    utils::buffered_file_open,
    AzError,
    ChildMapData,
    MapLiquidTypeFlag,
    ParentMapData,
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VMapManager2InitSet;

pub fn vmap_mgr2_plugin<C, L, V>(app: &mut App)
where
    C: VmapConfig,
    L: LiquidFlagsGetter,
    V: VmapDisabledChecker,
{
    app.insert_resource(VmapInstanceMapTrees(HashMap::default()))
        .init_asset::<WorldModel>()
        .init_asset_loader::<WorldModelAssetLoader>()
        .add_systems(
            Startup,
            vmap_mgr2_init_check::<C, L, V>.run_if(az_startup_succeeded()).in_set(VMapManager2InitSet),
        );
}

fn vmap_mgr2_init_check<C: VmapConfig, L: LiquidFlagsGetter, V: VmapDisabledChecker>(
    cfg: Option<Res<ConfigMgr<C>>>,
    liq_flags_getter: Option<Res<L>>,
    vmap_disabled_checker: Option<Res<V>>,
    child_map_data: Option<Res<ChildMapData>>,
    parent_map_data: Option<Res<ParentMapData>>,
    mut ev_startup_failed: EventWriter<AzStartupFailedEvent>,
) {
    let mut missing_resources = vec![];
    if cfg.is_none() {
        missing_resources.push("ConfigMgr");
    }
    if liq_flags_getter.is_none() {
        missing_resources.push("LiquidFlagsGetter");
    }
    if vmap_disabled_checker.is_none() {
        missing_resources.push("VMapDisableChecker");
    }
    if child_map_data.is_none() {
        missing_resources.push("ChildMapData");
    }
    if parent_map_data.is_none() {
        missing_resources.push("ParentMapData");
    }

    if !missing_resources.is_empty() {
        ev_startup_failed.send_default();
        error!(
            missing = ?missing_resources,
            "required resources needed for Vmap management. Possible programming error?"
        );
    }
}

pub trait VmapConfig: Send + Sync + 'static {
    fn vmaps_dir(&self) -> PathBuf;

    fn enable_line_of_sight_calc(&self) -> bool {
        true
    }
    fn enable_height_calc(&self) -> bool {
        true
    }
}

pub trait LiquidFlagsGetter: Resource {
    fn get_liquid_flags(&self, _liquid_type_id: u32) -> FlagSet<MapLiquidTypeFlag> {
        None.into()
    }
}

pub trait VmapDisabledChecker: Resource {
    /// DisableMgr::IsVMAPDisabledFor in TC/AC
    fn is_vmap_disabled_for(&self, _entry: u32, _flags: u8) -> bool {
        false
    }
}

/// The resource that contains vmap management stuff
#[derive(SystemParam)]
pub struct VMapMgr2<'w, C, L, V>
where
    C: VmapConfig,
    L: LiquidFlagsGetter,
    V: VmapDisabledChecker,
{
    pub helper:         VMapMgr2Helper<'w, C, L, V>,
    /// Tree to check collision
    pub model_store:    VMapModelStore<'w, C>,
    /// iInstanceMapTrees in TC/AC.
    instance_map_trees: ResMut<'w, VmapInstanceMapTrees>,
}

#[derive(Resource)]
struct VmapInstanceMapTrees(HashMap<u32, StaticMapTree>);

deref_boilerplate!(VmapInstanceMapTrees, HashMap<u32, StaticMapTree>, 0);

/// Helper for vmapmgr2 => Generally contains things that don't require exclusive (i.e. mutable) access
#[derive(SystemParam)]
pub struct VMapMgr2Helper<'w, C, L, V>
where
    C: VmapConfig,
    L: LiquidFlagsGetter,
    V: VmapDisabledChecker,
{
    pub cfg_mgr:             Res<'w, ConfigMgr<C>>,
    pub liquid_flags_getter: Res<'w, L>,
    _vmap_disabled_checker:  Res<'w, V>,
    /// Child map data, containings map_ids to their children IDs.
    _child_map_data:         Res<'w, ChildMapData>,
    /// Parent map data, containings map_ids to their parent ID.
    pub parent_map_data:     Res<'w, ParentMapData>,
}

#[derive(SystemParam)]
pub struct VMapModelStore<'w, C>
where
    C: VmapConfig,
{
    cfg_mgr:                Res<'w, ConfigMgr<C>>,
    pub loaded_model_files: ResMut<'w, Assets<WorldModel>>,
    asset_loader:           Res<'w, AssetServer>,
}

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

impl<C> VMapModelStore<'_, C>
where
    C: VmapConfig,
{
    /// acquireModelInstance in TC/AC
    pub fn acquire_model_instance(&mut self, filename: &str, load_async: bool) -> Option<Handle<WorldModel>> {
        let path = push_extension(self.cfg_mgr.vmaps_dir().join(filename), "vmo");
        if load_async {
            Some(self.asset_loader.load(path))
        } else if let Some(h) = self.asset_loader.get_handle(path.to_path_buf()) {
            Some(h)
        } else {
            match buffered_file_open(&path) {
                Err(e) => {
                    error!("misc: VMapMgr2: could not load {}; err {e}", path.display());
                    None
                },
                Ok(mut f) => match WorldModel::read_file(&mut f) {
                    Ok(m) => Some(self.loaded_model_files.add(m)),
                    Err(e) => {
                        debug!("error trying to open world model file: {filename}; e: {e}");
                        None
                    },
                },
            }
        }
    }
}

#[derive(Default)]
pub struct WorldModelAssetLoader;

impl AssetLoader for WorldModelAssetLoader {
    type Asset = WorldModel;
    type Error = AzError;
    type Settings = ();

    async fn load<'a>(
        &'a self,
        reader: &'a mut BevyAssetIoReader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        let filename = load_context.asset_path();
        reader
            .read_to_end(&mut bytes)
            .await
            .inspect_err(|e| error!("misc: could not load world model {filename}; err {e}"))?;
        let mut cursor = io::Cursor::new(bytes.as_slice());
        WorldModel::read_file(&mut cursor).inspect_err(|e| debug!("error trying to open world model file: {filename}; e: {e}"))
    }

    fn extensions(&self) -> &[&str] {
        &["vmo"]
    }
}

impl<C, L, V> VMapMgr2<'_, C, L, V>
where
    C: VmapConfig,
    L: LiquidFlagsGetter,
    V: VmapDisabledChecker,
{
    /// load one tile (internal use only)
    /// loadSingleMap in TC / _loadMap or loadMap in AC
    ///
    #[instrument(skip_all, fields(tile = format!("[Map {map_id:04}] [{tile_x:02},{tile_y:02}]")))]
    pub fn load_single_map_tile(&mut self, map_id: u32, tile_x: u16, tile_y: u16, load_async: bool) -> super::VmapFactoryLoadResult<&StaticMapTree> {
        let tree = match self.instance_map_trees.get_mut(&map_id) {
            None => {
                let t = StaticMapTree::init_from_file(self.helper.cfg_mgr.vmaps_dir(), map_id)
                    .map_err(|e| super::VmapFactoryLoadError::General(format!("error loading map tree: {}", e)))?;

                self.instance_map_trees.entry(map_id).or_insert(t)
            },
            Some(i) => i,
        };

        tree.load_map_tile(tile_x, tile_y, &self.helper, &mut self.model_store, load_async)
            .map_err(|e| super::VmapFactoryLoadError::General(format!("error loading map tile: {}", e)))?;

        let tree = self
            .instance_map_trees
            .get(&map_id)
            .expect("after successful load map tile the tree must definitely exist");
        Ok(tree)
    }

    // unloadSingleMap in TC
    pub fn unload_single_map_tile(&mut self, map_id: u32, tile_x: u16, tile_y: u16) {
        let remove_tree = if let Some(instance_tree) = self.instance_map_trees.get_mut(&map_id) {
            instance_tree.unload_map_tile(tile_x, tile_y, &self.helper);
            instance_tree.has_no_loaded_values()
        } else {
            false
        };

        if remove_tree {
            self.instance_map_trees.remove(&map_id);
        }
    }

    pub fn instance_map_tree(&self, map_id: u32) -> Option<&StaticMapTree> {
        self.instance_map_trees.get(&map_id)
    }
}

impl<C, L, V> VMapMgr for VMapMgr2<'_, C, L, V>
where
    C: VmapConfig,
    L: LiquidFlagsGetter,
    V: VmapDisabledChecker,
{
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
        // self.get_liquid_level(p_map_id, x, y, z, req_liquid_type, level, floor, typ)
        // let _a = (self.get_liquid_flags)(3);
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
