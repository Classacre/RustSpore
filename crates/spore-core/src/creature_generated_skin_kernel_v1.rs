//! Controlled finite four-lane 4F8AE0 influence kernel.
//!
//! Given an explicitly supplied point and four lanes from one 0x80-byte source
//! record, emit ordered positive append requests. This does not query a spatial
//! table, perform vector allocation, bind part/palette identities, evaluate a
//! full mesh or observe a live editor. The source's checked record pointers,
//! append helper, x87 stack and masked exceptions are caller admissions.
//! Nonfinite checks and transactional errors are stricter than native behavior;
//! native sticky flags and partial append side effects are not reproduced.
use crate::native_sse::{self, Policy, SseError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lane {
    pub center: [f32; 3],
    /// The source word at +30, compared strictly to squared distance.
    pub cutoff: f32,
    pub scale: f32,
    pub coefficient: f32,
    /// Raw source +70 word; not assumed to be a row or palette index.
    pub key: u32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppendRequest {
    pub lane: u8,
    pub key: u32,
    pub weight: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Output {
    pub entries: [Option<AppendRequest>; 4],
    pub len: usize,
}
/// Admit stable valid nonaliasing source/append storage, masked x87 exceptions,
/// sufficient x87 stack and exact finite binary32 FLD/compare semantics.
/// The actual append helper and source FP status are not executed or modeled.
#[derive(Clone, Copy, Debug)]
pub struct KernelAdmission;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
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
        write!(f, "generated skin influence kernel: {self:?}")
    }
}
impl std::error::Error for Error {}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn sub(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("subss {a}, {b}",a=inout(xmm_reg) a,b=in(xmm_reg) b,options(nostack,preserves_flags));
    }
    a
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {a}, {b}",a=inout(xmm_reg) a,b=in(xmm_reg) b,options(nostack,preserves_flags));
    }
    a
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mul(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("mulss {a}, {b}",a=inout(xmm_reg) a,b=in(xmm_reg) b,options(nostack,preserves_flags));
    }
    a
}

