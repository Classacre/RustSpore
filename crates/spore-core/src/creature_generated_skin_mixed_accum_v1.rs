//! Ordered synthetic composition of 52B700 query branches and skin ranking.
//!
//! Snapshots, spatial-query order, no correction/smoothing and successful
//! source vector operations are caller admissions, not live mesh evidence.
//! Each branch uses its own stricter finite/bounds checks and restores FP
//! state; native partial writes and accumulated FP status are not reproduced.
use crate::{
    creature_generated_skin_edge_accum_v1 as edge, creature_generated_skin_high_accum_v1 as high,
    creature_generated_skin_ranking_v1 as ranking, creature_generated_skin_record_v1::Candidate,
    native_sse::Policy, native_x87::Context,
};

/// Abstract admission units, not a count of operations, a native limit, or an
/// elapsed-time guarantee. Input and visit caps also bound the fixture.
pub const MAX_TOTAL_WORK: usize = 1_000_000;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Query {
    pub key: u32,
    pub weight: f32,
}
/// Stable nonaliasing snapshots and successful source vector operations;
/// native correction and smoothing branches are excluded. Masked x87 traps
/// and adequate native x87 stack capacity are also admitted.
#[derive(Clone, Copy, Debug)]
pub struct Admission;
#[derive(Clone, Debug, PartialEq)]
pub struct Composed {
    pub weights: Vec<f32>,
    pub ranked: [Candidate; 4],
    pub record: [u8; 20],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooManyWords,
    TooMuchWork,
    High(high::Error),
    Edge(edge::Error),
    Ranking(ranking::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mixed skin accumulation: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Replay each query in supplied order against an owned accumulator, then rank
/// and write a record. Any error discards all intermediate results. Arbitrary
/// initial weights are an admitted partial snapshot; the traced native producer
/// normally initializes its accumulation vector to +0. The admission proxy
/// charges per-query array sizes, selected-member lengths and output length;
/// it is not an exact or conservative count of every scan, copy or comparison.
pub fn accumulate_rank_record(
    initial: &[f32],
    point: [f32; 3],
    queries: &[Query],
    edges: &[[u32; 2]],
    endpoints: &[edge::Endpoint<'_>],
    remap: Option<&[u32]>,
    policy: Policy,
    context: Context,
    _admission: Admission,
) -> Result<Composed, Error> {
    if initial.len() > 4096
        || queries.len() > 4096
        || edges.len() > 4096
        || endpoints.len() > 4096
        || remap.is_some_and(|m| m.len() > 4096)
    {
        return Err(Error::TooManyWords);
    }
    let per_query = initial
        .len()
        .checked_add(endpoints.len())
        .and_then(|n| n.checked_add(4))
        .ok_or(Error::TooMuchWork)?;
    let mut work = queries
        .len()
        .checked_mul(per_query)
        .and_then(|n| initial.len().checked_mul(8).and_then(|r| n.checked_add(r)))
        .ok_or(Error::TooMuchWork)?;
    if work > MAX_TOTAL_WORK {
        return Err(Error::TooMuchWork);
    }
    let lists: Vec<&[u32]> = endpoints.iter().map(|e| e.members).collect();
    let mut weights = initial.to_vec();
    for q in queries {
        let visits = if q.key & 0x8000_0000 != 0 {
            lists
                .get((q.key & 0x7fff_ffff) as usize)
                .ok_or(Error::High(high::Error::MissingEntry))?
                .len()
        } else {
            let pair = edges
                .get(q.key as usize)
                .ok_or(Error::Edge(edge::Error::MissingEdge))?;
            let a = endpoints
                .get(pair[0] as usize)
                .ok_or(Error::Edge(edge::Error::MissingEndpoint))?;
            let b = endpoints
                .get(pair[1] as usize)
                .ok_or(Error::Edge(edge::Error::MissingEndpoint))?;
            a.members.len().max(b.members.len())
        };
        work = work
            .checked_add(visits.checked_mul(2).ok_or(Error::TooMuchWork)?)
            .ok_or(Error::TooMuchWork)?;
        if work > MAX_TOTAL_WORK {
            return Err(Error::TooMuchWork);
        }
        weights = if q.key & 0x8000_0000 != 0 {
            high::accumulate(
                &weights,
                &[high::Query {
                    key: q.key,
                    weight: q.weight,
                }],
                &lists,
                remap,
                policy,
                context,
                high::AccumulationAdmission,
            )
            .map_err(Error::High)?
        } else {
            edge::accumulate(
                &weights,
                point,
                &[edge::Query {
                    key: q.key,
                    weight: q.weight,
                }],
                edges,
                endpoints,
                remap,
                policy,
                context,
                edge::SnapshotAdmission,
            )
            .map_err(Error::Edge)?
        };
    }
    let ranked =
        ranking::rank(&weights, policy, ranking::SnapshotAdmission).map_err(Error::Ranking)?;
    let record = ranking::rank_record(&weights, policy, ranking::SnapshotAdmission)
        .map_err(Error::Ranking)?;
    Ok(Composed {
        weights,
        ranked,
        record,
    })
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    fn run(
        initial: &[f32],
        queries: &[Query],
        remap: Option<&[u32]>,
        endpoints: &[edge::Endpoint<'_>],
    ) -> Result<Composed, Error> {
        accumulate_rank_record(
            initial,
            [1., 0., 0.],
            queries,
            &[[0, 1]],
            endpoints,
            remap,
            Policy::Mxcsr1f80,
            Context::from_control_word(0x027f).unwrap(),
            Admission,
        )
    }
    #[test]
    fn interleaving_preserves_rounding_order() {
        let members = [0];
        let endpoints = [
            edge::Endpoint {
                position: [0., 0., 0.],
                members: &members,
            },
            edge::Endpoint {
                position: [2., 0., 0.],
                members: &members,
            },
        ];
        let c = Query { key: 0, weight: 1. };
        let h = Query {
            key: 0x8000_0000,
            weight: 1.,
        };
        let big = Query {
            key: 0x8000_0000,
            weight: 16777216.,
        };
        let initial = [0.];
        let ordered = run(&initial, &[c, h, big], None, &endpoints).unwrap();
        let regrouped = run(&initial, &[h, big, c], None, &endpoints).unwrap();
        assert_eq!(ordered.weights[0].to_bits(), 0x4b800001);
        assert_eq!(regrouped.weights[0].to_bits(), 0x4b800000);
        assert_eq!(initial, [0.]);
        assert_eq!(ordered.ranked[0].index_word, 0);
    }
    #[test]
    fn direct_edge_remap_can_exceed_member_gate() {
        let members = [1];
        let endpoints = [
            edge::Endpoint {
                position: [0., 0., 0.],
                members: &members,
            },
            edge::Endpoint {
                position: [2., 0., 0.],
                members: &members,
            },
        ];
        let result = run(
            &[0.; 71],
            &[
                Query { key: 0, weight: 1. },
                Query {
                    key: 0x8000_0000,
                    weight: 2.,
                },
            ],
            Some(&[70, 65]),
            &endpoints,
        )
        .unwrap();
        assert_eq!(result.weights[70], 1.);
        assert_eq!(result.ranked[0].index_word, 70);
        assert_eq!(result.record[0], 70);
    }
    #[test]
    fn errors_leave_input_unchanged_and_bound_work() {
        let members = [0];
        let endpoints = [
            edge::Endpoint {
                position: [0., 0., 0.],
                members: &members,
            },
            edge::Endpoint {
                position: [2., 0., 0.],
                members: &members,
            },
        ];
        let initial = [0.];
        assert!(matches!(
            run(
                &initial,
                &[Query { key: 0, weight: 1. }, Query { key: 3, weight: 1. }],
                None,
                &endpoints
            ),
            Err(Error::Edge(edge::Error::MissingEdge))
        ));
        assert_eq!(initial, [0.]);
        let large = vec![0.; 2000];
        let queries = vec![Query { key: 0, weight: 1. }; 1000];
        assert_eq!(
            run(&large, &queries, None, &endpoints),
            Err(Error::TooMuchWork)
        );
    }
}
