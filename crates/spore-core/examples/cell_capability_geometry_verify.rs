//! Compare the E51EC0 skip-flag reduction with complete native tail bytes.
use spore_core::{
    cell_capability_geometry::{
        derive_cell_capability_geometry, CellCapabilityGeometryError, CellCapabilityPart,
    },
    native_x87::Context,
};
use std::{
    collections::BTreeSet,
    io::{Read, Write},
};

fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}
fn float(data: &[u8], at: usize) -> f32 {
    f32::from_bits(word(data, at))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let nonfinite = args.get(3).is_some_and(|arg| arg == "--nonfinite");
    if args.len() != 3 && !(args.len() == 4 && nonfinite) {
        return Err("usage: cell_capability_geometry_verify <input.bin> <native-output.bin> <new rust-output.bin> [--nonfinite]".into());
    }
    #[cfg(target_arch = "x86_64")]
    {
        let mut mxcsr = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{address}]", address = in(reg) &mut mxcsr, options(nostack, preserves_flags));
        }
        if mxcsr & 0xffc0 != 0x1f80 {
            return Err("fixture requires MXCSR control1F80".into());
        }
    }
    let input = std::fs::File::open(&args[0])?;
    let native = std::fs::File::open(&args[1])?;
    let input_bytes = input.metadata()?.len();
    if input_bytes == 0
        || input_bytes % 4784 != 0
        || native.metadata()?.len() != input_bytes / 4784 * 144
    {
        return Err("native geometry fixture record counts/extents differ".into());
    }
    let cases = input_bytes / 4784;
    if nonfinite && cases != 5 {
        return Err("expected five explicit nonfinite-policy cases".into());
    }
    let mut input = std::io::BufReader::new(input);
    let mut native = std::io::BufReader::new(native);
    let mut output = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?,
    );
    let mut contexts = BTreeSet::new();
    if nonfinite {
        writeln!(
            output,
            "case\tchecked_error\tnative_center_x\tnative_center_y\tnative_center_z\tnative_radius"
        )?;
    }
    for ordinal in 0..cases {
        let mut request = [0u8; 4784];
        let mut expected = [0u8; 144];
        input.read_exact(&mut request)?;
        native.read_exact(&mut expected)?;
        let count = word(&request, 0) as usize;
        if count > 64 || word(&expected, 0) != 1 {
            return Err(format!("invalid protocol atcase{ordinal}").into());
        }
        let control = u16::from_le_bytes(request[4..6].try_into().unwrap());
        contexts.insert(control);
        let parts: Vec<_> = (0..count)
            .map(|index| {
                let at = 32 + index * 72;
                CellCapabilityPart {
                    flag: request[at],
                    class: word(&request, at + 4),
                    center: std::array::from_fn(|axis| float(&request, at + 8 + axis * 4)),
                    radius: float(&request, at + 56),
                }
            })
            .collect();
        let value = derive_cell_capability_geometry(
            &parts,
            std::array::from_fn(|axis| float(&request, 8 + 4 * axis)),
            Context::from_control_word(control)?,
        );
        if nonfinite {
            let error = value
                .err()
                .ok_or_else(|| format!("nonfinite policy case{ordinal} unexpectedly succeeded"))?;
            let required = if ordinal < 4 {
                CellCapabilityGeometryError::NonFiniteInput
            } else {
                CellCapabilityGeometryError::NonFiniteIntermediate
            };
            if error != required {
                return Err(
                    format!("nonfinite policy case{ordinal}: {error:?} != {required:?}").into(),
                );
            }
            writeln!(
                output,
                "{ordinal}\t{error:?}\t{:08X}\t{:08X}\t{:08X}\t{:08X}",
                word(&expected, 56),
                word(&expected, 60),
                word(&expected, 64),
                word(&expected, 68)
            )?;
            continue;
        }
        let value =
            value.map_err(|error| format!("case{ordinal},count{count},CW{control:04X}:{error}"))?;
        let mut actual = [0u8; 144];
        actual[..4].copy_from_slice(&1u32.to_le_bytes());
        actual[4..].copy_from_slice(&request[4640..4780]);
        value.write_native_tail((&mut actual[4..]).try_into().unwrap());
        if actual != expected {
            let byte = actual
                .iter()
                .zip(expected)
                .position(|(a, b)| *a != b)
                .unwrap();
            let aligned = byte / 4 * 4;
            return Err(format!("case{ordinal},count{count},CW{control:04X} differs atreplybyte{byte}:Rust{:08X},native{:08X}; nativecachefield{:X}",
                word(&actual, aligned), word(&expected, aligned), 0x1204 + aligned.saturating_sub(4)).into());
        }
        output.write_all(&actual)?;
    }
    output.flush()?;
    println!("CELL_CAPABILITY_GEOMETRY_VERIFY cases={cases} contexts={} nonfinite_policy={nonfinite} exact_bits={} mxcsr_control=1F80", contexts.len(), !nonfinite);
    Ok(())
}
