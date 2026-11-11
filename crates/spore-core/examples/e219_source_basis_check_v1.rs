//! Hash-pinned diagnostic of the explicitly selected E219 external-1 basis.
//! The loader association is conditional on the bounded fixup fixture; this
//! does not establish live controller selection or evaluate an animated pose.
use sha2::{Digest, Sha256};
use spore_core::{rw4, rw4_animation};
use std::{env, fs};

const ASSET_HASH: &str = "aed300c20813388a06b3dfa86644e53d573bb18e34167ba9b385563314037cb2";
const ROWS_HASH: &str = "662b90e2ca27a297e7d7e9e2d5f47f27aae70c91d0dc2c2d5f3c4c7c581480f2";
const ROW_START: usize = 0x28f70;
const ROW_BYTES: usize = 7 * 64;

fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn check(data: &[u8]) -> Result<usize, String> {
    if digest(data) != ASSET_HASH {
        return Err("unexpected source RW4 hash".into());
    }
    let rows = data
        .get(ROW_START..ROW_START + ROW_BYTES)
        .ok_or("short source rows")?;
    if digest(rows) != ROWS_HASH {
        return Err("unexpected source row hash".into());
    }
    let rw = rw4::parse(data).map_err(|e| e.to_string())?;
    let set = rw4_animation::parse(&rw, data).map_err(|e| e.to_string())?;
    let binding = set
        .bindings
        .iter()
        .find(|v| v.section_index == 7)
        .ok_or("missing binding 7")?
        .value;
    if binding.animation_skin != 5 || binding.skeleton != 9 || binding.skin_matrix_buffer != 6 {
        return Err("unexpected binding references".into());
    }
    let skin = &set
        .animation_skins
        .iter()
        .find(|v| v.section_index == 5)
        .ok_or("missing skin 5")?
        .value;
    if skin.raw_columns.len() != 7 {
        return Err("unexpected basis count".into());
    }
    for (row, source) in skin.raw_columns.iter().zip(rows.chunks_exact(64)) {
        for (value, bytes) in row.iter().zip(source.chunks_exact(4)) {
            if value.to_bits() != u32::from_le_bytes(bytes.try_into().unwrap()) {
                return Err("parsed basis differs from source bytes".into());
            }
        }
    }
    let identity = [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0.,
    ];
    Ok(skin
        .raw_columns
        .iter()
        .filter(|row| **row != identity)
        .count())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: e219_source_basis_check_v1 <external-1.rw4>")?;
    let nonidentity = check(&fs::read(path)?)?;
    println!("Pinned E219 binding 7 -> skin 5: 7 source rows, {nonidentity} nonidentity; conditional fixup only, no live pose or evaluator comparison");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unpinned_input() {
        assert!(check(b"not the pinned source").is_err());
    }
}
