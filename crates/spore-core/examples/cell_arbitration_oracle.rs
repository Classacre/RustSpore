//! Text-only adapter for the local E59B40 native comparison.
use spore_core::cell_feeding_contact_arbitration_e59b40_v1::{
    submit_contact_candidate, ContactOutput,
};

fn contact(a: &[u32]) -> ContactOutput {
    ContactOutput {
        actor_ids: [a[0], a[1]],
        selector: a[2] as i32,
        feature_indices: [a[3], a[4]],
        depth_bits: a[5],
        point_words: [a[6], a[7], a[8]],
        normal_words: [a[9], a[10], a[11]],
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 25 || args.iter().any(|s| {
        s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        std::process::exit(10);
    }
    let a: Vec<u32> = args.iter().map(|s| u32::from_str_radix(s, 16).unwrap()).collect();
    let mut output = contact(&a[..12]);
    let candidate = contact(&a[12..24]);
    let _ = submit_contact_candidate(&mut output, candidate, a[24]);
    let words = [
        output.actor_ids[0], output.actor_ids[1], output.selector as u32,
        output.feature_indices[0], output.feature_indices[1], output.depth_bits,
        output.point_words[0], output.point_words[1], output.point_words[2],
        output.normal_words[0], output.normal_words[1], output.normal_words[2],
    ];
    println!("{}", words.iter().map(|n| format!("{n:08x}")).collect::<Vec<_>>().join(" "));
}
