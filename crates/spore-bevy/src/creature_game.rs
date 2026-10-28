//! GPU-rendered Creature gameplay baseline using the deterministic core rules.
//! Default art is procedural; an optional source-art inspection path is available.
//! Retail assembly, animation and gameplay fidelity remain separate gates.

use bevy::{
    asset::RenderAssetUsages,
    mesh::Indices,
    image::{ImageSampler, ImageSamplerDescriptor, ImageAddressMode, ImageFilterMode},
    prelude::*,
    render::{
        render_resource::{PrimitiveTopology, Extent3d, TextureDimension, TextureFormat},
        renderer::RenderAdapterInfo,
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use spore_core::creature_game::{
    CreatureActor, CreatureInput, CreatureSnapshot, CreatureState, CreatureVec2, CreatureWorld,
    Diet, FoodKind, SocialAction,
};
use spore_core::editor::{Creation, CreationKind, GameplayLoadout};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Resource)]
struct Game {
    world: CreatureWorld,
    paused: bool,
    elapsed: f64,
    notice: String,
    rebuild: bool,
    target: Option<u32>,
    save_path: PathBuf,
    shot: Option<PathBuf>,
    demo_seconds: Option<f64>,
    capture_frame: Option<u64>,
    frames: u64,
}
impl Game {
    fn new(seed: u64, diet: Diet, save_path: PathBuf) -> Self {
        Self {
            world: CreatureWorld::new(seed, diet),
            paused: false,
            elapsed: 0.0,
            notice: String::new(),
            rebuild: true,
            target: None,
            save_path,
            shot: None,
            demo_seconds: None,
            capture_frame: None,
            frames: 0,
        }
    }
}

#[derive(Resource)]
struct Art {
    sphere: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    ring: Handle<Mesh>,
    player: Handle<StandardMaterial>,
    species: Vec<Handle<StandardMaterial>>,
    white: Handle<StandardMaterial>,
    dark: Handle<StandardMaterial>,
    feet: Handle<StandardMaterial>,
    plant: Handle<StandardMaterial>,
    meat: Handle<StandardMaterial>,
    bone: Handle<StandardMaterial>,
    home: Handle<StandardMaterial>,
    nest: Handle<StandardMaterial>,
    focus: Handle<StandardMaterial>,
    pack: Handle<StandardMaterial>,
}
#[derive(Resource, Default)]
struct Objects(HashMap<(u8, u32), Entity>);
#[derive(Resource, Default)]
struct AuthoredArt {
    creation: Option<Creation>,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}
#[derive(Resource)]
struct SourceInspection {
    asset: crate::assets::AuthoredCreatureAsset,
    sampler: ImageSamplerDescriptor,
}
#[derive(Resource)]
struct SourceInspectionRender { mesh: Handle<Mesh>, material: Handle<StandardMaterial> }
#[derive(Resource, Default)]
struct OffscreenSimulationReady(bool);
#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    distance: f32,
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 2.25,
            pitch: 0.62,
            distance: 20.0,
        }
    }
}
#[derive(Component)]
struct FollowCamera;
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct TargetHud;
#[derive(Component)]
struct SpeciesHud;
#[derive(Component)]
struct NoticeHud;
#[derive(Component)]
struct Banner;
#[derive(Component)]
struct Meter(u8);
#[derive(Component)]
struct Leg {
    actor: u32,
    phase: f32,
}
#[derive(Component)]
struct PackMark(u32);
#[derive(Component)]
struct TargetMark;

pub fn run(args: &[String]) -> Result<(), String> {
    let mut seed = 14;
    let mut diet = Diet::Herbivore;
    let mut explicit_diet = false;
    let mut creation_path = None;
    let mut shot = None;
    let mut demo_seconds = None;
    let mut source_install: Option<PathBuf> = None;
    let mut source_id: Option<u64> = None;
    let mut offscreen = false;
    let mut save_path = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RustSpore")
        .join("saves")
        .join("creature.rsp");
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--offscreen" { offscreen = true; index += 1; continue; }
        let value = || {
            args.get(index + 1)
                .ok_or_else(|| format!("{} requires a value", args[index]))
        };
        match args[index].as_str() {
            "--seed" => seed = value()?.parse().map_err(|_| "invalid seed".to_owned())?,
            "--diet" => {
                explicit_diet = true;
                diet = match value()?.as_str() {
                    "herbivore" => Diet::Herbivore,
                    "carnivore" => Diet::Carnivore,
                    "omnivore" => Diet::Omnivore,
                    _ => return Err("diet must be herbivore, carnivore or omnivore".into()),
                }
            }
            "--creation" => creation_path = Some(PathBuf::from(value()?)),
            "--source-creature-install" => source_install = Some(PathBuf::from(value()?)),
            "--source-creature-id" => {
                let text = value()?.trim();
                let text = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text);
                source_id = Some(u64::from_str_radix(text, 16).map_err(|_| "source Creature ID must be hexadecimal u64".to_string())?);
            }
            "--shot" => shot = Some(PathBuf::from(value()?)),
            "--save-file" => save_path = PathBuf::from(value()?),
            "--demo" => {
                let duration: f64 = value()?
                    .parse()
                    .map_err(|_| "invalid demo duration".to_owned())?;
                if !duration.is_finite() || duration <= 0.0 || duration > 600.0 {
                    return Err("demo duration must be in (0, 600] seconds".into());
                }
                demo_seconds = Some(duration);
            }
            other => return Err(format!("unknown Creature option {other}")),
        }
        index += 2;
    }
    if source_install.is_some() != source_id.is_some() { return Err("--source-creature-install and --source-creature-id must be provided together".into()); }
    if source_install.is_some() && creation_path.is_some() { return Err("source Creature inspection cannot be combined with --creation".into()); }
    if offscreen && shot.is_none() { return Err("--offscreen requires --shot <new PNG>".into()); }
    if shot.as_ref().is_some_and(|path| path.exists()) {
        return Err("screenshot already exists; choose a new output path".into());
    }
    if let Some(parent) = shot
        .as_ref()
        .and_then(|path| path.parent())
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let source = if let (Some(install), Some(id)) = (&source_install, source_id) {
        Some(load_source_inspection(install, id)?)
    } else { None };
    let world = configured_world(seed, diet, explicit_diet, creation_path.as_deref())?;
    let mut game = Game::new(seed, world.player_diet, save_path);
    game.world = world;
    game.shot = shot;
    game.demo_seconds = demo_seconds;
    let mut app = App::new();
    if offscreen {
        crate::offscreen::configure(&mut app, 1280, 800);
        app.init_resource::<OffscreenSimulationReady>();
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore | Creature".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }));
    }
    if let Some(source) = source { app.insert_resource(source); }
    app.insert_resource(ClearColor(Color::srgb(0.38, 0.57, 0.67)))
        .insert_resource(game)
        .init_resource::<Objects>()
        .init_resource::<AuthoredArt>()
        .init_resource::<Orbit>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                drive,
                sync_scene,
                animate_legs,
                follow_camera,
                update_hud,
                capture,
            )
                .chain(),
        )
        .run();
    Ok(())
}

