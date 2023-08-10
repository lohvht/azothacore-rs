use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt::Display,
    fs,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use bvh::{bounding_hierarchy::BHShape, bvh::BVH};
use num_traits::Num;
use tracing::{debug, error};

use crate::{
    az_error,
    cmp_or_return,
    common::collision::{
        management::vmap_mgr2::VMapModelStore,
        models::model_instance::{ModelInstance, VmapModelSpawn},
        vmap_definitions::VMAP_MAGIC,
    },
    sanity_check_read_all_bytes_from_reader,
    tools::extractor_common::{bincode_deserialise, bincode_serialise},
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
            return Err(az_error!(
                "Path does not have an extension_to_match; p was {}",
                p.as_ref().display()
            ));
        },
    };
    Ok(file_stem)
}

pub struct StaticMapTree {
    map_id:             u32,
    tree:               BVH,
    /// a tuple containing the tree entry and their respective reference counts
    /// arranged by their respective BH indices
    pub tree_values:    HashMap<usize, (ModelInstance, usize)>,
    pub tile_to_bh_ids: HashMap<u32, Vec<usize>>,

    /// mapping between spawn IDs and BH indices
    spawn_indices: HashMap<u32, usize>,

    // /// iLoadedTiles in TC
    // /// Store all the map tile idents that are loaded for that map
    // /// some maps are not splitted into tiles and we have to make sure, not removing the map before all tiles are removed
    // /// empty tiles have no tile file, hence map with bool instead of just a set (consistency check)
    // /// TODO: Remove me
    // loaded_tiles: HashMap<u32, bool>,

    //             std::vector<std::pair<int32, int32>> iLoadedPrimaryTiles;
    //             // stores <tree_index, reference_count> to invalidate tree values, unload map, and to be able to report errors
    //             loadedSpawnMap iLoadedSpawns;
    base_path: PathBuf,
}

impl StaticMapTree {
    // equivalent of InitMap in TC
    pub fn init_from_file<P: AsRef<Path>>(base_path: P, map_id: u32) -> AzResult<Self> {
        let fname = StaticMapTree::map_file_name(base_path.as_ref(), map_id);
        debug!("StaticMapTree::InitMap() : initializing StaticMapTree '{}'", fname.display());

        let mut input = fs::File::open(&fname)?;

        Self::init_from_reader(base_path.as_ref(), map_id, &mut input)
    }

    fn init_from_reader<P: AsRef<Path>, R: io::Read>(base_path: P, map_id: u32, r: &mut R) -> AzResult<Self> {
        let mut r = r;
        let (tree, spawn_indices) = Self::read_map_tree(&mut r)?;
        let tree_values = HashMap::new();
        // let loaded_tiles = HashMap::new();
        let tile_to_bh_ids = HashMap::new();
        Ok(Self {
            base_path: base_path.as_ref().to_owned(),
            map_id,
            tree,
            tree_values,
            spawn_indices,
            tile_to_bh_ids,
            // loaded_tiles,
        })
    }

