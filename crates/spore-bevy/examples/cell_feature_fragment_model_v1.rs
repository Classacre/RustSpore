//! Adapter for the fixed source-feature projection corpus; does not execute native bytes.
#[path = "../src/cell_source_feature_geometry_v1.rs"]
mod feature;
use spore_core::cell_part_geometry::NativeCellPart;
use std::{env,fs,io::{self,Write}};
fn word(s:&[u8],o:usize)->u32{u32::from_le_bytes(s[o..o+4].try_into().unwrap())}
fn main()->Result<(),Box<dyn std::error::Error>>{
    let args:Vec<_>=env::args_os().collect();
    if args.len()!=2 {return Err("expected corpus path".into())}
    let data=fs::read(&args[1])?;
    if data.len()!=6*200 {return Err("unexpected corpus extent".into())}
    let mut out=Vec::new();
    for case in data.chunks_exact(200) {
        let a=&case[..128];let r=&case[128..];
        let row=NativeCellPart{word_06_is_ffff:false,class:2,center:[f32::from_bits(word(r,8)),f32::from_bits(word(r,12)),f32::from_bits(word(r,16))],matrix_bits:[0;9],radius:f32::from_bits(word(r,56)),tail_bits:[0;3]};
        let t=feature::Transform{position:[word(a,0x4c),word(a,0x50),word(a,0x54)],scale:word(a,0x58),flags:a[0x48],matrix:std::array::from_fn(|i|word(a,0x5c+4*i))};
        let p=feature::project(&row,t).map_err(|e|format!("projection: {e:?}"))?;
        for v in p.center.into_iter().chain([p.radius]) {out.extend(v.to_le_bytes());}
    }
    io::stdout().write_all(&out)?; Ok(())
}
