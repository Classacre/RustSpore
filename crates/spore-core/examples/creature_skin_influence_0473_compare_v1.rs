//! Conditional caller-supplied grid and points; output requests are not skin weights or palette IDs.
use std::{env, fs, io::{self, Write}};
use spore_core::{creature_generated_skin_grid_query_v1::{query, SnapshotAdmission}, creature_generated_skin_kernel_v1::Lane, native_sse::Policy};
fn word(b: &[u8], p: usize) -> u32 { u32::from_le_bytes(b[p..p+4].try_into().unwrap()) }
fn main() {
    let args: Vec<_> = env::args().collect(); assert_eq!(args.len(), 3);
    let b = fs::read(&args[1]).unwrap(); let points = fs::read(&args[2]).unwrap();
    assert!(b.len() >= 208); assert_eq!(points.len(), 528);
    let v = |i: usize| f32::from_bits(word(&b, i*4));
    let origin = [v(9), v(10), v(11)]; let scale = [v(12), v(13), v(14)];
    let mut off = 208; let mut owned: Vec<Vec<[Lane; 4]>> = vec![Vec::new(); 512];
    for cell in &mut owned {
        let n = word(&b, off) as usize; off += 4; assert!(n <= 16 && off+n*128 <= b.len());
        for _ in 0..n {
            let f = |field: usize, lane: usize| f32::from_bits(word(&b, off+field*16+lane*4));
            cell.push(std::array::from_fn(|lane| Lane { center: [f(0,lane),f(1,lane),f(2,lane)], cutoff: f(3,lane), scale: f(4,lane), coefficient: f(5,lane), key: word(&b,off+112+lane*4) })); off += 128;
        }
    }
    assert_eq!(off,b.len());
    let cells: [Option<&[[Lane;4]]>;512] = std::array::from_fn(|i| if owned[i].is_empty() { None } else { Some(owned[i].as_slice()) });
    let empty = [None;512]; let mut out = Vec::new();
    for case in 0..46 {
        let point = if case == 45 { [32.,0.,0.] } else { let p = if case == 44 { 0 } else { case*12 }; [f32::from_bits(word(&points,p)), f32::from_bits(word(&points,p+4)), f32::from_bits(word(&points,p+8))] };
        let result = query(point, origin, scale, if case < 44 { &cells } else { &empty }, Policy::Mxcsr1f80, SnapshotAdmission).unwrap();
        if case == 44 { assert!(result.address.cell.is_some()); } if case == 45 { assert!(result.address.cell.is_none()); }
        assert!(result.requests.len() <= 64); out.extend((result.requests.len() as u32).to_le_bytes());
        for r in &result.requests { out.extend(r.key.to_le_bytes()); out.extend(r.weight.to_bits().to_le_bytes()); }
        out.resize(out.len()+(64-result.requests.len())*8,0);
    }
    io::stdout().write_all(&out).unwrap();
}

