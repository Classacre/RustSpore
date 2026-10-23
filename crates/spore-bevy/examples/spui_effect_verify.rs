//! Verify source FadeEffect parameters and bounded native linear ticks without a renderer.
use spore_core::{
    spui,
    spui_effect::{FADE_EFFECT_CLASS, FadeError, FadeParameters},
};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

fn collect(path: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect(&entry.path(), paths)?;
        } else if entry
            .path()
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("package"))
        {
            paths.push(entry.path());
        }
    }
    Ok(())
}
fn color(value: Option<u32>) -> String {
    value
        .map(|v| format!("{v:08X}"))
        .unwrap_or_else(|| "-".into())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: spui_effect_verify <install> <new FadeEffect TSV>".into());
    }
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])?;
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    let (mut packages, mut resources, mut fades, mut linear) = (0, 0, 0, 0);
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x250f_e9a2)
        {
            resources += 1;
            let file = spui::parse(&index.read(entry)?)?;
            let body = spui::parse_elements(&file)?;
            for element in body
                .elements
                .iter()
                .filter(|e| e.proxy_id == FADE_EFFECT_CLASS)
            {
                let parameters = FadeParameters::from_element(element)?;
                let (status, result) = match parameters.linear() {
                    Ok(mut fade) => {
                        linear += 1;
                        let initial = fade.attach(true);
                        fade.visibility_changed(false)?;
                        let first = fade.tick(1.0f32 / 60.)?;
                        fade.visibility_changed(true)?;
                        let second = fade.tick(1.0f32 / 60.)?;
                        (
                            "linear",
                            format!(
                                "{initial:08X}\t{:08X}\t{}\t{}\t{:08X}\t{}\t{}",
                                first.progress.to_bits(),
                                color(first.shade_color),
                                u8::from(first.completed),
                                second.progress.to_bits(),
                                color(second.shade_color),
                                u8::from(second.completed)
                            ),
                        )
                    }
                    Err(FadeError::UnsupportedInterpolation(_)) => {
                        ("unsupported-interpolation", "-\t-\t-\t-\t-\t-\t-".into())
                    }
                    Err(FadeError::UnsupportedTrigger(_)) => {
                        ("unsupported-trigger", "-\t-\t-\t-\t-\t-\t-".into())
                    }
                    Err(FadeError::InvalidDuration) => {
                        ("invalid-duration", "-\t-\t-\t-\t-\t-\t-".into())
                    }
                    Err(error) => return Err(error.into()),
                };
                writeln!(
                    output,
                    "{}\t{:016X}\t{}\t{:08X}\t{:08X}\t{:08X}\t{status}\t{result}",
                    path.file_name().unwrap().to_string_lossy(),
                    entry.key.instance_id,
                    element.index,
                    parameters.duration_seconds.to_bits(),
                    parameters.trigger,
                    parameters.interpolation
                )?;
                fades += 1;
            }
        }
    }
    println!(
        "source FadeEffect packages={packages} resources={resources} fades={fades} linear={linear}; no complete UI claim"
    );
    Ok(())
}
