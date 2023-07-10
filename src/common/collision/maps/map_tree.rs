use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    io,
    path::{Path, PathBuf},
};

use bvh::{bounding_hierarchy::BHShape, bvh::BVH};
use num_traits::Num;
use tracing::error;

use crate::{
    cmp_or_return,
    common::collision::{models::model_instance::VmapModelSpawn, vmap_definitions::VMAP_MAGIC},
    sanity_check_read_all_bytes_from_reader,
    tools::extractor_common::{bincode_deserialise, bincode_serialise},
    GenericResult,
};

pub struct StaticMapTree {}

impl StaticMapTree {
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
    ) -> GenericResult<()> {
        let mapfilename = Self::map_file_name(dir, map_id);
        let mut mapfile = fs::File::create(&mapfilename).inspect_err(|e| {
            error!("cannot open {}, err was: {e}", mapfilename.display());
        })?;

        Self::write_map_tree(&mut mapfile, ptree, model_spawns_used)
    }

    fn write_map_tree<W: io::Write>(w: &mut W, ptree: &BVH, model_spawns_used: &[&VmapModelSpawn]) -> GenericResult<()> {
        let mut w = w;
        //general info
        w.write_all(VMAP_MAGIC)?;
        // Nodes
        w.write_all(b"NODE")?;

        bincode_serialise(&mut w, ptree)?;

        // spawn id to index map
        // uint32 map_spawnsSize = map_spawns.size();
        w.write_all(b"SIDX")?;
        let map_spawn_id_to_bvh_id = model_spawns_used.iter().map(|m| (m.id, m.bh_node_index())).collect::<HashMap<_, _>>();
        bincode_serialise(w, &map_spawn_id_to_bvh_id)?;
        Ok(())
    }

    pub fn read_map_tree<R: io::Read>(r: &mut R) -> GenericResult<(BVH, HashMap<u32, usize>)> {
        let mut r = r;

        cmp_or_return!(r, VMAP_MAGIC)?;
        cmp_or_return!(r, b"NODE")?;
        let tree = bincode_deserialise(&mut r)?;
        cmp_or_return!(r, b"SIDX")?;
        let map_spawn_id_to_bvh_id = bincode_deserialise(&mut r)?;

        sanity_check_read_all_bytes_from_reader!(r)?;

        Ok((tree, map_spawn_id_to_bvh_id))
    }

    pub fn write_map_tile_spawns_file<P, M, X, Y>(dir: P, map_id: M, y: X, x: Y, model_spawns: &[&VmapModelSpawn]) -> GenericResult<()>
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

    fn write_map_tile_spawns<W: io::Write>(w: &mut W, model_spawns: &[&VmapModelSpawn]) -> GenericResult<()> {
        let mut w = w;

        w.write_all(VMAP_MAGIC)?;
        bincode_serialise(&mut w, &model_spawns)?;
        Ok(())
    }

    pub fn read_map_tile_spawns<R: io::Read>(r: &mut R) -> GenericResult<Vec<VmapModelSpawn>> {
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
//             uint32 iMapID;
//             BIH iTree;
//             ModelInstance* iTreeValues; // the tree entries
//             uint32 iNTreeValues;
//             std::unordered_map<uint32, uint32> iSpawnIndices;

//             // Store all the map tile idents that are loaded for that map
//             // some maps are not splitted into tiles and we have to make sure, not removing the map before all tiles are removed
//             // empty tiles have no tile file, hence map with bool instead of just a set (consistency check)
//             loadedTileMap iLoadedTiles;
//             std::vector<std::pair<int32, int32>> iLoadedPrimaryTiles;
//             // stores <tree_index, reference_count> to invalidate tree values, unload map, and to be able to report errors
//             loadedSpawnMap iLoadedSpawns;
//             std::string iBasePath;

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
//             bool LoadMapTile(uint32 tileX, uint32 tileY, VMapManager2* vm);
//             void UnloadMapTile(uint32 tileX, uint32 tileY, VMapManager2* vm);
//             uint32 numLoadedTiles() const { return uint32(iLoadedTiles.size()); }
//             void getModelInstances(ModelInstance* &models, uint32 &count);

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
