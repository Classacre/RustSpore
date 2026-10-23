//! Compare reconstructed quaternion arithmetic with a reviewed native fixture.
//! Only Rust and external data are evaluated; original modules are not loaded.
use spore_core::anim_quaternion::{self, ArithmeticContext, QuaternionError, SsePolicy};
use spore_core::native_crt_acos::ConstantWindow;
use spore_core::native_x87::Context;
use std::io::Write;

const SENTINEL: u32 = 0x7FC0_BEEF;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: anim_quaternion_verify <verified-constant-window.bin> <inputs.bin> <new-output.bin>".into());
    }
    // The fixture owner checks the pinned source/window digests. ConstantWindow
    // borrows these local bytes and checks its exact required length.
    let bytes = std::fs::read(&args[0])?;
    let constants = ConstantWindow::new(&bytes)?;
    let input = std::fs::read(&args[1])?;
    if input.is_empty() || input.len() % 48 != 0 {
        return Err("expected complete nonempty 48-byte input records".into());
    }
    let mut output = std::io::BufWriter::new(std::fs::File::create_new(&args[2])?);
    let mut counts = [0usize; 3];
    let mut excluded = 0usize;
    for (index, row) in input.chunks_exact(48).enumerate() {
        let words: [u32; 12] =
            std::array::from_fn(|i| u32::from_le_bytes(row[i * 4..i * 4 + 4].try_into().unwrap()));
        let operation = words[0] as usize;
        if operation > 2
            || words[2..10].iter().any(|v| v & 0x7FFF_FFFF > 0x4080_0000)
            || words[10] & 0x7FFF_FFFF > 0x3F80_0000
            || words[10] >> 31 != 0 && words[10] & 0x7FFF_FFFF != 0
            || words[11] > 1
            || words[11] != 0 && operation != 0
            || words[1] & 0xFFFF_F0FF != 0x7F
            || words[1] & 0x300 == 0x100
        {
            return Err(format!("input record {index} violates fixture bounds").into());
        }
        let context = ArithmeticContext {
            x87: Context::from_control_word(words[1] as u16)?,
            sse: SsePolicy::Mxcsr1f80,
        };
        let left = std::array::from_fn(|i| f32::from_bits(words[2 + i]));
        let right = std::array::from_fn(|i| f32::from_bits(words[6 + i]));
        let result = match operation {
            0 => anim_quaternion::normalize(context, left).map(|v| {
                let mut bits = [0; 6];
                bits[..4].copy_from_slice(&v.quaternion.map(f32::to_bits));
                bits[4] = if words[11] == 0 {
                    v.length.to_bits()
                } else {
                    SENTINEL
                };
                bits
            }),
            1 => anim_quaternion::hemisphere_add(context, left, right).map(|v| {
                let mut bits = [0; 6];
                bits[..4].copy_from_slice(&v.map(f32::to_bits));
                bits[4] = SENTINEL;
                bits
            }),
            2 => anim_quaternion::slerp(context, left, right, f32::from_bits(words[10]), constants)
                .map(|v| {
                    let mut bits = [0; 6];
                    bits[..4].copy_from_slice(&v.map(f32::to_bits));
                    bits[4] = SENTINEL;
                    bits
                }),
            _ => unreachable!(),
        };
        let bits = match result {
            Ok(bits) => bits,
            Err(QuaternionError::NonFiniteOutput) if operation == 0 => {
                excluded += 1;
                [0, 0, 0, 0, 0, 1]
            }
            Err(error) => return Err(format!("record {index}: {error}").into()),
        };
        for word in bits {
            output.write_all(&word.to_le_bytes())?;
        }
        counts[operation] += 1;
    }
    output.flush()?;
    println!("normalize_rows={} hemisphere_add_rows={} slerp_rows={} nonfinite_normalization_rows={excluded} original_host_executed=false policy=explicit-CW-MXCSR1F80", counts[0], counts[1], counts[2]);
    Ok(())
}
