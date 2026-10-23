//! Rust-only verification of separately reviewed source slerp extrapolation.
use spore_core::anim_quaternion::{slerp_extrapolated, ArithmeticContext, SsePolicy};
use spore_core::native_crt_acos::ConstantWindow;
use spore_core::native_x87::Context;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: anim_quaternion_extrapolate_verify <verified-constants.bin> <inputs.bin> <new-output.bin>".into());
    }
    let bytes = std::fs::read(&args[0])?;
    let constants = ConstantWindow::new(&bytes)?;
    let input = std::fs::read(&args[1])?;
    if input.is_empty() || input.len() % 48 != 0 {
        return Err("expected nonempty complete48-byte records".into());
    }
    let mut output = std::io::BufWriter::new(std::fs::File::create_new(&args[2])?);
    for (n, row) in input.chunks_exact(48).enumerate() {
        let words: [u32; 12] =
            std::array::from_fn(|i| u32::from_le_bytes(row[i * 4..i * 4 + 4].try_into().unwrap()));
        if words[0] != 2 || words[11] != 0 || words[1] & 0xFFFF_F0FF != 0x7F {
            return Err(format!("unexpected record {n}").into());
        }
        let context = ArithmeticContext {
            x87: Context::from_control_word(words[1] as u16)?,
            sse: SsePolicy::Mxcsr1f80,
        };
        let result = slerp_extrapolated(
            context,
            std::array::from_fn(|i| f32::from_bits(words[i + 2])),
            std::array::from_fn(|i| f32::from_bits(words[i + 6])),
            f32::from_bits(words[10]),
            constants,
        )?;
        for word in result.map(f32::to_bits).into_iter().chain([0x7FC0_BEEF, 0]) {
            output.write_all(&word.to_le_bytes())?;
        }
    }
    output.flush()?;
    println!("rows={} domain=extrapolated[-1,2] policy=explicit-CW-MXCSR1F80 original_host_executed=false", input.len()/48);
    Ok(())
}