fn load_source_inspection(install: &Path, instance: u64) -> Result<SourceInspection, String> {
    let base = install.join("Data/Spore_Content.package");
    let source = crate::assets::AssetSource::open(&base)?;
    let paths = crate::assets::PackageSet::discover(install).paths().to_vec();
    let source = source.with_set(crate::assets::PackageSet::from_paths(paths.clone()));
    let asset = source.authored_creature_asset(instance, 0).map_err(|e| e.to_string())?;
    let tex = &asset.texture;
    if tex.width == 0 || tex.height == 0 || tex.width > 4096 || tex.height > 4096
        || (tex.width as usize).checked_mul(tex.height as usize).and_then(|n| n.checked_mul(4)) != Some(tex.rgba.len()) {
        return Err("source Creature texture dimensions or decoded extent unsupported".into());
    }
    let texture = asset.texture_binding;
    if texture.availability() != Some(1) { return Err("source sampler unavailable".into()); }
    let address = match texture.address_mode() {
        Some(1) => ImageAddressMode::Repeat, Some(2) => ImageAddressMode::MirrorRepeat,
        Some(3) => ImageAddressMode::ClampToEdge, _ => return Err("source sampler addressing unsupported".into()),
    };
    let filter = match texture.min_mag_filter() {
        Some(1) => ImageFilterMode::Nearest, Some(2) => ImageFilterMode::Linear,
        _ => return Err("source sampler filtering unsupported".into()),
    };
    let mip = match texture.mip_filter() {
        Some(0 | 1) => ImageFilterMode::Nearest, Some(2) => ImageFilterMode::Linear,
        _ => return Err("source sampler mip filtering unsupported".into()),
    };
    let sampler = ImageSamplerDescriptor { address_mode_u: address, address_mode_v: address, address_mode_w: address,
        min_filter: filter, mag_filter: filter, mipmap_filter: mip, lod_min_clamp: 0.0, lod_max_clamp: 0.0, ..default() };
    let (min, max) = asset.mesh.bounds().ok_or("source Creature mesh is empty")?;
    let extent = (max.x-min.x).max(max.y-min.y).max(max.z-min.z);
    let center = [(min.x+max.x)*0.5, (min.y+max.y)*0.5, (min.z+max.z)*0.5];
    if !extent.is_finite() || extent <= 1e-6 || center.iter().any(|v| !v.is_finite()) {
        return Err("source Creature bounds invalid".into());
    }
    let lookup = crate::assets::PackageSet::from_paths(paths);
    for (role, key) in [("creature", asset.reference.creature), ("prop", asset.reference.prop),
        ("model", asset.reference.model), ("texture", asset.texture_key)] {
        let path = if source.resource_keys().any(|k| k == key) { base.clone() } else {
            lookup.locate(key.type_id, key.scoped_instance()).ok_or("source resource provenance unavailable")?
        };
        println!("CREATURE_SOURCE_RESOURCE role={role} package={} key={key}", path.display());
    }
    println!("CREATURE_SOURCE_INSPECTION instance={instance:016X} assignment=controlled_prototype_player renderer=pbr pose=static runtime_behavior=unverified overlay=inspection_order camera_scale=inspection basis=source_y_z_x_inference ground=source_z_min mip_chain=base_only saves=disabled");
    Ok(SourceInspection { asset, sampler })
}

/// Inspection inference: map source +Z up and +Y forward to Bevy +Y up and
/// the prototype actor's +X forward. This is not the original runtime pose.
fn source_to_bevy_basis(v: [f32; 3]) -> [f32; 3] { [v[1], v[2], v[0]] }

fn configured_world(
    seed: u64,
    diet: Diet,
    explicit_diet: bool,
    path: Option<&Path>,
) -> Result<CreatureWorld, String> {
    let Some(path) = path else {
        return Ok(CreatureWorld::new(seed, diet));
    };
    let creation = crate::cell_game::read_creation(path)?;
    if creation.kind != CreationKind::Creature {
        return Err("Creature gameplay requires a Creature creation".into());
    }
    let authored_diet = match creation
        .gameplay_loadout()
        .map_err(|error| error.to_string())?
    {
        GameplayLoadout::Creature(loadout) => loadout.diet,
        _ => return Err("Creature gameplay requires a Creature creation".into()),
    };
    if explicit_diet && authored_diet != diet {
        return Err(
            "--diet contradicts the authored mouth; edit the creation or omit --diet".into(),
        );
    }
    let mut world = CreatureWorld::new(seed, authored_diet);
    world
        .apply_creation(creation)
        .map_err(|error| error.to_string())?;
    crate::editor::creation_mesh(world.creation.as_ref().unwrap(), false)?;
    Ok(world)
}

