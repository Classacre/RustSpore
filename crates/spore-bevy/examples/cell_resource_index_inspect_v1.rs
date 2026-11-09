//! Read-only aggregate inspection of one explicitly supplied archive.
//! The archive must remain stable throughout the read; before/after hashes
//! detect changes but cannot prove an atomic snapshot. No payload is emitted.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::cell_resource_index_v1::{parse, spans};
use std::{error::Error, fs::File, io::Read, path::Path};

// Source 9329B0/932A50: non-reflected CRC, init all ones, final complement.
fn source_crc(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in bytes {
        crc ^= u32::from(byte) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 { (crc << 1) ^ 0x04c1_1db7 } else { crc << 1 };
        }
    }
    !crc
}

fn hash(path: &Path) -> Result<[u8; 32], Box<dyn Error>> {
    let mut f = File::open(path)?;
    if f.metadata()?.len() > 128 * 1024 * 1024 {
        return Err("archive exceeds limit".into());
    }
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    let mut total = 0u64;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > 128 * 1024 * 1024 {
            return Err("archive exceeds limit".into());
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().into())
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("supply one archive path")?;
    if args.next().is_some() {
        return Err("supply one archive path".into());
    }
    let path = Path::new(&path);
    let before = hash(path)?;
    let archive = NativePackageIndex::open(path)?;
    let entries = archive.entries();
    let find = |key: NativeResourceKey| -> Result<_, Box<dyn Error>> {
        let mut matches = entries.iter().filter(|e| e.key == key);
        let entry = matches.next().ok_or("missing resource")?;
        if matches.next().is_some() {
            return Err("duplicate resource".into());
        }
        if entry.decompressed_size > 16 * 1024 * 1024 || entry.stored_size > 16 * 1024 * 1024 {
            return Err("resource exceeds limit".into());
        }
        Ok(entry)
    };
    let index_key = NativeResourceKey {
        instance_id: 0x0179d304,
        type_id: 0x0179d310,
        group_id: 0,
    };
    let data = archive.read(find(index_key)?)?;
    let index = parse(&data, 4096).map_err(|_| "invalid index frame")?;
    let keys: [[u8; 4]; 4] =
        [0xabcdef00u32, 0xabcd0000, 0xabcd0001, 0xabcd0011].map(u32::to_be_bytes);
    let mut counts = Vec::new();
    for pair in &index.pairs {
        let key = NativeResourceKey {
            instance_id: 0x0179d304,
            type_id: pair[0],
            group_id: 0,
        };
        let bytes = archive.read(find(key)?)?;
        let mut count = 0usize;
        let mut short = 0usize;
        let mut class_matches = 0usize;
        let mut crc_checked = 0usize;
        let mut crc_matches = 0usize;
        for span in spans(&bytes, pair[1] as usize, 1_000_000)
            .map_err(|_| "invalid conditional span frame")?
        {
            count += 1;
            if span.len() - 8 <= 1_000_000 {
                crc_checked += 1;
                let expected = u32::from_be_bytes(span[4..8].try_into().unwrap());
                crc_matches += usize::from(source_crc(&span[8..]) == expected);
            }
            if span.len() < 12 {
                short += 1;
            } else if span[8..12] == 0xabcdef00u32.to_be_bytes() {
                class_matches += 1;
            }
        }
        let hits = keys.map(|key| bytes.windows(4).filter(|w| *w == key).count());
        counts.push((count, short, class_matches, crc_checked, crc_matches, hits));
    }
    if hash(path)? != before {
        return Err("archive changed while reading".into());
    }
    println!("index_pairs={} archive_hash_stable=true", counts.len());
    for (ordinal, (count, short, class_matches, crc_checked, crc_matches, hits)) in counts.iter().enumerate() {
        println!(
            "resource_ordinal={ordinal} conditional_spans={count} short_spans={short} source_class_word_matches={class_matches} crc_checked={crc_checked} crc_matches={crc_matches} source_key_byte_counts={hits:?}"
        );
    }
    Ok(())
}
