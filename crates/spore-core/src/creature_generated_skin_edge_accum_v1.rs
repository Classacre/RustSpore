//! Bounded clear-high-bit 52B700 edge projection and accumulation fixture.
//!
//! Supplied points, edge/endpoint snapshots, weights and remap are not a spatial
//! query or proof of source-row/palette identity. Finite/bounds/zero-denominator
//! checks and transactional errors are stricter than native partial writes.
//! Source FP status is excluded. x86_64 only: FXSAVE64 uses abridged tags; older
//! AMD64 CPUs may change diagnostic x87 FIP/FDP/FOP fields when ES was clear.
use crate::{
    native_sse::{self, Policy, SseError},
    native_x87::Context,
};
pub const MAX_WORDS: usize = 4096;
pub const MAX_VISITS: usize = 1_000_000;
#[derive(Clone, Copy, Debug)]
pub struct Endpoint<'a> {
    pub position: [f32; 3],
    pub members: &'a [u32],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Query {
    pub key: u32,
    pub weight: f32,
}
/// Admit stable, valid, nonaliasing snapshot storage, masked x87 exceptions and
/// sufficient native x87 stack. All coordinates are already in the source space.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotAdmission;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooManyWords,
    TooManyVisits,
    NonFiniteInput,
    HighBitKey,
    MissingEdge,
    MissingEndpoint,
    EmptyMembers,
    MissingRemap,
    MissingAccumulator,
    ZeroLengthEdge,
    NonFiniteIntermediate,
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "clear-high-bit skin accumulation: {self:?}")
    }
}
impl std::error::Error for Error {}
#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
struct Environment([u8; 512]);
#[cfg(target_arch = "x86_64")]
fn share(factor: f32, weight: f32, count: usize, context: Context) -> Result<f32, Error> {
    let mut env = Environment([0; 512]);
    let control = context.control_word();
    let count = count as i64;
    let mut out = 0f32;
    let mut sw = 0u16;
    // Source 52BE2D..59: retain x87 intermediates until final binary32 FSTP.
    // SAFETY: owned aligned buffers, nonzero bounded count and isolated x87 stack;
    // caller state is restored in the same block, with ST registers clobbered.
    unsafe {
        std::arch::asm!("fxsave64 [{env}]","fninit","fldcw [{control}]","fld1","fsub dword ptr [{factor}]","fmul dword ptr [{weight}]","fild qword ptr [{count}]","fdivp st(1), st(0)","fstp dword ptr [{out}]","fnstsw word ptr [{sw}]","fxrstor64 [{env}]",env=in(reg)&mut env,control=in(reg)&control,factor=in(reg)&factor,weight=in(reg)&weight,count=in(reg)&count,out=in(reg)&mut out,sw=in(reg)&mut sw,out("st(0)") _,out("st(1)") _,out("st(2)") _,out("st(3)") _,out("st(4)") _,out("st(5)") _,out("st(6)") _,out("st(7)") _,options(nostack));
    }
    if sw & 0x004d != 0 || !out.is_finite() {
        return Err(Error::NonFiniteIntermediate);
    }
    Ok(out)
}
#[cfg(target_arch = "x86_64")]
macro_rules! op{($name:ident,$inst:literal)=>{fn $name(mut a:f32,b:f32)->f32{unsafe{std::arch::asm!($inst,a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nostack,preserves_flags));}a}}}
#[cfg(target_arch = "x86_64")]
op!(add, "addss {a}, {b}");
#[cfg(target_arch = "x86_64")]
op!(sub, "subss {a}, {b}");
#[cfg(target_arch = "x86_64")]
op!(mul, "mulss {a}, {b}");
#[cfg(target_arch = "x86_64")]
op!(div, "divss {a}, {b}");
#[cfg(target_arch = "x86_64")]
op!(max, "maxss {a}, {b}");
#[cfg(target_arch = "x86_64")]
op!(min, "minss {a}, {b}");
#[cfg(target_arch = "x86_64")]
fn map_index(raw: u32, remap: Option<&[u32]>) -> Result<u32, Error> {
    match remap {
        Some(m) => m.get(raw as usize).copied().ok_or(Error::MissingRemap),
        None => Ok(raw),
    }
}

