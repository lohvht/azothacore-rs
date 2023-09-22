use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io,
    path::Path,
    sync::mpsc::channel,
};

use flagset::FlagSet;
use futures::future;
use nalgebra::{Matrix3, Rotation, Vector2, Vector3};
use parry3d::{bounding_volume::Aabb, math::Point, partitioning::Qbvh};
use rayon::prelude::*;
use tracing::{error, info, warn};

use crate::{
    bincode_deserialise,
    bincode_serialise,
    buffered_file_create,
    buffered_file_open,
    cmp_or_return,
    common::collision::{
        maps::map_tree::StaticMapTree,
        models::{
            game_object_model::GameObjectModelData,
            model_instance::{ModelFlags, VmapModelSpawnWithMapId},
            world_model::{GroupModel, WmoLiquid, WorldModel},
        },
        vmap_definitions::RAW_VMAP_MAGIC,
    },
    read_le,
    sanity_check_read_all_bytes_from_reader,
    tools::{
        extractor_common::{get_fixed_plain_name, ExtractorConfig},
        vmap4_extractor::TempGameObjectModel,
    },
    AzResult,
};

pub fn read_map_spawns(map_spawns: impl Iterator<Item = VmapModelSpawnWithMapId>) -> BTreeMap<u32, BTreeMap<u32, VmapModelSpawnWithMapId>> {
    // retrieve the unique entries
    let mut map_data: BTreeMap<u32, BTreeMap<u32, VmapModelSpawnWithMapId>> = BTreeMap::new();
    for spawn in map_spawns {
        let unique_entries = map_data.entry(spawn.map_num).or_default();
        if unique_entries.is_empty() {
            info!("Spawning map {}", spawn.map_num);
        }
        unique_entries.insert(spawn.id, spawn);
    }
    map_data
}

