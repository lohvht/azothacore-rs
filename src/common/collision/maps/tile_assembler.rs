use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io,
    path::Path,
    sync::mpsc::channel,
};

use bvh::{aabb::AABB, bounding_hierarchy::BHShape, bvh::BVH};
use flagset::FlagSet;
use futures::future;
use nalgebra::{Matrix3, Rotation, Vector2, Vector3};
use rayon::prelude::*;
use tracing::{error, info, warn};

use crate::{
    cmp_or_return,
    common::collision::{
        maps::map_tree::StaticMapTree,
        models::{
            game_object_model::GameObjectModelData,
            model_instance::{ModelFlags, VmapModelSpawn},
            world_model::{GroupModel, WmoLiquid, WorldModel},
        },
        vmap_definitions::RAW_VMAP_MAGIC,
    },
    read_le,
    sanity_check_read_all_bytes_from_reader,
    tools::{
        extractor_common::{bincode_deserialise, bincode_serialise, get_fixed_plain_name, ExtractorConfig},
        vmap4_extractor::{wmo::WMOLiquidHeader, TempGameObjectModel},
    },
    GenericResult,
};

pub fn tile_assembler_convert_world2(
    args: &ExtractorConfig,
    map_data: BTreeMap<u32, BTreeMap<u32, VmapModelSpawn>>,
    temp_gameobject_models: Vec<TempGameObjectModel>,
) -> GenericResult<()> {
    let src = args.output_vmap_sz_work_dir_wmo();
    let dst = args.output_vmap_output_path();

    let src_display = src.display();
    let dst_display = dst.display();
    info!("using {src_display} as source directory and writing output to {dst_display}");

    fs::create_dir_all(&dst)?;

    let inv_tile_size = 3f32 / 1600f32;

    let (sender, receiver) = channel();

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
            let bounds = AABB::with_bounds(bounds[0].into(), bounds[1].into());
            let low = Vector2::new((bounds.min.x * inv_tile_size).floor() as u16, (bounds.min.y * inv_tile_size).floor() as u16);
            let high = Vector2::new((bounds.max.x * inv_tile_size).ceil() as u16, (bounds.max.y * inv_tile_size).ceil() as u16);

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
        let ptree = BVH::build(&mut map_spawns);
        // unborrow map_spawns
        let map_spawns = map_spawns.into_iter().map(|m| &*m).collect::<Vec<_>>();

        // write map tree file
        StaticMapTree::write_map_tree_to_file(&dst, map_id, &ptree, &map_spawns)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("write map tree to file err: {e}")))?;

        if args.debug_validation {
            info!("Debug validating map tree {map_id}");
            let mapfilename = StaticMapTree::map_file_name(&dst, map_id);
            let mut mapfile = fs::File::open(&mapfilename).inspect_err(|e| {
                error!("cannot open {}, err was: {e}", mapfilename.display());
            })?;
            let (r_bvh, r_spawn_id_to_bvh_id) =
                StaticMapTree::read_map_tree(&mut mapfile).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("read map tree to file err: {e}")))?;

            if r_bvh.nodes.len() != ptree.nodes.len() {
                error!(
                    "NODES FOR MAP BVH SHOULD MATCH IN LEN, CALCULATED: {}, READ: {}",
                    ptree.nodes.len(),
                    r_bvh.nodes.len()
                )
            }
            if map_spawns.len() != r_spawn_id_to_bvh_id.len() {
                error!(
                    "SPAWN IDS FOR MAP SHOULD MATCH IN LEN, CALCULATED: {}, READ: {}",
                    map_spawns.len(),
                    r_spawn_id_to_bvh_id.len()
                )
            }
            for m in map_spawns.iter() {
                match r_spawn_id_to_bvh_id.get(&m.id) {
                    None => {
                        error!("CALCULATED SPAWN ID SHOULD BE IN READ SPAWN ID, CALCULATED: ID: {}", m.id)
                    },
                    Some(bvh_id) if *bvh_id != m.bh_node_index() => {
                        error!("CALCULATED SPAWN ID MISMATCH, CALCULATED: ID: {}, READ ID: {}", m.bh_node_index(), bvh_id)
                    },
                    _ => {},
                }
            }
            let calc_spawn_id_to_bvh_id = map_spawns.iter().map(|v| (v.id, v.bh_node_index())).collect::<HashMap<_, _>>();
            if r_spawn_id_to_bvh_id != calc_spawn_id_to_bvh_id {
                error!(
                    "SPAWN IDS FOR MAP SHOULD MATCH IN LEN, CALCULATED: {}, READ: {}",
                    calc_spawn_id_to_bvh_id.len(),
                    r_spawn_id_to_bvh_id.len()
                )
            }
            for (spawn_id, r_bh_id) in r_spawn_id_to_bvh_id.iter() {
                let has_r_bh_id = r_bvh.nodes.iter().any(|n| match n {
                    bvh::bvh::BVHNode::Leaf { shape_index, .. } => *shape_index == *r_bh_id,
                    bvh::bvh::BVHNode::Node {
                        child_l_index, child_r_index, ..
                    } => *child_l_index == *r_bh_id || *child_r_index == *r_bh_id,
                    _ => false,
                });
                if !has_r_bh_id {
                    error!("Spawn ID may be invalid as its respective BH ID isnt found. spawn_id {spawn_id}, bh_id: {r_bh_id}");
                }
            }
            info!("Debug validating map tree done {map_id}")
        }

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
            StaticMapTree::write_map_tile_spawns_file(&dst, map_id, y, x, &all_tile_entries)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("write_map_tile_spawns_file err: {e}")))?;
            if args.debug_validation {
                let tilefilename = StaticMapTree::get_tile_file_name(&dst, map_id, y, x);
                let mut tilefile = fs::File::open(&tilefilename).inspect_err(|e| {
                    error!("cannot open {}, err was: {e}", tilefilename.display());
                })?;
                let r_spawns = StaticMapTree::read_map_tile_spawns(&mut tilefile)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("read map tree to file err: {e}")))?;

                if r_spawns.len() != all_tile_entries.len() {
                    error!(
                        "SPAWNS FOR MAP TILE SHOULD MATCH IN LEN, CALCULATED: {}, READ: {}",
                        all_tile_entries.len(),
                        r_spawns.len(),
                    )
                }

                for (i, te) in all_tile_entries.iter().enumerate() {
                    if te.id != r_spawns[i].id {
                        error!(
                            "SPAWNS FOR MAP TILE SHOULD IN ID AND POS, CALCULATED: {}, READ: {}",
                            all_tile_entries.len(),
                            r_spawns.len(),
                        )
                    }
                }
            }
        }
        io::Result::Ok(())
    })?;

    let mut spawned_model_files: BTreeSet<_> = receiver.iter().collect();

    // add an object models, listed in temp_gameobject_models file
    info!("Exporting game object models");
    export_gameobject_models(&src, &dst, temp_gameobject_models, &mut spawned_model_files)?;
    // export objects
    info!("Converting Model Files");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().max_blocking_threads(50).build()?;
    let mut jhs = Vec::with_capacity(spawned_model_files.len());
    for mfile_name in spawned_model_files {
        let src = src.clone();
        let dest = dst.clone();
        jhs.push(rt.spawn_blocking(|| {
            info!("Converting {mfile_name}");
            convert_raw_file(src, dest, mfile_name.into())
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

fn calculate_transformed_bound<P: AsRef<Path>>(src: P, spawn: &mut VmapModelSpawn) -> GenericResult<()> {
    if spawn.bound.is_some() {
        return Ok(());
    }

    let model_filename = src.as_ref().join(&spawn.name);

    let model_position = ModelPosition::new(spawn.i_rot, spawn.i_scale);

    let mut input = fs::File::open(&model_filename).inspect_err(|e| {
        error!("ERROR: Can't open raw model file: {} - err {e}", model_filename.display());
    })?;
    let raw_model = WorldModel_Raw::read(&mut input).inspect_err(|e| {
        error!("ERROR: read raw world model error: {} - err {e}", model_filename.display());
    })?;

    let groups = raw_model.groups.len();
    if groups != 1 {
        warn!("Warning: '{}' does not seem to be a M2 model!", model_filename.display());
    }
    let mut model_bound = AABB::empty();

    model_bound.grow_mut(&model_position.transform(&raw_model.groups[0].bbcorn1).into());
    model_bound.grow_mut(&model_position.transform(&raw_model.groups[0].bbcorn2).into());

    let mut min = model_bound.min.into();
    let mut max = model_bound.max.into();
    min += spawn.i_pos;
    max += spawn.i_pos;

    spawn.bound = Some([min, max]);
    spawn.flags |= ModelFlags::ModHasBound;
    Ok(())
}

fn convert_raw_file<P: AsRef<Path>>(src: P, dst: P, p_model_filename: P) -> io::Result<()> {
    let filename = src.as_ref().join(p_model_filename.as_ref());
    let out = dst.as_ref().join(format!("{}.vmo", p_model_filename.as_ref().display()));
    if out.try_exists()? {
        return Ok(());
    }

    let mut raw_model_file = fs::File::open(&filename).inspect_err(|e| {
        error!("convert_raw_file err: {}; err was {e}", filename.display());
    })?;

    let raw_model = WorldModel_Raw::read(&mut raw_model_file).inspect_err(|e| {
        error!("read raw_world_model err: {}; err was {e}", filename.display());
    })?;

    let groups = raw_model
        .groups
        .into_iter()
        .map(|raw_group| {
            let liq = raw_group
                .liquid
                .map(|raw_liq| {
                    WmoLiquid::new(
                        raw_liq.header.xtiles as u32,
                        raw_liq.header.ytiles as u32,
                        Vector3::new(raw_liq.header.pos_x, raw_liq.header.pos_y, raw_liq.header.pos_z),
                        raw_group.liquid_type,
                        raw_liq.liquid_heights,
                        raw_liq.liquid_flags,
                    )
                })
                .or_else(|| {
                    if (raw_group.liquidflags & 3) > 0 && (raw_group.liquidflags & 1) == 0 {
                        Some(WmoLiquid::new_without_flags(raw_group.bbcorn2.z))
                    } else {
                        None
                    }
                });
            GroupModel::new(
                raw_group.mogp_flags,
                raw_group.group_wmo_id,
                AABB::with_bounds(raw_group.bbcorn1.into(), raw_group.bbcorn2.into()),
                raw_group.mesh_triangle_indices,
                raw_group.vertices_chunks,
                liq,
            )
        })
        .collect();

    let model = WorldModel::new(raw_model.root_wmo_id, groups);

    let mut outfile = fs::File::create(out).inspect_err(|e| {
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
    temp_gameobject_models: Vec<TempGameObjectModel>,
    spawned_model_files: &mut BTreeSet<String>,
) -> GenericResult<()> {
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
            let mut raw_model_file = match fs::File::open(&raw_model_file_path) {
                Err(e) => {
                    warn!("cannot open raw file for some reason: path: {}, err {e}", raw_model_file_path.display());
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
            let mut bounds = AABB::empty();
            for grp in raw_model.groups {
                for v in grp.vertices_chunks {
                    bounds.grow_mut(&v.into());
                }
            }
            if bounds.is_empty() {
                warn!("Model {model_name} has empty bounding box or has an invalid bounding box");
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

    fn transform(&self, p_in: &Vector3<f32>) -> Vector3<f32> {
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
        for g in s.groups.iter() {
            if let Some(GroupModel_Liquid_Raw { header: hlq, .. }) = &g.liquid {
                if hlq.xverts != hlq.xtiles + 1 {
                    panic!("SANITY CHECK, xverts {} must be 1 more than xtiles {}", hlq.xverts, hlq.xtiles);
                }
                if hlq.yverts != hlq.ytiles + 1 {
                    panic!("SANITY CHECK, yverts {} must be 1 more than ytiles {}", hlq.yverts, hlq.ytiles);
                }
            }
        }
        Ok(s)
    }
}

#[allow(non_camel_case_types)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct GroupModel_Raw {
    pub mogp_flags:            u32,
    pub group_wmo_id:          u32,
    pub bbcorn1:               Vector3<f32>,
    pub bbcorn2:               Vector3<f32>,
    pub liquidflags:           u32,
    /// Either from MOBA's MOVI indices count or from M2 collisionIndices size
    /// 1 group have at least 1 of these (group models can have >= 1 MOBAs)
    pub n_bounding_triangles:  Vec<u16>,
    /// either indices from MOVI or from M2 Model triangle indices
    pub mesh_triangle_indices: Vec<Vector3<u16>>,
    /// Either from MOVT or  in (X,Z,-Y) order
    pub vertices_chunks:       Vec<Vector3<f32>>,
    pub liquid_type:           u32,
    pub liquid:                Option<GroupModel_Liquid_Raw>,
}

#[allow(non_camel_case_types)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct GroupModel_Liquid_Raw {
    pub header:         WMOLiquidHeader,
    pub liquid_heights: Vec<f32>,
    pub liquid_flags:   Vec<u8>,
}