    pub fn load_map_tile(
        &mut self,
        tile_x: u16,
        tile_y: u16,
        parent_map_data: Arc<HashMap<u32, u32>>,
        model_store: Arc<Mutex<VMapModelStore>>,
    ) -> AzResult<()> {
        let spawns = match Self::read_map_tile_spawns_file(&self.base_path, self.map_id, tile_y, tile_x, parent_map_data) {
            Err(e) => {
                debug!(
                    "Error opening map tile, map may or may not have a map tile - map_id {} [x:{}, y:{}] err {e}",
                    self.map_id, tile_x, tile_y
                );
                // self.loaded_tiles.insert(packed_id, false);
                // TC_METRIC_EVENT("map_events", "LoadMapTile",
                // "Map: " + std::to_string(iMapID) + " TileX: " + std::to_string(tileX) + " TileY: " + std::to_string(tileY));
                return Err(e);
            },
            Ok(s) => s,
        };

        let packed_id = Self::pack_tile_id(tile_x, tile_y);
        let mut result = Ok(());
        for spawn in spawns {
            // update tree
            let referenced_val = match self.spawn_indices.get(&spawn.id) {
                None => {
                    // spawn index must exist inside spawn indices
                    result = Err(az_error!(
                        "spawn_id in map tree must exist within the spawn indices: spawn_id was {} for spawn name {}",
                        spawn.id,
                        spawn.name,
                    ));
                    break;
                },
                Some(i) => *i,
            };
            if spawn.bh_node_index() != referenced_val {
                error!("StaticMapTree::LoadMapTile() : WorldModel spawn is invalid {} [{tile_x}, {tile_y}]; spawn {} (name: {}) has tree ID of {} but spawn_indices contains {referenced_val}",
                spawn.map_num, spawn.name, spawn.id, spawn.bh_node_index(),
                );
                continue;
            }
            let (tree_val, count) = if let Some(e) = self.tree_values.get_mut(&referenced_val) {
                e
            } else {
                let mut vm = model_store.lock().unwrap();
                // acquire model instance
                let model = match vm.acquire_model_instance(&self.base_path, &spawn.name) {
                    None => {
                        error!("StaticMapTree::LoadMapTile() : could not acquire WorldModel pointer [{tile_x}, {tile_y}]");
                        continue;
                    },
                    Some(m) => m,
                };
                self.tile_to_bh_ids.entry(packed_id).or_default().push(referenced_val);
                self.tree_values
                    .entry(referenced_val)
                    .or_insert((ModelInstance::new(spawn.clone(), model), 0))
            };
            *count += 1;
            if tree_val.id != spawn.id {
                debug!(
                    "StaticMapTree::LoadMapTile() : trying to load wrong spawn in node; spawn_id was {}; tree val was {}",
                    tree_val.id, spawn.id,
                )
            } else if tree_val.name != spawn.name {
                debug!(
                    "StaticMapTree::LoadMapTile() : name collision on GUID {}; name was {} but got {}",
                    spawn.id, tree_val.name, spawn.name,
                )
            }
        }
        // TC_METRIC_EVENT("map_events", "LoadMapTile",
        //     "Map: " + std::to_string(iMapID) + " TileX: " + std::to_string(tileX) + " TileY: " + std::to_string(tileY));
        result
    }

    pub fn unload_map_tile(&mut self, tile_x: u16, tile_y: u16, model_store: Arc<Mutex<VMapModelStore>>) {
        let tile_id = Self::pack_tile_id(tile_x, tile_y);

        let spawn_tree_ids = self.tile_to_bh_ids.entry(tile_id).or_default();
        let mut new_tile_spawn_tree_ids = Vec::with_capacity(spawn_tree_ids.len());
        while let Some(bvh_id) = spawn_tree_ids.pop() {
            let mut has_reference = false;
            if let Some((spawn, reference_counts)) = self.tree_values.get_mut(&bvh_id) {
                // release model instance
                model_store.lock().unwrap().release_model_instance(&spawn.name);

                // update tree
                if *reference_counts == 0 {
                    error!(
                        "misc: StaticMapTree::UnloadMapTile() : trying to unload non-referenced model '{}' (ID:{})",
                        spawn.name, spawn.id,
                    );
                } else {
                    *reference_counts -= 1;
                    has_reference = *reference_counts != 0;
                }
            } else {
                error!("misc: StaticMapTree::UnloadMapTile() : trying to a tree value that does not exist (tree_id:{bvh_id})");
            }
            if has_reference {
                new_tile_spawn_tree_ids.push(bvh_id);
            } else {
                // Do the removal from tree values as well.
                self.tree_values.remove(&bvh_id);
            }
        }
        self.tile_to_bh_ids.insert(tile_id, new_tile_spawn_tree_ids);
    }

    pub fn pack_tile_id(tile_x: u16, tile_y: u16) -> u32 {
        let packed = (tile_x as u32) << 16;
        packed | (tile_y as u32)
    }

    pub fn unpack_tile_id(id: u32) -> (u16, u16) {
        let tile_x = (id >> 16) as _;
        let tile_y = ((id << 16) >> 16) as _;

        (tile_x, tile_y)
    }

    pub fn map_file_name<P: AsRef<Path>, M: Num + Display>(dir: P, map_id: M) -> PathBuf {
        dir.as_ref().join(format!("{map_id:04}.vmtree"))
    }

    pub fn map_id_from_map_file_map<P: AsRef<Path>>(p: P) -> AzResult<u32> {
        let map_id = (file_stem_if_ext_matched(p, "vmtree")?).parse::<u32>()?;
        Ok(map_id)
    }

