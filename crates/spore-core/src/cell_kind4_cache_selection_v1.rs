//! Conditional E65640/E5C0F0 kind-4 cache and four-row selection.
//!
//! Static evidence: re/cell-kind4-cache-selection-static-v1.md. This is
//! not live Cell gameplay. Caller supplies handle words, runtime row snapshot,
//! RNG and FP state, cache and type-0 third word; no provider, fixup, native
//! pointer, original actor, actual cache lifetime or mounted winner is inferred.
use std::{cmp::Ordering, collections::BTreeMap};
use crate::{cell_kind4_list_disk_v1::Projection, cell_request_rng_v1::Rng,
    native_x87::{Context, Extended, Precision, X87Error}};

const IDS: [u32; 4] = [0xa1c4_6df2, 0xe219_e5d0, 0xe219_e5d3, 0xe219_e5d2];
const MAX_CACHE: usize = 256;

/// Caller admits that these explicit row bytes represent the already-resolved,
/// stable runtime array, and that cache access is exclusive/nonreentrant.
/// This token does not perform resource resolution or prove those conditions.
#[derive(Debug, Clone, Copy)]
pub struct SnapshotAdmission;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot { Pending, Ready([u32; 3]) }
#[derive(Debug, Default)]
pub struct Cache { slots: BTreeMap<u32, Slot> }
impl Cache {
    pub fn slot(&self, key: u32) -> Option<Slot> { self.slots.get(&key).copied() }
    /// Seed an explicitly supplied pre-existing hit; never overwrite a slot.
    pub fn admit_ready(&mut self, key: u32, triple: [u32; 3], _admission: SnapshotAdmission) -> Result<(), Failure> {
        if self.slots.contains_key(&key) { return Err(Failure::AlreadyPresent); }
        if self.slots.len() == MAX_CACHE { return Err(Failure::CacheFull); }
        self.slots.insert(key, Slot::Ready(triple));
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure { CacheFull, AlreadyPresent, ReentrantPending, UnsupportedRows, UnsupportedControl, UnsupportedRng, Arithmetic(X87Error) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultV1 {
    Hit { triple: [u32; 3], rng_state: u32 },
    Selected { triple: [u32; 3], selected: usize, rng_state: u32, draws: u8 },
    /// An insertion already happened; later failure leaves an explicit pending
    /// placeholder. The native failure contents/lifetime are not established.
    Pending { failure: Failure, rng_state: u32, draws: u8 },
    /// Safe capacity policy or pre-existing pending key; no new insertion.
    Rejected(Failure),
}

fn candidate_rows(view: &Projection<'_>) -> Option<[u32; 4]> {
    if view.count != 4 || view.rows.len() != 4*0x1c { return None; }
    let mut ids = [0; 4];
    for (i, row) in view.rows.chunks_exact(0x1c).enumerate() {
        let word = |at: usize| u32::from_le_bytes(row[at..at+4].try_into().unwrap());
        if word(0) != 0 || word(8) != 0x3f80_0000 || word(4) != IDS[i] { return None; }
        ids[i] = word(4);
    }
    Some(ids)
}

/// Key is an admitted post-fixup structure-handle word; `rows` is an admitted
/// resolved snapshot of the list selected on a cache miss. A hit does not
/// inspect rows, control, or RNG. `third` is the explicitly supplied value read
/// by the type-0 branch. No semantic meaning is assigned to this triple.
///
/// `E65640` obtains the cache slot before `E5C0F0`; this model retains a
/// placeholder on later unsupported/error paths. Native recovery, allocation
/// failure and placeholder observations are not claimed. RNG and cache changes
/// are reported explicitly and are not rolled back after a local error.
pub fn select(cache: &mut Cache, key: u32, rows: Option<&Projection<'_>>,
    rng_before: u32, control_word: u16, third: u32, _admission: SnapshotAdmission) -> ResultV1 {
    match cache.slots.get(&key).copied() {
        Some(Slot::Ready(triple)) => return ResultV1::Hit { triple, rng_state: rng_before },
        Some(Slot::Pending) => return ResultV1::Rejected(Failure::ReentrantPending),
        None => {}
    }
    if cache.slots.len() == MAX_CACHE { return ResultV1::Rejected(Failure::CacheFull); }
    cache.slots.insert(key, Slot::Pending);
    let pending = |failure, rng_state, draws| ResultV1::Pending { failure, rng_state, draws };
    let ids = match rows.and_then(candidate_rows) {
        Some(ids) => ids,
        None => return pending(Failure::UnsupportedRows, rng_before, 0),
    };
    if control_word & 0x3f != 0x3f { return pending(Failure::UnsupportedControl, rng_before, 0); }
    let x87 = match Context::from_control_word(control_word) {
        Ok(c) if c.precision != Precision::Bits24 => c,
        _ => return pending(Failure::UnsupportedControl, rng_before, 0),
    };
    let mut rng = match Rng::new(rng_before) {
        Ok(r) => r,
        Err(_) => return pending(Failure::UnsupportedRng, rng_before, 0),
    };
    let one = match Extended::from_f32(1.0) { Ok(x) => x, Err(e) => return pending(Failure::Arithmetic(e), rng.state(), 0) };
    let mut selected = None;
    for i in 0..4 {
        let fraction = rng.next_fraction();
        let draws = (i + 1) as u8;
        // Avoid claiming the omitted RNG negative-zero bit or status parity.
        if fraction.numerator == 0 { return pending(Failure::UnsupportedRng, rng.state(), draws); }
        // Four exactly representable weights yield exact totals 1..4 in both
        // the native f32 and f64 stores. Fraction as_f64 is exact. For this
        // domain 0 < fraction < 1, so the source clamp is a no-op.
        let random = match Extended::from_f64(fraction.as_f64()) { Ok(x) => x, Err(e) => return pending(Failure::Arithmetic(e), rng.state(), draws) };
        let total = match Extended::from_f32((i + 1) as f32) { Ok(x) => x, Err(e) => return pending(Failure::Arithmetic(e), rng.state(), draws) };
        let product = match x87.mul(random, total) { Ok(x) => x, Err(e) => return pending(Failure::Arithmetic(e), rng.state(), draws) };
        match x87.compare(one, product) {
            Ok(Ordering::Greater) => selected = Some(i),
            Ok(_) => {},
            Err(e) => return pending(Failure::Arithmetic(e), rng.state(), draws),
        }
    }
    // Under the admitted fraction range, the first weight wins initially.
    let selected = match selected { Some(i) => i, None => return pending(Failure::UnsupportedRng, rng.state(), 4) };
    let triple = [ids[selected], 0x3d97_a8e4, third];
    cache.slots.insert(key, Slot::Ready(triple));
    ResultV1::Selected { triple, selected, rng_state: rng.state(), draws: 4 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_kind4_list_disk_v1;
    fn bytes() -> Vec<u8> {
        let mut b=vec![0; 8+4*0x1c]; b[..4].copy_from_slice(&4u32.to_le_bytes()); b[4..8].copy_from_slice(&8u32.to_le_bytes());
        for (i, id) in IDS.iter().enumerate() { let at=8+i*0x1c; b[at+4..at+8].copy_from_slice(&id.to_le_bytes()); b[at+8..at+12].copy_from_slice(&1f32.to_bits().to_le_bytes()); }
        b
    }
    #[test]
    fn miss_draws_four_and_hit_ignores_different_list_and_rng() {
        let data=bytes(); let rows=cell_kind4_list_disk_v1::parse(&data).unwrap(); let mut c=Cache::default();
        let out=select(&mut c,0x42,Some(&rows),1,0x027f,7,SnapshotAdmission);
        assert_eq!(out,ResultV1::Selected { triple:[IDS[1],0x3d97_a8e4,7],selected:1,rng_state:0xd369_bf11,draws:4 });
        assert_eq!(select(&mut c,0x42,None,0,0,999,SnapshotAdmission), ResultV1::Hit { triple:[IDS[1],0x3d97_a8e4,7],rng_state:0 });
    }
    #[test]
    fn distinct_keys_do_not_alias_and_all_control_rounding_modes_are_admitted() {
        let data=bytes(); let rows=cell_kind4_list_disk_v1::parse(&data).unwrap(); let mut c=Cache::default();
        for (i,cw) in [0x027f,0x067f,0x0a7f,0x0e7f,0x037f,0x077f,0x0b7f,0x0f7f].into_iter().enumerate() {
            assert_eq!(select(&mut c,i as u32,Some(&rows),0x1234_5678,cw,9,SnapshotAdmission),
                ResultV1::Selected { triple:[IDS[3],0x3d97_a8e4,9],selected:3,rng_state:0xa135_45f8,draws:4 });
        }
    }
    #[test]
    fn supplied_hit_never_requires_rows_or_rng_and_cannot_be_overwritten() {
        let mut c=Cache::default(); c.admit_ready(11,[1,2,3],SnapshotAdmission).unwrap();
        assert_eq!(c.admit_ready(11,[4,5,6],SnapshotAdmission),Err(Failure::AlreadyPresent));
        assert_eq!(select(&mut c,11,None,0,0,0,SnapshotAdmission),ResultV1::Hit { triple:[1,2,3],rng_state:0 });
    }
    #[test]
    fn unsupported_miss_keeps_pending_slot_without_fabricating_result() {
        let mut c=Cache::default();
        assert_eq!(select(&mut c,9,None,1,0x027f,0,SnapshotAdmission),ResultV1::Pending { failure:Failure::UnsupportedRows,rng_state:1,draws:0 });
        assert_eq!(c.slot(9),Some(Slot::Pending));
        assert_eq!(select(&mut c,9,None,1,0x027f,0,SnapshotAdmission),ResultV1::Rejected(Failure::ReentrantPending));
    }
    #[test]
    fn unsupported_zero_fraction_consumes_draw_and_marks_pending() {
        let data=bytes(); let rows=cell_kind4_list_disk_v1::parse(&data).unwrap(); let mut c=Cache::default();
        assert_eq!(select(&mut c,3,Some(&rows),0x8000_0000,0x027f,0,SnapshotAdmission),
            ResultV1::Pending { failure:Failure::UnsupportedRng,rng_state:0x8000_0000,draws:1 });
    }
    #[test]
    fn changed_weight_or_control_fails_after_insertion_before_draw() {
        let mut data=bytes(); data[16..20].copy_from_slice(&2f32.to_bits().to_le_bytes());
        let rows=cell_kind4_list_disk_v1::parse(&data).unwrap(); let mut c=Cache::default();
        assert_eq!(select(&mut c,1,Some(&rows),1,0x027f,0,SnapshotAdmission),ResultV1::Pending { failure:Failure::UnsupportedRows,rng_state:1,draws:0 });
        let data=bytes();let rows=cell_kind4_list_disk_v1::parse(&data).unwrap();
        assert_eq!(select(&mut c,2,Some(&rows),1,0x007f,0,SnapshotAdmission),ResultV1::Pending { failure:Failure::UnsupportedControl,rng_state:1,draws:0 });
    }
    #[test]
    fn fourth_row_equal_threshold_does_not_replace_third() {
        let data=bytes(); let rows=cell_kind4_list_disk_v1::parse(&data).unwrap();
        for cw in [0x027f,0x067f,0x0a7f,0x0e7f,0x037f,0x077f,0x0b7f,0x0f7f] {
            let mut c=Cache::default();
            // This seed stays C0000000 under the odd source multiplier, so
            // every draw is 1/4. Row four's product equals weight 1 exactly.
            assert_eq!(select(&mut c,42,Some(&rows),0xc000_0000,cw,7,SnapshotAdmission),
                ResultV1::Selected { triple:[IDS[2],0x3d97_a8e4,7],selected:2,
                                     rng_state:0xc000_0000,draws:4 });
            assert_eq!(c.slot(42),Some(Slot::Ready([IDS[2],0x3d97_a8e4,7])));
        }
    }
    #[test]
    fn full_cache_rejects_new_key_without_mutation_but_allows_hit() {
        let data=bytes(); let rows=cell_kind4_list_disk_v1::parse(&data).unwrap();
        let mut c=Cache::default();
        for i in 0..MAX_CACHE { c.admit_ready(i as u32,[i as u32,2,3],SnapshotAdmission).unwrap(); }
        let before=c.slots.clone();
        assert_eq!(select(&mut c,MAX_CACHE as u32,Some(&rows),1,0x027f,7,SnapshotAdmission),
                   ResultV1::Rejected(Failure::CacheFull));
        assert_eq!(c.slots,before);
        assert_eq!(select(&mut c,0,None,0,0,99,SnapshotAdmission),
                   ResultV1::Hit { triple:[0,2,3],rng_state:0 });
        assert_eq!(c.slots,before);
    }
}
