//! Compare every typed source float color with independent local fixture bits.
use spore_core::prop::{self, PropValue};
use std::io::Read;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: prop_color_verify <RSCOLOR1 corpus>")?;
    let mut input = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut header = [0; 12];
    input.read_exact(&mut header)?;
    if &header[..8] != b"RSCOLOR1" {
        return Err("wrong color fixture header".into());
    }
    let count = u32::from_le_bytes(header[8..].try_into()?);
    if count == 0 || count > 100_000 {
        return Err("invalid fixture count".into());
    }
    let (mut fields, mut colors, mut components) = (0, 0, 0);
    for ordinal in 0..count {
        input.read_exact(&mut header)?;
        let length = u32::from_le_bytes(header[..4].try_into()?) as usize;
        if length > 32 * 1024 * 1024 {
            return Err("fixture resource exceeds bound".into());
        }
        let expected = u64::from_le_bytes(header[4..].try_into()?);
        let mut bytes = vec![0; length];
        input.read_exact(&mut bytes)?;
        let tree = prop::parse(&bytes)?;
        if !tree.exact || tree.consumed != bytes.len() {
            return Err(format!("resource {ordinal}: partial framing").into());
        }
        let mut hash = 0xcbf29ce484222325u64;
        let mut add = |bytes: &[u8]| {
            for byte in bytes {
                hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
            }
        };
        for item in tree
            .items
            .iter()
            .filter(|i| matches!(i.prop_type, 0x32 | 0x34))
        {
            let dimension = if item.prop_type == 0x32 { 3 } else { 4 };
            add(&item.id.to_le_bytes());
            add(&item.prop_type.to_le_bytes());
            add(&item.specifier.to_le_bytes());
            add(&((dimension * item.values.len()) as u32).to_le_bytes());
            fields += 1;
            colors += item.values.len();
            for value in &item.values {
                let values = match value {
                    PropValue::ColorRGB(r, g, b) if dimension == 3 => vec![*r, *g, *b],
                    PropValue::ColorRGBA(r, g, b, a) if dimension == 4 => vec![*r, *g, *b, *a],
                    _ => return Err(format!("resource {ordinal}: untyped color").into()),
                };
                for value in values {
                    add(&value.to_bits().to_le_bytes());
                    components += 1;
                }
            }
        }
        if hash != expected {
            return Err(format!("resource {ordinal}: color bits differ").into());
        }
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing fixture bytes".into());
    }
    println!("PROP_COLOR_VERIFY resources={count} fields={fields} colors={colors} components={components} exact=true");
    Ok(())
}
