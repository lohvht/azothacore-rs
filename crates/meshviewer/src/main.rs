mod debug_files;
mod navmesh;

use std::{fmt::Write, fs, path::PathBuf};

use azothacore_common::{
    collision::maps::map_defines::MmapNavTerrainFlag,
    recastnavigation_handles::{DetourNavMesh, poly_area, poly_type},
};
use bevy::{
    asset::RenderAssetUsages,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    mesh::{PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};
use clap::Parser;
use recastnavigation_sys::dtPolyTypes_DT_POLYTYPE_OFFMESH_CONNECTION;

use crate::{
    debug_files::{read_detail_mesh, read_input_mesh, read_poly_mesh},
    navmesh::{PathResult, find_path, load_navmesh},
};

/// Shows the mmap generator's debug meshes and navmesh for one map, and tests paths on the navmesh.
#[derive(Parser, Resource)]
struct Args {
    map_id: u32,
    /// Debug meshes folder, written by the mmap generator with `debug_output = true`.
    #[arg(long, default_value = "data/meshes")]
    meshes: PathBuf,
    /// Navmesh folder (`.mmap`, `.mmtile`).
    #[arg(long, default_value = "data/mmaps")]
    mmaps:  PathBuf,
    /// Only load this tile, as X,Y like the generator log's `[X,Y]`. Repeatable.
    #[arg(long = "tile", value_parser = parse_tile)]
    tiles:  Vec<(u16, u16)>,
}

fn parse_tile(s: &str) -> Result<(u16, u16), String> {
    let (x, y) = s.split_once(',').ok_or("expected X,Y")?;
    Ok((x.trim().parse().map_err(|e| format!("{e}"))?, y.trim().parse().map_err(|e| format!("{e}"))?))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    Input,
    PolyMesh,
    DetailMesh,
    Navmesh,
    NavmeshEdges,
}

impl Layer {
    const ALL: [Layer; 5] = [Layer::Input, Layer::PolyMesh, Layer::DetailMesh, Layer::Navmesh, Layer::NavmeshEdges];

    fn label(self) -> &'static str {
        match self {
            Layer::Input => "input geometry (.mesh)",
            Layer::PolyMesh => "poly mesh (.pmesh)",
            Layer::DetailMesh => "detail mesh (.dmesh)",
            Layer::Navmesh => "navmesh (.mmtile)",
            Layer::NavmeshEdges => "navmesh polygon edges",
        }
    }

    fn key(self) -> KeyCode {
        match self {
            Layer::Input => KeyCode::Digit1,
            Layer::PolyMesh => KeyCode::Digit2,
            Layer::DetailMesh => KeyCode::Digit3,
            Layer::Navmesh => KeyCode::Digit4,
            Layer::NavmeshEdges => KeyCode::Digit5,
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Component)]
struct LayerTag(Layer);

#[derive(Resource)]
struct Layers {
    visible:   [bool; 5],
    triangles: [usize; 5],
    files:     [usize; 5],
}

/// Triangles that a click can land on.
#[derive(Resource, Default)]
struct PickTriangles {
    input:   Vec<[Vec3; 3]>,
    navmesh: Vec<[Vec3; 3]>,
}

struct NavMesh(Option<DetourNavMesh>);

#[derive(Resource, Default)]
struct PathTest {
    start:  Option<Vec3>,
    end:    Option<Vec3>,
    result: Option<Result<PathResult, String>>,
}

#[derive(Resource)]
struct LoadInfo(String);

#[derive(Component)]
struct Hud;

fn main() {
    let args = Args::parse();
    let (nav, nav_info) = match load_navmesh(&args.mmaps, args.map_id, &args.tiles) {
        Ok((nav, n)) => (Some(nav), format!("{n} navmesh tiles")),
        Err(e) => (None, format!("no navmesh: {e}")),
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("meshviewer: map {}", args.map_id),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FreeCameraPlugin)
        .insert_resource(ClearColor(Color::srgb(0.12, 0.12, 0.14)))
        .insert_resource(GlobalAmbientLight {
            brightness: 600.0,
            ..default()
        })
        .insert_resource(LoadInfo(nav_info))
        .insert_resource(args)
        .insert_non_send(NavMesh(nav))
        .init_resource::<PickTriangles>()
        .init_resource::<PathTest>()
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_layers, pick_path_points, draw_path, update_hud).chain())
        .run();
}

