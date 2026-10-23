//! Original 3D Civilization RTS baseline driven entirely by core commands.
use bevy::{
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use spore_core::civilization::*;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Resource)]
struct Game {
    world: CivilizationWorld,
    city: u32,
    slot: u8,
    domain: VehicleDomain,
    style: VehicleRole,
    paused: bool,
    notice: String,
    save_path: PathBuf,
    shot: Option<PathBuf>,
    demo: Option<f64>,
    elapsed: f64,
    frames: u64,
    captured: Option<u64>,
}
impl Game {
    fn new(seed: u64, path: PathBuf) -> Self {
        let world = CivilizationWorld::new(seed);
        let city = world.cities[0].id;
        Self {
            world,
            city,
            slot: 1,
            domain: VehicleDomain::Air,
            style: VehicleRole::Religious,
            paused: false,
            notice: String::new(),
            save_path: path,
            shot: None,
            demo: None,
            elapsed: 0.,
            frames: 0,
            captured: None,
        }
    }
}
#[derive(Resource)]
struct Art {
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    ring: Handle<Mesh>,
    owners: Vec<Handle<StandardMaterial>>,
    roles: Vec<Handle<StandardMaterial>>,
    house: Handle<StandardMaterial>,
    factory: Handle<StandardMaterial>,
    fun: Handle<StandardMaterial>,
    field: Handle<StandardMaterial>,
    white: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}
#[derive(Resource, Default)]
struct Objects(HashMap<(u8, u32), Entity>);
#[derive(Resource)]
struct CameraRig {
    center: Vec2,
    yaw: f32,
    distance: f32,
}
impl Default for CameraRig {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            yaw: std::f32::consts::FRAC_PI_2,
            distance: 145.,
        }
    }
}
#[derive(Component)]
struct RtsCamera;
#[derive(Component)]
struct Header;
#[derive(Component)]
struct Details;
#[derive(Component)]
struct Notice;
#[derive(Component)]
struct Banner;

