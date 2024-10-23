use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use num::Num;
use parry3d::partitioning::Qbvh;
use tracing::{debug, error};

use crate::{
    az_error,
    cmp_or_return,
    collision::{
        management::vmap_mgr2::{LiquidFlagsGetter, VMapMgr2Helper, VMapModelStore, VmapConfig, VmapDisabledChecker},
        models::model_instance::{ModelInstance, VmapModelSpawn},
        vmap_definitions::VMAP_MAGIC,
    },
    sanity_check_read_all_bytes_from_reader,
    utils::{bincode_deserialise, bincode_serialise, buffered_file_create, buffered_file_open},
    AzResult,
};

pub struct StaticMapTree {
    map_id:        u32,
    _tree:         Qbvh<usize>,
    /// the tree entries
    tree_values:   HashMap<usize, (ModelInstance, BTreeSet<(u16, u16)>)>,
    /// mapping between spawn IDs and BH indices
    spawn_indices: HashMap<u32, usize>,
}

impl StaticMapTree {
    // equivalent of InitMap in TC
    pub fn init_from_file<P: AsRef<Path>>(base_path: P, map_id: u32) -> AzResult<Self> {
        let fname = Self::map_file_name(base_path.as_ref(), map_id);
        debug!("StaticMapTree::InitMap() : initializing StaticMapTree '{}'", fname.display());

        let mut input = buffered_file_open(&fname)?;

        Self::init_from_reader(map_id, &mut input)
    }

    fn init_from_reader<R: io::Read>(map_id: u32, mut r: &mut R) -> AzResult<Self> {
        let (_tree, spawn_indices) = Self::read_map_tree(&mut r)?;
        Ok(Self {
            map_id,
            _tree,
            tree_values: HashMap::new(),
            spawn_indices,
        })
    }

    /// LoadMapTile in TC / AC
    pub fn load_map_tile<C, L, V>(
        &mut self,
        tile_x: u16,
        tile_y: u16,
        vm_helper: &VMapMgr2Helper<C, L, V>,
        vm_store: &mut VMapModelStore<C>,
        load_async: bool,
    ) -> AzResult<()>
    where
        C: VmapConfig,
        L: LiquidFlagsGetter,
        V: VmapDisabledChecker,
    {
        let packed_id = (tile_x, tile_y);
        let TileFileOpenResult {
            spawns,
            name: file_result_name,
            used_map_id,
        } = match Self::open_map_tile_spawns_file(self.map_id, tile_x, tile_y, vm_helper) {
            Err(e) => {
                debug!(
                    "Error opening map tile, map may or may not have a map tile - map_id {} [x:{}, y:{}] err {e}",
                    self.map_id, tile_x, tile_y
                );
                // TC_METRIC_EVENT("map_events", "LoadMapTile",
                // "Map: " + std::to_string(iMapID) + " TileX: " + std::to_string(tileX) + " TileY: " + std::to_string(tileY));
                return Err(e);
            },
            Ok(s) => s,
        };

        let mut result = Ok(());
        for spawn in spawns {
            // update tree
            if let Some(reference_val) = self.spawn_indices.get(&spawn.id) {
                if let Some((_, loaded_tiles)) = self.tree_values.get_mut(reference_val) {
                    loaded_tiles.insert(packed_id);
                } else {
                    // acquire model instance
                    let Some(model) = vm_store.acquire_model_instance(&spawn.name, load_async) else {
                        continue;
                    };
                    let m = ModelInstance::new(spawn, model);
                    self.tree_values.insert(*reference_val, (m, BTreeSet::from([packed_id])));
                };
            } else if used_map_id == self.map_id {
                // unknown parent spawn might appear in because it overlaps multiple tiles
                // in case the original tile is swapped but its neighbour is now (adding this spawn)
                // we want to not mark it as loading error and just skip that model
                result = Err(az_error!(
                    "StaticMapTree::LoadMapTile() : invalid tree element (spawn {}) referenced in tile {} by map {}",
                    spawn.id,
                    file_result_name.display(),
                    self.map_id
                ));
                break;
            }
        }
        // TC_METRIC_EVENT("map_events", "LoadMapTile",
        //     "Map: " + std::to_string(iMapID) + " TileX: " + std::to_string(tileX) + " TileY: " + std::to_string(tileY));
        result
    }

