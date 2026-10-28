//! Emit every v7 field as JSON for an independent corpus comparison.
//! Input is a decompressed payload on stdin. Keep output under target/.
use spore_core::animlist_v7;
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
    let limits = animlist_v7::Limits::default();
    let mut bytes = Vec::new();
    io::stdin().lock().take((limits.bytes as u64) + 1).read_to_end(&mut bytes)?;
    let parsed = animlist_v7::parse(&bytes, limits).map_err(|e| format!("{e:?}"))?;
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    write!(out, "{{\"records\":[")?;
    for (i, r) in parsed.records.iter().enumerate() {
        if i != 0 { write!(out, ",")?; }
        write!(out, "{{\"key\":{},\"paths_utf16le_hex\":[", r.key)?;
        for (j, path) in r.paths_utf16le.iter().enumerate() {
            if j != 0 { write!(out, ",")?; }
            write!(out, "\"{}\"", hex(path))?;
        }
        write!(out, "],\"path_words\":[")?;
        for (j, word) in r.path_words.iter().enumerate() {
            if j != 0 { write!(out, ",")?; }
            write!(out, "{}", word)?;
        }
        write!(out, "],\"shared_words\":[{},{}],\"flag_before_word\":{},\"word_between_flags\":{},\"flag_after_word\":{},\"trailing_words\":[{},{},{}]}}",
            r.shared_words[0], r.shared_words[1], r.flag_before_word, r.word_between_flags,
            r.flag_after_word, r.trailing_words[0], r.trailing_words[1], r.trailing_words[2])?;
    }
    writeln!(out, "]}}")?;
    out.flush()?;
    Ok(())
}
fn main() {
    if let Err(e) = run() { eprintln!("AnimList v7 parse failed: {e}"); std::process::exit(1); }
}