pub fn tile_assembler_convert_world2(
    args: &ExtractorConfig,
    map_spawns: impl Iterator<Item = VmapModelSpawnWithMapId>,
    temp_gameobject_models: impl Iterator<Item = TempGameObjectModel>,
) -> AzResult<()> {
    let src = args.output_vmap_sz_work_dir_wmo();
    let dst = args.output_vmap_output_path();

    let src_display = src.display();
    let dst_display = dst.display();
    info!("using {src_display} as source directory and writing output to {dst_display}");

    fs::create_dir_all(&dst)?;

    let inv_tile_size = 3f32 / 1600f32;

    let (sender, receiver) = channel();

    let map_data = read_map_spawns(map_spawns);
    // export Map data
    map_data.into_par_iter().try_for_each_with(sender, |s, (map_id, mut data)| {
        // tile entries => packedTileId to set of tilespawns
        let mut tile_entries = BTreeMap::new();
        let mut parent_tile_entries = BTreeMap::new();
        // build global map tree
        let mut map_spawns = Vec::with_capacity(data.len());
        info!("Calculating model bounds for map {map_id}...");
        for (_spawn_id, entry) in data.iter_mut() {
            // M2 models don't have a bound set in WDT/ADT placement data, i still think they're not used for LoS at all on retail
            if entry.flags.contains(ModelFlags::ModM2) && calculate_transformed_bound(&src, entry).is_err() {
                continue;
            }

            let entry_tile_entries = if entry.flags.contains(ModelFlags::ModParentSpawn) {
                &mut parent_tile_entries
            } else {
                &mut tile_entries
            };
            let bounds = entry.bound.expect("By here bounds should never be unset");
            let low = Vector2::new((bounds.mins.x * inv_tile_size) as u16, (bounds.mins.y * inv_tile_size) as u16);
            let high = Vector2::new((bounds.maxs.x * inv_tile_size) as u16, (bounds.maxs.y * inv_tile_size) as u16);

            for x in low.x..=high.x {
                for y in low.y..=high.y {
                    entry_tile_entries
                        .entry(StaticMapTree::pack_tile_id(x, y))
                        .or_insert(BTreeSet::new())
                        .insert(TileSpawn {
                            id:    entry.id,
                            flags: entry.flags,
                        });
                }
            }
            s.send(entry.name.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("send fail, err: {e}")))?;
            map_spawns.push(entry);
        }

        info!("Creating map tree for map {map_id}. map_spawns len is {}...", map_spawns.len());
        let mut ptree = Qbvh::new();
        // unborrow map_spawns
        let map_spawns = map_spawns.into_iter().map(|m| &m.spawn).collect::<Vec<_>>();
        let map_data = map_spawns.iter().enumerate().map(|(idx, m)| (idx, m.bound.unwrap()));
        ptree.clear_and_rebuild(map_data, 0.0);

        // write map tree file
        StaticMapTree::write_map_tree_to_file(&dst, map_id, &ptree, &map_spawns)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("write map tree to file err: {e}")))?;

        // write map tile files, similar to ADT files, only with extra BVH tree node info
        for (tile_id, tile_entries) in tile_entries.iter() {
            let (x, y) = StaticMapTree::unpack_tile_id(*tile_id);
            let empty = BTreeSet::new();
            let parent_tile_entries = parent_tile_entries.get(tile_id).unwrap_or(&empty);
            let mut all_tile_entries = Vec::with_capacity(tile_entries.len() + parent_tile_entries.len());
            for te in [tile_entries, parent_tile_entries] {
                for spawn in te {
                    let model_spawn = match data.get(&spawn.id) {
                        None => {
                            warn!(
                                "tile_entries model spawn does not exist in map data for {map_id} for ID {} some reason. should not happen",
                                spawn.id,
                            );
                            continue;
                        },
                        Some(ms) => ms,
                    };
                    all_tile_entries.push(model_spawn);
                }
            }
            let all_tile_entries = all_tile_entries.into_iter().map(|m| &m.spawn).collect::<Vec<_>>();
            StaticMapTree::write_map_tile_spawns_file(&dst, map_id, x, y, &all_tile_entries)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("write_map_tile_spawns_file err: {e}")))?;
        }
        io::Result::Ok(())
    })?;

    let mut spawned_model_files: BTreeSet<_> = receiver.iter().collect();

    // add an object models, listed in temp_gameobject_models file
    info!("Exporting game object models");
    export_gameobject_models(&src, &dst, temp_gameobject_models, &mut spawned_model_files)?;
    // export objects
    info!("Converting Model Files");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(50)
        .build()?;
    let mut jhs = Vec::with_capacity(spawned_model_files.len());
    for mfile_name in spawned_model_files {
        let src = src.clone();
        let dest = dst.clone();
        jhs.push(rt.spawn_blocking(|| {
            info!("Converting {mfile_name}");
            convert_raw_file(src, dest, mfile_name)
        }))
    }
    rt.block_on(async {
        for r in future::join_all(jhs).await {
            match r {
                Err(e) => {
                    warn!("join error while converting: err {e}");
                },
                Ok(Err(e)) => {
                    warn!("error converting: err {e}");
                },
                _ => {},
            }
        }
    });

    Ok(())
}

fn calculate_transformed_bound<P: AsRef<Path>>(src: P, spawn: &mut VmapModelSpawnWithMapId) -> AzResult<()> {
    let model_filename = src.as_ref().join(&spawn.name);

    let model_position = ModelPosition::new(spawn.i_rot, spawn.i_scale);

    let mut input = buffered_file_open(&model_filename).inspect_err(|e| {
        error!("ERROR: Can't open raw model file: {} - err {e}", model_filename.display());
    })?;
    let raw_model = WorldModel_Raw::read(&mut input).inspect_err(|e| {
        error!("ERROR: read raw world model error: {} - err {e}", model_filename.display());
    })?;

    let groups = raw_model.groups.len();
    if groups != 1 {
        warn!("Warning: '{}' does not seem to be a M2 model!", model_filename.display());
    }
    let mut model_bound = Aabb::new_invalid();

    model_bound.take_point(model_position.transform(raw_model.groups[0].bbcorn.mins));
    model_bound.take_point(model_position.transform(raw_model.groups[0].bbcorn.maxs));

    model_bound.mins += spawn.i_pos;
    model_bound.maxs += spawn.i_pos;

    spawn.bound = Some(model_bound);
    Ok(())
}

