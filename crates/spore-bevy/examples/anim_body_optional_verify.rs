//! Stream bounded native optional-foot fixtures through the Rust projection.
#[path="support/optional_foot_protocol.rs"] mod protocol;
use spore_core::{anim_body_optional_feet::run_optional_foot_driver,native_x87::Context};
use std::io::{Read,Write};
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args=std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len()!=2 { return Err("usage: anim_body_optional_verify inputs.bin new-output.bin".into()); }
    let mut input=std::io::BufReader::new(std::fs::File::open(&args[0])?);
    let mut output=std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    let mut request=vec![0u8;protocol::INPUT_BYTES];let mut rows=0usize;let mut appended=0usize;
    loop {
        if input.read(&mut request[..1])?==0 { break; }
        input.read_exact(&mut request[1..])?;
        let control=u16::from_le_bytes(request[..2].try_into().unwrap());Context::from_control_word(control)?;
        if request[2..4]!=[0,0] { return Err("nonzero reserved header".into()); }
        let count=protocol::word(&request,4) as usize;
        if !(1..=255).contains(&count) { return Err("fixture count outside1..255".into()); }
        let flag=protocol::word(&request,12);if flag>1 { return Err("nonboolean3F5".into()); }
        let blocks=request[16..16+count*protocol::RECORD_BYTES].chunks_exact(protocol::RECORD_BYTES)
            .map(protocol::decode).collect::<Vec<_>>();
        if request[16+count*protocol::RECORD_BYTES..].iter().any(|v|*v!=0) { return Err("nonzero fixed padding".into()); }
        let result=run_optional_foot_driver(&blocks,f32::from_bits(protocol::word(&request,8)),flag!=0)?;
        for w in [result.blocks.len() as u32,result.appended_blocks as u32,u32::from(result.field3f5),result.append_batches as u32] {
            output.write_all(&w.to_le_bytes())?;
        }
        for block in &result.blocks { output.write_all(&protocol::encode(block))?; }
        let zero=[0u8;protocol::RECORD_BYTES];for _ in result.blocks.len()..255 { output.write_all(&zero)?; }
        rows+=1;appended+=result.appended_blocks;
    }
    output.flush()?;println!("rows={rows} appended_blocks={appended}");Ok(())
}