/// Evaluate the source lane order under explicit SSE controls. All fields are
/// checked finite, including source fields on lanes the native routine skips.
pub fn evaluate(
    point: [f32; 3],
    lanes: [Lane; 4],
    policy: Policy,
    _admit: KernelAdmission,
) -> Result<Output, Error> {
    if !point.into_iter().all(f32::is_finite)
        || lanes.iter().any(|l| {
            !l.center
                .into_iter()
                .chain([l.cutoff, l.scale, l.coefficient])
                .all(f32::is_finite)
        })
    {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let mut out = Output {
                entries: [None; 4],
                len: 0,
            };
            for (lane, l) in lanes.iter().enumerate() {
                // x87 comparison treats both signs of zero as equal.
                if l.cutoff.to_bits() & 0x7fff_ffff == 0 {
                    continue;
                }
                let dx = sub(point[0], l.center[0]);
                let dy = sub(point[1], l.center[1]);
                let dz = sub(point[2], l.center[2]);
                let xx = mul(dx, dx);
                let yy = mul(dy, dy);
                let xy = add(xx, yy);
                let zz = mul(dz, dz);
                let distance = add(xy, zz);
                if ![dx, dy, dz, xx, yy, xy, zz, distance]
                    .into_iter()
                    .all(f32::is_finite)
                {
                    return Err(Error::NonFiniteIntermediate);
                }
                if l.cutoff <= distance {
                    continue;
                }
                let t = sub(mul(distance, l.scale), 1.0);
                let q = mul(t, t);
                let first = mul(l.coefficient, q);
                let weight = mul(first, q);
                if ![t, q, first, weight].into_iter().all(f32::is_finite) {
                    return Err(Error::NonFiniteIntermediate);
                }
                if weight > 0.0 {
                    out.entries[out.len] = Some(AppendRequest {
                        lane: lane as u8,
                        key: l.key,
                        weight,
                    });
                    out.len += 1;
                }
            }
            Ok(out)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = (point, lanes);
            Err(Error::UnsupportedArchitecture)
        }
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn lane() -> Lane {
        Lane {
            center: [0.; 3],
            cutoff: 2.,
            scale: 0.25,
            coefficient: 0.5,
            key: 123,
        }
    }
    fn run(p: [f32; 3], l: [Lane; 4]) -> Result<Output, Error> {
        evaluate(p, l, Policy::Mxcsr1f80, KernelAdmission)
    }
    #[test]
    fn strict_cutoff_zero_and_ordered_requests() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let mut l = [lane(); 4];
        l[0].cutoff = -0.;
        l[1].cutoff = 1.;
        l[2].cutoff = f32::from_bits(1.0f32.to_bits() + 1);
        l[2].key = 0xdeadbeef;
        l[3].coefficient = -0.5;
        let out = run([1., 0., 0.], l).unwrap();
        assert_eq!(out.len, 1);
        assert_eq!(
            out.entries[0],
            Some(AppendRequest {
                lane: 2,
                key: 0xdeadbeef,
                weight: f32::from_bits(0x3e220000)
            })
        );
        assert_eq!(out.entries[1..], [None; 3]);
    }
    #[test]
    fn four_lanes_and_rounded_weight() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let l = std::array::from_fn(|i| Lane {
            key: i as u32,
            ..lane()
        });
        let out = run([1., 0., 0.], l).unwrap();
        assert_eq!(out.len, 4);
        for i in 0..4 {
            let x = out.entries[i].unwrap();
            assert_eq!(
                (x.lane, x.key, x.weight.to_bits()),
                (i as u8, i as u32, 0x3e220000)
            );
        }
        let mut l = [lane(); 4];
        l[0].scale = 1.;
        assert_eq!(run([1., 0., 0.], l).unwrap().entries[0].unwrap().lane, 1);
    }
    #[test]
    fn multiplication_order_changes_the_last_bit() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let mut lanes = [lane(); 4];
        lanes[0].scale = f32::from_bits(0x3fd878f4);
        lanes[0].coefficient = f32::from_bits(0x3f06af70);
        for lane in &mut lanes[1..] {
            lane.cutoff = 0.;
        }
        let out = run([f32::from_bits(0x3de5123d), 0., 0.], lanes).unwrap();
        assert_eq!(out.entries[0].unwrap().weight.to_bits(), 0x3ef7495e);
        // Reassociating coefficient * (q * q) yields 0x3ef7495f here.
    }
    #[test]
    fn nonfinite_overflow_are_rejected_before_emitting() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let mut l = [lane(); 4];
        l[2].scale = f32::NAN;
        assert_eq!(run([0.; 3], l), Err(Error::NonFiniteInput));
        assert_eq!(
            run([f32::MAX, 0., 0.], [lane(); 4]),
            Err(Error::NonFiniteIntermediate)
        );
    }
    #[test]
    fn signed_zero_and_subnormal_under_hostile_mxcsr() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        fn read() -> u32 {
            let mut x = 0;
            unsafe {
                std::arch::asm!("stmxcsr [{p}]",p=in(reg) &mut x,options(nostack,preserves_flags));
            }
            x
        }
        struct Guard(u32);
        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &self.0,options(nostack,preserves_flags));
                }
            }
        }
        let _g = Guard(read());
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &hostile,options(nostack,preserves_flags));
        }
        let mut l = [lane(); 4];
        l[0].center = [0.; 3];
        l[0].cutoff = 1.;
        l[0].scale = 0.;
        l[0].coefficient = f32::from_bits(1);
        l[1].cutoff = -0.;
        l[2].cutoff = 0.;
        l[3].cutoff = 0.;
        let o = run([-0., 0., 0.], l).unwrap();
        assert_eq!(o.len, 1);
        assert_eq!(o.entries[0].unwrap().weight.to_bits(), 1);
        assert_eq!(read(), hostile);
    }
}
#[cfg(all(test, not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[test]
fn unsupported_architecture_is_explicit() {
    let l = Lane {
        center: [0.; 3],
        cutoff: 1.,
        scale: 1.,
        coefficient: 1.,
        key: 0,
    };
    assert_eq!(
        evaluate([0.; 3], [l; 4], Policy::Mxcsr1f80, KernelAdmission),
        Err(Error::UnsupportedArchitecture)
    );
}
