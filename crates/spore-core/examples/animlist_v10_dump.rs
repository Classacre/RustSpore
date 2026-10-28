//! Emit all v10 fields as JSON for a separate parser comparison.
//! Input is a decompressed payload on stdin. Redirect output only under target/.
use spore_core::animlist_v10;

use std::io::{self, Read, Write};

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        result.push(DIGITS[(b >> 4) as usize] as char);
        result.push(DIGITS[(b & 15) as usize] as char);
    }
    result
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let limits = animlist_v10::Limits::default();
    let mut bytes = Vec::new();
    io::stdin().lock().take((limits.bytes as u64) + 1).read_to_end(&mut bytes)?;
    let parsed = animlist_v10::parse(&bytes, limits).map_err(|e| format!("{e:?}"))?;
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    write!(out, "{{\"records\":[")?;
    for (i, r) in parsed.records.iter().enumerate() {
        if i != 0 { write!(out, ",")?; }
        write!(out, "{{\"key\":{},\"name_utf16le_hex\":\"{}\",\"words_before_flags\":[{},{}],\"flags\":[{},{},{}],\"words_after_flags\":[{},{},{}],\"variants\":[",
            r.key, hex(r.name_utf16le), r.words_before_flags[0], r.words_before_flags[1],
            r.flags[0], r.flags[1], r.flags[2], r.words_after_flags[0], r.words_after_flags[1], r.words_after_flags[2])?;
        for (j, v) in r.variants.iter().enumerate() {
            if j != 0 { write!(out, ",")?; }
            write!(out, "{{\"threshold_bits\":{},\"entries\":[", v.threshold_bits)?;
            for (k, e) in v.entries.iter().enumerate() {
                if k != 0 { write!(out, ",")?; }
                write!(out, "{{\"words\":[{},{},{}],\"path_utf16le_hex\":\"{}\"}}", e.words[0], e.words[1], e.words[2], hex(e.path_utf16le))?;
            }
            write!(out, "]}}")?;
        }
        write!(out, "]}}")?;
    }
    writeln!(out, "]}}")?;
    out.flush()?;
    Ok(())
}
fn main() {
    if let Err(e) = run() { eprintln!("AnimList v10 parse failed: {e}"); std::process::exit(1); }
}
