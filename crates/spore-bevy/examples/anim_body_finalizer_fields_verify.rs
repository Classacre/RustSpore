//! Stream independently prepared native finalizer stage inputs through Rust.
#[path="support/optional_foot_protocol.rs"] mod protocol;
use spore_core::{anim_body_finalizer_fields::{reroot_anchor_fields,ordered_local_orientations,
    relabel_ordered_capabilities},native_x87::Context};
use std::io::{Read,Write};
const RECORD:usize=324;
const INPUT:usize=32+255*RECORD;
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args=std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len()!=2 { return Err("usage: anim_body_finalizer_fields_verify inputs.bin new-output.bin".into()); }
    let mut input=std::io::BufReader::new(std::fs::File::open(&args[0])?);
    let mut output=std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    let mut request=vec![0u8;INPUT];let mut rows=0usize;
    loop {
        if input.read(&mut request[..1])?==0 { break; }
        input.read_exact(&mut request[1..])?;
        let control=u16::from_le_bytes(request[..2].try_into().unwrap());
        let context=Context::from_control_word(control)?;
        if request[2..4]!=[0,0] || protocol::word(&request,28)!=0 { return Err("nonzero reserved header".into()); }
        let mode=protocol::word(&request,4);let count=protocol::word(&request,8) as usize;
        if !(1..=255).contains(&count) || mode>2 { return Err("fixture mode/count outside bound".into()); }
        let selected=protocol::word(&request,12) as usize;
        let starts=[protocol::word(&request,16) as usize,protocol::word(&request,20) as usize];
        let no_foot=protocol::word(&request,24);if no_foot>1 { return Err("nonboolean no-foot flag".into()); }
        if mode==0 && (starts!=[0,0] || no_foot!=0) || mode==1 && (selected!=0 || starts!=[0,0] || no_foot!=0)
            || mode==2 && selected!=0 { return Err("nonzero unconsumed header fields".into()); }
        let records=request[32..32+count*RECORD].chunks_exact(RECORD).collect::<Vec<_>>();
        let blocks=records.iter().map(|row|protocol::decode(&row[..protocol::RECORD_BYTES])).collect::<Vec<_>>();
        if request[32+count*RECORD..].iter().any(|value|*value!=0) { return Err("nonzero fixed padding".into()); }
        let result=match mode {
            0=>reroot_anchor_fields(&blocks,selected,context)?,
            1=>ordered_local_orientations(&blocks)?,
            2=>{
                let children=records.iter().zip(&blocks).map(|(row,block)| {
                    let start=protocol::word(row,316) as usize;start..start+block.child_count as usize
                }).collect::<Vec<_>>();
                relabel_ordered_capabilities(&blocks,&children,starts,no_foot!=0)?
            },
            _=>unreachable!(),
        };
        output.write_all(&request[..32])?;
        for (block,record) in result.iter().zip(records) {
            output.write_all(&protocol::encode(block))?;output.write_all(&record[316..324])?;
        }
        let zeros=[0u8;RECORD];for _ in count..255 { output.write_all(&zeros)?; }
        rows+=1;
    }
    output.flush()?;println!("rows={rows} projection_bytes={INPUT}");Ok(())
}
