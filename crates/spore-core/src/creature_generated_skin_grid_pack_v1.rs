//! Checked packing of explicitly supplied per-cell candidate indices.
//! Candidate selection, grid bounds and lane production remain caller inputs.
use crate::creature_generated_skin_grid_query_v1::{CELL_COUNT, MAX_RECORDS_PER_CELL};
use crate::creature_generated_skin_kernel_v1::Lane;

/// Checked model cap across all cells, not a native limit.
pub const MAX_TOTAL_RECORDS: usize = 65_536;

/// Admit masked x87 exceptions and adequate x87 stack for native lane copies.
#[derive(Clone, Copy, Debug)]
pub struct CandidateAdmission;

#[derive(Debug, PartialEq, Eq)]
pub enum Error { InvalidIndex, NonFiniteLane, TooManyRecords, AllocationFailed }

pub struct PackedGrid { cells: [Vec<[Lane; 4]>; CELL_COUNT] }
impl PackedGrid {
    /// References for the existing query and one-vertex fixture. Empty cells
    /// are null, matching the zero-candidate branch of the traced builder.
    pub fn cells(&self) -> [Option<&[[Lane; 4]]>; CELL_COUNT] {
        std::array::from_fn(|i| (!self.cells[i].is_empty()).then_some(self.cells[i].as_slice()))
    }
}

/// Packs each supplied candidate sequence in its given order. Candidate values
/// index the supplied *active* lane array. Padding lanes are zero, including
/// cutoff, so they cannot emit a finite query. Source allocation and filtering
/// are admitted; this function does not infer candidate lists from geometry.
pub fn pack(lanes: &[Lane], candidates: &[Vec<usize>; CELL_COUNT], _: CandidateAdmission) -> Result<PackedGrid, Error> {
    // Validate the complete snapshot and total allocation before building cells.
    let mut total = 0usize;
    for indices in candidates {
        let count = indices.len().checked_add(3).ok_or(Error::TooManyRecords)? / 4;
        if count > MAX_RECORDS_PER_CELL { return Err(Error::TooManyRecords); }
        total = total.checked_add(count).ok_or(Error::TooManyRecords)?;
        if total > MAX_TOTAL_RECORDS { return Err(Error::TooManyRecords); }
        if indices.iter().any(|&i| i >= lanes.len()) { return Err(Error::InvalidIndex); }
        if indices.iter().any(|&i| {
            let lane = &lanes[i];
            !lane.center.into_iter().chain([lane.cutoff, lane.scale, lane.coefficient]).all(f32::is_finite)
        }) { return Err(Error::NonFiniteLane); }
    }
    let blank = Lane { center: [0.; 3], cutoff: 0., scale: 0., coefficient: 0., key: 0 };
    let mut cells: [Vec<[Lane; 4]>; CELL_COUNT] = std::array::from_fn(|_| Vec::new());
    for (out, indices) in cells.iter_mut().zip(candidates) {
        let count = indices.len().checked_add(3).ok_or(Error::TooManyRecords)? / 4;
        if count > MAX_RECORDS_PER_CELL { return Err(Error::TooManyRecords); }
        out.try_reserve_exact(count).map_err(|_| Error::AllocationFailed)?;
        for chunk in indices.chunks(4) {
            let mut record = [blank; 4];
            for (slot, &index) in chunk.iter().enumerate() {
                record[slot] = *lanes.get(index).ok_or(Error::InvalidIndex)?;
            }
            out.push(record);
        }
    }
    Ok(PackedGrid { cells })
}


/// Geometry supplied for the 512 cells in native x-major, z-minor order.
/// The half extents and centers must come from the same prepared grid snapshot.
#[derive(Clone, Copy, Debug)]
pub struct CellGeometry { pub center: [f32; 3], pub half_extent: [f32; 3] }

#[derive(Debug, PartialEq, Eq)]
pub enum FilterError { NonFiniteInput, NonFiniteIntermediate, TooManyLanes, AllocationFailed, UnsupportedArchitecture, Packing(Error) }
impl From<crate::native_sse::SseError> for FilterError {
    fn from(_: crate::native_sse::SseError) -> Self { Self::UnsupportedArchitecture }
}

