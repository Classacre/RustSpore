//! Compare complete bounded flagella/cilia and jet math with original x86 code.
use spore_core::cell_swimming::{flagella_cilia_delta, jet_delta, CellPropulsion};
use spore_core::native_x87::Context;
use std::collections::BTreeSet;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: cell_propulsion_verify <RSCELLP1 fixture>")?;
    #[cfg(target_arch = "x86_64")]
    {
        let mut mxcsr = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{address}]", address = in(reg) &mut mxcsr,
                options(nostack, preserves_flags));
        }
        if mxcsr & 0xffc0 != 0x1f80 {
            return Err(format!("unsupported MXCSR control {:08X}", mxcsr & 0xffc0).into());
        }
    }
    let mut input = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut header = [0; 12];
    input.read_exact(&mut header)?;
    if &header[..8] != b"RSCELLP1" {
        return Err("wrong propulsion fixture magic".into());
    }
    let count = u32::from_le_bytes(header[8..].try_into()?);
    if count == 0 || count > 1_000_000 {
        return Err("invalid propulsion fixture count".into());
    }
    let mut lanes = BTreeSet::new();
    for ordinal in 0..count {
        let mut record = [0; 116];
        input.read_exact(&mut record)?;
        let word =
            |offset: usize| u32::from_le_bytes(record[offset..offset + 4].try_into().unwrap());
        let float = |offset| f32::from_bits(word(offset));
        let vector = |offset: usize| std::array::from_fn(|axis| float(offset + 4 * axis));
        let mode = word(0);
        let control = u16::from_le_bytes(record[4..6].try_into()?);
        let parameters = CellPropulsion {
            direction: vector(8),
            position: vector(20),
            current_vector: vector(32),
            target: vector(44),
            speed: float(56),
            factor_6: float(60),
            heading_test_7: float(64),
            hint_8: word(68) as u8,
            factor_9: float(72),
            sampled_environment: vector(76),
            zero_speed_vector: vector(88),
        };
        let context = Context::from_control_word(control)?;
        let (actual, status) = match mode {
            1 => (flagella_cilia_delta(parameters, context)?, 0),
            2 => {
                let value = jet_delta(parameters, context)?;
                (value.delta, u32::from(value.active))
            }
            _ => return Err("unknown propulsion fixture mode".into()),
        };
        let expected = [word(100), word(104), word(108)];
        if actual.map(f32::to_bits) != expected || status != word(112) {
            return Err(format!("propulsion case {ordinal} mode{mode} CW{control:04X}: {:08X?}/{} != {:08X?}/{}; input={parameters:?}", actual.map(f32::to_bits), status, expected, word(112)).into());
        }
        lanes.insert((mode, control));
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing propulsion fixture bytes".into());
    }
    println!("CELL_PROPULSION_VERIFY cases={count} mode_policy_lanes={} exact_bits=true mxcsr_control=00001F80 native_policy_established=false", lanes.len());
    Ok(())
}
