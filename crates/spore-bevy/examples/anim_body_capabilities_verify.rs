//! Rust-only projection of separately reviewed capability input records.
use spore_core::{
    anim_body_capabilities::{prepare_capabilities, CapabilitySlots},
    native_x87::Context,
    part_capabilities::{CapabilityAggregate, PartCapabilities},
};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: anim_body_capabilities_verify <inputs.bin> <new-output.bin>".into());
    }
    let inputs = std::fs::read(&args[0])?;
    if inputs.is_empty() || inputs.len() % 1264 != 0 {
        return Err("expected nonempty complete1264-byte records".into());
    }
    let mut output = std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    for (n, record) in inputs.chunks_exact(1264).enumerate() {
        let word = |at| u32::from_le_bytes(record[at..at + 4].try_into().unwrap());
        if word(0) > u32::from(u16::MAX) || word(4) > 8 || word(8) != 0 || word(12) != 0 {
            return Err(format!("unexpected record header {n}").into());
        }
        Context::from_control_word(word(0) as u16)?;
        let slots = CapabilitySlots {
            ids: std::array::from_fn(|i| word(16 + i * 4)),
            numbers: record[144..176].try_into().unwrap(),
        };
        let mut aggregates = Vec::new();
        for i in 0..word(4) as usize {
            let at = 176 + i * 136;
            let count = word(at + 4) as usize;
            if count > 32 {
                return Err(format!("unexpected aggregate extent in record {n}").into());
            }
            aggregates.push(CapabilityAggregate {
                capability: word(at),
                members: (0..count).map(|j| word(at + 8 + j * 4)).collect(),
            });
        }
        let dictionary = PartCapabilities {
            version: 4,
            priority_bits: None,
            names: vec![],
            remaps: vec![],
            aggregates,
            deforms: vec![],
        };
        let prepared = prepare_capabilities(slots, &dictionary)
            .map_err(|error| format!("record {n}: {error}"))?;
        for value in [prepared.presence_mask(), prepared.nonzero_mask()]
            .into_iter()
            .chain(prepared.slots().ids)
        {
            output.write_all(&value.to_le_bytes())?;
        }
        output.write_all(&prepared.slots().numbers)?;
    }
    output.flush()?;
    println!("rows={} original_host_executed=false", inputs.len() / 1264);
    Ok(())
}
