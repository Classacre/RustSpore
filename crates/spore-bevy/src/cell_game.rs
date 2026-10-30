//! Playable Cell-stage vertical slice with deterministic simulation and GPU visuals.
//! Retail content, editors and cross-stage progression are separate milestone gates.

#[path = "cell_kind0_inspection.rs"]
mod cell_kind0_inspection;

use bevy::{
    asset::RenderAssetUsages,
    camera::{visibility::RenderLayers, ScalingMode},
    image::{ImageSampler, ImageSamplerDescriptor, ImageAddressMode, ImageFilterMode},
    mesh::Indices,
    prelude::*,
    render::{
        render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
        renderer::RenderAdapterInfo,
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use spore_core::cell::{
    CellBehavior, CellInput, CellSnapshot, CellState, CellWorld, Diet, FoodKind,
};
use spore_core::editor::{Creation, CreationKind, EditorSnapshot, MAX_SAVE_BYTES};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Resource)]
struct Game {
    world: CellWorld,
    seed: u64,
    diet: Diet,
    paused: bool,
    elapsed: f64,
    notice: String,
    rebuild: bool,
    save_path: PathBuf,
    shot: Option<PathBuf>,
    demo_seconds: Option<f64>,
    capture_frame: Option<u64>,
    frames: u64,
}

#[derive(Resource)]
struct Art {
    circle: Handle<Mesh>,
    ring: Handle<Mesh>,
    leaf: Handle<Mesh>,
    line: Handle<Mesh>,
    player: Handle<ColorMaterial>,
    predator: Handle<ColorMaterial>,
    prey: Handle<ColorMaterial>,
    plant: Handle<ColorMaterial>,
    meat: Handle<ColorMaterial>,
    membrane: Handle<ColorMaterial>,
    nucleus: Handle<ColorMaterial>,
    white: Handle<ColorMaterial>,
    dark: Handle<ColorMaterial>,
    rock: Handle<ColorMaterial>,
}

#[derive(Resource, Default)]
struct SceneObjects(HashMap<(u8, u32), Entity>);
#[derive(Resource, Default)]
struct AuthoredArt {
    creation: Option<Creation>,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<ColorMaterial>>,
}
/// Set once by capture after all required render and text assets are ready.
#[derive(Resource, Default)]
struct OffscreenSimulationReady(bool);

