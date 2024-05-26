use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt::Display,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use num::Num;
use parry3d::partitioning::Qbvh;
use tracing::{debug, error};

use crate::{
    az_error,
    cmp_or_return,
    collision::{
        management::vmap_mgr2::VMapModelStore,
        models::model_instance::{ModelInstance, VmapModelSpawn},
        vmap_definitions::VMAP_MAGIC,
    },
    sanity_check_read_all_bytes_from_reader,
    utils::{bincode_deserialise, bincode_serialise, buffered_file_create, buffered_file_open},
    AzResult,
};

fn file_stem_if_ext_matched<P: AsRef<Path>>(p: P, extension_to_match: &str) -> AzResult<String> {
    let file_stem = match p.as_ref().extension() {
        Some(ext) if ext != extension_to_match => {
            return Err(az_error!("Path has unexpected extension_to_match; p was {}", p.as_ref().display()));
        },
        Some(_ext) => match p.as_ref().with_extension("").file_stem().and_then(OsStr::to_str) {
            None => {
                return Err(az_error!("Path does not have a file_stem; p was {}", p.as_ref().display()));
            },
            Some(s) => s.to_string(),
        },
        None => {
            return Err(az_error!("Path does not have an extension_to_match; p was {}", p.as_ref().display()));
        },
    };
    Ok(file_stem)
}

pub struct StaticMapTree {
    map_id:        u32,
    _tree:         Qbvh<usize>,
    /// the tree entries
    tree_values:   HashMap<usize, Arc<ModelInstance>>,
    /// mapping between spawn IDs and BH indices
    spawn_indices: HashMap<u32, usize>,
    /// Store all the map tile idents that are loaded for that map
    /// some maps are not splitted into tiles and we have to make sure, not removing the map before all tiles are removed
    /// empty tiles have no tile file, hence map with bool instead of just a set (consistency check)
    loaded_tiles:  HashMap<(u16, u16), Vec<Arc<ModelInstance>>>,
    // std::vector<std::pair<int32, int32>> iLoadedPrimaryTiles;
    base_path:     PathBuf,
}

impl StaticMapTree {
    // equivalent of InitMap in TC
    pub fn init_from_file<P: AsRef<Path>>(base_path: P, map_id: u32) -> AzResult<Self> {
        let fname = StaticMapTree::map_file_name(base_path.as_ref(), map_id);
        debug!("StaticMapTree::InitMap() : initializing StaticMapTree '{}'", fname.display());

        let mut input = buffered_file_open(&fname)?;

        Self::init_from_reader(base_path.as_ref(), map_id, &mut input)
    }

    fn init_from_reader<P: AsRef<Path>, R: io::Read>(base_path: P, map_id: u32, r: &mut R) -> AzResult<Self> {
        let mut r = r;
        let (_tree, spawn_indices) = Self::read_map_tree(&mut r)?;
        Ok(Self {
            base_path: base_path.as_ref().to_owned(),
            map_id,
            _tree,
            tree_values: HashMap::new(),
            spawn_indices,
            loaded_tiles: HashMap::new(),
        })
    }

