//! Compare the original centered field sampler with source-order CPU goldens.
use spore_core::cell_flow_field::{sample_flow_field, CellFlowFrame, CELL_FLOW_SAMPLES};
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: cell_flow_field_verify <RSCELLF1 fixture>")?;
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
    if &header[..8] != b"RSCELLF1" {
        return Err("wrong flow-field fixture magic".into());
    }
    let count = u32::from_le_bytes(header[8..].try_into()?);
    if count == 0 || count > 1_000_000 {
        return Err("invalid flow-field fixture count".into());
    }
    for ordinal in 0..count {
        let mut record = [0; 3636];
        input.read_exact(&mut record)?;
        let word =
            |index: usize| u32::from_le_bytes(record[index * 4..index * 4 + 4].try_into().unwrap());
        let float = |index| f32::from_bits(word(index));
        let field: [[f32; 3]; CELL_FLOW_SAMPLES] =
            std::array::from_fn(|index| std::array::from_fn(|axis| float(6 + index * 3 + axis)));
        let actual = sample_flow_field(
            [float(0), float(1)],
            CellFlowFrame {
                origin: [float(2), float(3)],
                scale: [float(4), float(5)],
            },
            &field,
        )?
        .map(f32::to_bits);
        let expected = [word(906), word(907), word(908)];
        if actual != expected {
            return Err(
                format!("flow-field case {ordinal}: {actual:08X?} != {expected:08X?}").into(),
            );
        }
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing flow-field fixture bytes".into());
    }
    println!("CELL_FLOW_FIELD_VERIFY cases={count} exact_bits=true mxcsr_control=00001F80");
    Ok(())
}