#[derive(Resource)]
struct SourcePlayer {
    mesh: spore_core::mesh::MeshData,
    skin: crate::assets::TextureData,
    mips: Option<crate::assets::TextureMipChain>,
    sampler: ImageSamplerDescriptor,
    center: Vec3,
    scale: f32,
    rotation: Quat,
    kind: SourceKind,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
enum SourceKind { Cell, Creature }

#[derive(Resource)]
struct SourcePlayerRender {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    center: Vec3,
    scale: f32,
    rotation: Quat,
}

/// Inspection-only basis: source (x,y,z) to screen-facing (y,z,x).
fn creature_inspection_rotation() -> Quat {
    Quat::from_xyzw(-0.5, -0.5, -0.5, 0.5)
}

fn inspection_mip_lod_max(filter: i32, levels: usize) -> f32 {
    if filter == 0 { 0.0 } else { levels.saturating_sub(1) as f32 }
}

fn creature_mip_chain_required(texture_type: u32, filter: i32) -> Result<bool, String> {
    match (texture_type, filter) {
        (crate::assets::RASTER, _) => Ok(true),
        (crate::assets::RW4, 0) => Ok(false),
        (crate::assets::RW4, _) => Err("source Creature RW4 mip chain unsupported when mip filtering is enabled".into()),
        _ => Err("source Creature texture type unsupported".into()),
    }
}

fn inspection_image(texture: &crate::assets::TextureData, mips: Option<&crate::assets::TextureMipChain>, sampler: &ImageSamplerDescriptor) -> Image {
    let mut image = Image::new(Extent3d { width: texture.width, height: texture.height, depth_or_array_layers: 1 }, TextureDimension::D2, texture.rgba.clone(), TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    if let Some(mips) = mips {
        image.texture_descriptor.mip_level_count = mips.levels.len() as u32;
        image.data = Some(mips.levels.iter().flat_map(|level| level.rgba.iter().copied()).collect());
    }
    image.sampler = ImageSampler::Descriptor(sampler.clone());
    image
}

fn load_source_creature(install: &Path, instance: u64) -> Result<(spore_core::mesh::MeshData, crate::assets::TextureData, Option<crate::assets::TextureMipChain>, ImageSamplerDescriptor), String> {
    let base = install.join("Data/Spore_Content.package");
    let paths = crate::assets::PackageSet::discover(install).paths().to_vec();
    let source = crate::assets::AssetSource::open(&base)?
        .with_set(crate::assets::PackageSet::from_paths(paths.clone()));
    let asset = source.authored_creature_asset(instance, 0).map_err(|e| e.to_string())?;
    let tex = &asset.texture;
    if tex.width == 0 || tex.height == 0 || tex.width > 4096 || tex.height > 4096
        || (tex.width as usize).checked_mul(tex.height as usize).and_then(|n| n.checked_mul(4)) != Some(tex.rgba.len()) {
        return Err("source Creature texture dimensions or decoded extent unsupported".into());
    }
    let binding = asset.texture_binding;
    if binding.availability() != Some(1) { return Err("source sampler unavailable".into()); }
    let address = match binding.address_mode() {
        Some(1) => ImageAddressMode::Repeat, Some(2) => ImageAddressMode::MirrorRepeat,
        Some(3) => ImageAddressMode::ClampToEdge, _ => return Err("source sampler addressing unsupported".into()),
    };
    let filter = match binding.min_mag_filter() {
        Some(1) => ImageFilterMode::Nearest, Some(2) => ImageFilterMode::Linear,
        _ => return Err("source sampler filtering unsupported".into()),
    };
    let mip = match binding.mip_filter() {
        Some(0 | 1) => ImageFilterMode::Nearest, Some(2) => ImageFilterMode::Linear,
        _ => return Err("source sampler mip filtering unsupported".into()),
    };
    let mip_filter = binding.mip_filter().unwrap();
    let mips = if creature_mip_chain_required(asset.texture_key.type_id, mip_filter)? {
        let chain = source.authored_creature_raster_mips(asset.texture_key)?;
        let first = chain.levels.first().ok_or("source Creature mip chain empty")?;
        if first.width != tex.width || first.height != tex.height || first.rgba != tex.rgba {
            return Err("source Creature base mip differs from texture".into());
        }
        Some(chain)
    } else { None };
    let sampler = ImageSamplerDescriptor { address_mode_u: address, address_mode_v: address, address_mode_w: address,
        min_filter: filter, mag_filter: filter, mipmap_filter: mip, lod_min_clamp: 0.0,
        lod_max_clamp: inspection_mip_lod_max(mip_filter, mips.as_ref().map_or(1, |chain| chain.levels.len())), ..default() };
    let lookup = crate::assets::PackageSet::from_paths(paths);
    for (role, key) in [("creature", asset.reference.creature), ("prop", asset.reference.prop),
        ("model", asset.reference.model), ("texture", asset.texture_key)] {
        let path = if source.resource_keys().any(|k| k == key) { base.clone() } else {
            lookup.locate(key.type_id, key.scoped_instance()).ok_or("source resource provenance unavailable")?
        };
        println!("CELL_SOURCE_CREATURE_RESOURCE role={role} package={} key={key}", path.display());
    }
    println!("CELL_SOURCE_CREATURE_INSPECTION instance={instance:016X} assignment=controlled_prototype_player renderer=pbr static=true original_animation=false runtime_selection=unverified overlay=inspection-order basis=source_y_z_x_inference mip_chain={} levels={}", if mips.is_some() { "declared" } else { "base_only_filter_disabled" }, mips.as_ref().map_or(1, |chain| chain.levels.len()));
    Ok((asset.mesh, asset.texture, mips, sampler))
}

#[derive(Component)]
struct Hud;
#[derive(Component)]
struct Banner;
#[derive(Component)]
struct Meter(u8);

pub fn run(args: &[String]) -> Result<(), String> {
    let mut seed = 14;
    let mut diet = Diet::Herbivore;
    let mut explicit_diet = false;
    let mut creation_path = None;
    let mut shot = None;
    let mut demo_seconds = None;
    let mut source_install: Option<PathBuf> = None;
    let mut source_id: Option<u32> = None;
    let mut creature_install: Option<PathBuf> = None;
    let mut creature_instance: Option<u64> = None;
    let mut offscreen = false;
    let mut save_path = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RustSpore")
        .join("saves")
        .join("cell.rsp");
    let mut i = 0;
    while i < args.len() {
        let value = || {
            args.get(i + 1)
                .ok_or_else(|| format!("{} requires a value", args[i]))
        };
        if args[i] == "--offscreen" {
            offscreen = true;
            i += 1;
            continue;
        }
        match args[i].as_str() {
            "--source-cell-install" => source_install = Some(PathBuf::from(value()?)),
            "--source-creature-install" => creature_install = Some(PathBuf::from(value()?)),
            "--source-creature-instance" => {
                let text = value()?.trim();
                let text = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text);
                if text.len() != 16 { return Err("source Creature instance must be exactly 16 hexadecimal digits".into()); }
                creature_instance = Some(u64::from_str_radix(text, 16).map_err(|_| "source Creature instance must be exactly 16 hexadecimal digits".to_string())?);
            }
            "--source-cell-id" => {
                let text = value()?.trim();
                let text = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")).unwrap_or(text);
                source_id = Some(u32::from_str_radix(text, 16).map_err(|_| "source Cell ID must be hexadecimal u32".to_string())?);
            }
            "--seed" => seed = value()?.parse().map_err(|_| "invalid seed".to_string())?,
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
            "--shot" => shot = Some(PathBuf::from(value()?)),
            "--demo" => {
                let seconds: f64 = value()?
                    .parse()
                    .map_err(|_| "invalid demo duration".to_string())?;
                if !seconds.is_finite() || seconds <= 0.0 || seconds > 600.0 {
                    return Err("demo duration must be in (0, 600] seconds".into());
                }
                demo_seconds = Some(seconds);
            }
            "--save-file" => save_path = PathBuf::from(value()?),
            arg => return Err(format!("unknown Cell option {arg}")),
        }
        i += 2;
    }
    if source_install.is_some() != source_id.is_some() {
        return Err("--source-cell-install and --source-cell-id must be provided together".into());
    }
    if creature_install.is_some() != creature_instance.is_some() {
        return Err("--source-creature-install and --source-creature-instance must be provided together".into());
    }
    if source_install.is_some() && creature_install.is_some() {
        return Err("source Cell and source Creature inspection art cannot be combined".into());
    }
    if (source_install.is_some() || creature_install.is_some()) && creation_path.is_some() {
        return Err("source inspection art cannot be combined with --creation".into());
    }
    if offscreen && shot.is_none() {
        return Err("--offscreen requires --shot <new PNG>".into());
    }
    let selected_art = if let (Some(install), Some(id)) = (&source_install, source_id) {
        let (mesh, skin, sampler) = if id == cell_kind0_inspection::CELL_ID {
            let asset = cell_kind0_inspection::load(install)?;
            (asset.mesh, asset.texture, asset.sampler)
        } else {
            let library = crate::source_cell::SourceCellLibrary::open_install(install)?;
            let asset = library.load(id)?;
            let sampler = crate::source_cell::source_skin_sampler(&asset)?;
            println!("CELL_SOURCE_INSPECTION creation={:016X} pending_effects={} renderer=pbr static=true original_animation=false overlay=inspection-order", asset.definition.creation_instance, asset.pending_structure_entries.len());
            for (role, identity) in [("cell", &asset.definition.cell), ("structure", &asset.definition.structure),
                ("model_prop", &asset.model_prop), ("model", &asset.model), ("skin", &asset.skin_source)] {
                println!("CELL_SOURCE_RESOURCE role={role} package={:?} key={}", identity.package, identity.key);
            }
            (asset.mesh, asset.skin, sampler)
        };
        Some((mesh, skin, None, sampler, Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), SourceKind::Cell))
    } else if let (Some(install), Some(instance)) = (&creature_install, creature_instance) {
        let (mesh, skin, mips, sampler) = load_source_creature(install, instance)?;
        Some((mesh, skin, mips, sampler, creature_inspection_rotation(), SourceKind::Creature))
    } else { None };
    let source_player = if let Some((mesh, skin, mips, sampler, rotation, kind)) = selected_art {
        let (min, max) = mesh.bounds().ok_or("source inspection mesh is empty")?;
        let center = Vec3::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5, (min.z + max.z) * 0.5);
        let extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
        if !center.is_finite() || !extent.is_finite() || extent <= 1e-6 {
            return Err("source inspection mesh has invalid or degenerate bounds".into());
        }
        Some(SourcePlayer { mesh, skin, mips, sampler, center, scale: 2.0 / extent, rotation, kind })
    } else { None };
    let world = configured_world(seed, diet, explicit_diet, creation_path.as_deref())?;
    diet = world.player.diet;
    if shot.as_ref().is_some_and(|path| path.exists()) {
        return Err("screenshot already exists; choose a new output path".into());
    }
    if let Some(parent) = shot
        .as_ref()
        .and_then(|path| path.parent())
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut app = App::new();
    if offscreen {
        crate::offscreen::configure(&mut app, 1280, 800);
        app.init_resource::<OffscreenSimulationReady>();
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore | Cell".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }));
    }
    if let Some(source) = source_player { app.insert_resource(source.kind); app.insert_resource(source); }
    app.insert_resource(ClearColor(Color::srgb(0.012, 0.032, 0.046)))
        .insert_resource(Game {
            world,
            seed,
            diet,
            paused: false,
            elapsed: 0.0,
            notice: String::new(),
            rebuild: true,
            save_path,
            shot,
            demo_seconds,
            capture_frame: None,
            frames: 0,
        })
        .init_resource::<SceneObjects>()
        .init_resource::<AuthoredArt>()
        .add_systems(Startup, setup)
        .add_systems(Update, (drive, sync_scene, update_hud, capture).chain())
        .run();
    Ok(())
}

