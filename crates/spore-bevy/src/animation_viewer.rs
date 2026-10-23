//! Real RW4 skeletons, weights and clips rendered with Bevy GPU skinning.
//! The local-pose interpretation is measured on matched RW4 rigs. Interpolation
//! is the explicit RustSpore linear/slerp policy, not a recovered engine curve.

use bevy::{
    asset::RenderAssetUsages,
    mesh::{
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
        Indices, PrimitiveTopology, VertexAttributeValues,
    },
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use spore_core::{
    mesh::MeshData,
    rw4_animation::{
        self as animation, AffineTransform, KeyframeClip, SampleValue, Skeleton, TrackValues,
        VertexSkinning,
    },
};
use std::path::PathBuf;

#[derive(Resource)]
struct Model {
    geometry: MeshData,
    skeleton: Skeleton,
    inverse_bind: Vec<Mat4>,
    bind_local: Vec<Mat4>,
    influences: Vec<VertexSkinning>,
    clips: Vec<KeyframeClip>,
    clip: usize,
    time: f32,
    paused: bool,
    bind: bool,
    normalization: Mat4,
    shot: Option<PathBuf>,
    frames: u32,
    captured: Option<u32>,
    instance: u64,
    cpu_reference: bool,
    camera: Vec3,
}
#[derive(Component)]
struct Joint(usize);
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct Orbit;

fn matrix(a: AffineTransform) -> Mat4 {
    let r = a.rows;
    Mat4::from_cols_array(&[
        r[0][0], r[1][0], r[2][0], 0., r[0][1], r[1][1], r[2][1], 0., r[0][2], r[1][2], r[2][2],
        0., r[0][3], r[1][3], r[2][3], 1.,
    ])
}

fn globals(local: &[Mat4], skeleton: &Skeleton) -> Result<Vec<Mat4>, String> {
    if local.len() != skeleton.bones.len() {
        return Err("pose and skeleton lengths differ".into());
    }
    let mut output = vec![Mat4::IDENTITY; local.len()];
    let mut states = vec![0; local.len()];
    for i in 0..local.len() {
        let mut path = Vec::new();
        let mut cursor = Some(i);
        while let Some(j) = cursor {
            if j >= local.len() {
                return Err("skeleton parent out of bounds".into());
            }
            match states[j] {
                1 => return Err("skeleton parent cycle".into()),
                2 => break,
                _ => (),
            }
            states[j] = 1;
            path.push(j);
            cursor = skeleton.bones[j].parent;
        }
        for j in path.into_iter().rev() {
            output[j] = skeleton.bones[j]
                .parent
                .map_or(local[j], |p| output[p] * local[j]);
            if !output[j].is_finite() {
                return Err("nonfinite global pose".into());
            }
            states[j] = 2;
        }
    }
    Ok(output)
}

fn local_pose(model: &Model) -> Result<Vec<Mat4>, String> {
    let mut local = model.bind_local.clone();
    if !model.bind {
        for sample in model.clips[model.clip]
            .sample(model.time, !model.paused)
            .map_err(|e| e.to_string())?
        {
            if let SampleValue::Transform(value) = sample.value {
                if let Some(i) = model
                    .skeleton
                    .bones
                    .iter()
                    .position(|b| b.id == sample.bone_id)
                {
                    local[i] = matrix(value.affine().map_err(|e| e.to_string())?);
                }
            }
        }
    }
    Ok(local)
}

fn pose(model: &Model) -> Result<Vec<Mat4>, String> {
    globals(&local_pose(model)?, &model.skeleton)
}

fn gpu_transform(matrix: Mat4) -> Result<Transform, String> {
    let value = Transform::from_matrix(matrix);
    if !matrix.is_finite() || !value.to_matrix().abs_diff_eq(matrix, 1e-4) {
        return Err("joint local matrix contains shear or a degenerate transform".into());
    }
    Ok(value)
}

fn cpu_deform(model: &Model, joints: &[Mat4]) -> Vec<Vec3> {
    model
        .geometry
        .positions
        .iter()
        .zip(&model.influences)
        .map(|(p, w)| {
            let p = Vec3::new(p.x, p.y, p.z);
            (0..4)
                .map(|k| {
                    (joints[w.joints[k] as usize] * model.inverse_bind[w.joints[k] as usize])
                        .transform_point3(p)
                        * w.weights[k]
                })
                .sum()
        })
        .collect()
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("usage: --animate <package> <RW4 instance hex> [--clip <index>] [--pose <seconds|bind>] [--shot <new PNG>]".into());
    }
    let instance = crate::parse_hex_arg(&args[1], "RW4 instance")?;
    let index = dbpf::PackageIndex::open(&args[0]).map_err(|e| e.to_string())?;
    let entry = index
        .find_instance(0x2f4e681b, instance)
        .ok_or_else(|| format!("no RW4 {instance:016X} in package"))?;
    let data = index.read(entry).map_err(|e| e.to_string())?;
    let rw = spore_core::rw4::parse(&data).map_err(|e| e.to_string())?;
    let set = animation::parse(&rw, &data).map_err(|e| e.to_string())?;
    if set.bindings.len() != 1 {
        return Err(format!(
            "expected one skin binding, got {}",
            set.bindings.len()
        ));
    }
    let binding = set.bindings[0].value;
    let skeleton = set
        .skeletons
        .iter()
        .find(|s| s.section_index == binding.skeleton)
        .ok_or("missing bound skeleton")?
        .value
        .clone();
    let skin = &set
        .animation_skins
        .iter()
        .find(|s| s.section_index == binding.animation_skin)
        .ok_or("missing bound inverse skin")?
        .value;
    if skin.inverse_bind.len() != skeleton.bones.len() || skeleton.bones.is_empty() {
        return Err("inverse bind/skeleton size mismatch".into());
    }
    if skeleton.bones.len() > 256 {
        return Err("this renderer supports at most 256 joints per skin".into());
    }
    let inverse_bind: Vec<_> = skin.inverse_bind.iter().map(|m| matrix(*m)).collect();
    let mut bind_global = Vec::new();
    for m in &skin.inverse_bind {
        bind_global.push(matrix(m.inverse().ok_or("singular inverse bind")?));
    }
    let bind_local = skeleton
        .bones
        .iter()
        .enumerate()
        .map(|(i, b)| {
            b.parent
                .map_or(bind_global[i], |p| inverse_bind[p] * bind_global[i])
        })
        .collect();
    let geometry = spore_core::mesh::from_rw4(&data, &rw).map_err(|e| e.to_string())?;
    let mut influences = Vec::new();
    for mesh in rw.meshes(&data) {
        influences.extend(
            animation::vertex_skinning(&rw, &data, &mesh, skeleton.bones.len())
                .map_err(|e| e.to_string())?,
        );
    }
    if influences.len() != geometry.positions.len() {
        return Err(format!(
            "geometry/influence mismatch {} vs {}",
            geometry.positions.len(),
            influences.len()
        ));
    }
    let clips: Vec<_> = set
        .clips
        .into_iter()
        .filter(|c| {
            let tracks: Vec<_> = c
                .value
                .tracks
                .iter()
                .filter(|t| matches!(t.values, TrackValues::Transform(_)))
                .collect();
            c.value.is_sampleable()
                && c.value.id == skeleton.id
                && !tracks.is_empty()
                && tracks.len() == skeleton.bones.len()
                && tracks
                    .iter()
                    .all(|t| skeleton.bones.iter().filter(|b| b.id == t.bone_id).count() == 1)
                && tracks.iter().enumerate().all(|(i, t)| {
                    tracks[..i]
                        .iter()
                        .all(|previous| previous.bone_id != t.bone_id)
                })
        })
        .map(|c| c.value)
        .collect();
    if clips.is_empty() {
        return Err("no sampleable transform clips matching this rig".into());
    }
    let (min, max) = geometry.bounds().ok_or("empty geometry")?;
    let center = Vec3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let normalization =
        Mat4::from_scale(Vec3::splat(1.0 / geometry.radius())) * Mat4::from_translation(-center);
    let mut model = Model {
        geometry,
        skeleton,
        inverse_bind,
        bind_local,
        influences,
        clips,
        clip: 0,
        time: 0.,
        paused: false,
        bind: false,
        normalization,
        shot: None,
        frames: 0,
        captured: None,
        instance,
        cpu_reference: false,
        camera: Vec3::new(2.5, 1.7, 3.3),
    };
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--cpu-reference" {
            model.cpu_reference = true;
            i += 1;
            continue;
        }
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("{} requires a value", args[i]))?;
        match args[i].as_str() {
            "--view" => {
                model.camera = match value.as_str() {
                    "front" => Vec3::new(2.5, 1.7, 3.3),
                    "reverse" => Vec3::new(-2.5, -1.7, -3.3),
                    _ => return Err("view must be front or reverse".into()),
                }
            }
            "--clip" => {
                model.clip = value.parse::<usize>().map_err(|_| "invalid clip index")?;
                if model.clip >= model.clips.len() {
                    return Err("clip out of range".into());
                }
            }
            "--pose" if value == "bind" => {
                model.bind = true;
                model.paused = true;
            }
            "--pose" => {
                model.time = value.parse::<f32>().map_err(|_| "invalid pose time")?;
                if !model.time.is_finite() || model.time < 0. {
                    return Err("pose time must be finite and nonnegative".into());
                }
                model.paused = true;
            }
            "--shot" => {
                let path = PathBuf::from(value);
                if path.exists() {
                    return Err("screenshot path exists".into());
                }
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                model.shot = Some(path);
            }
            other => return Err(format!("unknown animation option {other}")),
        }
        i += 2;
    }
    if model.cpu_reference && !model.paused {
        return Err("CPU reference requires a static --pose".into());
    }
    let bind_pose = globals(&model.bind_local, &model.skeleton)?;
    let bind_points = cpu_deform(&model, &bind_pose);
    let bind_error = bind_points
        .iter()
        .zip(&model.geometry.positions)
        .map(|(a, b)| a.distance(Vec3::new(b.x, b.y, b.z)))
        .fold(0.0, f32::max);
    if bind_error > model.geometry.radius().max(1.) * 0.0001 {
        return Err(format!("bind pose changes source geometry by {bind_error}"));
    }
    let points = cpu_deform(&model, &pose(&model)?);
    for matrix in local_pose(&model)? {
        gpu_transform(matrix)?;
    }
    if points.iter().any(|p| !p.is_finite()) {
        return Err("nonfinite deformed geometry".into());
    }
    let displacement = points
        .iter()
        .zip(&model.geometry.positions)
        .map(|(a, b)| a.distance(Vec3::new(b.x, b.y, b.z)))
        .fold(0.0, f32::max);
    println!("SKIN_MODEL instance={instance:016X} vertices={} triangles={} bones={} clips={} bind_max_error={bind_error:.9} pose_max_displacement={displacement:.9}",model.geometry.positions.len(),model.geometry.triangle_count(),model.skeleton.bones.len(),model.clips.len());
    let mut app = App::new();
    if model.shot.is_some() {
        crate::offscreen::configure(&mut app, 1280, 800);
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore | RW4 animation".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }));
    }
    app.insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.05)))
        .insert_resource(model)
        .insert_resource(GlobalAmbientLight {
            brightness: 350.,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (animate, hud, capture).chain())
        .run();
    Ok(())
}