#[derive(Default)]
struct TriBuilder {
    positions: Vec<[f32; 3]>,
    colors:    Vec<[f32; 4]>,
}

impl TriBuilder {
    fn push(&mut self, tri: [[f32; 3]; 3], color: Color) {
        let c = color.to_linear().to_f32_array();
        self.positions.extend(tri);
        self.colors.extend([c; 3]);
    }

    fn build(self) -> Option<Mesh> {
        if self.positions.is_empty() {
            return None;
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.compute_flat_normals();
        Some(mesh)
    }
}

#[derive(Default)]
struct LineBuilder {
    positions: Vec<[f32; 3]>,
}

impl LineBuilder {
    fn push_loop(&mut self, corners: &[[f32; 3]]) {
        for (i, a) in corners.iter().enumerate() {
            self.positions.push(*a);
            self.positions.push(corners[(i + 1) % corners.len()]);
        }
    }

    fn build(self) -> Option<Mesh> {
        (!self.positions.is_empty())
            .then(|| Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions))
    }
}

fn area_color(area: u8) -> Color {
    use MmapNavTerrainFlag::*;
    match area {
        a if a == Ground.area_id() => Color::srgb(0.35, 0.75, 0.35),
        a if a == GroundSteep.area_id() => Color::srgb(0.85, 0.6, 0.25),
        a if a == Water.area_id() => Color::srgb(0.25, 0.5, 0.95),
        a if a == MagmaSlime.area_id() => Color::srgb(0.9, 0.25, 0.2),
        _ => Color::srgb(0.7, 0.7, 0.7),
    }
}

fn to_vec3s(tri: [[f32; 3]; 3]) -> [Vec3; 3] {
    tri.map(Vec3::from)
}

/// Debug files for the map, optionally only the given (x, y) tiles. The generator names them
/// `{map:04}{y:02}{x:02}`, so a tile filter on files from older builds with other names drops them.
fn debug_files(args: &Args, extension: &str) -> Vec<PathBuf> {
    let prefix = format!("{:04}", args.map_id);
    let wanted: Vec<_> = args.tiles.iter().map(|(x, y)| format!("{prefix}{y:02}{x:02}")).collect();
    let Ok(dir) = fs::read_dir(&args.meshes) else {
        return vec![];
    };
    let mut files: Vec<_> = dir
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == extension))
        .filter(|p| {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
            stem.starts_with(&prefix) && (wanted.is_empty() || wanted.iter().any(|w| w == stem))
        })
        .collect();
    files.sort();
    files
}