pub(crate) fn read_creation(path: &std::path::Path) -> Result<Creation, String> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_SAVE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_SAVE_BYTES {
        return Err("creation exceeds the bounded save limit".into());
    }
    EditorSnapshot::from_bytes(&bytes)
        .map(|snapshot| snapshot.creation)
        .map_err(|error| error.to_string())
}
fn configured_world(
    seed: u64,
    diet: Diet,
    explicit_diet: bool,
    path: Option<&std::path::Path>,
) -> Result<CellWorld, String> {
    let Some(path) = path else {
        return Ok(CellWorld::new(seed, diet));
    };
    let creation = read_creation(path)?;
    if creation.kind != CreationKind::Cell {
        return Err("Cell gameplay requires a Cell creation".into());
    }
    let authored_diet = match creation
        .gameplay_loadout()
        .map_err(|error| error.to_string())?
    {
        spore_core::editor::GameplayLoadout::Cell(loadout) => loadout.diet,
        _ => return Err("Cell gameplay requires a Cell creation".into()),
    };
    let mut world = CellWorld::new(seed, authored_diet);
    world
        .apply_creation(creation)
        .map_err(|error| error.to_string())?;
    if explicit_diet && world.player.diet != diet {
        return Err(
            "--diet contradicts the authored mouth; edit the creation or omit --diet".into(),
        );
    }
    crate::editor::creation_mesh(world.creation.as_ref().unwrap(), true)?;
    Ok(world)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    adapter: Res<RenderAdapterInfo>,
    source: Option<Res<SourcePlayer>>,
    mut source_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>,
) {
    println!(
        "CELL_GPU name={:?} backend={:?} type={:?}",
        adapter.name, adapter.backend, adapter.device_type
    );
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 76.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    let creature_mode = source.as_ref().is_some_and(|source| source.kind == SourceKind::Creature);
    if let Some(source) = source {
        let data = &source.mesh;
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());
        mesh.insert_indices(Indices::U32(data.indices.clone()));
        let texture = &source.skin;
        let image = inspection_image(texture, source.mips.as_ref(), &source.sampler);
        let image = images.add(image);
        if let Some(readiness) = readiness.as_mut() { readiness.require_image(&image); }
        let material = source_materials.add(StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(image), perceptual_roughness: 0.7, metallic: 0.0, cull_mode: None, double_sided: true, ..default() });
        commands.insert_resource(SourcePlayerRender { mesh: meshes.add(mesh), material, center: source.center, scale: source.scale, rotation: source.rotation });
        commands.spawn((Camera3d::default(), Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
            Projection::Orthographic(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: 76.0 }, ..OrthographicProjection::default_3d() }),
            Transform::from_xyz(0., 0., 100.), RenderLayers::layer(1), IsDefaultUiCamera,
            AmbientLight { color: Color::WHITE, brightness: 300., ..default() }));
        commands.spawn((DirectionalLight { illuminance: 9000., ..default() }, Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -0.35, 0.)), RenderLayers::layer(1)));
        let label = if source.kind == SourceKind::Creature { "SOURCE CREATURE: static PBR inspection art; experimental gameplay" } else { "SOURCE CELL: static PBR inspection art; experimental gameplay" };
        commands.spawn((Text::new(label), TextFont { font_size: 14., ..default() }, TextColor(Color::WHITE), Node { position_type: PositionType::Absolute, right: px(20), top: px(16), ..default() }));
    }
    let circle = meshes.add(Circle::new(1.0));
    let line = meshes.add(Rectangle::new(1.0, 1.0));
    let ring = meshes.add(Annulus::new(0.96, 1.04));
    let leaf = meshes.add(RegularPolygon::new(1.0, 3));
    let bg = materials.add(Color::srgb(0.025, 0.092, 0.108));
    let border = materials.add(Color::srgb(0.06, 0.19, 0.20));
    commands.spawn((
        Mesh2d(line.clone()),
        MeshMaterial2d(border),
        Transform::from_xyz(0.0, 0.0, -9.0).with_scale(Vec3::new(82.0, 58.0, 1.0)),
    ));
    commands.spawn((
        Mesh2d(line.clone()),
        MeshMaterial2d(bg),
        Transform::from_xyz(0.0, 0.0, -8.0).with_scale(Vec3::new(80.0, 56.0, 1.0)),
    ));
    let speck = materials.add(Color::srgba(0.19, 0.45, 0.43, 0.12));
    for i in 0..110 {
        let x = ((i * 139 + 17) % 791) as f32 / 10.0 - 39.5;
        let y = ((i * 83 + 31) % 551) as f32 / 10.0 - 27.5;
        commands.spawn((
            Mesh2d(circle.clone()),
            MeshMaterial2d(speck.clone()),
            Transform::from_xyz(x, y, -6.0).with_scale(Vec3::splat(0.04 + (i % 5) as f32 * 0.024)),
        ));
    }
    commands.insert_resource(Art {
        circle,
        ring,
        leaf,
        line,
        player: materials.add(Color::srgb(0.16, 0.80, 0.68)),
        predator: materials.add(Color::srgb(0.93, 0.35, 0.23)),
        prey: materials.add(Color::srgb(0.50, 0.45, 0.77)),
        plant: materials.add(Color::srgb(0.62, 0.90, 0.34)),
        meat: materials.add(Color::srgb(0.96, 0.55, 0.44)),
        membrane: materials.add(Color::srgba(0.61, 1.0, 0.88, 0.70)),
        nucleus: materials.add(Color::srgba(0.02, 0.28, 0.30, 0.76)),
        white: materials.add(Color::srgb(0.92, 0.98, 0.90)),
        dark: materials.add(Color::srgb(0.018, 0.055, 0.064)),
        rock: materials.add(Color::srgb(0.16, 0.25, 0.27)),
    });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                left: px(26),
                right: px(26),
                height: px(78),
                padding: UiRect::all(px(12)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(16),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.045, 0.06, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CELL"),
                TextFont {
                    font_size: 25.0,
                    ..default()
                },
                TextColor(Color::srgb(0.62, 0.95, 0.83)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.84, 0.91, 0.89)),
                Node {
                    width: px(280),
                    ..default()
                },
                Hud,
            ));
            for (i, label, colour) in [
                (0, "HEALTH", Color::srgb(0.34, 0.85, 0.66)),
                (1, "ENERGY", Color::srgb(0.36, 0.65, 0.94)),
                (2, "GROWTH", Color::srgb(0.91, 0.77, 0.36)),
            ] {
                parent
                    .spawn(Node {
                        width: px(175),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        ..default()
                    })
                    .with_children(|group| {
                        group.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.56, 0.71, 0.71)),
                        ));
                        group
                            .spawn((
                                Node {
                                    width: percent(100),
                                    height: px(6),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.08, 0.17, 0.19)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Node {
                                        width: percent(100),
                                        height: percent(100),
                                        ..default()
                                    },
                                    BackgroundColor(colour),
                                    Meter(i),
                                ));
                            });
                    });
            }
        });
    let controls = if creature_mode { "W/S swim   A/D turn   Shift boost   Space bite   P pause\nR restart   1/2/3 diet   Esc exit" } else { "W/S swim   A/D turn   Shift boost   Space bite   P pause\nR restart   1/2/3 diet   F5 save   F9 load   Esc exit" };
    commands.spawn((Text::new(controls), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.61, 0.77, 0.77)), Node { position_type: PositionType::Absolute, left: px(28), bottom: px(22), ..default() }));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.92, 0.71)),
        Node {
            position_type: PositionType::Absolute,
            top: percent(40),
            left: percent(37),
            ..default()
        },
        Banner,
    ));
}