fn terrain_height(x: f32, z: f32) -> f32 {
    0.18 * (x * 0.13).sin() * (z * 0.17).cos() + 0.07 * (z * 0.31).sin()
}
fn location(position: CreatureVec2, up: f32) -> Vec3 {
    Vec3::new(
        position.x,
        terrain_height(position.x, position.y) + up,
        position.y,
    )
}
fn terrain(half: CreatureVec2) -> Mesh {
    let (nx, nz) = (64, 48);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    for z in 0..=nz {
        for x in 0..=nx {
            let px = (x as f32 / nx as f32 * 2.0 - 1.0) * half.x;
            let pz = (z as f32 / nz as f32 * 2.0 - 1.0) * half.y;
            let dx = 0.18 * 0.13 * (px * 0.13).cos() * (pz * 0.17).cos();
            let dz = -0.18 * 0.17 * (px * 0.13).sin() * (pz * 0.17).sin()
                + 0.07 * 0.31 * (pz * 0.31).cos();
            positions.push([px, terrain_height(px, pz), pz]);
            normals.push(Vec3::new(-dx, 1.0, -dz).normalize().to_array());
            uvs.push([x as f32 / nx as f32, z as f32 / nz as f32]);
            let blend = ((px * 0.18).sin() * (pz * 0.22).cos() * 0.5 + 0.5) * 0.12;
            colors.push([0.31 + blend, 0.43 + blend, 0.24 + blend * 0.5, 1.0]);
            if x < nx && z < nz {
                let a = (z * (nx + 1) + x) as u32;
                let b = a + (nx + 1) as u32;
                indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            }
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
fn material(materials: &mut Assets<StandardMaterial>, color: Color) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.88,
        ..default()
    })
}
fn setup(
    mut commands: Commands,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    adapter: Res<RenderAdapterInfo>,
    source: Option<Res<SourceInspection>>,
    mut images: ResMut<Assets<Image>>,
    mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>,
) {
    println!(
        "CREATURE_GPU name={:?} backend={:?} type={:?}",
        adapter.name, adapter.backend, adapter.device_type
    );
    let source_inspection = source.is_some();
    if let Some(source) = source {
        let data = &source.asset.mesh;
        let (min, max) = data.bounds().expect("source bounds checked before app startup");
        let center = [(min.x+max.x)*0.5, (min.y+max.y)*0.5, min.z];
        let scale = 2.0 / (max.x-min.x).max(max.y-min.y).max(max.z-min.z);
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.iter().map(|p| source_to_bevy_basis([(p.x-center[0])*scale, (p.y-center[1])*scale, (p.z-center[2])*scale])).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.iter().map(|p| source_to_bevy_basis([p.x,p.y,p.z])).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());
        mesh.insert_indices(Indices::U32(data.indices.clone()));
        let texture = &source.asset.texture;
        let mut image = Image::new(Extent3d { width: texture.width, height: texture.height, depth_or_array_layers: 1 }, TextureDimension::D2, texture.rgba.clone(), TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
        image.sampler = ImageSampler::Descriptor(source.sampler.clone());
        let image = images.add(image);
        if let Some(readiness) = readiness.as_mut() { readiness.require_image(&image); }
        let material = materials.add(StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(image), perceptual_roughness: 0.85, cull_mode: None, double_sided: true, ..default() });
        commands.insert_resource(SourceInspectionRender { mesh: meshes.add(mesh), material });
    }
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            color: Color::srgb(0.85, 0.93, 1.0),
            brightness: 500.0,
            ..default()
        },
        Transform::from_xyz(-12.0, 12.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        FollowCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.95, -0.6, 0.0)),
    ));
    let ground = material(&mut materials, Color::WHITE);
    commands.spawn((
        Mesh3d(meshes.add(terrain(game.world.config.half_extents))),
        MeshMaterial3d(ground),
        Transform::default(),
    ));
    let sphere = meshes.add(Sphere::new(1.0));
    let cylinder = meshes.add(Cylinder::new(1.0, 1.0));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let ring = meshes.add(Annulus::new(0.93, 1.0));
    let rock = material(&mut materials, Color::srgb(0.37, 0.38, 0.31));
    let grass = material(&mut materials, Color::srgb(0.28, 0.41, 0.18));
    // Small ground detail has no collision; all blocking boundaries are the arena edge.
    for index in 0..180 {
        let x = ((index * 137 + 11) % 1231) as f32 * 0.1 - 61.5;
        let z = ((index * 89 + 41) % 911) as f32 * 0.1 - 45.5;
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(grass.clone()),
            Transform::from_translation(location(CreatureVec2::new(x, z), 0.16))
                .with_scale(Vec3::new(0.12, 0.32 + (index % 4) as f32 * 0.1, 0.1))
                .with_rotation(Quat::from_rotation_y(index as f32)),
        ));
    }
    for index in 0..44 {
        let angle = index as f32 * std::f32::consts::TAU / 44.0;
        let position = CreatureVec2::new(
            angle.cos() * game.world.config.half_extents.x,
            angle.sin() * game.world.config.half_extents.y,
        );
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(rock.clone()),
            Transform::from_translation(location(position, 0.5)).with_scale(Vec3::new(
                1.3 + (index % 3) as f32 * 0.4,
                1.0,
                1.5,
            )),
        ));
    }
    commands.insert_resource(Art {
        sphere,
        cylinder,
        ring,
        player: material(&mut materials, Color::srgb(0.13, 0.68, 0.61)),
        species: [
            Color::srgb(0.76, 0.53, 0.24),
            Color::srgb(0.57, 0.45, 0.74),
            Color::srgb(0.76, 0.27, 0.24),
            Color::srgb(0.35, 0.57, 0.76),
        ]
        .into_iter()
        .map(|color| material(&mut materials, color))
        .collect(),
        white: material(&mut materials, Color::srgb(0.96, 0.94, 0.78)),
        dark: material(&mut materials, Color::srgb(0.035, 0.065, 0.07)),
        feet: material(&mut materials, Color::srgb(0.19, 0.27, 0.21)),
        plant: material(&mut materials, Color::srgb(0.72, 0.84, 0.18)),
        meat: material(&mut materials, Color::srgb(0.87, 0.30, 0.28)),
        bone: material(&mut materials, Color::srgb(0.96, 0.86, 0.56)),
        home: material(&mut materials, Color::srgb(0.24, 0.83, 0.77)),
        nest: material(&mut materials, Color::srgb(0.47, 0.33, 0.18)),
        focus: material(&mut materials, Color::srgb(0.97, 0.82, 0.24)),
        pack: material(&mut materials, Color::srgb(0.35, 0.92, 0.89)),
    });
    ui(&mut commands, source_inspection);
}

fn ui(commands: &mut Commands, source_inspection: bool) {
    let panel = Color::srgba(0.035, 0.065, 0.075, 0.92);
    let muted = Color::srgb(0.67, 0.78, 0.76);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(18),
                left: px(22),
                right: px(22),
                height: px(86),
                padding: UiRect::all(px(15)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(20),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(panel),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new("CREATURE\nTHE FIRST FOOTSTEPS"),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::srgb(0.52, 0.94, 0.82)),
                Node {
                    width: px(225),
                    ..default()
                },
            ));
            row.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.92, 0.82)),
                Node {
                    width: px(285),
                    ..default()
                },
                Hud,
            ));
            for (number, name, color) in [
                (0, "HEALTH", Color::srgb(0.41, 0.83, 0.60)),
                (1, "STAMINA", Color::srgb(0.39, 0.68, 0.92)),
                (2, "HUNGER", Color::srgb(0.91, 0.73, 0.35)),
            ] {
                row.spawn(Node {
                    width: px(150),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(9),
                    ..default()
                })
                .with_children(|group| {
                    group.spawn((
                        Text::new(name),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(muted),
                    ));
                    group
                        .spawn((
                            Node {
                                height: px(7),
                                width: percent(100),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.19, 0.19)),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                Node {
                                    height: percent(100),
                                    width: percent(100),
                                    ..default()
                                },
                                BackgroundColor(color),
                                Meter(number),
                            ));
                        });
                });
            }
        });
    commands.spawn((Node { position_type: PositionType::Absolute, left: px(22), top: px(120), width: px(280), padding: UiRect::all(px(14)),
        border_radius: BorderRadius::all(px(10)), flex_direction: FlexDirection::Column, row_gap: px(12), ..default() }, BackgroundColor(panel))).with_children(|column| {
        column.spawn((Text::new("A PATH TO THE TRIBE"), TextFont { font_size: 15.0, ..default() }, TextColor(Color::srgb(0.96,0.82,0.48))));
        column.spawn((Text::new("Discover bones for DNA and new parts.\nResolve two species by alliance or combat.\nMatch each creature's displayed social cue."), TextFont { font_size: 13.0, ..default() }, TextColor(muted)));
        column.spawn((Text::new(""), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.84,0.9,0.84)), SpeciesHud));
    });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(22),
                top: px(120),
                width: px(286),
                padding: UiRect::all(px(14)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(panel),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.88, 0.63)),
                TargetHud,
            ));
        });
    commands.spawn((Node { position_type: PositionType::Absolute, left: px(22), right: px(22), bottom: px(18), padding: UiRect::all(px(14)),
        border_radius: BorderRadius::all(px(10)), flex_direction: FlexDirection::Column, row_gap: px(7), ..default() }, BackgroundColor(panel))).with_children(|column| {
        column.spawn((Text::new("W/S walk   A/D turn   Shift sprint   E eat / collect / recruit   Space attack   1 sing / 2 dance / 3 charm   H rest at home"), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.88,0.91,0.83))));
        column.spawn((Text::new(if source_inspection { "Tab target   Arrows orbit / tilt   +/- zoom   P pause   R restart   Esc exit" } else { "Tab target   Arrows orbit / tilt   +/- zoom   P pause   R restart   F5 save   F9 load   Esc exit" }), TextFont { font_size: 13.0, ..default() }, TextColor(muted)));
        column.spawn((Text::new(""), TextFont { font_size: 12.0, ..default() }, TextColor(Color::srgb(0.68,0.85,0.79)), NoticeHud));
        column.spawn((Text::new(if source_inspection { "Source art inspection with prototype rules. Save and load disabled; retail assembly, animation and gameplay fidelity remain unverified." } else { "Original functional baseline. Procedural art and rules; retail creature assembly, fidelity and full editors remain pending." }), TextFont { font_size: 11.0, ..default() }, TextColor(Color::srgb(0.57,0.67,0.65))));
    });
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            left: percent(29),
            right: percent(29),
            top: percent(42),
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.72)),
                Banner,
            ));
        });
}

