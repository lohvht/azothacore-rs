use std::{io, marker::PhantomData};

use nalgebra::{DMatrix, Vector3};
use parry3d::{bounding_volume::Aabb, partitioning::Qbvh, shape::TriMesh};

use crate::{
    cmp_or_return,
    collision::vmap_definitions::{LIQUID_TILE_SIZE, VMAP_MAGIC},
    read_le,
    utils::{bincode_deserialise, bincode_serialise},
    AzResult,
};

/// Holds a model (converted M2 or WMO) in its original coordinate space
pub struct WorldModel {
    root_wmoid:       u32,
    pub group_models: Vec<GroupModel>,
    group_tree:       Qbvh<usize>,
}

impl WorldModel {
    pub fn new(root_wmoid: u32, group_models: Vec<GroupModel>) -> Self {
        let mut group_tree = Qbvh::new();
        let bvh_data = group_models.iter().enumerate().map(|(idx, gm)| (idx, gm.i_bound));
        // NOTE: we apply no dilation factor because we won't
        // update this tree dynamically.
        group_tree.clear_and_rebuild(bvh_data, 0.0);
        Self {
            root_wmoid,
            group_models,
            group_tree,
        }
    }

    pub fn write_file<W: io::Write>(&self, mut out: &mut W) -> AzResult<()> {
        // FILE* wf = fopen(filename.c_str(), "wb");
        // if (!wf)
        //     return false;

        out.write_all(VMAP_MAGIC)?;
        out.write_all(b"WMOD")?;
        out.write_all(&self.root_wmoid.to_le_bytes()[..])?;

        // write group models
        let count = self.group_models.len();
        if count > 0 {
            out.write_all(b"GMOD")?;
            out.write_all(&count.to_le_bytes())?;
            for g in self.group_models.iter() {
                g.write_to_file(&mut out)?;
            }
            // write group BIH
            out.write_all(b"GBIH")?;
            bincode_serialise(&mut out, &self.group_tree)?;
        }

        Ok(())
    }

    pub fn read_file<R: io::Read>(r: &mut R) -> AzResult<Self> {
        let mut r = r;

        cmp_or_return!(r, VMAP_MAGIC)?;
        cmp_or_return!(r, b"WMOD")?;
        let root_wmoid = read_le!(r, u32)?;

        let (group_models, group_tree) = match cmp_or_return!(r, b"GMOD") {
            Err(e) if matches!(e.kind(), io::ErrorKind::UnexpectedEof) => {
                let gm = vec![];
                let gt = Qbvh::new();
                (gm, gt)
            },
            Err(e) => return Err(e)?,
            Ok(_) => {
                let count = read_le!(r, usize)?;
                let mut gm = Vec::with_capacity(count);
                for _ in 0..count {
                    gm.push(GroupModel::read_from_file(&mut r)?);
                }
                // Read group BIH
                cmp_or_return!(r, b"GBIH")?;
                let gt = bincode_deserialise(&mut r)?;
                (gm, gt)
            },
        };

        Ok(Self {
            root_wmoid,
            group_models,
            group_tree,
        })
    }
    //     //! pass group models to WorldModel and create BIH. Passed vector is swapped with old geometry!
    //     bool IntersectRay(const G3D::Ray &ray, float &distance, bool stopAtFirstHit, ModelIgnoreFlags ignoreFlags) const;
    //     bool IntersectPoint(const G3D::Vector3 &p, const G3D::Vector3 &down, float &dist, AreaInfo &info) const;
    //     bool GetLocationInfo(const G3D::Vector3 &p, const G3D::Vector3 &down, float &dist, LocationInfo &info) const;
    //     void getGroupModels(std::vector<GroupModel>& outGroupModels);
    //     uint32 Flags;
    // protected:
}

/// holding additional info for WMO group files
#[derive(serde::Deserialize, serde::Serialize)]
pub struct GroupModel {
    pub i_bound:       Aabb,
    /// 0x8 outdor; 0x2000 indoor
    pub i_mogp_flags:  u32,
    pub i_group_wmoid: u32,
    pub mesh:          Option<TriMesh>,
    pub i_liquid:      Option<WmoLiquid>,
    phantom:           PhantomData<()>,
}