fn setup(
    mut commands: Commands,
    model: Res<Model>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inverse_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    adapter: Res<RenderAdapterInfo>,
) {
    println!(
        "SKIN_GPU name={:?} backend={:?} type={:?}",
        adapter.name, adapter.backend, adapter.device_type
    );
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(model.camera).looking_at(Vec3::ZERO, Vec3::Y),
        Orbit,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(3., 5., 4.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let p = local_pose(&model).expect("validated initial pose");
    let root = commands
        .spawn((
            Transform::from_matrix(model.normalization),
            Visibility::Visible,
        ))
        .id();
    let joints: Vec<_> = p
        .iter()
        .enumerate()
        .map(|(i, m)| {
            commands
                .spawn((
                    Joint(i),
                    gpu_transform(*m).expect("validated joint"),
                    Visibility::Inherited,
                ))
                .id()
        })
        .collect();
    for (i, bone) in model.skeleton.bones.iter().enumerate() {
        commands
            .entity(bone.parent.map_or(root, |p| joints[p]))
            .add_child(joints[i]);
    }
    let (positions, normals) = if model.cpu_reference {
        let globals = pose(&model).expect("validated pose");
        let positions = cpu_deform(&model, &globals)
            .iter()
            .map(|p| model.normalization.transform_point3(*p).to_array())
            .collect::<Vec<_>>();
        let normals = model
            .geometry
            .normals
            .iter()
            .zip(&model.influences)
            .map(|(n, w)| {
                let blended = (0..4).fold(Mat4::ZERO, |m, k| {
                    m + (globals[w.joints[k] as usize] * model.inverse_bind[w.joints[k] as usize])
                        * w.weights[k]
                });
                (Mat3::from_mat4(blended).inverse().transpose() * Vec3::new(n.x, n.y, n.z))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect::<Vec<_>>();
        (positions, normals)
    } else {
        (
            model
                .geometry
                .positions
                .iter()
                .map(|p| [p.x, p.y, p.z])
                .collect(),
            model
                .geometry
                .normals
                .iter()
                .map(|p| [p.x, p.y, p.z])
                .collect(),
        )
    };
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, model.geometry.uvs.clone())
    .with_inserted_indices(Indices::U32(model.geometry.indices.clone()));
    if !model.cpu_reference {
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(model.influences.iter().map(|w| w.joints).collect()),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_WEIGHT,
            model
                .influences
                .iter()
                .map(|w| w.weights)
                .collect::<Vec<_>>(),
        );
    }
    let mesh_entity = commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.85, 0.65, 0.32),
                perceptual_roughness: 0.7,
                cull_mode: None,
                ..default()
            })),
            bevy::camera::visibility::NoFrustumCulling,
        ))
        .id();
    if !model.cpu_reference {
        commands.entity(mesh_entity).insert(SkinnedMesh {
            inverse_bindposes: inverse_assets.add(model.inverse_bind.clone()),
            joints,
        });
    }
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 18.,
            ..default()
        },
        TextColor(Color::srgb(0.83, 0.90, 0.91)),
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            top: px(22),
            ..default()
        },
        Hud,
    ));
}

