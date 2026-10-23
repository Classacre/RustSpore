//! Original RTS presentation for the deterministic Tribe baseline.
//! Procedural art and prototype rules are separate from retail-content fidelity.
use bevy::{
    camera::ScalingMode,
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use spore_core::tribe::{
    BuildingKind, SocialPerformance, ToolKind, TribeCommand, TribeError, TribeEvent, TribeOrder,
    TribeSnapshot, TribeState, TribeVec2, TribeWorld,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoPhase {
    Start,
    Building,
    Equipping,
    Gathering,
    Diplomacy,
    Done,
}
#[derive(Resource)]
struct Game {
    world: TribeWorld,
    paused: bool,
    elapsed: f64,
    notice: String,
    target: u32,
    placement: Option<BuildingKind>,
    tool: ToolKind,
    cursor: Option<TribeVec2>,
    save_path: PathBuf,
    shot: Option<PathBuf>,
    demo_seconds: Option<f64>,
    demo_phase: DemoPhase,
    demo_delivered: f32,
    frames: u64,
    capture_frame: Option<u64>,
    rebuild: bool,
}
impl Game {
    fn new(seed: u64, save_path: PathBuf) -> Self {
        Self {
            world: TribeWorld::new(seed),
            paused: false,
            elapsed: 0.,
            notice: "Select your members, build a hut, and gather food.".into(),
            target: 1,
            placement: None,
            tool: ToolKind::Instrument,
            cursor: None,
            save_path,
            shot: None,
            demo_seconds: None,
            demo_phase: DemoPhase::Start,
            demo_delivered: 0.,
            frames: 0,
            capture_frame: None,
            rebuild: true,
        }
    }
    fn command(&mut self, command: TribeCommand) {
        match self.world.issue_selected(command) {
            Ok(events) => {
                self.notice = "Order accepted.".into();
                for event in events {
                    if let TribeEvent::ConstructionStarted { kind, cost, .. } = event {
                        self.notice = format!("Building {kind:?}: {cost:.0} food spent.");
                    }
                }
            }
            Err(e) => self.notice = error_notice(e),
        }
    }
    fn select_all(&mut self) {
        let ids: Vec<_> = self
            .world
            .members
            .iter()
            .filter(|m| m.tribe_id == 0 && m.alive())
            .map(|m| m.id)
            .collect();
        let _ = self.world.select(&ids);
    }
    fn gather(&mut self) {
        let home = self.world.tribes[0].home;
        if let Some(id) = self
            .world
            .food_sites
            .iter()
            .filter(|s| s.amount > 0.)
            .min_by(|a, b| {
                a.position
                    .distance(home)
                    .total_cmp(&b.position.distance(home))
            })
            .map(|s| s.id)
        {
            self.command(TribeCommand::GatherFood(id));
        } else {
            self.notice = "All food sites are depleted.".into();
        }
    }
    fn socialize(&mut self, performance: SocialPerformance) {
        self.command(TribeCommand::PerformSocial {
            tribe: self.target,
            performance,
        });
    }
    fn heal(&mut self) {
        if let Some(id) = self
            .world
            .members
            .iter()
            .filter(|m| m.tribe_id == 0 && m.alive() && m.health < m.max_health)
            .min_by(|a, b| a.health.total_cmp(&b.health))
            .map(|m| m.id)
        {
            self.command(TribeCommand::Heal(id));
        } else {
            self.notice = "Everyone in your tribe is healthy.".into();
        }
    }
    fn auto_place(&mut self) {
        if let Some(kind) = self.placement {
            let home = self.world.tribes[0].home;
            for offset in [
                TribeVec2::new(4., 4.),
                TribeVec2::new(4., -4.),
                TribeVec2::new(-4., 4.),
                TribeVec2::new(-4., -4.),
                TribeVec2::new(8., 0.),
                TribeVec2::new(0., 8.),
            ] {
                let position = home.add(offset);
                if placement_valid(&self.world, position, kind) {
                    self.command(TribeCommand::Build { kind, position });
                    self.placement = None;
                    return;
                }
            }
            self.notice = "No clear building site near home, or insufficient food.".into();
        }
    }
}
fn error_notice(e: TribeError) -> String {
    match e {
        TribeError::NoSelection => "Select your tribe members first.",
        TribeError::NotEnoughFood => "Not enough stored food. Gather and deliver food first.",
        TribeError::MissingBuilding => "Build and complete the matching hut first.",
        TribeError::MissingTool => {
            "Each performer needs an instrument. Equip at an Instrument Hut."
        }
        TribeError::OccupiedLocation => "That building site is blocked. Choose open ground.",
        TribeError::NotPlaying => "This stage has ended. Press R to restart.",
        _ => "That order has no valid target.",
    }
    .into()
}
#[derive(Clone, Copy, Component)]
enum Action {
    SelectAll,
    Gather,
    Build(BuildingKind),
    Equip(ToolKind),
    Social(SocialPerformance),
    Attack,
    Heal,
}
#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct Ground;
#[derive(Component)]
struct Dynamic;
#[derive(Component)]
struct Status;
#[derive(Component)]
struct TargetText;
#[derive(Component)]
struct SelectionText;
#[derive(Component)]
struct Notice;
#[derive(Component)]
struct Banner;
#[derive(Component)]
struct Ghost;
#[derive(Component)]
struct HealthBar(u32);
#[derive(Resource, Default)]
struct Objects(HashMap<(u8, u32), Entity>);
#[derive(Resource)]
struct Art {
    sphere: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    cube: Handle<Mesh>,
    cone: Handle<Mesh>,
    ring: Handle<Mesh>,
    teams: Vec<Handle<StandardMaterial>>,
    dark: Handle<StandardMaterial>,
    roof: Handle<StandardMaterial>,
    food: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
    white: Handle<StandardMaterial>,
    valid: Handle<StandardMaterial>,
    invalid: Handle<StandardMaterial>,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let mut seed = 14;
    let mut shot = None;
    let mut demo = None;
    let mut save_path = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RustSpore")
        .join("saves")
        .join("tribe.rsp");
    let mut i = 0;
    while i < args.len() {
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("{} requires a value", args[i]))?;
        match args[i].as_str() {
            "--seed" => seed = value.parse().map_err(|_| "invalid seed")?,
            "--shot" => shot = Some(PathBuf::from(value)),
            "--save-file" => save_path = PathBuf::from(value),
            "--demo" => {
                let seconds: f64 = value.parse().map_err(|_| "invalid demo duration")?;
                if !seconds.is_finite() || seconds <= 0. || seconds > 600. {
                    return Err("demo duration must be in (0,600] seconds".into());
                }
                demo = Some(seconds);
            }
            other => return Err(format!("unknown Tribe option {other}")),
        }
        i += 2;
    }
    if shot.as_ref().is_some_and(|p| p.exists()) {
        return Err("screenshot already exists; choose a new path".into());
    }
    if let Some(parent) = shot
        .as_ref()
        .and_then(|p| p.parent())
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut game = Game::new(seed, save_path);
    game.shot = shot;
    game.demo_seconds = demo;
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore | Tribe".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.48, 0.61, 0.63)))
        .insert_resource(game)
        .init_resource::<Objects>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                ui_actions,
                mouse_orders,
                drive,
                sync_scene,
                update_hud,
                capture,
            )
                .chain(),
        )
        .run();
    Ok(())
}
fn location(p: TribeVec2, y: f32) -> Vec3 {
    Vec3::new(p.x, y, p.y)
}
fn material(materials: &mut Assets<StandardMaterial>, color: Color) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.9,
        ..default()
    })
}
fn setup(
    mut commands: Commands,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    adapter: Res<RenderAdapterInfo>,
) {
    println!(
        "TRIBE_GPU name={:?} backend={:?} type={:?}",
        adapter.name, adapter.backend, adapter.device_type
    );
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 76.,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0., 68., 57.).looking_at(Vec3::ZERO, Vec3::Y),
        AmbientLight {
            brightness: 450.,
            ..default()
        },
        MainCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.6, 0.)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1., 1.))),
        MeshMaterial3d(material(&mut materials, Color::srgb(0.35, 0.45, 0.23))),
        Transform::from_scale(Vec3::new(
            game.world.config.half_extents.x * 2.,
            1.,
            game.world.config.half_extents.y * 2.,
        )),
        Ground,
    ));
    let art = Art {
        sphere: meshes.add(Sphere::new(1.)),
        cylinder: meshes.add(Cylinder::new(1., 1.)),
        cube: meshes.add(Cuboid::new(1., 1., 1.)),
        cone: meshes.add(Cone::new(1., 1.)),
        ring: meshes.add(Annulus::new(0.83, 1.)),
        teams: [
            Color::srgb(0.16, 0.75, 0.71),
            Color::srgb(0.84, 0.39, 0.25),
            Color::srgb(0.55, 0.38, 0.78),
            Color::srgb(0.85, 0.71, 0.24),
            Color::srgb(0.3, 0.54, 0.83),
            Color::srgb(0.73, 0.47, 0.57),
        ]
        .into_iter()
        .map(|c| material(&mut materials, c))
        .collect(),
        dark: material(&mut materials, Color::srgb(0.19, 0.15, 0.1)),
        roof: material(&mut materials, Color::srgb(0.63, 0.39, 0.16)),
        food: material(&mut materials, Color::srgb(0.82, 0.9, 0.24)),
        selected: material(&mut materials, Color::srgb(1., 0.91, 0.44)),
        white: material(&mut materials, Color::srgb(0.92, 0.88, 0.68)),
        valid: material(&mut materials, Color::srgba(0.3, 1., 0.5, 0.7)),
        invalid: material(&mut materials, Color::srgb(1., 0.2, 0.15)),
    };
    commands.spawn((
        Mesh3d(art.cylinder.clone()),
        MeshMaterial3d(art.valid.clone()),
        Transform::default(),
        Visibility::Hidden,
        Ghost,
    ));
    commands.insert_resource(art);
    setup_ui(&mut commands);
}
fn label(parent: &mut ChildSpawnerCommands, text: &str, size: f32, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    ));
}
fn button(parent: &mut ChildSpawnerCommands, text: &str, action: Action) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: percent(100),
                height: px(27),
                padding: UiRect::axes(px(9), px(4)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.17, 0.25, 0.23)),
        ))
        .with_children(|p| label(p, text, 12., Color::srgb(0.89, 0.94, 0.86)));
}
fn setup_ui(commands: &mut Commands) {
    let panel = Color::srgba(0.035, 0.075, 0.075, 0.96);
    let muted = Color::srgb(0.68, 0.79, 0.73);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(18),
                right: px(18),
                top: px(16),
                height: px(70),
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(panel),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 17.,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.95, 0.87)),
                Status,
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(18),
                top: px(94),
                width: px(244),
                padding: UiRect::all(px(12)),
                row_gap: px(5),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(panel),
        ))
        .with_children(|p| {
            label(p, "TRIBE COMMANDS", 15., Color::srgb(0.98, 0.82, 0.43));
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 12.,
                    ..default()
                },
                TextColor(muted),
                SelectionText,
            ));
            button(p, "1  Select all members", Action::SelectAll);
            button(p, "G  Gather nearest food", Action::Gather);
            label(p, "BUILD / PLACE WITH LEFT CLICK", 11., muted);
            for (text, kind) in [
                ("2  Gathering Hut  |  10 food", BuildingKind::GatheringHut),
                ("3  Weapon Hut     |  10 food", BuildingKind::WeaponHut),
                ("4  Instrument Hut |  10 food", BuildingKind::InstrumentHut),
                ("5  Healing Hut    |  12 food", BuildingKind::HealingHut),
            ] {
                button(p, text, Action::Build(kind));
            }
            label(p, "TOOLS: 3 FOOD PER MEMBER", 11., muted);
            button(
                p,
                "Equip gathering tools",
                Action::Equip(ToolKind::Gathering),
            );
            button(p, "Equip weapons", Action::Equip(ToolKind::Weapon));
            button(
                p,
                "T  Equip instruments",
                Action::Equip(ToolKind::Instrument),
            );
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 12.,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.81, 0.48)),
                TargetText,
            ));
            button(p, "Q  Sing", Action::Social(SocialPerformance::Sing));
            button(p, "W  Dance", Action::Social(SocialPerformance::Dance));
            button(p, "E  Play", Action::Social(SocialPerformance::Play));
            button(p, "A  Attack targeted tribe", Action::Attack);
            button(p, "H  Heal injured member", Action::Heal);
        });
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 15.,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.88, 0.61)),
        Node {
            position_type: PositionType::Absolute,
            left: px(282),
            right: px(22),
            bottom: px(76),
            ..default()
        },
        Notice,
    ));
    commands.spawn((Node{position_type:PositionType::Absolute,left:px(18),right:px(18),bottom:px(16),height:px(48),padding:UiRect::all(px(9)),..default()},BackgroundColor(panel))).with_children(|p|label(p,"Click / Shift-click select   Right-click move / food / attack   Tab target   C auto-place hut   Arrows move\nP pause   R restart   F5 save   F9 load   Esc exit   |   Original prototype rules and procedural art",12.,muted));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 28.,
            ..default()
        },
        TextColor(Color::srgb(1., 0.91, 0.62)),
        Node {
            position_type: PositionType::Absolute,
            left: percent(35),
            top: percent(38),
            max_width: px(680),
            ..default()
        },
        Banner,
    ));
}