fn setup(
    mut commands: Commands,
    args: Res<Args>,
    nav: NonSend<NavMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut picks: ResMut<PickTriangles>,
) {
    let surface = |depth_bias: f32| StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        double_sided: true,
        cull_mode: None,
        depth_bias,
        ..default()
    };
    let input_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.55, 0.58),
        ..surface(-800.0)
    });
    let poly_mat = materials.add(surface(-600.0));
    let detail_mat = materials.add(surface(-600.0));
    let nav_mat = materials.add(surface(-400.0));
    // wgpu rejects depth bias on line topology, so the surfaces are pushed back instead.
    let edge_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.1, 0.05),
        unlit: true,
        ..default()
    });

    let mut layers = Layers {
        visible:   [true, false, false, true, true],
        triangles: [0; 5],
        files:     [0; 5],
    };
    let mut bounds = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let mut spawn = |mesh: Option<Mesh>, material: &Handle<StandardMaterial>, layer: Layer, layers: &mut Layers| {
        let Some(mesh) = mesh else { return };
        if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            for p in positions {
                bounds.0 = bounds.0.min(Vec3::from(*p));
                bounds.1 = bounds.1.max(Vec3::from(*p));
            }
        }
        let visibility = if layers.visible[layer.index()] {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone()), LayerTag(layer), visibility));
        layers.files[layer.index()] += 1;
    };

    for path in debug_files(&args, "mesh") {
        let input = match fs::read(&path).and_then(|b| read_input_mesh(&b)) {
            Ok(m) => m,
            Err(e) => {
                warn!("skipping {}: {e}", path.display());
                continue;
            },
        };
        let mut tris = TriBuilder::default();
        for t in input.triangles {
            tris.push(t, Color::WHITE);
            picks.input.push(to_vec3s(t));
        }
        layers.triangles[Layer::Input.index()] += tris.positions.len() / 3;
        spawn(tris.build(), &input_mat, Layer::Input, &mut layers);
    }

    for path in debug_files(&args, "pmesh") {
        let poly = match fs::read(&path).and_then(|b| read_poly_mesh(&b)) {
            Ok(m) => m,
            Err(e) => {
                warn!("skipping {}: {e}", path.display());
                continue;
            },
        };
        let mut tris = TriBuilder::default();
        for (corners, area) in &poly.polys {
            for i in 1..corners.len().saturating_sub(1) {
                tris.push([corners[0], corners[i], corners[i + 1]], area_color(*area));
            }
        }
        layers.triangles[Layer::PolyMesh.index()] += tris.positions.len() / 3;
        spawn(tris.build(), &poly_mat, Layer::PolyMesh, &mut layers);
    }

    for path in debug_files(&args, "dmesh") {
        let detail = match fs::read(&path).and_then(|b| read_detail_mesh(&b)) {
            Ok(m) => m,
            Err(e) => {
                warn!("skipping {}: {e}", path.display());
                continue;
            },
        };
        let mut tris = TriBuilder::default();
        for (i, submesh) in detail.submeshes.iter().enumerate() {
            let color = Color::hsl((i as f32 * 137.5) % 360.0, 0.45, 0.55);
            for t in submesh {
                tris.push(*t, color);
            }
        }
        layers.triangles[Layer::DetailMesh.index()] += tris.positions.len() / 3;
        spawn(tris.build(), &detail_mat, Layer::DetailMesh, &mut layers);
    }

    if let Some(nav) = &nav.0 {
        for tile in nav.tiles() {
            let mut tris = TriBuilder::default();
            tile.for_each_detail_triangle(|poly, t| {
                tris.push(t, area_color(poly_area(poly)));
                picks.navmesh.push(to_vec3s(t));
            });
            let mut edges = LineBuilder::default();
            for poly in tile.polys().iter().filter(|p| poly_type(p) != dtPolyTypes_DT_POLYTYPE_OFFMESH_CONNECTION) {
                edges.push_loop(&tile.poly_vertices(poly).collect::<Vec<_>>());
            }
            layers.triangles[Layer::Navmesh.index()] += tris.positions.len() / 3;
            spawn(tris.build(), &nav_mat, Layer::Navmesh, &mut layers);
            spawn(edges.build(), &edge_mat, Layer::NavmeshEdges, &mut layers);
        }
    }

    let (center, size) = if bounds.0.x <= bounds.1.x {
        ((bounds.0 + bounds.1) / 2.0, (bounds.1 - bounds.0).max_element().max(50.0))
    } else {
        (Vec3::ZERO, 500.0)
    };
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection { far: 100_000.0, ..default() }),
        Transform::from_translation(center + Vec3::new(0.0, size * 0.6, size * 0.6)).looking_at(center, Vec3::Y),
        FreeCamera {
            walk_speed: 60.0,
            run_speed: 300.0,
            ..default()
        },
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 6000.0,
            ..default()
        },
        Transform::from_xyz(0.3, 1.0, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        },
    ));
    commands.insert_resource(layers);
}

fn toggle_layers(keys: Res<ButtonInput<KeyCode>>, mut layers: ResMut<Layers>, mut tagged: Query<(&LayerTag, &mut Visibility)>) {
    let mut changed = false;
    for layer in Layer::ALL {
        if keys.just_pressed(layer.key()) {
            layers.visible[layer.index()] ^= true;
            changed = true;
        }
    }
    if !changed {
        return;
    }
    for (tag, mut visibility) in &mut tagged {
        *visibility = if layers.visible[tag.0.index()] {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn ray_triangle(ray: Ray3d, [a, b, c]: &[Vec3; 3]) -> Option<f32> {
    let dir = *ray.direction;
    let (e1, e2) = (*b - *a, *c - *a);
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-8 {
        return None;
    }
    let inv = 1.0 / det;
    let s = ray.origin - *a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 0.0).then_some(t)
}

#[expect(clippy::too_many_arguments)]
fn pick_path_points(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    layers: Res<Layers>,
    picks: Res<PickTriangles>,
    nav: NonSend<NavMesh>,
    mut path: ResMut<PathTest>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        *path = PathTest::default();
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right) {
        return;
    }
    let Some(cursor) = window.cursor_position() else { return };
    let (camera, camera_transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };

    let (show_input, show_nav) = (layers.visible[Layer::Input.index()], layers.visible[Layer::Navmesh.index()]);
    let targets: Vec<&[[Vec3; 3]]> = match (show_input, show_nav) {
        (true, false) => vec![&picks.input],
        (false, true) => vec![&picks.navmesh],
        _ => vec![&picks.input, &picks.navmesh],
    };
    let hit = targets
        .into_iter()
        .flatten()
        .filter_map(|t| ray_triangle(ray, t))
        .min_by(f32::total_cmp)
        .map(|t| ray.get_point(t));
    let Some(hit) = hit else { return };

    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        path.start = Some(hit);
    } else {
        path.end = Some(hit);
    }
    let (Some(start), Some(end)) = (path.start, path.end) else {
        path.result = None;
        return;
    };
    path.result = Some(match &nav.0 {
        Some(nav) => find_path(nav, start.into(), end.into()),
        None => Err("no navmesh loaded".into()),
    });
}

