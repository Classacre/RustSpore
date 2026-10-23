//! Interactive editor for the original, versioned creation format.
//! Source links are opaque references; this is not the retail part catalog.
use bevy::{
    input::keyboard::KeyboardInput,
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use spore_core::editor::{
    AuthoredPart, Creation, CreationKind, EditorCommand, EditorDocument, EditorSnapshot,
    GameplayLoadout, PartArchetype as P, PartTransform, PART_CATALOG,
};
use std::path::PathBuf;

#[derive(Resource)]
struct Editor {
    document: EditorDocument,
    selected: u32,
    axis: usize,
    mirror: bool,
    yaw: f32,
    notice: String,
    save: PathBuf,
    shot: Option<PathBuf>,
    elapsed: f32,
    frames: u32,
    captured: Option<u32>,
    demo: Option<f32>,
    demo_step: u8,
    rename: Option<(Option<u32>, String)>,
    dirty: bool,
    pending_game: Option<Vec<std::ffi::OsString>>,
}
#[derive(Component)]
struct PartVisual;
#[derive(Component)]
struct CameraMarker;
#[derive(Component)]
struct Header;
#[derive(Component)]
struct Details;
#[derive(Component)]
struct Notice;
#[derive(Component, Clone, Copy)]
enum Action {
    Add(P),
    Paint(usize),
    Undo,
    Redo,
    Save,
    Load,
    SelectNext,
    Mirror,
    Play,
}

fn kind(text: &str) -> Result<CreationKind, String> {
    match text {
        "cell" => Ok(CreationKind::Cell),
        "creature" => Ok(CreationKind::Creature),
        "building" => Ok(CreationKind::Building),
        "vehicle" => Ok(CreationKind::Vehicle),
        "adventure" => Ok(CreationKind::Adventure),
        _ => Err("editor kind must be cell, creature, building, vehicle or adventure".into()),
    }
}
fn placement(archetype: P) -> PartTransform {
    let translation = match archetype {
        P::HerbivoreMouth | P::CarnivoreMouth | P::OmnivoreMouth => [1.55, 0., 0.],
        P::Eye => [0.9, 0.65, 0.65],
        P::Flagellum | P::Tail => [-1.65, 0., 0.45],
        P::Fin => [0., 0., 1.15],
        P::Leg => [-0.25, -0.9, 0.75],
        P::Arm => [0.8, 0., 0.95],
        P::Claw => [0., -0.65, 0.],
        P::Voice | P::Horn => [0.5, 1., 0.],
        P::Wing => [0., 0.3, 1.2],
        P::Armor => [-0.5, 0.85, 0.],
        P::Wall => [0., 1., 0.],
        P::Roof => [0., 1.1, 0.],
        P::Door => [0., 0., 0.75],
        P::Wheel => [0., -0.3, 0.85],
        P::Thruster => [-1.4, 0., 0.],
        P::Cockpit => [0.5, 0.8, 0.],
        P::Weapon => [1.2, 0.5, 0.],
        P::Platform => [1.5, 0.5, 0.],
        P::Goal | P::Spawn => [0., 0.6, 0.],
        P::Prop => [0., 1., 0.],
        _ => [0.; 3],
    };
    PartTransform {
        translation,
        rotation: if matches!(archetype, P::Flagellum | P::Tail) {
            [0., 0., std::f32::consts::FRAC_PI_2]
        } else {
            [0.; 3]
        },
        ..Default::default()
    }
}
fn initial_document(kind: CreationKind) -> Result<EditorDocument, String> {
    let mut doc = EditorDocument::new(kind, "New creation", 250).map_err(|e| e.to_string())?;
    if matches!(kind, CreationKind::Cell | CreationKind::Creature) {
        let mut mouth = AuthoredPart::new(2, "Mouth", P::HerbivoreMouth, Some(1));
        mouth.transform = placement(mouth.archetype);
        doc.transact(EditorCommand::AddPart(mouth))
            .map_err(|e| e.to_string())?;
        let mut eye = AuthoredPart::new(3, "Eye", P::Eye, Some(1));
        eye.transform = placement(P::Eye);
        doc.transact(EditorCommand::AddMirrored {
            part: eye,
            mirror_id: 4,
            mirror_name: "Other eye".into(),
        })
        .map_err(|e| e.to_string())?;
        let motion = if kind == CreationKind::Cell {
            P::Flagellum
        } else {
            P::Leg
        };
        let mut part = AuthoredPart::new(5, "Movement", motion, Some(1));
        part.transform = placement(motion);
        doc.transact(EditorCommand::AddMirrored {
            part,
            mirror_id: 6,
            mirror_name: "Other movement".into(),
        })
        .map_err(|e| e.to_string())?;
    }
    Ok(doc)
}

pub fn run(args: &[String]) -> Result<(), String> {
    let creation_kind = kind(args.first().map(String::as_str).unwrap_or("creature"))?;
    let mut document = initial_document(creation_kind)?;
    let mut save = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RustSpore/creations")
        .join(format!("{:?}.rsc", creation_kind));
    let mut shot = None;
    let mut demo = None;
    let mut export = None;
    let mut i = usize::from(!args.is_empty());
    while i < args.len() {
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("{} requires a value", args[i]))?;
        match args[i].as_str() {
            "--export" => export = Some(PathBuf::from(value)),
            "--save-file" => save = PathBuf::from(value),
            "--load" => {
                document = EditorDocument::from_snapshot(
                    EditorSnapshot::from_bytes(
                        &crate::cell_game::read_save(std::path::Path::new(value))
                            .map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                if document.creation().kind != creation_kind {
                    return Err("loaded creation has a different editor kind".into());
                }
            }
            "--shot" => {
                let p = PathBuf::from(value);
                if p.exists() {
                    return Err("screenshot path exists".into());
                }
                if let Some(parent) = p.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                shot = Some(p);
            }
            "--demo" => {
                let seconds: f32 = value.parse().map_err(|_| "invalid demo duration")?;
                if !seconds.is_finite() || !(4.0..=600.).contains(&seconds) {
                    return Err("editor demo duration must be 4..600 seconds".into());
                }
                demo = Some(seconds);
            }
            other => return Err(format!("unknown editor option {other}")),
        }
        i += 2;
    }
    if let Some(path) = export {
        let bytes = document.snapshot().to_bytes().map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        println!(
            "EDITOR_EXPORT {} parts={} spent={}",
            path.display(),
            document.creation().parts.len(),
            document.creation().spent()
        );
        return Ok(());
    }
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore | Creation studio".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.035, 0.055, 0.07)))
        .insert_resource(GlobalAmbientLight {
            brightness: 500.,
            ..default()
        })
        .insert_resource(Editor {
            document,
            selected: 1,
            axis: 0,
            mirror: true,
            yaw: 0.6,
            notice: "Choose a part, then shape and paint your creation".into(),
            save,
            shot,
            elapsed: 0.,
            frames: 0,
            captured: None,
            demo,
            demo_step: 0,
            rename: None,
            dirty: true,
            pending_game: None,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (buttons, controls, rebuild, ui, capture, launch_game).chain(),
        )
        .run();
    Ok(())
}

fn button(parent: &mut ChildSpawnerCommands, label: &str, action: Action) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(px(10), px(5)),
                min_height: px(26),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.18, 0.21)),
            action,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.93, 0.90)),
            ));
        });
}
fn setup(
    mut commands: Commands,
    editor: Res<Editor>,
    adapter: Res<RenderAdapterInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!(
        "EDITOR_GPU name={:?} backend={:?}",
        adapter.name, adapter.backend
    );
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(7., 4.5, 8.).looking_at(Vec3::ZERO, Vec3::Y),
        CameraMarker,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 7000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3., 5., 4.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(4.5, 0.18))),
        MeshMaterial3d(materials.add(Color::srgb(0.10, 0.16, 0.18))),
        Transform::from_xyz(0., -2., 0.),
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(22),
                right: px(22),
                top: px(16),
                height: px(76),
                padding: UiRect::all(px(16)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.065, 0.08, 0.96)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 19.,
                    ..default()
                },
                TextColor(Color::srgb(0.67, 0.95, 0.82)),
                Header,
            ));
            for slot in 0..3 {
                button(p, &format!("Paint {}", slot + 1), Action::Paint(slot));
            }
            button(p, "Undo", Action::Undo);
            button(p, "Redo", Action::Redo);
            button(p, "Save", Action::Save);
            button(p, "Load", Action::Load);
            if matches!(
                editor.document.creation().kind,
                CreationKind::Cell | CreationKind::Creature
            ) {
                button(p, "Play [F6]", Action::Play);
            }
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(22),
                top: px(108),
                bottom: px(145),
                width: px(220),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                overflow: Overflow::scroll_y(),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.065, 0.08, 0.94)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("PARTS"),
                TextFont {
                    font_size: 18.,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.91, 0.82)),
            ));
            for part in PART_CATALOG.iter().filter(|p| {
                p.kinds.contains(&editor.document.creation().kind) && !p.allowed_parents.is_empty()
            }) {
                button(
                    p,
                    &format!("{}  {}", part.label, part.cost),
                    Action::Add(part.archetype),
                );
            }
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(22),
                top: px(108),
                width: px(235),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.065, 0.08, 0.94)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 13.,
                    ..default()
                },
                TextColor(Color::srgb(0.80, 0.91, 0.88)),
                Details,
            ));
            button(p, "Next part  [Tab]", Action::SelectNext);
            button(p, "Mirror pairing  [M]", Action::Mirror);
        });
    commands.spawn((Node{position_type:PositionType::Absolute,left:px(22),right:px(22),bottom:px(16),padding:UiRect::all(px(14)),flex_direction:FlexDirection::Column,row_gap:px(8),border_radius:BorderRadius::all(px(10)),..default()},BackgroundColor(Color::srgba(0.025,0.065,0.08,0.96)))).with_children(|p|{
        p.spawn((Text::new(""),TextFont{font_size:14.,..default()},TextColor(Color::srgb(0.96,0.85,0.57)),Notice));
        p.spawn((Text::new("Tab select   X/Y/Z axis   +/- move   Shift +/- rotate   Ctrl +/- scale   Delete remove\nArrows orbit   Ctrl Z/Y undo/redo   F2 rename part   Ctrl F2 name creation   F5/F9 save/load\nOriginal part catalog. Authored creations use the RustSpore .rsc format."),TextFont{font_size:12.,..default()},TextColor(Color::srgb(0.57,0.74,0.75))));
    });
}

