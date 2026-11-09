//! Controlled saved-asset inspection, not a published Cell actor or scene.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const TRIPLE: [u32; 3] = [0xe219e5d3, 0x3d97a8e4, 0x40616200];
const PROP_SHA: &str = "34f7aa2f8f8e57b14c7e25f60d6474d13d5e77cb8fe449edd9688a151358db8d";

fn output_path(path: &Path) -> Result<PathBuf, String> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("workspace path unavailable")?;
    let target = workspace
        .join("target")
        .canonicalize()
        .map_err(|e| format!("target: {e}"))?;
    output_under(path, &target)
}

fn output_under(path: &Path, target: &Path) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| format!("output parent: {e}"))?;
    if !parent.starts_with(target)
        || !path
            .extension()
            .is_some_and(|x| x.eq_ignore_ascii_case("png"))
    {
        return Err("output must be a new PNG under workspace target".into());
    }
    let output = parent.join(path.file_name().ok_or("output filename required")?);
    if output.exists() {
        return Err("capture output already exists".into());
    }
    Ok(output)
}

fn check_prop_bytes(bytes: &[u8], expected_size: usize) -> Result<(), String> {
    if bytes.len() != expected_size || format!("{:x}", Sha256::digest(bytes)) != PROP_SHA {
        return Err("admitted PROP payload mismatch".into());
    }
    Ok(())
}

fn check_prop(install: &Path) -> Result<(), String> {
    let source = NativePackageIndex::open(&install.join("Data/PatchData.package"))
        .map_err(|e| e.to_string())?;
    let key = NativeResourceKey {
        instance_id: TRIPLE[0],
        type_id: 0x00b1b104,
        group_id: TRIPLE[2],
    };
    let entry = source
        .index()
        .find_unique(key)
        .map_err(|e| e.to_string())?
        .ok_or("admitted PROP missing")?;
    if entry.stored_size > 1024 * 1024 || entry.decompressed_size > 1024 * 1024 {
        return Err("PROP exceeds bound".into());
    }
    let bytes = source.read(entry).map_err(|e| e.to_string())?;
    check_prop_bytes(&bytes, entry.decompressed_size as usize)?;
    // The authenticated payload has one explicit 00F9EFBB key pointing to
    // E219E5D3:00E6BCE5:40617100; see cell-controlled-player-visual-candidate-v1.md.
    Ok(())
}