pub fn convert_raw_file<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(src: P1, dst: P2, p_model_filename: P3) -> io::Result<()> {
    let filename = src.as_ref().join(p_model_filename.as_ref());
    let out = dst.as_ref().join(format!("{}.vmo", p_model_filename.as_ref().display()));
    if out.try_exists()? {
        return Ok(());
    }

    let mut raw_model_file = buffered_file_open(&filename).inspect_err(|e| {
        error!("convert_raw_file err: {}; err was {e}", filename.display());
    })?;

    let raw_model = WorldModel_Raw::read(&mut raw_model_file).inspect_err(|e| {
        error!("read raw_world_model err: {}; err was {e}", filename.display());
    })?;

    let groups = raw_model
        .groups
        .into_iter()
        .map(|raw_group| {
            GroupModel::new(
                raw_group.mogp_flags,
                raw_group.group_wmo_id,
                raw_group.bbcorn,
                raw_group.mesh_triangle_indices,
                raw_group.vertices_chunks,
                raw_group.liquid,
            )
        })
        .collect();

    let model = WorldModel::new(raw_model.root_wmo_id, groups);

    let mut outfile = buffered_file_create(out).inspect_err(|e| {
        error!("create new  vmofile err: {}; err was {e}", filename.display());
    })?;
    model
        .write_file(&mut outfile)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("MODEL RAW VMO WRITE ERR: {e}")))?;
    Ok(())
}

fn export_gameobject_models<P: AsRef<Path> + std::marker::Sync>(
    src: P,
    dst: P,
    temp_gameobject_models: impl Iterator<Item = TempGameObjectModel>,
    spawned_model_files: &mut BTreeSet<String>,
) -> AzResult<()> {
    let temp_gameobject_models = temp_gameobject_models.collect::<Vec<_>>();
    let total_count = temp_gameobject_models.len();
    let (model_file_send, model_file_receive) = channel();
    let (model_list_send, model_list_receive) = channel();
    temp_gameobject_models
        .into_par_iter()
        .for_each_with((model_file_send, model_list_send), |(s1, s2), tmp| {
            let TempGameObjectModel {
                id: display_id,
                is_wmo,
                file_name: model_name,
            } = tmp;

            let raw_model_file_path = src.as_ref().join(get_fixed_plain_name(&model_name));
            let mut raw_model_file = match buffered_file_open(&raw_model_file_path) {
                Err(e) => {
                    warn!(
                        "cannot open raw file for some reason: path: {}, err {e}",
                        raw_model_file_path.display()
                    );
                    return;
                },
                Ok(f) => f,
            };
            let raw_model = match WorldModel_Raw::read(&mut raw_model_file) {
                Err(e) => {
                    warn!(
                        "read raw file model file failed for some reason: path: {}, err {e}",
                        raw_model_file_path.display()
                    );
                    return;
                },
                Ok(m) => m,
            };
            if s1.send(model_name.clone()).is_err() {
                return;
            }
            let mut bounds = Aabb::new(Point::new(f32::NAN, f32::NAN, f32::NAN), Point::new(f32::NAN, f32::NAN, f32::NAN));
            let mut bound_empty = true;
            for grp in raw_model.groups {
                for v in grp.vertices_chunks {
                    if bound_empty {
                        bounds = Aabb::new(v.into(), v.into());
                        bound_empty = false;
                    } else {
                        bounds.take_point(v.into());
                    }
                }
            }
            if bounds.maxs.iter().any(|x| x.is_nan()) || bounds.mins.iter().any(|x| x.is_nan()) {
                warn!("Model {model_name} has empty bounding box");
                return;
            }
            if !(bounds.maxs.iter().all(|x| x.is_finite()) && bounds.mins.iter().all(|x| x.is_finite())) {
                warn!("Model {model_name} has invalid bounding box");
                return;
            }
            _ = s2.send((
                display_id,
                GameObjectModelData {
                    display_id,
                    bounds,
                    is_wmo,
                    name: model_name,
                },
            ));
        });

    for s in model_file_receive {
        spawned_model_files.insert(s);
    }
    let model_list = model_list_receive.iter().collect();
    GameObjectModelData::write_to_file(dst, &model_list)?;
    let success_count = model_list.len();
    info!("GameObjectModels written: {success_count} / {total_count}");

    Ok(())
}

