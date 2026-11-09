//! Checked composition of supplied row, grid and endpoint snapshots for one
//! initial generated-skin record. This is not a surface or baked-model builder.
//!
//! Native row mutation, endpoint-coordinate construction, grid construction,
//! query allocation, correction and smoothing are outside this fixture. A
//! supplied row index is not automatically a live animation-palette identity.

use crate::{
    creature_generated_skin_edge_accum_v1 as edge, creature_generated_skin_edges_v1 as edges,
    creature_generated_skin_grid_query_v1 as grid, creature_generated_skin_kernel_v1::Lane,
    creature_generated_skin_mixed_accum_v1 as mixed, creature_generated_skin_remap_v1 as remap,
    native_sse::Policy, native_x87::Context,
};

/// Admit stable post-mutation rows, native-successful preparation and vector
/// operations, a supplied corresponding grid and endpoint coordinates, masked
/// FP exceptions, no positive correction, and nonpositive smoothing iterations.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotAdmission;

pub struct Input<'a> {
    pub rows: &'a [[u8; edges::ROW_SIZE]],
    pub endpoint_positions: &'a [[f32; 3]],
    pub point: [f32; 3],
    pub grid_origin: [f32; 3],
    pub grid_scale: [f32; 3],
    pub grid_cells: &'a [Option<&'a [[Lane; 4]]>; grid::CELL_COUNT],
    pub sse: Policy,
    pub x87: Context,
}

#[derive(Debug, PartialEq)]
pub struct Output {
    pub topology: remap::Composed,
    pub query: grid::Query,
    pub initial_record: mixed::Composed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Topology(remap::Error),
    EndpointCount,
    NonFiniteEndpoint,
    Query(grid::Error),
    Packing(crate::creature_generated_skin_grid_pack_v1::Error),
    Filter(crate::creature_generated_skin_grid_pack_v1::FilterError),
    TooManyQueries,
    AllocationFailed,
    Accumulation(mixed::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "generated skin vertex fixture: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Input for a source-row snapshot with explicitly supplied lanes and per-cell
/// candidate indices. These candidates are not computed from the rows.
pub struct PackedInput<'a> {
    pub rows: &'a [[u8; edges::ROW_SIZE]],
    pub endpoint_positions: &'a [[f32; 3]],
    pub point: [f32; 3],
    pub grid_origin: [f32; 3],
    pub grid_scale: [f32; 3],
    pub lanes: &'a [Lane],
    pub candidates: &'a [Vec<usize>; grid::CELL_COUNT],
    pub sse: Policy,
    pub x87: Context,
}

/// Pack admitted candidate lists and evaluate a single point against the same
/// explicitly supplied row snapshot. No candidate selection is inferred.
pub fn evaluate_packed(input: PackedInput<'_>, admit: SnapshotAdmission) -> Result<Output, Error> {
    use crate::creature_generated_skin_grid_pack_v1::{pack, CandidateAdmission};
    let packed = pack(input.lanes, input.candidates, CandidateAdmission).map_err(Error::Packing)?;
    let cells = packed.cells();
    evaluate(Input {
        rows: input.rows, endpoint_positions: input.endpoint_positions, point: input.point,
        grid_origin: input.grid_origin, grid_scale: input.grid_scale, grid_cells: &cells,
        sse: input.sse, x87: input.x87,
    }, admit)
}

/// Prepared boxes and lanes are explicit; no raw-row-to-geometry identity is inferred.
pub struct FilteredInput<'a> {
    pub rows: &'a [[u8; edges::ROW_SIZE]],
    pub endpoint_positions: &'a [[f32; 3]],
    pub point: [f32; 3],
    pub grid_origin: [f32; 3],
    pub grid_scale: [f32; 3],
    pub lanes: &'a [Lane],
    pub cell_geometry: &'a [crate::creature_generated_skin_grid_pack_v1::CellGeometry; grid::CELL_COUNT],
    pub threshold: f32,
    pub sse: Policy,
    pub x87: Context,
}

pub fn evaluate_filtered(input: FilteredInput<'_>, admit: SnapshotAdmission) -> Result<Output, Error> {
    use crate::creature_generated_skin_grid_pack_v1::{filter_and_pack, CandidateAdmission};
    let packed = filter_and_pack(input.lanes, input.cell_geometry, input.threshold, input.sse, CandidateAdmission).map_err(Error::Filter)?;
    let cells = packed.cells();
    evaluate(Input { rows: input.rows, endpoint_positions: input.endpoint_positions, point: input.point,
        grid_origin: input.grid_origin, grid_scale: input.grid_scale, grid_cells: &cells,
        sse: input.sse, x87: input.x87 }, admit)
}

