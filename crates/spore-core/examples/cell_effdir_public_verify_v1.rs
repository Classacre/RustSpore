//! Verify an explicitly supplied decoded EFFDIR payload; no runtime selection.
use spore_core::cell_effdir_framing_v3 as effdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("expected a decoded EFFDIR path")?;
    let bytes = std::fs::read(path)?;
    let directory = effdir::parse(&bytes).map_err(|e| format!("{e:?}"))?;
    let key = effdir::Key { group: 0, instance: 0x05c7_a4d0 };
    for target in directory.resolve(key) {
        if let effdir::Target::Visual(visual) = target {
            for block in &visual.blocks {
                if let Some(effdir::ScreenRef::Screen(screen)) = directory.screen_ref(block) {
                    println!("visual_block={} mode={} flags={:08X} texture={:08X}:{:08X}",
                        block.index, screen.mode, screen.flags, screen.texture.group, screen.texture.instance);
                }
            }
        }
    }
    Ok(())
}
