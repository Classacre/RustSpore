//! The asset viewer: a Bevy app that renders one resource at a time.
//!
//! Controls:
//!   Left / Right   previous / next mesh
//!   Up / Down      orbit pitch
//!   A / D          orbit yaw
//!   + / -          zoom
//!   T              next package texture (switches away from the model's own)
//!   M              toggle between the model's own texture and the package list
//!   R              reset the camera
//!   Esc            quit
//!
//! The viewer inspects saved geometry with Bevy's material. Creature targets use
//! their explicit authored LOD0 and its source texture; native animation and the
//! original shader/scene pipeline remain separate fidelity gates.

use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use crate::assets::AssetSource;

/// What the viewer should show: a mesh from the list, a prop's model, or a whole
/// creature's saved authored model.
#[derive(Debug, Clone, Copy)]
pub enum Target {
    Prop(u64),
    Creature(u64),
    EditedAssembly,
}

/// Marker for the entity holding the mesh currently on screen.
#[derive(Component)]
struct CurrentMesh;

/// Marker for the heads-up text.
#[derive(Component)]
struct Hud;

#[derive(Resource)]
struct Viewer {
    source: AssetSource,
    mesh_index: usize,
    texture_index: usize,
    yaw: f32,
    pitch: f32,
    distance: f32,
    status: String,
    /// Whether to prefer the texture stored inside the model over the indexed one.
    use_own_texture: bool,
    /// When set, show a prop's model or a saved authored creature instead of the
    /// indexed mesh.
    target: Option<Target>,
    edited_assembly: Option<(spore_core::mesh::MeshData, String)>,
    /// Set when the selected mesh or texture changes, cleared when rebuilt.
    dirty: bool,
    /// Replace the bound texture with a generated checkerboard.
    ///
    /// This is the only way to tell "the texture is applied" from "the model
    /// happens to be white", and it makes a wrong UV scale or a flipped V obvious.
    checker: bool,
    /// When set, render a few frames, write a screenshot to this path, and exit.
    shot: Option<PathBuf>,
    frames: u32,
    capture_frame: Option<u32>,
    /// Held open after exclusive creation; only the edited assembly uses this.
    exclusive_shot: Option<std::fs::File>,
    exclusive_result: std::sync::Arc<std::sync::Mutex<Option<Result<(), String>>>>,
    edited_started: std::time::Instant,
    edited_capture_started: Option<std::time::Instant>,
}

/// Run the viewer. Blocks until the window closes.
///
/// `shot` renders a handful of frames, writes a PNG of the window and exits. It
/// exists so the pipeline can be checked from a shell, where there are no eyes.
pub fn run(
    path: &str,
    shot: Option<PathBuf>,
    start: usize,
    checker: bool,
    target: Option<Target>,
) -> Result<(), String> {
    run_inner(path, shot, start, checker, target, None)
}

/// Headless experimental inspection of a specifically hash-bound edited payload.
pub fn run_edited_assembly(path: &str, payload: &std::path::Path, hash: &str, shot: PathBuf) -> Result<(), String> {
    let target_root = std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"))
        .map_err(|e| format!("target directory: {e}"))?;
    let parent = shot.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    let parent = std::fs::canonicalize(parent).map_err(|e| format!("screenshot parent: {e}"))?;
    if !parent.starts_with(&target_root) { return Err("screenshot parent must resolve under workspace target".into()); }
    if !shot.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")) { return Err("screenshot must have a .png extension".into()); }
    use std::io::Read;
    let file = std::fs::File::open(payload).map_err(|e| format!("edited payload: {e}"))?;
    let mut bytes = Vec::new();
    file.take(64 * 1024 * 1024 + 1).read_to_end(&mut bytes).map_err(|e| format!("edited payload: {e}"))?;
    if bytes.len() > 64 * 1024 * 1024 { return Err("edited payload exceeds 64 MiB".into()); }
    run_inner(path, Some(parent.join(shot.file_name().ok_or("missing screenshot filename")?)), 0, false,
        Some(Target::EditedAssembly), Some((bytes, hash.to_string())))
}