fn apply_action(game: &mut Game, action: Action) {
    match action {
        Action::SelectAll => {
            game.select_all();
            game.rebuild = true;
        }
        Action::Gather => game.gather(),
        Action::Build(kind) => {
            game.placement = Some(kind);
            game.notice = format!("Place {kind:?} on open ground, or press C to place near home.");
        }
        Action::Equip(tool) => {
            game.tool = tool;
            game.command(TribeCommand::EquipTool(tool));
        }
        Action::Social(p) => game.socialize(p),
        Action::Attack => {
            let target = game.target;
            game.command(TribeCommand::Attack(target));
        }
        Action::Heal => game.heal(),
    }
}
fn ui_actions(
    mut buttons: Query<
        (&Interaction, &Action, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut game: ResMut<Game>,
) {
    for (interaction, action, mut color) in &mut buttons {
        match interaction {
            Interaction::Pressed => {
                apply_action(&mut game, *action);
                color.0 = Color::srgb(0.29, 0.45, 0.36);
            }
            Interaction::Hovered => color.0 = Color::srgb(0.23, 0.35, 0.29),
            Interaction::None => color.0 = Color::srgb(0.17, 0.25, 0.23),
        }
    }
}
fn placement_valid(world: &TribeWorld, p: TribeVec2, kind: BuildingKind) -> bool {
    p.x.is_finite()
        && p.y.is_finite()
        && p.x.abs() < world.config.half_extents.x - 0.55
        && p.y.abs() < world.config.half_extents.y - 0.55
        && !world.selected.is_empty()
        && world.tribes[0].food >= kind.cost()
        && world
            .buildings
            .iter()
            .all(|b| b.position.distance(p) >= 3.2)
        && world.tribes.iter().all(|t| t.home.distance(p) >= 3.2)
}
fn handle_click(game: &mut Game, p: TribeVec2, left: bool, shift: bool) {
    if left {
        if let Some(kind) = game.placement.take() {
            game.command(TribeCommand::Build { kind, position: p });
            return;
        }
        if let Some(id) = game
            .world
            .members
            .iter()
            .filter(|m| m.tribe_id == 0 && m.alive() && m.position.distance(p) < 1.8)
            .min_by(|a, b| a.position.distance(p).total_cmp(&b.position.distance(p)))
            .map(|m| m.id)
        {
            let mut ids = if shift {
                game.world.selected.clone()
            } else {
                vec![]
            };
            if shift && ids.contains(&id) {
                ids.retain(|i| *i != id);
            } else {
                ids.push(id);
            }
            let _ = game.world.select(&ids);
            game.rebuild = true;
        } else if let Some(id) = game
            .world
            .tribes
            .iter()
            .find(|t| t.id != 0 && t.home.distance(p) < 5.5)
            .map(|t| t.id)
        {
            game.target = id;
        } else if !shift {
            let _ = game.world.select(&[]);
            game.rebuild = true;
        }
    } else {
        game.placement = None;
        if let Some(id) = game
            .world
            .food_sites
            .iter()
            .filter(|s| s.amount > 0. && s.position.distance(p) < 2.3)
            .min_by(|a, b| a.position.distance(p).total_cmp(&b.position.distance(p)))
            .map(|s| s.id)
        {
            game.command(TribeCommand::GatherFood(id));
        } else if let Some(id) = game
            .world
            .tribes
            .iter()
            .find(|t| t.id != 0 && t.home.distance(p) < 5.5)
            .map(|t| t.id)
        {
            game.target = id;
            game.command(TribeCommand::Attack(id));
        } else if let Some(id) = game
            .world
            .members
            .iter()
            .find(|m| {
                m.tribe_id == 0
                    && m.alive()
                    && m.health < m.max_health
                    && m.position.distance(p) < 1.8
            })
            .map(|m| m.id)
        {
            game.command(TribeCommand::Heal(id));
        } else {
            game.command(TribeCommand::Move(p));
        }
    }
}
fn mouse_orders(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, transform)) = camera.single() else {
        return;
    };
    game.cursor = None;
    if let Some(cursor) = window.cursor_position() {
        if cursor.x < 280. || cursor.y < 90. || cursor.y > window.height() - 80. {
            return;
        }
        if let Ok(ray) = camera.viewport_to_world(transform, cursor) {
            if let Some(point) =
                ray.plane_intersection_point(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
            {
                let p = TribeVec2::new(point.x, point.z);
                game.cursor = Some(p);
                if buttons.just_pressed(MouseButton::Left) {
                    handle_click(
                        &mut game,
                        p,
                        true,
                        keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
                    );
                }
                if buttons.just_pressed(MouseButton::Right) {
                    handle_click(&mut game, p, false, false);
                }
            }
        }
    }
}
fn read_save(path: &Path) -> Result<TribeWorld, String> {
    let bytes = crate::cell_game::read_save(path).map_err(|e| e.to_string())?;
    TribeWorld::from_snapshot(TribeSnapshot::from_bytes(&bytes).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn save_game(game: &Game) -> Result<(), String> {
    let bytes = game
        .world
        .snapshot()
        .to_bytes()
        .map_err(|e| e.to_string())?;
    crate::cell_game::write_save(&game.save_path, &bytes).map_err(|e| e.to_string())
}
fn drive(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>) {
    let seconds = time.delta_secs_f64().min(0.25);
    game.elapsed += seconds;
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
        game.demo_phase = DemoPhase::Start;
        game.demo_delivered = 0.;
        game.notice = "New tribe, same seed.".into();
    }
    if keys.just_pressed(KeyCode::F5) {
        game.notice = match save_game(&game) {
            Ok(()) => "Tribe saved.".into(),
            Err(e) => format!("Save failed: {e}"),
        };
    }
    if keys.just_pressed(KeyCode::F9) {
        match read_save(&game.save_path) {
            Ok(w) => {
                game.world = w;
                game.rebuild = true;
                game.paused = false;
                game.notice = "Tribe loaded.".into();
            }
            Err(e) => game.notice = format!("Load failed: {e}"),
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        let targets: Vec<_> = game
            .world
            .tribes
            .iter()
            .filter(|t| t.id != 0 && !t.resolved())
            .map(|t| t.id)
            .collect();
        if let Some(first) = targets.first() {
            game.target = targets
                .iter()
                .position(|id| *id == game.target)
                .map_or(*first, |i| targets[(i + 1) % targets.len()]);
        }
    }
    for (key, action) in [
        (KeyCode::Digit1, Action::SelectAll),
        (KeyCode::KeyG, Action::Gather),
        (KeyCode::Digit2, Action::Build(BuildingKind::GatheringHut)),
        (KeyCode::Digit3, Action::Build(BuildingKind::WeaponHut)),
        (KeyCode::Digit4, Action::Build(BuildingKind::InstrumentHut)),
        (KeyCode::Digit5, Action::Build(BuildingKind::HealingHut)),
        (KeyCode::KeyT, Action::Equip(ToolKind::Instrument)),
        (KeyCode::KeyQ, Action::Social(SocialPerformance::Sing)),
        (KeyCode::KeyW, Action::Social(SocialPerformance::Dance)),
        (KeyCode::KeyE, Action::Social(SocialPerformance::Play)),
        (KeyCode::KeyA, Action::Attack),
        (KeyCode::KeyH, Action::Heal),
    ] {
        if keys.just_pressed(key) {
            apply_action(&mut game, action);
        }
    }
    if keys.just_pressed(KeyCode::KeyC) {
        game.auto_place();
    }
    let dx = (keys.just_pressed(KeyCode::ArrowRight) as i8
        - keys.just_pressed(KeyCode::ArrowLeft) as i8) as f32;
    let dy = (keys.just_pressed(KeyCode::ArrowDown) as i8
        - keys.just_pressed(KeyCode::ArrowUp) as i8) as f32;
    if dx != 0. || dy != 0. {
        if let Some(p) = game
            .world
            .members
            .iter()
            .find(|m| game.world.selected.contains(&m.id))
            .map(|m| m.position)
        {
            game.command(TribeCommand::Move(p.add(TribeVec2::new(dx * 6., dy * 6.))));
        }
    }
    if !game.paused {
        if game.demo_seconds.is_some() {
            demo_actions(&mut game);
        }
        let report = game.world.advance(seconds);
        for event in report.events {
            match event {
                TribeEvent::FoodDelivered { tribe: 0, amount } => {
                    game.demo_delivered += amount;
                    game.notice = format!("Food delivered: +{amount:.0}");
                }
                TribeEvent::BuildingCompleted { .. }
                | TribeEvent::ToolEquipped { .. }
                | TribeEvent::MemberDied { .. }
                | TribeEvent::TribeAllied { .. }
                | TribeEvent::TribeConquered { .. } => {
                    game.rebuild = true;
                    game.notice = format!("{event:?}");
                }
                _ => {}
            }
        }
    }
}
fn demo_actions(game: &mut Game) {
    if game.world.state != TribeState::Playing {
        game.demo_phase = DemoPhase::Done;
        return;
    }
    match game.demo_phase {
        DemoPhase::Start => {
            game.select_all();
            // Leave the food-hauling corridor east of home clear. This is a
            // normal paid placement command, not a simulation-state shortcut.
            let position = game.world.tribes[0].home.add(TribeVec2::new(0., -5.));
            game.command(TribeCommand::Build {
                kind: BuildingKind::InstrumentHut,
                position,
            });
            if game
                .world
                .buildings
                .iter()
                .any(|b| b.tribe_id == 0 && b.kind == BuildingKind::InstrumentHut)
            {
                game.demo_phase = DemoPhase::Building;
            }
        }
        DemoPhase::Building => {
            if game
                .world
                .buildings
                .iter()
                .any(|b| b.tribe_id == 0 && b.kind == BuildingKind::InstrumentHut && b.complete())
            {
                game.select_all();
                game.command(TribeCommand::EquipTool(ToolKind::Instrument));
                game.demo_phase = DemoPhase::Equipping;
            }
        }
        DemoPhase::Equipping => {
            if game
                .world
                .members
                .iter()
                .filter(|m| m.tribe_id == 0 && m.alive())
                .all(|m| m.tool == Some(ToolKind::Instrument))
            {
                game.select_all();
                game.gather();
                game.demo_phase = DemoPhase::Gathering;
            }
        }
        DemoPhase::Gathering => {
            let alive = game
                .world
                .members
                .iter()
                .filter(|m| m.tribe_id == 0 && m.alive())
                .count();
            if game.demo_delivered >= alive as f32 * 8. - 0.01 {
                game.demo_phase = DemoPhase::Diplomacy;
            }
        }
        DemoPhase::Diplomacy => {
            if let Some((id, cue)) = game
                .world
                .tribes
                .iter()
                .find(|t| t.id != 0 && !t.resolved())
                .map(|t| (t.id, t.social_cue))
            {
                game.target = id;
                game.select_all();
                game.socialize(cue);
            } else {
                game.demo_phase = DemoPhase::Done;
            }
        }
        DemoPhase::Done => {}
    }
}

fn piece(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    scale: Vec3,
) {
    parent.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_scale(scale),
    ));
}
fn sync_scene(
    mut commands: Commands,
    mut game: ResMut<Game>,
    art: Res<Art>,
    mut objects: ResMut<Objects>,
    mut bars: Query<(&HealthBar, &mut Transform), Without<Ground>>,
    mut grounds: Query<&mut Transform, (With<Ground>, Without<HealthBar>)>,
    mut ghosts: Query<
        (
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        (With<Ghost>, Without<Ground>, Without<HealthBar>),
    >,
) {
    if game.rebuild {
        for (_, e) in objects.0.drain() {
            commands.entity(e).despawn();
        }
        game.rebuild = false;
    }
    let w = &game.world;
    for mut ground in &mut grounds {
        ground.scale = Vec3::new(
            w.config.half_extents.x * 2.,
            1.,
            w.config.half_extents.y * 2.,
        );
    }
    let mut wanted = std::collections::HashSet::new();
    for member in w
        .members
        .iter()
        .filter(|m| m.alive() && !w.tribes[m.tribe_id as usize].conquered)
    {
        let key = (0, member.id);
        wanted.insert(key);
        if let Some(e) = objects.0.get(&key) {
            commands
                .entity(*e)
                .insert(Transform::from_translation(location(member.position, 0.)));
        } else {
            let team = art.teams[member.tribe_id as usize % art.teams.len()].clone();
            let selected = w.selected.contains(&member.id);
            let entity = commands
                .spawn((
                    Transform::from_translation(location(member.position, 0.)),
                    Visibility::Visible,
                    Dynamic,
                ))
                .with_children(|p| {
                    piece(
                        p,
                        art.cylinder.clone(),
                        team.clone(),
                        Vec3::new(0., 0.8, 0.),
                        Vec3::new(0.45, 1.15, 0.45),
                    );
                    piece(
                        p,
                        art.sphere.clone(),
                        art.white.clone(),
                        Vec3::new(0., 1.65, 0.),
                        Vec3::splat(0.43),
                    );
                    piece(
                        p,
                        art.cone.clone(),
                        team,
                        Vec3::new(0., 2.15, 0.),
                        Vec3::new(0.55, 0.55, 0.55),
                    );
                    for x in [-0.24, 0.24] {
                        piece(
                            p,
                            art.cube.clone(),
                            art.dark.clone(),
                            Vec3::new(x, 0.2, 0.08),
                            Vec3::new(0.28, 0.4, 0.4),
                        );
                    }
                    if let Some(tool) = member.tool {
                        match tool {
                            ToolKind::Weapon => {
                                piece(
                                    p,
                                    art.cylinder.clone(),
                                    art.dark.clone(),
                                    Vec3::new(0.72, 1.2, 0.),
                                    Vec3::new(0.11, 1.6, 0.11),
                                );
                                piece(
                                    p,
                                    art.sphere.clone(),
                                    art.roof.clone(),
                                    Vec3::new(0.72, 2., 0.),
                                    Vec3::splat(0.3),
                                );
                            }
                            ToolKind::Instrument => {
                                piece(
                                    p,
                                    art.sphere.clone(),
                                    art.roof.clone(),
                                    Vec3::new(0.65, 1., 0.),
                                    Vec3::new(0.34, 0.45, 0.22),
                                );
                                piece(
                                    p,
                                    art.cylinder.clone(),
                                    art.white.clone(),
                                    Vec3::new(0.65, 1.55, 0.),
                                    Vec3::new(0.09, 0.75, 0.09),
                                );
                            }
                            ToolKind::Gathering => piece(
                                p,
                                art.sphere.clone(),
                                art.food.clone(),
                                Vec3::new(0.65, 0.75, 0.),
                                Vec3::splat(0.35),
                            ),
                        }
                    }
                    if selected {
                        p.spawn((
                            Mesh3d(art.ring.clone()),
                            MeshMaterial3d(art.selected.clone()),
                            Transform::from_xyz(0., 0.05, 0.)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                                .with_scale(Vec3::splat(0.95)),
                        ));
                    }
                    p.spawn((
                        Mesh3d(art.cube.clone()),
                        MeshMaterial3d(art.selected.clone()),
                        Transform::from_xyz(0., 2.75, 0.).with_scale(Vec3::new(1.1, 0.12, 0.1)),
                        HealthBar(member.id),
                    ));
                })
                .id();
            objects.0.insert(key, entity);
        }
    }
    for tribe in &w.tribes {
        let key = (1, tribe.id);
        wanted.insert(key);
        if objects.0.contains_key(&key) {
            continue;
        }
        let team = if tribe.conquered {
            art.dark.clone()
        } else {
            art.teams[tribe.id as usize % art.teams.len()].clone()
        };
        let entity = commands
            .spawn((
                Transform::from_translation(location(tribe.home, 0.)),
                Visibility::Visible,
                Dynamic,
            ))
            .with_children(|p| {
                p.spawn((
                    Mesh3d(art.ring.clone()),
                    MeshMaterial3d(if tribe.allied() {
                        art.selected.clone()
                    } else {
                        team.clone()
                    }),
                    Transform::from_xyz(0., 0.04, 0.)
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(3.8)),
                ));
                piece(
                    p,
                    art.cylinder.clone(),
                    art.dark.clone(),
                    Vec3::new(0., 1.9, 0.),
                    Vec3::new(0.6, 3.8, 0.6),
                );
                piece(
                    p,
                    art.cube.clone(),
                    team.clone(),
                    Vec3::new(0., 2.1, 0.),
                    Vec3::new(2.6, 0.5, 0.4),
                );
                piece(
                    p,
                    art.sphere.clone(),
                    team,
                    Vec3::new(0., 4., 0.),
                    Vec3::new(0.9, 0.75, 0.6),
                );
                piece(
                    p,
                    art.cone.clone(),
                    art.white.clone(),
                    Vec3::new(0., 4.9, 0.),
                    Vec3::new(0.65, 1.1, 0.65),
                );
            })
            .id();
        objects.0.insert(key, entity);
    }
    for site in &w.food_sites {
        let key = (2, site.id);
        wanted.insert(key);
        let scale = (site.amount / site.initial_amount.max(1.)).sqrt().max(0.15);
        if let Some(e) = objects.0.get(&key) {
            commands.entity(*e).insert(
                Transform::from_translation(location(site.position, 0.))
                    .with_scale(Vec3::new(scale, scale, scale)),
            );
        } else {
            let entity = commands
                .spawn((
                    Transform::from_translation(location(site.position, 0.))
                        .with_scale(Vec3::splat(scale)),
                    Visibility::Visible,
                    Dynamic,
                ))
                .with_children(|p| {
                    piece(
                        p,
                        art.cylinder.clone(),
                        art.roof.clone(),
                        Vec3::new(0., 0.25, 0.),
                        Vec3::new(1.2, 0.5, 1.2),
                    );
                    for j in 0..6 {
                        let a = j as f32 * std::f32::consts::TAU / 6.;
                        piece(
                            p,
                            art.sphere.clone(),
                            art.food.clone(),
                            Vec3::new(a.cos() * 0.65, 0.65, a.sin() * 0.65),
                            Vec3::splat(0.45),
                        );
                    }
                })
                .id();
            objects.0.insert(key, entity);
        }
    }
    for building in &w.buildings {
        let key = (3, building.id);
        wanted.insert(key);
        let height = 0.2 + 0.8 * building.progress;
        if let Some(e) = objects.0.get(&key) {
            commands.entity(*e).insert(
                Transform::from_translation(location(building.position, 0.))
                    .with_scale(Vec3::new(1., height, 1.)),
            );
        } else {
            let team = art.teams[building.tribe_id as usize % art.teams.len()].clone();
            let entity = commands
                .spawn((
                    Transform::from_translation(location(building.position, 0.))
                        .with_scale(Vec3::new(1., height, 1.)),
                    Visibility::Visible,
                    Dynamic,
                ))
                .with_children(|p| {
                    piece(
                        p,
                        art.cylinder.clone(),
                        art.roof.clone(),
                        Vec3::new(0., 1., 0.),
                        Vec3::new(1.35, 2., 1.35),
                    );
                    piece(
                        p,
                        art.cone.clone(),
                        team,
                        Vec3::new(0., 2.6, 0.),
                        Vec3::new(1.7, 1.5, 1.7),
                    );
                    let ornament = match building.kind {
                        BuildingKind::GatheringHut => art.food.clone(),
                        BuildingKind::WeaponHut => art.dark.clone(),
                        BuildingKind::InstrumentHut => art.white.clone(),
                        BuildingKind::HealingHut => art.selected.clone(),
                    };
                    piece(
                        p,
                        art.sphere.clone(),
                        ornament,
                        Vec3::new(0., 3.6, 0.),
                        Vec3::splat(0.4),
                    );
                })
                .id();
            objects.0.insert(key, entity);
        }
    }
    let removed: Vec<_> = objects
        .0
        .keys()
        .filter(|key| !wanted.contains(key))
        .copied()
        .collect();
    for key in removed {
        if let Some(e) = objects.0.remove(&key) {
            commands.entity(e).despawn();
        }
    }
    for (bar, mut transform) in &mut bars {
        if let Some(m) = w.members.iter().find(|m| m.id == bar.0) {
            let ratio = (m.health / m.max_health).clamp(0., 1.);
            transform.scale.x = 1.1 * ratio;
            transform.translation.x = (ratio - 1.) * 0.55;
        }
    }
    for (mut transform, mut material, mut visibility) in &mut ghosts {
        if let (Some(kind), Some(p)) = (game.placement, game.cursor) {
            *visibility = Visibility::Visible;
            *transform = Transform::from_translation(location(p, 0.15))
                .with_scale(Vec3::new(1.35, 0.3, 1.35));
            material.0 = if placement_valid(w, p, kind) {
                art.valid.clone()
            } else {
                art.invalid.clone()
            };
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
fn job(order: &TribeOrder) -> &'static str {
    match order {
        TribeOrder::Idle => "idle",
        TribeOrder::Move(_) => "moving",
        TribeOrder::GatherFood(_) => "gathering",
        TribeOrder::Build(_) => "building",
        TribeOrder::EquipTool(_) => "equipping",
        TribeOrder::PerformSocial { .. } => "performing",
        TribeOrder::Attack(_) => "attacking",
        TribeOrder::Heal(_) => "healing",
    }
}
fn update_hud(
    game: Res<Game>,
    mut texts: ParamSet<(
        Query<&mut Text, With<Status>>,
        Query<&mut Text, With<SelectionText>>,
        Query<&mut Text, With<TargetText>>,
        Query<&mut Text, With<Notice>>,
        Query<&mut Text, With<Banner>>,
    )>,
) {
    let w = &game.world;
    let alive = w
        .members
        .iter()
        .filter(|m| m.tribe_id == 0 && m.alive())
        .count();
    let resolved = w
        .tribes
        .iter()
        .filter(|t| t.id != 0 && t.resolved())
        .count();
    for mut t in &mut texts.p0() {
        t.0=format!("TRIBE  |  Food {:.0}  |  Members {}  |  Resolved {}/{}  |  {}\nBuild tools, gather finite food, then ally with or conquer every other tribe.",w.tribes[0].food,alive,resolved,w.tribes.len()-1,if game.paused{"PAUSED"}else{"Playing"});
    }
    let mut summary = format!("Selected {}/{}", w.selected.len(), alive);
    if let Some(m) = w.members.iter().find(|m| w.selected.contains(&m.id)) {
        summary += &format!(
            " | {}\nHP {:.0} | {} | Carry {:.0}",
            match m.tool {
                Some(ToolKind::Gathering) => "Gathering",
                Some(ToolKind::Weapon) => "Weapon",
                Some(ToolKind::Instrument) => "Instrument",
                None => "Unarmed",
            },
            m.health,
            job(&m.order),
            m.carried_food
        );
    }
    if let Some(b) = w
        .buildings
        .iter()
        .find(|b| b.tribe_id == 0 && !b.complete())
    {
        summary += &format!("\nConstruction {:.0}%", b.progress * 100.);
    }
    for mut t in &mut texts.p1() {
        t.0 = summary.clone();
    }
    let target = w.tribes.iter().find(|t| t.id == game.target);
    let message = target.map_or_else(
        || "No opposing tribe".into(),
        |t| {
            format!(
                "TARGET TRIBE {}  |  Tab next\nCue: {:?}  |  Relation {} / 60\nTotem {:.0}  |  {}",
                t.id,
                t.social_cue,
                t.relationship,
                t.totem_health,
                if t.conquered {
                    "conquered"
                } else if t.allied() {
                    "allied"
                } else {
                    "opposes your performance"
                }
            )
        },
    );
    for mut t in &mut texts.p2() {
        t.0 = message.clone();
    }
    for mut t in &mut texts.p3() {
        t.0 = game.notice.clone();
    }
    for mut t in &mut texts.p4() {
        t.0 = if game.paused {
            "PAUSED\nP to continue".into()
        } else {
            match w.state{TribeState::Playing=>String::new(),TribeState::CivilizationReady=>"CIVILIZATION READY\nEvery opposing tribe is allied or conquered\nR to start a new tribe".into(),TribeState::Defeated=>"YOUR TRIBE HAS FALLEN\nR to restart".into()}
        };
    }
}
fn capture(mut commands: Commands, mut game: ResMut<Game>) {
    game.frames += 1;
    if game.capture_frame.is_none()
        && game.elapsed >= game.demo_seconds.unwrap_or(3.).max(3.)
        && (game.shot.is_some() || game.demo_seconds.is_some())
    {
        if let Some(path) = &game.shot {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path.clone()));
        }
        game.capture_frame = Some(game.frames);
        println!("TRIBE_RESULT state={:?} tick={} food={:.3} selected={} buildings={} delivered={:.3} resolved={} alive={}",game.world.state,game.world.tick,game.world.tribes[0].food,game.world.selected.len(),game.world.buildings.len(),game.demo_delivered,game.world.tribes.iter().filter(|t|t.id!=0&&t.resolved()).count(),game.world.members.iter().filter(|m|m.tribe_id==0&&m.alive()).count());
    }
    if let Some(frame) = game.capture_frame {
        if game.frames > frame + 10 && game.shot.as_ref().is_none_or(|p| p.exists()) {
            std::process::exit(0);
        }
        if game.frames > frame + 600 {
            eprintln!("Tribe screenshot capture failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app(seed: u64, path: PathBuf, demo: bool) -> App {
        let mut app = App::new();
        let mut game = Game::new(seed, path);
        if demo {
            game.demo_seconds = Some(90.);
        }
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(game)
            .add_systems(Update, drive);
        app
    }
    fn frame(app: &mut App, seconds: f64, pressed: &[KeyCode]) {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.reset_all();
        for &key in pressed {
            keys.press(key);
        }
        drop(keys);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(seconds));
        app.update();
    }
    fn temp_path() -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("rustspore-tribe-ui-{}-{nonce}", std::process::id()))
            .join("tribe.rsp")
    }
    #[test]
    fn click_selection_shift_selection_and_context_food_move_attack_use_core_orders() {
        let mut game = Game::new(14, PathBuf::new());
        let first = game.world.members[0].clone();
        let second = game.world.members[1].clone();
        handle_click(&mut game, first.position, true, false);
        assert_eq!(game.world.selected, vec![first.id]);
        handle_click(&mut game, second.position, true, true);
        assert_eq!(game.world.selected.len(), 2);
        let site = game.world.food_sites[0].clone();
        handle_click(&mut game, site.position, false, false);
        assert!(game
            .world
            .members
            .iter()
            .filter(|m| game.world.selected.contains(&m.id))
            .all(|m| m.order == TribeOrder::GatherFood(site.id)));
        let point = game.world.tribes[0].home.add(TribeVec2::new(0., 12.));
        handle_click(&mut game, point, false, false);
        assert_eq!(game.world.members[0].order, TribeOrder::Move(point));
        let target = game.world.tribes[1].home;
        handle_click(&mut game, target, false, false);
        assert_eq!(game.world.members[0].order, TribeOrder::Attack(1));
        assert_eq!(game.target, 1);
    }
    #[test]
    fn placement_preview_and_actual_click_reject_blocked_sites_and_charge_once() {
        let mut game = Game::new(14, PathBuf::new());
        let home = game.world.tribes[0].home;
        assert!(!placement_valid(&game.world, home, BuildingKind::WeaponHut));
        apply_action(&mut game, Action::Build(BuildingKind::WeaponHut));
        handle_click(&mut game, home, true, false);
        assert!(game.world.buildings.is_empty());
        let food = game.world.tribes[0].food;
        apply_action(&mut game, Action::Build(BuildingKind::WeaponHut));
        handle_click(&mut game, home.add(TribeVec2::new(4., 4.)), true, false);
        assert_eq!(game.world.buildings.len(), 1);
        assert_eq!(game.world.tribes[0].food, food - 10.);
        assert!(!placement_valid(
            &game.world,
            game.world.buildings[0].position,
            BuildingKind::WeaponHut
        ));
    }
    #[test]
    fn actual_drive_keyboard_pause_atomic_save_load_restart_and_bad_load() {
        let path = temp_path();
        let mut app = app(14, path.clone(), false);
        frame(&mut app, 0., &[KeyCode::Digit4, KeyCode::KeyC]);
        assert_eq!(app.world().resource::<Game>().world.buildings.len(), 1);
        frame(&mut app, 0.1, &[]);
        let before = app.world().resource::<Game>().world.clone();
        frame(&mut app, 0.1, &[KeyCode::KeyP]);
        assert_eq!(app.world().resource::<Game>().world, before);
        frame(&mut app, 0., &[KeyCode::KeyP]);
        frame(&mut app, 0., &[KeyCode::F5]);
        assert_eq!(read_save(&path).unwrap(), before);
        frame(&mut app, 0.1, &[KeyCode::ArrowRight]);
        assert_ne!(app.world().resource::<Game>().world, before);
        frame(&mut app, 0., &[KeyCode::F9]);
        assert_eq!(app.world().resource::<Game>().world, before);
        std::fs::write(&path, b"not a tribe save").unwrap();
        frame(&mut app, 0., &[KeyCode::F9]);
        assert_eq!(app.world().resource::<Game>().world, before);
        assert!(app
            .world()
            .resource::<Game>()
            .notice
            .starts_with("Load failed:"));
        frame(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(app.world().resource::<Game>().world, TribeWorld::new(14));
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn unmodified_demo_builds_equips_gathers_and_allies_every_opponent() {
        let mut app = app(14, PathBuf::new(), true);
        for i in 0..1500 {
            frame(&mut app, 0.1, &[]);
            if i % 50 == 0 && std::env::var_os("RUSTSPORE_TRIBE_TRACE").is_some() {
                let g = app.world().resource::<Game>();
                eprintln!(
                    "TRIBE_TRACE phase={:?} tick={} food={} delivered={} members={:?}",
                    g.demo_phase,
                    g.world.tick,
                    g.world.tribes[0].food,
                    g.demo_delivered,
                    g.world
                        .members
                        .iter()
                        .filter(|m| m.tribe_id == 0)
                        .map(|m| (m.id, m.health, m.tool, &m.order, m.carried_food, m.position))
                        .collect::<Vec<_>>()
                );
            }
            if app.world().resource::<Game>().world.state != TribeState::Playing {
                break;
            }
        }
        let game = app.world().resource::<Game>();
        assert_eq!(
            game.world.state,
            TribeState::CivilizationReady,
            "demo phase {:?}, tick {}, delivered {}, food {}, relations {:?}, orders {:?}",
            game.demo_phase,
            game.world.tick,
            game.demo_delivered,
            game.world.tribes[0].food,
            game.world
                .tribes
                .iter()
                .map(|t| t.relationship)
                .collect::<Vec<_>>(),
            game.world
                .members
                .iter()
                .filter(|m| m.tribe_id == 0)
                .map(|m| (&m.order, m.position))
                .collect::<Vec<_>>()
        );
        assert!(game.demo_delivered >= 32.);
        assert!(game
            .world
            .buildings
            .iter()
            .any(|b| b.tribe_id == 0 && b.kind == BuildingKind::InstrumentHut && b.complete()));
        assert!(game
            .world
            .tribes
            .iter()
            .filter(|t| t.id != 0)
            .all(|t| t.allied()));
        assert!(game
            .world
            .members
            .iter()
            .filter(|m| m.tribe_id == 0)
            .all(|m| m.alive() && m.tool == Some(ToolKind::Instrument)));
        game.world.snapshot().to_bytes().unwrap();
    }
    #[test]
    fn frontend_save_replay_preserves_jobs_rng_and_future_commands() {
        let path = temp_path();
        let mut a = app(66, path.clone(), false);
        frame(&mut a, 0., &[KeyCode::KeyG]);
        for _ in 0..100 {
            frame(&mut a, 0.1, &[]);
        }
        frame(&mut a, 0., &[KeyCode::F5]);
        let mut b = app(999, path.clone(), false);
        frame(&mut b, 0., &[KeyCode::F9]);
        assert_eq!(
            a.world().resource::<Game>().world,
            b.world().resource::<Game>().world
        );
        for i in 0..500 {
            let keys = if i == 50 {
                vec![KeyCode::ArrowUp]
            } else {
                vec![]
            };
            frame(&mut a, 0.05, &keys);
            frame(&mut b, 0.05, &keys);
        }
        assert_eq!(
            a.world().resource::<Game>().world,
            b.world().resource::<Game>().world
        );
        assert_eq!(
            a.world()
                .resource::<Game>()
                .world
                .snapshot()
                .to_bytes()
                .unwrap(),
            b.world()
                .resource::<Game>()
                .world
                .snapshot()
                .to_bytes()
                .unwrap()
        );
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(path.parent().unwrap()).unwrap();
    }
}
