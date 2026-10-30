//! The `rustspore` binary.
//!
//! Modes, all taking a DBPF package path:
//!
//! ```text
//! rustspore <package>              report the package, then run the Bevy scene
//! rustspore --report <package>     report the package and exit
//! rustspore --verify <package>     decompress every coded resource, check each
//!                                  against the index's declared size, then exit
//! rustspore                        report source reconstruction status
//! ```
//!
//! `--verify` exists because a decoder that has never been pointed at real data is
//! an untested decoder. It exits non-zero if any resource fails.

use bevy::prelude::*;
use spore_core::GameStage;

mod animation_viewer;
mod assets;
mod cell_game;
mod civilization_game;
mod creature_game;
mod editor;
mod launcher;
#[cfg(all(windows, feature = "native_renderer"))]
mod native_cell_renderer;
mod offscreen;
mod source_cell;
mod source_creature;
mod tribe_game;
mod viewer;

/// CLI arguments may come from line-oriented surveys, including CRLF output.
/// Trim surrounding whitespace, but never turn an invalid identifier into zero.
fn parse_hex_arg(text: &str, name: &str) -> Result<u64, String> {
    let digits = text.trim();
    let digits = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
        .unwrap_or(digits);
    u64::from_str_radix(digits, 16)
        .map_err(|_| format!("invalid {name} {text:?}: expected a hexadecimal u64"))
}

fn parse_type_arg(text: &str) -> Result<u32, String> {
    let invalid = || format!("invalid type {text:?}: expected a hexadecimal u32");
    let value = parse_hex_arg(text, "type").map_err(|_| invalid())?;
    u32::try_from(value).map_err(|_| invalid())
}

fn parse_mesh_index(text: &str) -> Result<usize, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("invalid mesh index {text:?}: expected a non-negative integer"))
}

fn parse_shot_target(text: &str) -> Result<viewer::Target, String> {
    let value = text.trim();
    // The documented bare instance form selects a prop.
    let (kind, hex) = value.split_once('=').unwrap_or(("prop", value));
    let instance = parse_hex_arg(hex, "instance")?;
    match kind {
        "prop" => Ok(viewer::Target::Prop(instance)),
        "creature" => Ok(viewer::Target::Creature(instance)),
        _ => Err(format!(
            "invalid target {text:?}: expected prop=<hex> or creature=<hex>"
        )),
    }
}

