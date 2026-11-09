//! Bounded spatial lookup and lane dispatch for `4F9630` and `4F9C30`.
//!
//! The grid, origin, scale and lane records are supplied snapshots. This does
//! not build the native grid or establish its relationship to a creature,
//! generated geometry, animation palette or a live editor. The checked model
//! rejects nonfinite inputs, invalid conversions and excessive work; native
//! exception and partial-append behavior is not reproduced.

use crate::{
    creature_generated_skin_kernel_v1::{self as kernel, KernelAdmission, Lane},
    native_sse::{self, Policy, SseError},
};

pub const CELL_COUNT: usize = 512;
pub const MAX_RECORDS_PER_CELL: usize = 4096;

/// Admit stable source tables and record order, valid native memory/append
/// operations, and the assumptions required by the finite lane kernel.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotAdmission;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    pub coordinates: [i32; 3],
    /// Wrapping 32-bit `z + 8*y + 64*x`, tested unsigned by the source.
    pub linear_word: u32,
    pub cell: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Request {
    pub record: usize,
    pub lane: u8,
    pub key: u32,
    pub weight: f32,
}

#[derive(Debug, PartialEq)]
pub struct Query {
    pub address: Address,
    /// False for an out-of-range address or a null supplied table entry.
    pub cell_present: bool,
    pub requests: Vec<Request>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
    NonFiniteIntermediate,
    UnsupportedIntegerConversion,
    TooManyRecords,
    AllocationFailed,
    Kernel(kernel::Error),
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "generated skin spatial query: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Reproduce the finite addressed-cell arithmetic under a declared SSE policy.
/// Axis values are not individually clamped: distinct triples can produce the
/// same wrapping linear word, as in the source's final unsigned comparison.
pub fn address(
    point: [f32; 3],
    origin: [f32; 3],
    scale: [f32; 3],
    policy: Policy,
) -> Result<Address, Error> {
    if point
        .into_iter()
        .chain(origin)
        .chain(scale)
        .any(|v| !v.is_finite())
    {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let mut coordinates = [0i32; 3];
            for i in 0..3 {
                let mut delta = point[i];
                // SAFETY: SSE2 and the masked, scoped MXCSR policy were checked
                // above. These operations retain the native operand order.
                unsafe {
                    std::arch::asm!("subss {dst}, {src}", dst = inout(xmm_reg) delta,
                        src = in(xmm_reg) origin[i], options(nostack, preserves_flags));
                }
                let mut scaled = delta;
                unsafe {
                    std::arch::asm!("mulss {dst}, {src}", dst = inout(xmm_reg) scaled,
                        src = in(xmm_reg) scale[i], options(nostack, preserves_flags));
                }
                if !delta.is_finite() || !scaled.is_finite() {
                    return Err(Error::NonFiniteIntermediate);
                }
                // Values outside the representable interval make CVTTSS2SI
                // produce its indefinite result; this checked model rejects
                // that exceptional case instead of conflating it with MIN.
                if scaled < -2_147_483_648.0 || scaled >= 2_147_483_648.0 {
                    return Err(Error::UnsupportedIntegerConversion);
                }
                coordinates[i] = scaled as i32;
            }
            let [x, y, z] = coordinates.map(|v| v as u32);
            let linear_word = z
                .wrapping_add(y.wrapping_mul(8))
                .wrapping_add(x.wrapping_mul(64));
            Ok(Address {
                coordinates,
                linear_word,
                cell: (linear_word < CELL_COUNT as u32).then_some(linear_word as usize),
            })
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = (point, origin, scale);
            Err(Error::UnsupportedArchitecture)
        }
    })
}

