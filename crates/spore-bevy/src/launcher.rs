//! Entry screen for the currently available gameplay and creation modes.
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use std::path::PathBuf;

#[derive(Resource)]
struct Menu {
    notice: String,
    next: Option<&'static [&'static str]>,
    quit: bool,
    shot: Option<PathBuf>,
    frames: u32,
    captured: Option<u32>,
}
#[derive(Component, Clone, Copy)]
enum Action {
    Run(&'static [&'static str]),
    Quit,
}
#[derive(Component)]
struct Notice;

pub fn run(args: &[String]) -> Result<(), String> {
    let shot = match args {
        [] => None,
        [flag, path] if flag == "--shot" => {
            let path = PathBuf::from(path);
            if path.exists() {
                return Err("menu screenshot path exists".into());
            }
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            Some(path)
        }
        _ => return Err("usage: --menu [--shot <new PNG>]".into()),
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore".into(),
                resolution: (1280, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.025, 0.055, 0.070)))
        .insert_resource(Menu {
            notice: "Choose a world to explore or create something of your own.".into(),
            next: None,
            quit: false,
            shot,
            frames: 0,
            captured: None,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (buttons, notice, capture, launch).chain())
        .run();
    Ok(())
}
fn text(parent: &mut ChildSpawnerCommands, label: &str, size: f32, color: Color) {
    parent.spawn((
        Text::new(label),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    ));
}
fn button(parent: &mut ChildSpawnerCommands, label: &str, action: Action) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(px(20), px(13)),
                border_radius: BorderRadius::all(px(7)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.11, 0.30, 0.28)),
            action,
        ))
        .with_children(|p| text(p, label, 16., Color::srgb(0.85, 0.98, 0.91)));
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Node {width:percent(100),height:percent(100),padding:UiRect::axes(px(54),px(36)),flex_direction:FlexDirection::Column,row_gap:px(22),..default()}).with_children(|root|{
        root.spawn(Node {justify_content:JustifyContent::SpaceBetween,align_items:AlignItems::Center,..default()}).with_children(|header|{
            header.spawn(Node {flex_direction:FlexDirection::Column,row_gap:px(6),..default()}).with_children(|title|{text(title,"RUSTSPORE",36.,Color::srgb(0.63,0.94,0.82));text(title,"A world taking shape",17.,Color::srgb(0.55,0.72,0.74));});
            text(header,"LOCAL PROTOTYPE",13.,Color::srgb(0.75,0.76,0.52));
        });
        root.spawn(Node {width:percent(100),height:px(245),column_gap:px(20),..default()}).with_children(|row|{
            for (title,description,args) in [
                ("01  CELL","Swim, feed and grow.\nSurvive the tide pool.",&["--cell"][..]),
                ("02  CREATURE","Find food, discover parts\nand make allies or rivals.",&["--creature-game"][..]),
                ("03  TRIBE","Gather food, build huts\nand guide your tribe.",&["--tribe"][..]),
            ] {
                row.spawn((Node {flex_grow:1.,flex_basis:px(0),padding:UiRect::all(px(23)),flex_direction:FlexDirection::Column,justify_content:JustifyContent::SpaceBetween,border_radius:BorderRadius::all(px(12)),..default()},BackgroundColor(Color::srgb(0.045,0.11,0.13)))).with_children(|card|{
                    text(card,title,23.,Color::srgb(0.84,0.92,0.79));text(card,description,15.,Color::srgb(0.58,0.75,0.75));button(card,"Start exploring",Action::Run(args));
                });
            }
        });
        root.spawn(Node {flex_direction:FlexDirection::Column,row_gap:px(13),..default()}).with_children(|section|{
            text(section,"CREATION STUDIO",20.,Color::srgb(0.83,0.89,0.76));
            section.spawn(Node {column_gap:px(12),..default()}).with_children(|row|{
                for (label,args) in [("Cell",&["--editor","cell"][..]),("Creature",&["--editor","creature"][..]),("Building",&["--editor","building"][..]),("Vehicle",&["--editor","vehicle"][..]),("Adventure",&["--editor","adventure"][..])]{button(row,label,Action::Run(args));}
            });
        });
        root.spawn(Node {flex_direction:FlexDirection::Column,row_gap:px(8),..default()}).with_children(|p|{
            text(p,"Civilization and Space are in development.",14.,Color::srgb(0.58,0.71,0.71));
            text(p,"These playable foundations use original art and rules. The full campaign and source-content fidelity remain in progress.",13.,Color::srgb(0.48,0.65,0.67));
        });
        root.spawn(Node {margin:UiRect::top(Val::Auto),align_items:AlignItems::Center,justify_content:JustifyContent::SpaceBetween,..default()}).with_children(|footer|{
            footer.spawn((Text::new(""),TextFont{font_size:14.,..default()},TextColor(Color::srgb(0.85,0.81,0.60)),Notice));button(footer,"Quit",Action::Quit);
        });
    });
}
fn buttons(
    mut menu: ResMut<Menu>,
    mut query: Query<(&Interaction, &Action, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, action, mut color) in &mut query {
        color.0 = if *interaction == Interaction::Hovered {
            Color::srgb(0.16, 0.42, 0.36)
        } else {
            Color::srgb(0.11, 0.30, 0.28)
        };
        if *interaction == Interaction::Pressed {
            match action {
                Action::Run(args) => {
                    menu.next = Some(args);
                    menu.notice = "Opening your world...".into();
                }
                Action::Quit => menu.quit = true,
            }
        }
    }
}
fn notice(menu: Res<Menu>, mut text: Query<&mut Text, With<Notice>>) {
    for mut t in &mut text {
        t.0 = menu.notice.clone();
    }
}
fn launch(
    mut menu: ResMut<Menu>,
    mut exit: MessageWriter<AppExit>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if menu.quit || keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }
    if let Some(args) = menu.next.take() {
        match std::env::current_exe().and_then(|exe| {
            std::process::Command::new(exe)
                .arg("--experimental")
                .args(args)
                .stdin(std::process::Stdio::null())
                .spawn()
        }) {
            Ok(_) => {
                exit.write(AppExit::Success);
            }
            Err(e) => menu.notice = format!("Could not open this world: {e}"),
        }
    }
}
fn capture(time: Res<Time>, mut commands: Commands, mut menu: ResMut<Menu>) {
    menu.frames += 1;
    if menu.captured.is_none() && menu.shot.is_some() && time.elapsed_secs() >= 3. {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(menu.shot.as_ref().unwrap().clone()));
        menu.captured = Some(menu.frames);
    }
    if let Some(frame) = menu.captured {
        if menu.frames > frame + 10 && menu.shot.as_ref().is_some_and(|p| p.exists()) {
            std::process::exit(0);
        }
        if menu.frames > frame + 600 {
            eprintln!("menu capture failed");
            std::process::exit(1);
        }
    }
}
