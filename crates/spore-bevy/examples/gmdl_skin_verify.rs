//! Full-install GMDL-tail and influence verification. Raw source assets stay local.
//! tools/build.bat run -p spore-bevy --example gmdl_skin_verify -- <install> --digests <out.tsv>
use spore_core::gmdl_skin::{self, GmdlSkin, GmdlTransform};
use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

fn collect(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for e in std::fs::read_dir(path)? {
            let e = e?;
            if !e.file_type()?.is_symlink() {
                collect(&e.path(), out)?;
            }
        }
    } else if path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("package"))
    {
        out.push(path.to_owned());
    } else if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            path.display().to_string(),
        ));
    }
    Ok(())
}
struct Fnv(u64);
impl Fnv {
    fn new() -> Self {
        Self(0xCBF29CE484222325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for v in b {
            self.0 = (self.0 ^ u64::from(*v)).wrapping_mul(0x100000001B3);
        }
    }
    fn word(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
    fn transform(&mut self, t: GmdlTransform) {
        self.word(u32::from(t.flags));
        self.word(u32::from(t.transform_count));
        for v in t.translation {
            self.word(v.to_bits());
        }
        self.word(t.scale.to_bits());
        for v in t.rotation.into_iter().flatten() {
            self.word(v.to_bits());
        }
    }
}
fn digest(s: &GmdlSkin) -> Fnv {
    let mut h = Fnv::new();
    h.word(s.source_offset as u32);
    h.word(s.materials.len() as u32);
    for m in &s.materials {
        h.word(u32::from(m.version9_prefix.is_some()));
        if let Some(v) = m.version9_prefix {
            h.word(v);
        }
        h.word(m.shader_data.len() as u32);
        for d in &m.shader_data {
            h.word(d.id);
            h.word(d.payload.len() as u32);
            h.bytes(&d.payload);
        }
    }
    h.word(s.palettes.len() as u32);
    for p in &s.palettes {
        h.word(p.first_word);
        h.word(p.bone_count);
    }
    h.word(s.parts.len() as u32);
    for p in &s.parts {
        h.word(p.source_offset as u32);
        h.transform(p.transform_a);
        h.transform(p.transform_b);
        h.word(p.count_or_sentinel);
        for v in p.model_key_words {
            h.word(v);
        }
        h.word(p.bone_presence);
        h.word(p.bone_data_offset as u32);
        h.word(p.bones.len() as u32);
        for b in &p.bones {
            for v in b.words {
                h.word(v);
            }
        }
    }
    for v in s.footer {
        h.word(v);
    }
    h
}
fn main() {
    let mut args = std::env::args_os().skip(1);
    let mut packages = Vec::new();
    let mut output = None;
    while let Some(arg) = args.next() {
        if arg == "--digests" {
            let mut writer = BufWriter::new(
                std::fs::File::create(args.next().expect("digest path"))
                    .expect("create digest file"),
            );
            writeln!(writer, "package\tresource_key\tkind\tcount\tfnv64").unwrap();
            output = Some(writer);
        } else {
            collect(Path::new(&arg), &mut packages).expect("collect packages");
        }
    }
    packages.sort();
    packages.dedup();
    if packages.is_empty() {
        eprintln!("supply package/directory");
        std::process::exit(2);
    }
    let (
        mut opened,
        mut models,
        mut tails,
        mut parts,
        mut bones,
        mut meshes,
        mut vertices,
        mut equations,
        mut problems,
    ) = (
        0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize,
    );
    let mut unsupported = BTreeMap::<String, usize>::new();
    let mut palette_words = BTreeMap::<u32, usize>::new();
    let mut palette_lengths = BTreeMap::<usize, usize>::new();
    let mut model_versions = BTreeMap::<i32, usize>::new();
    let (
        mut palette_models,
        mut cumulative_palettes,
        mut descriptor_mesh_count,
        mut populated_equations,
    ) = (0usize, 0usize, 0usize, 0usize);
    for path in &packages {
        let index = match dbpf::PackageIndex::open(path) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                problems += 1;
                continue;
            }
        };
        opened += 1;
        let mut local = 0;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x00E6BCE5)
        {
            models += 1;
            local += 1;
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let data = index.read(entry)?;
                let model = spore_core::gmdl::parse(&data)?;
                *model_versions.entry(model.version).or_default() += 1;
                let Some(skin) = gmdl_skin::parse(&data, &model)? else {
                    return Ok(());
                };
                tails += 1;
                *palette_lengths.entry(skin.palettes.len()).or_default() += 1;
                for p in &skin.palettes {
                    *palette_words.entry(p.first_word).or_default() += 1;
                }
                if !skin.palettes.is_empty() {
                    palette_models += 1;
                    let mut end = 0u32;
                    cumulative_palettes += usize::from(skin.palettes.iter().all(|p| {
                        let matches = p.first_word == end;
                        end = end.saturating_add(p.bone_count);
                        matches
                    }));
                    descriptor_mesh_count += usize::from(skin.palettes.len() == model.meshes.len());
                    populated_equations += usize::from(
                        skin.part_palette_count()
                            == Some(skin.palettes.iter().map(|p| p.bone_count as usize).sum()),
                    );
                }
                parts += skin.parts.len();
                bones += skin.parts.iter().map(|p| p.bones.len()).sum::<usize>();
                equations += usize::from(
                    skin.part_palette_count()
                        == Some(skin.palettes.iter().map(|p| p.bone_count as usize).sum()),
                );
                if let Some(w) = &mut output {
                    writeln!(
                        w,
                        "{}\t{}\ttail\t{}\t{:016X}",
                        path.display(),
                        entry.key,
                        skin.parts.len(),
                        digest(&skin).0
                    )?;
                }
                for i in 0..model.meshes.len() {
                    match gmdl_skin::vertex_skinning(&data, &model, &skin, i) {
                        Ok(values) => {
                            meshes += 1;
                            vertices += values.len();
                            let mut h = Fnv::new();
                            h.word(values.len() as u32);
                            for v in &values {
                                for j in v.joints {
                                    h.word(u32::from(j));
                                }
                                for w in v.weights {
                                    h.word(w.to_bits());
                                }
                            }
                            if let Some(w) = &mut output {
                                writeln!(
                                    w,
                                    "{}\t{}\tinfluences:{i}\t{}\t{:016X}",
                                    path.display(),
                                    entry.key,
                                    values.len(),
                                    h.0
                                )?;
                            }
                        }
                        Err(e) => {
                            let count = unsupported.entry(e.to_string()).or_default();
                            *count += 1;
                            if *count <= 3 {
                                eprintln!(
                                    "unsupported {} {} mesh{i}: {e} palettes={:?}",
                                    path.display(),
                                    entry.key,
                                    skin.palettes
                                );
                            }
                        }
                    }
                }
                Ok(())
            })();
            if let Err(e) = result {
                problems += 1;
                eprintln!("{} {}: {e}", path.display(), entry.key);
            }
        }
        if local > 0 {
            println!("package={} gmdl={local}", path.display());
        }
    }
    println!("packages={opened}/{} gmdl={models} tails={tails} parts={parts} bone_records={bones} palette_equations={equations}/{tails} skinned_meshes={meshes} skinned_vertices={vertices} problems={problems}",packages.len());
    println!("unsupported_skinning={unsupported:?}");
    println!("versions={model_versions:?} palette_first_words={palette_words:?} palette_lengths={palette_lengths:?}");
    println!("populated_palettes={palette_models} cumulative_first_words={cumulative_palettes} descriptor_count_matches_meshes={descriptor_mesh_count} populated_part_count_equations={populated_equations}");
    if problems > 0 || models == 0 {
        std::process::exit(1);
    }
}