fn apply(editor: &mut Editor, command: EditorCommand) {
    match editor.document.transact(command) {
        Ok(changed) => {
            editor.dirty |= changed;
            editor.notice = if changed {
                "Creation updated".into()
            } else {
                "No change".into()
            };
        }
        Err(e) => editor.notice = format!("Change rejected: {e}"),
    }
}
fn action(editor: &mut Editor, action: Action) {
    match action {
        Action::Add(archetype) => {
            let creation = editor.document.creation();
            let definition = archetype.definition();
            let parent = creation
                .part(editor.selected)
                .filter(|p| {
                    definition
                        .allowed_parents
                        .contains(&p.archetype.definition().attachment)
                })
                .or_else(|| {
                    creation.parts.iter().find(|p| {
                        definition
                            .allowed_parents
                            .contains(&p.archetype.definition().attachment)
                    })
                })
                .map(|p| p.id);
            let Some(parent) = parent else {
                editor.notice = "Add a compatible body or limb first".into();
                return;
            };
            let Ok(id) = creation.next_part_id() else {
                editor.notice = "Part limit reached".into();
                return;
            };
            let mut part = AuthoredPart::new(
                id,
                format!("{} {id}", definition.label),
                archetype,
                Some(parent),
            );
            part.transform = placement(archetype);
            let mirrored = editor.mirror && part.transform.translation[2].abs() > 0.001;
            let command = if mirrored {
                let Some(other) = (1..=257).find(|n| *n != id && creation.part(*n).is_none())
                else {
                    return;
                };
                EditorCommand::AddMirrored {
                    part,
                    mirror_id: other,
                    mirror_name: format!("{} {other}", definition.label),
                }
            } else {
                EditorCommand::AddPart(part)
            };
            let revision = editor.document.revision();
            apply(editor, command);
            if editor.document.revision() != revision {
                editor.selected = id;
            }
        }
        Action::Paint(slot) => {
            let mut palette = editor.document.creation().palette;
            let c = palette[slot];
            palette[slot] = [c[1], c[2], if c[0] < 0.2 { 0.85 } else { 0.15 }];
            apply(editor, EditorCommand::SetPalette(palette));
        }
        Action::Undo | Action::Redo => {
            let result = if matches!(action, Action::Undo) {
                editor.document.undo()
            } else {
                editor.document.redo()
            };
            match result {
                Ok(changed) => {
                    editor.dirty |= changed;
                    editor.notice = if changed {
                        "History restored".into()
                    } else {
                        "No history in that direction".into()
                    };
                }
                Err(e) => editor.notice = e.to_string(),
            };
            if editor.document.creation().part(editor.selected).is_none() {
                editor.selected = 1;
            }
        }
        Action::SelectNext => {
            let parts = &editor.document.creation().parts;
            let i = parts
                .iter()
                .position(|p| p.id == editor.selected)
                .unwrap_or(0);
            editor.selected = parts[(i + 1) % parts.len()].id;
            editor.dirty = true;
        }
        Action::Mirror => {
            editor.mirror = !editor.mirror;
            editor.notice = format!(
                "Mirror pairing {}",
                if editor.mirror {
                    "enabled for new parts"
                } else {
                    "disabled"
                }
            );
        }
        Action::Save => {
            let result = editor
                .document
                .snapshot()
                .to_bytes()
                .map_err(|e| e.to_string())
                .and_then(|b| {
                    crate::cell_game::write_save(&editor.save, &b).map_err(|e| e.to_string())
                });
            editor.notice = match result {
                Ok(()) => "Creation saved".into(),
                Err(e) => format!("Save failed: {e}"),
            };
        }
        Action::Load => {
            let result = crate::cell_game::read_save(&editor.save)
                .map_err(|e| e.to_string())
                .and_then(|b| EditorSnapshot::from_bytes(&b).map_err(|e| e.to_string()))
                .and_then(|s| EditorDocument::from_snapshot(s).map_err(|e| e.to_string()));
            match result {
                Ok(document) if document.creation().kind == editor.document.creation().kind => {
                    editor.document = document;
                    editor.selected = 1;
                    editor.dirty = true;
                    editor.notice = "Creation loaded".into();
                }
                Ok(_) => editor.notice = "Load rejected: different creation kind".into(),
                Err(e) => editor.notice = format!("Load failed: {e}"),
            };
        }
        Action::Play => {
            let creation = editor.document.creation().clone();
            let result = match creation.kind {
                CreationKind::Cell => {
                    spore_core::cell::CellWorld::new(14, spore_core::cell::Diet::Herbivore)
                        .apply_creation(creation)
                }
                CreationKind::Creature => {
                    let checked = spore_core::creature_game::CreatureWorld::new(
                        14,
                        spore_core::cell::Diet::Herbivore,
                    )
                    .apply_creation(creation);
                    if let Err(e) = checked {
                        editor.notice = format!("Cannot play this creation: {e}");
                        return;
                    }
                    Ok(())
                }
                _ => {
                    editor.notice =
                        "Gameplay integration for this creation kind is still in progress".into();
                    return;
                }
            };
            if let Err(e) = result {
                editor.notice = format!("Cannot play this creation: {e}");
                return;
            }
            let saved = editor
                .document
                .snapshot()
                .to_bytes()
                .map_err(|e| e.to_string())
                .and_then(|b| {
                    crate::cell_game::write_save(&editor.save, &b).map_err(|e| e.to_string())
                });
            match saved {
                Ok(()) => {
                    editor.pending_game = Some(vec![
                        if editor.document.creation().kind == CreationKind::Cell {
                            "--cell"
                        } else {
                            "--creature-game"
                        }
                        .into(),
                        "--creation".into(),
                        editor.save.clone().into_os_string(),
                    ]);
                    editor.notice = "Creation saved. Starting game...".into();
                }
                Err(e) => editor.notice = format!("Cannot save creation: {e}"),
            }
        }
    }
}
fn buttons(
    mut editor: ResMut<Editor>,
    mut buttons: Query<(&Interaction, &Action, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, button, mut color) in &mut buttons {
        color.0 = match interaction {
            Interaction::Pressed => Color::srgb(0.16, 0.42, 0.37),
            Interaction::Hovered => Color::srgb(0.15, 0.29, 0.31),
            Interaction::None => Color::srgb(0.10, 0.18, 0.21),
        };
        if *interaction == Interaction::Pressed {
            action(&mut editor, *button);
        }
    }
}
fn controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut input: MessageReader<KeyboardInput>,
    mut editor: ResMut<Editor>,
    mut camera: Query<&mut Transform, With<CameraMarker>>,
) {
    editor.elapsed += time.delta_secs();
    if let Some((target, mut draft)) = editor.rename.take() {
        for event in input
            .read()
            .filter(|e| e.state == bevy::input::ButtonState::Pressed)
        {
            if let Some(text) = &event.text {
                for ch in text.chars().filter(|c| !c.is_control()) {
                    if draft.len() + ch.len_utf8() <= 64 {
                        draft.push(ch);
                    }
                }
            }
        }
        if keys.just_pressed(KeyCode::Backspace) {
            draft.pop();
        }
        if keys.just_pressed(KeyCode::Enter) {
            apply(
                &mut editor,
                match target {
                    Some(id) => EditorCommand::RenamePart { id, name: draft },
                    None => EditorCommand::RenameCreation(draft),
                },
            );
        } else if !keys.just_pressed(KeyCode::Escape) {
            editor.notice = format!("Name: {draft}_  [Enter confirms, Esc cancels]");
            editor.rename = Some((target, draft));
        }
        return;
    }
    input.clear();
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if keys.just_pressed(KeyCode::F2) {
        let target = if ctrl { None } else { Some(editor.selected) };
        let value = target
            .and_then(|id| editor.document.creation().part(id).map(|p| p.name.clone()))
            .unwrap_or_else(|| editor.document.creation().name.clone());
        editor.rename = Some((target, value));
        return;
    }
    for (key, a) in [
        (KeyCode::Tab, Action::SelectNext),
        (KeyCode::KeyM, Action::Mirror),
        (KeyCode::F5, Action::Save),
        (KeyCode::F9, Action::Load),
        (KeyCode::F6, Action::Play),
    ] {
        if keys.just_pressed(key) {
            action(&mut editor, a);
        }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        action(&mut editor, Action::Undo);
    } else if ctrl && keys.just_pressed(KeyCode::KeyY) {
        action(&mut editor, Action::Redo);
    }
    if !ctrl {
        for (i, key) in [KeyCode::KeyX, KeyCode::KeyY, KeyCode::KeyZ]
            .iter()
            .enumerate()
        {
            if keys.just_pressed(*key) {
                editor.axis = i;
            }
        }
    }
    if keys.just_pressed(KeyCode::Delete) {
        let id = editor.selected;
        apply(&mut editor, EditorCommand::RemoveSubtree(id));
        if editor.document.creation().part(id).is_none() {
            editor.selected = 1;
        }
    }
    let direction = (keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd))
        as i8
        - (keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract)) as i8;
    if direction != 0 {
        if let Some(part) = editor.document.creation().part(editor.selected) {
            let id = part.id;
            let mut transform = part.transform;
            let axis = editor.axis;
            if ctrl {
                transform.scale[axis] += direction as f32 * 0.1;
            } else if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                transform.rotation[axis] += direction as f32 * 0.15;
            } else {
                transform.translation[axis] += direction as f32 * 0.15;
            }
            apply(&mut editor, EditorCommand::SetTransform { id, transform });
        }
    }
    let orbit = keys.pressed(KeyCode::ArrowRight) as i8 - keys.pressed(KeyCode::ArrowLeft) as i8;
    editor.yaw += orbit as f32 * time.delta_secs();
    let height = if keys.pressed(KeyCode::ArrowDown) {
        2.5
    } else if keys.pressed(KeyCode::ArrowUp) {
        7.0
    } else {
        4.5
    };
    for mut t in &mut camera {
        t.translation = Vec3::new(editor.yaw.sin() * 10., height, editor.yaw.cos() * 10.);
        t.look_at(Vec3::ZERO, Vec3::Y);
    }
    if editor.demo.is_some()
        && editor.elapsed >= 1. + editor.demo_step as f32 * 0.7
        && editor.demo_step < 4
    {
        match editor.demo_step {
            0 => action(&mut editor, Action::Paint(0)),
            1 => {
                let p = editor
                    .document
                    .creation()
                    .parts
                    .iter()
                    .find(|p| p.parent.is_some())
                    .map(|p| p.id);
                if let Some(id) = p {
                    editor.selected = id;
                    if let Some(part) = editor.document.creation().part(id) {
                        let mut transform = part.transform;
                        transform.translation[1] += 0.35;
                        apply(&mut editor, EditorCommand::SetTransform { id, transform });
                    }
                }
            }
            2 => action(&mut editor, Action::Undo),
            _ => action(&mut editor, Action::Redo),
        }
        editor.demo_step += 1;
    }
}