impl GroupModel {
    pub fn new(
        i_mogp_flags: u32,
        i_group_wmoid: u32,
        i_bound: Aabb,
        mesh_triangle_indices: Vec<Vector3<u16>>,
        vertices_chunks: Vec<Vector3<f32>>,
        i_liquid: Option<WmoLiquid>,
    ) -> Self {
        let mesh = if vertices_chunks.is_empty() || mesh_triangle_indices.is_empty() {
            None
        } else {
            let vertices = vertices_chunks.into_iter().map(|v| v.into()).collect();
            let indices = mesh_triangle_indices
                .into_iter()
                .map(|i| [i.x.into(), i.y.into(), i.z.into()])
                .collect::<Vec<_>>();
            Some(TriMesh::new(vertices, indices))
        };

        Self {
            i_bound,
            i_mogp_flags,
            i_group_wmoid,
            i_liquid,
            mesh,
            phantom: PhantomData,
        }
    }

    fn write_to_file<W: io::Write>(&self, mut out: &mut W) -> AzResult<()> {
        bincode_serialise(&mut out, &self)?;
        Ok(())
    }

    fn read_from_file<R: io::Read>(mut r: &mut R) -> AzResult<Self> {
        let s = bincode_deserialise(&mut r)?;
        Ok(s)
    }

    // public:
    // GroupModel() : iBound(), iMogpFlags(0), iGroupWMOID(0), iLiquid(NULL) { }
    // GroupModel(const GroupModel &other);
    // GroupModel(uint32 mogpFlags, uint32 groupWMOID, const G3D::AABox &bound):
    //             iBound(bound), iMogpFlags(mogpFlags), iGroupWMOID(groupWMOID), iLiquid(NULL) { }
    // ~GroupModel() { delete iLiquid; }

