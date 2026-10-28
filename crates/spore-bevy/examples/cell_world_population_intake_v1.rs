//! Inspect a world or population payload from an explicitly named package/key.
//! No overlay, live selection, or runtime handle fixup is implied.
#[path = "../../spore-core/src/cell_world_population_view_v1.rs"]
mod view;
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use std::{fs::File, io::Read};

const MAX_INDEX: u32 = 64 * 1024 * 1024;
const MAX_ENTRIES: u32 = 2_000_000;
const MAX_RESOURCE: u32 = 16 * 1024 * 1024;
// CLI output policy, not an original format constraint.
const MAX_OUTPUT_ROWS: u32 = 4096;

fn hex(value: &str) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(u32::from_str_radix(value.trim_start_matches("0x"), 16)?)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 { return Err("usage: <world|populate> <package> <type-hex> <group-hex> <instance-hex>".into()); }
    let key = NativeResourceKey { type_id: hex(&args[3])?, group_id: hex(&args[4])?, instance_id: hex(&args[5])? };
    if !matches!((args[1].as_str(), key.type_id), ("world", 0x9b8e862f) | ("populate", 0xda141c1b)) {
        return Err("mode and resource type do not match".into());
    }
    let mut file = File::open(&args[2])?;
    let mut raw = [0u8; dbpf::HEADER_SIZE];
    file.read_exact(&mut raw)?;
    let header = dbpf::DbpfHeader::parse(&raw)?;
    if header.index_size > MAX_INDEX || header.index_entry_count > MAX_ENTRIES {
        return Err("package index exceeds size limit".into());
    }
    // The package must remain unchanged between this check and the later reopen.
    let archive = NativePackageIndex::open(&args[2])?;
    let entry = archive.index().find_unique(key)?.ok_or("key absent in explicit package")?;
    if entry.stored_size > MAX_RESOURCE || entry.decompressed_size > MAX_RESOURCE {
        return Err("selected resource exceeds size limit".into());
    }
    let bytes = archive.read(entry)?;
    let count_offset = if args[1] == "world" { 0 } else { 8 };
    let raw_count: [u8; 4] = bytes.get(count_offset..count_offset + 4)
        .ok_or("truncated projected array count")?.try_into()?;
    if u32::from_le_bytes(raw_count) > MAX_OUTPUT_ROWS {
        return Err("projected row count exceeds CLI output policy (4096)".into());
    }
    println!("package={} key={} decoded_bytes={} raw_disk_words=true runtime_correspondence=unverified opaque_sections_unchecked=true cli_max_output_rows={}", archive.path().display(), key, bytes.len(), MAX_OUTPUT_ROWS);
    match args[1].as_str() {
        "world" if key.type_id == 0x9b8e862f => {
            let v = view::parse_world(&bytes)?;
            println!("projected_rows={} raw_header_08={} raw_header_0c={:08X}", v.rows.len(), v.raw_word_08, v.raw_word_0c);
            for (i, row) in v.rows.iter().enumerate() {
                println!("row={i} raw_word_00={:08X} raw_bytes_04={:02X?} raw_word_08={:08X}", row.raw_word_00, row.raw_bytes_04, row.raw_word_08);
            }
        }
        "populate" if key.type_id == 0xda141c1b => {
            let rows = view::parse_population_rows(&bytes)?;
            for (i, row) in rows.iter().enumerate() {
                println!("row={i} raw_word_28={:08X} raw_word_18={:08X} raw_word_1c={:08X} raw_word_20={:08X} raw_word_3c={:08X} raw_word_40={:08X} raw_byte_44={:02X}",
                    row.word(0x28).unwrap(), row.word(0x18).unwrap(), row.word(0x1c).unwrap(),
                    row.word(0x20).unwrap(), row.word(0x3c).unwrap(), row.word(0x40).unwrap(), row.bytes[0x44]);
            }
        }
        _ => return Err("mode and resource type do not match".into()),
    }
    Ok(())
}
