use std::{io, ops::Deref, sync::Arc};

use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    bvh::BVH,
};
use nalgebra::{DMatrix, Vector3};

use crate::{
    cmp_or_return,
    common::collision::vmap_definitions::VMAP_MAGIC,
    read_le,
    tools::extractor_common::{bincode_deserialise, bincode_serialise},
    AzResult,
};

/// Holds a model (converted M2 or WMO) in its original coordinate space
pub struct WorldModel {
    root_wmoid:       u32,
    pub group_models: Vec<GroupModel>,
    group_tree:       BVH,
}

impl WorldModel {
    pub fn new(root_wmoid: u32, group_models: Vec<GroupModel>) -> Self {
        let mut s = Self {
            root_wmoid,
            group_models,
            group_tree: BVH { nodes: vec![] },
        };
        if !s.group_models.is_empty() {
            s.group_tree = BVH::build(&mut s.group_models);
        }

        s
    }

    pub fn write_file<W: io::Write>(&self, out: &mut W) -> AzResult<()> {
        let mut out = out;
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
                let gt = BVH { nodes: vec![] };
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

impl Bounded for GroupModel {
    fn aabb(&self) -> AABB {
        self.inner.i_bound
    }
}

impl BHShape for GroupModel {
    fn bh_node_index(&self) -> usize {
        self.inner.bvh_node_id
    }

    fn set_bh_node_index(&mut self, idx: usize) {
        self.inner.bvh_node_id = idx
    }
}

/// holding additional info for WMO group files
pub struct GroupModel {
    pub vertices:  Arc<Vec<Vector3<f32>>>,
    pub triangles: Vec<MeshTriangle>,
    inner:         GroupModelInner,
}

impl Deref for GroupModel {
    type Target = GroupModelInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GroupModelInner {
    i_bound:       AABB,
    /// 0x8 outdor; 0x2000 indoor
    i_mogp_flags:  u32,
    i_group_wmoid: u32,
    mesh_tree:     BVH,
    pub i_liquid:  Option<WmoLiquid>,
    bvh_node_id:   usize,
}

pub struct MeshTriangle {
    inner:    MeshTriangleInner,
    vertices: Arc<Vec<Vector3<f32>>>,
}

impl Deref for MeshTriangle {
    type Target = MeshTriangleInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MeshTriangleInner {
    node_idx:      usize,
    pub tri_idxes: Vector3<u16>,
}

impl Bounded for MeshTriangle {
    fn aabb(&self) -> AABB {
        let mut bound = AABB::empty();

        bound.grow_mut(&self.vertices[self.inner.tri_idxes.x as usize].into());
        bound.grow_mut(&self.vertices[self.inner.tri_idxes.y as usize].into());
        bound.grow_mut(&self.vertices[self.inner.tri_idxes.z as usize].into());
        bound
    }
}

impl BHShape for MeshTriangle {
    fn bh_node_index(&self) -> usize {
        self.inner.node_idx
    }

    fn set_bh_node_index(&mut self, idx: usize) {
        self.inner.node_idx = idx
    }
}

impl GroupModel {
    pub fn new(
        i_mogp_flags: u32,
        i_group_wmoid: u32,
        i_bound: AABB,
        mesh_triangle_indices: Vec<Vector3<u16>>,
        vertices_chunks: Vec<Vector3<f32>>,
        i_liquid: Option<WmoLiquid>,
    ) -> Self {
        let mut s = Self {
            inner:     GroupModelInner {
                i_bound,
                i_mogp_flags,
                i_group_wmoid,
                mesh_tree: BVH { nodes: vec![] },
                i_liquid,
                bvh_node_id: 0,
            },
            vertices:  Arc::new(vertices_chunks),
            triangles: vec![],
        };
        for tri in mesh_triangle_indices {
            s.triangles.push(MeshTriangle {
                inner:    MeshTriangleInner {
                    node_idx:  0,
                    tri_idxes: tri,
                },
                vertices: s.vertices.clone(),
            });
        }
        if !s.triangles.is_empty() {
            s.inner.mesh_tree = BVH::build(&mut s.triangles);
        }

        s
    }

    fn write_to_file<W: io::Write>(&self, out: &mut W) -> AzResult<()> {
        let mut out = out;

        let vert = self.vertices.iter().collect::<Vec<_>>();
        bincode_serialise(&mut out, &vert)?;

        let trim = self.triangles.iter().map(|t| &t.inner).collect::<Vec<_>>();
        bincode_serialise(&mut out, &trim)?;

        bincode_serialise(&mut out, &self.inner)?;
        Ok(())
    }

    fn read_from_file<R: io::Read>(r: &mut R) -> AzResult<Self> {
        let mut r = r;
        // Do all the reading first
        let vertices: Vec<Vector3<f32>> = bincode_deserialise(&mut r)?;
        let triangle_inner: Vec<MeshTriangleInner> = bincode_deserialise(&mut r)?;
        let inner = bincode_deserialise(&mut r)?;

        // Re-form Group
        let vertices = Arc::new(vertices);
        let triangles = triangle_inner
            .into_iter()
            .map(|ti| MeshTriangle {
                inner:    ti,
                vertices: vertices.clone(),
            })
            .collect();
        Ok(Self {
            inner,
            vertices,
            triangles,
        })
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WmoLiquid {
    ///!< the lower corner
    pub i_corner: Vector3<f32>,
    ///!< liquid type
    pub i_type:   u32,
    /// height values in a matrix. indexed via (y, x)
    ///     => the y axis is the number of rows (i.e. height)
    ///     => the x axis is the number of cols (i.e. width)
    pub i_height: DMatrix<f32>,
    ///!< info if liquid tile is used
    pub i_flags:  Option<DMatrix<u8>>,
}

impl WmoLiquid {
    /// width => i_tiles_x
    /// height => i_tiles_y
    pub fn new(width: u32, height: u32, corner: Vector3<f32>, typ: u32, i_height: Vec<f32>, i_flags: Vec<u8>) -> Self {
        let ncols = (width + 1) as usize;
        let nrows = (height + 1) as usize;
        // Initialised in reversed order, b/c the i_height and i_flags are supposed to be in row order, but `DMatrix::from_vec`
        // is filled in col order.
        let i_height = DMatrix::from_vec(ncols, nrows, i_height).transpose();
        let ncols = width as usize;
        let nrows = height as usize;
        let i_flags = Some(DMatrix::from_vec(ncols, nrows, i_flags).transpose());

        // iHeight[tx+1 +  ty    * rowOffset] - iHeight[tx   + ty * rowOffset]; ==> i_height[(ty, tx+1)] - i_height[(ty, tx)]

        WmoLiquid {
            i_corner: corner,
            i_type: typ,
            i_height,
            i_flags,
        }
    }

    pub fn new_without_flags(height: f32) -> Self {
        Self {
            i_corner: Vector3::zeros(),
            i_type:   0,
            i_height: DMatrix::from_vec(1, 1, vec![height]),
            i_flags:  None,
        }
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