    /// UnloadMapTile in TC/AC
    /// unload_map_tile unloads the map tile. returns if the resultant operation has resulted in no
    /// loaded trees
    pub fn unload_map_tile<C, L, V>(&mut self, tile_x: u16, tile_y: u16, vm: &VMapMgr2Helper<C, L, V>)
    where
        C: VmapConfig,
        L: LiquidFlagsGetter,
        V: VmapDisabledChecker,
    {
        let tile_id = (tile_x, tile_y);
        // file associated with tile
        if let Ok(TileFileOpenResult {
            spawns,
            name: _tile_file_name,
            used_map_id,
        }) = Self::open_map_tile_spawns_file(self.map_id, tile_x, tile_y, vm)
        {
            for spawn in spawns {
                // update tree
                if let Some(reference_val) = self.spawn_indices.get(&spawn.id) {
                    let mut remove_ref = false;
                    match self.tree_values.get_mut(reference_val) {
                        None => {
                            error!(
                                "misc: StaticMapTree::UnloadMapTile() : trying to unload non-referenced model '{}' (ID:{})",
                                spawn.name, spawn.id,
                            );
                        },
                        Some((_, loaded_tiles)) => {
                            loaded_tiles.remove(&tile_id);
                            if loaded_tiles.is_empty() {
                                remove_ref = true;
                            }
                        },
                    }
                    if remove_ref {
                        self.tree_values.remove(reference_val);
                    }
                } else if used_map_id == self.map_id {
                    // logic documented in StaticMapTree::LoadMapTile
                    break;
                }
            }
        }
        // TC_METRIC_EVENT("map_events", "UnloadMapTile",
        //     "Map: " + std::to_string(iMapID) + " TileX: " + std::to_string(tileX) + " TileY: " + std::to_string(tileY));
    }

    pub fn tile_model_instances(&self, tile_x: u16, tile_y: u16) -> impl Iterator<Item = &ModelInstance> {
        let tile_id = (tile_x, tile_y);
        self.tree_values
            .values()
            .filter_map(move |(e, tiles)| if tiles.contains(&tile_id) { Some(e) } else { None })
    }

    pub fn has_no_loaded_values(&self) -> bool {
        self.tree_values.is_empty()
    }

    pub fn map_file_name<P: AsRef<Path>, M: Num + Display>(dir: P, map_id: M) -> PathBuf {
        dir.as_ref().join(format!("{map_id:04}.vmtree"))
    }

    pub fn get_tile_file_name<P, M, X, Y>(dir: P, map_id: M, x: X, y: Y) -> PathBuf
    where
        P: AsRef<Path>,
        M: Num + Display,
        X: Num + Display,
        Y: Num + Display,
    {
        dir.as_ref().join(format!("{map_id:04}_{y:02}_{x:02}.vmtile"))
    }

    pub fn write_map_tree_to_file<P: AsRef<Path>, M: Num + Display>(
        dir: P,
        map_id: M,
        ptree: &Qbvh<usize>,
        model_spawns_used: &[&VmapModelSpawn],
    ) -> AzResult<()> {
        let mapfilename = Self::map_file_name(dir, map_id);
        let mut mapfile = buffered_file_create(&mapfilename).map_err(|e| {
            error!("cannot open {}, err was: {e}", mapfilename.display());
            e
        })?;

        Self::write_map_tree(&mut mapfile, ptree, model_spawns_used)
    }

    fn write_map_tree<W: io::Write>(w: &mut W, ptree: &Qbvh<usize>, model_spawns_used: &[&VmapModelSpawn]) -> AzResult<()> {
        let mut w = w;
        //general info
        w.write_all(VMAP_MAGIC)?;
        // Nodes
        w.write_all(b"NODE")?;

        bincode_serialise(&mut w, ptree)?;

        // spawn id to index map
        // uint32 map_spawnsSize = map_spawns.size();
        w.write_all(b"SIDX")?;
        let map_spawn_id_to_bvh_id = model_spawns_used.iter().enumerate().map(|(i, m)| (m.id, i)).collect::<Vec<_>>();
        bincode_serialise(w, &map_spawn_id_to_bvh_id)?;
        Ok(())
    }

    pub fn read_map_tree<R: io::Read>(mut r: &mut R) -> AzResult<(Qbvh<usize>, HashMap<u32, usize>)> {
        cmp_or_return!(r, VMAP_MAGIC)?;
        cmp_or_return!(r, b"NODE")?;
        let tree = bincode_deserialise(&mut r)?;
        cmp_or_return!(r, b"SIDX")?;
        let map_spawn_id_to_bvh_id: Vec<(u32, usize)> = bincode_deserialise(&mut r)?;

        let map_spawn_id_to_bvh_id = map_spawn_id_to_bvh_id.into_iter().collect();

        sanity_check_read_all_bytes_from_reader!(r)?;

        Ok((tree, map_spawn_id_to_bvh_id))
    }