#[derive(Debug)]
struct Options {
    install: PathBuf,
    seed: u32,
    progress: i32,
    shot: PathBuf,
    amd_adapter: bool,
    trace_inputs: bool,
}
fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut install = None;
    let mut seed = None;
    let mut progress = None;
    let mut shot = None;
    let mut adapter = None;
    let mut trace = None;
    for pair in args.chunks(2) {
        if pair.len() != 2 {
            return Err("each option requires a value".into());
        }
        match pair[0].as_str() {
            "--install" if install.is_none() => install = Some(PathBuf::from(&pair[1])),
            "--seed" if seed.is_none() => {
                seed = Some(pair[1].parse::<u32>().map_err(|_| "invalid seed")?)
            }
            "--progress" if progress.is_none() => {
                progress = Some(pair[1].parse::<i32>().map_err(|_| "invalid progress")?)
            }
            "--shot" if shot.is_none() => shot = Some(PathBuf::from(&pair[1])),
            "--adapter" if adapter.is_none() => {
                adapter = Some(match pair[1].as_str() {
                    "rtx" => false,
                    "amd" => true,
                    _ => return Err("adapter must be rtx or amd".into()),
                })
            }
            "--trace-inputs" if trace.is_none() && pair[1] == "admit-stable-packages" => {
                trace = Some(true);
            }
            _ => return Err(format!("unknown or repeated option {}", pair[0])),
        }
    }
    Ok(Options {
        install: install.ok_or("--install required")?,
        seed: seed.ok_or("--seed required")?,
        progress: progress.unwrap_or(0),
        shot: shot.ok_or("--shot required")?,
        amd_adapter: adapter.unwrap_or(false),
        trace_inputs: trace.unwrap_or(false),
    })
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let output = output_path(&options.shot)?;
    let install = &options.install;
    let result = crate::cell_source_player_v1::inspect(install, options.seed, options.progress)?;
    if result.triple != TRIPLE {
        return Err(format!(
            "unsupported controlled triple {:08X?}",
            result.triple
        ));
    }
    check_prop(install)?;
    println!("CONTROLLED_PLAYER_VISUAL triple={:08X?} prop_sha256={} model_and_raster_hash_checks_required=true", result.triple, PROP_SHA);
    println!("inspection only: no published actor, animation, source scene, retail provider precedence or gameplay; shader and other material resources are unpinned; camera and lighting are inspection inputs");
    println!("package paths must remain stable during reads; output parent must already exist under workspace target; failed writes can leave a partial PNG");
    println!("controlled adapter request={}; previous RTX validation does not establish AMD results", if options.amd_adapter { "amd" } else { "rtx" });
    #[cfg(all(windows, feature = "native_renderer"))]
    return crate::native_cell_renderer::capture_controlled_player(install, &output, options.amd_adapter, options.trace_inputs)
        .map_err(|e| e.to_string());
    #[cfg(not(all(windows, feature = "native_renderer")))]
    {
        let _ = output;
        Err("original-program offscreen capture requires Windows native rendering".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_require_unique_explicit_inputs() {
        let valid = args(&["--install", "root", "--seed", "1", "--shot", "target/a.png"]);
        assert_eq!(parse_options(&valid).unwrap().progress, 0);
        assert!(!parse_options(&valid).unwrap().amd_adapter);
        let mut traced = valid.clone();
        traced.extend(args(&["--trace-inputs", "admit-stable-packages"]));
        assert!(parse_options(&traced).unwrap().trace_inputs);
        traced.extend(args(&["--trace-inputs", "admit-stable-packages"]));
        assert!(parse_options(&traced).is_err());
        let mut unadmitted = valid.clone();
        unadmitted.extend(args(&["--trace-inputs", "yes"]));
        assert!(parse_options(&unadmitted).is_err());
        let mut amd = valid.clone();
        amd.extend(args(&["--adapter", "amd"]));
        assert!(parse_options(&amd).unwrap().amd_adapter);
        let mut invalid_adapter = valid.clone();
        invalid_adapter.extend(args(&["--adapter", "other"]));
        assert!(parse_options(&invalid_adapter).is_err());
        amd.extend(args(&["--adapter", "rtx"]));
        assert!(parse_options(&amd).is_err());
        assert!(parse_options(&valid[..4]).is_err());
        assert!(parse_options(&args(&[
            "--install",
            "root",
            "--seed",
            "1",
            "--seed",
            "2",
            "--shot",
            "x"
        ]))
        .is_err());
        assert!(parse_options(&args(&[
            "--install",
            "root",
            "--seed",
            "bad",
            "--shot",
            "x"
        ]))
        .is_err());
        assert!(parse_options(&args(&["--install"])).is_err());
    }

    #[test]
    fn invalid_or_truncated_prop_is_rejected() {
        assert!(check_prop_bytes(b"not a pinned prop", 17).is_err());
        assert!(check_prop_bytes(b"", 242).is_err());
    }

    #[test]
    fn output_rejects_outside_non_png_and_existing_files() {
        let root = std::env::temp_dir().join(format!(
            "rustspore-player-path-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let inside = root.join("target");
        std::fs::create_dir_all(&inside).unwrap();
        let canonical = inside.canonicalize().unwrap();
        let new_png = inside.join("new.png");
        assert!(output_under(&new_png, &canonical).is_ok());
        assert!(output_under(&inside.join("bad.txt"), &canonical).is_err());
        assert!(output_under(&root.join("outside.png"), &canonical).is_err());
        std::fs::write(&new_png, b"existing").unwrap();
        assert!(output_under(&new_png, &canonical).is_err());
        std::fs::remove_file(&new_png).unwrap();
        std::fs::remove_dir(&inside).unwrap();
        std::fs::remove_dir(&root).unwrap();
    }
}
