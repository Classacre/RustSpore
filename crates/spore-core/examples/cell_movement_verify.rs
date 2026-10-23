//! Compare the pure axial helper with bounded source SSE hardware goldens.
use spore_core::cell_movement::{axial_seek_target, CellMovementAxes};
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: cell_movement_verify <RSCELLM1 fixture>")?;
    #[cfg(target_arch = "x86_64")]
    {
        let mut mxcsr = 0u32;
        // Observe the caller's policy. The verifier does not modify it.
        unsafe {
            std::arch::asm!("stmxcsr [{address}]", address = in(reg) &mut mxcsr,
            options(nostack, preserves_flags));
        }
        if mxcsr & 0xffc0 != 0x1f80 {
            return Err(format!("unsupported caller MXCSR control {:08X}", mxcsr & 0xffc0).into());
        }
    }
    let mut input = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut header = [0; 12];
    input.read_exact(&mut header)?;
    if &header[..8] != b"RSCELLM1" {
        return Err("wrong movement fixture magic".into());
    }
    let count = u32::from_le_bytes(header[8..].try_into()?);
    if count == 0 || count > 1_000_000 {
        return Err("invalid movement fixture count".into());
    }
    for ordinal in 0..count {
        let mut record = [0; 73];
        input.read_exact(&mut record)?;
        let word = |at: usize| u32::from_le_bytes(record[at..at + 4].try_into().unwrap());
        let values: [f32; 15] = std::array::from_fn(|i| f32::from_bits(word(i * 4)));
        let axes = CellMovementAxes {
            primary: values[6..9].try_into()?,
            secondary: values[9..12].try_into()?,
            tertiary: values[12..15].try_into()?,
        };
        let actual = axial_seek_target(
            values[..3].try_into()?,
            values[3..6].try_into()?,
            axes,
            record[60],
        )?
        .map(f32::to_bits);
        let expected = [word(61), word(65), word(69)];
        if actual != expected {
            return Err(format!(
                "axial case {ordinal}: {actual:08X?} differs from {expected:08X?}"
            )
            .into());
        }
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing movement fixture bytes".into());
    }
    println!("CELL_MOVEMENT_VERIFY cases={count} exact_bits=true mxcsr_control=00001F80");
    Ok(())
}