fn shape(archetype: P) -> (Mesh, Vec3) {
    match archetype {
        P::Body => (Sphere::new(1.).mesh().uv(24, 16), Vec3::new(1.65, 0.9, 1.)),
        P::Eye => (Sphere::new(1.).mesh().uv(16, 12), Vec3::splat(0.22)),
        P::HerbivoreMouth | P::CarnivoreMouth | P::OmnivoreMouth => {
            (Sphere::new(1.).mesh().uv(16, 12), Vec3::new(0.4, 0.3, 0.45))
        }
        P::Flagellum | P::Tail => (Capsule3d::new(0.1, 1.2).mesh().build(), Vec3::ONE),
        P::Leg | P::Arm => (Capsule3d::new(0.18, 1.).mesh().build(), Vec3::ONE),
        P::Claw | P::Horn | P::Voice | P::Goal => (Cone::new(0.25, 0.8).mesh().build(), Vec3::ONE),
        P::Fin | P::Wing => (
            Sphere::new(1.).mesh().uv(16, 12),
            Vec3::new(0.75, 0.12, 0.55),
        ),
        P::Armor => (
            Sphere::new(1.).mesh().uv(16, 12),
            Vec3::new(0.7, 0.18, 0.65),
        ),
        P::Wheel => (Cylinder::new(0.4, 0.2).mesh().build(), Vec3::ONE),
        P::Terrain => (Cuboid::new(5., 0.25, 4.).mesh().build(), Vec3::ONE),
        P::Foundation | P::Chassis => (Cuboid::new(2.8, 0.5, 1.8).mesh().build(), Vec3::ONE),
        P::Wall => (Cuboid::new(2., 1.8, 0.2).mesh().build(), Vec3::ONE),
        P::Roof => (Cone::new(1.6, 1.2).mesh().build(), Vec3::ONE),
        P::Door => (Cuboid::new(0.6, 1.2, 0.1).mesh().build(), Vec3::ONE),
        P::Platform => (Cuboid::new(1.4, 0.25, 1.4).mesh().build(), Vec3::ONE),
        _ => (Sphere::new(1.).mesh().uv(16, 12), Vec3::splat(0.5)),
    }
}

