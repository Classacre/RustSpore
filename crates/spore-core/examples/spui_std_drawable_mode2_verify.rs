//! Explicit bit-pattern fixture driver; no installed game or native execution.
use spore_core::spui_std_drawable_mode2::mode2_calls;
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let words: Vec<u32> = line.split('\t').map(|w| u32::from_str_radix(w, 16)).collect::<Result<_, _>>()?;
        if words.len() != 16 { return Err("expected 16 hexadecimal words".into()); }
        let floats = |s: &[u32]| s.iter().copied().map(f32::from_bits).collect::<Vec<_>>();
        let dest = floats(&words[0..4]).try_into().unwrap();
        let size = [words[4] as i32, words[5] as i32];
        let uv = floats(&words[6..10]).try_into().unwrap();
        let split = floats(&words[10..14]).try_into().unwrap();
        let scale = floats(&words[14..16]).try_into().unwrap();
        match mode2_calls(dest, size, uv, split, scale) {
            Err(_) => println!("error"),
            Ok(calls) => {
                print!("{}", calls.len());
                for call in calls {
                    for v in call.destination.into_iter().chain(call.relative_uv) { print!("\t{:08X}", v.to_bits()); }
                }
                println!();
            }
        }
    }
    Ok(())
}
