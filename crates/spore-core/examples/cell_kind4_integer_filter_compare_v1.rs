//! Bounded text protocol for the explicit integer set projection.
//! No runtime registration, ordering or actor-selection claim.
use std::io::{self, Read, Write};
use spore_core::cell_kind4_integer_filter_v1::{self as filter, Condition, Entry};

const MAX_INPUT: u64 = 64 * 1024 * 1024;
const MAX_CASES: usize = 128;
const MAX_OUTPUT: usize = 64 * 1024 * 1024;
fn read_num<T: std::str::FromStr>(it: &mut std::str::SplitWhitespace<'_>) -> Result<T, &'static str> {
    it.next().ok_or("missing word")?.parse().map_err(|_| "invalid word")
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut raw = Vec::new();
    io::stdin().lock().take(MAX_INPUT + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 > MAX_INPUT { return Err("input limit".into()); }
    let input = std::str::from_utf8(&raw)?;
    let mut it = input.split_whitespace();
    if it.next() != Some("K4F1") { return Err("invalid protocol".into()); }
    let nc: usize = read_num(&mut it)?;
    if nc > filter::MAX_CANDIDATES { return Err("candidate limit".into()); }
    let mut candidates = Vec::with_capacity(nc);
    for _ in 0..nc { candidates.push(read_num::<u32>(&mut it)?); }
    let ne: usize = read_num(&mut it)?;
    if ne > filter::MAX_ENTRIES { return Err("entry limit".into()); }
    let mut storage = Vec::with_capacity(ne);
    let mut total = 0usize;
    for _ in 0..ne {
        let field: u32 = read_num(&mut it)?;
        let value: i32 = read_num(&mut it)?;
        let count: usize = read_num(&mut it)?;
        total = total.checked_add(count).ok_or("posting overflow")?;
        if total > filter::MAX_POSTINGS { return Err("posting limit".into()); }
        let mut postings = Vec::with_capacity(count);
        for _ in 0..count { postings.push(read_num::<u32>(&mut it)?); }
        storage.push((field, value, postings));
    }
    let entries: Vec<_> = storage.iter().map(|(field,value,postings)| Entry { field: *field, value: *value, postings }).collect();
    let cases: usize = read_num(&mut it)?;
    if cases > MAX_CASES { return Err("case limit".into()); }
    let mut out = String::from("K4O1\n");
    for _ in 0..cases {
        let universe: u8 = read_num(&mut it)?;
        if universe > 1 { return Err("invalid universe".into()); }
        let count: usize = read_num(&mut it)?;
        if count > filter::MAX_CONDITIONS { return Err("condition limit".into()); }
        let mut conditions = Vec::with_capacity(count);
        for _ in 0..count {
            let mode: u8 = read_num(&mut it)?;
            let field: u32 = read_num(&mut it)?;
            let lower: i32 = read_num(&mut it)?;
            let upper: i32 = read_num(&mut it)?;
            match mode {
                0 => conditions.push(Condition { field, start: lower, end: upper }),
                1 => if let Some(c) = filter::adjusted_pair(field, lower, upper) { conditions.push(c); },
                _ => return Err("invalid condition mode".into()),
            }
        }
        let selected: &[u32] = if universe == 1 { &[] } else { &candidates };
        let result = filter::filter(selected, &entries, &conditions).map_err(|e| format!("filter: {e:?}"))?;
        let mut line = result.len().to_string();
        for ordinal in result { line.push(' '); line.push_str(&ordinal.to_string()); }
        line.push('\n');
        if out.len().checked_add(line.len()).is_none_or(|n| n > MAX_OUTPUT) {
            return Err("output limit".into());
        }
        out.push_str(&line);
    }
    if it.next().is_some() { return Err("trailing protocol words".into()); }
    io::stdout().lock().write_all(out.as_bytes())?;
    Ok(())
}