/**
This struct is used to convert raw vector data into balanced BSP-Trees.
To start the conversion call convertWorld().
*/
//===============================================

#[allow(dead_code)]
struct ModelPosition {
    i_rotation: Matrix3<f32>,
    i_pos:      Vector3<f32>,
    i_dir:      Vector3<f32>,
    i_scale:    f32,
}

impl ModelPosition {
    fn new(i_dir: Vector3<f32>, i_scale: f32) -> Self {
        // iRotation = G3D::Matrix3::fromEulerAnglesZYX(G3D::pif()*iDir.y/180.f, G3D::pif()*iDir.x/180.f, G3D::pif()*iDir.z/180.f);
        let i_rotation = *Rotation::from_euler_angles(i_dir.z.to_radians(), i_dir.x.to_radians(), i_dir.y.to_radians()).matrix();
        Self {
            i_rotation,
            i_pos: Vector3::zeros(),
            i_dir,
            i_scale,
        }
    }

    #[allow(dead_code)]
    fn move_to_base_pos(&mut self, p_base_pos: &Vector3<f32>) {
        self.i_pos -= p_base_pos;
    }

    fn transform(&self, p_in: Point<f32>) -> Point<f32> {
        let mut out = p_in * self.i_scale;
        out = self.i_rotation * out;
        out
    }
}

#[derive(PartialEq, Eq)]
struct TileSpawn {
    id:    u32,
    flags: FlagSet<ModelFlags>,
}

impl Ord for TileSpawn {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let cmp = self.id.cmp(&other.id);
        if !cmp.is_eq() {
            return cmp;
        }

        self.flags.bits().cmp(&other.flags.bits())
    }
}

impl PartialOrd for TileSpawn {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(non_camel_case_types)]
pub struct WorldModel_Raw {
    pub n_vectors:   usize,
    pub root_wmo_id: u32,
    pub groups:      Vec<GroupModel_Raw>,
}

impl WorldModel_Raw {
    pub fn write<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        let mut out = out;
        out.write_all(RAW_VMAP_MAGIC)?;
        out.write_all(&(self.n_vectors as u32).to_le_bytes())?;
        out.write_all(&self.root_wmo_id.to_le_bytes())?;
        bincode_serialise(&mut out, &self.groups).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("BINCODE WRITE ERR: {e}")))?;
        Ok(())
    }

    pub fn read_world_model_raw_header<R: io::Read>(input: &mut R) -> io::Result<(usize, u32)> {
        cmp_or_return!(input, RAW_VMAP_MAGIC)?;
        let n_vectors = read_le!(input, u32)? as usize;
        let root_wmo_id = read_le!(input, u32)?;
        Ok((n_vectors, root_wmo_id))
    }

    pub fn read<R: io::Read>(input: &mut R) -> io::Result<WorldModel_Raw> {
        let mut input = input;
        let (n_vectors, root_wmo_id) = Self::read_world_model_raw_header(input)?;
        let groups = bincode_deserialise(&mut input).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("BINCODE READ ERR: {e}")))?;

        sanity_check_read_all_bytes_from_reader!(input)?;
        let s = Self {
            root_wmo_id,
            n_vectors,
            groups,
        };
        Ok(s)
    }
}

#[allow(non_camel_case_types)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct GroupModel_Raw {
    pub mogp_flags:            u32,
    pub group_wmo_id:          u32,
    pub bbcorn:                Aabb,
    /// Either from MOBA's MOVI indices count or from M2 collisionIndices size
    /// 1 group have at least 1 of these (group models can have >= 1 MOBAs)
    pub n_bounding_triangles:  Vec<u16>,
    /// either indices from MOVI or from M2 Model triangle indices
    pub mesh_triangle_indices: Vec<Vector3<u16>>,
    /// Either from MOVT or  in (X,Z,-Y) order
    pub vertices_chunks:       Vec<Vector3<f32>>,
    pub liquid:                Option<WmoLiquid>,
}
