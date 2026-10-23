//! Fresh canonical source intake through optional construction/final fields.
#[path="support/optional_foot_protocol.rs"] mod protocol;
use dbpf::native_index::NativePackageIndex;
use spore_core::{anim_body_finalizer_fields::finalize_body_fields,
    anim_body_optional_feet::{source_optional_feet,FootConstructionBlock},
    anim_body_topology::{select_root,RootPolicy,TopologyError},creature_data,native_x87::Context,
    part_capabilities::{self,PartCapabilities}};
use std::{io::Write,path::{Path,PathBuf}};
fn collect(path:&Path,out:&mut Vec<PathBuf>)->std::io::Result<()> {
    if path.is_dir() { for entry in std::fs::read_dir(path)? { let entry=entry?;
        if !entry.file_type()?.is_symlink() { collect(&entry.path(),out)?; }
    }} else if path.is_file() && path.extension().is_some_and(|e|e.eq_ignore_ascii_case("package")) { out.push(path.to_path_buf()); }
    Ok(())
}
struct Digest(u64);
impl Digest {
    fn bytes(&mut self,data:&[u8]) { for byte in data { self.0=(self.0^u64::from(*byte)).wrapping_mul(0x100000001B3); } }
    fn word(&mut self,value:u32) { self.bytes(&value.to_le_bytes()); }
    fn record(&mut self,block:&FootConstructionBlock,child_start:u32,descendants:u32) {
        self.bytes(&protocol::encode(block));self.word(child_start);self.word(descendants);
    }
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args=std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len()!=2 { return Err("usage: anim_body_finalizer_source_verify install new-digests.txt".into()); }
    let install=Path::new(&args[0]);let mut packages=Vec::new();collect(install,&mut packages)?;packages.sort();packages.dedup();
    let mut dictionary:Option<PartCapabilities>=None;let mut dictionaries=0usize;
    for package in &packages {
        let index=NativePackageIndex::open(package)?;
        for entry in index.entries().iter().filter(|entry|entry.key.type_id==0x7C19AA7A) {
            let value=part_capabilities::parse(&index.read(entry)?)?;
            if let Some(first)=&dictionary {
                if first.aggregates!=value.aggregates { return Err("source aggregates differ; choose explicit dictionary".into()); }
            } else { dictionary=Some(value); }
            dictionaries+=1;
        }
    }
    let dictionary=dictionary.ok_or("no source dictionary")?;
    let policy=RootPolicy { nearest_origin:false,spine_midpoint_fallback:true };
    let (mut resources,mut rows,mut admitted,mut rejected)=(0usize,0usize,0usize,0usize);
    let mut lines=Vec::new();
    for package in &packages {
        let index=NativePackageIndex::open(package)?;
        for entry in index.entries().iter().filter(|entry|entry.key.type_id==0x0F43029A) {
            let source=creature_data::parse(&index.read(entry)?)?;
            if !source.exact { return Err("unconsumed CreatureData bytes".into()); }
            for control in [0x007F,0x027F,0x037F] {
                let context=Context::from_control_word(control)?;
                let optional=source_optional_feet(&source,&dictionary,context,f32::from_bits(0x3DCCCCCD),false)?;
                let result=&optional.result;let blocks=&result.blocks;
                let mut digest=Digest(0xCBF29CE484222325);digest.bytes(b"native-body-finalizer-fields-source-v1\0");
                for word in [u32::from(control),0x3DCCCCCD,2,source.parts.len() as u32,optional.root_nomination as u32,
                    u32::from(result.driver_called),blocks.len() as u32,result.appended_blocks as u32,
                    u32::from(result.field3f5),result.append_batches as u32] { digest.word(word); }
                for mapping in [&optional.source_to_initial,&optional.initial_to_source] {
                    for &value in mapping { digest.word(value as u32); }
                }
                digest.bytes(&source.data0);
                for block in blocks { digest.record(block,0,0); }
                let topology=blocks.iter().map(FootConstructionBlock::topology).collect::<Vec<_>>();
                let selected=select_root(&topology,policy);
                let admission=match &selected {
                    Ok(_)=>0,
                    Err(TopologyError::InitialRootMissingNonzeroSpine)=>1,
                    Err(TopologyError::SelectedRootMissingNonzeroSpine(_))=>2,
                    Err(TopologyError::NoSelectedRoot)=>3,
                    Err(error)=>return Err(format!("unexpected topology error: {error}").into()),
                };
                digest.word(admission);
                if let Ok(selection)=selected {
                    let finalized=finalize_body_fields(blocks,selection.root,selection.no_foot_candidates,context)?;
                    for word in [selection.root as u32,u32::from(selection.no_foot_candidates),
                        selection.foot_candidates.len() as u32,selection.foot_votes.iter().copied().max().unwrap_or(0)] { digest.word(word); }
                    for i in 0..255 { digest.word(selection.foot_votes.get(i).copied().unwrap_or(0)); }
                    for i in 0..255 { digest.word(selection.foot_candidates.get(i).map_or(0,|value|*value as u32)); }
                    for mapping in [&finalized.order.new_to_old,&finalized.order.old_to_new] {
                        for &value in mapping { digest.word(value as u32); }
                    }
                    for word in [finalized.initial_root_in_final_order as u32,finalized.last_spine_in_final_order as u32,
                        finalized.source_palette_prefix_count as u32,finalized.blocks.len() as u32] { digest.word(word); }
                    for (i,block) in finalized.blocks.iter().enumerate() {
                        let order=&finalized.order.blocks[i];
                        digest.record(block,order.children.start as u32,order.descendants.len() as u32);
                    }
                    admitted+=1;
                } else { rejected+=1; }
                let relative=package.strip_prefix(install)?.to_string_lossy().replace('\\',"/");let key=entry.key;
                lines.push(format!("{relative} {:08X}:{:08X}:{:08X} {control:04X} {admission} {:016X}",
                    key.type_id,key.group_id,key.instance_id,digest.0));
            }
            resources+=1;rows+=source.parts.len();
        }
    }
    lines.sort();let mut output=std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    for line in &lines { writeln!(output,"{line}")?; }output.flush()?;
    println!("indexes={} dictionaries={dictionaries} resources={resources} source_rows={rows} contexts={} admitted={admitted} rejected={rejected}",packages.len(),lines.len());
    Ok(())
}
