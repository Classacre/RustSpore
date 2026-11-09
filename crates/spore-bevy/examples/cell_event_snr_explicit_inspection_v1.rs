//! Explicit row-to-SNR inspection under conditional native-loader admissions.
//! No live event, random selection, provider precedence or playback is asserted.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::{snr_pcm16, snr_xas};
use std::{error::Error, fs::File, io::Read, path::Path};
const SCOPE: u32 = 0x021407ee;
// Narrow synthetic admission: E64E90's current-record +20 == 1 branch,
// E59C10 category outputs {1,2}, and the initialized source table.
fn admitted_contact_event(a: u32, b: u32, flag: u8) -> Option<u32> {
    if flag != 1 || !((a == 1 && b == 2) || (a == 2 && b == 1)) {
        return None;
    }
    let k = a.max(b) * (a.max(b) - 1) / 2 + a.min(b);
    if k != 2 {
        return None;
    }
    let mut h = 0x811c9dc5u32;
    for b in b"cell_collision_creature_plant_impact" {
        h = h.wrapping_mul(0x01000193) ^ u32::from(*b);
    }
    Some(h)
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
#[path = "../src/cell_event_sample_v1.rs"]
mod cell_event_sample_v1;
use cell_event_sample_v1::select;
fn package_read(path: &Path, key: NativeResourceKey, max: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut h = [0; dbpf::HEADER_SIZE];
    f.read_exact(&mut h)?;
    let h = dbpf::DbpfHeader::parse(&h)?;
    if h.index_size > 64 * 1024 * 1024 || h.index_entry_count > 2_000_000 {
        return Err("package index exceeds limits".into());
    }
    let pkg = NativePackageIndex::open(path)?;
    let e = pkg
        .index()
        .find_unique(key)?
        .ok_or("unique resource not found")?;
    if e.stored_size.max(e.decompressed_size) > max {
        return Err("resource exceeds limit".into());
    }
    Ok(pkg.read(e)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if !(a.len() == 7 || a.len() == 8)
        || a[6] != "--admit-loader-complete-unchanged-and-stable-packages"
        || (a.len() == 8 && a[7] != "--admit-contact-categories-1-2-current-impact")
    {
        return Err("usage: cell_event_snr_explicit_inspection_v1 <event-package> <event-instance-8hex> <event-sha256> <row-decimal> <snr-package> <snr-sha256> --admit-loader-complete-unchanged-and-stable-packages [--admit-contact-categories-1-2-current-impact]".into());
    }
    for x in [&a[2], &a[5]] {
        if x.len() != 64 || !x.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid payload SHA-256".into());
        }
    }
    if a[1].len() != 8 || !a[1].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("event instance must be eight hex digits".into());
    }
    let id = u32::from_str_radix(&a[1], 16)?;
    if a.len() == 8 && admitted_contact_event(1, 2, 1) != Some(id) {
        return Err("event instance does not match admitted contact-table selection".into());
    }
    let row: usize = a[3].parse()?;
    let ek = NativeResourceKey {
        instance_id: id,
        type_id: 0x02b9f662,
        group_id: SCOPE,
    };
    let event = package_read(Path::new(&a[0]), ek, 4 * 1024 * 1024)?;
    if !hash(&event).eq_ignore_ascii_case(&a[2]) {
        return Err("event payload SHA-256 mismatch".into());
    }
    let selected = select(&event, row)?;
    let sk = NativeResourceKey {
        instance_id: selected,
        type_id: 0x01a527db,
        group_id: SCOPE,
    };
    let snr = package_read(Path::new(&a[4]), sk, 32 * 1024 * 1024)?;
    if !hash(&snr).eq_ignore_ascii_case(&a[5]) {
        return Err("SNR payload SHA-256 mismatch".into());
    }
    let decoded = match snr.first().map(|v| v & 15) {
        Some(2) => snr_pcm16::decode(&snr),
        Some(4) => snr_xas::decode(&snr),
        _ => return Err("unsupported SNR codec".into()),
    }?;
    let mut h = Sha256::new();
    for p in &decoded.pcm {
        h.update(p.to_le_bytes())
    }
    println!("INSPECTION event={ek} event_sha256={} row={} selected_snr={sk} snr_sha256={} rate={} channels={} samples={} loop_start={:?} pcm_le_sha256={:x}",hash(&event),row,hash(&snr),decoded.sample_rate,decoded.channels,decoded.pcm.len(),decoded.loop_start,h.finalize());
    println!(
        "Declared event property count={} (count-driven native subset; no separator skipping).",
        u32::from_be_bytes(event[..4].try_into()?)
    );
    println!("Restricted count-driven reader model; caller admits the dedicated loader, stable package paths, unchanged complete reads and getter identity. Row is explicit, not native RNG/cache selection. No live routing, provider precedence, device output or native codec parity is established.");
    if a.len() == 8 {
        println!("CONTACT INSPECTION: admitted current record, flag 1, category outputs 1 and 2, initialized source table; FNV-1 lowercase cell_collision_creature_plant_impact = 34C2FF0B. No live contact or native scheduling observed.");
    }
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
    fn admitted_contact_table_is_bounded() {
        assert_eq!(admitted_contact_event(1, 2, 1), Some(0x34c2ff0b));
        assert_eq!(admitted_contact_event(2, 1, 1), Some(0x34c2ff0b));
        assert_eq!(admitted_contact_event(1, 2, 0), None);
        assert_eq!(admitted_contact_event(1, 1, 1), None);
    }
    fn payload(rows: &[[u8; 12]]) -> Vec<u8> {
        let mut b = 1u32.to_be_bytes().to_vec();
        b.extend(0x701ed91eu32.to_be_bytes());
        b.extend(0x20u16.to_be_bytes());
        b.extend(0x9cu16.to_be_bytes());
        b.extend((rows.len() as u32).to_be_bytes());
        b.extend(12u32.to_be_bytes());
        for r in rows {
            b.extend(r)
        }
        b
    }
    #[test]
    fn selected_word_is_little_endian_and_explicit() {
        let mut r = [0; 12];
        r[..4].copy_from_slice(&0x12345678u32.to_le_bytes());
        let p = payload(&[r]);
        assert_eq!(select(&p, 0).unwrap(), 0x12345678);
        assert!(select(&p, 1).is_err());
        assert!(select(&payload(&[[0; 12]]), 0).is_err());
    }
    #[test]
    fn count_and_default_scalar_framing_are_not_inspection_walk() {
        let mut row = [0; 12];
        row[..4].copy_from_slice(&0x12345678u32.to_le_bytes());
        let sample = payload(&[row]);
        let mut p = 2u32.to_be_bytes().to_vec();
        p.extend(0x5f6317d5u32.to_be_bytes());
        p.extend(0x20u16.to_be_bytes());
        p.extend(0u16.to_be_bytes());
        p.extend([0u8; 16]);
        p.extend(&sample[4..]);
        assert_eq!(select(&p, 0).unwrap(), 0x12345678);
        let mut zero = p.clone();
        zero[..4].copy_from_slice(&0u32.to_be_bytes());
        assert!(select(&zero, 0).is_err());
        let mut low = p.clone();
        low[..4].copy_from_slice(&1u32.to_be_bytes());
        assert!(select(&low, 0).is_err());
        let mut high = p.clone();
        high[..4].copy_from_slice(&0x80000002u32.to_be_bytes());
        assert!(select(&high, 0).is_err());
        let mut too_many = p.clone();
        too_many[..4].copy_from_slice(&3u32.to_be_bytes());
        assert!(select(&too_many, 0).is_err());
        let mut separator = sample.clone();
        separator.splice(4..4, [1, 0, 0, 0]);
        assert!(select(&separator, 0).is_err());
        let mut unsupported = sample;
        unsupported.splice(4..4, [0, 0, 0, 1, 0, 1, 0, 0, 0]);
        unsupported[..4].copy_from_slice(&2u32.to_be_bytes());
        assert!(select(&unsupported, 0).is_err());
    }
    #[test]
    fn rejects_inexact_duplicate_and_wrong_stride() {
        let r = [1; 12];
        let p = payload(&[r]);
        let mut short = p.clone();
        short.pop();
        assert!(select(&short, 0).is_err());
        let mut dup = p.clone();
        dup[..4].copy_from_slice(&2u32.to_be_bytes());
        dup.extend(&p[4..]);
        assert!(select(&dup, 0).is_err());
        let mut stride = p;
        stride[16..20].copy_from_slice(&11u32.to_be_bytes());
        stride.pop();
        assert!(select(&stride, 0).is_err());
    }
}