    pub fn write_map_tile_spawns_file<P, M, X, Y>(dir: P, map_id: M, x: X, y: Y, model_spawns: &[&VmapModelSpawn]) -> AzResult<()>
    where
        P: AsRef<Path>,
        M: Num + Display,
        X: Num + Display,
        Y: Num + Display,
    {
        let tile_file_name = Self::get_tile_file_name(dir, map_id, x, y);
        let mut tile_file = buffered_file_create(tile_file_name)?;
        Self::write_map_tile_spawns(&mut tile_file, model_spawns)
    }

    fn write_map_tile_spawns<W: io::Write>(w: &mut W, model_spawns: &[&VmapModelSpawn]) -> AzResult<()> {
        let mut w = w;

        w.write_all(VMAP_MAGIC)?;
        bincode_serialise(&mut w, &model_spawns)?;
        Ok(())
    }

    fn open_map_tile_spawns_file<C, L, V>(map_id: u32, x: u16, y: u16, vm: &VMapMgr2Helper<C, L, V>) -> AzResult<TileFileOpenResult>
    where
        C: VmapConfig,
        L: LiquidFlagsGetter,
        V: VmapDisabledChecker,
    {
        let mut tried_map_ids = vec![];
        let mut used_map_id = Some(&map_id);
        while let Some(map_id) = used_map_id {
            let file_name = Self::get_tile_file_name(vm.cfg_mgr.vmaps_dir(), *map_id, x, y);
            let spawns = match buffered_file_open(&file_name)
                .map_err(|e| e.into())
                .and_then(|mut f| Self::read_map_tile_spawns(&mut f))
            {
                Err(_) => {
                    tried_map_ids.push(*map_id);
                    used_map_id = vm.parent_map_data.get(map_id);
                    continue;
                },
                Ok(m) => m,
            };
            return Ok(TileFileOpenResult {
                spawns,
                name: file_name,
                used_map_id: *map_id,
            });
        }

        Err(az_error!(
            "error retrieving map tile spawns for the map {map_id:04}[{x},{y}], tried finding from these maps: {tried_map_ids:?}"
        ))
    }

    pub fn read_map_tile_spawns<R: io::Read>(r: &mut R) -> AzResult<Vec<VmapModelSpawn>> {
        let mut r = r;

        cmp_or_return!(r, VMAP_MAGIC)?;
        let res = bincode_deserialise(&mut r)?;

        sanity_check_read_all_bytes_from_reader!(r)?;

        Ok(res)
    }
}

struct TileFileOpenResult {
    spawns:      Vec<VmapModelSpawn>,
    name:        PathBuf,
    used_map_id: u32,
}
// class TC_COMMON_API StaticMapTree
// {
// static LoadResult CanLoadMap(const std::string &basePath, uint32 mapID, uint32 tileX, uint32 tileY, VMapManager2* vm);
//         typedef std::unordered_map<uint32, bool> loadedTileMap;
//         typedef std::unordered_map<uint32, uint32> loadedSpawnMap;
//         private:

//         private:
//             bool getIntersectionTime(const G3D::Ray& pRay, float &pMaxDist, bool pStopAtFirstHit, ModelIgnoreFlags ignoreFlags) const;
//             //bool containsLoadedMapTile(unsigned int pTileIdent) const { return(iLoadedMapTiles.containsKey(pTileIdent)); }
//         public:

//             StaticMapTree(uint32 mapID, const std::string &basePath);
//             ~StaticMapTree();

//             bool isInLineOfSight(const G3D::Vector3& pos1, const G3D::Vector3& pos2, ModelIgnoreFlags ignoreFlags) const;
//             bool getObjectHitPos(const G3D::Vector3& pos1, const G3D::Vector3& pos2, G3D::Vector3& pResultHitPos, float pModifyDist) const;
//             float getHeight(const G3D::Vector3& pPos, float maxSearchDist) const;
//             bool getAreaInfo(G3D::Vector3 &pos, uint32 &flags, int32 &adtId, int32 &rootId, int32 &groupId) const;
//             bool GetLocationInfo(const G3D::Vector3 &pos, LocationInfo &info) const;

//             bool InitMap(std::string const& fname);
//             void UnloadMap(VMapManager2* vm);
//             uint32 numLoadedTiles() const { return uint32(iLoadedTiles.size()); }

//         private:
//             StaticMapTree(StaticMapTree const& right) = delete;
//             StaticMapTree& operator=(StaticMapTree const& right) = delete;
// };