/// Apply ordered clear-high-bit query records to a copied accumulator. The
/// direct edge contribution is checked only against actual accumulator length;
/// member contributions retain the native unsigned <64 gate. Corrections,
/// smoothing and mixed high-bit records are not executed by this function.
pub fn accumulate(
    initial: &[f32],
    point: [f32; 3],
    queries: &[Query],
    edges: &[[u32; 2]],
    endpoints: &[Endpoint<'_>],
    remap: Option<&[u32]>,
    policy: Policy,
    context: Context,
    _admission: SnapshotAdmission,
) -> Result<Vec<f32>, Error> {
    if initial.len() > MAX_WORDS
        || queries.len() > MAX_WORDS
        || edges.len() > MAX_WORDS
        || endpoints.len() > MAX_WORDS
        || remap.is_some_and(|m| m.len() > MAX_WORDS)
    {
        return Err(Error::TooManyWords);
    }
    if initial.iter().any(|v| !v.is_finite())
        || !point.into_iter().all(f32::is_finite)
        || queries.iter().any(|q| !q.weight.is_finite())
        || endpoints
            .iter()
            .any(|e| !e.position.into_iter().all(f32::is_finite))
    {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy, || {
        #[cfg(target_arch = "x86_64")]
        {
            let mut result = initial.to_vec();
            let mut visits = 0usize;
            for q in queries {
                if q.key & 0x80000000 != 0 {
                    return Err(Error::HighBitKey);
                }
                let edge = *edges.get(q.key as usize).ok_or(Error::MissingEdge)?;
                let a = endpoints
                    .get(edge[0] as usize)
                    .ok_or(Error::MissingEndpoint)?;
                let b = endpoints
                    .get(edge[1] as usize)
                    .ok_or(Error::MissingEndpoint)?;
                let d = [
                    sub(b.position[0], a.position[0]),
                    sub(b.position[1], a.position[1]),
                    sub(b.position[2], a.position[2]),
                ];
                let v = [
                    sub(point[0], a.position[0]),
                    sub(point[1], a.position[1]),
                    sub(point[2], a.position[2]),
                ];
                let numerator = add(add(mul(v[0], d[0]), mul(v[1], d[1])), mul(v[2], d[2]));
                let denominator = add(add(mul(d[0], d[0]), mul(d[1], d[1])), mul(d[2], d[2]));
                if !d
                    .into_iter()
                    .chain(v)
                    .chain([numerator, denominator])
                    .all(f32::is_finite)
                {
                    return Err(Error::NonFiniteIntermediate);
                }
                if denominator == 0.0 {
                    return Err(Error::ZeroLengthEdge);
                }
                let t = div(numerator, denominator);
                if !t.is_finite() {
                    return Err(Error::NonFiniteIntermediate);
                }
                // Preserve MAXSS/MINSS destination/source ordering, including signed zero.
                let projection = min(max(0., t), 1.);
                let (endpoint, factor) = if projection > 0.5 {
                    (b, mul(sub(1., projection), 2.))
                } else {
                    (a, mul(projection, 2.))
                };
                let direct = mul(q.weight, factor);
                if !factor.is_finite() || !direct.is_finite() {
                    return Err(Error::NonFiniteIntermediate);
                }
                if direct > 0.0 {
                    let index = map_index(q.key, remap)? as usize;
                    let slot = result.get_mut(index).ok_or(Error::MissingAccumulator)?;
                    let value = add(direct, *slot);
                    if !value.is_finite() {
                        return Err(Error::NonFiniteIntermediate);
                    }
                    *slot = value;
                }
                if endpoint.members.is_empty() {
                    return Err(Error::EmptyMembers);
                }
                if endpoint.members.len() > MAX_WORDS {
                    return Err(Error::TooManyWords);
                }
                let s = share(factor, q.weight, endpoint.members.len(), context)?;
                if s > 0.0 {
                    visits = visits
                        .checked_add(endpoint.members.len())
                        .ok_or(Error::TooManyVisits)?;
                    if visits > MAX_VISITS {
                        return Err(Error::TooManyVisits);
                    }
                    for &member in endpoint.members {
                        let mapped = map_index(member, remap)?;
                        if mapped < 64 {
                            let slot = result
                                .get_mut(mapped as usize)
                                .ok_or(Error::MissingAccumulator)?;
                            let value = add(*slot, s);
                            if !value.is_finite() {
                                return Err(Error::NonFiniteIntermediate);
                            }
                            *slot = value;
                        }
                    }
                }
            }
            Ok(result)
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (initial, point, queries, edges, endpoints, remap, context);
            Err(Error::UnsupportedArchitecture)
        }
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
    fn run(point: [f32; 3], remap: Option<&[u32]>) -> Result<Vec<f32>, Error> {
        let a = Endpoint {
            position: [0.; 3],
            members: &[0, 1],
        };
        let b = Endpoint {
            position: [2., 0., 0.],
            members: &[1, 2],
        };
        accumulate(
            &[0.; 4],
            point,
            &[Query { key: 0, weight: 1. }],
            &[[0, 1]],
            &[a, b],
            remap,
            Policy::Mxcsr1f80,
            ctx(),
            SnapshotAdmission,
        )
    }
    #[test]
    fn projection_endpoints_midpoint_and_remap() {
        assert_eq!(run([0., 0., 0.], None).unwrap(), [0.5, 0.5, 0., 0.]);
        assert_eq!(run([1., 0., 0.], None).unwrap(), [1., 0., 0., 0.]);
        assert_eq!(run([1.5, 0., 0.], None).unwrap(), [0.5, 0.25, 0.25, 0.]);
        assert_eq!(
            run([1.5, 0., 0.], Some(&[3, 2, 2])).unwrap(),
            [0., 0., 0.5, 0.5]
        );
        assert_eq!(run([3., 0., 0.], None).unwrap(), [0., 0.5, 0.5, 0.]);
    }
    #[test]
    fn edge_index_is_not_limited_to_64() {
        let a = Endpoint {
            position: [0.; 3],
            members: &[1],
        };
        let b = Endpoint {
            position: [2., 0., 0.],
            members: &[1],
        };
        let o = accumulate(
            &[0.; 71],
            [1., 0., 0.],
            &[Query { key: 0, weight: 1. }],
            &[[0, 1]],
            &[a, b],
            Some(&[70, 65]),
            Policy::Mxcsr1f80,
            ctx(),
            SnapshotAdmission,
        )
        .unwrap();
        assert_eq!(o[70], 1.);
        let skipped = accumulate(
            &[0.; 71],
            [0., 0., 0.],
            &[Query { key: 0, weight: 1. }],
            &[[0, 1]],
            &[a, b],
            Some(&[70, 65]),
            Policy::Mxcsr1f80,
            ctx(),
            SnapshotAdmission,
        )
        .unwrap();
        assert!(skipped.iter().all(|&v| v == 0.));
    }
    #[test]
    fn bounds_zero_and_nonfinite_reject_transactionally() {
        let a = Endpoint {
            position: [0.; 3],
            members: &[],
        };
        let b = Endpoint {
            position: [0.; 3],
            members: &[1],
        };
        let q = [Query { key: 0, weight: 1. }];
        assert_eq!(
            accumulate(
                &[0.; 2],
                [0.; 3],
                &q,
                &[[0, 1]],
                &[a, b],
                None,
                Policy::Mxcsr1f80,
                ctx(),
                SnapshotAdmission
            ),
            Err(Error::ZeroLengthEdge)
        );
        let b = Endpoint {
            position: [2., 0., 0.],
            members: &[1],
        };
        assert_eq!(
            accumulate(
                &[0.; 2],
                [0.; 3],
                &q,
                &[[0, 1]],
                &[a, b],
                None,
                Policy::Mxcsr1f80,
                ctx(),
                SnapshotAdmission
            ),
            Err(Error::EmptyMembers)
        );
        assert_eq!(run([f32::NAN, 0., 0.], None), Err(Error::NonFiniteInput));
        assert_eq!(run([1.5, 0., 0.], Some(&[0])), Err(Error::MissingRemap));
    }
    #[test]
    fn rounding_order_and_repeated_updates() {
        let a = Endpoint {
            position: [0.; 3],
            members: &[0, 0, 1],
        };
        let b = Endpoint {
            position: [2., 0., 0.],
            members: &[0],
        };
        let out = accumulate(
            &[0.; 2],
            [0.; 3],
            &[Query { key: 0, weight: 1. }, Query { key: 0, weight: 1. }],
            &[[0, 1]],
            &[a, b],
            None,
            Policy::Mxcsr1f80,
            ctx(),
            SnapshotAdmission,
        )
        .unwrap();
        assert_eq!(out[0].to_bits(), 0x3faaaaab);
        assert_eq!(out[1].to_bits(), 0x3f2aaaab);
    }
    #[test]
    fn hostile_mxcsr_and_x87_control_are_restored() {
        fn controls() -> (u32, u16) {
            let mut mx = 0u32;
            let mut cw = 0u16;
            unsafe {
                std::arch::asm!("stmxcsr [{mx}]", "fnstcw [{cw}]", mx=in(reg) &mut mx, cw=in(reg) &mut cw, options(nostack,preserves_flags));
            }
            (mx, cw)
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &self.0, options(nostack,preserves_flags));
                }
            }
        }
        let (mx, cw) = controls();
        let _restore = Restore(mx);
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &hostile, options(nostack,preserves_flags));
        }
        assert_eq!(run([1.5, 0., 0.], None).unwrap(), [0.5, 0.25, 0.25, 0.]);
        assert_eq!(controls(), (hostile, cw));
        assert_eq!(run([1.5, 0., 0.], Some(&[0])), Err(Error::MissingRemap));
        assert_eq!(controls(), (hostile, cw));
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
            [0.; 3],
            &[],
            &[],
            &[],
            None,
            Policy::Mxcsr1f80,
            c,
            SnapshotAdmission
        ),
        Err(Error::UnsupportedArchitecture)
    );
}