fn run_inner(
    path: &str, shot: Option<PathBuf>, start: usize, checker: bool,
    target: Option<Target>, edited: Option<(Vec<u8>, String)>,
) -> Result<(), String> {
    if let Some(path) = &shot {
        if !matches!(target, Some(Target::EditedAssembly)) && path.exists() {
            return Err("screenshot path already exists; choose a new output".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let mut source = AssetSource::open(path)?;
    if let Some(set) = crate::package_set() {
        source = source.with_set(set);
    }
    let mesh_index = initial_mesh_index(source.mesh_count(), start, matches!(target, Some(Target::EditedAssembly)))
        .ok_or_else(|| format!("{path} contains no meshes"))?;

    let edited_assembly = if let Some((bytes, hash)) = edited {
        let (summary, mesh) = source.edited_creature_assembly(&bytes, &hash).map_err(|e| e.to_string())?;
        let description = format!("Experimental source-part assembly inspection\nPayload SHA-256 {hash}\n{} parts resolved; Bevy untextured preview", summary.parts_drawn);
        println!("{description}\n{} vertices, {} triangles; no baked model or source skin", summary.vertices, summary.triangles);
        Some((mesh, description))
    } else { None };

    let exclusive_shot = if matches!(target, Some(Target::EditedAssembly)) {
        let destination = shot.as_ref().ok_or("edited assembly requires a screenshot")?;
        Some(reserve_edited_screenshot(destination)?)
    } else { None };
    let headless = shot.is_some();
    let edited_destination = if matches!(target, Some(Target::EditedAssembly)) { shot.clone() } else { None };
    let exclusive_result = std::sync::Arc::new(std::sync::Mutex::new(None));
    let viewer = Viewer {
        source,
        mesh_index,
        texture_index: 0,
        yaw: 0.6,
        pitch: 0.3,
        distance: 3.4,
        status: String::new(),
        use_own_texture: true,
        target,
        edited_assembly,
        checker,
        dirty: true,
        shot,
        frames: 0,
        capture_frame: None,
        exclusive_shot,
        exclusive_result: exclusive_result.clone(),
        edited_started: std::time::Instant::now(),
        edited_capture_started: None,
    };

    let mut app = App::new();
    if headless {
        crate::offscreen::configure(&mut app, 1280, 800);
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore viewer".to_string(),
                ..default()
            }),
            ..default()
        }));
    }
    if matches!(target, Some(Target::EditedAssembly)) {
        app.add_systems(Startup, log_edited_adapter);
    }
    app.insert_resource(viewer)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                controls,
                rebuild_when_dirty,
                apply_camera,
                update_hud,
                maybe_shoot,
            ),
        )
        .run();

    if let Some(path) = edited_destination {
        let outcome = exclusive_result.lock().map_err(|_| format!(
            "screenshot result lock poisoned; partial output may remain at {}", path.display()
        ))?.clone();
        return edited_capture_outcome(outcome, &path);
    }
    Ok(())
}

fn log_edited_adapter(adapter: Res<bevy::render::renderer::RenderAdapterInfo>) {
    println!("ASSEMBLY_GPU name={:?} backend={:?} type={:?}", adapter.name, adapter.backend, adapter.device_type);
}

fn setup_scene(mut commands: Commands) {
    // AmbientLight is a component in Bevy 0.18 and requires a camera.
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            color: Color::WHITE,
            brightness: 400.0,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.5, 0.0)),
    ));
    commands.spawn((
        Text::new("loading"),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(12.0),
            ..default()
        },
        Hud,
    ));
}

#[derive(Debug, PartialEq, Eq)]
enum TextureChoice { Source, Indexed, Untextured }

fn texture_choice(use_own: bool, creature: bool, source_available: bool) -> TextureChoice {
    if use_own {
        if source_available { TextureChoice::Source } else { TextureChoice::Untextured }
    } else if creature {
        TextureChoice::Untextured
    } else {
        TextureChoice::Indexed
    }
}

#[cfg(test)]
mod texture_choice_tests {
    use super::{texture_choice, TextureChoice};

