//! Compare E55E40 under each explicit x87 policy with source-order CPU goldens.
use spore_core::cell_swimming::{turn_in_place_delta, CellTurnInPlace};
use spore_core::native_x87::Context;
use std::collections::BTreeMap;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: cell_swimming_verify <RSCELLT1 fixture>")?;
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
    if &header[..8] != b"RSCELLT1" {
        return Err("wrong swimming fixture magic".into());
    }
    let count = u32::from_le_bytes(header[8..].try_into()?);
    if count == 0 || count > 1_000_000 {
        return Err("invalid swimming fixture count".into());
    }
    let mut policies = BTreeMap::<u16, u32>::new();
    for ordinal in 0..count {
        let mut record = [0; 78];
        input.read_exact(&mut record)?;
        let word =
            |index: usize| u32::from_le_bytes(record[index * 4..index * 4 + 4].try_into().unwrap());
        let vector = |index: usize| std::array::from_fn(|axis| f32::from_bits(word(index + axis)));
        let control = u16::from_le_bytes(record[64..66].try_into()?);
        let actual = turn_in_place_delta(
            CellTurnInPlace {
                position: vector(0),
                current_vector: vector(3),
                target: vector(6),
                step: f32::from_bits(word(9)),
                sampled_environment: vector(10),
                zero_step_vector: vector(13),
            },
            Context::from_control_word(control)?,
        )?
        .map(f32::to_bits);
        let expected: [u32; 3] = std::array::from_fn(|axis| {
            u32::from_le_bytes(record[66 + axis * 4..70 + axis * 4].try_into().unwrap())
        });
        if actual != expected {
            return Err(format!(
                "swimming case {ordinal}, CW {control:04X}: {actual:08X?} != {expected:08X?}"
            )
            .into());
        }
        *policies.entry(control).or_default() += 1;
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing swimming fixture bytes".into());
    }
    println!(
        "CELL_SWIMMING_VERIFY cases={count} explicit_x87_policies={} exact_bits=true mxcsr_control=00001F80 native_policy_established=false",
        policies.len()
    );
    Ok(())
}