fn cli_value<T>(result: Result<T, String>) -> T {
    result.unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(2);
    })
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let experimental = args.first().is_some_and(|arg| arg == "--experimental");
    if experimental {
        args.remove(0);
    }
    if !experimental
        && matches!(
            args.first().map(String::as_str),
            Some(
                "--menu" | "--cell" | "--creature-game" | "--tribe" | "--civilization" | "--editor"
            )
        )
    {
        eprintln!("This mode uses experimental art/rules and does not reproduce SPORE.\nUse --experimental before the mode to run engineering prototypes.\nUse --viewer/--animate for source-asset inspection. Source-backed gameplay is still in development.");
        std::process::exit(2);
    }

    match args.first().map(String::as_str) {
        None => {
            if experimental {
                cli_value(launcher::run(&[]));
            } else {
                println!("RustSpore: source-asset reconstruction in progress.\nThe faithful game is not yet implemented.\nUse --viewer <package> or --animate <package> <RW4 hex> to inspect installed SPORE assets.\nExperimental simulations require --experimental. Use --help for commands.");
            }
            return;
        }
        Some("--menu") => {
            cli_value(launcher::run(&args[1..]));
            return;
        }
        Some("--source-cell") => {
            if let Err(err) = source_cell::run(&args[1..]) {
                eprintln!("source Cell: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--source-creature") => {
            if let Err(err) = source_creature::run(&args[1..]) {
                eprintln!("source creature: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--help") | Some("-h") => {
            println!("Saved creature artwork with original programs: --source-creature [--install <SPORE dir>] --creature <hex> --shot <new PNG> [--coverage <new PNG> for 40616200A1C46DF2]");
            println!("Source Cell artwork with original programs: --source-cell [--install <SPORE dir>] [--cell <hex>] --shot <new PNG>\nInspection PBR comparison: add --pbr-inspection\nJawhead synthetic position-only diagnostic: add --diagnostic-skin-translation (normals/tangents unchanged; not original animation playback)\nCorpus check: --source-cell [--install <SPORE dir>] --audit");
            println!("RustSpore source-data tools\n  --animate <package> <RW4 hex> [--pose bind|seconds] [--shot <new PNG>]\n  --viewer <package>\n  --report <package>\n  --verify <package>\nEngineering prototypes (prototype rules and default procedural art; each requires --experimental)\n  --experimental --menu [--shot <new PNG>]\n  --experimental --cell [--seed <n>] [--diet herbivore|carnivore|omnivore] [--creation <file.rsc>] [--source-cell-install <dir> --source-cell-id <hex> | --source-creature-install <dir> --source-creature-instance <16-hex>] [--offscreen --shot <new PNG>]\n  --experimental --creature-game [--seed <n>] [--creation <file.rsc>] [--source-creature-install <dir> --source-creature-id <hex>]\n  --experimental --tribe [--seed <n>]\n  --experimental --civilization [--seed <n>] [--role military|religious|economic] [--demo <seconds>] [--shot <new PNG>]\n  --experimental --editor cell|creature|building|vehicle|adventure [--load <file.rsc>] [--export <new file.rsc>]\nSee README.md for source-data tools, controls and current milestone limits.");
            return;
        }
        Some("--tribe") => {
            if let Err(err) = tribe_game::run(&args[1..]) {
                eprintln!("tribe: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--civilization") => {
            if let Err(err) = civilization_game::run(&args[1..]) {
                eprintln!("civilization: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--editor") => {
            if let Err(err) = editor::run(&args[1..]) {
                eprintln!("editor: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--creature-game") => {
            if let Err(err) = creature_game::run(&args[1..]) {
                eprintln!("creature game: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--animate") => {
            if let Err(err) = animation_viewer::run(&args[1..]) {
                eprintln!("animation: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--cell") => {
            if let Err(err) = cell_game::run(&args[1..]) {
                eprintln!("cell: {err}");
                std::process::exit(2);
            }
            return;
        }
        Some("--verify") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --verify <package>");
                std::process::exit(2);
            };
            std::process::exit(verify_package(path));
        }
        Some("--report") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --report <package>");
                std::process::exit(2);
            };
            print_package(path);
            return;
        }
        Some("--types") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --types <package>");
                std::process::exit(2);
            };
            print_types(path);
            return;
        }
        Some("--raster-digest") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --raster-digest <package>");
                std::process::exit(2);
            };
            std::process::exit(raster_digest(path));
        }
        Some("--gmsh-digest") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --gmsh-digest <package>");
                std::process::exit(2);
            };
            std::process::exit(gmsh_digest(path));
        }
        Some("--mesh-selftest") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --mesh-selftest <package>");
                std::process::exit(2);
            };
            std::process::exit(mesh_selftest(path));
        }
        Some("--gmdl-elements") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --gmdl-elements <package>");
                std::process::exit(2);
            };
            std::process::exit(gmdl_elements(path));
        }
        Some("--gmdl-info") => {
            let (Some(path), Some(inst)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: rustspore --gmdl-info <package> <instance-hex>");
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            std::process::exit(gmdl_info(path, instance));
        }
        Some("--rw4-info") => {
            let (Some(path), Some(inst)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: rustspore --rw4-info <package> <instance-hex>");
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            std::process::exit(rw4_info(path, instance));
        }
        Some("--rw4-census") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --rw4-census <package>");
                std::process::exit(2);
            };
            std::process::exit(rw4_census(path));
        }
        Some("--creature-overlap") => {
            let (Some(path), Some(inst)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: rustspore --creature-overlap <package> <instance-hex>");
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            std::process::exit(creature_overlap(path, instance));
        }
        Some("--creature-parts") => {
            let (Some(path), Some(inst)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: rustspore --creature-parts <package> <instance-hex>");
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            std::process::exit(creature_parts(path, instance));
        }
        Some("--creature-info") => {
            let (Some(path), Some(inst)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: rustspore --creature-info <package> <instance-hex>");
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            std::process::exit(creature_info(path, instance));
        }
        Some("--creature-skin") => {
            let (Some(path), Some(inst)) = (args.get(1), args.get(2)) else {
                eprintln!(
                    "usage: rustspore --creature-skin <package> <instance-hex> [out.png] [--force]"
                );
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            let force = args.iter().any(|a| a == "--force");
            let out = args
                .get(3)
                .map(|s| s.as_str())
                .filter(|s| !s.starts_with("--"));
            std::process::exit(creature_skin(path, instance, out, force));
        }
        Some("--locate") => {
            let Some(inst) = args.get(1) else {
                eprintln!("usage: rustspore --locate <instance-hex> [type-hex]");
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            let type_id = args
                .get(2)
                .map(|t| cli_value(parse_type_arg(t)))
                .unwrap_or(0);
            std::process::exit(locate(instance, type_id));
        }
        Some("--prop-models") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --prop-models <package>");
                std::process::exit(2);
            };
            std::process::exit(prop_models(path));
        }
        Some("--rw4-texture-png") => {
            let (Some(path), Some(inst), Some(out)) = (args.get(1), args.get(2), args.get(3))
            else {
                eprintln!("usage: rustspore --rw4-texture-png <package> <instance-hex> <out.png>");
                std::process::exit(2);
            };
            let instance = cli_value(parse_hex_arg(inst, "instance"));
            std::process::exit(rw4_texture_png(path, instance, out));
        }
        Some("--rw4-textures") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --rw4-textures <package>");
                std::process::exit(2);
            };
            std::process::exit(rw4_textures(path));
        }
        Some("--viewer") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rustspore --viewer <package>");
                std::process::exit(2);
            };
            match viewer::run(path, None, 0, false, None) {
                Ok(()) => return,
                Err(err) => {
                    eprintln!("viewer: {err}");
                    std::process::exit(2);
                }
            }
        }
        Some("--shot") => {
            let (Some(path), Some(out)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: rustspore --shot <package> <out.png> [mesh-index] [checker] [prop=<hex>|creature=<hex>]");
                std::process::exit(2);
            };
            let start = args
                .get(3)
                .map(|s| cli_value(parse_mesh_index(s)))
                .unwrap_or(0);
            // A fourth argument of "checker" swaps the texture for a test pattern.
            let checker = args.get(4).map(|a| a == "checker").unwrap_or(false);
            let target = args.get(5).map(|a| cli_value(parse_shot_target(a)));
            match viewer::run(
                path,
                Some(std::path::PathBuf::from(out)),
                start,
                checker,
                target,
            ) {
                Ok(()) => return,
                Err(err) => {
                    eprintln!("viewer: {err}");
                    std::process::exit(2);
                }
            }
        }
        Some(path) if !path.starts_with("--") => print_package(path),
        Some(option) => {
            eprintln!("unknown option {option}; use --help");
            std::process::exit(2);
        }
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

