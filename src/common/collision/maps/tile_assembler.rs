use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io,
    path::{Path, PathBuf},
};

use bvh::{aabb::AABB, bvh::BVH};
use flagset::FlagSet;
use nalgebra::{Matrix3, Rotation, Vector2, Vector3};
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
        extractor_common::{bincode_deserialise, bincode_serialise, get_fixed_plain_name},
        vmap4_extractor::{wmo::WMOLiquidHeader, TempGameObjectModel},
    },
    GenericResult,
};

pub fn tile_assembler_convert_world2(
    i_dest_dir: PathBuf,
    i_src_dir: PathBuf,
    map_data: BTreeMap<u32, BTreeMap<u32, VmapModelSpawn>>,
    temp_gameobject_models: Vec<TempGameObjectModel>,
) -> GenericResult<()> {
    let inv_tile_size = 3f32 / 1600f32;

    let mut spawned_model_files = BTreeSet::new();
    let mut map_data = map_data;

    // export Map data
    while let Some((map_id, mut data)) = map_data.pop_first() {
        // tile entries => packedTileId to set of tilespawns
        let mut tile_entries = BTreeMap::new();
        let mut parent_tile_entries = BTreeMap::new();
        // build global map tree
        let mut map_spawns = Vec::with_capacity(data.len());
        info!("Calculating model bounds for map {map_id}...");
        for (_spawn_id, entry) in data.iter_mut() {
            // M2 models don't have a bound set in WDT/ADT placement data, i still think they're not used for LoS at all on retail
            if entry.flags.contains(ModelFlags::ModM2) && calculate_transformed_bound(&i_src_dir, entry).is_err() {
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
            spawned_model_files.insert(entry.name.clone());
            map_spawns.push(entry);
        }

        info!("Creating map tree for map {map_id}. map_spawns len is {}...", map_spawns.len());
        let ptree = BVH::build(&mut map_spawns);
        // unborrow map_spawns
        let map_spawns = map_spawns.into_iter().map(|m| &*m).collect::<Vec<_>>();

        // write map tree file
        StaticMapTree::write_map_tree_to_file(&i_dest_dir, map_id, &ptree, &map_spawns)?;

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
            StaticMapTree::write_map_tile_spawns_file(&i_dest_dir, map_id, y, x, &all_tile_entries)?;
        }
    }

    // add an object models, listed in temp_gameobject_models file
    info!("Exporting game object models");
    export_gameobject_models(&i_src_dir, &i_dest_dir, temp_gameobject_models, &mut spawned_model_files)?;
    // export objects
    info!("Converting Model Files");
    for mfile_name in spawned_model_files.iter() {
        info!("Converting {mfile_name}");
        if let Err(e) = convert_raw_file(&i_src_dir, &i_dest_dir, mfile_name) {
            warn!("error converting {mfile_name}: err {e}");
            break;
        }
    }

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

fn convert_raw_file<P: AsRef<Path>>(i_src_dir: P, i_dest_dir: P, p_model_filename: &str) -> GenericResult<()> {
    let filename = i_src_dir.as_ref().join(p_model_filename);

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

    let mut outfile = fs::File::create(i_dest_dir.as_ref().join(format!("{p_model_filename}.vmo"))).inspect_err(|e| {
        error!("create new  vmofile err: {}; err was {e}", filename.display());
    })?;
    model.write_file(&mut outfile)?;
    Ok(())
}

fn export_gameobject_models<P: AsRef<Path>>(
    i_src_dir: P,
    i_dest_dir: P,
    temp_gameobject_models: Vec<TempGameObjectModel>,
    spawned_model_files: &mut BTreeSet<String>,
) -> GenericResult<()> {
    let mut model_list = BTreeMap::new();
    let mut success_count = 0;
    let total_count = temp_gameobject_models.len();
    for tmp in temp_gameobject_models {
        let TempGameObjectModel {
            id: display_id,
            is_wmo,
            file_name: model_name,
        } = tmp;

        let raw_model_file_path = i_src_dir.as_ref().join(get_fixed_plain_name(&model_name));
        let mut raw_model_file = match fs::File::open(&raw_model_file_path) {
            Err(e) => {
                warn!("cannot open raw file for some reason: path: {}, err {e}", raw_model_file_path.display());
                continue;
            },
            Ok(f) => f,
        };
        let raw_model = match WorldModel_Raw::read(&mut raw_model_file) {
            Err(e) => {
                warn!(
                    "read raw file model file failed for some reason: path: {}, err {e}",
                    raw_model_file_path.display()
                );
                continue;
            },
            Ok(m) => m,
        };

        spawned_model_files.insert(model_name.clone());
        let mut bounds = AABB::empty();
        for grp in raw_model.groups {
            for v in grp.vertices_chunks {
                bounds.grow_mut(&v.into());
            }
        }
        if bounds.is_empty() {
            warn!("Model {model_name} has empty bounding box or has an invalid bounding box");
            continue;
        }
        success_count += 1;
        model_list.insert(
            display_id,
            GameObjectModelData {
                display_id,
                bounds,
                is_wmo,
                name: model_name,
            },
        );
    }
    GameObjectModelData::write_to_file(i_dest_dir, &model_list)?;
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