    pub fn load_map_tile(
        &mut self,
        tile_x: u16,
        tile_y: u16,
        parent_map_data: Arc<HashMap<u32, u32>>,
        model_store: Arc<Mutex<VMapModelStore>>,
    ) -> AzResult<()> {
        let packed_id = (tile_x, tile_y);
        let TileFileOpenResult {
            spawns,
            name: file_result_name,
            used_map_id,
        } = match Self::open_map_tile_spawns_file(&self.base_path, self.map_id, tile_x, tile_y, parent_map_data) {
            Err(e) => {
                debug!(
                    "Error opening map tile, map may or may not have a map tile - map_id {} [x:{}, y:{}] err {e}",
                    self.map_id, tile_x, tile_y
                );
                self.loaded_tiles.insert(packed_id, vec![]);
                // TC_METRIC_EVENT("map_events", "LoadMapTile",
                // "Map: " + std::to_string(iMapID) + " TileX: " + std::to_string(tileX) + " TileY: " + std::to_string(tileY));
                return Err(e);
            },
            Ok(s) => s,
        };

        let mut result = Ok(());
        let loaded_tile_spawns = self.loaded_tiles.entry(packed_id).or_insert(Vec::with_capacity(spawns.len()));
        for spawn in spawns {
            // update tree
            if let Some(reference_val) = self.spawn_indices.get(&spawn.id) {
                if let Some(m) = self.tree_values.get_mut(reference_val) {
                    loaded_tile_spawns.push(m.clone());
                } else {
                    // acquire model instance
                    let model = match model_store.lock().unwrap().acquire_model_instance(&self.base_path, &spawn.name) {
                        None => {
                            error!("StaticMapTree::LoadMapTile() : could not acquire WorldModel pointer [{tile_x}, {tile_y}]");
                            continue;
                        },
                        Some(m) => m,
                    };
                    let m = Arc::new(ModelInstance::new(spawn, model));
                    loaded_tile_spawns.push(m.clone());
                    self.tree_values.insert(*reference_val, m);
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

    /// unload_map_tile unloads the map tile. returns if the resultant operation has resulted in no
    /// loaded trees
    pub fn unload_map_tile(&mut self, tile_x: u16, tile_y: u16, model_store: Arc<Mutex<VMapModelStore>>, parent_map_data: Arc<HashMap<u32, u32>>) {
        let tile_id = (tile_x, tile_y);
        // Drop the spawns in `loaded_tiles`
        let had_tile_loaded = match self.loaded_tiles.remove(&tile_id) {
            None => {
                error!(
                    "StaticMapTree::UnloadMapTile() : trying to unload non-loaded tile - Map:{} X:{} Y:{}",
                    self.map_id, tile_x, tile_y
                );
                return;
            },
            Some(v) => !v.is_empty(),
        };
        // file associated with tile
        if had_tile_loaded {
            if let Ok(TileFileOpenResult {
                spawns,
                name: _tile_file_name,
                used_map_id,
            }) = Self::open_map_tile_spawns_file(&self.base_path, self.map_id, tile_x, tile_y, parent_map_data)
            {
                for spawn in spawns {
                    // update tree
                    if let Some(reference_node) = self.spawn_indices.get(&spawn.id) {
                        let count = self.tree_values.get(reference_node).map_or(0, Arc::strong_count);
                        if count == 0 {
                            error!(
                                "misc: StaticMapTree::UnloadMapTile() : trying to unload non-referenced model '{}' (ID:{})",
                                spawn.name, spawn.id,
                            );
                        } else if count == 1 {
                            self.tree_values.remove(reference_node);
                        }
                        // release model instance
                        model_store.lock().unwrap().release_model_instance(&spawn.name);
                    } else if used_map_id == self.map_id {
                        // logic documented in StaticMapTree::LoadMapTile
                        break;
                    }
                }
            }
        }
        // TC_METRIC_EVENT("map_events", "UnloadMapTile",
        //     "Map: " + std::to_string(iMapID) + " TileX: " + std::to_string(tileX) + " TileY: " + std::to_string(tileY));
    }

    pub fn get_tile_model_instances(&self, tile_x: u16, tile_y: u16) -> Vec<Arc<ModelInstance>> {
        self.loaded_tiles.get(&(tile_x, tile_y)).into_iter().flat_map(|v| v.iter().cloned()).collect()
    }

    pub fn has_no_loaded_tiles(&self) -> bool {
        self.loaded_tiles.is_empty()
    }

    pub fn map_file_name<P: AsRef<Path>, M: Num + Display>(dir: P, map_id: M) -> PathBuf {
        dir.as_ref().join(format!("{map_id:04}.vmtree"))
    }

    pub fn map_id_from_map_file_map<P: AsRef<Path>>(p: P) -> AzResult<u32> {
        let map_id = (file_stem_if_ext_matched(p, "vmtree")?).parse::<u32>()?;
        Ok(map_id)
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

    pub fn read_map_tree<R: io::Read>(r: &mut R) -> AzResult<(Qbvh<usize>, HashMap<u32, usize>)> {
        let mut r = r;

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

    fn open_map_tile_spawns_file<P>(dir: P, map_id: u32, x: u16, y: u16, parent_map_data: Arc<HashMap<u32, u32>>) -> AzResult<TileFileOpenResult>
    where
        P: AsRef<Path>,
    {
        let mut tried_map_ids = vec![];
        let mut used_map_id = Some(&map_id);
        while let Some(map_id) = used_map_id {
            let file_name = Self::get_tile_file_name(&dir, *map_id, x, y);
            let spawns = match buffered_file_open(&file_name)
                .map_err(|e| e.into())
                .and_then(|mut f| Self::read_map_tile_spawns(&mut f))
            {
                Err(_) => {
                    tried_map_ids.push(*map_id);
                    used_map_id = parent_map_data.get(map_id);
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