pub fn run(args: &[String]) -> Result<(), String> {
    let mut seed = 14;
    let mut style = VehicleRole::Religious;
    let mut shot = None;
    let mut demo = None;
    let mut save = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RustSpore/saves/civilization.rsp");
    let mut at = 0;
    while at < args.len() {
        let value = || {
            args.get(at + 1)
                .ok_or_else(|| format!("{} requires a value", args[at]))
        };
        match args[at].as_str() {
            "--seed" => seed = value()?.parse().map_err(|_| "invalid seed")?,
            "--save-file" => save = PathBuf::from(value()?),
            "--shot" => shot = Some(PathBuf::from(value()?)),
            "--role" => {
                style = match value()?.as_str() {
                    "military" => VehicleRole::Military,
                    "religious" => VehicleRole::Religious,
                    "economic" => VehicleRole::Economic,
                    _ => return Err("role must be military, religious or economic".into()),
                }
            }
            "--demo" => {
                let seconds: f64 = value()?.parse().map_err(|_| "invalid demo duration")?;
                if !seconds.is_finite() || !(3.0..=600.).contains(&seconds) {
                    return Err("demo duration must be 3..600 seconds".into());
                }
                demo = Some(seconds);
            }
            other => return Err(format!("unknown Civilization option {other}")),
        }
        at += 2;
    }
    if shot.as_ref().is_some_and(|path| path.exists()) {
        return Err("screenshot exists; choose a new output path".into());
    }
    if let Some(parent) = shot
        .as_ref()
        .and_then(|path| path.parent())
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut game = Game::new(seed, save);
    game.style = style;
    game.shot = shot;
    game.demo = demo;
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore | Civilization".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.38, 0.55, 0.64)))
        .insert_resource(game)
        .init_resource::<Objects>()
        .init_resource::<CameraRig>()
        .add_systems(Startup, setup)
        .add_systems(Update, (drive, mouse, scene, camera, hud, capture).chain())
        .run();
    Ok(())
}
fn pos(point: CivilizationVec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, point.y)
}
fn mat(materials: &mut Assets<StandardMaterial>, color: Color) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.8,
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
        "CIVILIZATION_GPU name={:?} backend={:?} type={:?}",
        adapter.name, adapter.backend, adapter.device_type
    );
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            brightness: 500.,
            ..default()
        },
        Transform::from_xyz(0., 115., 95.).looking_at(Vec3::ZERO, Vec3::Y),
        RtsCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.7, 0.)),
    ));
    let cube = meshes.add(Cuboid::new(1., 1., 1.));
    let sphere = meshes.add(Sphere::new(1.));
    let cylinder = meshes.add(Cylinder::new(1., 1.));
    let ring = meshes.add(Annulus::new(0.96, 1.));
    let ground = mat(&mut materials, Color::srgb(0.24, 0.36, 0.23));
    let river = mat(&mut materials, Color::srgb(0.13, 0.40, 0.53));
    let bridge = mat(&mut materials, Color::srgb(0.65, 0.57, 0.40));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(ground),
        Transform::from_xyz(0., -0.15, 0.).with_scale(Vec3::new(
            game.world.config.half_extents.x * 2.,
            0.3,
            game.world.config.half_extents.y * 2.,
        )),
    ));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(river),
        Transform::from_xyz(0., 0.02, 0.).with_scale(Vec3::new(
            16.,
            0.04,
            game.world.config.half_extents.y * 2.,
        )),
    ));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(bridge.clone()),
        Transform::from_xyz(0., 0.12, 0.).with_scale(Vec3::new(20., 0.2, 12.)),
    ));
    for side in [-1., 1.] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(bridge.clone()),
            Transform::from_xyz(0., 0.6, side * 6.).with_scale(Vec3::new(20., 0.7, 0.25)),
        ));
    }
    let owners = [
        Color::srgb(0.17, 0.76, 0.63),
        Color::srgb(0.91, 0.41, 0.25),
        Color::srgb(0.64, 0.43, 0.82),
        Color::srgb(0.31, 0.61, 0.88),
        Color::srgb(0.8, 0.65, 0.2),
        Color::srgb(0.75, 0.35, 0.60),
        Color::srgb(0.5, 0.74, 0.34),
        Color::srgb(0.65, 0.65, 0.72),
    ]
    .into_iter()
    .map(|color| mat(&mut materials, color))
    .collect();
    let roles = [
        Color::srgb(0.95, 0.58, 0.22),
        Color::srgb(0.74, 0.57, 0.94),
        Color::srgb(0.38, 0.81, 0.93),
    ]
    .into_iter()
    .map(|color| mat(&mut materials, color))
    .collect();
    commands.insert_resource(Art {
        cube,
        sphere,
        cylinder,
        ring,
        owners,
        roles,
        house: mat(&mut materials, Color::srgb(0.86, 0.79, 0.57)),
        factory: mat(&mut materials, Color::srgb(0.46, 0.50, 0.51)),
        fun: mat(&mut materials, Color::srgb(0.85, 0.43, 0.66)),
        field: mat(&mut materials, Color::srgb(0.93, 0.76, 0.27)),
        white: mat(&mut materials, Color::srgb(0.93, 0.94, 0.85)),
        selected: mat(&mut materials, Color::srgb(0.97, 0.91, 0.41)),
    });
    ui(&mut commands);
}
fn ui(commands: &mut Commands) {
    let bg = Color::srgba(0.025, 0.052, 0.065, 0.94);
    let pale = Color::srgb(0.85, 0.91, 0.87);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(18),
                left: px(22),
                right: px(22),
                height: px(78),
                padding: UiRect::all(px(15)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(bg),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 20.,
                    ..default()
                },
                TextColor(pale),
                Header,
            ));
        });
    commands.spawn((Node{position_type:PositionType::Absolute,left:px(22),top:px(113),width:px(263),padding:UiRect::all(px(14)),border_radius:BorderRadius::all(px(10)),..default()},BackgroundColor(bg))).with_children(|panel|{panel.spawn((Text::new("THE NEXT HORIZON\n\nControl or ally every city.\nMilitary: attack a city's health.\nFaith: convert it into an ally.\nTrade: complete 3 visits, then buy.\n\nGeysers grant finite income.\nHouses grow the population.\nFactories accelerate production.\nEntertainment lifts happiness.\nPlacement beside houses matters.\n\nTeal is your nation. Other colours\nremain active opposing nations.\n\nOriginal prototype rules and art.\nRetail fidelity and full vehicle /\ncity editors remain separate gates."),TextFont{font_size:13.,..default()},TextColor(pale)));});
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(22),
                top: px(113),
                width: px(286),
                padding: UiRect::all(px(14)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(bg),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.,
                    ..default()
                },
                TextColor(pale),
                Details,
            ));
        });
    commands.spawn((Node{position_type:PositionType::Absolute,left:px(22),right:px(22),bottom:px(18),padding:UiRect::all(px(13)),border_radius:BorderRadius::all(px(10)),flex_direction:FlexDirection::Column,row_gap:px(6),..default()},BackgroundColor(bg))).with_children(|panel|{
        for line in ["Left click / Shift: select units or city   Right click: context order   Tab: city target   J: next unit   G: claim closest geyser","Z / X / C: produce military / religious / economic   1 / 2 / 3: land / sea / air   K: attack   F: convert   T: trade   O: buy city","B / N / M: build house / factory / fun   [ / ]: building slot   V: demolish slot   WASD: pan   Q / E: orbit   +/-: zoom","P pause   R restart   F5 save   F9 load   Esc exit   |   Commands require selected units or a controlled city"]{panel.spawn((Text::new(line),TextFont{font_size:12.,..default()},TextColor(pale)));}
        panel.spawn((Text::new(""),TextFont{font_size:12.,..default()},TextColor(Color::srgb(0.96,0.83,0.47)),Notice));
    });
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 28.,
            ..default()
        },
        TextColor(Color::srgb(1., 0.96, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            left: percent(33),
            top: percent(40),
            ..default()
        },
        Banner,
    ));
}
fn error_message(error: CivilizationError) -> &'static str {
    match error {
        CivilizationError::InsufficientFunds => "Not enough credits",
        CivilizationError::NotOwned => "Choose a controlled city or an opposing target",
        CivilizationError::InvalidSelection => "Select your vehicles first",
        CivilizationError::WrongRole => "That order needs vehicles of the matching role",
        CivilizationError::WrongDomain => {
            "This unit domain cannot reach that location or use this dock"
        }
        CivilizationError::TradeRequired => "Complete three trade visits before buying the city",
        CivilizationError::Cooldown => "Wait for construction to finish",
        CivilizationError::InvalidPlacement => "Choose an empty building slot 0..11",
        CivilizationError::QueueFull => "Production queue or unit capacity is full",
        CivilizationError::Terminal => "This campaign is finished; R restarts",
        _ => "That command cannot be completed",
    }
}
fn perform(game: &mut Game, command: CivilizationCommand) {
    game.notice = match game.world.issue_command(command) {
        Ok(()) => "Command accepted".into(),
        Err(error) => error_message(error).into(),
    };
}
fn city_order(game: &mut Game, city: u32) {
    let original = game.world.selected.clone();
    let mut issued = 0;
    for role in [
        VehicleRole::Military,
        VehicleRole::Religious,
        VehicleRole::Economic,
    ] {
        let ids: Vec<_> = original
            .iter()
            .copied()
            .filter(|id| {
                game.world
                    .vehicles
                    .iter()
                    .any(|vehicle| vehicle.id == *id && vehicle.role == role)
            })
            .collect();
        if ids.is_empty() {
            continue;
        }
        if game.world.select(ids).is_err() {
            continue;
        }
        let command = match role {
            VehicleRole::Military => CivilizationCommand::Attack {
                target: CivilizationTarget::City(city),
            },
            VehicleRole::Religious => CivilizationCommand::Convert { city },
            VehicleRole::Economic => CivilizationCommand::Trade { city },
        };
        if game.world.issue_command(command).is_ok() {
            issued += 1;
        }
    }
    let _ = game.world.select(original);
    game.notice = if issued > 0 {
        "Context orders issued by vehicle role".into()
    } else {
        "Select vehicles that can reach an opposing city".into()
    };
}
fn pointer(game: &mut Game, point: CivilizationVec2, add: bool, context: bool) {
    if context {
        if let Some(city) = game
            .world
            .cities
            .iter()
            .filter(|city| city.controller() != 0 && city.position.distance(point) < 7.)
            .min_by(|a, b| {
                a.position
                    .distance(point)
                    .total_cmp(&b.position.distance(point))
            })
            .map(|city| city.id)
        {
            game.city = city;
            city_order(game, city);
            return;
        }
        if let Some(field) = game
            .world
            .geysers
            .iter()
            .filter(|field| field.reserves > 0. && field.position.distance(point) < 3.)
            .min_by(|a, b| {
                a.position
                    .distance(point)
                    .total_cmp(&b.position.distance(point))
            })
            .map(|field| field.id)
        {
            perform(game, CivilizationCommand::Claim { geyser: field });
            return;
        }
        if let Some(target) = game
            .world
            .vehicles
            .iter()
            .find(|vehicle| vehicle.owner != 0 && vehicle.position.distance(point) < 2.)
            .map(|vehicle| vehicle.id)
        {
            perform(
                game,
                CivilizationCommand::Attack {
                    target: CivilizationTarget::Vehicle(target),
                },
            );
            return;
        }
        perform(game, CivilizationCommand::Move { destination: point });
        return;
    }
    if let Some(id) = game
        .world
        .vehicles
        .iter()
        .filter(|vehicle| vehicle.owner == 0 && vehicle.position.distance(point) < 2.3)
        .min_by(|a, b| {
            a.position
                .distance(point)
                .total_cmp(&b.position.distance(point))
        })
        .map(|vehicle| vehicle.id)
    {
        let mut selected = if add {
            game.world.selected.clone()
        } else {
            Vec::new()
        };
        if let Some(at) = selected.iter().position(|selected| *selected == id) {
            selected.remove(at);
        } else {
            selected.push(id);
        }
        let _ = game.world.select(selected);
        return;
    }
    if let Some(city) = game
        .world
        .cities
        .iter()
        .filter(|city| city.position.distance(point) < 7.)
        .min_by(|a, b| {
            a.position
                .distance(point)
                .total_cmp(&b.position.distance(point))
        })
        .map(|city| city.id)
    {
        game.city = city;
        if !add {
            let _ = game.world.select(Vec::new());
        }
        return;
    }
    if !add {
        let _ = game.world.select(Vec::new());
    }
}
fn load(path: &Path) -> Result<CivilizationWorld, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("save exceeds 4 MiB".into());
    }
    CivilizationSnapshot::from_bytes(&bytes)
        .and_then(CivilizationWorld::from_snapshot)
        .map_err(|error| error.to_string())
}
fn drive(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>) {
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if keys.just_pressed(KeyCode::KeyP) {
        game.paused = !game.paused;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.world.restart();
        game.city = game.world.cities[0].id;
        game.paused = false;
        game.notice = "Campaign restarted".into();
    }
    if keys.just_pressed(KeyCode::F5) {
        game.notice = match game
            .world
            .snapshot()
            .to_bytes()
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                crate::cell_game::write_save(&game.save_path, &bytes)
                    .map_err(|error| error.to_string())
            }) {
            Ok(()) => "Civilization saved".into(),
            Err(error) => format!("Save failed: {error}"),
        };
    }
    if keys.just_pressed(KeyCode::F9) {
        match load(&game.save_path) {
            Ok(world) => {
                game.world = world;
                game.city = game
                    .world
                    .cities
                    .iter()
                    .find(|city| city.controller() == 0)
                    .unwrap_or(&game.world.cities[0])
                    .id;
                game.paused = false;
                game.notice = "Civilization loaded".into();
            }
            Err(error) => game.notice = format!("Load failed: {error}"),
        }
    }
    if keys.just_pressed(KeyCode::Digit1) {
        game.domain = VehicleDomain::Land;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        game.domain = VehicleDomain::Sea;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        game.domain = VehicleDomain::Air;
    }
    if keys.just_pressed(KeyCode::Tab) {
        let at = game
            .world
            .cities
            .iter()
            .position(|city| city.id == game.city)
            .unwrap_or(0);
        game.city = game.world.cities[(at + 1) % game.world.cities.len()].id;
    }
    if keys.just_pressed(KeyCode::KeyJ) {
        let ids: Vec<_> = game
            .world
            .vehicles
            .iter()
            .filter(|vehicle| vehicle.owner == 0)
            .map(|vehicle| vehicle.id)
            .collect();
        if !ids.is_empty() {
            let at = game
                .world
                .selected
                .last()
                .and_then(|id| ids.iter().position(|item| item == id))
                .map(|at| (at + 1) % ids.len())
                .unwrap_or(0);
            let _ = game.world.select(vec![ids[at]]);
        }
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        game.slot = (game.slot + 11) % 12;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        game.slot = (game.slot + 1) % 12;
    }
    let city = game.city;
    let slot = game.slot;
    let domain = game.domain;
    for (key, kind) in [
        (KeyCode::KeyB, BuildingKind::House),
        (KeyCode::KeyN, BuildingKind::Factory),
        (KeyCode::KeyM, BuildingKind::Entertainment),
    ] {
        if keys.just_pressed(key) {
            perform(&mut game, CivilizationCommand::Build { city, kind, slot });
        }
    }
    if keys.just_pressed(KeyCode::KeyV) {
        perform(&mut game, CivilizationCommand::Demolish { city, slot });
    }
    for (key, role) in [
        (KeyCode::KeyZ, VehicleRole::Military),
        (KeyCode::KeyX, VehicleRole::Religious),
        (KeyCode::KeyC, VehicleRole::Economic),
    ] {
        if keys.just_pressed(key) {
            perform(
                &mut game,
                CivilizationCommand::Produce { city, role, domain },
            );
        }
    }
    for (key, command) in [
        (
            KeyCode::KeyK,
            CivilizationCommand::Attack {
                target: CivilizationTarget::City(city),
            },
        ),
        (KeyCode::KeyF, CivilizationCommand::Convert { city }),
        (KeyCode::KeyT, CivilizationCommand::Trade { city }),
        (KeyCode::KeyO, CivilizationCommand::Purchase { city }),
    ] {
        if keys.just_pressed(key) {
            perform(&mut game, command);
        }
    }
    if keys.just_pressed(KeyCode::KeyG) {
        let origin = game
            .world
            .selected
            .first()
            .and_then(|id| game.world.vehicles.iter().find(|vehicle| vehicle.id == *id))
            .map(|vehicle| vehicle.position)
            .unwrap_or(game.world.cities[0].position);
        if let Some(geyser) = game
            .world
            .geysers
            .iter()
            .filter(|field| field.owner != Some(0) && field.reserves > 0.)
            .min_by(|a, b| {
                a.position
                    .distance(origin)
                    .total_cmp(&b.position.distance(origin))
            })
            .map(|field| field.id)
        {
            perform(&mut game, CivilizationCommand::Claim { geyser });
        }
    }
    let dt = time.delta_secs_f64().min(0.1);
    game.elapsed += dt;
    if !game.paused && game.captured.is_none() {
        if game.demo.is_some() {
            demo_orders(&mut game);
        }
        game.world.advance(dt);
    }
}
fn demo_orders(game: &mut Game) {
    if game.world.state != CivilizationState::Playing {
        return;
    }
    let owned = game
        .world
        .cities
        .iter()
        .filter(|city| city.controller() == 0)
        .map(|city| city.id)
        .collect::<Vec<_>>();
    if owned.is_empty() {
        return;
    }
    let defender = game
        .world
        .vehicles
        .iter()
        .find(|vehicle| vehicle.owner == 0 && vehicle.role == VehicleRole::Military)
        .map(|vehicle| vehicle.id);
    let jobs = game
        .world
        .cities
        .iter()
        .filter(|city| city.controller() == 0)
        .flat_map(|city| &city.production)
        .map(|job| job.role)
        .collect::<Vec<_>>();
    let total = game
        .world
        .vehicles
        .iter()
        .filter(|vehicle| vehicle.owner == 0)
        .count()
        + jobs.len();
    let military_pending = jobs.contains(&VehicleRole::Military);
    if total < 5 {
        let role = if defender.is_none() && !military_pending {
            VehicleRole::Military
        } else {
            game.style
        };
        if let Some(city) = game
            .world
            .cities
            .iter()
            .find(|city| city.controller() == 0 && city.production.len() < 3)
            .map(|city| city.id)
        {
            let _ = game.world.issue_command(CivilizationCommand::Produce {
                city,
                role,
                domain: VehicleDomain::Air,
            });
        }
    }
    if let Some(id) = defender {
        let threat = game
            .world
            .vehicles
            .iter()
            .filter(|vehicle| {
                vehicle.owner != 0
                    && game.world.cities.iter().any(|city| {
                        city.controller() == 0 && city.position.distance(vehicle.position) < 23.
                    })
            })
            .min_by(|a, b| {
                a.position
                    .distance(
                        game.world
                            .vehicles
                            .iter()
                            .find(|vehicle| vehicle.id == id)
                            .unwrap()
                            .position,
                    )
                    .total_cmp(
                        &b.position.distance(
                            game.world
                                .vehicles
                                .iter()
                                .find(|vehicle| vehicle.id == id)
                                .unwrap()
                                .position,
                        ),
                    )
            })
            .map(|vehicle| vehicle.id);
        let _ = game.world.select(vec![id]);
        if let Some(target) = threat {
            let _ = game.world.issue_command(CivilizationCommand::Attack {
                target: CivilizationTarget::Vehicle(target),
            });
        } else {
            let vehicle = game
                .world
                .vehicles
                .iter()
                .find(|vehicle| vehicle.id == id)
                .unwrap();
            let home = game
                .world
                .cities
                .iter()
                .find(|city| city.controller() == 0)
                .unwrap()
                .position;
            if matches!(vehicle.order, VehicleOrder::Idle) && vehicle.position.distance(home) > 10.
            {
                let _ = game
                    .world
                    .issue_command(CivilizationCommand::Move { destination: home });
            }
        }
    }
    let ids = game
        .world
        .vehicles
        .iter()
        .filter(|vehicle| {
            vehicle.owner == 0
                && Some(vehicle.id) != defender
                && matches!(vehicle.order, VehicleOrder::Idle)
        })
        .map(|vehicle| vehicle.id)
        .collect::<Vec<_>>();
    for id in ids {
        let vehicle = game
            .world
            .vehicles
            .iter()
            .find(|vehicle| vehicle.id == id)
            .unwrap();
        let role = vehicle.role;
        let target = game
            .world
            .cities
            .iter()
            .filter(|city| city.controller() != 0)
            .min_by(|a, b| {
                a.position
                    .distance(vehicle.position)
                    .total_cmp(&b.position.distance(vehicle.position))
            })
            .map(|city| city.id);
        if let Some(city) = target {
            let _ = game.world.select(vec![id]);
            let command = match role {
                VehicleRole::Military => CivilizationCommand::Attack {
                    target: CivilizationTarget::City(city),
                },
                VehicleRole::Religious => CivilizationCommand::Convert { city },
                VehicleRole::Economic => CivilizationCommand::Trade { city },
            };
            let _ = game.world.issue_command(command);
            game.city = city;
        }
    }
    let purchases = game
        .world
        .cities
        .iter()
        .filter(|city| city.controller() != 0 && city.trade_owner == 0 && city.trade >= 60.)
        .map(|city| city.id)
        .collect::<Vec<_>>();
    for city in purchases {
        let _ = game
            .world
            .issue_command(CivilizationCommand::Purchase { city });
    }
}
fn mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    mut game: ResMut<Game>,
) {
    if game.paused
        || game.demo.is_some()
        || (!buttons.just_pressed(MouseButton::Left) && !buttons.just_pressed(MouseButton::Right))
    {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if cursor.y < 105.
        || cursor.y > window.height() - 160.
        || cursor.x < 300.
        || cursor.x > window.width() - 315.
    {
        return;
    }
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let point = ray.get_point(distance);
    pointer(
        &mut game,
        CivilizationVec2::new(point.x, point.z),
        keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
        buttons.just_pressed(MouseButton::Right),
    );
}
fn spawn_vehicle(commands: &mut Commands, art: &Art, vehicle: &CivilizationVehicle) -> Entity {
    let colour = art.roles[match vehicle.role {
        VehicleRole::Military => 0,
        VehicleRole::Religious => 1,
        VehicleRole::Economic => 2,
    }]
    .clone();
    commands
        .spawn((Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(if vehicle.domain == VehicleDomain::Land {
                    art.cube.clone()
                } else {
                    art.sphere.clone()
                }),
                MeshMaterial3d(colour),
                Transform::default().with_scale(Vec3::new(1.5, 0.55, 0.8)),
            ));
            parent.spawn((
                Mesh3d(art.cube.clone()),
                MeshMaterial3d(art.owners[vehicle.owner as usize].clone()),
                Transform::from_xyz(0., 0.55, 0.).with_scale(Vec3::new(0.55, 0.3, 0.5)),
            ));
            if vehicle.domain == VehicleDomain::Air {
                parent.spawn((
                    Mesh3d(art.cube.clone()),
                    MeshMaterial3d(art.white.clone()),
                    Transform::default().with_scale(Vec3::new(0.7, 0.15, 2.8)),
                ));
            } else if vehicle.domain == VehicleDomain::Land {
                for side in [-1., 1.] {
                    for x in [-0.7, 0.7] {
                        parent.spawn((
                            Mesh3d(art.sphere.clone()),
                            MeshMaterial3d(art.factory.clone()),
                            Transform::from_xyz(x, -0.3, side * 0.6).with_scale(Vec3::splat(0.3)),
                        ));
                    }
                }
            }
        })
        .id()
}
fn scene(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Game>,
    mut objects: ResMut<Objects>,
    mut transforms: Query<&mut Transform, Without<RtsCamera>>,
) {
    let mut live = HashSet::new();
    let mut set =
        |key: (u8, u32), transform: Transform, spawn: &mut dyn FnMut(&mut Commands) -> Entity| {
            live.insert(key);
            let entity = *objects.0.entry(key).or_insert_with(|| spawn(&mut commands));
            if let Ok(mut current) = transforms.get_mut(entity) {
                *current = transform;
            } else {
                commands.entity(entity).insert(transform);
            }
        };
    for city in &game.world.cities {
        let owner = city.controller() as usize;
        set(
            (10 + owner as u8, city.id),
            Transform::from_translation(pos(city.position, 1.2))
                .with_scale(Vec3::new(1.8, 2.4, 1.8)),
            &mut |commands| {
                commands
                    .spawn((
                        Mesh3d(art.cylinder.clone()),
                        MeshMaterial3d(art.owners[owner].clone()),
                        Transform::default(),
                    ))
                    .id()
            },
        );
        set(
            (30 + owner as u8, city.id),
            Transform::from_translation(pos(city.position, 0.09))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(14.)),
            &mut |commands| {
                commands
                    .spawn((
                        Mesh3d(art.ring.clone()),
                        MeshMaterial3d(art.owners[owner].clone()),
                        Transform::default(),
                    ))
                    .id()
            },
        );
        for building in &city.buildings {
            let colour = match building.kind {
                BuildingKind::House => art.house.clone(),
                BuildingKind::Factory => art.factory.clone(),
                BuildingKind::Entertainment => art.fun.clone(),
            };
            let height = if building.kind == BuildingKind::Factory {
                2.7
            } else {
                1.5
            };
            set(
                (1, building.id),
                Transform::from_translation(pos(
                    city.building_position(building.slot),
                    height * 0.5,
                ))
                .with_scale(Vec3::new(1.3, height, 1.3)),
                &mut |commands| {
                    commands
                        .spawn((
                            Mesh3d(art.cube.clone()),
                            MeshMaterial3d(colour.clone()),
                            Transform::default(),
                        ))
                        .id()
                },
            );
        }
        if city.coastal {
            set(
                (5, city.id),
                Transform::from_translation(Vec3::new(
                    city.position.x.signum() * 6.,
                    0.25,
                    city.position.y,
                ))
                .with_scale(Vec3::new(6., 0.45, 3.)),
                &mut |commands| {
                    commands
                        .spawn((
                            Mesh3d(art.cube.clone()),
                            MeshMaterial3d(art.house.clone()),
                            Transform::default(),
                        ))
                        .id()
                },
            );
        }
    }
    for field in &game.world.geysers {
        let colour = field
            .owner
            .map(|owner| art.owners[owner as usize].clone())
            .unwrap_or(art.field.clone());
        let height = 0.25 + field.reserves / field.max_reserves * 1.3;
        set(
            (50 + field.owner.unwrap_or(8), field.id),
            Transform::from_translation(pos(field.position, height * 0.5))
                .with_scale(Vec3::new(0.9, height, 0.9)),
            &mut |commands| {
                commands
                    .spawn((
                        Mesh3d(art.cylinder.clone()),
                        MeshMaterial3d(colour.clone()),
                        Transform::default(),
                    ))
                    .id()
            },
        );
    }
    for vehicle in &game.world.vehicles {
        let height = match vehicle.domain {
            VehicleDomain::Land => 0.7,
            VehicleDomain::Sea => 0.4,
            VehicleDomain::Air => 4.5,
        };
        set(
            (3, vehicle.id),
            Transform::from_translation(pos(vehicle.position, height))
                .with_rotation(Quat::from_rotation_y(-vehicle.heading))
                .with_scale(Vec3::splat(vehicle.radius())),
            &mut |commands| spawn_vehicle(commands, &art, vehicle),
        );
        if game.world.selected.contains(&vehicle.id) {
            set(
                (4, vehicle.id),
                Transform::from_translation(pos(vehicle.position, 0.14))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(1.65)),
                &mut |commands| {
                    commands
                        .spawn((
                            Mesh3d(art.ring.clone()),
                            MeshMaterial3d(art.selected.clone()),
                            Transform::default(),
                        ))
                        .id()
                },
            );
        }
    }
    if let Some(city) = game.world.cities.iter().find(|city| city.id == game.city) {
        set(
            (6, 0),
            Transform::from_translation(pos(city.position, 0.16))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(7.)),
            &mut |commands| {
                commands
                    .spawn((
                        Mesh3d(art.ring.clone()),
                        MeshMaterial3d(art.selected.clone()),
                        Transform::default(),
                    ))
                    .id()
            },
        );
        set(
            (7, 0),
            Transform::from_translation(pos(city.building_position(game.slot), 0.15))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(1.)),
            &mut |commands| {
                commands
                    .spawn((
                        Mesh3d(art.ring.clone()),
                        MeshMaterial3d(art.selected.clone()),
                        Transform::default(),
                    ))
                    .id()
            },
        );
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
fn camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    mut rig: ResMut<CameraRig>,
    mut cameras: Query<&mut Transform, With<RtsCamera>>,
) {
    let dt = time.delta_secs().min(0.1);
    let axis =
        |positive, negative| (keys.pressed(positive) as i8 - keys.pressed(negative) as i8) as f32;
    rig.center.x = (rig.center.x + axis(KeyCode::KeyD, KeyCode::KeyA) * dt * 35.).clamp(
        -game.world.config.half_extents.x,
        game.world.config.half_extents.x,
    );
    rig.center.y = (rig.center.y + axis(KeyCode::KeyS, KeyCode::KeyW) * dt * 35.).clamp(
        -game.world.config.half_extents.y,
        game.world.config.half_extents.y,
    );
    rig.yaw += axis(KeyCode::KeyE, KeyCode::KeyQ) * dt;
    rig.distance =
        (rig.distance + axis(KeyCode::Minus, KeyCode::Equal) * dt * 65.).clamp(30., 190.);
    let target = Vec3::new(rig.center.x, 0., rig.center.y);
    let offset = Vec3::new(rig.yaw.cos() * 0.62, 0.78, rig.yaw.sin() * 0.62) * rig.distance;
    for mut camera in &mut cameras {
        *camera = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
    }
}
fn hud(
    game: Res<Game>,
    mut texts: ParamSet<(
        Query<&mut Text, With<Header>>,
        Query<&mut Text, With<Details>>,
        Query<&mut Text, With<Notice>>,
        Query<&mut Text, With<Banner>>,
    )>,
) {
    let world = &game.world;
    let income = world
        .cities
        .iter()
        .filter(|city| city.controller() == 0)
        .map(|city| city.income_per_second())
        .sum::<f64>();
    for mut text in &mut texts.p0() {
        text.0=format!("CIVILIZATION  |  Credits {:.0}  (+{:.1}/s cities)  |  Control {} / {} cities\nSelected {} vehicles   Production {:?}   Demo style {:?}   Tick {}",world.currency(),income,world.cities.iter().filter(|city|city.controller()==0).count(),world.cities.len(),world.selected.len(),game.domain,game.style,world.tick);
    }
    for mut text in &mut texts.p1() {
        text.0 = if let Some(city) = world.cities.iter().find(|city| city.id == game.city) {
            format!("{}  |  Nation {}\n{}\n\nHealth {:.0} / {:.0}\nPopulation {}\nIndustry {:.2}  Happiness {:.0}%\nIncome {:.1} credits/s\nFaith {:.0}%  (nation {})\nTrade {:.0}%  (nation {})\nPurchase {:.0} credits\n\nBuilding slot {}: {}\nQueued {} / 6\n\nProduce {:?}:\nZ military {:.0}\nX religious {:.0}\nC economic {:.0}\n\nB house 50 | N factory 80\nM entertainment 60",city.name,city.controller(),if city.allied_to==Some(0){"Allied city"}else if city.controller()==0{"Your city"}else{"Opposing city"},city.health,city.max_health,city.population,city.industry,city.happiness*100.,city.income_per_second(),city.faith,city.influence_owner,city.trade,city.trade_owner,purchase_cost(city),game.slot,city.buildings.iter().find(|building|building.slot==game.slot).map(|building|format!("{:?}",building.kind)).unwrap_or_else(||"empty".into()),city.production.len(),game.domain,vehicle_cost(VehicleRole::Military,game.domain),vehicle_cost(VehicleRole::Religious,game.domain),vehicle_cost(VehicleRole::Economic,game.domain))
        } else {
            String::new()
        };
        let selected = world
            .vehicles
            .iter()
            .filter(|vehicle| world.selected.contains(&vehicle.id))
            .take(3)
            .map(|vehicle| {
                format!(
                    "{:?}/{:?} {:.0}/{:.0} HP",
                    vehicle.role, vehicle.domain, vehicle.health, vehicle.max_health
                )
            })
            .collect::<Vec<_>>();
        if !selected.is_empty() {
            text.0
                .push_str(&format!("\n\nSelected units\n{}", selected.join("\n")));
        }
    }
    for mut text in &mut texts.p2() {
        text.0 = if game.notice.is_empty() {
            format!(
                "Seed {} | {} finite geysers | AI nations operate real production and orders",
                world.seed(),
                world.geysers.len()
            )
        } else {
            game.notice.chars().take(145).collect()
        };
    }
    for mut text in &mut texts.p3() {
        text.0 = if game.paused {
            "PAUSED\nP to continue".into()
        } else {
            match world.state {
                CivilizationState::Playing => String::new(),
                CivilizationState::SpaceReady => {
                    "SPACE ELIGIBLE\nEvery city is controlled or allied\nR starts a new campaign"
                        .into()
                }
                CivilizationState::Defeated => "NATION DEFEATED\nR to restart".into(),
            }
        };
    }
}
fn capture(mut commands: Commands, mut game: ResMut<Game>) {
    game.frames += 1;
    if game.captured.is_none()
        && game.elapsed >= game.demo.unwrap_or(3.)
        && (game.demo.is_some() || game.shot.is_some())
    {
        if let Some(path) = &game.shot {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path.clone()));
        }
        game.captured = Some(game.frames);
        println!("CIVILIZATION_RESULT state={:?} tick={} credits={:.3} controlled={} vehicles={} buildings={} faith={:.1} trade={:.1}",game.world.state,game.world.tick,game.world.currency(),game.world.cities.iter().filter(|city|city.controller()==0).count(),game.world.vehicles.len(),game.world.cities.iter().map(|city|city.buildings.len()).sum::<usize>(),game.world.cities.iter().map(|city|city.faith).sum::<f32>(),game.world.cities.iter().map(|city|city.trade).sum::<f32>());
    }
    if let Some(frame) = game.captured {
        if game.frames > frame + 10 && game.shot.as_ref().is_none_or(|path| path.exists()) {
            std::process::exit(0);
        }
        if game.frames > frame + 600 {
            eprintln!("Civilization screenshot capture failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn actual_keyboard_app_builds_produces_pauses_saves_loads_and_restarts() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!(
            "rustspore-civ-controls-{}-{nonce}",
            std::process::id()
        ));
        let path = folder.join("civilization.rsp");
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(ButtonInput::<KeyCode>::default())
            .insert_resource(Game::new(19, path.clone()))
            .add_systems(Update, drive);
        step(&mut app, 0., &[KeyCode::KeyB]);
        assert_eq!(
            app.world().resource::<Game>().world.cities[0]
                .buildings
                .len(),
            4
        );
        step(&mut app, 0., &[KeyCode::Digit3, KeyCode::KeyX]);
        assert_eq!(
            app.world().resource::<Game>().world.cities[0]
                .production
                .len(),
            1
        );
        step(&mut app, 0.1, &[]);
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, 0.1, &[KeyCode::KeyP]);
        assert_eq!(app.world().resource::<Game>().world, before);
        step(&mut app, 0., &[KeyCode::F5]);
        let saved = std::fs::read(&path).unwrap();
        step(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<Game>().world.cities[0]
                .buildings
                .len(),
            3
        );
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
        std::fs::write(&path, b"broken").unwrap();
        let before = app.world().resource::<Game>().world.clone();
        step(&mut app, 0., &[KeyCode::F9]);
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
    fn live_pointer_helper_selects_shift_groups_and_context_issues_real_orders() {
        let mut game = Game::new(42, PathBuf::from("unused"));
        let home = game.city;
        game.world
            .issue_command(CivilizationCommand::Produce {
                city: home,
                role: VehicleRole::Military,
                domain: VehicleDomain::Air,
            })
            .unwrap();
        game.world
            .issue_command(CivilizationCommand::Produce {
                city: home,
                role: VehicleRole::Religious,
                domain: VehicleDomain::Air,
            })
            .unwrap();
        for _ in 0..240 {
            game.world.step();
        }
        let ids = game
            .world
            .vehicles
            .iter()
            .filter(|vehicle| vehicle.owner == 0)
            .map(|vehicle| vehicle.id)
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 2);
        let a = game
            .world
            .vehicles
            .iter()
            .find(|vehicle| vehicle.id == ids[0])
            .unwrap()
            .position;
        pointer(&mut game, a, false, false);
        assert_eq!(game.world.selected.len(), 1);
        pointer(
            &mut game,
            a.add(CivilizationVec2::new(-12., 10.)),
            false,
            true,
        );
        for _ in 0..70 {
            game.world.step();
        }
        let second = game
            .world
            .vehicles
            .iter()
            .find(|vehicle| vehicle.id == ids[1])
            .unwrap()
            .position;
        pointer(&mut game, second, true, false);
        assert_eq!(game.world.selected, ids);
        let target = game.world.cities[1].position;
        pointer(&mut game, target, false, true);
        assert!(game
            .world
            .vehicles
            .iter()
            .any(|vehicle| vehicle.owner == 0
                && matches!(vehicle.order, VehicleOrder::Attack { .. })));
        assert!(game
            .world
            .vehicles
            .iter()
            .any(|vehicle| vehicle.owner == 0
                && matches!(vehicle.order, VehicleOrder::Convert { .. })));
        assert_eq!(game.world.selected, ids);
        pointer(&mut game, a, true, false);
        assert_eq!(game.world.selected.len(), 1);
    }
    #[test]
    fn demo_uses_paid_production_movement_and_real_conversion_with_opponents_active() {
        let mut game = Game::new(14, PathBuf::from("unused"));
        let initial = game.world.currency();
        for _ in 0..900 {
            demo_orders(&mut game);
            game.world.advance(0.1);
            if game.world.state != CivilizationState::Playing {
                break;
            }
        }
        assert!(game.world.vehicles.iter().any(|vehicle| vehicle.owner == 0));
        assert!(game
            .world
            .cities
            .iter()
            .any(|city| city.faith > 0. || city.allied_to == Some(0)));
        assert!(game.world.factions.iter().skip(1).all(|faction| faction.ai));
        assert!(game.world.tick > 600);
        assert!(game.world.currency() != initial);
        game.world.snapshot().to_bytes().unwrap();
    }
}