/// Decompress every coded resource and check the result against the size the
/// index declares. Returns a process exit code.
///
/// This is the real-data test for the RefPack decoder: the Python validator
/// established the codec against 800 resources, and this checks the Rust port
/// against every coded resource in one package.
/// How close an assembled creature is to the merged model, as nearest-neighbour
/// distances between their vertices.
///
/// Rendering both and comparing pixels is not a fair test: the viewer normalises each
/// mesh by its own radius, so the same shape lands at different sizes on screen. This
/// compares the geometry directly and reports distances as a fraction of the merged
/// model's diagonal, so 0.02 means "within 2% of the model's size" and 0.5 means
/// "half the model away from anything".
fn creature_overlap(path: &str, instance: u64) -> i32 {
    let mut source = match assets::AssetSource::open(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    if let Some(set) = package_set() {
        source = source.with_set(set);
    }
    let (_, assembled) = match source.creature(instance) {
        Ok(v) => v,
        Err(err) => {
            println!("rig   : {err}");
            return 1;
        }
    };
    let (_, merged) = match source.prop_mesh(instance) {
        Ok(v) => v,
        Err(err) => {
            println!("merged: {err}");
            return 1;
        }
    };

    let diagonal = match merged.bounds() {
        Some((min, max)) => {
            let d = [
                (max.x - min.x) as f64,
                (max.y - min.y) as f64,
                (max.z - min.z) as f64,
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        }
        None => 0.0,
    };
    if diagonal <= 0.0 {
        println!("merged model has no extent");
        return 1;
    }

    let nn = |from: &[spore_core::Vec3], to: &[spore_core::Vec3]| -> (f64, f64) {
        let mut sum = 0.0f64;
        let mut worst = 0.0f64;
        for a in from {
            let mut best = f64::INFINITY;
            for b in to {
                let dx = (a.x - b.x) as f64;
                let dy = (a.y - b.y) as f64;
                let dz = (a.z - b.z) as f64;
                let d = dx * dx + dy * dy + dz * dz;
                if d < best {
                    best = d;
                }
            }
            let d = best.sqrt();
            sum += d;
            if d > worst {
                worst = d;
            }
        }
        (sum / from.len() as f64, worst)
    };

    let (mean_a, max_a) = nn(&assembled.positions, &merged.positions);
    let (mean_m, max_m) = nn(&merged.positions, &assembled.positions);
    println!("merged diagonal      : {diagonal:.3}");
    println!(
        "assembly -> merged   : mean {:.3} ({:.1}% of diagonal), worst {:.3}",
        mean_a,
        100.0 * mean_a / diagonal,
        max_a
    );
    println!(
        "merged -> assembly   : mean {:.3} ({:.1}% of diagonal), worst {:.3}",
        mean_m,
        100.0 * mean_m / diagonal,
        max_m
    );
    0
}

/// Per-part table: the rig row's bounding box next to the model's own bounds.
///
/// The row carries a bounding box at offset 20 and five floats at 108 that have no
/// established meaning. If the ratio of the row box to the model's own extent is
/// constant across parts, and matches one of those floats, the scale is identified
/// without guessing from renders.
fn creature_parts(path: &str, instance: u64) -> i32 {
    let mut source = match assets::AssetSource::open(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    if let Some(set) = package_set() {
        source = source.with_set(set);
    }
    let Some(data) = source.creature_raw(instance) else {
        eprintln!("no creature {instance:016X}");
        return 1;
    };
    let Ok(parsed) = spore_core::creature_data::parse(&data) else {
        eprintln!("creaturedata parse failed");
        return 1;
    };

    println!(
        "{:>3} {:>16} {:>9} {:>9} {:>7}  {}",
        "idx", "part", "row_ext", "model_ext", "ratio", "floats at 108"
    );
    for part in &parsed.parts {
        let row_ext = [
            part.bounds_max[0] - part.bounds_min[0],
            part.bounds_max[1] - part.bounds_min[1],
            part.bounds_max[2] - part.bounds_min[2],
        ];
        let row_max = row_ext.iter().cloned().fold(0.0f32, f32::max);
        let model_max = match source.prop_mesh(part.part_instance) {
            Ok((_, mesh)) => match mesh.bounds() {
                Some((min, max)) => [max.x - min.x, max.y - min.y, max.z - min.z]
                    .iter()
                    .cloned()
                    .fold(0.0f32, f32::max),
                None => f32::NAN,
            },
            Err(_) => f32::NAN,
        };
        // Where the row's own box lands after its transform. A part whose centre is
        // far from the others is either genuinely an appendage or misplaced.
        let c = [
            (part.bounds_min[0] + part.bounds_max[0]) * 0.5,
            (part.bounds_min[1] + part.bounds_max[1]) * 0.5,
            (part.bounds_min[2] + part.bounds_max[2]) * 0.5,
        ];
        let r = &part.rotation;
        let world = [
            r[0][0] * c[0] + r[0][1] * c[1] + r[0][2] * c[2] + part.translation[0],
            r[1][0] * c[0] + r[1][1] * c[1] + r[1][2] * c[2] + part.translation[1],
            r[2][0] * c[0] + r[2][1] * c[1] + r[2][2] * c[2] + part.translation[2],
        ];
        println!(
            "{:>3} {:>16X} {:>9.3} {:>9.3} {:>7.3}  {:.3} {:.3} {:.3} {:.3} {:.3}  world ({:.2}, {:.2}, {:.2})",
            part.index,
            part.part_instance,
            row_max,
            model_max,
            if model_max.is_finite() && model_max != 0.0 {
                row_max / model_max
            } else {
                f32::NAN
            },
            part.reserved_floats[0],
            part.reserved_floats[1],
            part.reserved_floats[2],
            part.reserved_floats[3],
            part.reserved_floats[4],
            world[0],
            world[1],
            world[2],
        );
    }
    0
}

/// Compare an assembled creature against the merged model its own prop names.
///
/// A creature with a prop model has two independent descriptions of the same thing:
/// the rig, and the model the game ships pre-merged. Their bounding boxes should
/// agree if the rig transforms are applied correctly. If the assembly is much
/// smaller, the transforms are local and need a parent chain.
fn creature_info(path: &str, instance: u64) -> i32 {
    let mut source = match assets::AssetSource::open(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    if let Some(set) = package_set() {
        source = source.with_set(set);
    }

    match source.creature(instance) {
        Ok((assembly, mesh)) => {
            println!(
                "rig   : {} of {} parts, {} missing, {} failed",
                assembly.parts_drawn,
                assembly.parts_total,
                assembly.parts_missing,
                assembly.parts_failed
            );
            println!(
                "        {} vertices, {} triangles",
                assembly.vertices, assembly.triangles
            );
            if let Some((min, max)) = mesh.bounds() {
                println!(
                    "        bounds x {:.3}..{:.3}  y {:.3}..{:.3}  z {:.3}..{:.3}  radius {:.3}",
                    min.x,
                    max.x,
                    min.y,
                    max.y,
                    min.z,
                    max.z,
                    mesh.radius()
                );
            }
        }
        Err(err) => println!("rig   : {err}"),
    }

    match source.creature_palette(instance) {
        Some(p) => println!(
            "palette: {:.2} {:.2} {:.2}   {:.2} {:.2} {:.2}   {:.2} {:.2} {:.2}",
            p[0][0], p[0][1], p[0][2], p[1][0], p[1][1], p[1][2], p[2][0], p[2][1], p[2][2]
        ),
        None => println!("palette: none"),
    }

    match source.prop_mesh(instance) {
        Ok((resolved, mesh)) => {
            let (t, i) = resolved.model;
            println!(
                "merged: {t:08X}:{i:016X}, {} vertices",
                mesh.positions.len()
            );
            if let Some((min, max)) = mesh.bounds() {
                println!(
                    "        bounds x {:.3}..{:.3}  y {:.3}..{:.3}  z {:.3}..{:.3}  radius {:.3}",
                    min.x,
                    max.x,
                    min.y,
                    max.y,
                    min.z,
                    max.z,
                    mesh.radius()
                );
            }
        }
        Err(err) => println!("merged: {err}"),
    }
    0
}

/// Report and optionally write a creature's diffuse map.
///
/// The point of this mode is to make the generated path inspectable. A texture shown on
/// a model in the viewer is hard to judge: the question "did the generator paint the UV
/// space the geometry actually uses" is answered by the numbers here, and the PNG is
/// the evidence a person can look at.
fn creature_skin(path: &str, instance: u64, out: Option<&str>, force: bool) -> i32 {
    let mut source = match assets::AssetSource::open(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    if let Some(set) = package_set() {
        source = source.with_set(set);
    }

    // Whether the install ships a skin decides which path runs, so say it plainly
    // rather than letting the note in the texture label carry the whole story.
    match source.creature_skin(instance) {
        Some((t, note)) => println!("shipped : {note} ({}x{})", t.width, t.height),
        None => println!("shipped : none"),
    }
    match source.creature_palette(instance) {
        Some(p) => println!(
            "palette : {:.3} {:.3} {:.3}   {:.3} {:.3} {:.3}   {:.3} {:.3} {:.3}",
            p[0][0], p[0][1], p[0][2], p[1][0], p[1][1], p[1][2], p[2][0], p[2][1], p[2][2]
        ),
        None => println!("palette : none"),
    }
    match source.creature(instance) {
        Ok((_, mesh)) => {
            let s = spore_core::skin::uv_stats(&mesh);
            println!(
                "uv      : {}/{} triangles usable ({:.1}%), u {:.3}..{:.3} v {:.3}..{:.3}, {} clipped",
                s.triangles_with_uv,
                s.triangles_total,
                s.usable_fraction() * 100.0,
                s.u_min,
                s.u_max,
                s.v_min,
                s.v_max,
                s.clipped
            );
        }
        Err(err) => println!("uv      : no mesh ({err})"),
    }

    let chosen = if force {
        // Regenerate even when a skin ships, so the generated atlas can be compared
        // against the hand-painted one for the same creature. That comparison is the
        // only check on whether the generator is painting the UV space the geometry
        // actually uses.
        let palette = source.creature_palette(instance);
        let mesh = source.creature(instance).ok().map(|(_, m)| m);
        match (palette, mesh) {
            (Some(p), Some(m)) => spore_core::skin::generate(&m, &p, 512).map(|s| {
                (
                    s.width,
                    s.height,
                    s.rgba,
                    format!(
                        "forced generation, {} texels ({} marked)",
                        s.covered, s.marked
                    ),
                )
            }),
            _ => None,
        }
    } else {
        source
            .creature_texture(instance)
            .map(|(t, note)| (t.width, t.height, t.rgba, note))
    };
    let Some((width, height, rgba, note)) = chosen else {
        println!("texture : none");
        return 1;
    };
    println!("texture : {note}");

    // How much of the painted surface ended up each palette colour. A generator that
    // never applies the third colour is not painting markings, so this is the check
    // that the accent is actually used rather than assumed.
    if let Some(p) = source.creature_palette(instance) {
        let mut counts = [0usize; 3];
        let mut total = 0usize;
        for px in rgba.chunks_exact(4) {
            if px[3] == 0 {
                continue;
            }
            total += 1;
            let rgb = [
                px[0] as f32 / 255.0,
                px[1] as f32 / 255.0,
                px[2] as f32 / 255.0,
            ];
            let mut best = 0;
            let mut best_d = f32::INFINITY;
            for (k, c) in p.iter().enumerate() {
                let d = (rgb[0] - c[0]).powi(2) + (rgb[1] - c[1]).powi(2) + (rgb[2] - c[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = k;
                }
            }
            counts[best] += 1;
        }
        if total > 0 {
            println!(
                "nearest : colour0 {:.1}%  colour1 {:.1}%  colour2 {:.1}%  of {} texels",
                counts[0] as f32 * 100.0 / total as f32,
                counts[1] as f32 * 100.0 / total as f32,
                counts[2] as f32 * 100.0 / total as f32,
                total
            );
        }
    }

    let Some(out) = out else {
        return 0;
    };
    let Some(img) = image::RgbaImage::from_raw(width, height, rgba) else {
        eprintln!("buffer size does not match {width}x{height}");
        return 1;
    };
    if let Err(err) = img.save(out) {
        eprintln!("could not write {out}: {err}");
        return 1;
    }
    println!("wrote   : {out}");
    0
}

/// Say which package holds a resource, using the same search path the viewer uses.
///
/// This exists to answer "is the reference dangling, or is my resolver wrong?" without
/// guessing: a prop key that resolves to nothing here is either absent from the install
/// or a bug.
fn locate(instance: u64, type_id: u32) -> i32 {
    let Some(set) = package_set() else {
        eprintln!("set RUSTSPORE_PACKAGES to a package or a directory");
        return 2;
    };
    println!(
        "searching {} package(s) for {type_id:08X}:{instance:016X}",
        set.len()
    );
    match set.lookup(type_id, instance) {
        Some((t, i)) => {
            let where_ = set.locate(t, i);
            println!(
                "found  {t:08X}:{i:016X} in {}",
                where_
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "?".to_string())
            );
            0
        }
        None => {
            println!("not found in any package");
            1
        }
    }
}

/// Extra packages to search, from `RUSTSPORE_PACKAGES`.
///
/// The value is a `;`-separated list of files or directories. A directory is
/// searched recursively for `.package` files. This exists because a prop's parent
/// and often its texture live in another package, and passing those on the command
/// line would collide with the positional arguments the other modes already use.
fn package_set() -> Option<assets::PackageSet> {
    let value = std::env::var("RUSTSPORE_PACKAGES").ok()?;
    let mut paths = Vec::new();
    let mut set: Option<assets::PackageSet> = None;
    for part in value.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let p = std::path::PathBuf::from(part);
        if p.is_dir() {
            let found = assets::PackageSet::discover(&p);
            if found.is_empty() {
                continue;
            }
            // Fold discovered packages into one set rather than nesting sets.
            for path in found.paths() {
                paths.push(path.clone());
            }
        } else {
            paths.push(p);
        }
    }
    if paths.is_empty() {
        return set.take();
    }
    Some(assets::PackageSet::from_paths(paths))
}

fn verify_package(path: &str) -> i32 {
    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };

    let mut coded = 0usize;
    let mut raw = 0usize;
    let mut exact = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let mut first_errors: Vec<String> = Vec::new();

    for entry in file.entries() {
        if !entry.is_coded() {
            raw += 1;
            continue;
        }
        coded += 1;
        match file.resource_data(entry) {
            Ok(data) => {
                if data.len() == entry.decompressed_size as usize {
                    exact += 1;
                } else {
                    failures.push(format!(
                        "0x{:08X}/0x{:08X}: produced {} of {} declared bytes",
                        entry.key.type_id,
                        entry.key.instance_id,
                        data.len(),
                        entry.decompressed_size
                    ));
                }
            }
            Err(err) => {
                if first_errors.len() < 10 {
                    first_errors.push(format!(
                        "0x{:08X}/0x{:08X}: {err}",
                        entry.key.type_id, entry.key.instance_id
                    ));
                }
                failures.push(String::from("error"));
            }
        }
    }

    println!("package          : {}", file.path().display());
    println!(
        "resources        : {} ({coded} coded, {raw} raw)",
        file.entries().len()
    );
    println!(
        "coded decompressed to the declared size: {exact} of {coded} ({:.4}%)",
        if coded == 0 {
            100.0
        } else {
            100.0 * exact as f64 / coded as f64
        }
    );

    for line in first_errors.iter() {
        println!("  error: {line}");
    }
    for line in failures.iter().take(10) {
        println!("  mismatch: {line}");
    }

    if failures.is_empty() {
        println!("result           : OK");
        0
    } else {
        println!("result           : {} FAILED", failures.len());
        1
    }
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
    println!(
        "  index offset     : {} (0x{:X})",
        h.index_offset, h.index_offset
    );
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

    let shape = file.record_shape();
    println!(
        "  record shape     : {}-byte records, {} byte prefix, {} byte trailer ({:?})",
        shape.record_size, shape.prefix_size, shape.trailer_size, shape.kind
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

/// Print the full resource type histogram for one package, one line per type.
///
/// Format: `<type_id_hex> <count>`. Intended to be aggregated across packages with
/// `awk`, which is how the roadmap's coverage numbers are produced.
fn print_types(path: &str) {
    use std::collections::HashMap;

    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            std::process::exit(2);
        }
    };

    let mut histogram: HashMap<u32, usize> = HashMap::new();
    for entry in file.entries() {
        *histogram.entry(entry.key.type_id).or_default() += 1;
    }
    let mut ranked: Vec<(u32, usize)> = histogram.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (type_id, count) in ranked {
        println!("0x{type_id:08X} {count}");
    }
}

/// FNV-1a 64-bit, the same hash the Python tools use.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Decode every RASTER resource and print a digest of the RGBA output.
///
/// Format: `<instance:016X> <width> <height> <format:08X> <rgba_fnv1a:016X>`.
/// `tools/raster_digest.py` prints the same lines using the Python decoder, so a
/// diff of the two is a byte-exact check of the Rust pixel decoder on real data.
fn raster_digest(path: &str) -> i32 {
    const RASTER_TYPE: u32 = 0x2F4E_681C;

    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };

    let mut ok = 0usize;
    let mut failed = 0usize;
    for entry in file.entries() {
        if entry.key.type_id != RASTER_TYPE {
            continue;
        }
        let data = match file.resource_data(entry) {
            Ok(d) => d,
            Err(err) => {
                eprintln!("read 0x{:016X}: {err}", entry.key.instance_id);
                failed += 1;
                continue;
            }
        };
        match spore_core::raster::parse(&data).and_then(|r| {
            let rgba = r.decode_rgba()?;
            Ok((r.width, r.height, r.format, rgba))
        }) {
            Ok((w, h, fmt, rgba)) => {
                println!(
                    "{:016X} {:>5} {:>5} {:08X} {:016X}",
                    entry.key.instance_id,
                    w,
                    h,
                    fmt,
                    fnv1a64(&rgba)
                );
                ok += 1;
            }
            Err(err) => {
                eprintln!("decode 0x{:016X}: {err}", entry.key.instance_id);
                failed += 1;
            }
        }
    }
    eprintln!("decoded {ok}, failed {failed}");
    if failed == 0 {
        0
    } else {
        1
    }
}

/// Decode every gmsh resource and print a digest of the typed geometry.
///
/// Format: `<instance:016X> <vertices> <indices> <normals> <uvs> <fnv1a:016X>`.
/// The hash covers positions, indices, normals and UVs as little-endian bytes, so
/// `tools/gmsh_digest.py` can reproduce it exactly from the Python parser.
fn gmsh_digest(path: &str) -> i32 {
    const GMSH_TYPE: u32 = 0x01C1_35DA;

    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };

    let mut ok = 0usize;
    let mut failed = 0usize;
    for entry in file.entries() {
        if entry.key.type_id != GMSH_TYPE {
            continue;
        }
        let data = match file.resource_data(entry) {
            Ok(d) => d,
            Err(err) => {
                eprintln!("read 0x{:016X}: {err}", entry.key.instance_id);
                failed += 1;
                continue;
            }
        };
        match spore_core::gmsh::parse(&data) {
            Ok(mesh) => {
                let mut pos: Vec<u8> = Vec::new();
                for p in &mesh.positions {
                    pos.extend_from_slice(&p.x.to_le_bytes());
                    pos.extend_from_slice(&p.y.to_le_bytes());
                    pos.extend_from_slice(&p.z.to_le_bytes());
                }
                let idx = mesh.indices.clone().unwrap_or_default();
                let mut ib: Vec<u8> = Vec::new();
                for i in &idx {
                    ib.extend_from_slice(&i.to_le_bytes());
                }
                let normals = mesh.normals.as_ref().map(Vec::len).unwrap_or(0);
                let mut nb: Vec<u8> = Vec::new();
                for n in mesh.normals.iter().flatten() {
                    nb.extend_from_slice(&n.x.to_le_bytes());
                    nb.extend_from_slice(&n.y.to_le_bytes());
                    nb.extend_from_slice(&n.z.to_le_bytes());
                }
                let uvs = mesh.uvs.as_ref().map(Vec::len).unwrap_or(0);
                let mut ub: Vec<u8> = Vec::new();
                for uv in mesh.uvs.iter().flatten() {
                    ub.extend_from_slice(&uv[0].to_le_bytes());
                    ub.extend_from_slice(&uv[1].to_le_bytes());
                }
                println!(
                    "{:016X} {:>5} {:>6} {:>6} {:>5} {:016X} {:016X} {:016X} {:016X}",
                    entry.key.instance_id,
                    mesh.positions.len(),
                    idx.len(),
                    normals,
                    uvs,
                    fnv1a64(&pos),
                    fnv1a64(&ib),
                    fnv1a64(&nb),
                    fnv1a64(&ub)
                );
                ok += 1;
            }
            Err(err) => {
                eprintln!("parse 0x{:016X}: {err}", entry.key.instance_id);
                failed += 1;
            }
        }
    }
    eprintln!("parsed {ok}, failed {failed}");
    if failed == 0 {
        0
    } else {
        1
    }
}

/// Convert every mesh resource to renderable geometry and check the invariants a
/// renderer depends on. Runs without Bevy so it can be used in a shell.
///
/// This goes through `AssetSource`, which is the same path the viewer uses, so it
/// covers gmsh and gmdl together.
fn mesh_selftest(path: &str) -> i32 {
    let source = match assets::AssetSource::open(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };

    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut from_file = 0usize;
    let mut computed = 0usize;
    let mut tris = 0usize;
    let mut verts = 0usize;
    let mut with_uvs = 0usize;
    let mut gmsh = 0usize;
    let mut gmdl = 0usize;
    let mut rw4 = 0usize;
    let mut empty = 0usize;
    let mut sanitised = 0usize;
    for i in 0..source.mesh_count() {
        let key = source.mesh_key(i).expect("index in range");
        match source.mesh(i) {
            Ok(mesh) => {
                if mesh.normals_from_file {
                    from_file += 1;
                } else {
                    computed += 1;
                }
                if mesh.uvs.iter().any(|uv| uv[0] != 0.0 || uv[1] != 0.0) {
                    with_uvs += 1;
                }
                tris += mesh.triangle_count();
                verts += mesh.positions.len();
                match key.type_id {
                    assets::GMSH => gmsh += 1,
                    assets::GMDL => gmdl += 1,
                    assets::RW4 => rw4 += 1,
                    _ => {}
                }
                sanitised += mesh.sanitised;
                ok += 1;
            }
            Err(assets::AssetError::Empty) => {
                // A valid resource with no geometry, not a conversion failure.
                empty += 1;
            }
            Err(err) => {
                eprintln!("convert {key}: {err}");
                failed += 1;
            }
        }
    }

    println!("mesh self-test on {path}");
    println!("  meshes converted : {ok} (gmsh {gmsh}, gmdl {gmdl}, rw4 {rw4})");
    println!("  empty resources  : {empty}");
    println!("  failures         : {failed}");
    println!("  sanitised values : {sanitised}");
    println!("  normals from file: {from_file}");
    println!("  normals computed : {computed}");
    println!("  meshes with UVs  : {with_uvs}");
    println!("  vertices         : {verts}");
    println!("  triangles        : {tris}");
    if failed == 0 {
        0
    } else {
        1
    }
}

/// Census the gmdl vertex element declarations: which stream, element type,
/// usage and usage index appear, and how often.
fn gmdl_elements(path: &str) -> i32 {
    use std::collections::BTreeMap;

    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };

    let mut counts: BTreeMap<(i16, i8, i8, i8), usize> = BTreeMap::new();
    let mut models = 0usize;
    for entry in file.entries() {
        if entry.key.type_id != assets::GMDL {
            continue;
        }
        let Ok(data) = file.resource_data(entry) else {
            continue;
        };
        let Ok(g) = spore_core::gmdl::parse(&data) else {
            continue;
        };
        models += 1;
        for f in &g.vertex_formats {
            for e in &f.elements {
                *counts
                    .entry((e.stream, e.element_type, e.usage, e.usage_index))
                    .or_default() += 1;
            }
        }
    }

    println!("gmdl models parsed: {models}");
    println!("(stream, type, usage, usage_index) -> occurrences");
    let mut v: Vec<_> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for ((stream, ty, usage, ui), n) in v.into_iter().take(20) {
        println!("  stream {stream:>2}  type {ty:>2}  usage {usage:>2}  index {ui:>2}  -> {n}");
    }
    0
}