fn animate(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut model: ResMut<Model>,
    mut joints: Query<(&Joint, &mut Transform), Without<Orbit>>,
    mut camera: Query<&mut Transform, (With<Orbit>, Without<Joint>)>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    // CPU-reference geometry is baked once for reproducible render comparisons.
    if model.cpu_reference {
        return;
    }
    if keys.just_pressed(KeyCode::Space) {
        model.paused = !model.paused;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        model.bind = !model.bind;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        model.time = 0.;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        model.clip = (model.clip + 1) % model.clips.len();
        model.time = 0.;
        model.bind = false;
    }
    if !model.paused && !model.bind {
        model.time += time.delta_secs();
    }
    let pose = match local_pose(&model) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("animation sampling failed: {e}");
            std::process::exit(1);
        }
    };
    for (joint, mut transform) in &mut joints {
        *transform = match gpu_transform(pose[joint.0]) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("animation joint {}: {e}", joint.0);
                std::process::exit(1);
            }
        };
    }
    let direction =
        keys.pressed(KeyCode::ArrowRight) as i8 - keys.pressed(KeyCode::ArrowLeft) as i8;
    if direction != 0 {
        for mut t in &mut camera {
            t.translation =
                Quat::from_rotation_y(direction as f32 * time.delta_secs()) * t.translation;
            t.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

fn hud(model: Res<Model>, mut text: Query<&mut Text, With<Hud>>) {
    for mut t in &mut text {
        t.0=format!("RW4 {:016X} | {} vertices | {} bones\nClip {}/{} {:08X} | {:.3}s / {:.3}s | {}\nSpace pause   B bind pose   N next clip   R restart   Left/Right orbit   Esc exit\nRustSpore linear/slerp sampling; source texture and morph curves pending",model.instance,model.geometry.positions.len(),model.skeleton.bones.len(),model.clip+1,model.clips.len(),model.clips[model.clip].id,model.time,model.clips[model.clip].duration,if model.bind {"bind"}else if model.paused {"paused"}else{"playing"});
    }
}

fn capture(
    time: Res<Time>,
    mut commands: Commands,
    mut model: ResMut<Model>,
    target: Option<Res<crate::offscreen::OffscreenTarget>>,
    mut readiness: Option<ResMut<crate::offscreen::OffscreenReadiness>>,
    labels: Query<&bevy::text::TextLayoutInfo, With<Hud>>,
) {
    if model.shot.is_none() {
        return;
    }
    model.frames += 1;
    if let Some(error) = readiness.as_ref().and_then(|r| r.failure()) {
        eprintln!("animation GPU pipeline failed: {error}");
        std::process::exit(1);
    }
    if model.captured.is_none() && time.elapsed_secs() > 120. {
        eprintln!("animation renderer did not become ready for capture");
        std::process::exit(1);
    }
    let text_ready = readiness.as_mut().is_none_or(|r| {
        labels.iter().count() == 1 && labels.iter().all(|layout| r.require_text(layout))
    });
    if model.captured.is_none()
        && model.frames >= 40
        && time.elapsed_secs() >= 3.0
        && text_ready
        && readiness.as_ref().is_none_or(|r| r.ready())
    {
        if let Some(path) = &model.shot {
            commands
                .spawn(
                    target
                        .as_ref()
                        .map_or_else(Screenshot::primary_window, |target| target.screenshot()),
                )
                .observe(save_to_disk(path.clone()));
            model.captured = Some(model.frames);
        }
    }
    if let Some(frame) = model.captured {
        if model.frames > frame + 10 && model.shot.as_ref().is_some_and(|p| p.exists()) {
            std::process::exit(0);
        }
        if model.frames > frame + 600 {
            eprintln!("animation capture failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use animation::Bone;
    #[test]
    fn actual_playback_system_pauses_switches_restarts_and_restores_bind_pose() {
        use animation::{Track, Transform as PoseTransform, TransformKey};
        let clip = |offset: f32| KeyframeClip {
            id: 1,
            duration: 1.,
            flags: 0,
            reserved_bytes: 0,
            tracks: vec![Track {
                bone_id: 42,
                kind: 0x101,
                stride: 32,
                values: TrackValues::Transform(vec![
                    TransformKey {
                        time: 0.,
                        value: PoseTransform {
                            translation: [offset, 0., 0.],
                            ..PoseTransform::IDENTITY
                        },
                    },
                    TransformKey {
                        time: 1.,
                        value: PoseTransform {
                            translation: [offset + 1., 0., 0.],
                            ..PoseTransform::IDENTITY
                        },
                    },
                ]),
            }],
        };
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Model {
                geometry: MeshData::default(),
                skeleton: Skeleton {
                    id: 1,
                    flags: 0,
                    bones: vec![Bone {
                        id: 42,
                        flags: 0,
                        parent: None,
                    }],
                },
                inverse_bind: vec![Mat4::IDENTITY],
                bind_local: vec![Mat4::IDENTITY],
                influences: vec![],
                clips: vec![clip(0.), clip(2.)],
                clip: 0,
                time: 0.,
                paused: false,
                bind: false,
                normalization: Mat4::IDENTITY,
                shot: None,
                frames: 0,
                captured: None,
                instance: 1,
                cpu_reference: false,
                camera: Vec3::Z,
            })
            .add_systems(Update, animate);
        let joint = app.world_mut().spawn((Joint(0), Transform::IDENTITY)).id();
        let step = |app: &mut App, seconds: f32, key: Option<KeyCode>| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            if let Some(k) = key {
                keys.press(k);
            }
            drop(keys);
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(seconds));
            app.update();
        };
        step(&mut app, 0.25, None);
        assert!((app.world().get::<Transform>(joint).unwrap().translation.x - 0.25).abs() < 1e-6);
        step(&mut app, 0.25, Some(KeyCode::Space));
        assert_eq!(app.world().resource::<Model>().time, 0.25);
        step(&mut app, 0., Some(KeyCode::KeyN));
        assert_eq!(app.world().resource::<Model>().clip, 1);
        assert_eq!(
            app.world().get::<Transform>(joint).unwrap().translation.x,
            2.
        );
        step(&mut app, 0., Some(KeyCode::KeyB));
        assert_eq!(
            app.world().get::<Transform>(joint).unwrap().translation,
            Vec3::ZERO
        );
        step(&mut app, 0., Some(KeyCode::KeyB));
        step(&mut app, 0.25, Some(KeyCode::Space));
        assert_eq!(
            app.world().get::<Transform>(joint).unwrap().translation.x,
            2.25
        );
        step(&mut app, 0., Some(KeyCode::KeyR));
        assert_eq!(app.world().resource::<Model>().time, 0.);
        assert_eq!(
            app.world().get::<Transform>(joint).unwrap().translation.x,
            2.
        );
    }
    #[test]
    fn hierarchy_composes_child_and_rejects_cycles() {
        let mut s = Skeleton {
            id: 0,
            flags: 0,
            bones: vec![
                Bone {
                    id: 1,
                    flags: 0,
                    parent: None,
                },
                Bone {
                    id: 2,
                    flags: 0,
                    parent: Some(0),
                },
            ],
        };
        let local = vec![
            Mat4::from_translation(Vec3::X),
            Mat4::from_translation(Vec3::Y),
        ];
        assert_eq!(
            globals(&local, &s).unwrap()[1].transform_point3(Vec3::ZERO),
            Vec3::new(1., 1., 0.)
        );
        s.bones[0].parent = Some(1);
        assert!(globals(&local, &s).is_err());
    }
    #[test]
    fn skin_column_conversion_preserves_affine_point_and_inverse() {
        let a = AffineTransform {
            rows: [[0., -2., 0., 3.], [2., 0., 0., 4.], [0., 0., 2., 5.]],
        };
        let p = [1., 2., 3.];
        let v = matrix(a).transform_point3(Vec3::from(p));
        assert_eq!(v.to_array(), a.transform_point(p));
        assert!((matrix(a.inverse().unwrap()) * matrix(a)).abs_diff_eq(Mat4::IDENTITY, 1e-6));
    }
}