/// Filter explicitly supplied prepared lanes against supplied cell boxes, then
/// pack the resulting ordered indices. This models 4FAC60's finite arithmetic;
/// the caller admits native padding as inactive and consistent prepared data.
/// The source visits records backwards, with lanes 3,2,1,0 in each record.
pub fn filter_and_pack(lanes: &[Lane], cells: &[CellGeometry; CELL_COUNT], threshold: f32,
    policy: crate::native_sse::Policy, admit: CandidateAdmission) -> Result<PackedGrid, FilterError> {
    if lanes.len() > MAX_RECORDS_PER_CELL * 4 { return Err(FilterError::TooManyLanes); }
    if !threshold.is_finite() || cells.iter().any(|c| c.center.into_iter().chain(c.half_extent).any(|x| !x.is_finite())) ||
        lanes.iter().any(|l| l.center.into_iter().chain([l.cutoff,l.scale,l.coefficient]).any(|x| !x.is_finite())) {
        return Err(FilterError::NonFiniteInput);
    }
    let candidates = crate::native_sse::scoped(policy, || {
        let mut lists: [Vec<usize>; CELL_COUNT] = std::array::from_fn(|_| Vec::new());
        for (cell, out) in cells.iter().zip(&mut lists) {
            out.try_reserve_exact(lanes.len()).map_err(|_| FilterError::AllocationFailed)?;
            let mut sums = [0f32; 4];
            for i in (0..lanes.len()).rev() {
                let l = lanes[i];
                let mut d = [0f32; 3];
                for axis in 0..3 {
                    let delta = l.center[axis] - cell.center[axis];
                    let offset = delta.abs() - cell.half_extent[axis];
                    d[axis] = offset.max(0.0);
                    if !delta.is_finite() || !offset.is_finite() { return Err(FilterError::NonFiniteIntermediate); }
                }
                let xx = d[0] * d[0]; let yy = d[1] * d[1]; let zz = d[2] * d[2];
                let xy = xx + yy; let distance = xy + zz;
                let t = distance * l.scale - 1.0;
                let q = t * t; let fourth = q * q; let raw = fourth * l.coefficient;
                if [xx,yy,zz,xy,distance,t,q,fourth,raw].into_iter().any(|x| !x.is_finite()) { return Err(FilterError::NonFiniteIntermediate); }
                let weight = if distance < l.cutoff { raw } else { 0.0 };
                sums[i % 4] += weight;
                if !sums[i % 4].is_finite() { return Err(FilterError::NonFiniteIntermediate); }
                if weight > 0.0 { out.push(i); }
            }
            let sum = (sums[0] + sums[2]) + (sums[1] + sums[3]);
            if !sum.is_finite() { return Err(FilterError::NonFiniteIntermediate); }
            if sum < threshold { out.clear(); }
        }
        Ok(lists)
    })?;
    pack(lanes, &candidates, admit).map_err(FilterError::Packing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature_generated_skin_grid_query_v1::{query, SnapshotAdmission};
    use crate::native_sse::Policy;
    #[test]
    fn source_filter_boxes_reverse_order_and_threshold_feed_query() {
        use crate::native_sse::Policy;
        let lanes: Vec<_> = (0..5).map(|key| Lane { center: [key as f32,0.,0.], cutoff: 1., scale: 0., coefficient: 1., key }).collect();
        let box0 = CellGeometry { center: [2.,0.,0.], half_extent: [2.,0.,0.] };
        let mut boxes = [box0; CELL_COUNT];
        boxes[1].center = [100.,0.,0.];
        let packed = filter_and_pack(&lanes, &boxes, 5., Policy::Mxcsr1f80, CandidateAdmission).unwrap();
        let cells = packed.cells();
        let out = query([0.;3], [0.;3], [1.;3], &cells, Policy::Mxcsr1f80, SnapshotAdmission).unwrap();
        // Query selects only key 0, because its own cutoff is strict.
        assert_eq!(out.requests.iter().map(|r| r.key).collect::<Vec<_>>(), [0]);
        assert_eq!(cells[0].unwrap().iter().flat_map(|r| r.iter()).take(5).map(|l| l.key).collect::<Vec<_>>(), [4,3,2,1,0]);
        assert!(cells[1].is_none());
        assert!(filter_and_pack(&lanes, &boxes, 5.01, Policy::Mxcsr1f80, CandidateAdmission).unwrap().cells()[0].is_none());
    }
    #[test]
    fn packing_preserves_candidate_order_and_query_ignores_padding() {
        let lanes: Vec<_> = (0..5).map(|key| Lane { center: [0.; 3], cutoff: 1., scale: 0., coefficient: 1., key }).collect();
        let mut candidates: [Vec<usize>; CELL_COUNT] = std::array::from_fn(|_| Vec::new());
        candidates[0] = vec![4, 1, 3, 0, 2];
        let grid = pack(&lanes, &candidates, CandidateAdmission).unwrap();
        let cells = grid.cells();
        assert!(cells[1].is_none());
        let out = query([0.; 3], [0.; 3], [1.; 3], &cells, Policy::Mxcsr1f80, SnapshotAdmission).unwrap();
        assert_eq!(out.requests.iter().map(|r| r.key).collect::<Vec<_>>(), [4, 1, 3, 0, 2]);
        assert_eq!(out.requests.iter().map(|r| (r.record, r.lane)).collect::<Vec<_>>(), [(0,0),(0,1),(0,2),(0,3),(1,0)]);
    }
    #[test]
    fn rejects_excess_total_before_packing() {
        let lane = Lane { center: [0.; 3], cutoff: 0., scale: 0., coefficient: 0., key: 0 };
        let mut candidates: [Vec<usize>; CELL_COUNT] = std::array::from_fn(|_| Vec::new());
        for cell in candidates.iter_mut().take(17) { *cell = vec![0; MAX_RECORDS_PER_CELL * 4]; }
        assert!(matches!(pack(&[lane], &candidates, CandidateAdmission), Err(Error::TooManyRecords)));
    }
    #[test]
    fn rejects_out_of_range_candidates() {
        let mut candidates: [Vec<usize>; CELL_COUNT] = std::array::from_fn(|_| Vec::new());
        candidates[0].push(0);
        assert!(matches!(pack(&[], &candidates, CandidateAdmission), Err(Error::InvalidIndex)));
    }
    #[test]
    fn rejects_nonfinite_selected_lanes() {
        let lane = Lane { center: [f32::NAN, 0., 0.], cutoff: 1., scale: 0., coefficient: 1., key: 0 };
        let mut candidates: [Vec<usize>; CELL_COUNT] = std::array::from_fn(|_| Vec::new());
        candidates[1].push(0);
        assert!(matches!(pack(&[lane], &candidates, CandidateAdmission), Err(Error::NonFiniteLane)));
    }
}