    pub fn get_tile_file_name<P, M, X, Y>(dir: P, map_id: M, y: X, x: Y) -> PathBuf
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
        ptree: &BVH,
        model_spawns_used: &[&VmapModelSpawn],
    ) -> AzResult<()> {
        let mapfilename = Self::map_file_name(dir, map_id);
        let mut mapfile = fs::File::create(&mapfilename).inspect_err(|e| {
            error!("cannot open {}, err was: {e}", mapfilename.display());
        })?;

        Self::write_map_tree(&mut mapfile, ptree, model_spawns_used)
    }

    fn write_map_tree<W: io::Write>(w: &mut W, ptree: &BVH, model_spawns_used: &[&VmapModelSpawn]) -> AzResult<()> {
        let mut w = w;
        //general info
        w.write_all(VMAP_MAGIC)?;
        // Nodes
        w.write_all(b"NODE")?;

        bincode_serialise(&mut w, ptree)?;

        // spawn id to index map
        // uint32 map_spawnsSize = map_spawns.size();
        w.write_all(b"SIDX")?;
        let map_spawn_id_to_bvh_id = model_spawns_used.iter().map(|m| (m.id, m.bh_node_index())).collect::<Vec<_>>();
        bincode_serialise(w, &map_spawn_id_to_bvh_id)?;
        Ok(())
    }

    pub fn read_map_tree<R: io::Read>(r: &mut R) -> AzResult<(BVH, HashMap<u32, usize>)> {
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

    pub fn write_map_tile_spawns_file<P, M, X, Y>(dir: P, map_id: M, y: X, x: Y, model_spawns: &[&VmapModelSpawn]) -> AzResult<()>
    where
        P: AsRef<Path>,
        M: Num + Display,
        X: Num + Display,
        Y: Num + Display,
    {
        let tile_file_name = Self::get_tile_file_name(dir, map_id, y, x);
        let mut tile_file = fs::File::create(tile_file_name)?;
        Self::write_map_tile_spawns(&mut tile_file, model_spawns)
    }

    fn write_map_tile_spawns<W: io::Write>(w: &mut W, model_spawns: &[&VmapModelSpawn]) -> AzResult<()> {
        let mut w = w;

        w.write_all(VMAP_MAGIC)?;
        bincode_serialise(&mut w, &model_spawns)?;
        Ok(())
    }

    fn read_map_tile_spawns_file<P>(
        dir: P,
        map_id: u32,
        y: u16,
        x: u16,
        parent_map_data: Arc<HashMap<u32, u32>>,
    ) -> AzResult<Vec<VmapModelSpawn>>
    where
        P: AsRef<Path>,
    {
        let file_name = Self::get_tile_file_name(&dir, map_id, y, x);
        let mut file = fs::File::open(file_name).or_else(|e| match parent_map_data.get(&map_id) {
            None => Err(e),
            Some(parent_id) => {
                let file_name = Self::get_tile_file_name(&dir, *parent_id, y, x);
                fs::File::open(file_name)
            },
        })?;

        Self::read_map_tile_spawns(&mut file)
    }

    pub fn read_map_tile_spawns<R: io::Read>(r: &mut R) -> AzResult<Vec<VmapModelSpawn>> {
        let mut r = r;

        cmp_or_return!(r, VMAP_MAGIC)?;
        let res = bincode_deserialise(&mut r)?;

        sanity_check_read_all_bytes_from_reader!(r)?;

        Ok(res)
    }
}

// class TC_COMMON_API StaticMapTree
// {
// static TileFileOpenResult OpenMapTileFile(std::string const& basePath, uint32 mapID, uint32 tileX, uint32 tileY, VMapManager2* vm);
// static LoadResult CanLoadMap(const std::string &basePath, uint32 mapID, uint32 tileX, uint32 tileY, VMapManager2* vm);
//         typedef std::unordered_map<uint32, bool> loadedTileMap;
//         typedef std::unordered_map<uint32, uint32> loadedSpawnMap;
//         private:
//             struct TileFileOpenResult
//             {
//                 FILE* File;
//                 std::string Name;
//             };

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

#[cfg(test)]
mod tests {
    use super::StaticMapTree;

    #[test]
    fn it_ensures_that_static_map_tree_packing_unpacking_works() {
        let tests = [
            (u16::MIN, u16::MIN, 0),
            (u16::MIN, u16::MAX, 65535),
            (u16::MAX, u16::MIN, 4294901760),
            (u16::MAX, u16::MAX, 4294967295),
        ];

        for (idx, (tile_x, tile_y, expected_packed)) in tests.into_iter().enumerate() {
            let result_packed = StaticMapTree::pack_tile_id(tile_x, tile_y);
            assert_eq!(
                result_packed, expected_packed,
                "test {idx} failed: got packed {result_packed}, expected {expected_packed}"
            );
            let (result_x, result_y) = StaticMapTree::unpack_tile_id(result_packed);
            assert_eq!(result_x, tile_x, "test {idx} failed: got x {result_x}, expected {tile_x}");
            assert_eq!(result_y, tile_y, "test {idx} failed: got y {result_y}, expected {tile_y}");
        }
    }
}
