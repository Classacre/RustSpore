//! Bounded high-bit branch of 52B700 query-to-skin accumulation.
//!
//! All query records, member lists, initial accumulation words and optional
//! remap are caller-supplied finite snapshots. This does not run the spatial
//! query or establish any row, palette, mesh or live-editor identity. Bounds,
//! finite checks and transactional errors are stricter than native behavior;
//! FP sticky state and partial writes are not reproduced.
//!
//! x86_64 only: isolated FXSAVE64/FXRSTOR64 uses abridged x87 tags; older AMD64
//! CPUs may change diagnostic FIP/FDP/FOP fields when ES was clear. Original
//! native execution additionally requires a valid x87 stack and masked traps.
use crate::{
    creature_generated_skin_ranking_v1::{self as ranking, SnapshotAdmission},
    native_sse::{self, Policy, SseError},
    native_x87::Context,
};

/// Explicit model capacity bounds, not native limits.
pub const MAX_WORDS: usize = 4096;
pub const MAX_VISITS: usize = 1_000_000;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Query {
    pub key: u32,
    pub weight: f32,
}
/// Caller admits stable nonaliasing source snapshots and that the supplied
/// initial accumulator is the intended state. Native 52B700 normally prepares
/// it to +0; nonzero initial values are controlled partial-state experiments.
#[derive(Clone, Copy, Debug)]
pub struct AccumulationAdmission;
/// Caller additionally admits the source's argument-10 correction and
/// argument-9 smoothing branches are not taken; no actual runtime state follows.
#[derive(Clone, Copy, Debug)]
pub struct NoCorrectionOrSmoothing;
#[derive(Clone, Debug, PartialEq)]
pub struct Composed {
    pub weights: Vec<f32>,
    pub ranked: [crate::creature_generated_skin_record_v1::Candidate; 4],
    pub record: [u8; 20],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooManyWords,
    TooManyVisits,
    NonFiniteInput,
    ClearHighBit,
    MissingEntry,
    EmptyMembers,
    MissingRemap,
    MissingAccumulator,
    NonFiniteIntermediate,
    UnsupportedArchitecture,
    Ranking(ranking::Error),
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "high-bit skin accumulation: {self:?}")
    }
}
impl std::error::Error for Error {}
#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
struct Environment([u8; 512]);
#[cfg(target_arch = "x86_64")]
fn quotient(weight: f32, count: usize, context: Context) -> Result<f32, Error> {
    let mut env = Environment([0; 512]);
    let control = context.control_word();
    let count = count as i64;
    let mut out = 0f32;
    let mut sw = 0u16;
    // Same FILD qword / FDIVR m32 / FSTP32 sequence as 52BA11..1D.
    // SAFETY: fixed aligned owned memory, nonzero bounded count, isolated stack,
    // and x87 state restored within the single asm block; all ST registers clobbered.
    unsafe {
        std::arch::asm!("fxsave64 [{env}]","fninit","fldcw [{control}]","fild qword ptr [{count}]","fdivr dword ptr [{weight}]","fstp dword ptr [{out}]","fnstsw word ptr [{sw}]","fxrstor64 [{env}]",env=in(reg) &mut env,control=in(reg) &control,count=in(reg) &count,weight=in(reg) &weight,out=in(reg) &mut out,sw=in(reg) &mut sw,out("st(0)") _,out("st(1)") _,out("st(2)") _,out("st(3)") _,out("st(4)") _,out("st(5)") _,out("st(6)") _,out("st(7)") _,options(nostack));
    }
    if sw & 0x004d != 0 || !out.is_finite() {
        return Err(Error::NonFiniteIntermediate);
    }
    Ok(out)
}
#[cfg(target_arch = "x86_64")]
fn add(mut old: f32, share: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {old}, {share}",old=inout(xmm_reg) old,share=in(xmm_reg) share,options(nostack,preserves_flags));
    }
    old
}

