use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self},
    mem::{size_of, size_of_val},
    path::{Path, PathBuf},
};

use bvh::{aabb::AABB, bvh::BVH};
use flagset::FlagSet;
use nalgebra::{Matrix3, Rotation, Vector2, Vector3};
use tracing::{info, warn};

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
    tools::vmap4_extractor::{wmo::WMOLiquidHeader, TempGameObjectModel},
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
    // tile entries => packedTileId to set of tilespawns
    let mut tile_entries = BTreeMap::new();
    let mut parent_tile_entries = BTreeMap::new();

    // export Map data
    for (map_id, mut data) in map_data {
        // build global map tree
        let mut map_spawns = Vec::with_capacity(data.len());
        info!("Calculating model bounds for map {map_id}...");
        for (_spawn_id, entry) in data.iter_mut() {
            // M2 models don't have a bound set in WDT/ADT placement data, i still think they're not used for LoS at all on retail
            if !(entry.flags & ModelFlags::ModM2).is_empty() && calculate_transformed_bound(&i_src_dir, entry).is_err() {
                continue;
            }

            let entry_tile_entries = if !(entry.flags & ModelFlags::ModParentSpawn).is_empty() {
                &mut parent_tile_entries
            } else {
                &mut tile_entries
            };
            let bounds = entry.bound.expect("By here bounds should never be unset");
            let low = Vector2::new((bounds[0].x * inv_tile_size).round() as u16, (bounds[0].y * inv_tile_size).round() as u16);
            let high = Vector2::new((bounds[1].x * inv_tile_size).round() as u16, (bounds[1].y * inv_tile_size).round() as u16);

            for x in low.x..high.x {
                for y in low.y..high.y {
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

        info!("Creating map tree for map {map_id}...");
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
                            warn!("tile_entries model spawn does not exist in map data for {map_id} for some reason. should not happen");
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

    let mut input = fs::File::open(&model_filename)?;
    let raw_model = WorldModel_Raw::read(&mut input)?;

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
    let raw_model = WorldModel_Raw::read(&mut fs::File::open(filename)?)?;

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

    let mut outfile = fs::File::open(i_dest_dir.as_ref().join(format!("{p_model_filename}.vmo")))?;
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
    for tmp in temp_gameobject_models {
        let TempGameObjectModel {
            id: display_id,
            is_wmo,
            file_name: model_name,
        } = tmp;

        let raw_model_file_path = i_src_dir.as_ref().join(&model_name);
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
            warn!("\nModel {model_name} has empty bounding box or has an invalid bounding box");
            continue;
        }
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
        out.write_all(RAW_VMAP_MAGIC)?;
        out.write_all(&(self.n_vectors as u32).to_le_bytes())?;
        out.write_all(&(self.groups.len() as u32).to_le_bytes())?;
        out.write_all(&self.root_wmo_id.to_le_bytes())?;
        for g in self.groups.iter() {
            g.write(out)?;
        }
        Ok(())
    }

    pub fn read_world_model_raw_header<R: io::Read>(input: &mut R) -> io::Result<(usize, usize, u32)> {
        cmp_or_return!(input, RAW_VMAP_MAGIC)?;
        let n_vectors = read_le!(input, u32)? as usize;
        let n_groups = read_le!(input, u32)? as usize;
        let root_wmo_id = read_le!(input, u32)?;
        Ok((n_vectors, n_groups, root_wmo_id))
    }

    pub fn read<R: io::Read>(input: &mut R) -> io::Result<WorldModel_Raw> {
        let (n_vectors, n_groups, root_wmo_id) = Self::read_world_model_raw_header(input)?;
        let mut groups = Vec::with_capacity(n_groups);
        for _ in 0..n_groups {
            groups.push(GroupModel_Raw::read(input)?);
        }

        sanity_check_read_all_bytes_from_reader!(input)?;

        Ok(Self {
            root_wmo_id,
            n_vectors,
            groups,
        })
    }
}

#[allow(non_camel_case_types)]
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

macro_rules! vec_block_size {
    ( $collection:expr ) => {{
        if $collection.len() > 0 {
            $collection.len() * size_of_val(&$collection[0])
        } else {
            0
        }
    }};
}

macro_rules! vec_block_write {
    ( $out:expr, $collection:expr ) => {{
        for i in $collection.iter() {
            $out.write_all(&i.to_le_bytes())?;
        }
    }};
}

impl GroupModel_Raw {
    pub fn write<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        out.write_all(&self.mogp_flags.to_le_bytes())?;
        out.write_all(&self.group_wmo_id.to_le_bytes())?;
        vec_block_write!(out, self.bbcorn1);
        vec_block_write!(out, self.bbcorn2);
        out.write_all(&self.liquidflags.to_le_bytes())?;
        // GRP section
        out.write_all(b"GRP ")?;
        let block_size = size_of::<u32>() + vec_block_size!(self.n_bounding_triangles);
        out.write_all(&(block_size as u32).to_le_bytes())?;
        vec_block_write!(out, self.n_bounding_triangles);
        // INDX section
        out.write_all(b"INDX")?;
        let flat_triangle_indices = self.mesh_triangle_indices.iter().flat_map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>();
        let block_size = size_of::<u32>() + vec_block_size!(flat_triangle_indices);
        out.write_all(&(block_size as u32).to_le_bytes())?;
        vec_block_write!(out, flat_triangle_indices);
        // VERT section
        out.write_all(b"VERT")?;
        let flat_vert_chunks = self.vertices_chunks.iter().flat_map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>();
        let block_size = size_of::<u32>() + vec_block_size!(flat_vert_chunks);
        out.write_all(&(block_size as u32).to_le_bytes())?;
        vec_block_write!(out, flat_vert_chunks);
        // LIQU section
        if (self.liquidflags & 3) > 0 {
            let mut liqu_total_size = size_of::<u32>();
            if let Some(liq) = &self.liquid {
                liqu_total_size += liq.header.raw_size_of() + vec_block_size!(liq.liquid_heights) + vec_block_size!(liq.liquid_flags);
            }
            out.write_all(b"LIQU")?;
            out.write_all(&(liqu_total_size as u32).to_le_bytes())?;
            out.write_all(&self.liquid_type.to_le_bytes())?;
            if let Some(liq) = &self.liquid {
                out.write_all(&liq.header.xverts.to_le_bytes())?;
                out.write_all(&liq.header.yverts.to_le_bytes())?;
                out.write_all(&liq.header.xtiles.to_le_bytes())?;
                out.write_all(&liq.header.ytiles.to_le_bytes())?;
                out.write_all(&liq.header.pos_x.to_le_bytes())?;
                out.write_all(&liq.header.pos_y.to_le_bytes())?;
                out.write_all(&liq.header.pos_z.to_le_bytes())?;
                out.write_all(&liq.header.material.to_le_bytes())?;
                vec_block_write!(out, liq.liquid_heights);
                vec_block_write!(out, liq.liquid_flags);
            }
        }
        Ok(())
    }

    fn read<R: io::Read>(input: &mut R) -> io::Result<GroupModel_Raw> {
        let mogp_flags = read_le!(input, u32)?;
        let group_wmo_id = read_le!(input, u32)?;
        let bbcorn1 = Vector3::new(read_le!(input, f32)?, read_le!(input, f32)?, read_le!(input, f32)?);
        let bbcorn2 = Vector3::new(read_le!(input, f32)?, read_le!(input, f32)?, read_le!(input, f32)?);
        let liquidflags = read_le!(input, u32)?;

        // will this ever be used? what is it good for anyway??
        cmp_or_return!(input, b"GRP ")?;
        let _block_size = read_le!(input, u32)?;
        let branches = read_le!(input, u32)?;
        let mut n_bounding_triangles = Vec::with_capacity(branches as usize);
        for _ in 0..branches {
            n_bounding_triangles.push(read_le!(input, u16)?);
        }
        // ---- indexes
        cmp_or_return!(input, b"INDX")?;
        let _block_size = read_le!(input, u32)?;
        let mut nindexes = read_le!(input, u32)?;
        let mut mesh_triangle_indices = Vec::with_capacity((nindexes / 3) as usize);
        while nindexes > 0 {
            mesh_triangle_indices.push(Vector3::new(read_le!(input, u16)?, read_le!(input, u16)?, read_le!(input, u16)?));
            nindexes -= 3;
        }
        // ---- vectors
        cmp_or_return!(input, b"VERT")?;
        let _block_size = read_le!(input, u32)?;
        let mut nvectors = read_le!(input, u32)?;
        let mut vertices_chunks = Vec::with_capacity((nvectors / 3) as usize);
        while nvectors > 0 {
            vertices_chunks.push(Vector3::new(read_le!(input, f32)?, read_le!(input, f32)?, read_le!(input, f32)?));
            nvectors -= 3;
        }
        let mut liquid_type = 0;
        let mut liquid = None;
        if (liquidflags & 3) > 0 {
            cmp_or_return!(input, b"LIQU")?;
            let _block_size = read_le!(input, u32)?;
            liquid_type = read_le!(input, u32)?;
            if (liquidflags & 1) > 0 {
                let hlq = WMOLiquidHeader {
                    xverts:   read_le!(input, i32)?,
                    yverts:   read_le!(input, i32)?,
                    xtiles:   read_le!(input, i32)?,
                    ytiles:   read_le!(input, i32)?,
                    pos_x:    read_le!(input, f32)?,
                    pos_y:    read_le!(input, f32)?,
                    pos_z:    read_le!(input, f32)?,
                    material: read_le!(input, i16)?,
                };
                if hlq.xverts != hlq.xtiles + 1 {
                    panic!("SANITY CHECK, xverts {} must be 1 more than xtiles {}", hlq.xverts, hlq.xtiles);
                }
                if hlq.yverts != hlq.ytiles + 1 {
                    panic!("SANITY CHECK, yverts {} must be 1 more than ytiles {}", hlq.yverts, hlq.ytiles);
                }

                let size = (hlq.xverts * hlq.yverts) as usize;
                let mut liquid_heights = Vec::with_capacity(size);
                for _ in 0..size {
                    liquid_heights.push(read_le!(input, f32)?);
                }
                let size = (hlq.xtiles * hlq.ytiles) as usize;
                let mut liquid_flags = Vec::with_capacity(size);
                for _ in 0..size {
                    liquid_flags.push(read_le!(input, u8)?);
                }
                liquid = Some(GroupModel_Liquid_Raw {
                    header: hlq,
                    liquid_heights,
                    liquid_flags,
                })
            }
        }

        Ok(Self {
            mogp_flags,
            group_wmo_id,
            bbcorn1,
            bbcorn2,
            liquidflags,
            n_bounding_triangles,
            mesh_triangle_indices,
            vertices_chunks,
            liquid_type,
            liquid,
        })
    }
}

#[allow(non_camel_case_types)]
pub struct GroupModel_Liquid_Raw {
    pub header:         WMOLiquidHeader,
    pub liquid_heights: Vec<f32>,
    pub liquid_flags:   Vec<u8>,
}
