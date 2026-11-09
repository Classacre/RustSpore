//! Narrow FF32D0 one-child projection: one explicit child scalar descriptor
//! from the source's 19-entry table, then a six-field parent. Successful full
//! reads, synthetic initialization and stable storage are caller admissions;
//! actual deallocation, lifetime and live vehicle identity are not modeled.
use crate::{civilization_first_entry_v1, descriptor_frame_v1};

/// Fixed cleanup admissions: one child, +90 null, two empty strings,
/// and nonzero synthetic headers for the two disjoint subreserve buffers.
/// These events represent guarded callbacks, not actual deallocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupEvent { SubreserveB, SubreserveA, Child, FirstEmptyCopy, SecondEmptyCopy }

pub fn fixed_cleanup_events() -> [CleanupEvent; 5] {
    use CleanupEvent::*;
    [SubreserveB, SubreserveA, Child, FirstEmptyCopy, SecondEmptyCopy]
}

#[derive(Debug, PartialEq, Eq)]
pub struct OneChild<'a> {
    pub words: [u16; 14],
    pub child_scalar: u32,
    pub parent: civilization_first_entry_v1::Entry<'a>,
}

/// `parent_scratch` is caller-owned so the returned parent can borrow its tail.
pub fn read<'a>(bytes: &[u8], parent_scratch: &'a mut Vec<u8>, max_bytes: usize,
                max_keys: usize) -> Result<OneChild<'a>, &'static str> {
    if bytes.len() > max_bytes { return Err("limit"); }
    if bytes.get(..4) != Some(&[0, 0, 0, 1]) { return Err("child count"); }
    let shorts = bytes.get(4..32).ok_or("short words")?;
    let mut words = [0; 14];
    for (i, x) in shorts.chunks_exact(2).enumerate() {
        words[i] = u16::from_be_bytes([x[0], x[1]]);
    }
    let size = u32::from_be_bytes(bytes.get(36..40).ok_or("short frame")?
        .try_into().unwrap()) as usize;
    let end = 40usize.checked_add(size).ok_or("overflow")?;
    let frame = bytes.get(32..end).ok_or("short child frame")?;
    let fields = descriptor_frame_v1::parse(frame, 0x01a80d26, &[0x032214b1], 1, max_bytes)
        .map_err(|_| "child frame")?;
    let child_scalar = u32::from_be_bytes(fields[0].bytes.try_into().map_err(|_| "child scalar")?);
    let after_count = end.checked_add(4).ok_or("overflow")?;
    if bytes.get(end..after_count) != Some(&[0, 0, 0, 0]) { return Err("second count"); }
    let parent = bytes.get(after_count..).ok_or("parent")?;
    parent_scratch.clear();
    parent_scratch.extend_from_slice(&[0; 8]);
    parent_scratch.extend_from_slice(parent);
    let parent = civilization_first_entry_v1::entry(parent_scratch, max_bytes, max_keys)?;
    Ok(OneChild { words, child_scalar, parent })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_prefix_and_short_child() {
        let mut scratch = Vec::new();
        assert!(read(&[0; 4], &mut scratch, 100, 16).is_err());
        assert!(read(&[0, 0, 0, 1], &mut scratch, 100, 16).is_err());
        assert!(read(&[0, 0, 0, 1], &mut scratch, 3, 16).is_err());
    }
}
