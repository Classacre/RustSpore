//! Conditional source positions and synthetic faces/secondary attribute streams.
use spore_core::creature_skin_corner_triples_v1::{build, project_records};
use std::{env, fs, io::{self, Write}};
fn words(path: &str) -> Vec<u32> {
    let bytes = fs::read(path).unwrap(); assert_eq!(bytes.len() % 4, 0);
    bytes.chunks_exact(4).map(|x| u32::from_le_bytes(x.try_into().unwrap())).collect()
}
fn main() {
    let args: Vec<_> = env::args().collect(); assert_eq!(args.len(),5);
    let positions=words(&args[1]); let secondary=words(&args[2]); let tertiary=words(&args[3]);
    let records=fs::read(&args[4]).unwrap(); assert_eq!(records.len(),2640);
    let records:Vec<[u8;20]>=records[880..1760].chunks_exact(20).map(|x|x.try_into().unwrap()).collect();
    let faces:Vec<u32>=(0..84).collect(); let joined=build(&positions,&secondary,&tertiary,&faces,44).unwrap();
    let mut output=Vec::new();output.extend((joined.triples.len() as u32).to_le_bytes());
    for triple in &joined.triples {for word in triple {output.extend(word.to_le_bytes());}}
    for &corner in &joined.corners {output.extend(corner.to_le_bytes());}
    for attr in project_records(&joined,&records).unwrap() {
        for x in attr.offsets {output.extend(x.to_le_bytes());}
        for x in attr.weight_words {output.extend(x.to_le_bytes());}
    }
    io::stdout().write_all(&output).unwrap();
}
