//! Bounded 2D sweep-pair reconstruction from BBAD00/BBA9C0/BBB840.
//!
//! Requires explicitly supplied final endpoint values and slot
//! order. It does not construct actor geometry, update endpoints, model pool
//! lifecycle, or establish runtime Cell reachability. Ties, nonfinite and
//! subnormal endpoints and invalid intervals are rejected. Native comparator
//! results can depend on MXCSR state; no general FP-state parity is claimed.
//! Merge scratch limits, free-list mutations and history-dependent endpoint
//! lifecycle remain unproved; this draft is not a persistent collection.
//! See re/cell-source-health-integration-proposal-v1.md.

/// Bounds reflect the Cell startup's 0x1000 collection capacities and the
/// observed 0x1000-byte local active array. They are safe admission limits,
/// not claims that native code checks them.
pub const MAX_SLOTS: usize = 4096;
pub const MAX_PAIRS: usize = 4096;
pub const MAX_ACTIVE: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub struct SweepSlot {
    pub id: i32,
    /// Exact raw tag word; equal nonzero tags reject a pair.
    pub tag: u32,
    pub min: [f32; 2],
    pub max: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepError {
    TooManySlots,
    PairCapacityTooLarge,
    NonFiniteEndpoint,
    SubnormalEndpoint,
    InvalidInterval,
    EndpointTie,
    ActiveCapacityExceeded,
    InternalEndpointMismatch,
}

#[derive(Clone, Copy)]
struct Endpoint { value: f32, slot: usize, opening: bool }

/// Stored order corresponds to BBAD00's append order. `dispatch_order` walks
/// this array backwards, as BBAB30/BBB840 do when the array stays unchanged.
#[derive(Debug, PartialEq, Eq)]
pub struct SweepPairs {
    stored: Vec<(i32, i32)>,
    /// Number of accepted candidates suppressed when the output is full.
    /// Counts are a diagnostic of this checked model, not a native field.
    pub suppressed: usize,
}

impl SweepPairs {
    pub fn stored_order(&self) -> &[(i32, i32)] { &self.stored }
    pub fn dispatch_order(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.stored.iter().rev().copied()
    }
}

/// Rebuild pairs for the supplied current slot records, modeling the native
/// finite, unique-endpoint order. The explicit `pair_capacity` is the source
/// collection's pair-buffer capacity, not an actor limit. There is no persistent
/// endpoint/free-list state, so this cannot establish native update lifecycle.
/// Any rejection returns no partial collection.
pub fn rebuild(slots: &[SweepSlot], pair_capacity: usize) -> Result<SweepPairs, SweepError> {
    if slots.len() > MAX_SLOTS { return Err(SweepError::TooManySlots); }
    if pair_capacity > MAX_PAIRS { return Err(SweepError::PairCapacityTooLarge); }
    let mut endpoints = [Vec::with_capacity(slots.len() * 2), Vec::with_capacity(slots.len() * 2)];
    for (index, slot) in slots.iter().enumerate() {
        for (axis, array) in endpoints.iter_mut().enumerate() {
            let (min, max) = (slot.min[axis], slot.max[axis]);
            if !min.is_finite() || !max.is_finite() { return Err(SweepError::NonFiniteEndpoint); }
            if min.is_subnormal() || max.is_subnormal() { return Err(SweepError::SubnormalEndpoint); }
            if min >= max { return Err(SweepError::InvalidInterval); }
            array.push(Endpoint { value: min, slot: index, opening: true });
            array.push(Endpoint { value: max, slot: index, opening: false });
        }
    }
    for array in &mut endpoints {
        // BBA700 sorts finite unequal endpoint values ascending. The native
        // comparator has no equality result; reject ties before interpreting it.
        array.sort_unstable_by(|a, b| a.value.total_cmp(&b.value));
        if array.windows(2).any(|pair| pair[0].value == pair[1].value) {
            return Err(SweepError::EndpointTie);
        }
    }
    let mut active = Vec::<usize>::new();
    let mut stored = Vec::new();
    let mut suppressed = 0usize;
    for point in &endpoints[0] {
        if point.opening {
            let current = &slots[point.slot];
            for &other_index in &active {
                let other = &slots[other_index];
                if current.tag != 0 && current.tag == other.tag { continue; }
                // With unique finite endpoints, rank overlap on axis 1 equals
                // strict interval overlap. Equality is excluded at admission.
                if current.max[1] < other.min[1] || other.max[1] < current.min[1] { continue; }
                if stored.len() == pair_capacity {
                    suppressed += 1;
                } else {
                    stored.push(if current.id < other.id { (current.id, other.id) } else { (other.id, current.id) });
                }
            }
            if active.len() == MAX_ACTIVE { return Err(SweepError::ActiveCapacityExceeded); }
            active.push(point.slot);
        } else {
            let position = active.iter().position(|&slot| slot == point.slot)
                .ok_or(SweepError::InternalEndpointMismatch)?;
            active.swap_remove(position);
        }
    }
    if !active.is_empty() { return Err(SweepError::InternalEndpointMismatch); }
    Ok(SweepPairs { stored, suppressed })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slot(id: i32, tag: u32, x0: f32, x1: f32, y0: f32, y1: f32) -> SweepSlot {
        SweepSlot { id, tag, min: [x0, y0], max: [x1, y1] }
    }
    fn fixtures() -> Vec<SweepSlot> {
        vec![slot(5,0,0.,3.,0.,20.), slot(-1,0,1.,10.,1.,21.),
             slot(3,0,2.,9.,2.,22.), slot(2,0,4.,8.,3.,23.)]
    }
    #[test]
    fn signed_pair_order_reverse_dispatch_and_swap_last() {
        let result = rebuild(&fixtures(), 10).unwrap();
        assert_eq!(result.stored_order(), &[(-1,5),(3,5),(-1,3),(2,3),(-1,2)]);
        assert_eq!(result.dispatch_order().collect::<Vec<_>>(), vec![(-1,2),(2,3),(-1,3),(3,5),(-1,5)]);
    }
    #[test]
    fn nonzero_equal_tags_reject_and_zero_does_not() {
        let mut slots = fixtures();
        slots[0].tag=7; slots[1].tag=7;
        assert_eq!(rebuild(&slots,10).unwrap().stored_order(), &[(3,5),(-1,3),(2,3),(-1,2)]);
        slots[0].tag=0; slots[1].tag=0;
        assert_eq!(rebuild(&slots,10).unwrap().stored_order().len(),5);
    }
    #[test]
    fn capacity_preserves_first_appends_and_counts_suppression() {
        let result = rebuild(&fixtures(),2).unwrap();
        assert_eq!(result.stored_order(), &[(-1,5),(3,5)]);
        assert_eq!(result.dispatch_order().collect::<Vec<_>>(), vec![(3,5),(-1,5)]);
        assert_eq!(result.suppressed,3);
        assert_eq!(rebuild(&fixtures(),0).unwrap().suppressed,5);
    }
    #[test]
    fn other_axis_separation_and_unsafe_inputs() {
        let mut slots = fixtures();
        slots[1].min[1]=30.; slots[1].max[1]=31.;
        assert!(!rebuild(&slots,10).unwrap().stored_order().iter().any(|p| p.0 == -1 || p.1 == -1));
        slots[1].min[0]=slots[0].min[0];
        assert_eq!(rebuild(&slots,10), Err(SweepError::EndpointTie));
        slots[1].min[0]=f32::NAN;
        assert_eq!(rebuild(&slots,10), Err(SweepError::NonFiniteEndpoint));
        slots[1].min[0]=11.;
        assert_eq!(rebuild(&slots,10), Err(SweepError::InvalidInterval));
    }
    #[test]
    fn active_limit_and_pair_limit_fail_closed() {
        let slots = (0..=MAX_ACTIVE).map(|i| slot(i as i32, 0, i as f32, 5000.+i as f32,
                                                       i as f32, 6000.+i as f32)).collect::<Vec<_>>();
        assert_eq!(rebuild(&slots,0), Err(SweepError::ActiveCapacityExceeded));
        assert_eq!(rebuild(&[], MAX_PAIRS+1), Err(SweepError::PairCapacityTooLarge));
    }

    #[test]
    fn positive_and_negative_subnormal_endpoints_are_rejected() {
        let mut slots = fixtures();
        slots[0].min[0] = f32::from_bits(1);
        assert_eq!(rebuild(&slots,10), Err(SweepError::SubnormalEndpoint));
        slots[0].min[0] = f32::from_bits(0x8000_0001);
        assert_eq!(rebuild(&slots,10), Err(SweepError::SubnormalEndpoint));
    }
}