fn keyboard_input(keys: &ButtonInput<KeyCode>) -> CellInput {
    let axis =
        |positive, negative| (keys.pressed(positive) as i8 - keys.pressed(negative) as i8) as f32;
    CellInput {
        thrust: axis(KeyCode::KeyW, KeyCode::KeyS),
        turn: axis(KeyCode::KeyA, KeyCode::KeyD),
        boost: keys.pressed(KeyCode::ShiftLeft),
        bite: keys.pressed(KeyCode::Space),
    }
}

pub(crate) fn write_save(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = path.with_extension(format!("tmp-{}-{nonce}", std::process::id()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn read_save(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save exceeds 16 MiB limit",
        ));
    }
    Ok(bytes)
}

fn validate_source_snapshot(world: &CellWorld, source_enabled: bool) -> Result<(), String> {
    if source_enabled && world.creation.is_some() {
        return Err("save contains an authored creation; restart without source Cell inspection art to load it".into());
    }
    Ok(())
}

fn offscreen_step_delta(delta: f64, elapsed: f64, duration: f64, ready: bool) -> f64 {
    if !ready { return 0.0; }
    delta.min(0.1).min((duration - elapsed).max(0.0))
}

fn drive(time: Res<Time>, mut keys: ResMut<ButtonInput<KeyCode>>, mut game: ResMut<Game>, source: Option<Res<SourceKind>>, ready: Option<Res<OffscreenSimulationReady>>) {
    // Render preparation, text layout, and capture continue in their own systems.
    // Do not let pipeline compilation time advance the offscreen simulation.
    if ready.as_ref().is_some_and(|ready| !ready.0) { return; }
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if keys.just_pressed(KeyCode::KeyP) {
        game.paused = !game.paused;
    }
    let new_diet = if keys.just_pressed(KeyCode::Digit1) {
        Some(Diet::Herbivore)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(Diet::Carnivore)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(Diet::Omnivore)
    } else {
        None
    };
    if new_diet.is_some() && game.world.creation.is_some() {
        game.notice = "Edit mouth in studio to change diet".into();
    }
    if keys.just_pressed(KeyCode::KeyR) || (new_diet.is_some() && game.world.creation.is_none()) {
        if let Some(diet) = new_diet.filter(|_| game.world.creation.is_none()) {
            game.diet = diet;
        }
        if game.world.creation.is_some() {
            game.world.restart();
        } else {
            game.world = CellWorld::new(game.seed, game.diet);
        }
        game.paused = false;
        game.rebuild = true;
        game.notice.clear();
    }
    if keys.just_pressed(KeyCode::F5) {
        if source.as_deref() == Some(&SourceKind::Creature) {
            game.notice = "Save unavailable during source Creature inspection".into();
        } else {
        let result = game
            .world
            .snapshot()
            .to_bytes()
            .map_err(|e| e.to_string())
            .and_then(|saved| write_save(&game.save_path, &saved).map_err(|e| e.to_string()));
        game.notice = match result {
            Ok(()) => "Saved".into(),
            Err(e) => format!("Save failed: {e}"),
        };
        }
    }
    if keys.just_pressed(KeyCode::F9) {
        if source.as_deref() == Some(&SourceKind::Creature) {
            game.notice = "Load unavailable during source Creature inspection".into();
        } else {
        let result = read_save(&game.save_path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| CellSnapshot::from_bytes(&bytes).map_err(|e| e.to_string()))
            .and_then(|snapshot| CellWorld::from_snapshot(snapshot).map_err(|e| e.to_string()))
            .and_then(|world| {
                validate_source_snapshot(&world, source.is_some())?;
                if let Some(creation) = &world.creation {
                    crate::editor::creation_mesh(creation, true)?;
                }
                Ok(world)
            });
        match result {
            Ok(world) => {
                game.seed = world.seed();
                game.diet = world.player.diet;
                game.world = world;
                game.paused = false;
                game.rebuild = true;
                game.notice = "Loaded".into();
            }
            Err(e) => game.notice = format!("Load failed: {e}"),
        }
        }
    }
    let dt = if ready.is_some() {
        offscreen_step_delta(time.delta_secs_f64(), game.elapsed, game.demo_seconds.unwrap_or(1.5), true)
    } else {
        time.delta_secs_f64().min(0.1)
    };
    game.elapsed += dt;
    if game.demo_seconds.is_some() {
        for key in [
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::Space,
            KeyCode::ShiftLeft,
        ] {
            keys.release(key);
        }
        let player = &game.world.player;
        let desired = game
            .world
            .food
            .iter()
            .filter(|food| {
                game.world.can_perceive(food.position)
                    && matches!(
                        (game.diet, food.kind),
                        (Diet::Omnivore, _)
                            | (Diet::Herbivore, FoodKind::Plant)
                            | (Diet::Carnivore, FoodKind::Meat)
                    )
            })
            .min_by(|a, b| {
                let distance = |p: spore_core::cell::CellVec2| {
                    (p.x - player.position.x).powi(2) + (p.y - player.position.y).powi(2)
                };
                distance(a.position).total_cmp(&distance(b.position))
            })
            .map(|food| {
                (food.position.y - player.position.y).atan2(food.position.x - player.position.x)
            })
            .unwrap_or(player.heading);
        let error = (desired - player.heading + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        if error.abs() > 0.06 {
            keys.press(if error > 0.0 {
                KeyCode::KeyA
            } else {
                KeyCode::KeyD
            });
        }
        if error.abs() < 0.6 {
            keys.press(KeyCode::KeyW);
        }
        keys.press(KeyCode::Space);
    }
    if !game.paused && game.capture_frame.is_none() && dt > 0.0 {
        game.world.advance(dt, keyboard_input(&keys));
    }
}

fn add_actor(commands: &mut Commands, art: &Art, player: bool, dangerous: bool) -> Entity {
    let colour = if player {
        art.player.clone()
    } else if dangerous {
        art.predator.clone()
    } else {
        art.prey.clone()
    };
    commands
        .spawn((Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(colour),
                Transform::default().with_scale(Vec3::new(1.05, 0.85, 1.0)),
            ));
            parent.spawn((
                Mesh2d(art.ring.clone()),
                MeshMaterial2d(art.membrane.clone()),
                Transform::from_xyz(0.0, 0.0, 0.05).with_scale(Vec3::new(1.05, 0.85, 1.0)),
            ));
            parent.spawn((
                Mesh2d(art.circle.clone()),
                MeshMaterial2d(art.nucleus.clone()),
                Transform::from_xyz(-0.10, 0.0, 0.10).with_scale(Vec3::new(0.48, 0.43, 1.0)),
            ));
            for side in [-1.0, 1.0] {
                parent.spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.white.clone()),
                    Transform::from_xyz(0.55, 0.31 * side, 0.14).with_scale(Vec3::splat(0.17)),
                ));
                parent.spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.dark.clone()),
                    Transform::from_xyz(0.63, 0.31 * side, 0.16).with_scale(Vec3::splat(0.075)),
                ));
                parent.spawn((
                    Mesh2d(art.line.clone()),
                    MeshMaterial2d(art.membrane.clone()),
                    Transform::from_xyz(-1.20, 0.22 * side, 0.02)
                        .with_rotation(Quat::from_rotation_z(-0.2 * side))
                        .with_scale(Vec3::new(0.75, 0.075, 1.0)),
                ));
            }
            parent.spawn((
                Mesh2d(art.leaf.clone()),
                MeshMaterial2d(art.white.clone()),
                Transform::from_xyz(1.0, 0.0, 0.12)
                    .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(0.24, 0.32, 1.0)),
            ));
        })
        .id()
}

