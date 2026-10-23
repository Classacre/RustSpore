//! Offscreen saved-creature capture through authored references and source shaders.
//! The fresh lighting context and fitted camera are explicit inspection inputs.
use crate::assets::{AssetSource, PackageSet};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
struct Options {
    install: PathBuf,
    creature: u64,
    shot: PathBuf,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut install = std::env::var_os("RUSTSPORE_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("D:/Games/SPORE Collection"));
        let mut creature = None;
        let mut shot = None;
        let mut index = 0;
        while index < args.len() {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{} requires a value", args[index]))?;
            match args[index].as_str() {
                "--install" => install = PathBuf::from(value),
                "--creature" => creature = Some(crate::parse_hex_arg(value, "creature")?),
                "--shot" => shot = Some(PathBuf::from(value)),
                other => return Err(format!("unknown source creature option {other:?}")),
            }
            index += 2;
        }
        Ok(Self {
            install,
            creature: creature.ok_or("--source-creature requires --creature <hex>")?,
            shot: shot.ok_or("--source-creature requires --shot <new.png>")?,
        })
    }
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let options = Options::parse(args)?;
    if options.shot.exists() {
        return Err("capture output already exists; choose a new path".into());
    }
    let source = AssetSource::open(options.install.join("Data/PatchData.package"))?
        .with_set(PackageSet::discover(&options.install));
    let authored = source
        .authored_creature_model(options.creature, 0)
        .map_err(|e| e.to_string())?;
    println!("SOURCE_CREATURE_ART creation={} prop={} lod={} model={} renderer=original-d3d9 animation=unimplemented scene=inspection",
        authored.creature, authored.prop, authored.lod, authored.model);
    #[cfg(all(windows, feature = "native_renderer"))]
    return crate::native_cell_renderer::capture_creature_key(
        &options.install,
        &options.shot,
        authored.model,
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
    }
}