/// Print one gmdl model's vertex formats and the first vertices of each buffer.
/// Used to diagnose resources that fail conversion.
fn gmdl_info(path: &str, instance: u64) -> i32 {
    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    for entry in file.entries() {
        if entry.key.type_id != assets::GMDL || entry.key.instance_id != instance {
            continue;
        }
        let Ok(data) = file.resource_data(entry) else {
            return 1;
        };
        let Ok(g) = spore_core::gmdl::parse(&data) else {
            eprintln!("parse failed");
            return 1;
        };
        println!(
            "instance 0x{instance:016X} size {} meshes {}",
            data.len(),
            g.meshes.len()
        );
        for (i, f) in g.vertex_formats.iter().enumerate() {
            println!("format {i}: stride {}", f.stride());
            for e in &f.elements {
                println!(
                    "   stream {} offset {} type {} method {} usage {} index {}",
                    e.stream, e.offset, e.element_type, e.method, e.usage, e.usage_index
                );
            }
        }
        for (i, vb) in g.vertex_buffers.iter().enumerate() {
            println!(
                "buffer {i}: format {} vertices {} stride {} at {}",
                vb.format_index, vb.vertex_count, vb.stride, vb.data_offset
            );
            let mut bad = 0usize;
            let mut first_bad = None;
            for v in 0..vb.vertex_count.max(0) as usize {
                let at = vb.data_offset + v * vb.stride;
                if at + 12 > data.len() {
                    break;
                }
                let x = f32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
                let y =
                    f32::from_le_bytes([data[at + 4], data[at + 5], data[at + 6], data[at + 7]]);
                let z =
                    f32::from_le_bytes([data[at + 8], data[at + 9], data[at + 10], data[at + 11]]);
                if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                    bad += 1;
                    if first_bad.is_none() {
                        first_bad = Some((v, x, y, z));
                    }
                }
            }
            println!("   non-finite positions: {bad} of {}", vb.vertex_count);
            if let Some((v, x, y, z)) = first_bad {
                println!("   first at v{v}: {x} {y} {z}");
            }
            // UV and position ranges from the format's own element offsets, which
            // is the only way to tell "UVs are applied" from "UVs all land on 0".
            if let Some(fmt) = g.vertex_formats.get(vb.format_index as usize) {
                for (label, usage) in [("position", 0i8), ("uv", 5i8)] {
                    let Some(el) = fmt.elements.iter().find(|e| e.usage == usage) else {
                        continue;
                    };
                    let off = el.offset.max(0) as usize;
                    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
                    let mut finite = 0usize;
                    for v in 0..vb.vertex_count.max(0) as usize {
                        let at = vb.data_offset + v * vb.stride + off;
                        if at + 8 > data.len() {
                            break;
                        }
                        let a = f32::from_le_bytes([
                            data[at],
                            data[at + 1],
                            data[at + 2],
                            data[at + 3],
                        ]);
                        let b = f32::from_le_bytes([
                            data[at + 4],
                            data[at + 5],
                            data[at + 6],
                            data[at + 7],
                        ]);
                        if a.is_finite() && b.is_finite() {
                            lo = lo.min(a).min(b);
                            hi = hi.max(a).max(b);
                            finite += 1;
                        }
                    }
                    println!(
                        "   {label} at offset {off}: {finite} finite, range {lo:.3} .. {hi:.3}"
                    );
                }
            }
            for v in 0..3.min(vb.vertex_count.max(0) as usize) {
                let at = vb.data_offset + v * vb.stride;
                if at + 12 <= data.len() {
                    let x =
                        f32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
                    let y = f32::from_le_bytes([
                        data[at + 4],
                        data[at + 5],
                        data[at + 6],
                        data[at + 7],
                    ]);
                    let z = f32::from_le_bytes([
                        data[at + 8],
                        data[at + 9],
                        data[at + 10],
                        data[at + 11],
                    ]);
                    println!("   v{v}: {x} {y} {z}");
                }
            }
        }
        return 0;
    }
    eprintln!("instance not found");
    1
}