/// Apply ordered high-bit records, including repeated members and collisions.
/// After optional remapping the native unsigned >=64 values are skipped; below
/// 64 must still fit the supplied accumulator. No source data is mutated.
pub fn accumulate(
    initial: &[f32],
    queries: &[Query],
    member_lists: &[&[u32]],
    remap: Option<&[u32]>,
    policy: Policy,
    context: Context,
    _admit: AccumulationAdmission,
) -> Result<Vec<f32>, Error> {
    if initial.len() > MAX_WORDS
        || queries.len() > MAX_WORDS
        || member_lists.len() > MAX_WORDS
        || remap.is_some_and(|r| r.len() > MAX_WORDS)
    {
        return Err(Error::TooManyWords);
    }
    if initial.iter().any(|x| !x.is_finite()) || queries.iter().any(|x| !x.weight.is_finite()) {
        return Err(Error::NonFiniteInput);
    }
    let mut visits = 0usize;
    for q in queries {
        if q.key & 0x80000000 == 0 {
            return Err(Error::ClearHighBit);
        }
        let list = member_lists
            .get((q.key & 0x7fffffff) as usize)
            .ok_or(Error::MissingEntry)?;
        if list.is_empty() {
            return Err(Error::EmptyMembers);
        }
        if list.len() > MAX_WORDS {
            return Err(Error::TooManyWords);
        }
        visits = visits.checked_add(list.len()).ok_or(Error::TooManyVisits)?;
        if visits > MAX_VISITS {
            return Err(Error::TooManyVisits);
        }
        for &member in *list {
            let mapped = match remap {
                Some(map) => *map.get(member as usize).ok_or(Error::MissingRemap)?,
                None => member,
            };
            if mapped < 64 && mapped as usize >= initial.len() {
                return Err(Error::MissingAccumulator);
            }
        }
    }
    native_sse::scoped(policy, || {
        #[cfg(target_arch = "x86_64")]
        {
            let mut result = initial.to_vec();
            for q in queries {
                let list = member_lists[(q.key & 0x7fffffff) as usize];
                let share = quotient(q.weight, list.len(), context)?;
                for &member in list {
                    let mapped = remap.map_or(member, |m| m[member as usize]);
                    if mapped < 64 {
                        let index = mapped as usize;
                        let next = add(result[index], share);
                        if !next.is_finite() {
                            return Err(Error::NonFiniteIntermediate);
                        }
                        result[index] = next;
                    }
                }
            }
            Ok(result)
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (initial, queries, member_lists, remap, context);
            Err(Error::UnsupportedArchitecture)
        }
    })
}
/// Explicitly controlled composition into the existing ranking and writer.
pub fn accumulate_rank_record(
    initial: &[f32],
    queries: &[Query],
    member_lists: &[&[u32]],
    remap: Option<&[u32]>,
    policy: Policy,
    context: Context,
    admit: AccumulationAdmission,
    _exclude: NoCorrectionOrSmoothing,
) -> Result<Composed, Error> {
    let weights = accumulate(
        initial,
        queries,
        member_lists,
        remap,
        policy,
        context,
        admit,
    )?;
    let ranked = ranking::rank(&weights, policy, SnapshotAdmission).map_err(Error::Ranking)?;
    let record =
        ranking::rank_record(&weights, policy, SnapshotAdmission).map_err(Error::Ranking)?;
    Ok(Composed {
        weights,
        ranked,
        record,
    })
}
#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn ctx() -> Context {
        Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        }
    }
    fn run(
        initial: &[f32],
        q: &[Query],
        lists: &[&[u32]],
        map: Option<&[u32]>,
    ) -> Result<Vec<f32>, Error> {
        accumulate(
            initial,
            q,
            lists,
            map,
            Policy::Mxcsr1f80,
            ctx(),
            AccumulationAdmission,
        )
    }
    #[test]
    fn ordered_duplicate_remap_and_skip() {
        let lists: [&[u32]; 2] = [&[0, 1, 1, 2], &[3]];
        let map = [2, 2, 64, 1];
        let q = [
            Query {
                key: 0x80000000,
                weight: 4.,
            },
            Query {
                key: 0x80000001,
                weight: 2.,
            },
        ];
        let out = run(&[0.; 4], &q, &lists, Some(&map)).unwrap();
        assert_eq!(out, [0., 2., 3., 0.]);
    }
    #[test]
    fn x87_division_and_ranking() {
        let lists: [&[u32]; 1] = [&[0, 1, 1]];
        let q = [Query {
            key: 0x80000000,
            weight: 1.,
        }];
        let out = accumulate_rank_record(
            &[0.; 3],
            &q,
            &lists,
            None,
            Policy::Mxcsr1f80,
            ctx(),
            AccumulationAdmission,
            NoCorrectionOrSmoothing,
        )
        .unwrap();
        assert_eq!(out.weights[0].to_bits(), 0x3eaaaaab);
        assert_eq!(out.weights[1].to_bits(), 0x3f2aaaab);
        assert_eq!(out.ranked[0].index_word, 1);
        assert_eq!(&out.record[..4], &[1, 0, 0, 0]);
    }
    #[test]
    fn rejection_boundaries() {
        let empty: [&[u32]; 1] = [&[]];
        let q = [Query {
            key: 0x80000000,
            weight: 1.,
        }];
        assert_eq!(run(&[0.], &q, &empty, None), Err(Error::EmptyMembers));
        assert_eq!(
            run(&[0.], &[Query { key: 0, weight: 1. }], &empty, None),
            Err(Error::ClearHighBit)
        );
        let lists: [&[u32]; 1] = [&[1]];
        assert_eq!(run(&[0.], &q, &lists, None), Err(Error::MissingAccumulator));
        assert_eq!(run(&[0.], &q, &lists, Some(&[0])), Err(Error::MissingRemap));
        assert_eq!(
            run(
                &[0.],
                &[Query {
                    key: 0x80000000,
                    weight: f32::NAN
                }],
                &lists,
                None
            ),
            Err(Error::NonFiniteInput)
        );
    }
    #[test]
    fn signed_zero_and_overflow_are_transactional() {
        let lists: [&[u32]; 1] = [&[0]];
        let q = [Query {
            key: 0x80000000,
            weight: -0.,
        }];
        let r = run(&[-0.], &q, &lists, None).unwrap();
        assert_eq!(r[0].to_bits(), 0x80000000);
        let q = [Query {
            key: 0x80000000,
            weight: f32::MAX,
        }];
        assert_eq!(
            run(&[f32::MAX], &q, &lists, None),
            Err(Error::NonFiniteIntermediate)
        );
    }
}
#[cfg(all(test, not(target_arch = "x86_64")))]
#[test]
fn unsupported_architecture() {
    let c = Context {
        precision: crate::native_x87::Precision::Bits53,
        rounding: crate::native_x87::Rounding::NearestEven,
    };
    assert_eq!(
        accumulate(
            &[0.],
            &[],
            &[],
            None,
            Policy::Mxcsr1f80,
            c,
            AccumulationAdmission
        ),
        Err(Error::UnsupportedArchitecture)
    );
}