/// Compose the bounded stages in source order from explicit snapshots. Native
/// accumulation starts with +0 for each edge; the returned record is the
/// initial unsmoothed result. Errors discard all locally accumulated output.
pub fn evaluate(input: Input<'_>, _admit: SnapshotAdmission) -> Result<Output, Error> {
    let topology =
        remap::compose(input.rows, remap::FrozenPreparedAdmission).map_err(Error::Topology)?;
    if input.endpoint_positions.len() != topology.endpoint_count {
        return Err(Error::EndpointCount);
    }
    if input
        .endpoint_positions
        .iter()
        .flatten()
        .any(|v| !v.is_finite())
    {
        return Err(Error::NonFiniteEndpoint);
    }
    let query = grid::query(
        input.point,
        input.grid_origin,
        input.grid_scale,
        input.grid_cells,
        input.sse,
        grid::SnapshotAdmission,
    )
    .map_err(Error::Query)?;
    if query.requests.len() > 4096 {
        return Err(Error::TooManyQueries);
    }
    let mut endpoints = Vec::new();
    endpoints
        .try_reserve_exact(topology.endpoint_count)
        .map_err(|_| Error::AllocationFailed)?;
    for (position, members) in input.endpoint_positions.iter().zip(&topology.members) {
        endpoints.push(edge::Endpoint {
            position: *position,
            members,
        });
    }
    let mut queries = Vec::new();
    queries
        .try_reserve_exact(query.requests.len())
        .map_err(|_| Error::AllocationFailed)?;
    for request in &query.requests {
        queries.push(mixed::Query {
            key: request.key,
            weight: request.weight,
        });
    }
    let mut initial = Vec::new();
    initial
        .try_reserve_exact(topology.edges.len())
        .map_err(|_| Error::AllocationFailed)?;
    initial.resize(topology.edges.len(), 0.0f32);
    let initial_record = mixed::accumulate_rank_record(
        &initial,
        input.point,
        &queries,
        &topology.edges,
        &endpoints,
        Some(&topology.remap.edge_to_row),
        input.sse,
        input.x87,
        mixed::Admission,
    )
    .map_err(Error::Accumulation)?;
    Ok(Output {
        topology,
        query,
        initial_record,
    })
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};

    fn row(class: u8, link: i16, parent: i16) -> [u8; edges::ROW_SIZE] {
        let mut row = [0; edges::ROW_SIZE];
        row[2..4].copy_from_slice(&link.to_le_bytes());
        row[4..6].copy_from_slice(&parent.to_le_bytes());
        row[11] = class;
        row
    }
    fn run(rows: &[[u8; edges::ROW_SIZE]], positions: &[[f32; 3]]) -> Result<Output, Error> {
        let lane = Lane {
            center: [0.; 3],
            cutoff: 1.,
            scale: 0.,
            coefficient: 1.,
            key: 0x8000_0001,
        };
        let mut blank = lane;
        blank.cutoff = 0.;
        let records = [[lane, blank, blank, blank]];
        let mut cells: [Option<&[[Lane; 4]]>; grid::CELL_COUNT] = [None; grid::CELL_COUNT];
        cells[0] = Some(records.as_slice());
        evaluate(
            Input {
                rows,
                endpoint_positions: positions,
                point: [0.; 3],
                grid_origin: [0.; 3],
                grid_scale: [1.; 3],
                grid_cells: &cells,
                sse: Policy::Mxcsr1f80,
                x87: Context {
                    precision: Precision::Bits53,
                    rounding: Rounding::NearestEven,
                },
            },
            SnapshotAdmission,
        )
    }
    #[test]
    fn supplied_grid_reaches_row_remap_and_ranked_record() {
        let positions = [[0.; 3], [1., 0., 0.]];
        let rows = [row(4, 0, 0), row(4, 1, 0)];
        let result = run(&rows, &positions).unwrap();
        assert_eq!(result.topology.remap.edge_to_row, [0, 1]);
        assert_eq!(result.query.requests.len(), 1);
        assert_eq!(result.initial_record.weights, [0., 1.]);
        assert_eq!(&result.initial_record.record[..4], &[1, 0, 0, 0]);
        assert_eq!(&result.initial_record.record[4..8], &1f32.to_le_bytes());
        let rows_with_class_one = [row(4, 0, 0), row(1, 1, 0)];
        let result = run(&rows_with_class_one, &positions).unwrap();
        assert_eq!(result.topology.remap.edge_to_row, [0, 0]);
        assert_eq!(result.initial_record.weights, [1., 0.]);
        assert_eq!(result.initial_record.record[0], 0);
    }
    #[test]
    fn supplied_candidate_pack_reaches_row_accumulation() {
        let rows = [row(4, 0, 0), row(4, 1, 0)];
        let positions = [[0.; 3], [1., 0., 0.]];
        let lane = Lane { center: [0.; 3], cutoff: 1., scale: 0., coefficient: 1., key: 0x8000_0001 };
        let mut candidates: [Vec<usize>; grid::CELL_COUNT] = std::array::from_fn(|_| Vec::new());
        candidates[0].push(0);
        let result = evaluate_packed(PackedInput {
            rows: &rows, endpoint_positions: &positions, point: [0.; 3],
            grid_origin: [0.; 3], grid_scale: [1.; 3], lanes: &[lane], candidates: &candidates,
            sse: Policy::Mxcsr1f80,
            x87: Context { precision: Precision::Bits53, rounding: Rounding::NearestEven },
        }, SnapshotAdmission).unwrap();
        assert_eq!(result.query.requests.len(), 1);
        assert_eq!(result.initial_record.weights, [0., 1.]);
    }

    #[test]
    fn filtered_geometry_reaches_row_accumulation() {
        use crate::creature_generated_skin_grid_pack_v1::CellGeometry;
        let rows = [row(4, 0, 0), row(4, 1, 0)];
        let positions = [[0.; 3], [1., 0., 0.]];
        let lane = Lane { center: [0.; 3], cutoff: 1., scale: 0., coefficient: 1., key: 0x8000_0001 };
        let boxes = [CellGeometry { center: [0.;3], half_extent: [0.;3] }; grid::CELL_COUNT];
        let result = evaluate_filtered(FilteredInput { rows: &rows, endpoint_positions: &positions,
            point: [0.;3], grid_origin: [0.;3], grid_scale: [1.;3], lanes: &[lane],
            cell_geometry: &boxes, threshold: 1., sse: Policy::Mxcsr1f80,
            x87: Context { precision: Precision::Bits53, rounding: Rounding::NearestEven } }, SnapshotAdmission).unwrap();
        assert_eq!(result.query.requests.len(), 1);
        assert_eq!(result.initial_record.weights, [0., 1.]);
    }

    #[test]
    fn rejects_unmatched_or_nonfinite_endpoint_snapshots() {
        let rows = [row(4, 0, 0), row(4, 1, 0)];
        assert_eq!(run(&rows, &[[0.; 3]]), Err(Error::EndpointCount));
        assert_eq!(
            run(&rows, &[[0.; 3], [f32::NAN, 0., 0.]]),
            Err(Error::NonFiniteEndpoint)
        );
    }

    #[test]
    fn mixed_query_records_reach_the_shared_accumulator_in_order() {
        let rows = [row(4, 0, 0), row(4, 1, 0)];
        let positions = [[0.; 3], [2., 0., 0.]];
        let lane = Lane {
            center: [1., 0., 0.],
            cutoff: 1.,
            scale: 0.,
            coefficient: 1.,
            key: 0x8000_0001,
        };
        let mut direct = lane;
        direct.key = 1;
        let mut blank = lane;
        blank.cutoff = 0.;
        let records = [[lane, direct, blank, blank]];
        let mut cells: [Option<&[[Lane; 4]]>; grid::CELL_COUNT] = [None; grid::CELL_COUNT];
        cells[0] = Some(records.as_slice());
        let result = evaluate(
            Input {
                rows: &rows,
                endpoint_positions: &positions,
                point: [1., 0., 0.],
                grid_origin: [1., 0., 0.],
                grid_scale: [1.; 3],
                grid_cells: &cells,
                sse: Policy::Mxcsr1f80,
                x87: Context {
                    precision: Precision::Bits53,
                    rounding: Rounding::NearestEven,
                },
            },
            SnapshotAdmission,
        )
        .unwrap();
        assert_eq!(
            result
                .query
                .requests
                .iter()
                .map(|r| r.key)
                .collect::<Vec<_>>(),
            [0x8000_0001, 1]
        );
        assert_eq!(result.initial_record.weights, [0., 2.]);
        assert_eq!(
            &result.initial_record.record[..8],
            &[1, 0, 0, 0, 0, 0, 128, 63]
        );
    }
}