/// Print an RW4 container's object table and its first mesh's links.
fn rw4_info(path: &str, instance: u64) -> i32 {
    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    for entry in file.entries() {
        if entry.key.type_id != assets::RW4 || entry.key.instance_id != instance {
            continue;
        }
        let Ok(data) = file.resource_data(entry) else {
            return 1;
        };
        let Ok(rw) = spore_core::rw4::parse(&data) else {
            eprintln!("parse failed");
            return 1;
        };
        println!(
            "instance 0x{instance:016X} size {} objects {}",
            data.len(),
            rw.sections.len()
        );
        let mut counts = std::collections::BTreeMap::new();
        for s in &rw.sections {
            *counts.entry(s.info.type_code).or_insert(0usize) += 1;
        }
        for (t, n) in counts {
            println!("   type 0x{t:06X} x{n}");
        }
        let meshes = rw.meshes(&data);
        println!("meshes: {}", meshes.len());
        // Raw ints of the first mesh section, to check field order against reality.
        if let Some(sec) = rw.sections.iter().find(|s| s.info.type_code == 0x20009) {
            let b = &data[sec.start..sec.start + sec.data_size];
            let n = (b.len() / 4).min(12);
            print!("   raw mesh ints:");
            for i in 0..n {
                print!(
                    " {}",
                    i32::from_le_bytes([b[4 * i], b[4 * i + 1], b[4 * i + 2], b[4 * i + 3]])
                );
            }
            println!();
        }
        for m in meshes.iter().take(2) {
            println!(
                "   mesh: idx {} vbs {:?} tris {} first_index {} prims {} first_vertex {} verts {}",
                m.index_buffer,
                m.vertex_buffers,
                m.triangle_count,
                m.first_index,
                m.primitive_count,
                m.first_vertex,
                m.vertex_count
            );
            for vb_index in &m.vertex_buffers {
                match rw.vertex_buffer(&data, *vb_index) {
                    Some(vb) => {
                        let desc = rw.vertex_description(&data, vb.description);
                        let buf = rw.object(&data, vb.data);
                        println!(
                            "      vb desc {} count {} vsize {} data {} (len {:?})",
                            vb.description,
                            vb.vertex_count,
                            vb.vertex_size,
                            vb.data,
                            buf.map(|b| b.len())
                        );
                        if let Some(d) = desc {
                            println!("      elements {}", d.elements.len());
                            for e in &d.elements {
                                println!(
                                    "         stream {} offset {} type {} usage {} index {}",
                                    e.stream, e.offset, e.element_type, e.usage, e.usage_index
                                );
                            }
                            if let Some(buf) = buf {
                                element_stats(buf, &d);
                            }
                        } else {
                            println!("      description failed to decode");
                        }
                    }
                    None => println!("      vb failed to decode"),
                }
            }
        }
        return 0;
    }
    eprintln!("instance not found");
    1
}

