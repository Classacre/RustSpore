//! Project the explicit native root-candidate cohort; does not open a window.
use spore_core::anim_body_topology::{select_root_candidate, RootPolicy, TopologyBlock};
use std::{
    env,
    error::Error,
    fs::OpenOptions,
    io::{Read, Write},
};

const INPUT: usize = 8176;
const OUTPUT: usize = 2056;
fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}
fn optional(value: u32) -> Option<usize> {
    (value != u32::MAX).then_some(value as usize)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: anim_body_root_verify inputs.bin new-rust-output.bin".into());
    }
    let mut input = std::fs::File::open(&args[0])?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    let mut request = [0u8; INPUT];
    let mut rows = 0;
    loop {
        let n = input.read(&mut request[..1])?;
        if n == 0 {
            break;
        }
        input.read_exact(&mut request[1..])?;
        let count = word(&request, 4) as usize;
        if !(1..=255).contains(&count) {
            return Err("invalid count".into());
        }
        let blocks = (0..count)
            .map(|i| {
                let at = 16 + i * 32;
                TopologyBlock {
                    parent: optional(word(&request, at)),
                    presence_mask: word(&request, at + 4),
                    nonzero_mask: word(&request, at + 8),
                    center: std::array::from_fn(|j| {
                        f32::from_bits(word(&request, at + 12 + j * 4))
                    }),
                    source_palette_id: word(&request, at + 24),
                    symmetric: optional(word(&request, at + 28)),
                }
            })
            .collect::<Vec<_>>();
        let policy = word(&request, 8);
        let result = select_root_candidate(
            &blocks,
            RootPolicy {
                nearest_origin: policy & 1 != 0,
                spine_midpoint_fallback: policy & 2 != 0,
            },
        )?;
        let mut reply = [0u8; OUTPUT];
        let mut set = |at: usize, v: u32| reply[at..at + 4].copy_from_slice(&v.to_le_bytes());
        set(0, result.root.map_or(u32::MAX, |i| i as u32));
        set(4, u32::from(result.no_foot_candidates));
        set(8, result.foot_candidates.len() as u32);
        set(12, result.foot_votes.iter().copied().max().unwrap_or(0));
        for (i, &votes) in result.foot_votes.iter().enumerate() {
            set(16 + i * 4, votes);
        }
        for (i, &candidate) in result.foot_candidates.iter().enumerate() {
            set(1036 + i * 4, candidate as u32);
        }
        output.write_all(&reply)?;
        rows += 1;
    }
    output.flush()?;
    println!("native root-candidate projections: {rows}");
    Ok(())
}
