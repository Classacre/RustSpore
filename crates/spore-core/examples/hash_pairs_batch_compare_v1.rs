//! Bounded stdin batch roundtrip through the Rust hash-pair parser.
//! HPS1, LE case count, then LE byte length and raw payload for each case.
//! Output: HPO1, LE count, then LE section count, LE byte length, and the
//! reconstructed BE payload for each case. Only local source-derived data.
use std::io::{self, Read, Write};
const MAX_BATCH: usize = 64 * 1024 * 1024;
const MAX_CASE: usize = 1024 * 1024;
const MAX_CASES: usize = 2590;
fn word(data: &[u8], at: &mut usize) -> Result<u32, &'static str> {
    let end = at.checked_add(4).ok_or("offset overflow")?;
    let bytes: [u8; 4] = data.get(*at..end).ok_or("truncated word")?.try_into().map_err(|_| "word")?;
    *at = end; Ok(u32::from_le_bytes(bytes))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    io::stdin().lock().take((MAX_BATCH + 1) as u64).read_to_end(&mut input)?;
    if input.len()>MAX_BATCH || input.get(..4)!=Some(&b"HPS1"[..]) { return Err("invalid batch header or size".into()); }
    let mut at=4; let count=word(&input,&mut at)? as usize;
    if count>MAX_CASES { return Err("too many cases".into()); }
    let mut out=Vec::new(); out.extend_from_slice(b"HPO1"); out.extend_from_slice(&(count as u32).to_le_bytes());
    for _ in 0..count {
        let len=word(&input,&mut at)? as usize;
        if len>MAX_CASE { return Err("case too large".into()); }
        let end=at.checked_add(len).ok_or("case offset overflow")?;
        let raw=input.get(at..end).ok_or("truncated case")?; at=end;
        let parsed=spore_core::hash_pairs::parse(raw)?;
        if !parsed.exact || parsed.consumed!=len { return Err("parser did not consume case".into()); }
        let mut sections:Vec<&[spore_core::hash_pairs::HashPair]>=Vec::new();
        if parsed.section_count>=1 { sections.push(&parsed.section_a); }
        if parsed.section_count>=2 { sections.push(&parsed.section_b); }
        sections.extend(parsed.extra_sections.iter().map(Vec::as_slice));
        if sections.len()!=parsed.section_count { return Err("inconsistent section count".into()); }
        let mut reconstructed=Vec::with_capacity(len); reconstructed.extend_from_slice(&parsed.version.to_be_bytes());
        for section in sections {
            reconstructed.extend_from_slice(&(u32::try_from(section.len())?).to_be_bytes());
            for pair in section { for v in [pair.hash,pair.marker,pair.value] { reconstructed.extend_from_slice(&v.to_be_bytes()); } }
        }
        if reconstructed.len()!=len || out.len().checked_add(8+len).is_none_or(|n|n>MAX_BATCH) { return Err("invalid output size".into()); }
        out.extend_from_slice(&(u32::try_from(parsed.section_count)?).to_le_bytes()); out.extend_from_slice(&(len as u32).to_le_bytes()); out.extend_from_slice(&reconstructed);
    }
    if at!=input.len() { return Err("trailing batch data".into()); }
    io::stdout().lock().write_all(&out)?; Ok(())
}
