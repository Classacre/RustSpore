//! Compare checked Cell geometry against the original bounded x86 host stream.
use spore_core::cell_part_geometry::{build_cell_parts, build_special_cell_part};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: cell_part_geometry_verify <input.bin> <native-output.bin> <new rust-output.bin>".into());
    }
    #[cfg(target_arch = "x86_64")]
    {
        let mut mxcsr = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{address}]", address = in(reg) &mut mxcsr, options(nostack, preserves_flags));
        }
        if mxcsr & 0xffc0 != 0x1f80 {
            return Err("geometry fixture requires MXCSR control1F80".into());
        }
    }
    let input = std::fs::File::open(&args[0])?;
    let native = std::fs::File::open(&args[1])?;
    let input_bytes = input.metadata()?.len();
    if input_bytes % 224 != 0 || native.metadata()?.len() != input_bytes / 224 * 160 {
        return Err("geometry fixture record counts/lengths differ".into());
    }
    let cases = input_bytes / 224;
    let mut input = std::io::BufReader::new(input);
    let mut native = std::io::BufReader::new(native);
    let mut output = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?,
    );
    let mut policies = BTreeMap::<u16, u64>::new();
    let mut ordinary = 0u64;
    let mut special = 0u64;
    let mut clamped = 0u64;
    for ordinal in 0..cases {
        let mut request = [0u8; 224];
        let mut expected = [0u8; 160];
        input.read_exact(&mut request)?;
        native.read_exact(&mut expected)?;
        let count = word(&request, 0) as usize;
        if count > 256 {
            return Err(format!("invalid fixture count at{ordinal}").into());
        }
        let control = u16::from_le_bytes(request[20..22].try_into().unwrap());
        if control & 0xf0ff != 0x7f || control & 0x300 == 0x100 {
            return Err(format!("invalid fixture CW at{ordinal}").into());
        }
        *policies.entry(control).or_default() += 1;
        let special_resource = word(&request, 12) == 0x476a98c7;
        let parts = if special_resource {
            special += 1;
            vec![build_special_cell_part(
                std::array::from_fn(|i| word(&request, 172 + 4 * i)),
                std::array::from_fn(|i| word(&request, 184 + 4 * i)),
            )]
        } else {
            ordinary += 1;
            clamped += u64::from(count > 64);
            let row: [u8; 140] = request[32..172].try_into().unwrap();
            build_cell_parts(
                &vec![row; count],
                &vec![word(&request, 4); count],
                word(&request, 8),
            )
            .map_err(|err| format!("geometry case{ordinal} rejected:{err}"))?
        };
        let mut actual = [0xa5u8; 160];
        actual[..4].copy_from_slice(&(parts.len() as u32).to_le_bytes());
        let provider_count = u32::from(!special_resource);
        actual[4..8].copy_from_slice(&provider_count.to_le_bytes());
        actual[8..12].copy_from_slice(
            &(if special_resource {
                0
            } else {
                parts.len() as u32
            })
            .to_le_bytes(),
        );
        actual[12..16].copy_from_slice(&provider_count.to_le_bytes());
        if let Some(first) = parts.first() {
            first.write_native((&mut actual[16..88]).try_into().unwrap());
            parts
                .last()
                .unwrap()
                .write_native((&mut actual[88..160]).try_into().unwrap());
        }
        if actual != expected {
            let byte = actual
                .iter()
                .zip(expected)
                .position(|(a, b)| *a != b)
                .unwrap();
            return Err(format!("geometry case{ordinal} differs atbyte{byte}: Rust{:02X} native{:02X}; count{count} class{} key{:08X}:{:08X}:{:08X} CW{control:04X}", actual[byte], expected[byte], word(&request, 4), word(&request, 8), word(&request, 12), word(&request, 16)).into());
        }
        output.write_all(&actual)?;
    }
    output.flush()?;
    println!("CELL_PART_GEOMETRY_VERIFY cases={cases} ordinary={ordinary} special={special} clamped={clamped} contexts={} exact_bits=true mxcsr_control=1F80", policies.len());
    Ok(())
}