    #[test]
    fn source_mode_never_falls_back_to_an_indexed_texture() {
        for creature in [false, true] {
            assert_eq!(texture_choice(true, creature, false), TextureChoice::Untextured);
            assert_eq!(texture_choice(true, creature, true), TextureChoice::Source);
        }
        assert_eq!(texture_choice(false, false, false), TextureChoice::Indexed);
        assert_eq!(texture_choice(false, false, true), TextureChoice::Indexed);
        assert_eq!(texture_choice(false, true, true), TextureChoice::Untextured);
    }
}

fn controls(keys: Res<ButtonInput<KeyCode>>, mut viewer: ResMut<Viewer>) {
    if keys.just_pressed(KeyCode::Escape) {
        if matches!(viewer.target, Some(Target::EditedAssembly)) {
            if let Some(path) = &viewer.shot { eprintln!("preview interrupted; partial output may remain at {}", path.display()); }
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    if viewer.source.mesh_count() > 0 && (keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Space)) {
        viewer.mesh_index = (viewer.mesh_index + 1) % viewer.source.mesh_count();
        viewer.dirty = true;
    }
    if viewer.source.mesh_count() > 0 && keys.just_pressed(KeyCode::ArrowLeft) {
        let n = viewer.source.mesh_count();
        viewer.mesh_index = (viewer.mesh_index + n - 1) % n;
        viewer.dirty = true;
    }
    if keys.just_pressed(KeyCode::KeyT) && viewer.source.texture_count() > 0 {
        viewer.texture_index = (viewer.texture_index + 1) % viewer.source.texture_count();
        viewer.use_own_texture = false;
        viewer.dirty = true;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        viewer.use_own_texture = !viewer.use_own_texture;
        viewer.dirty = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        viewer.yaw = 0.6;
        viewer.pitch = 0.3;
        viewer.distance = 3.4;
    }
    if keys.pressed(KeyCode::KeyA) {
        viewer.yaw -= 0.02;
    }
    if keys.pressed(KeyCode::KeyD) {
        viewer.yaw += 0.02;
    }
    if keys.pressed(KeyCode::KeyW) {
        viewer.pitch = (viewer.pitch + 0.02).min(1.5);
    }
    if keys.pressed(KeyCode::KeyS) {
        viewer.pitch = (viewer.pitch - 0.02).max(-1.5);
    }
    if keys.pressed(KeyCode::Equal) || keys.pressed(KeyCode::NumpadAdd) {
        viewer.distance = (viewer.distance * 0.98).max(1.2);
    }
    if keys.pressed(KeyCode::Minus) || keys.pressed(KeyCode::NumpadSubtract) {
        viewer.distance = (viewer.distance * 1.02).min(20.0);
    }
}

/// Build the Bevy mesh and material for the selected resource, when it changed.
fn rebuild_when_dirty(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut viewer: ResMut<Viewer>,
    existing: Query<Entity, With<CurrentMesh>>,
    mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>,
) {
    if !viewer.dirty {
        return;
    }
    viewer.dirty = false;

    for e in &existing {
        commands.entity(e).despawn();
    }

    let mut authored_texture = None;
    let mut authored_texture_note = String::new();
    let mut authored_sampler = None;
    let (key, data, prop_note) = match viewer.target {
        Some(Target::EditedAssembly) => {
            let (mesh, note) = viewer.edited_assembly.as_ref().expect("prevalidated assembly");
            ("EDITED PAYLOAD".to_string(), mesh.clone(), note.clone())
        },
        Some(Target::Creature(instance)) => match viewer.source.authored_creature_asset(instance, 0) {
            Ok(asset) => {
                let reference = asset.reference;
                authored_texture_note = format!("source texture {} {}x{} (base mip inspection)",
                    asset.texture_key, asset.texture.width, asset.texture.height);
                authored_texture = Some(asset.texture);
                authored_sampler = Some(asset.texture_binding);
                (reference.model.to_string(), asset.mesh, format!(
                    "creation {instance:016X}, PROP {}, authored LOD{}\nBevy material inspection; native shaders and animation pending",
                    reference.prop, reference.lod))
            },
            Err(err) => {
                viewer.status = format!("creature {instance:016X}\nERROR: {err}");
                return;
            }
        },
        Some(Target::Prop(instance)) => match viewer.source.prop_mesh(instance) {
            Ok((resolved, mesh)) => {
                let (model_type, model_instance) = resolved.model;
                if model_type == crate::assets::GMDL {
                    match viewer.source.prop_gmdl_source_texture(instance) {
                        Ok((texture_key, binding, texture)) => {
                            authored_texture_note = format!("GMDL sampler 0 {} {}x{} (base mip inspection)", texture_key, texture.width, texture.height);
                            authored_texture = Some(texture);
                            authored_sampler = Some(binding);
                        }
                        Err(err) => authored_texture_note = format!("GMDL sampler 0 unavailable: {err}"),
                    }
                }
                (
                    format!("{model_type:08X}:{model_instance:016X}"),
                    mesh,
                    format!(
                        "prop {:016X}, {} LOD slot(s), {} texture(s)",
                        instance,
                        resolved.lods.len(),
                        resolved.textures.len()
                    ),
                )
            }
            Err(err) => {
                viewer.status = format!("prop {instance:016X}\nERROR: {err}");
                return;
            }
        },
        None => {
            let key = viewer
                .source
                .mesh_key(viewer.mesh_index)
                .expect("index in range");
            match viewer.source.mesh(viewer.mesh_index) {
                Ok(d) => (key.to_string(), d, String::new()),
                Err(err) => {
                    viewer.status = format!(
                        "mesh {}/{}  {key}\nERROR: {err}",
                        viewer.mesh_index + 1,
                        viewer.source.mesh_count()
                    );
                    return;
                }
            }
        }
    };

    let positions: Vec<[f32; 3]> = data.positions.iter().map(|p| [p.x, p.y, p.z]).collect();
    let normals: Vec<[f32; 3]> = data.normals.iter().map(|n| [n.x, n.y, n.z]).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());
    mesh.insert_indices(Indices::U32(data.indices.clone()));
    let mesh_handle = meshes.add(mesh);

    // Only use an indexed package texture when the user explicitly selects it.
    let base_colour = Color::srgb(0.85, 0.85, 0.85);
    let mut material = StandardMaterial {
        base_color: Color::srgb(0.85, 0.85, 0.85),
        perceptual_roughness: 0.75,
        metallic: 0.0,
        ..default()
    };
    let mut tex_note = String::new();

    let own = if !viewer.use_own_texture {
        None
    } else {
        match viewer.target {
            Some(Target::Prop(_)) if !authored_texture_note.is_empty() => {
                tex_note = authored_texture_note;
                authored_texture.take()
            },
            Some(Target::Prop(instance)) => match viewer.source.prop_texture(instance) {
                Some((t, note)) => {
                    tex_note = note;
                    Some(t)
                }
                None => None,
            },
            Some(Target::EditedAssembly) => None,
            Some(Target::Creature(_)) => {
                tex_note = authored_texture_note;
                authored_texture.take()
            },
            None => viewer.source.mesh_texture(viewer.mesh_index),
        }
    };
    let chosen = match texture_choice(viewer.use_own_texture, matches!(viewer.target, Some(Target::Creature(_) | Target::EditedAssembly)), own.is_some()) {
        TextureChoice::Source => {
            if tex_note.is_empty() {
                tex_note = viewer
                    .source
                    .mesh_texture_label(viewer.mesh_index)
                    .unwrap_or_else(|| "embedded texture".to_string());
            }
            own
        }
        TextureChoice::Untextured => {
            if viewer.use_own_texture && tex_note.is_empty() {
                tex_note = "source texture unavailable (untextured)".to_string();
            }
            None
        }
        TextureChoice::Indexed => {
            if viewer.source.texture_count() > 0 {
                match viewer.source.texture(viewer.texture_index) {
                    Ok(t) => {
                        tex_note = format!(
                            "texture {}/{} {}x{}",
                            viewer.texture_index + 1,
                            viewer.source.texture_count(),
                            t.width,
                            t.height
                        );
                        Some(t)
                    }
                    Err(err) => {
                        tex_note = format!("texture error: {err}");
                        None
                    }
                }
            } else {
                None
            }
        }
    };

    if tex_note.is_empty() {
        tex_note = "no texture".to_string();
    }

    let chosen = if viewer.checker {
        tex_note = "checkerboard (UV test)".to_string();
        Some(checkerboard())
    } else {
        chosen
    };

    material.base_color = base_colour;

    if let Some(t) = chosen {
        let mut img = Image::new(
            Extent3d {
                width: t.width,
                height: t.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            t.rgba,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let sampler = if let Some(binding) = authored_sampler.filter(|_| !viewer.checker) {
            match authored_sampler_descriptor(binding) {
                Ok(sampler) => sampler,
                Err(error) => {
                    viewer.status = format!("{key}\nERROR: {error}");
                    return;
                }
            }
        } else { ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            ..default()
        }};
        img.sampler = ImageSampler::Descriptor(sampler);
        let image = images.add(img);
        if let Some(readiness) = readiness.as_mut() { readiness.require_image(&image); }
        material.base_color_texture = Some(image);
        material.base_color = Color::WHITE;
    }
    let material_handle = materials.add(material);

    // Centre the mesh and scale it to a consistent size, because resource scales
    // vary by orders of magnitude.
    let radius = data.radius();
    let (min, max) = data
        .bounds()
        .unwrap_or((spore_core::Vec3::ZERO, spore_core::Vec3::ZERO));
    let centre = Vec3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let scale = 1.0 / radius;

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_scale(Vec3::splat(scale)).with_translation(-centre * scale),
        CurrentMesh,
    ));

    let heading = if prop_note.is_empty() {
        format!(
            "mesh {}/{}  {key}",
            viewer.mesh_index + 1,
            viewer.source.mesh_count()
        )
    } else {
        format!("{key}\n{prop_note}")
    };
    viewer.status = format!(
        "{heading}\n{} vertices, {} triangles, normals {}  |  {}\n{}",
        data.positions.len(),
        data.triangle_count(),
        if data.normals_from_file {
            "from file"
        } else {
            "computed"
        },
        tex_note,
        "arrows/WASD orbit, +/- zoom, T texture, M own/package, R reset, Esc quit"
    );
}

