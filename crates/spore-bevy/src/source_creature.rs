//! Offscreen saved-creature capture through authored references and source shaders.
//! The fresh lighting context and fitted camera are explicit inspection inputs.
use crate::assets::{AssetSource, PackageSet};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
struct Options {
    install: PathBuf,
    creature: u64,
    shot: PathBuf,
    coverage: Option<PathBuf>,
    second_shot: Option<PathBuf>,
    second_mask: Option<PathBuf>,
    reference: Option<PathBuf>,
    offset: Option<String>,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut install = std::env::var_os("RUSTSPORE_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("D:/Games/SPORE Collection"));
        let mut creature = None;
        let mut shot = None;
        let mut coverage = None;
        let mut second_shot = None;
        let mut second_mask = None;
        let mut reference = None;
        let mut offset = None;
        let mut index = 0;
        while index < args.len() {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{} requires a value", args[index]))?;
            match args[index].as_str() {
                "--install" => install = PathBuf::from(value),
                "--creature" => creature = Some(crate::parse_hex_arg(value, "creature")?),
                "--shot" if shot.is_none() => shot = Some(PathBuf::from(value)),
                "--coverage" if coverage.is_none() => coverage = Some(PathBuf::from(value)),
                "--second-shot" if second_shot.is_none() => second_shot = Some(PathBuf::from(value)),
                "--second-mask" if second_mask.is_none() => second_mask = Some(PathBuf::from(value)),
                "--reference-shot" if reference.is_none() => reference = Some(PathBuf::from(value)),
                "--second-offset" if offset.is_none() => offset = Some(value.clone()),
                other => return Err(format!("unknown source creature option {other:?}")),
            }
            index += 2;
        }
        let creature = creature.ok_or("--source-creature requires --creature <hex>")?;
        let shot = shot.ok_or("--source-creature requires --shot <new.png>")?;
        if shot.file_name().is_none() || coverage.as_ref().is_some_and(|path| path.file_name().is_none()) {
            return Err("shot and coverage require file paths".into());
        }
        if coverage.as_ref().is_some_and(|path| path == &shot) { return Err("coverage and shot must use different paths".into()); }
        if coverage.is_some() && creature != 0x4061_6200_a1c4_6df2 { return Err("coverage is restricted to Creature 40616200A1C46DF2".into()); }
        let count = [second_shot.is_some(), second_mask.is_some(), reference.is_some(), offset.is_some()].into_iter().filter(|x| *x).count();
        if count != 0 && (count != 4 || coverage.is_none()) { return Err("second frame needs coverage, reference, shot, mask and offset".into()); }
        if let Some(value) = &offset {
            let parsed: Result<Vec<f32>, _> = value.split(',').map(str::parse).collect();
            if !parsed.is_ok_and(|v| v.len() == 3 && v.iter().all(|x| x.is_finite() && x.abs() <= 10.0)) {
                return Err("second offset requires three finite comma-separated values within +/-10".into());
            }
        }
        if second_shot.as_ref().is_some_and(|p| p.file_name().is_none()) || second_mask.as_ref().is_some_and(|p| p.file_name().is_none()) || reference.as_ref().is_some_and(|p| p.file_name().is_none()) {
            return Err("second-frame paths require filenames".into());
        }
        Ok(Self { install, creature, shot, coverage, second_shot, second_mask, reference, offset })
    }
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let options = Options::parse(args)?;
    if options.shot.exists() {
        return Err("capture output already exists; choose a new path".into());
    }
    if options.coverage.as_ref().is_some_and(|path| path.exists()) { return Err("coverage output already exists; choose a new path".into()); }
    if let (Some(shot), Some(mask), Some(reference)) = (&options.second_shot, &options.second_mask, &options.reference) {
        let coverage = options.coverage.as_ref().ok_or("coverage required")?;
        let paths = [&options.shot, coverage, shot, mask];
        let mut resolved = Vec::new();
        for path in paths {
            if path.exists() { return Err("diagnostic outputs must be new".into()); }
            let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
            let key = parent.canonicalize().map_err(|e| e.to_string())?.join(path.file_name().ok_or("output filename required")?);
            let folded = key.to_string_lossy().to_lowercase();
            if resolved.contains(&folded) { return Err("diagnostic output paths alias".into()); }
            resolved.push(folded);
        }
        let reference = reference.canonicalize().map_err(|e| e.to_string())?;
        if resolved.contains(&reference.to_string_lossy().to_lowercase()) { return Err("reference aliases output".into()); }
    }
    let source = AssetSource::open(options.install.join("Data/PatchData.package"))?
        .with_set(PackageSet::discover(&options.install));
    let authored = source
        .authored_creature_model(options.creature, 0)
        .map_err(|e| e.to_string())?;
    println!("SOURCE_CREATURE_ART creation={} prop={} lod={} model={} renderer=original-d3d9 animation=unimplemented scene=inspection",
        authored.creature, authored.prop, authored.lod, authored.model);
    #[cfg(all(windows, feature = "native_renderer"))]
    let second = if let (Some(shot), Some(mask), Some(reference), Some(offset)) = (&options.second_shot, &options.second_mask, &options.reference, &options.offset) {
        let values: Vec<f32> = offset.split(',').map(|v| v.parse::<f32>().expect("validated offset")).collect();
        Some(crate::native_cell_renderer::SecondFrame { reference, shot, mask, offset: [values[0],values[1],values[2]] })
    } else { None };
    #[cfg(all(windows, feature = "native_renderer"))]
    return crate::native_cell_renderer::capture_creature_key(
        &options.install,
        &options.shot,
        authored.model,
        options.coverage.as_deref(),
        second.as_ref(),
    )
    .map_err(|e| e.to_string());
    #[cfg(not(all(windows, feature = "native_renderer")))]
    Err("original creature shader capture requires Windows native rendering".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(text: &[&str]) -> Vec<String> {
        text.iter().map(|s| (*s).to_owned()).collect()
    }
    #[test]
    fn source_identity_is_explicit_and_exact() {
        let parsed = Options::parse(&args(&[
            "--creature",
            "0X406B62000660EACA",
            "--shot",
            "target/test.png",
        ]))
        .unwrap();
        assert_eq!(parsed.creature, 0x406B62000660EACA);
        assert_eq!(parsed.shot, PathBuf::from("target/test.png"));
        assert!(Options::parse(&args(&[
            "--creature",
            "bad-id",
            "--shot",
            "target/test.png"
        ]))
        .is_err());
    }
    #[test]
    fn second_frame_requires_complete_finite_explicit_inputs() {
        let prefix = ["--creature", "40616200A1C46DF2", "--shot", "color.png", "--coverage", "mask.png"];
        let mut full = args(&prefix);
        full.extend(args(&["--reference-shot", "reference.png", "--second-shot", "second.png",
                           "--second-mask", "second-mask.png", "--second-offset", "0.1,0,-0.25"]));
        assert!(Options::parse(&full).is_ok());
        let mut invalid = full.clone();
        *invalid.last_mut().unwrap() = "NaN,0,0".into();
        assert!(Options::parse(&invalid).is_err());
        invalid = full.clone();
        *invalid.last_mut().unwrap() = "0,0".into();
        assert!(Options::parse(&invalid).is_err());
        assert!(Options::parse(&full[..full.len()-2]).is_err());
    }
    #[test]
    fn missing_output_identity_and_unknown_options_are_errors() {
        assert!(Options::parse(&args(&["--creature", "406B62000660EACA"])).is_err());
        assert!(Options::parse(&args(&["--shot", "target/test.png"])).is_err());
        assert!(Options::parse(&args(&["--creature"])).is_err());
        assert!(Options::parse(&args(&[
            "--model",
            "406B62000660EACA",
            "--shot",
            "target/test.png"
        ]))
        .is_err());
        assert!(Options::parse(&args(&["--creature", "40616200A1C46DF2", "--shot", "same.png", "--coverage", "same.png"])).is_err());
        assert!(Options::parse(&args(&["--creature", "406B62000660EACA", "--shot", "color.png", "--coverage", "mask.png"])).is_err());
        assert!(Options::parse(&args(&["--creature", "40616200A1C46DF2", "--shot", "color.png", "--coverage"])).is_err());
        assert!(Options::parse(&args(&["--creature", "40616200A1C46DF2", "--shot", "color.png", "--coverage", ""])).is_err());
        assert!(Options::parse(&args(&["--creature", "40616200A1C46DF2", "--shot", "color.png", "--coverage", "mask.png", "--coverage", "other.png"])).is_err());
        assert_eq!(Options::parse(&args(&["--creature", "40616200A1C46DF2", "--shot", "color.png", "--coverage", "mask.png"])).unwrap().coverage, Some(PathBuf::from("mask.png")));
    }
}
