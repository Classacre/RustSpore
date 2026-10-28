//! Bounded fixture adapter; no source runtime association is implied.
#[path = "cell_hierarchy_76f930_candidate_v1.rs"]
mod candidate;

use std::io::{self, Read};

fn run() -> Result<(), String> {
    // The protocol caps both arrays at 256 entries, so one extra byte is
    // sufficient to reject oversized input without an unbounded allocation.
    const MAX: u64 = 16 + 4 * (256 + 256 + 256);
    let mut input = Vec::new();
    io::stdin().take(MAX + 1).read_to_end(&mut input).map_err(|e| e.to_string())?;
    if input.len() < 16 || &input[..4] != b"H930" {
        return Err("invalid fixture header".into());
    }
    let word = |at: usize| u32::from_le_bytes(input[at..at + 4].try_into().unwrap());
    let (a_len, b_len, placement) = (word(4) as usize, word(8) as usize, word(12));
    if a_len > 256 || b_len > 256 || placement > 1 || input.len() != 16 + 4 * (a_len * 2 + b_len) {
        return Err("invalid fixture bounds or framing".into());
    }
    let mut words = input[16..].chunks_exact(4).map(|bytes| i32::from_le_bytes(bytes.try_into().unwrap()));
    let a: Vec<_> = words.by_ref().take(a_len).collect();
    let b: Vec<_> = words.by_ref().take(b_len).collect();
    let mapping: Vec<_> = words.collect();
    let result = candidate::check(&a, &b, &mapping).map_err(|e| format!("invalid fixture: {e:?}"))?;
    println!("{}", u8::from(result));
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