/// The editor and gameplay use the same authored graph and palette. This is
/// original geometry; opaque source resource references are not substituted.
pub(crate) fn creation_mesh(creation: &Creation, flat: bool) -> Result<Mesh, String> {
    let matrices = creation.world_matrices().map_err(|e| e.to_string())?;
    let mut combined: Option<Mesh> = None;
    for (part, rows) in creation.parts.iter().zip(matrices) {
        let matrix = Mat4::from_cols_array(&[
            rows[0][0], rows[1][0], rows[2][0], rows[3][0], rows[0][1], rows[1][1], rows[2][1],
            rows[3][1], rows[0][2], rows[1][2], rows[2][2], rows[3][2], rows[0][3], rows[1][3],
            rows[2][3], rows[3][3],
        ]);
        let (mut mesh, scale) = shape(part.archetype);
        let matrix = matrix * Mat4::from_scale(scale);
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(values)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for p in values {
                *p = matrix.transform_point3(Vec3::from(*p)).to_array();
            }
        }
        let normal = Mat3::from_mat4(matrix).inverse().transpose();
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(values)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            for n in values {
                *n = (normal * Vec3::from(*n)).normalize_or_zero().to_array();
            }
        }
        let c = if part.archetype == P::Eye {
            [0.96, 0.97, 0.88]
        } else {
            creation.palette[if part.parent.is_none() {
                0
            } else if matches!(part.archetype, P::Armor | P::Horn | P::Claw) {
                2
            } else {
                1
            }]
        };
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![Color::srgb(c[0], c[1], c[2]).to_linear().to_f32_array(); mesh.count_vertices()],
        );
        if let Some(all) = &mut combined {
            all.merge(&mesh).map_err(|e| e.to_string())?;
        } else {
            combined = Some(mesh);
        }
    }
    let mut mesh = combined.ok_or("empty authored creation")?;
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    else {
        return Err("creation has no positions".into());
    };
    let min = positions
        .iter()
        .fold(Vec3::splat(f32::INFINITY), |a, p| a.min(Vec3::from(*p)));
    let max = positions
        .iter()
        .fold(Vec3::splat(f32::NEG_INFINITY), |a, p| a.max(Vec3::from(*p)));
    let center = (min + max) * 0.5;
    let mut radius = positions
        .iter()
        .map(|p| Vec2::new(p[0] - center.x, p[2] - center.z).length())
        .fold(0., f32::max);
    if !flat {
        radius = radius.max((max.y - min.y) * 0.5);
    }
    radius = radius.max(0.01);
    for p in positions {
        let v = (Vec3::from(*p)
            - Vec3::new(center.x, if flat { center.y } else { min.y }, center.z))
            / radius;
        *p = if flat {
            [v.x, v.z, v.y * 0.01]
        } else {
            v.to_array()
        };
    }
    if flat {
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            for n in normals {
                *n = [0., 0., 1.];
            }
        }
    }
    Ok(mesh)
}
fn rebuild(
    mut commands: Commands,
    mut editor: ResMut<Editor>,
    old: Query<Entity, With<PartVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !editor.dirty {
        return;
    }
    for e in &old {
        commands.entity(e).despawn();
    }
    let creation = editor.document.creation();
    let matrices = creation
        .world_matrices()
        .expect("editor transaction validated graph");
    for (part, rows) in creation.parts.iter().zip(matrices) {
        let matrix = Mat4::from_cols_array(&[
            rows[0][0], rows[1][0], rows[2][0], rows[3][0], rows[0][1], rows[1][1], rows[2][1],
            rows[3][1], rows[0][2], rows[1][2], rows[2][2], rows[3][2], rows[0][3], rows[1][3],
            rows[2][3], rows[3][3],
        ]);
        let (mut mesh, scale) = shape(part.archetype);
        let matrix = matrix * Mat4::from_scale(scale);
        // Preserve the full affine, including shear from rotated children of
        // nonuniformly scaled parents, when baking the displayed geometry.
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(values)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for p in values {
                *p = matrix.transform_point3(Vec3::from(*p)).to_array();
            }
        }
        let normal_matrix = Mat3::from_mat4(matrix).inverse().transpose();
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(values)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            for n in values {
                *n = (normal_matrix * Vec3::from(*n))
                    .normalize_or_zero()
                    .to_array();
            }
        }
        let colour = if part.archetype == P::Eye {
            [0.96, 0.97, 0.88]
        } else {
            creation.palette[if part.parent.is_none() {
                0
            } else if matches!(part.archetype, P::Armor | P::Horn | P::Claw) {
                2
            } else {
                1
            }]
        };
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(colour[0], colour[1], colour[2]),
                emissive: if part.id == editor.selected {
                    LinearRgba::new(0.06, 0.17, 0.14, 1.)
                } else {
                    LinearRgba::BLACK
                },
                perceptual_roughness: 0.65,
                ..default()
            })),
            PartVisual,
        ));
    }
    editor.dirty = false;
}
fn ui(
    editor: Res<Editor>,
    mut texts: ParamSet<(
        Query<&mut Text, With<Header>>,
        Query<&mut Text, With<Details>>,
        Query<&mut Text, With<Notice>>,
    )>,
) {
    let c = editor.document.creation();
    for mut t in &mut texts.p0() {
        t.0 = format!(
            "CREATION STUDIO | {:?}\n{}   {}/{} DNA",
            c.kind,
            c.name.chars().take(25).collect::<String>(),
            c.spent(),
            c.budget
        );
    }
    for mut t in &mut texts.p1() {
        t.0 = if let Some(p) = c.part(editor.selected) {
            let stats=match c.gameplay_loadout(){
                Ok(GameplayLoadout::Cell(s))=>format!("{:?}\nHealth {:.0} | Swim {:.1}\nBite {:.0} | Boost {:.1}\nDefense {:.0}%",s.diet,s.health,s.speed,s.bite_damage,s.boost_capacity,s.defense*100.),
                Ok(GameplayLoadout::Creature(s))=>format!("{:?}\nHealth {:.0} | Speed {:.1}\nAttack {:.0} | Social {:.1}\nStamina {:.1} | Defense {:.0}%",s.diet,s.health,s.speed,s.attack_damage,s.social_power,s.stamina,s.defense*100.),
                Err(_)if matches!(c.kind,CreationKind::Cell|CreationKind::Creature)=>"Add a mouth for gameplay".into(),
                Err(_)=>"Saved creation\nGameplay integration pending".into(),
            };
            format!("SELECTED: {}\n{}\n\nMove {:.2}, {:.2}, {:.2}\nTurn {:.2}, {:.2}, {:.2}\nSize {:.2}, {:.2}, {:.2}\n\nAxis {} | Mirror {}\nParts {} | Undo {}\n\n{}",p.name,p.archetype.definition().label,p.transform.translation[0],p.transform.translation[1],p.transform.translation[2],p.transform.rotation[0],p.transform.rotation[1],p.transform.rotation[2],p.transform.scale[0],p.transform.scale[1],p.transform.scale[2],["X","Y","Z"][editor.axis],if editor.mirror{"on"}else{"off"},c.parts.len(),editor.document.undo_len(),stats)
        } else {
            String::new()
        };
    }
    for mut t in &mut texts.p2() {
        t.0 = editor.notice.clone();
    }
}
fn capture(time: Res<Time>, mut commands: Commands, mut editor: ResMut<Editor>) {
    editor.frames += 1;
    if editor.captured.is_none()
        && time.elapsed_secs() >= editor.demo.unwrap_or(3.)
        && editor.shot.is_some()
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(editor.shot.as_ref().unwrap().clone()));
        editor.captured = Some(editor.frames);
        println!(
            "EDITOR_RESULT revision={} parts={} spent={} valid={}",
            editor.document.revision(),
            editor.document.creation().parts.len(),
            editor.document.creation().spent(),
            editor.document.creation().validate().is_ok()
        );
    }
    if let Some(frame) = editor.captured {
        if editor.frames > frame + 10 && editor.shot.as_ref().is_some_and(|p| p.exists()) {
            std::process::exit(0);
        }
        if editor.frames > frame + 600 {
            eprintln!("editor capture failed");
            std::process::exit(1);
        }
    }
}