/// Count how RW4 meshes reference their geometry: a real vertex buffer, or the
/// "no object" sentinel that means the geometry is a blend shape instead.
fn rw4_census(path: &str) -> i32 {
    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    let mut files = 0usize;
    let mut with_mesh = 0usize;
    let mut real_vb = 0usize;
    let mut no_vb = 0usize;
    let mut multi_vb = 0usize;
    let mut blend_shape = 0usize;
    for entry in file.entries() {
        if entry.key.type_id != assets::RW4 {
            continue;
        }
        let Ok(data) = file.resource_data(entry) else {
            continue;
        };
        let Ok(rw) = spore_core::rw4::parse(&data) else {
            continue;
        };
        files += 1;
        let meshes = rw.meshes(&data);
        if !meshes.is_empty() {
            with_mesh += 1;
        }
        if rw.sections.iter().any(|s| s.info.type_code == 0x00FF_0002) {
            blend_shape += 1;
        }
        for m in &meshes {
            if m.vertex_buffers.len() > 1 {
                multi_vb += 1;
            }
            match m.vertex_buffers.first() {
                Some(v) if v >> 22 == 0 => real_vb += 1,
                _ => no_vb += 1,
            }
        }
    }
    println!("rw4 files            : {files}");
    println!("  containing a mesh  : {with_mesh}");
    println!("  containing a blend : {blend_shape}");
    println!("  meshes             : {}", real_vb + no_vb);
    println!("    with real vb     : {real_vb}");
    println!("    with no vb       : {no_vb}");
    println!("    multi-vb         : {multi_vb}");
    0
}

