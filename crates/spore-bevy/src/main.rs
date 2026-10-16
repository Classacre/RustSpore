//! The `rustspore` binary.
//!
//! Starts a minimal Bevy app. If a path is passed as the first command line
//! argument it is treated as a DBPF package, opened, and its header is printed
//! to stdout before the window opens. With no argument the app just runs an
//! empty 3D scene.

use bevy::prelude::*;
use spore_core::GameStage;

fn main() {
    if let Some(path) = std::env::args().nth(1) {
        print_package(&path);
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustSpore".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

/// Open a package and report its header, index and resource type histogram.
///
/// A failure here is reported and does not abort the app: the scene still runs.
fn print_package(path: &str) {
    use std::collections::HashMap;

    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return;
        }
    };

    let h = file.header();
    println!("package            : {}", file.path().display());
    println!("  file size        : {} bytes", file.len());
    println!("  magic            : {}", String::from_utf8_lossy(&h.magic));
    println!(
        "  u32 at 0x04      : {} (name uncertain, see docs/dbpf-format.md section 2.1)",
        h.version_0x04
    );
    println!("  index entry count: {}", h.index_entry_count);
    println!("  index offset     : {} (0x{:X})", h.index_offset, h.index_offset);
    println!("  index size       : {}", h.index_size);
    println!(
        "  index end        : {} vs file size {} => {}",
        h.index_end(),
        file.len(),
        if h.index_end() == file.len() as u64 {
            "consistent"
        } else {
            "MISMATCH"
        }
    );

    let entries = file.entries();
    let coded = entries.iter().filter(|e| e.is_coded()).count();
    println!(
        "  resources        : {} ({} coded, {} raw)",
        entries.len(),
        coded,
        entries.len() - coded
    );

    let mut histogram: HashMap<u32, usize> = HashMap::new();
    for entry in entries {
        *histogram.entry(entry.key.type_id).or_default() += 1;
    }
    let mut ranked: Vec<(u32, usize)> = histogram.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    println!("  top resource types:");
    for (type_id, count) in ranked.into_iter().take(10) {
        println!("    0x{type_id:08X}  {count:>7}");
    }
}

fn setup(mut commands: Commands) {
    // Camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // A single directional light so the empty scene is not pitch black.
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.6, 0.0)),
    ));

    // Text overlay.
    commands.spawn((
        Text::new(overlay_text()),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn overlay_text() -> String {
    let stages = GameStage::ALL
        .iter()
        .map(|stage| stage.name())
        .collect::<Vec<_>>()
        .join(", ");
    format!("RustSpore\nstages: {stages}")
}
