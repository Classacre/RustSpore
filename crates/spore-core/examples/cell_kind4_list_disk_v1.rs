//! Explicit local disk projection; does not establish runtime selection.
use spore_core::cell_kind4_list_disk_v1 as disk;

use std::{env, fs, process};

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: cell_kind4_list_disk_v1 <decoded-payload>");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("expected exactly one decoded payload");
        process::exit(2);
    }
    // A policy cap also prevents the example from loading arbitrary large files.
    let result = (|| {
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        use std::io::Read;
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() > 1024 * 1024 { return Err("payload exceeds 1 MiB".to_string()); }
        let projection = disk::parse(&bytes).map_err(|e| format!("invalid projection: {e:?}"))?;
        println!("count={} offset={} payload_bytes={}", projection.count, projection.array_offset, projection.payload.len());
        for (index, row) in projection.rows.chunks_exact(disk::ROW_BYTES).enumerate() {
            print!("row {index}:");
            for byte in row { print!(" {byte:02x}"); }
            println!();
        }
        Ok::<(), String>(())
    })();
    if let Err(error) = result { eprintln!("{error}"); process::exit(1); }
}