fn authored_sampler_descriptor(
    binding: spore_core::shader_binding::TextureBinding,
) -> Result<ImageSamplerDescriptor, String> {
    let address = match binding.address_mode() {
        Some(1) => ImageAddressMode::Repeat,
        Some(2) => ImageAddressMode::MirrorRepeat,
        Some(3) => ImageAddressMode::ClampToEdge,
        _ => return Err("authored sampler has unresolved addressing".into()),
    };
    let filter = match binding.min_mag_filter() {
        Some(1) => ImageFilterMode::Nearest,
        Some(2) => ImageFilterMode::Linear,
        _ => return Err("authored sampler has unresolved filtering".into()),
    };
    let mip = match binding.mip_filter() {
        Some(0 | 1) => ImageFilterMode::Nearest,
        Some(2) => ImageFilterMode::Linear,
        _ => return Err("authored sampler has unresolved mip filtering".into()),
    };
    // This generic inspection image decodes the saved base level only.
    // Original multi-level shader draws are a separate renderer gate.
    Ok(ImageSamplerDescriptor {
        address_mode_u: address, address_mode_v: address, address_mode_w: address,
        min_filter: filter, mag_filter: filter, mipmap_filter: mip,
        lod_min_clamp: 0.0, lod_max_clamp: 0.0, ..default()
    })
}