/// Walk the explicitly supplied cell's records in order, then each record's
/// four lanes in order. A null cell and an out-of-range address emit nothing.
/// No partially accumulated result is returned on a checked error.
pub fn query(
    point: [f32; 3],
    origin: [f32; 3],
    scale: [f32; 3],
    cells: &[Option<&[[Lane; 4]]>; CELL_COUNT],
    policy: Policy,
    _admit: SnapshotAdmission,
) -> Result<Query, Error> {
    let address = address(point, origin, scale, policy)?;
    let Some(records) = address.cell.and_then(|index| cells[index]) else {
        return Ok(Query {
            address,
            cell_present: false,
            requests: Vec::new(),
        });
    };
    if records.len() > MAX_RECORDS_PER_CELL {
        return Err(Error::TooManyRecords);
    }
    let mut requests = Vec::new();
    requests
        .try_reserve_exact(records.len() * 4)
        .map_err(|_| Error::AllocationFailed)?;
    for (record, lanes) in records.iter().enumerate() {
        let result =
            kernel::evaluate(point, *lanes, policy, KernelAdmission).map_err(Error::Kernel)?;
        for request in result.entries.into_iter().take(result.len).flatten() {
            requests.push(Request {
                record,
                lane: request.lane,
                key: request.key,
                weight: request.weight,
            });
        }
    }
    Ok(Query {
        address,
        cell_present: true,
        requests,
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn locate(point: [f32; 3]) -> Result<Address, Error> {
        address(point, [0.; 3], [1.; 3], Policy::Mxcsr1f80)
    }
    #[test]
    fn unsigned_linear_test_preserves_aliases_and_wrapping() {
        assert_eq!(locate([1., 2., 3.]).unwrap().cell, Some(83));
        assert_eq!(locate([-0.75, 0., 0.]).unwrap().coordinates, [0, 0, 0]);
        assert_eq!(locate([-1., 8., 0.]).unwrap().cell, Some(0));
        assert_eq!(locate([67_108_864., 0., 0.]).unwrap().cell, Some(0));
        assert_eq!(locate([-1., 0., 0.]).unwrap().cell, None);
        assert_eq!(locate([8., 0., 0.]).unwrap().cell, None);
    }
    #[test]
    fn finite_policy_rejects_bad_inputs_and_conversion_overflow() {
        assert_eq!(locate([f32::NAN, 0., 0.]), Err(Error::NonFiniteInput));
        assert_eq!(
            address(
                [f32::MAX, 0., 0.],
                [-f32::MAX, 0., 0.],
                [1.; 3],
                Policy::Mxcsr1f80
            ),
            Err(Error::NonFiniteIntermediate)
        );
        assert_eq!(
            locate([2_147_483_648., 0., 0.]),
            Err(Error::UnsupportedIntegerConversion)
        );
        assert_eq!(
            locate([-2_147_483_648., 0., 0.]).unwrap().coordinates[0],
            i32::MIN
        );
    }
    #[test]
    fn ordered_records_and_lanes_are_dispatched_from_selected_cell() {
        let lane = |key| Lane {
            center: [0.; 3],
            cutoff: 1.,
            scale: 0.,
            coefficient: 1.,
            key,
        };
        let mut records = [
            [lane(10), lane(11), lane(12), lane(13)],
            [lane(20), lane(21), lane(22), lane(23)],
        ];
        records[0][1].cutoff = 0.;
        records[1][2].coefficient = 0.;
        let mut cells: [Option<&[[Lane; 4]]>; CELL_COUNT] = [None; CELL_COUNT];
        cells[0] = Some(records.as_slice());
        let out = query(
            [0.; 3],
            [0.; 3],
            [1.; 3],
            &cells,
            Policy::Mxcsr1f80,
            SnapshotAdmission,
        )
        .unwrap();
        assert!(out.cell_present);
        assert_eq!(
            out.requests
                .iter()
                .map(|r| (r.record, r.lane, r.key))
                .collect::<Vec<_>>(),
            [
                (0, 0, 10),
                (0, 2, 12),
                (0, 3, 13),
                (1, 0, 20),
                (1, 1, 21),
                (1, 3, 23)
            ]
        );
        assert!(out.requests.iter().all(|r| r.weight == 1.));
        assert!(
            !query(
                [0., 0., 1.],
                [0.; 3],
                [1.; 3],
                &cells,
                Policy::Mxcsr1f80,
                SnapshotAdmission
            )
            .unwrap()
            .cell_present
        );
    }

    #[test]
    fn rejects_excess_work_and_late_invalid_record() {
        let lane = Lane {
            center: [0.; 3],
            cutoff: 1.,
            scale: 0.,
            coefficient: 1.,
            key: 0,
        };
        let too_many = vec![[lane; 4]; MAX_RECORDS_PER_CELL + 1];
        let mut cells: [Option<&[[Lane; 4]]>; CELL_COUNT] = [None; CELL_COUNT];
        cells[0] = Some(&too_many);
        assert_eq!(
            query(
                [0.; 3],
                [0.; 3],
                [1.; 3],
                &cells,
                Policy::Mxcsr1f80,
                SnapshotAdmission
            ),
            Err(Error::TooManyRecords)
        );
        let mut two = [[lane; 4]; 2];
        two[1][0].coefficient = f32::NAN;
        cells[0] = Some(&two);
        assert_eq!(
            query(
                [0.; 3],
                [0.; 3],
                [1.; 3],
                &cells,
                Policy::Mxcsr1f80,
                SnapshotAdmission
            ),
            Err(Error::Kernel(kernel::Error::NonFiniteInput))
        );
    }
}
