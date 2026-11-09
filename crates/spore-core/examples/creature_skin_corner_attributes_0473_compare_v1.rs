//! Conditional caller-alpha records and explicit synthetic corner indices.
use spore_core::creature_skin_corner_attributes_v1::join;
use std::{env, fs, io::{self, Write}};
fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 3);
    let bytes = fs::read(&args[1]).unwrap();
    let idx = fs::read(&args[2]).unwrap();
    assert_eq!(bytes.len(), 2640);
    assert_eq!(idx.len(), 528);
    let indices: Vec<u32> = idx.chunks_exact(4).map(|v| u32::from_le_bytes(v.try_into().unwrap())).collect();
    let mut output = Vec::new();
    for pass in 0..3 {
        let records: Vec<[u8; 20]> = bytes[pass*880..(pass+1)*880].chunks_exact(20).map(|v| v.try_into().unwrap()).collect();
        for entry in join(&records, &indices[pass*44..(pass+1)*44]).unwrap() {
            for value in entry.offsets { output.extend(value.to_le_bytes()); }
            for value in entry.weight_words { output.extend(value.to_le_bytes()); }
        }
    }
    io::stdout().write_all(&output).unwrap();
}