/// A checkerboard with a red edge, so orientation and scale are both readable.
fn checkerboard() -> crate::assets::TextureData {
    const N: u32 = 256;
    // Small cells on purpose: a model part usually occupies a small patch of its
    // atlas, so a coarse pattern looks like a flat colour and proves nothing.
    const CELL: u32 = 8;
    let mut rgba = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let dark = ((x / CELL) + (y / CELL)) % 2 == 0;
            let edge = x < 2 || y < 2;
            let px = if edge {
                [220u8, 40, 40, 255]
            } else if dark {
                [60, 60, 70, 255]
            } else {
                [225, 225, 230, 255]
            };
            rgba.extend_from_slice(&px);
        }
    }
    crate::assets::TextureData {
        width: N,
        height: N,
        rgba,
    }
}

fn apply_camera(viewer: Res<Viewer>, mut cam: Query<&mut Transform, With<Camera3d>>) {
    let Ok(mut t) = cam.single_mut() else {
        return;
    };
    let d = viewer.distance;
    let (sy, cy) = viewer.yaw.sin_cos();
    let (sp, cp) = viewer.pitch.sin_cos();
    let pos = Vec3::new(d * cp * sy, d * sp, d * cp * cy);
    *t = Transform::from_translation(pos).looking_at(Vec3::ZERO, Vec3::Y);
}

