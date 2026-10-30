//! Bounded E6D7F0 geometry projection under explicit MXCSR1F80.
//!
//! Static boundary: re/cell-actor-geometry-consumer-static-v1.md. No actor
//! lookup, E67A50 cache population, provider, lifetime, collision or live
//! source state is modeled. Bounds, finite checks and identity checks below
//! are safe fixture policies, not claims the original performed them.
use crate::native_sse::{self, Policy, SseError};

#[derive(Clone, Copy, Debug)]
pub struct ActorInput {
    pub handle_fc: u32,
    pub position: [u32; 3], // actor +4C/+50/+54 binary32 bits
    pub scalar: u32,       // actor +58 binary32 bits
    pub flags_48: u8,
    pub matrix: [u32; 9], // actor +5C..+7C, physical order
}
#[derive(Clone, Copy, Debug)]
pub struct AdmittedCacheSnapshot {
    /// Caller asserts that this snapshot belongs to the current E67A50 key.
    /// This check does not prove pool/cache lifetime or provider identity.
    pub key_fc: u32,
    pub center: [u32; 3], // +1238/+123C/+1240
    pub scalar: u32,      // +1244
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Output { pub center: [u32; 3], pub scalar: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { MissingCache, WrongCacheKey, NonFiniteInput, NonFiniteIntermediate, Sse(SseError) }
impl From<SseError> for Error { fn from(value: SseError) -> Self { Self::Sse(value) } }

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! operation {
    ($name:ident, $instruction:literal) => {
        fn $name(a: u32, b: u32) -> Result<u32, Error> {
            let result: u32;
            // SAFETY: called only in SSE2-checked scoped MXCSR1F80.
            unsafe { std::arch::asm!("movd {x}, {a:e}", "movd {y}, {b:e}",
                $instruction, "movd {result:e}, {x}",
                x=out(xmm_reg) _, y=out(xmm_reg) _, a=in(reg) a, b=in(reg) b,
                result=lateout(reg) result, options(nostack, preserves_flags)); }
            if !f32::from_bits(result).is_finite() { return Err(Error::NonFiniteIntermediate); }
            Ok(result)
        }
    };
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
operation!(mul, "mulss {x}, {y}");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
operation!(add, "addss {x}, {y}");
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn mul(_: u32, _: u32) -> Result<u32, Error> { Err(Error::Sse(SseError::UnsupportedArchitecture)) }
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn add(_: u32, _: u32) -> Result<u32, Error> { Err(Error::Sse(SseError::UnsupportedArchitecture)) }
fn finite(bits: u32) -> bool { f32::from_bits(bits).is_finite() }

/// Branch on +FC before examining unused fields. The +FC==0 direct path
/// preserves finite binary32 bit patterns (including signed zero/subnormal),
/// corresponding to source copies and the exact finite x87 FLD/FSTP scalar.
/// Nonzero path requires an explicit stable cache snapshot with matching first
/// handle word and nonaliasing buffers. Intermediate overflow returns an error
/// rather than exposing the partially written native output.
pub fn project(actor: ActorInput, cache: Option<AdmittedCacheSnapshot>, policy: Policy) -> Result<Output, Error> {
    if actor.position.iter().any(|&x| !finite(x)) || !finite(actor.scalar) { return Err(Error::NonFiniteInput); }
    if actor.handle_fc == 0 { return Ok(Output { center: actor.position, scalar: actor.scalar }); }
    let cache = cache.ok_or(Error::MissingCache)?;
    if cache.key_fc != actor.handle_fc { return Err(Error::WrongCacheKey); }
    if cache.center.iter().any(|&x| !finite(x)) || !finite(cache.scalar) { return Err(Error::NonFiniteInput); }
    let rotated = actor.flags_48 & 2 != 0;
    if rotated && actor.matrix.iter().any(|&x| !finite(x)) { return Err(Error::NonFiniteInput); }
    native_sse::scoped(policy, || {
        let scalar = mul(cache.scalar, actor.scalar)?;
        let [cx,cy,cz] = cache.center;
        let [m0,m1,m2,m3,m4,m5,m6,m7,m8] = actor.matrix;
        let (x,y,z) = if rotated {
            // Preserve the actual E6D88F..E6D8E4 instruction schedule.
            let m6z=mul(m6,cz)?; let m3y=mul(m3,cy)?; let sx=add(m3y,m6z)?;
            let m4y=mul(m4,cy)?; let xm0=mul(cx,m0)?; let x=add(sx,xm0)?;
            let m1x=mul(m1,cx)?; let sy=add(m1x,m4y)?; let m7z=mul(m7,cz)?; let y=add(sy,m7z)?;
            let m2x=mul(m2,cx)?; let m5y=mul(m5,cy)?; let sz=add(m2x,m5y)?;
            let m8z=mul(m8,cz)?; let z=add(sz,m8z)?;
            (x,y,z)
        } else { (cx,cy,cz) };
        // E6D8FB..E6D935: y*scale, scale*z, scale*x, then
        // position-y + y, position-z + z, position-x + x.
        let sy=mul(y,actor.scalar)?; let sz=mul(actor.scalar,z)?; let sx=mul(actor.scalar,x)?;
        let y=add(actor.position[1],sy)?; let z=add(actor.position[2],sz)?; let x=add(actor.position[0],sx)?;
        Ok(Output { center:[x,y,z], scalar })
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn b(x:f32)->u32 { x.to_bits() }
    fn actor() -> ActorInput { ActorInput { handle_fc:7, position:[b(10.),b(20.),b(30.)], scalar:b(2.), flags_48:2,
        matrix:[b(1.),b(0.),b(0.),b(0.),b(1.),b(0.),b(0.),b(0.),b(1.)] } }
    fn cache() -> AdmittedCacheSnapshot { AdmittedCacheSnapshot { key_fc:7, center:[b(1.),b(2.),b(3.)], scalar:b(2.) } }
    #[test]
    fn identity_projection_and_scalar() {
        assert_eq!(project(actor(),Some(cache()),Policy::Mxcsr1f80),Ok(Output { center:[b(12.),b(24.),b(36.)],scalar:b(4.) }));
    }
    #[test]
    fn nontrivial_matrix_uses_source_physical_indices() {
        let mut a=actor(); a.matrix=[b(1.),b(2.),b(3.),b(4.),b(5.),b(6.),b(7.),b(8.),b(9.)];
        assert_eq!(project(a,Some(cache()),Policy::Mxcsr1f80),Ok(Output { center:[b(70.),b(92.),b(114.)],scalar:b(4.) }));
    }
    #[test]
    fn x_sum_order_preserves_small_term_after_large_cancellation() {
        let mut a=actor(); a.position=[b(0.);3]; a.scalar=b(1.);
        a.matrix=[b(1.),0,0,b(1e20),0,0,b(-1e20),0,0];
        let mut c=cache(); c.center=[b(1.);3]; c.scalar=b(1.);
        // Source adds m3*y + m6*z first, exactly cancelling, then m0*x.
        // Reassociating m0*x into the large terms would round it away.
        assert_eq!(project(a,Some(c),Policy::Mxcsr1f80),
                   Ok(Output { center:[b(1.),b(0.),b(0.)],scalar:b(1.) }));
    }
    #[test]
    fn zero_fc_is_direct_copy_and_ignores_unused_fields() {
        let mut a=actor(); a.handle_fc=0; a.position=[0x8000_0000,1,b(3.)];a.scalar=0x8000_0000;a.matrix=[f32::NAN.to_bits();9];
        assert_eq!(project(a,None,Policy::Mxcsr1f80),Ok(Output { center:a.position,scalar:a.scalar }));
    }
    #[test]
    fn clear_rotation_flag_ignores_matrix_and_uses_explicit_cache() {
        let mut a=actor();a.flags_48=0;a.matrix=[f32::NAN.to_bits();9];
        assert_eq!(project(a,Some(cache()),Policy::Mxcsr1f80),Ok(Output { center:[b(12.),b(24.),b(36.)],scalar:b(4.) }));
        let mut bad=cache();bad.key_fc=8;
        assert_eq!(project(a,Some(bad),Policy::Mxcsr1f80),Err(Error::WrongCacheKey));
        assert_eq!(project(a,None,Policy::Mxcsr1f80),Err(Error::MissingCache));
    }
    #[test]
    fn rejects_nonfinite_and_overflow_without_returning_partial_output() {
        let mut a=actor();a.position[0]=f32::NAN.to_bits();
        assert_eq!(project(a,Some(cache()),Policy::Mxcsr1f80),Err(Error::NonFiniteInput));
        let mut a=actor();a.scalar=f32::MAX.to_bits();
        assert_eq!(project(a,Some(cache()),Policy::Mxcsr1f80),Err(Error::NonFiniteIntermediate));
    }
    #[test]
    fn negative_zero_is_preserved_in_both_geometry_branches() {
        let mut a=actor(); a.position=[0x8000_0000;3]; a.scalar=b(1.);
        let mut c=cache(); c.center=[0x8000_0000;3]; c.scalar=b(1.);
        // Every center product and sum is -0: the identity matrix contains
        // +0/+1, and all three supplied center components are -0.
        for flags in [0,2] {
            a.flags_48=flags;
            assert_eq!(project(a,Some(c),Policy::Mxcsr1f80),
                Ok(Output { center:[0x8000_0000;3],scalar:b(1.) }));
        }
    }
    #[test]
    fn gradual_underflow_and_zero_sign_follow_each_branch() {
        let mut a=actor(); a.position=[0x8000_0000;3]; a.scalar=b(0.5);
        let mut c=cache(); c.center=[3,0x8000_0000,0x8000_0003]; c.scalar=b(1.);
        // Half of three least subnormal units rounds to even two units.
        // With rotation disabled, -0 + -0 stays negative and -3/2 -> -2.
        a.flags_48=0;
        assert_eq!(project(a,Some(c),Policy::Mxcsr1f80),
            Ok(Output { center:[2,0x8000_0000,0x8000_0002],scalar:b(0.5) }));
        // Identity rotation's x/y sums include opposite-signed zeros; nearest
        // addition gives +0. z retains its negative subnormal magnitude.
        a.flags_48=2;
        assert_eq!(project(a,Some(c),Policy::Mxcsr1f80),
            Ok(Output { center:[2,0,0x8000_0002],scalar:b(0.5) }));
    }
}