    // //! pass mesh data to object and create BIH. Passed vectors get get swapped with old geometry!
    // void setMeshData(std::vector<G3D::Vector3> &vert, std::vector<MeshTriangle> &tri);
    // void setLiquidData(WmoLiquid*& liquid) { iLiquid = liquid; liquid = NULL; }
    // bool IntersectRay(const G3D::Ray &ray, float &distance, bool stopAtFirstHit) const;
    // bool IsInsideObject(const G3D::Vector3 &pos, const G3D::Vector3 &down, float &z_dist) const;
    // bool GetLiquidLevel(const G3D::Vector3 &pos, float &liqHeight) const;
    // uint32 GetLiquidType() const;
    // const G3D::AABox& GetBound() const { return iBound; }
    // uint32 GetMogpFlags() const { return iMogpFlags; }
    // uint32 GetWmoID() const { return iGroupWMOID; }
    // void getMeshData(std::vector<G3D::Vector3>& outVertices, std::vector<MeshTriangle>& outTriangles, WmoLiquid*& liquid);
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct WmoLiquid {
    ///!< liquid type
    pub i_type:  u32,
    pub heights: Result<WmoLiquidVertexHeightAndFlags, f32>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct WmoLiquidVertexHeightAndFlags {
    ///!< the lower corner
    pub i_corner: Vector3<f32>,
    /// height values in a matrix. indexed via (y, x)
    ///     => the y axis is the number of rows (i.e. height)
    ///     => the x axis is the number of cols (i.e. width)
    pub i_height: DMatrix<f32>,
    ///!< info if liquid tile is used
    pub i_flags:  DMatrix<u8>,
}

pub struct WmoLiquidParams {
    pub i_height: Vec<f32>,
    pub i_flags:  Vec<u8>,
    pub width:    usize,
    pub height:   usize,
    pub corner:   Vector3<f32>,
}

impl WmoLiquid {
    /// width => i_tiles_x
    /// height => i_tiles_y
    pub fn new(typ: u32, liq_data: Result<WmoLiquidParams, f32>) -> Self {
        let heights = match liq_data {
            Ok(WmoLiquidParams {
                i_height,
                i_flags,
                width,
                height,
                corner,
            }) => {
                let ncols = width + 1;
                let nrows = height + 1;
                // Initialised in reversed order, b/c the i_height and i_flags are supposed to be in row order, but `DMatrix::from_vec`
                // is filled in col order.
                let i_height = DMatrix::from_vec(ncols, nrows, i_height).transpose();
                let ncols = width;
                let nrows = height;
                let i_flags = DMatrix::from_vec(ncols, nrows, i_flags).transpose();
                Ok(WmoLiquidVertexHeightAndFlags {
                    i_corner: corner,
                    i_height,
                    i_flags,
                })
            },
            Err(h) => Err(h),
        };
        // iHeight[tx+1 +  ty    * rowOffset] - iHeight[tx   + ty * rowOffset]; ==> i_height[(ty, tx+1)] - i_height[(ty, tx)]
        WmoLiquid { i_type: typ, heights }
    }

    pub fn get_liquid_height(&self, pos: &Vector3<f32>) -> Option<f32> {
        // simple case
        let WmoLiquidVertexHeightAndFlags {
            i_flags,
            i_height,
            i_corner,
        } = match &self.heights {
            Err(liq_height) => return Some(*liq_height),
            Ok(hf) => hf,
        };

        let (i_tiles_y, i_tiles_x) = i_flags.shape();

        let tx_f = (pos.x - i_corner.x) / LIQUID_TILE_SIZE;
        let tx = tx_f.floor() as usize;
        if tx_f < 0.0 || tx >= i_tiles_x {
            return None;
        }
        let ty_f = (pos.y - i_corner.y) / LIQUID_TILE_SIZE;
        let ty = ty_f.floor() as usize;
        if ty_f < 0.0 || ty >= i_tiles_y {
            return None;
        }
        // check if tile shall be used for liquid level
        // checking for 0x08 *might* be enough, but disabled tiles always are 0x?F:
        if (i_flags[(ty, tx)] & 0x0F) == 0x0F {
            return None;
        }

        // (dx, dy) coordinates inside tile, in [0, 1]^2
        let dx = tx_f - tx as f32;
        let dy = ty_f - ty as f32;

        /* Tesselate tile to two triangles (not sure if client does it exactly like this)

            ^ dy
            |
          1 x---------x (1, 1)
            | (b)   / |
            |     /   |
            |   /     |
            | /   (a) |
            x---------x---> dx
          0           1
        */

        let h = if dx > dy {
            // case (a)
            let sx = i_height[(ty, tx + 1)] - i_height[(ty, tx)];
            let sy = i_height[(ty + 1, tx + 1)] - i_height[(ty, tx + 1)];
            i_height[(ty, tx)] + dx * sx + dy * sy
        } else {
            // case (b)
            let sx = i_height[(ty + 1, tx + 1)] - i_height[(ty + 1, tx)];
            let sy = i_height[(ty + 1, tx)] - i_height[(ty, tx)];
            i_height[(ty, tx)] + dx * sx + dy * sy
        };
        Some(h)
    }

    // public:
    //     WmoLiquid(uint32 width, uint32 height, const G3D::Vector3 &corner, uint32 type);
    //     WmoLiquid(const WmoLiquid &other);
    //     ~WmoLiquid();
    //     WmoLiquid& operator=(const WmoLiquid &other);
    //     bool GetLiquidHeight(const G3D::Vector3 &pos, float &liqHeight) const;
    //     uint32 GetType() const { return iType; }
    //     float *GetHeightStorage() { return iHeight; }
    //     uint8 *GetFlagsStorage() { return iFlags; }
    //     uint32 GetFileSize();
    //     bool writeToFile(FILE* wf);
    //     static bool readFromFile(FILE* rf, WmoLiquid* &liquid);
    // private:
    //     WmoLiquid() : iTilesX(0), iTilesY(0), iCorner(), iType(0), iHeight(NULL), iFlags(NULL) { }
}