fn keyboard_input(keys: &ButtonInput<KeyCode>, target: Option<u32>) -> CreatureInput {
    let axis =
        |positive, negative| (keys.pressed(positive) as i8 - keys.pressed(negative) as i8) as f32;
    CreatureInput {
        thrust: axis(KeyCode::KeyW, KeyCode::KeyS),
        turn: axis(KeyCode::KeyA, KeyCode::KeyD),
        sprint: keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
        attack: keys.pressed(KeyCode::Space),
        social: if keys.pressed(KeyCode::Digit1) {
            Some(SocialAction::Sing)
        } else if keys.pressed(KeyCode::Digit2) {
            Some(SocialAction::Dance)
        } else if keys.pressed(KeyCode::Digit3) {
            Some(SocialAction::Charm)
        } else {
            None
        },
        interact: keys.pressed(KeyCode::KeyE),
        rest: keys.pressed(KeyCode::KeyH),
        target,
    }
}
fn read_save(path: &Path) -> Result<CreatureWorld, String> {
    use std::io::Read;
    const LIMIT: u64 = 16 * 1024 * 1024;
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("save exceeds 16 MiB".into());
    }
    let world = CreatureSnapshot::from_bytes(&bytes)
        .and_then(CreatureWorld::from_snapshot)
        .map_err(|error| error.to_string())?;
    if let Some(creation) = &world.creation {
        crate::editor::creation_mesh(creation, false)?;
    }
    Ok(world)
}
fn closest(world: &CreatureWorld) -> Option<u32> {
    world
        .creatures
        .iter()
        .filter(|actor| !world.pack.contains(&actor.id))
        .min_by(|a, b| {
            world
                .player
                .position
                .distance(a.position)
                .total_cmp(&world.player.position.distance(b.position))
        })
        .map(|actor| actor.id)
}
fn drive(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, source: Option<Res<SourceInspection>>, ready: Option<Res<OffscreenSimulationReady>>) {
    if ready.as_ref().is_some_and(|ready| !ready.0) { return; }
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if keys.just_pressed(KeyCode::KeyP) {
        game.paused = !game.paused;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.world.restart();
        game.paused = false;
        game.rebuild = true;
        game.target = None;
        game.notice.clear();
    }
    if keys.just_pressed(KeyCode::F5) {
        if source.is_some() { game.notice = "Save disabled: source inspection identity is not stored".into(); return; }
        let result = game
            .world
            .snapshot()
            .to_bytes()
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                crate::cell_game::write_save(&game.save_path, &bytes)
                    .map_err(|error| error.to_string())
            });
        game.notice = match result {
            Ok(()) => "Saved Creature journey".into(),
            Err(error) => format!("Save failed: {error}"),
        };
    }
    if keys.just_pressed(KeyCode::F9) {
        if source.is_some() { game.notice = "Load disabled: source inspection identity is not stored".into(); return; }
        match read_save(&game.save_path) {
            Ok(world) => {
                game.world = world;
                game.paused = false;
                game.rebuild = true;
                game.target = None;
                game.notice = "Loaded Creature journey".into();
            }
            Err(error) => game.notice = format!("Load failed: {error}"),
        }
    }
    if game
        .target
        .is_none_or(|id| !game.world.creatures.iter().any(|actor| actor.id == id))
    {
        game.target = closest(&game.world);
    }
    if keys.just_pressed(KeyCode::Tab) {
        let mut ids: Vec<_> = game
            .world
            .creatures
            .iter()
            .map(|actor| {
                (
                    actor.id,
                    actor.position.distance(game.world.player.position),
                )
            })
            .collect();
        ids.sort_by(|a, b| a.1.total_cmp(&b.1));
        game.target = if ids.is_empty() {
            None
        } else {
            let current = ids
                .iter()
                .position(|(id, _)| Some(*id) == game.target)
                .unwrap_or(ids.len() - 1);
            Some(ids[(current + 1) % ids.len()].0)
        };
    }
    // Switch automatically when a different creature is much closer, keeping Tab
    // selection stable while the selected creature remains in interaction range.
    if let Some(id) = closest(&game.world) {
        let selected_distance = game
            .target
            .and_then(|target| game.world.creatures.iter().find(|actor| actor.id == target))
            .map(|actor| actor.position.distance(game.world.player.position))
            .unwrap_or(f32::MAX);
        let nearest_distance = game
            .world
            .creatures
            .iter()
            .find(|actor| actor.id == id)
            .unwrap()
            .position
            .distance(game.world.player.position);
        if selected_distance > 6.0 && nearest_distance < selected_distance * 0.7 {
            game.target = Some(id);
        }
    }
    let dt = time.delta_secs_f64().min(0.1);
    game.elapsed += dt;
    let input = if game.demo_seconds.is_some() {
        let input = demo_input(&game.world);
        game.target = input.target.or(game.target);
        input
    } else {
        keyboard_input(&keys, game.target)
    };
    if !game.paused && game.capture_frame.is_none() {
        game.world.advance(dt, input);
    }
}

