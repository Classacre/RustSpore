//! Fixed-record adapter; caller supplies the original-host output after review.
use spore_core::{cell_ui_progress_selection_v1::select,native_x87::Context};
use std::{error::Error,io::{self,Read,Write}};
fn main()->Result<(),Box<dyn Error>> {
    let mut bytes=Vec::new(); io::stdin().take(4097).read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len()>4096 || bytes.len()%28!=0 { return Err("bad host record extent".into()); }
    let mut out=io::BufWriter::new(io::stdout().lock());
    for row in bytes.chunks_exact(28) {
        let word=|offset|u32::from_le_bytes(row[offset..offset+4].try_into().unwrap());
        let cw=word(0); let input=word(4);
        if !matches!(cw,0x027f|0x037f) {return Err("unsupported x87 control".into())}
        let v=select(f32::from_bits(input),Context::from_control_word(cw as u16)?).map_err(|e|format!("model {e:?}"))?;
        writeln!(out,"{cw:08X} {input:08X} {:08X} {:08X} {:08X}",v.segment,v.bucket,v.fraction.to_bits())?;
    }
    out.flush()?; Ok(())
}