fn draw_path(path: Res<PathTest>, nav: NonSend<NavMesh>, mut gizmos: Gizmos) {
    let lift = Vec3::Y * 0.3;
    if let Some(p) = path.start {
        gizmos.sphere(Isometry3d::from_translation(p), 0.8, Color::srgb(0.1, 0.9, 0.1));
    }
    if let Some(p) = path.end {
        gizmos.sphere(Isometry3d::from_translation(p), 0.8, Color::srgb(0.95, 0.15, 0.1));
    }
    let (Some(Ok(result)), Some(nav)) = (&path.result, &nav.0) else { return };
    for poly in &result.corridor {
        let Some(corners) = nav.poly_vertices(*poly) else { continue };
        gizmos.linestrip(corners.iter().chain(corners.first()).map(|v| Vec3::from(*v) + lift), Color::srgb(0.2, 0.8, 0.9));
    }
    gizmos.linestrip(result.points.iter().map(|v| Vec3::from(*v) + lift * 2.0), Color::srgb(1.0, 0.85, 0.1));
}

/// Recast space is (WoW y, WoW z, WoW x).
fn wow_coords(v: Vec3) -> String {
    format!("({:.2}, {:.2}, {:.2})", v.z, v.x, v.y)
}

fn update_hud(
    args: Res<Args>,
    info: Res<LoadInfo>,
    layers: Res<Layers>,
    path: Res<PathTest>,
    picks: Res<PickTriangles>,
    mut hud: Single<&mut Text, With<Hud>>,
) {
    if !(layers.is_changed() || path.is_changed()) {
        return;
    }
    let mut s = format!("Map {}: {}\n", args.map_id, info.0);
    for layer in Layer::ALL {
        let i = layer.index();
        let state = if layers.visible[i] { "on " } else { "off" };
        let _ = writeln!(
            s,
            "[{}] {state} {:<24} {} meshes, {} tris",
            i + 1,
            layer.label(),
            layers.files[i],
            layers.triangles[i]
        );
    }
    s.push_str("Right mouse: look   WASD/QE: move   Shift: faster   Scroll: speed\n");
    s.push_str("Click: path end   Shift+click: path start   C: clear\n");
    if picks.input.is_empty() && picks.navmesh.is_empty() {
        s.push_str("Nothing to click on: no input geometry or navmesh loaded\n");
    }
    if let Some(p) = path.start {
        let _ = writeln!(s, "Start (WoW x, y, z): {}", wow_coords(p));
    }
    if let Some(p) = path.end {
        let _ = writeln!(s, "End   (WoW x, y, z): {}", wow_coords(p));
    }
    match &path.result {
        None => {},
        Some(Err(e)) => {
            let _ = writeln!(s, "Path: {e}");
        },
        Some(Ok(r)) => {
            let length: f32 = r.points.windows(2).map(|w| Vec3::from(w[0]).distance(Vec3::from(w[1]))).sum();
            let _ = writeln!(
                s,
                "Path: {} polys, {} points, {length:.1} yd{}\nSnapped start {} end {}",
                r.corridor.len(),
                r.points.len(),
                if r.partial { " (PARTIAL: end not reachable)" } else { "" },
                wow_coords(Vec3::from(r.start)),
                wow_coords(Vec3::from(r.end)),
            );
        },
    }
    hud.0 = s;
}