fn demo_input(world: &CreatureWorld) -> CreatureInput {
    let player = &world.player;
    let mut input = CreatureInput {
        interact: true,
        ..default()
    };
    let food = if player.hunger < 0.6 {
        world
            .food
            .iter()
            .filter(|food| world.player_diet.can_eat(food.kind))
            .min_by(|a, b| {
                player
                    .position
                    .distance(a.position)
                    .total_cmp(&player.position.distance(b.position))
            })
            .map(|food| food.position)
    } else {
        None
    };
    let bone = world
        .pickups
        .iter()
        .filter(|pickup| !world.unlocked_parts.contains(&pickup.part))
        .min_by(|a, b| {
            player
                .position
                .distance(a.position)
                .total_cmp(&player.position.distance(b.position))
        })
        .map(|pickup| pickup.position);
    let destination = if let Some(food) = food {
        Some(food)
    } else if world.unlocked_parts.len() < world.config.tribe_parts {
        bone
    } else {
        let target = world
            .creatures
            .iter()
            .filter(|actor| {
                world
                    .species
                    .iter()
                    .any(|species| species.id == actor.species_id && !species.resolved())
            })
            .min_by(|a, b| {
                let score = |actor: &CreatureActor| {
                    player.position.distance(actor.position)
                        + if world.species.iter().any(|species| {
                            species.id == actor.species_id && species.relationship < -20
                        }) {
                            100.0
                        } else {
                            0.0
                        }
                };
                score(a).total_cmp(&score(b))
            });
        if let Some(actor) = target {
            input.target = Some(actor.id);
            if player.position.distance(actor.position) < 3.4 {
                input.social = Some(actor.social_cue);
            }
            Some(actor.position)
        } else if world.dna < world.config.tribe_dna {
            bone.or_else(|| world.pickups.first().map(|pickup| pickup.position))
        } else {
            None
        }
    };
    if let Some(destination) = destination {
        let delta = destination.sub(player.position);
        let distance = delta.length();
        let error = (delta.y.atan2(delta.x) - player.heading + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        input.turn = (error * 2.2).clamp(-1.0, 1.0);
        let stop = if input.target.is_some() { 2.5 } else { 1.1 };
        if error.abs() < 0.7 && distance > stop {
            input.thrust = ((distance - stop) * 0.8).min(1.0);
        }
    }
    input
}

fn spawn_actor(
    commands: &mut Commands,
    art: &Art,
    actor: &CreatureActor,
    is_player: bool,
) -> Entity {
    let skin = if is_player {
        art.player.clone()
    } else {
        art.species[(actor.species_id as usize - 1) % art.species.len()].clone()
    };
    commands
        .spawn((Transform::default(), Visibility::Visible))
        .with_children(|body| {
            body.spawn((
                Mesh3d(art.sphere.clone()),
                MeshMaterial3d(skin.clone()),
                Transform::from_xyz(0.0, 1.05, 0.0).with_scale(Vec3::new(1.0, 0.64, 0.66)),
            ));
            body.spawn((
                Mesh3d(art.sphere.clone()),
                MeshMaterial3d(skin.clone()),
                Transform::from_xyz(0.82, 1.60, 0.0).with_scale(Vec3::new(0.61, 0.54, 0.56)),
            ));
            body.spawn((
                Mesh3d(art.sphere.clone()),
                MeshMaterial3d(art.feet.clone()),
                Transform::from_xyz(1.30, 1.38, 0.0).with_scale(Vec3::new(0.27, 0.15, 0.38)),
            ));
            for side in [-1.0, 1.0] {
                body.spawn((
                    Mesh3d(art.sphere.clone()),
                    MeshMaterial3d(art.white.clone()),
                    Transform::from_xyz(1.04, 1.81, 0.43 * side).with_scale(Vec3::splat(0.20)),
                ));
                body.spawn((
                    Mesh3d(art.sphere.clone()),
                    MeshMaterial3d(art.dark.clone()),
                    Transform::from_xyz(1.16, 1.83, 0.49 * side).with_scale(Vec3::splat(0.095)),
                ));
                body.spawn((
                    Mesh3d(art.cylinder.clone()),
                    MeshMaterial3d(art.white.clone()),
                    Transform::from_xyz(0.72, 2.14, 0.33 * side)
                        .with_rotation(Quat::from_rotation_z(0.32 * side))
                        .with_scale(Vec3::new(0.075, 0.55, 0.075)),
                ));
                for front in [-1.0, 1.0] {
                    let phase = if side * front > 0.0 {
                        0.0
                    } else {
                        std::f32::consts::PI
                    };
                    body.spawn((
                        Transform::from_xyz(front * 0.56, 0.92, side * 0.43),
                        Visibility::Visible,
                        Leg {
                            actor: actor.id,
                            phase,
                        },
                    ))
                    .with_children(|leg| {
                        leg.spawn((
                            Mesh3d(art.cylinder.clone()),
                            MeshMaterial3d(skin.clone()),
                            Transform::from_xyz(0.0, -0.35, 0.0)
                                .with_scale(Vec3::new(0.14, 0.70, 0.14)),
                        ));
                        leg.spawn((
                            Mesh3d(art.sphere.clone()),
                            MeshMaterial3d(art.feet.clone()),
                            Transform::from_xyz(0.12, -0.75, 0.0)
                                .with_scale(Vec3::new(0.29, 0.13, 0.20)),
                        ));
                    });
                }
            }
            body.spawn((
                Mesh3d(art.cylinder.clone()),
                MeshMaterial3d(skin),
                Transform::from_xyz(-1.05, 1.02, 0.0)
                    .with_rotation(Quat::from_rotation_z(1.25))
                    .with_scale(Vec3::new(0.14, 1.0, 0.14)),
            ));
            body.spawn((
                Mesh3d(art.ring.clone()),
                MeshMaterial3d(art.pack.clone()),
                Transform::from_xyz(0.0, 0.05, 0.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(1.2)),
                Visibility::Hidden,
                PackMark(actor.id),
            ));
        })
        .id()
}
fn set_transform(
    commands: &mut Commands,
    transforms: &mut Query<&mut Transform, Without<FollowCamera>>,
    entity: Entity,
    value: Transform,
) {
    if let Ok(mut transform) = transforms.get_mut(entity) {
        *transform = value;
    } else {
        commands.entity(entity).insert(value);
    }
}
fn sync_scene(
    mut commands: Commands,
    art: Res<Art>,
    mut objects: ResMut<Objects>,
    mut game: ResMut<Game>,
    mut transforms: Query<&mut Transform, Without<FollowCamera>>,
    mut pack_marks: Query<(&PackMark, &mut Visibility)>,
    mut authored: ResMut<AuthoredArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    source: Option<Res<SourceInspectionRender>>,
) {
    if authored.creation != game.world.creation {
        if let Some(mesh) = authored.mesh.take() {
            meshes.remove(mesh.id());
        }
        if let Some(creation) = &game.world.creation {
            match crate::editor::creation_mesh(creation, false) {
                Ok(mesh) => authored.mesh = Some(meshes.add(mesh)),
                Err(error) => {
                    game.notice = format!("Creation visual failed: {error}");
                }
            }
            if authored.material.is_none() {
                authored.material = Some(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.85,
                    ..default()
                }));
            }
        }
        authored.creation = game.world.creation.clone();
        game.rebuild = true;
    }
    if game.rebuild {
        for (_, entity) in objects.0.drain() {
            commands.entity(entity).despawn();
        }
        game.rebuild = false;
    }
    let mut live = HashSet::new();
    for (is_player, actor) in std::iter::once((true, &game.world.player))
        .chain(game.world.creatures.iter().map(|actor| (false, actor)))
    {
        let key = (0, actor.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            if is_player {
                if let Some(source) = source.as_ref() {
                    return commands.spawn((Mesh3d(source.mesh.clone()), MeshMaterial3d(source.material.clone()), Transform::default())).id();
                }
                if let (Some(mesh), Some(material)) = (&authored.mesh, &authored.material) {
                    return commands
                        .spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::default(),
                        ))
                        .id();
                }
            }
            spawn_actor(&mut commands, &art, actor, is_player)
        });
        set_transform(
            &mut commands,
            &mut transforms,
            entity,
            Transform::from_translation(location(actor.position, 0.0))
                .with_rotation(Quat::from_rotation_y(-actor.heading))
                .with_scale(Vec3::splat(if is_player && (game.world.creation.is_some() || source.is_some()) {
                    actor.radius()
                } else {
                    1.0
                })),
        );
    }
    for nest in &game.world.nests {
        let key = (1, nest.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((
                    Mesh3d(art.cylinder.clone()),
                    MeshMaterial3d(if nest.species_id == 0 {
                        art.home.clone()
                    } else {
                        art.nest.clone()
                    }),
                    Transform::default(),
                ))
                .id()
        });
        set_transform(
            &mut commands,
            &mut transforms,
            entity,
            Transform::from_translation(location(nest.position, 0.025)).with_scale(Vec3::new(
                nest.radius,
                0.035,
                nest.radius,
            )),
        );
    }
    for food in &game.world.food {
        let key = (2, food.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((
                    Mesh3d(art.sphere.clone()),
                    MeshMaterial3d(if food.kind == FoodKind::Plant {
                        art.plant.clone()
                    } else {
                        art.meat.clone()
                    }),
                    Transform::default(),
                ))
                .id()
        });
        set_transform(
            &mut commands,
            &mut transforms,
            entity,
            Transform::from_translation(location(food.position, 0.38))
                .with_scale(Vec3::new(0.5, 0.36, 0.45)),
        );
    }
    for pickup in &game.world.pickups {
        let key = (3, pickup.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((Transform::default(), Visibility::Visible))
                .with_children(|bone| {
                    bone.spawn((
                        Mesh3d(art.cylinder.clone()),
                        MeshMaterial3d(art.bone.clone()),
                        Transform::default()
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::new(0.12, 1.0, 0.12)),
                    ));
                    for x in [-0.5, 0.5] {
                        bone.spawn((
                            Mesh3d(art.sphere.clone()),
                            MeshMaterial3d(art.bone.clone()),
                            Transform::from_xyz(x, 0.0, 0.0)
                                .with_scale(Vec3::new(0.21, 0.26, 0.22)),
                        ));
                    }
                })
                .id()
        });
        set_transform(
            &mut commands,
            &mut transforms,
            entity,
            Transform::from_translation(location(
                pickup.position,
                0.7 + (game.elapsed as f32 * 2.0).sin() * 0.1,
            ))
            .with_rotation(Quat::from_rotation_y(
                game.elapsed as f32 * 0.55 + pickup.id as f32,
            )),
        );
    }
    if let Some(actor) = game
        .target
        .and_then(|id| game.world.creatures.iter().find(|actor| actor.id == id))
    {
        let key = (4, 0);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((
                    Mesh3d(art.ring.clone()),
                    MeshMaterial3d(art.focus.clone()),
                    Transform::default(),
                    TargetMark,
                ))
                .id()
        });
        set_transform(
            &mut commands,
            &mut transforms,
            entity,
            Transform::from_translation(location(actor.position, 0.075))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(1.65)),
        );
    }
    for (mark, mut visible) in &mut pack_marks {
        *visible = if game.world.pack.contains(&mark.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    objects.0.retain(|key, entity| {
        if live.contains(key) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
}
fn animate_legs(game: Res<Game>, mut legs: Query<(&Leg, &mut Transform)>) {
    for (leg, mut transform) in &mut legs {
        let actor = if leg.actor == game.world.player.id {
            Some(&game.world.player)
        } else {
            game.world
                .creatures
                .iter()
                .find(|actor| actor.id == leg.actor)
        };
        if let Some(actor) = actor {
            let swing = (game.world.tick as f32 * 0.20 + leg.phase).sin()
                * (actor.velocity.length() / 6.0).min(1.0)
                * 0.65;
            transform.rotation = Quat::from_rotation_z(swing);
        }
    }
}
fn follow_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    mut orbit: ResMut<Orbit>,
    mut cameras: Query<&mut Transform, With<FollowCamera>>,
) {
    let dt = time.delta_secs().min(0.1);
    let axis =
        |positive, negative| (keys.pressed(positive) as i8 - keys.pressed(negative) as i8) as f32;
    orbit.yaw += axis(KeyCode::ArrowLeft, KeyCode::ArrowRight) * dt * 1.6;
    orbit.pitch = (orbit.pitch + axis(KeyCode::ArrowUp, KeyCode::ArrowDown) * dt).clamp(0.2, 1.25);
    orbit.distance =
        (orbit.distance + axis(KeyCode::Minus, KeyCode::Equal) * dt * 12.0).clamp(8.0, 45.0);
    let target = location(game.world.player.position, 1.1);
    let offset = Vec3::new(
        orbit.yaw.cos() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        orbit.yaw.sin() * orbit.pitch.cos(),
    ) * orbit.distance;
    for mut camera in &mut cameras {
        *camera = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
    }
}

fn update_hud(
    game: Res<Game>,
    mut texts: ParamSet<(
        Query<&mut Text, With<Hud>>,
        Query<&mut Text, With<TargetHud>>,
        Query<&mut Text, With<SpeciesHud>>,
        Query<&mut Text, With<NoticeHud>>,
        Query<&mut Text, With<Banner>>,
    )>,
    mut meters: Query<(&Meter, &mut Node)>,
) {
    let world = &game.world;
    for mut text in &mut texts.p0() {
        text.0 = format!(
            "{:?}   DNA {} / {}\nParts {} / {}   Resolved {} / {}   Pack {} / {}",
            world.player_diet,
            world.dna,
            world.config.tribe_dna,
            world.unlocked_parts.len(),
            world.config.tribe_parts,
            world.resolved_species(),
            world.config.tribe_resolved_species,
            world.pack.len(),
            world.config.pack_capacity
        );
    }
    for mut text in &mut texts.p1() {
        text.0 = if let Some(actor) = game
            .target
            .and_then(|id| world.creatures.iter().find(|actor| actor.id == id))
        {
            let species = world
                .species
                .iter()
                .find(|species| species.id == actor.species_id)
                .unwrap();
            format!(
                "TARGET: {}   {:.1}m\nHealth {:.0}/{:.0}   Relation {:+}\nCue: {}\n{}",
                species.name,
                actor.position.distance(world.player.position),
                actor.health,
                actor.max_health,
                species.relationship,
                match actor.social_cue {
                    SocialAction::Sing => "1 SING",
                    SocialAction::Dance => "2 DANCE",
                    SocialAction::Charm => "3 CHARM",
                },
                if world.pack.contains(&actor.id) {
                    "Pack follower"
                } else if species.allied() {
                    "ALLIED | E to recruit within 3.5m"
                } else {
                    "Social range 4.5m | Attack range 3.1m"
                }
            )
        } else {
            "No living target\nExplore the nests across the valley".into()
        };
    }
    for mut text in &mut texts.p2() {
        text.0 = world
            .species
            .iter()
            .map(|species| {
                format!(
                    "{}  {:+}  {}",
                    species.name,
                    species.relationship,
                    if species.extinct {
                        "extinct"
                    } else if species.allied() {
                        "allied"
                    } else {
                        "unresolved"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let parts = if world.unlocked_parts.is_empty() {
            "none".into()
        } else {
            world
                .unlocked_parts
                .iter()
                .map(|part| format!("{part:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        text.0.push_str(&format!(
            "\n\nParts: {parts}\nSeed {} | tick {}",
            world.seed(),
            world.tick
        ));
        if let Some(creation) = &world.creation {
            text.0.push_str(&format!(
                "\nAuthored: {}",
                creation.name.chars().take(22).collect::<String>()
            ));
        }
    }
    for mut text in &mut texts.p3() {
        text.0 = if game.notice.is_empty() {
            format!(
                "Home nest: {:.0}m | Bones left: {} | {}",
                world.player.position.length(),
                world.pickups.len(),
                if game.demo_seconds.is_some() {
                    "Demo drives normal game actions"
                } else {
                    "E interacts with nearby objects"
                }
            )
        } else {
            game.notice.chars().take(140).collect()
        };
    }
    for mut text in &mut texts.p4() {
        text.0 = if game.paused {
            "PAUSED\nP to continue".into()
        } else {
            match world.state {
                CreatureState::Playing => String::new(),
                CreatureState::TribeReady => {
                    "TRIBE ELIGIBLE\nAll three progression gates complete\nR to explore again"
                        .into()
                }
                CreatureState::Dead => "YOUR JOURNEY ENDS\nR to restart at the nest".into(),
            }
        };
    }
    for (meter, mut node) in &mut meters {
        node.width = percent(
            100.0
                * match meter.0 {
                    0 => world.player.health / world.player.max_health,
                    1 => world.player.stamina,
                    _ => world.player.hunger,
                }
                .clamp(0.0, 1.0),
        );
    }
}
fn capture(mut commands: Commands, mut game: ResMut<Game>, target: Option<Res<crate::offscreen::OffscreenTarget>>, mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>, mut simulation_ready: Option<ResMut<OffscreenSimulationReady>>, layouts: Query<(&Text, Option<&bevy::text::TextLayoutInfo>)>, time: Res<Time>) {
    if let Some(readiness) = readiness.as_mut() {
        if let Some(error) = readiness.failure() { eprintln!("Creature offscreen render failed: {error}"); std::process::exit(1); }
        let text_ready = layouts.iter().all(|(text, layout)| text.0.is_empty() || layout.is_some_and(|layout| readiness.require_text(layout)));
        if !text_ready || !readiness.ready() {
            if time.elapsed_secs() > 120.0 { eprintln!("Creature offscreen renderer did not become ready"); std::process::exit(1); }
            return;
        }
        if let Some(ready) = simulation_ready.as_mut() { ready.0 = true; }
    }
    game.frames += 1;
    if game.capture_frame.is_none()
        && game.elapsed >= game.demo_seconds.unwrap_or(1.5)
        && (game.shot.is_some() || game.demo_seconds.is_some())
    {
        if let Some(path) = &game.shot {
            commands
                .spawn(target.as_ref().map_or_else(Screenshot::primary_window, |target| target.screenshot()))
                .observe(save_to_disk(path.clone()));
        }
        game.capture_frame = Some(game.frames);
        println!("CREATURE_RESULT state={:?} tick={} dna={} parts={} resolved={} pack={} health={:.3} x={:.3} y={:.3}",game.world.state,game.world.tick,game.world.dna,
            game.world.unlocked_parts.len(),game.world.resolved_species(),game.world.pack.len(),game.world.player.health,game.world.player.position.x,game.world.player.position.y);
    }
    if let Some(frame) = game.capture_frame {
        if game.frames > frame + 10 && game.shot.as_ref().is_none_or(|path| path.exists()) {
            std::process::exit(0);
        }
        if game.frames > frame + 600 {
            eprintln!("Creature screenshot capture failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_inspection_basis_maps_positions_and_normals() {
        assert_eq!(source_to_bevy_basis([1.0, 0.0, 0.0]), [0.0, 0.0, 1.0]);
        assert_eq!(source_to_bevy_basis([0.0, 1.0, 0.0]), [1.0, 0.0, 0.0]);
        assert_eq!(source_to_bevy_basis([0.0, 0.0, 1.0]), [0.0, 1.0, 0.0]);
        let normal = source_to_bevy_basis([0.0, 0.6, 0.8]);
        assert_eq!(normal, [0.6, 0.8, 0.0]);
        assert!((normal.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-6);
    }
    #[test]
    fn source_inspection_cli_requires_pair_and_excludes_saved_creation() {
        assert!(run(&["--source-creature-id".into(), "1234".into()]).unwrap_err().contains("provided together"));
        assert!(run(&["--source-creature-install".into(), "unused".into()]).unwrap_err().contains("provided together"));
        assert!(run(&["--source-creature-install".into(), "unused".into(), "--source-creature-id".into(), "1234".into(), "--creation".into(), "unused".into()]).unwrap_err().contains("cannot be combined"));
        assert!(run(&["--offscreen".into()]).unwrap_err().contains("requires --shot"));
    }
    #[test]
    fn source_inspection_rejects_unbound_saves_and_survives_restart() {
        use crate::assets::{AuthoredCreatureAsset, AuthoredCreatureModel, TextureData};
        use dbpf::native_index::NativeResourceKey;
        use spore_core::{mesh::MeshData, shader_binding::TextureBinding};
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let folder = std::env::temp_dir().join(format!("rustspore-source-creature-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("creature.rsp");
        let alternate = CreatureWorld::new(99, Diet::Herbivore).snapshot().to_bytes().unwrap();
        std::fs::write(&path, &alternate).unwrap();
        let identity = NativeResourceKey { instance_id: 0x1234, type_id: 0x5678, group_id: 0 };
        let source = SourceInspection {
            asset: AuthoredCreatureAsset {
                reference: AuthoredCreatureModel { creature: identity, prop: identity, model: identity, lod: 0 },
                mesh: MeshData::default(), texture_key: identity,
                texture_binding: TextureBinding { sampler: 0, saved_pointer_word: 0, settings: [0; 8], raster_instance: 0 },
                texture: TextureData { width: 1, height: 1, rgba: vec![1, 2, 3, 4] },
            }, sampler: ImageSamplerDescriptor::default(),
        };
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Game::new(77, Diet::Carnivore, path.clone()))
            .insert_resource(source)
            .add_systems(Update, drive);
        step(&mut app, 0.0, &[KeyCode::F5]);
        assert_eq!(std::fs::read(&path).unwrap(), alternate);
        assert!(app.world().resource::<Game>().notice.starts_with("Save disabled"));
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, 0.0, &[KeyCode::F9]);
        assert_eq!(app.world().resource::<Game>().world, before);
        assert!(app.world().resource::<Game>().notice.starts_with("Load disabled"));
        step(&mut app, 0.0, &[KeyCode::KeyR]);
        assert_eq!(app.world().resource::<Game>().world, CreatureWorld::new(77, Diet::Carnivore));
        let retained = app.world().resource::<SourceInspection>();
        assert_eq!(retained.asset.reference.creature, identity);
        assert_eq!(retained.asset.texture.rgba, [1, 2, 3, 4]);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
    fn step(app: &mut App, seconds: f64, pressed: &[KeyCode]) {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.reset_all();
        for key in pressed {
            keys.press(*key);
        }
        drop(keys);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(seconds));
        app.update();
    }
    #[test]
    fn keyboard_maps_movement_and_social_actions_without_diet_restarts() {
        let mut keys = ButtonInput::default();
        for key in [
            KeyCode::KeyW,
            KeyCode::KeyD,
            KeyCode::ShiftLeft,
            KeyCode::KeyE,
            KeyCode::Space,
            KeyCode::Digit2,
            KeyCode::KeyH,
        ] {
            keys.press(key);
        }
        let input = keyboard_input(&keys, Some(7));
        assert_eq!(input.thrust, 1.0);
        assert_eq!(input.turn, -1.0);
        assert_eq!(input.target, Some(7));
        assert!(input.sprint && input.interact && input.attack && input.rest);
        assert_eq!(input.social, Some(SocialAction::Dance));
    }
    #[test]
    fn actual_app_drive_moves_pauses_saves_loads_restarts_and_preserves_state_on_bad_load() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-creature-controls-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("creature.rsp");
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Game::new(77, Diet::Carnivore, path.clone()))
            .add_systems(Update, drive);
        step(&mut app, 0.1, &[KeyCode::KeyW]);
        let moved = app.world().resource::<Game>().world.clone();
        assert!(moved.player.position.x > 0.0);
        step(&mut app, 0.1, &[KeyCode::KeyP, KeyCode::KeyW]);
        assert_eq!(app.world().resource::<Game>().world, moved);
        step(&mut app, 0.0, &[KeyCode::KeyP]);
        step(&mut app, 0.0, &[KeyCode::F5]);
        let saved = std::fs::read(&path).unwrap();
        assert_eq!(read_save(&path).unwrap(), moved);
        step(&mut app, 0.1, &[KeyCode::KeyW]);
        assert_ne!(app.world().resource::<Game>().world, moved);
        step(&mut app, 0.0, &[KeyCode::F9]);
        assert_eq!(
            app.world()
                .resource::<Game>()
                .world
                .snapshot()
                .to_bytes()
                .unwrap(),
            saved
        );
        step(&mut app, 0.0, &[KeyCode::Digit1]);
        assert_eq!(
            app.world().resource::<Game>().world.player_diet,
            Diet::Carnivore
        );
        step(&mut app, 0.0, &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<Game>().world,
            CreatureWorld::new(77, Diet::Carnivore)
        );
        std::fs::write(&path, b"broken").unwrap();
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, 0.0, &[KeyCode::F9]);
        assert_eq!(app.world().resource::<Game>().world, before);
        assert!(app
            .world()
            .resource::<Game>()
            .notice
            .starts_with("Load failed:"));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
    #[test]
    fn demo_completes_real_bone_social_pack_and_tribe_gates_without_state_shortcuts() {
        let mut world = CreatureWorld::new(14, Diet::Herbivore);
        for _ in 0..3600 {
            let input = demo_input(&world);
            world.advance(0.1, input);
            if world.state != CreatureState::Playing {
                break;
            }
        }
        assert_eq!(
            world.state,
            CreatureState::TribeReady,
            "demo stalled at {:?}, DNA{}, parts{}, relations {:?}",
            world.player.position,
            world.dna,
            world.unlocked_parts.len(),
            world
                .species
                .iter()
                .map(|species| species.relationship)
                .collect::<Vec<_>>()
        );
        assert!(
            world.dna >= 100 && world.unlocked_parts.len() >= 2 && world.resolved_species() >= 2
        );
        assert!(!world.pack.is_empty());
        world.snapshot().to_bytes().unwrap();
    }

    fn authored_creation() -> Creation {
        use spore_core::editor::{AuthoredPart, PartArchetype};
        let mut creation = Creation::new(CreationKind::Creature, "Painted walker", 150).unwrap();
        for (id, kind) in [
            (2, PartArchetype::CarnivoreMouth),
            (3, PartArchetype::Leg),
            (4, PartArchetype::Eye),
        ] {
            creation
                .parts
                .push(AuthoredPart::new(id, format!("Part {id}"), kind, Some(1)));
        }
        creation.parts[2].transform.translation = [0., -1., 0.6];
        creation.palette[0] = [0.3, 0.2, 0.8];
        creation
    }

    #[test]
    fn authored_app_save_load_restart_preserves_geometry_palette_and_gameplay_stats() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-creature-authored-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("creature.rsp");
        let creation = authored_creation();
        let mut game = Game::new(77, Diet::Carnivore, path.clone());
        game.world.apply_creation(creation.clone()).unwrap();
        let stats = game.world.loadout.clone();
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(game)
            .add_systems(Update, drive);
        step(&mut app, 0., &[KeyCode::F5]);
        app.world_mut().resource_mut::<Game>().world = CreatureWorld::new(999, Diet::Herbivore);
        step(&mut app, 0., &[KeyCode::F9]);
        assert_eq!(
            app.world().resource::<Game>().world.creation,
            Some(creation.clone())
        );
        assert_eq!(app.world().resource::<Game>().world.loadout, stats);
        step(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<Game>().world.creation,
            Some(creation)
        );
        assert_eq!(app.world().resource::<Game>().world.loadout, stats);
        assert_eq!(
            app.world().resource::<Game>().world.player_diet,
            Diet::Carnivore
        );
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }

    #[test]
    fn creation_cli_checks_kind_diet_and_explicit_unsupported_gliding_before_gpu() {
        use spore_core::editor::{AuthoredPart, EditorSnapshot, PartArchetype};
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rustspore-creature-cli-{}-{nonce}.rsc",
            std::process::id()
        ));
        let args = vec!["--creation".into(), path.to_string_lossy().into_owned()];
        std::fs::write(&path, b"invalid").unwrap();
        assert!(run(&args).is_err());
        std::fs::write(
            &path,
            EditorSnapshot {
                version: 1,
                revision: 1,
                creation: Creation::new(CreationKind::Cell, "Wrong kind", 100).unwrap(),
            }
            .to_bytes()
            .unwrap(),
        )
        .unwrap();
        assert!(run(&args).unwrap_err().contains("Creature creation"));
        let creation = authored_creation();
        std::fs::write(
            &path,
            EditorSnapshot {
                version: 1,
                revision: 1,
                creation: creation.clone(),
            }
            .to_bytes()
            .unwrap(),
        )
        .unwrap();
        let mut conflict = args.clone();
        conflict.extend(["--diet".into(), "herbivore".into()]);
        assert!(run(&conflict).unwrap_err().contains("contradicts"));
        let world = configured_world(14, Diet::Herbivore, false, Some(&path)).unwrap();
        assert_eq!(world.player_diet, Diet::Carnivore);
        assert!(world
            .food
            .iter()
            .take(6)
            .all(|food| food.kind == FoodKind::Meat));
        let mut glider = creation;
        glider
            .parts
            .push(AuthoredPart::new(5, "Wing A", PartArchetype::Wing, Some(1)));
        glider
            .parts
            .push(AuthoredPart::new(6, "Wing B", PartArchetype::Wing, Some(1)));
        std::fs::write(
            &path,
            EditorSnapshot {
                version: 1,
                revision: 1,
                creation: glider,
            }
            .to_bytes()
            .unwrap(),
        )
        .unwrap();
        assert!(run(&args).unwrap_err().contains("UnsupportedCapability"));
        std::fs::remove_file(path).unwrap();
    }
}