fn sync_scene(
    mut commands: Commands,
    art: Res<Art>,
    mut objects: ResMut<SceneObjects>,
    mut game: ResMut<Game>,
    mut transforms: Query<&mut Transform>,
    mut authored: ResMut<AuthoredArt>,
    source: Option<Res<SourcePlayerRender>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if authored.creation != game.world.creation {
        if let Some(mesh) = authored.mesh.take() {
            meshes.remove(mesh.id());
        }
        if let Some(creation) = &game.world.creation {
            match crate::editor::creation_mesh(creation, true) {
                Ok(mesh) => authored.mesh = Some(meshes.add(mesh)),
                Err(error) => {
                    game.notice = format!("Creation visual failed: {error}");
                }
            }
            if authored.material.is_none() {
                authored.material = Some(materials.add(Color::WHITE));
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
        .chain(game.world.enemies.iter().map(|a| (false, a)))
    {
        if !is_player && !game.world.can_perceive(actor.position) {
            continue;
        }
        let key = (0, actor.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            if is_player {
                if let Some(source) = &source {
                    return commands.spawn((Transform::default(), Visibility::Visible)).with_children(|parent| {
                        // The selected inspection path supplies its own inferred basis;
                        // neither establishes original gameplay-facing orientation.
                        let rotation = source.rotation;
                        parent.spawn((Mesh3d(source.mesh.clone()), MeshMaterial3d(source.material.clone()),
                            Transform::from_translation(-(rotation * source.center) * source.scale)
                                .with_rotation(rotation).with_scale(Vec3::splat(source.scale)), RenderLayers::layer(1)));
                    }).id();
                }
                if let (Some(mesh), Some(material)) = (&authored.mesh, &authored.material) {
                    return commands
                        .spawn((
                            Mesh2d(mesh.clone()),
                            MeshMaterial2d(material.clone()),
                            Transform::default(),
                        ))
                        .id();
                }
            }
            add_actor(
                &mut commands,
                &art,
                is_player,
                actor.behavior == CellBehavior::Predator,
            )
        });
        let t = Transform::from_xyz(actor.position.x, actor.position.y, 2.0)
            .with_rotation(Quat::from_rotation_z(actor.heading))
            .with_scale(Vec3::splat(actor.radius()));
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = t;
        } else {
            commands.entity(entity).insert(t);
        }
    }
    for food in &game.world.food {
        if !game.world.can_perceive(food.position) {
            continue;
        }
        let key = (1, food.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((
                    Mesh2d(art.leaf.clone()),
                    MeshMaterial2d(if food.kind == FoodKind::Plant {
                        art.plant.clone()
                    } else {
                        art.meat.clone()
                    }),
                    Transform::default(),
                ))
                .id()
        });
        let t = Transform::from_xyz(food.position.x, food.position.y, 1.0)
            .with_rotation(Quat::from_rotation_z(food.id as f32 * 0.73))
            .with_scale(Vec3::splat(food.radius()));
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = t;
        } else {
            commands.entity(entity).insert(t);
        }
    }
    for obstacle in &game.world.obstacles {
        let key = (2, obstacle.id);
        live.insert(key);
        let entity = *objects.0.entry(key).or_insert_with(|| {
            commands
                .spawn((
                    Mesh2d(art.circle.clone()),
                    MeshMaterial2d(art.rock.clone()),
                    Transform::default(),
                ))
                .id()
        });
        let t = Transform::from_xyz(obstacle.position.x, obstacle.position.y, 0.0)
            .with_scale(Vec3::splat(obstacle.radius));
        if let Ok(mut transform) = transforms.get_mut(entity) {
            *transform = t;
        } else {
            commands.entity(entity).insert(t);
        }
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

fn update_hud(
    game: Res<Game>,
    mut texts: Query<&mut Text, (With<Hud>, Without<Banner>)>,
    mut banners: Query<&mut Text, (With<Banner>, Without<Hud>)>,
    mut meters: Query<(&Meter, &mut Node)>,
) {
    for mut text in &mut texts {
        text.0 = format!(
            "{:?}  |  {} nutrients\nMass {:.1}  |  {} rivals\n{}",
            game.diet,
            game.world.eaten,
            game.world.player.mass,
            game.world.enemies.len(),
            if game.notice.is_empty() {
                game.world
                    .creation
                    .as_ref()
                    .map(|creation| {
                        format!(
                            "{} | awareness {:.0}m",
                            creation.name.chars().take(12).collect::<String>(),
                            game.world.perception_radius()
                        )
                    })
                    .unwrap_or_default()
            } else {
                game.notice.chars().take(38).collect()
            }
        );
    }
    for (meter, mut node) in &mut meters {
        node.width = percent(
            100.0
                * match meter.0 {
                    0 => game.world.player.health / game.world.player.max_health,
                    1 => game.world.player.energy,
                    _ => game.world.progress(),
                }
                .clamp(0.0, 1.0),
        );
    }
    for mut text in &mut banners {
        text.0 = if game.paused {
            "PAUSED\nP to return".into()
        } else {
            match game.world.state {
                CellState::Playing => String::new(),
                CellState::Evolved => "READY TO EVOLVE\nR to swim again".into(),
                CellState::Dead => "THE TIDE MOVES ON\nR to try again".into(),
            }
        };
    }
}

fn capture(mut commands: Commands, mut game: ResMut<Game>, target: Option<Res<crate::offscreen::OffscreenTarget>>, mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>, mut simulation_ready: Option<ResMut<OffscreenSimulationReady>>, layouts: Query<(&Text, Option<&bevy::text::TextLayoutInfo>)>, time: Res<Time>) {
    if let Some(readiness) = readiness.as_mut() {
        if let Some(error) = readiness.failure() { eprintln!("Cell offscreen render failed: {error}"); std::process::exit(1); }
        let text_ready = layouts.iter().all(|(text, layout)| text.0.is_empty() || layout.is_some_and(|layout| readiness.require_text(layout)));
        if !text_ready || !readiness.ready() {
            if time.elapsed_secs() > 120. { eprintln!("Cell offscreen renderer did not become ready"); std::process::exit(1); }
            return;
        }
        if let Some(simulation_ready) = simulation_ready.as_mut() {
            simulation_ready.0 = true;
        }
    }
    game.frames += 1;
    let duration = game.demo_seconds.unwrap_or(1.5);
    if game.capture_frame.is_none()
        && game.elapsed >= duration
        && (game.shot.is_some() || game.demo_seconds.is_some())
    {
        if let Some(path) = &game.shot {
            commands
                .spawn(target.as_ref().map_or_else(Screenshot::primary_window, |target| target.screenshot()))
                .observe(save_to_disk(path.clone()));
        }
        game.capture_frame = Some(game.frames);
        println!("CELL_RESULT state={:?} tick={} eaten={} kills={} progress={:.6} health={:.3} x={:.4} y={:.4}", game.world.state, game.world.tick, game.world.eaten, game.world.kills, game.world.progress(), game.world.player.health, game.world.player.position.x, game.world.player.position.y);
    }
    if let Some(frame) = game.capture_frame {
        if game.frames > frame + 10 && game.shot.as_ref().is_none_or(|path| path.exists()) {
            std::process::exit(0);
        }
        if game.frames > frame + 600 {
            eprintln!("Cell screenshot capture failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_art_requires_explicit_pair_and_rejects_creation_conflict() {
        let args = |parts: &[&str]| parts.iter().map(|part| part.to_string()).collect::<Vec<_>>();
        assert!(run(&args(&["--source-cell-install", "missing-install"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-cell-id", "7D00E384"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-cell-install", "missing-install", "--source-cell-id", "7D00E384", "--creation", "unused.rsc"]))
            .unwrap_err().contains("cannot be combined"));
        assert!(run(&args(&["--source-cell-id", "not-hex"]))
            .unwrap_err().contains("hexadecimal u32"));
        assert!(run(&args(&["--offscreen"]))
            .unwrap_err().contains("requires --shot"));
        assert!(run(&args(&["--source-creature-install", "missing-install"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-creature-instance", "40616200A1C46DF2"]))
            .unwrap_err().contains("must be provided together"));
        assert!(run(&args(&["--source-creature-instance", "A1C46DF2"]))
            .unwrap_err().contains("16 hexadecimal digits"));
        assert!(run(&args(&["--source-creature-install", "missing", "--source-creature-instance", "40616200A1C46DF2", "--creation", "unused.rsc"]))
            .unwrap_err().contains("cannot be combined"));
        assert!(run(&args(&["--source-cell-install", "missing", "--source-cell-id", "FD24574B", "--source-creature-install", "missing", "--source-creature-instance", "40616200A1C46DF2"]))
            .unwrap_err().contains("cannot be combined"));
    }

    #[test]
    fn source_creature_basis_maps_axes_for_inspection() {
        let rotation = creature_inspection_rotation();
        assert!((rotation * Vec3::X - Vec3::Z).length() < 1e-6);
        assert!((rotation * Vec3::Y - Vec3::X).length() < 1e-6);
        assert!((rotation * Vec3::Z - Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn inspection_image_uploads_levels_in_source_order() {
        use crate::assets::{TextureData, TextureMipChain};
        let texture = TextureData { width: 2, height: 2, rgba: vec![7; 16] };
        let chain = TextureMipChain { levels: vec![
            TextureData { width: 2, height: 2, rgba: vec![7; 16] },
            TextureData { width: 1, height: 1, rgba: vec![9; 4] },
        ] };
        let sampler = ImageSamplerDescriptor::default();
        let image = inspection_image(&texture, Some(&chain), &sampler);
        assert_eq!(image.texture_descriptor.mip_level_count, 2);
        assert_eq!(image.data.unwrap(), [vec![7; 16], vec![9; 4]].concat());
        assert_eq!(inspection_image(&texture, None, &sampler).texture_descriptor.mip_level_count, 1);
        assert_eq!(inspection_mip_lod_max(0, 2), 0.0);
        assert_eq!(inspection_mip_lod_max(1, 2), 1.0);
        assert_eq!(inspection_mip_lod_max(2, 2), 1.0);
        assert_eq!(creature_mip_chain_required(crate::assets::RASTER, 0).unwrap(), true);
        assert_eq!(creature_mip_chain_required(crate::assets::RW4, 0).unwrap(), false);
        assert!(creature_mip_chain_required(crate::assets::RW4, 1).is_err());
        assert!(creature_mip_chain_required(crate::assets::RW4, 2).is_err());
        assert!(creature_mip_chain_required(0, 0).is_err());
    }

    #[test]
    fn creature_inspection_denies_save_load_and_retains_mode_on_restart() {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("rustspore-cell-creature-{}-{nonce}.rsp", std::process::id()));
        std::fs::write(&path, b"existing sentinel").unwrap();
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(SourceKind::Creature)
            .insert_resource(Game {
                world: CellWorld::new(14, Diet::Herbivore), seed: 14, diet: Diet::Herbivore,
                paused: false, elapsed: 0., notice: String::new(), rebuild: false,
                save_path: path.clone(), shot: None, demo_seconds: None, capture_frame: None, frames: 0,
            }).add_systems(Update, drive);
        let step = |app: &mut App, key: KeyCode| {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.reset_all(); input.press(key);
            drop(input);
            app.update();
        };
        step(&mut app, KeyCode::F5);
        assert!(app.world().resource::<Game>().notice.contains("Save unavailable"));
        assert_eq!(std::fs::read(&path).unwrap(), b"existing sentinel");
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, KeyCode::F9);
        assert!(app.world().resource::<Game>().notice.contains("Load unavailable"));
        assert_eq!(app.world().resource::<Game>().world, before);
        step(&mut app, KeyCode::KeyR);
        assert_eq!(app.world().resource::<Game>().world, CellWorld::new(14, Diet::Herbivore));
        assert!(*app.world().resource::<SourceKind>() == SourceKind::Creature);
        assert!(app.world().resource::<Game>().rebuild);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn offscreen_simulation_waits_and_stops_at_requested_duration() {
        assert_eq!(offscreen_step_delta(0.1, 0.0, 1.5, false), 0.0);
        assert_eq!(offscreen_step_delta(0.5, 0.0, 1.5, true), 0.1);
        let final_step = offscreen_step_delta(0.1, 1.48, 1.5, true);
        assert!((final_step - 0.02).abs() < 1e-12);
        assert_eq!(offscreen_step_delta(0.1, 1.5, 1.5, true), 0.0);
        assert_eq!(offscreen_step_delta(0.1, 1.6, 1.5, true), 0.0);
    }

    #[test]
    fn drive_waits_for_offscreen_simulation_latch() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<OffscreenSimulationReady>()
            .insert_resource(Game {
                world: CellWorld::new(14, Diet::Herbivore), seed: 14, diet: Diet::Herbivore,
                paused: false, elapsed: 0., notice: String::new(), rebuild: true,
                save_path: PathBuf::from("unused"), shot: None, demo_seconds: Some(1.0),
                capture_frame: None, frames: 0,
            })
            .add_systems(Update, drive);
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
        app.update();
        assert_eq!(app.world().resource::<Game>().elapsed, 0.0);
        assert_eq!(app.world().resource::<Game>().world.tick, 0);
        app.world_mut().resource_mut::<OffscreenSimulationReady>().0 = true;
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
        app.update();
        assert!((app.world().resource::<Game>().elapsed - 0.1).abs() < 1e-12);
        assert!(app.world().resource::<Game>().world.tick > 0);
    }

    #[test]
    fn keyboard_mapping_controls_the_same_simulation_input_as_gameplay() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);
        keys.press(KeyCode::Space);
        keys.press(KeyCode::ShiftLeft);
        let input = keyboard_input(&keys);
        assert_eq!(input.thrust, 1.0);
        assert_eq!(input.turn, -1.0);
        assert!(input.bite && input.boost);
        keys.press(KeyCode::KeyS);
        keys.press(KeyCode::KeyA);
        let input = keyboard_input(&keys);
        assert_eq!(input.thrust, 0.0);
        assert_eq!(input.turn, 0.0);
    }

    #[test]
    fn save_file_replacement_roundtrips_and_keeps_old_data_on_failed_destination() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-cell-save-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("cell.rsp");
        let world = CellWorld::new(42, Diet::Carnivore);
        let bytes = world.snapshot().to_bytes().unwrap();
        write_save(&path, b"previous").unwrap();
        write_save(&path, &bytes).unwrap();
        let loaded = CellWorld::from_snapshot(
            CellSnapshot::from_bytes(&std::fs::read(&path).unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(loaded.seed(), 42);
        assert_eq!(loaded.player.diet, Diet::Carnivore);
        assert!(write_save(&path.join("invalid"), b"must not overwrite").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }

    #[test]
    fn gameplay_controls_pause_restart_save_load_and_failed_load() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-cell-controls-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("cell.rsp");
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Game {
                world: CellWorld::new(14, Diet::Herbivore),
                seed: 14,
                diet: Diet::Herbivore,
                paused: false,
                elapsed: 0.,
                notice: String::new(),
                rebuild: true,
                save_path: path.clone(),
                shot: None,
                demo_seconds: None,
                capture_frame: None,
                frames: 0,
            })
            .add_systems(Update, drive);
        let step = |app: &mut App, seconds: f64, pressed: &[KeyCode]| {
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
        };
        step(&mut app, 0.1, &[KeyCode::KeyW]);
        let moved = app.world().resource::<Game>().world.clone();
        assert!(moved.player.position.x > 0.0 && moved.tick > 0);
        step(&mut app, 0.1, &[KeyCode::KeyP, KeyCode::KeyW]);
        assert_eq!(app.world().resource::<Game>().world, moved);
        step(&mut app, 0., &[KeyCode::KeyP]);
        step(&mut app, 0., &[KeyCode::F5]);
        let saved = app
            .world()
            .resource::<Game>()
            .world
            .snapshot()
            .to_bytes()
            .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        step(&mut app, 0.1, &[KeyCode::KeyW]);
        assert_ne!(app.world().resource::<Game>().world.tick, moved.tick);
        step(&mut app, 0., &[KeyCode::F9]);
        assert_eq!(
            app.world()
                .resource::<Game>()
                .world
                .snapshot()
                .to_bytes()
                .unwrap(),
            saved
        );
        step(&mut app, 0., &[KeyCode::Digit2]);
        assert_eq!(
            app.world().resource::<Game>().world.player.diet,
            Diet::Carnivore
        );
        step(&mut app, 0., &[KeyCode::F9]);
        step(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<Game>().world,
            CellWorld::new(14, Diet::Herbivore)
        );
        std::fs::write(&path, b"broken save").unwrap();
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, 0., &[KeyCode::F9]);
        let game = app.world().resource::<Game>();
        assert_eq!(game.world, before);
        assert!(game.notice.starts_with("Load failed:"));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }

    fn authored_creation() -> Creation {
        use spore_core::editor::{AuthoredPart, PartArchetype};
        let mut creation = Creation::new(CreationKind::Cell, "Painted swimmer", 100).unwrap();
        for (id, kind) in [
            (2, PartArchetype::CarnivoreMouth),
            (3, PartArchetype::Flagellum),
            (4, PartArchetype::Eye),
        ] {
            creation
                .parts
                .push(AuthoredPart::new(id, format!("Part {id}"), kind, Some(1)));
        }
        creation.palette[0] = [0.8, 0.2, 0.4];
        creation
    }

    #[test]
    fn source_inspection_rejects_authored_snapshot_without_changing_it() {
        let mut world = CellWorld::new(77, Diet::Carnivore);
        world.apply_creation(authored_creation()).unwrap();
        assert!(validate_source_snapshot(&world, true).unwrap_err().contains("authored creation"));
        assert!(validate_source_snapshot(&world, false).is_ok());
        assert!(validate_source_snapshot(&CellWorld::new(14, Diet::Herbivore), true).is_ok());
    }

    #[test]
    fn authored_app_save_load_restart_preserves_appearance_stats_and_mouth_diet() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-cell-authored-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("cell.rsp");
        let creation = authored_creation();
        let mut world = CellWorld::new(77, Diet::Carnivore);
        world.apply_creation(creation.clone()).unwrap();
        let expected = world.loadout.clone();
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Game {
                world,
                seed: 77,
                diet: Diet::Carnivore,
                paused: false,
                elapsed: 0.,
                notice: String::new(),
                rebuild: true,
                save_path: path.clone(),
                shot: None,
                demo_seconds: None,
                capture_frame: None,
                frames: 0,
            })
            .add_systems(Update, drive);
        let step = |app: &mut App, keys: &[KeyCode]| {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.reset_all();
            for key in keys {
                input.press(*key);
            }
            drop(input);
            app.update();
        };
        step(&mut app, &[KeyCode::F5]);
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            game.world = CellWorld::new(999, Diet::Herbivore);
            game.seed = 999;
            game.diet = Diet::Herbivore;
        }
        step(&mut app, &[KeyCode::F9]);
        assert_eq!(
            app.world().resource::<Game>().world.creation,
            Some(creation.clone())
        );
        assert_eq!(app.world().resource::<Game>().world.loadout, expected);
        step(&mut app, &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<Game>().world.creation,
            Some(creation.clone())
        );
        assert_eq!(app.world().resource::<Game>().world.loadout, expected);
        assert_eq!(app.world().resource::<Game>().world.seed(), 77);
        step(&mut app, &[KeyCode::Digit1]);
        let game = app.world().resource::<Game>();
        assert_eq!(game.world.creation, Some(creation));
        assert_eq!(game.diet, Diet::Carnivore);
        assert!(game.notice.contains("mouth"));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }

    #[test]
    fn creation_cli_rejects_invalid_wrong_kind_and_conflicting_diet_before_opening_gpu() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rustspore-cell-cli-{}-{nonce}.rsc",
            std::process::id()
        ));
        std::fs::write(&path, b"invalid").unwrap();
        let args = vec!["--creation".into(), path.to_string_lossy().into_owned()];
        assert!(run(&args).is_err());
        std::fs::write(
            &path,
            EditorSnapshot {
                version: 1,
                revision: 1,
                creation: Creation::new(CreationKind::Creature, "Wrong kind", 100).unwrap(),
            }
            .to_bytes()
            .unwrap(),
        )
        .unwrap();
        assert!(run(&args).unwrap_err().contains("Cell creation"));
        std::fs::write(
            &path,
            EditorSnapshot {
                version: 1,
                revision: 1,
                creation: authored_creation(),
            }
            .to_bytes()
            .unwrap(),
        )
        .unwrap();
        let mut conflict = args.clone();
        conflict.extend(["--diet".into(), "herbivore".into()]);
        assert!(run(&conflict).unwrap_err().contains("contradicts"));
        let world = configured_world(14, Diet::Herbivore, false, Some(&path)).unwrap();
        assert_eq!(world.player.diet, Diet::Carnivore);
        assert!(world
            .food
            .iter()
            .take(8)
            .all(|food| food.kind == FoodKind::Meat));
        std::fs::write(&path, vec![0; MAX_SAVE_BYTES + 1]).unwrap();
        assert!(run(&args).unwrap_err().contains("bounded"));
        std::fs::remove_file(path).unwrap();
    }
}
