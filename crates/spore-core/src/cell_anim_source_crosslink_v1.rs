//! Source-only join: table request keys to list records and v10 entry word +8
//! to caller-supplied indexed ANIM instances. Numeric matches are candidates,
//! including zero keys and filename paths; they do not prove a keyed load,
//! resource precedence, or playback.
use crate::{animlist_v7, animlist_v10, cell_anim_state_table_v1};

/// Caller-supplied package/resource identity. Index ordinal distinguishes duplicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity { pub package: String, pub index_ordinal: usize, pub type_id: u32, pub group: u32, pub instance: u64 }

pub enum Records<'a, 'b> { V7(&'a [animlist_v7::Record<'b>]), V10(&'a [animlist_v10::Record<'b>]) }
pub struct List<'a, 'b> { pub identity: Identity, pub records: Records<'a, 'b> }
#[derive(Debug, PartialEq, Eq)]
pub struct EntryLink { pub variant: usize, pub entry: usize, pub anim_instance: u32, pub anim_indexes: Vec<usize> }
#[derive(Debug, PartialEq, Eq)]
pub struct RecordLink { pub list_index: usize, pub record_index: usize, pub entries: Vec<EntryLink> }
#[derive(Debug, PartialEq, Eq)]
pub struct RowLink { pub row_index: usize, pub records: Vec<RecordLink> }

/// Explicit total output ceilings, independent of parser input limits.
#[derive(Debug, Clone, Copy)]
pub struct Limits { pub rows: usize, pub records: usize, pub entries: usize, pub anim_matches: usize }
#[derive(Debug, PartialEq, Eq)]
pub enum JoinError { LimitExceeded, AllocationFailed }

fn push<T>(items: &mut Vec<T>, value: T) -> Result<(), JoinError> {
    items.try_reserve(1).map_err(|_| JoinError::AllocationFailed)?;
    items.push(value);
    Ok(())
}
fn increment(total: &mut usize, limit: usize) -> Result<(), JoinError> {
    *total = total.checked_add(1).ok_or(JoinError::LimitExceeded)?;
    if *total > limit { return Err(JoinError::LimitExceeded); }
    Ok(())
}

/// Returns one row per input row in table order. All nested orders follow input
/// order and duplicates are retained. v7 has no keyed ANIM entries. The caller
/// provides limits for the total rows, record links, entries, and match edges.
/// An error discards partial output; no selection or resolution is asserted.
pub fn crosslink(rows: &[cell_anim_state_table_v1::Record], lists: &[List<'_, '_>], anims: &[Identity], limits: Limits) -> Result<Vec<RowLink>, JoinError> {
    if rows.len() > limits.rows { return Err(JoinError::LimitExceeded); }
    let mut output = Vec::new();
    output.try_reserve_exact(rows.len()).map_err(|_| JoinError::AllocationFailed)?;
    let (mut record_count, mut entry_count, mut match_count) = (0usize, 0usize, 0usize);
    for (row_index, row) in rows.iter().enumerate() {
        let mut matches = Vec::new();
        for (list_index, list) in lists.iter().enumerate() {
            match &list.records {
                Records::V7(records) => for (record_index, record) in records.iter().enumerate() {
                    if record.key == row.request_key() {
                        increment(&mut record_count, limits.records)?;
                        push(&mut matches, RecordLink { list_index, record_index, entries: Vec::new() })?;
                    }
                },
                Records::V10(records) => for (record_index, record) in records.iter().enumerate() {
                    if record.key != row.request_key() { continue; }
                    increment(&mut record_count, limits.records)?;
                    let mut entries = Vec::new();
                    for (variant, v) in record.variants.iter().enumerate() {
                        for (entry, e) in v.entries.iter().enumerate() {
                            increment(&mut entry_count, limits.entries)?;
                            let anim_instance = e.words[2];
                            let mut anim_indexes = Vec::new();
                            for (i, id) in anims.iter().enumerate() {
                                if id.type_id == 0xee17c6ad && id.instance == u64::from(anim_instance) {
                                    increment(&mut match_count, limits.anim_matches)?;
                                    push(&mut anim_indexes, i)?;
                                }
                            }
                            push(&mut entries, EntryLink { variant, entry, anim_instance, anim_indexes })?;
                        }
                    }
                    push(&mut matches, RecordLink { list_index, record_index, entries })?;
                },
            }
        }
        push(&mut output, RowLink { row_index, records: matches })?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(package: &str, ordinal: usize, typ: u32, instance: u64) -> Identity { Identity { package: package.into(), index_ordinal: ordinal, type_id: typ, group: 1, instance } }
    #[test]
    fn preserves_all_orders_duplicates_and_unmatched_rows() {
        let rows = [cell_anim_state_table_v1::Record { words: [0, 7, 0, 0, 0, 0, 0, 0, 0] }, cell_anim_state_table_v1::Record { words: [1, 8, 0, 0, 0, 0, 0, 0, 0] }, cell_anim_state_table_v1::Record { words: [2, 7, 0, 0, 0, 0, 0, 0, 0] }];
        let mk = || animlist_v10::Record { key: 7, name_utf16le: &[], words_before_flags: [0;2], flags: [0;3], words_after_flags: [0;3], variants: vec![animlist_v10::Variant { threshold_bits: 0, entries: vec![animlist_v10::Entry { words: [0, 0, 42], path_utf16le: &[] }, animlist_v10::Entry { words: [0, 0, 99], path_utf16le: &[] }] }] };
        let v10 = vec![mk(), mk()];
        let v7 = vec![animlist_v7::Record { key: 7, paths_utf16le: vec![&[]], path_words: vec![42], shared_words: [0;2], flag_before_word: 0, word_between_flags: 0, flag_after_word: 0, trailing_words: [0;3] }];
        let lists = [List { identity: id("a", 1, 1, 2), records: Records::V10(&v10) }, List { identity: id("b", 2, 1, 2), records: Records::V7(&v7) }];
        let anims = [id("one", 0, 0xee17c6ad, 42), id("two", 3, 0xee17c6ad, 42), id("wrong type", 4, 1, 42), id("high bits", 5, 0xee17c6ad, 0x10000002a)];
        let limits = Limits { rows: 3, records: 6, entries: 8, anim_matches: 8 };
        let links = crosslink(&rows, &lists, &anims, limits).unwrap();
        assert_eq!(links.iter().map(|r| r.row_index).collect::<Vec<_>>(), [0, 1, 2]);
        assert!(links[1].records.is_empty());
        assert_eq!(links[0].records.iter().map(|r| (r.list_index, r.record_index)).collect::<Vec<_>>(), [(0,0), (0,1), (1,0)]);
        assert_eq!(links[0].records[0].entries[0].anim_indexes, [0, 1]);
        assert!(links[0].records[0].entries[1].anim_indexes.is_empty());
        assert!(links[0].records[2].entries.is_empty());
        assert_eq!(links[0].records, links[2].records);
        for limited in [Limits { rows: 2, ..limits }, Limits { records: 5, ..limits },
            Limits { entries: 7, ..limits }, Limits { anim_matches: 7, ..limits }] {
            assert_eq!(crosslink(&rows, &lists, &anims, limited), Err(JoinError::LimitExceeded));
        }
    }
}