fn launch_game(mut editor: ResMut<Editor>, mut exit: MessageWriter<AppExit>) {
    if let Some(arguments) = editor.pending_game.take() {
        // A fresh process gives the interactive game its own winit event loop.
        // The current editor exits only after the saved creation's game starts.
        let child = std::env::current_exe().and_then(|exe| {
            std::process::Command::new(exe)
                .arg("--experimental")
                .args(arguments)
                .stdin(std::process::Stdio::null())
                .spawn()
        });
        match child {
            Ok(_) => {
                exit.write(AppExit::Success);
            }
            Err(e) => editor.notice = format!("Game could not start: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_button_system_edits_undoes_saves_loads_and_prepares_playable_creation() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder =
            std::env::temp_dir().join(format!("rustspore-studio-{}-{nonce}", std::process::id()));
        let path = folder.join("created.rsc");
        let mut app = App::new();
        app.insert_resource(Editor {
            document: initial_document(CreationKind::Creature).unwrap(),
            selected: 1,
            axis: 0,
            mirror: true,
            yaw: 0.,
            notice: String::new(),
            save: path.clone(),
            shot: None,
            elapsed: 0.,
            frames: 0,
            captured: None,
            demo: None,
            demo_step: 0,
            rename: None,
            dirty: true,
            pending_game: None,
        })
        .add_systems(Update, buttons);
        let entity = app
            .world_mut()
            .spawn((
                Interaction::None,
                Action::Add(P::Armor),
                BackgroundColor(Color::WHITE),
            ))
            .id();
        let click = |app: &mut App, a: Action| {
            app.world_mut()
                .entity_mut(entity)
                .insert((Interaction::None, a));
            app.update();
            app.world_mut()
                .entity_mut(entity)
                .insert(Interaction::Pressed);
            app.update();
        };
        let original = app.world().resource::<Editor>().document.creation().clone();
        click(&mut app, Action::Add(P::Armor));
        let authored = app.world().resource::<Editor>().document.creation().clone();
        assert_eq!(authored.parts.len(), original.parts.len() + 1);
        assert!(authored.spent() > original.spent());
        click(&mut app, Action::Undo);
        assert_eq!(
            *app.world().resource::<Editor>().document.creation(),
            original
        );
        click(&mut app, Action::Redo);
        assert_eq!(
            *app.world().resource::<Editor>().document.creation(),
            authored
        );
        click(&mut app, Action::Save);
        click(&mut app, Action::Paint(0));
        assert_ne!(
            *app.world().resource::<Editor>().document.creation(),
            authored
        );
        click(&mut app, Action::Load);
        assert_eq!(
            *app.world().resource::<Editor>().document.creation(),
            authored
        );
        click(&mut app, Action::Play);
        let editor = app.world().resource::<Editor>();
        let args = editor
            .pending_game
            .as_ref()
            .expect("valid creation prepares a game");
        assert_eq!(args[0], "--creature-game");
        assert_eq!(args[2], path.as_os_str());
        let saved = EditorSnapshot::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
        let mut world =
            spore_core::creature_game::CreatureWorld::new(14, spore_core::cell::Diet::Carnivore);
        world.apply_creation(saved.creation.clone()).unwrap();
        assert_eq!(world.creation, Some(authored.clone()));
        assert!(world.player.max_health > 100.);
        assert!(
            creation_mesh(&saved.creation, false)
                .unwrap()
                .count_vertices()
                > 100
        );
        assert!(
            creation_mesh(
                &initial_document(CreationKind::Cell).unwrap().creation(),
                true
            )
            .unwrap()
            .count_vertices()
                > 100
        );
        std::fs::write(&path, b"broken creation").unwrap();
        click(&mut app, Action::Load);
        assert_eq!(
            *app.world().resource::<Editor>().document.creation(),
            authored
        );
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
}