/// Resolve every prop to the model and textures it names.
///
/// The denominator matters here: most props describe something other than a model,
/// so "props resolved" is only meaningful against "props that name a model".
fn prop_models(path: &str) -> i32 {
    let mut source = match assets::AssetSource::open(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    if let Some(set) = package_set() {
        eprintln!("searching {} other package(s)", set.len());
        source = source.with_set(set);
    }
    let (props, named, resolved, with_tex, lods, missing) = source.prop_census();
    println!("props in package        : {props}");
    println!("  naming a model        : {named}");
    println!("  model found in package: {resolved}");
    println!("  model missing         : {missing}");
    println!("  with at least 1 texture: {with_tex}");
    println!("  LOD slots total       : {lods}");
    0
}

/// Write one model's embedded texture to a PNG.
///
/// The viewer shows the texture on the model, which is not enough to tell a
/// detail map from an albedo map. This writes it flat so the answer is visible.
fn rw4_texture_png(path: &str, instance: u64, out: &str) -> i32 {
    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };
    for entry in file.entries() {
        if entry.key.type_id != assets::RW4 || entry.key.instance_id != instance {
            continue;
        }
        let Ok(data) = file.resource_data(entry) else {
            return 1;
        };
        let Ok(rw) = spore_core::rw4::parse(&data) else {
            eprintln!("parse failed");
            return 1;
        };
        let Some(tex) = rw.texture(&data) else {
            eprintln!("no texture in this container");
            return 1;
        };
        let rgba =
            match spore_core::raster::decode_pixels(tex.format, tex.width, tex.height, tex.payload)
            {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("decode failed: {err}");
                    return 1;
                }
            };
        let Some(img) = image::RgbaImage::from_raw(tex.width, tex.height, rgba) else {
            eprintln!("buffer size does not match {}x{}", tex.width, tex.height);
            return 1;
        };
        if let Err(err) = img.save(out) {
            eprintln!("could not write {out}: {err}");
            return 1;
        }
        println!(
            "{out}: {} {}x{} {} mips",
            spore_core::raster::format_name(tex.format),
            tex.width,
            tex.height,
            tex.mipmap_levels
        );
        return 0;
    }
    eprintln!("instance not found");
    1
}

