//! Direct-model entry point for the main application's original Cell renderer.
#[path = "../src/native_cell_renderer.rs"]
mod native_cell_renderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: source_cell_shader_probe <install> <new.png> [GMDL hex]".into());
    }
    let instance = args
        .get(2)
        .map(|value| u64::from_str_radix(value.trim_start_matches("0x"), 16))
        .transpose()?
        .unwrap_or(0x40617100780a9eaf);
    native_cell_renderer::capture(
        std::path::Path::new(&args[0]),
        std::path::Path::new(&args[1]),
        instance,
    )
}
