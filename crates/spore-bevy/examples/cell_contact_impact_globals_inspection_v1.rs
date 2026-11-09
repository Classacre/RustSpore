//! Explicit installed-globals threshold inspection, not a gameplay contact.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::{cell_contact_impact_flag_v1::impact_flag, native_sse::Policy};
use std::{error::Error, fs::File, io::Read, path::Path};

const EXPECTED: &str = "e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72";
fn threshold(payload: &[u8]) -> Result<u32, Box<dyn Error>> {
    if payload.len() != 0x114 || format!("{:x}", Sha256::digest(payload)) != EXPECTED {
        return Err("globals layout or fixed payload SHA-256 mismatch".into());
    }
    Ok(u32::from_le_bytes(payload[0x70..0x74].try_into()?))
}
fn run() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 3 || a[2] != "--admit-stable-package-and-unchanged-globals" {
        return Err("usage: cell_contact_impact_globals_inspection_v1 <package> <distance-bits-8hex> --admit-stable-package-and-unchanged-globals".into());
    }
    if a[1].len() != 8 || !a[1].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("distance must be exactly eight hexadecimal binary32 bits".into());
    }
    let distance = u32::from_str_radix(&a[1], 16)?;
    let path = Path::new(&a[0]);
    let mut file = File::open(path)?;
    let mut header = [0; dbpf::HEADER_SIZE];
    file.read_exact(&mut header)?;
    let header = dbpf::DbpfHeader::parse(&header)?;
    if header.index_size > 64 * 1024 * 1024 || header.index_entry_count > 2_000_000 {
        return Err("package index exceeds limits".into());
    }
    let package = NativePackageIndex::open(path)?;
    let key = NativeResourceKey {
        instance_id: 0xa426730b,
        type_id: 0x2a3ce5b7,
        group_id: 0,
    };
    let entry = package
        .index()
        .find_unique(key)?
        .ok_or("globals key absent")?;
    if entry.stored_size > 1024 || entry.decompressed_size != 0x114 {
        return Err("unsupported globals resource extent".into());
    }
    let payload = package.read(entry)?;
    let limit = threshold(&payload)?;
    let flag = impact_flag(distance, limit, Policy::Mxcsr1f80)?;
    println!(
        "INSPECTION globals={key} sha256={EXPECTED} distance_bits={distance:08X} minImpactCollisionSpeed_bits={limit:08X} impact_flag={flag}"
    );
    println!(
        "Caller admits stable package path and unchanged globals; distance is explicit, MXCSR=1F80 with masked exceptions and discarded status. No source actor, preceding gate, contact, provider selection, event or playback established."
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("inspection failed: {e}");
        std::process::exit(1)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_wrong_layout_and_unpinned_content() {
        assert!(threshold(&[0; 0x113]).is_err());
        assert!(threshold(&[0; 0x114]).is_err());
    }
}