/// Screenshot support: wait for the scene to settle, capture, then exit.
fn initial_mesh_index(count: usize, start: usize, edited: bool) -> Option<usize> {
    if count == 0 && !edited { None } else { Some(start.min(count.saturating_sub(1))) }
}

fn reserve_edited_screenshot(path: &std::path::Path) -> Result<std::fs::File, String> {
    std::fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|e| format!("cannot exclusively create screenshot {}: {e}", path.display()))
}

fn edited_capture_outcome(outcome: Option<Result<(), String>>, path: &std::path::Path) -> Result<(), String> {
    match outcome {
        Some(Ok(())) => Ok(()),
        Some(Err(error)) => Err(format!("screenshot failed: {error}; partial output may remain at {}", path.display())),
        None => Err(format!("renderer exited without screenshot completion; partial output may remain at {}", path.display())),
    }
}

fn encode_edited_png<W: std::io::Write + std::io::Seek>(
    image: &Image, writer: &mut W,
) -> Result<(), String> {
    let dynamic = image.clone().try_into_dynamic().map_err(|e| format!("screenshot conversion: {e}"))?;
    dynamic.to_rgb8().write_to(writer, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encoding or writing: {e}"))
}

fn maybe_shoot(
    time: Res<Time>,
    mut commands: Commands,
    mut viewer: ResMut<Viewer>,
    target: Option<Res<crate::offscreen::OffscreenTarget>>,
    mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>,
    labels: Query<&bevy::text::TextLayoutInfo, With<Hud>>,
) {
    let Some(path) = viewer.shot.clone() else {
        return;
    };
    viewer.frames += 1;
    if let Some(error) = readiness.as_ref().and_then(|r| r.failure()) {
        eprintln!("viewer GPU pipeline failed: {error}{}", if matches!(viewer.target, Some(Target::EditedAssembly)) { format!("; partial output may remain at {}", path.display()) } else { String::new() });
        std::process::exit(1);
    }
    let edited = matches!(viewer.target, Some(Target::EditedAssembly));
    let readiness_timed_out = if edited { viewer.edited_started.elapsed().as_secs_f64() > 120.0 } else { time.elapsed_secs() > 120. };
    if viewer.capture_frame.is_none() && readiness_timed_out {
        eprintln!("viewer renderer did not become ready for capture{}", if matches!(viewer.target, Some(Target::EditedAssembly)) { format!("; partial output may remain at {}", path.display()) } else { String::new() });
        std::process::exit(1);
    }
    let text_ready = readiness.as_mut().is_none_or(|r| {
        labels.iter().count() == 1 && labels.iter().all(|layout| r.require_text(layout))
    });
    if viewer.capture_frame.is_none() && viewer.frames >= 40 && time.elapsed_secs() >= 3.0
        && text_ready && readiness.as_ref().is_none_or(|r| r.ready()) {
        let capture = target.as_ref()
            .map_or_else(Screenshot::primary_window, |target| target.screenshot());
        if let Some(mut file) = viewer.exclusive_shot.take() {
            let result = viewer.exclusive_result.clone();
            commands.spawn(capture).observe(move |event: On<bevy::render::view::screenshot::ScreenshotCaptured>| {
                use std::io::Write;
                let outcome = encode_edited_png(&event.image, &mut file)
                    .and_then(|_| file.flush().map_err(|e| format!("flush: {e}")))
                    .and_then(|_| file.sync_all().map_err(|e| format!("sync: {e}")));
                *result.lock().expect("screenshot result lock poisoned") = Some(outcome);
            });
        } else {
            commands.spawn(capture).observe(save_to_disk(path.clone()));
        }
        viewer.capture_frame = Some(viewer.frames);
        if edited { viewer.edited_capture_started = Some(std::time::Instant::now()); }
    }
    // Give the capture a moment to reach disk, then stop. The hard limit keeps a
    // failed capture from hanging the process.
    if let Some(frame) = viewer.capture_frame {
        if matches!(viewer.target, Some(Target::EditedAssembly)) {
            let outcome = viewer.exclusive_result.lock().expect("screenshot result lock poisoned").take();
            if let Some(outcome) = outcome {
                match outcome {
                    Ok(()) => { println!("screenshot written and synced: {}", path.display()); std::process::exit(0); }
                    Err(error) => { eprintln!("screenshot failed: {error}; partial output may remain at {}", path.display()); std::process::exit(1); }
                }
            }
            if viewer.edited_capture_started.is_some_and(|started| started.elapsed().as_secs_f64() > 120.0) {
                eprintln!("screenshot capture timed out; partial output may remain at {}", path.display());
                std::process::exit(1);
            }
            return;
        }
        if viewer.frames > frame + 10 && path.exists() {
            std::process::exit(0);
        }
        if viewer.frames > frame + 600 {
            eprintln!("viewer screenshot capture failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod edited_capture_tests {
    use super::*;
    use std::io::{self, Seek, SeekFrom, Write};

    fn pixel() -> Image {
        Image::new(Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            TextureDimension::D2, vec![20, 40, 60, 255], TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default())
    }

    struct Fails;
    impl Write for Fails {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> { Err(io::Error::other("injected write failure")) }
        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }
    impl Seek for Fails {
        fn seek(&mut self, _: SeekFrom) -> io::Result<u64> { Ok(0) }
    }

    #[test]
    fn edited_capture_is_exclusive_and_encodes_a_valid_png() {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("rustspore-capture-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("capture.png");
        let mut file = reserve_edited_screenshot(&path).unwrap();
        assert!(reserve_edited_screenshot(&path).is_err());
        encode_edited_png(&pixel(), &mut file).unwrap();
        file.flush().unwrap();
        file.sync_all().unwrap();
        drop(file);
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).unwrap().to_rgb8().into_raw(), [20, 40, 60]);
        assert!(reserve_edited_screenshot(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn edited_capture_reports_write_failure() {
        assert!(encode_edited_png(&pixel(), &mut Fails).unwrap_err().contains("injected write failure"));
    }

    #[test]
    fn edited_capture_requires_acknowledged_success() {
        let path = std::path::Path::new("capture.png");
        assert!(edited_capture_outcome(Some(Ok(())), path).is_ok());
        let missing = edited_capture_outcome(None, path).unwrap_err();
        assert!(missing.contains("without screenshot completion") && missing.contains("capture.png"));
        let failed = edited_capture_outcome(Some(Err("injected failure".into())), path).unwrap_err();
        assert!(failed.contains("injected failure") && failed.contains("partial output"));
    }

    #[test]
    fn edited_assembly_does_not_require_primary_meshes() {
        assert_eq!(initial_mesh_index(0, 0, false), None);
        assert_eq!(initial_mesh_index(0, 4, true), Some(0));
        assert_eq!(initial_mesh_index(2, 4, false), Some(1));
    }
}

fn update_hud(viewer: Res<Viewer>, mut text: Query<&mut Text, With<Hud>>) {
    for mut t in &mut text {
        if t.0 != viewer.status {
            t.0 = viewer.status.clone();
        }
    }
}
