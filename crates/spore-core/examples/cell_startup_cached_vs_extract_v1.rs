//! Extract one explicitly selected cached program; no shader execution.
use std::{fs::OpenOptions, io::Write, path::Path};
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    assert_eq!(args.len(), 3, "pack and new output required");
    let bytes = std::fs::read(&args[1]).unwrap();
    let pack = spore_core::shader_pack::parse(&bytes).unwrap();
    let recipe = [0x14,0x17,0x19,0x15,0x31,0x30,0x29,0x01];
    let program = pack.compiled_vertex.lookup(&recipe).unwrap().expect("cached VS absent");
    let mut out = OpenOptions::new().create_new(true).write(true).open(Path::new(&args[2])).unwrap();
    out.write_all(&program.bytecode).unwrap();
    println!("cached_vs recipe={recipe:02X?} bytes={} uniforms={}",program.bytecode.len(),program.uniforms.len());
}
