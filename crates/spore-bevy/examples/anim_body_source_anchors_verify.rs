//! Fresh canonical source anchor comparison, with an explicit source dictionary.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    anim_body_capabilities::prepare_source_capabilities,
    anim_body_source_anchors::source_body_anchors_for_creature,
    creature_data, native_x87::Context, part_capabilities::{self, PartCapabilities},
};
use std::{io::Write, path::{Path, PathBuf}};

fn collect(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry=entry?;
            if !entry.file_type()?.is_symlink() { collect(&entry.path(),out)?; }
        }
    } else if path.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("package")) {
        out.push(path.to_path_buf());
    }
    Ok(())
}
struct Digest(u64);
impl Digest {
    fn bytes(&mut self, bytes: &[u8]) {
        for byte in bytes { self.0=(self.0 ^ u64::from(*byte)).wrapping_mul(0x100000001B3); }
    }
    fn word(&mut self, word: u32) { self.bytes(&word.to_le_bytes()); }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args=std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len()!=2 { return Err("usage: anim_body_source_anchors_verify install new-digests.txt".into()); }
    let install=Path::new(&args[0]);
    let mut packages=Vec::new();collect(install,&mut packages)?;packages.sort();packages.dedup();
    let mut dictionary: Option<PartCapabilities>=None;
    let mut dictionaries=0usize;
    for package in &packages {
        let index=NativePackageIndex::open(package)?;
        for entry in index.entries().iter().filter(|e| e.key.type_id==0x7C19AA7A) {
            let value=part_capabilities::parse(&index.read(entry)?)?;
            if let Some(first)=&dictionary {
                if first.aggregates!=value.aggregates {
                    return Err("source aggregate tables differ; select a dictionary".into());
                }
            } else { dictionary=Some(value); }
            dictionaries+=1;
        }
    }
    let dictionary=dictionary.ok_or("no source aggregate dictionary")?;
    let (mut resources,mut rows,mut evaluated)=(0usize,0usize,0usize);
    let mut lines=Vec::new();
    for package in &packages {
        let index=NativePackageIndex::open(package)?;
        for entry in index.entries().iter().filter(|e| e.key.type_id==0x0F43029A) {
            let source=creature_data::parse(&index.read(entry)?)?;
            if !source.exact { return Err("source CreatureData unconsumed bytes".into()); }
            let masks=(0..source.parts.len()).map(|row|
                prepare_source_capabilities(&source,row,&dictionary).map(|v| v.presence_mask())
            ).collect::<Result<Vec<_>,_>>()?;
            for control in [0x007F,0x027F,0x037F] {
                let anchors=source_body_anchors_for_creature(&source,&dictionary,Context::from_control_word(control)?)?;
                let mut digest=Digest(0xCBF29CE484222325);
                digest.bytes(b"native-body-source-anchors-v1\0");
                digest.word(u32::from(control));digest.word(anchors.len() as u32);
                for (row,value) in anchors.iter().enumerate() {
                    if value.source_palette_id!=row as u32 { return Err("source palette identity changed".into()); }
                    digest.word(row as u32);digest.bytes(&source.data0[row*140..(row+1)*140]);
                    digest.word(masks[row]);digest.word(value.source_parent.unwrap_or(u32::MAX));digest.word(value.child_count);
                    for word in value.child_anchor.iter().chain(&value.parent_anchor).chain(&value.parent_delta)
                        .chain(std::iter::once(&value.parent_delta_length)) {
                        digest.word(word.to_bits());
                    }
                    evaluated+=1;
                }
                let key=entry.key;
                let relative=package.strip_prefix(install)?.to_string_lossy().replace('\\',"/");
                lines.push(format!("{relative} {:08X}:{:08X}:{:08X} {control:04X} {:016X}",
                    key.type_id,key.group_id,key.instance_id,digest.0));
            }
            resources+=1;rows+=source.parts.len();
        }
    }
    lines.sort();
    let mut out=std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    for line in &lines { writeln!(out,"{line}")?; }
    out.flush()?;
    println!("indexes={} dictionaries={dictionaries} resources={resources} rows={rows} resource_contexts={} evaluated_rows={evaluated}",packages.len(),lines.len());
    Ok(())
}