/// Decode every texture embedded in an RW4 container and report what happened.
///
/// The textures are not separate resources: they live inside the model file, so
/// this is the only path that proves a model can be shaded from its own container.
/// Each line is `instance format width height digest` so `tools/rw4_texture_digest.py`
/// can be diffed against it.
fn rw4_textures(path: &str) -> i32 {
    use std::collections::BTreeMap;

    let file = match dbpf::DbpfFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("could not open package {path}: {err}");
            return 2;
        }
    };

    let mut files = 0usize;
    let mut with_texture = 0usize;
    let mut decoded = 0usize;
    let mut failed = 0usize;
    let mut formats: BTreeMap<&'static str, usize> = BTreeMap::new();
    for entry in file.entries() {
        if entry.key.type_id != assets::RW4 {
            continue;
        }
        let Ok(data) = file.resource_data(entry) else {
            continue;
        };
        let Ok(rw) = spore_core::rw4::parse(&data) else {
            continue;
        };
        files += 1;
        let Some(tex) = rw.texture(&data) else {
            continue;
        };
        with_texture += 1;
        match spore_core::raster::decode_pixels(tex.format, tex.width, tex.height, tex.payload) {
            Ok(rgba) if !rgba.is_empty() => {
                decoded += 1;
                *formats
                    .entry(spore_core::raster::format_name(tex.format))
                    .or_default() += 1;
                println!(
                    "{:016X} {:>8} {:>5} {:>5} {:016X}",
                    entry.key.instance_id,
                    spore_core::raster::format_name(tex.format),
                    tex.width,
                    tex.height,
                    fnv1a64(&rgba)
                );
            }
            Ok(_) => {
                failed += 1;
                eprintln!(
                    "empty {:016X} {}x{}",
                    entry.key.instance_id, tex.width, tex.height
                );
            }
            Err(err) => {
                failed += 1;
                eprintln!(
                    "decode {:016X} 0x{:08X} {}x{}: {err}",
                    entry.key.instance_id, tex.format, tex.width, tex.height
                );
            }
        }
    }

    eprintln!("rw4 files {files}, with a texture {with_texture}");
    eprintln!("decoded {decoded}, failed {failed}");
    for (name, n) in formats {
        eprintln!("  {name}: {n}");
    }
    if failed == 0 {
        0
    } else {
        1
    }
}

/// Min and max of the position and UV elements, read the way `from_rw4` reads
/// them. A UV range outside 0..1 is not automatically wrong, but a range of a few
/// million is, and this is where that shows up.
fn element_stats(buf: &[u8], desc: &spore_core::rw4::VertexDescription) {
    let stride = desc.vertex_size as usize;
    if stride == 0 || buf.len() < stride {
        return;
    }
    let count = buf.len() / stride;
    let f32_at = |at: usize| -> f32 {
        if at + 4 <= buf.len() {
            f32::from_le_bytes([buf[at], buf[at + 1], buf[at + 2], buf[at + 3]])
        } else {
            f32::NAN
        }
    };
    for (name, usage) in [("position", 0i8), ("uv", 5i8)] {
        let Some(e) = desc.elements.iter().find(|e| e.usage == usage) else {
            continue;
        };
        let at0 = e.offset.max(0) as usize;
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        let mut bad = 0usize;
        let step = (count / 500).max(1);
        let mut sampled = 0usize;
        for i in (0..count).step_by(step) {
            let at = i * stride + at0;
            let a = f32_at(at);
            let b = f32_at(at + 4);
            if !a.is_finite() || !b.is_finite() {
                bad += 1;
                continue;
            }
            lo = lo.min(a).min(b);
            hi = hi.max(a).max(b);
            sampled += 1;
        }
        println!("      {name}: {sampled} sampled, range {lo:.3} .. {hi:.3}, non-finite {bad}");
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

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn hex_identifiers_accept_surrounding_whitespace_and_optional_prefix() {
        for text in [
            "406B62000660EACA\r",
            " \t0x406B62000660eaca\r\n",
            "0X406B62000660EACA",
        ] {
            assert_eq!(parse_hex_arg(text, "instance").unwrap(), 0x406B62000660EACA);
        }
        assert_eq!(parse_hex_arg("0", "instance").unwrap(), 0);
        assert_eq!(
            parse_hex_arg("FFFFFFFFFFFFFFFF", "instance").unwrap(),
            u64::MAX
        );
    }

    #[test]
    fn invalid_identifiers_are_errors_instead_of_zero() {
        for text in [
            "",
            " \r\n",
            "0x",
            "not-an-instance",
            "12 34",
            "-1",
            "10000000000000000",
        ] {
            assert!(
                parse_hex_arg(text, "instance").is_err(),
                "accepted {text:?}"
            );
        }
    }

    #[test]
    fn type_identifiers_must_fit_u32() {
        assert_eq!(parse_type_arg(" 0x2F4E681B\r").unwrap(), 0x2F4E681B);
        assert_eq!(parse_type_arg("FFFFFFFF").unwrap(), u32::MAX);
        assert!(parse_type_arg("100000000").is_err());
        assert!(parse_type_arg("invalid").is_err());
    }

    #[test]
    fn mesh_indices_trim_whitespace_and_reject_invalid_numbers() {
        assert_eq!(parse_mesh_index(" 42\r\n").unwrap(), 42);
        assert_eq!(parse_mesh_index("0").unwrap(), 0);
        for text in ["", "no-index", "-1", "1.5", "0x10"] {
            assert!(parse_mesh_index(text).is_err(), "accepted {text:?}");
        }
        assert!(parse_mesh_index(&format!("{}0", usize::MAX)).is_err());
    }

    #[test]
    fn shot_targets_validate_kind_and_instance() {
        assert!(matches!(
            parse_shot_target("creature=406B62000660EACA\r").unwrap(),
            viewer::Target::Creature(0x406B62000660EACA)
        ));
        assert!(matches!(
            parse_shot_target(" prop=0X2A ").unwrap(),
            viewer::Target::Prop(42)
        ));
        assert!(matches!(
            parse_shot_target(" \t0X2A\r\n").unwrap(),
            viewer::Target::Prop(42)
        ));
        for text in ["creature=invalid", "prop=", "prop", "unknown=42", ""] {
            assert!(parse_shot_target(text).is_err(), "accepted {text:?}");
        }
    }
}
