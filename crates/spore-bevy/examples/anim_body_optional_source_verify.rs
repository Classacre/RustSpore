//! Fresh canonical source intake for the actual optional construction gate.
#[path="support/optional_foot_protocol.rs"] mod protocol;
use dbpf::native_index::NativePackageIndex;
use spore_core::{anim_body_optional_feet::source_optional_feet,creature_data,native_x87::Context,
    part_capabilities::{self,PartCapabilities}};
use std::{io::Write,path::{Path,PathBuf}};
fn collect(path:&Path,out:&mut Vec<PathBuf>)->std::io::Result<()> {
    if path.is_dir() { for entry in std::fs::read_dir(path)? { let entry=entry?;
        if !entry.file_type()?.is_symlink() { collect(&entry.path(),out)?; } }
    } else if path.is_file() && path.extension().is_some_and(|e|e.eq_ignore_ascii_case("package")) { out.push(path.to_path_buf()); }
    Ok(())
}
struct Digest(u64);
impl Digest {
    fn bytes(&mut self,data:&[u8]) { for b in data { self.0=(self.0^u64::from(*b)).wrapping_mul(0x100000001B3); } }
    fn word(&mut self,v:u32) { self.bytes(&v.to_le_bytes()); }
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args=std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len()!=2 { return Err("usage: anim_body_optional_source_verify install new-digests.txt".into()); }
    let install=Path::new(&args[0]);let mut packages=Vec::new();collect(install,&mut packages)?;packages.sort();packages.dedup();
    let mut dictionary:Option<PartCapabilities>=None;let mut dictionaries=0usize;
    for p in &packages {
        let index=NativePackageIndex::open(p)?;
        for e in index.entries().iter().filter(|e|e.key.type_id==0x7C19AA7A) {
            let value=part_capabilities::parse(&index.read(e)?)?;
            if let Some(first)=&dictionary { if first.aggregates!=value.aggregates { return Err("source aggregates differ; select dictionary".into()); } }
            else { dictionary=Some(value); }dictionaries+=1;
        }
    }
    let dictionary=dictionary.ok_or("no source dictionary")?;
    let (mut resources,mut rows,mut call_contexts,mut appended)=(0usize,0usize,0usize,0usize);
    let mut lines=Vec::new();
    for p in &packages {
        let index=NativePackageIndex::open(p)?;
        for e in index.entries().iter().filter(|e|e.key.type_id==0x0F43029A) {
            let source=creature_data::parse(&index.read(e)?)?;if !source.exact { return Err("unconsumed source CreatureData".into()); }
            for control in [0x007F,0x027F,0x037F] { for flag in [false,true] {
                let prepared=source_optional_feet(&source,&dictionary,Context::from_control_word(control)?,f32::from_bits(0x3DCCCCCD),flag)?;
                let result=&prepared.result;let mut digest=Digest(0xCBF29CE484222325);digest.bytes(b"native-body-optional-source-v1\0");
                for w in [u32::from(control),u32::from(flag),prepared.root_nomination as u32,source.parts.len() as u32,u32::from(result.driver_called)] { digest.word(w); }
                for map in [&prepared.source_to_initial,&prepared.initial_to_source] { for &v in map { digest.word(v as u32); } }
                digest.bytes(&source.data0);
                for w in [result.blocks.len() as u32,result.appended_blocks as u32,u32::from(result.field3f5),result.append_batches as u32] { digest.word(w); }
                for b in &result.blocks { digest.bytes(&protocol::encode(b)); }
                let relative=p.strip_prefix(install)?.to_string_lossy().replace('\\',"/");let key=e.key;
                lines.push(format!("{relative} {:08X}:{:08X}:{:08X} {control:04X} {} {:016X}",key.type_id,key.group_id,key.instance_id,u32::from(flag),digest.0));
                call_contexts+=usize::from(result.driver_called);appended+=result.appended_blocks;
            }}
            resources+=1;rows+=source.parts.len();
        }
    }
    lines.sort();let mut out=std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    for line in &lines { writeln!(out,"{line}")?; }out.flush()?;
    println!("indexes={} dictionaries={dictionaries} resources={resources} rows={rows} contexts={} call_contexts={call_contexts} appended_blocks={appended}",packages.len(),lines.len());Ok(())
}
