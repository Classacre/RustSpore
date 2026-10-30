//! Bounded E7F327..E7F381 strict broadphase predicate only.
//! Inputs are caller-supplied E6D7F0 geometry values, not authenticated actors,
//! pool handles, cache outputs or observed contacts. This does not run E7F170,
//! pair traversal, callbacks, event dispatch or any source instructions.
//! Safe errors suppress output instead of reproducing native exceptions,
//! unordered comparisons, partial effects or status flags. Explicit MXCSR1F80.
use crate::{cell_actor_geometry_projection_v1::Output,
    native_sse::{self, Policy, SseError}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { NonFiniteInput, NonFiniteIntermediate, Sse(SseError) }
impl From<SseError> for Error { fn from(e:SseError)->Self { Self::Sse(e) } }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate { pub delta: [u32;3], pub squared_distance:u32, pub squared_sum:u32, pub passes:bool }

#[cfg(any(target_arch="x86",target_arch="x86_64"))]
macro_rules! op { ($name:ident,$inst:literal) => {
    fn $name(a:u32,b:u32)->Result<u32,Error> {
        let out:u32;
        // SAFETY: caller holds the SSE2-checked, masked MXCSR1F80 scope.
        unsafe { std::arch::asm!("movd {x}, {a:e}","movd {y}, {b:e}",$inst,"movd {out:e}, {x}",
            x=out(xmm_reg) _,y=out(xmm_reg) _,a=in(reg)a,b=in(reg)b,
            out=lateout(reg)out,options(nostack,preserves_flags)); }
        if !f32::from_bits(out).is_finite() { return Err(Error::NonFiniteIntermediate); }
        Ok(out)
    }
}; }
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(add,"addss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(sub,"subss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(mul,"mulss {x}, {y}");
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn add(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn sub(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn mul(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}

#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn strict_greater(a:u32,b:u32)->bool {
    let out:u8;
    // SAFETY: scoped SSE2, finite inputs and masked exceptions. SETA is the
    // inverse of the source JBE, including unordered (excluded by admission).
    unsafe { std::arch::asm!("movd {x}, {a:e}","movd {y}, {b:e}","comiss {x}, {y}","seta {out}",
        x=out(xmm_reg) _, y=out(xmm_reg) _, a=in(reg)a,b=in(reg)b,
        out=lateout(reg_byte)out,options(nostack)); }
    out != 0
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn strict_greater(_:u32,_:u32)->bool { false }

/// Both geometry outputs must be finite. Subnormal inputs/intermediates are
/// preserved by MXCSR1F80. The source obtains them only after both checked pool
/// lookups; callers separately admit that provenance and the geometry results.
/// `passes` means only the strict source predicate, not a resulting contact.
pub fn evaluate(a:Output,b:Output,policy:Policy)->Result<Gate,Error> {
    if a.center.iter().chain(b.center.iter()).chain([&a.scalar,&b.scalar]).any(|v|!f32::from_bits(*v).is_finite()) {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy,|| {
        let dz=sub(a.center[2],b.center[2])?;
        let dy=sub(a.center[1],b.center[1])?;
        let dx=sub(a.center[0],b.center[0])?;
        let sum=add(a.scalar,b.scalar)?;
        let z2=mul(dz,dz)?; let y2=mul(dy,dy)?; let x2=mul(dx,dx)?;
        let zy=add(z2,y2)?; let distance=add(zy,x2)?;
        let sum2=mul(sum,sum)?;
        Ok(Gate { delta:[dx,dy,dz], squared_distance:distance, squared_sum:sum2,
            passes:strict_greater(sum2,distance) })
    })
}

#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests {
    use super::*;
    fn b(x:f32)->u32{x.to_bits()}
    fn geom(x:f32,y:f32,z:f32,r:f32)->Output { Output { center:[b(x),b(y),b(z)],scalar:b(r) } }
    const P:Policy=Policy::Mxcsr1f80;
    #[test]
    fn equality_skips_and_adjacent_threshold_crosses() {
        let a=geom(1.,0.,0.,0.5); let c=geom(0.,0.,0.,0.5);
        let eq=evaluate(a,c,P).unwrap(); assert_eq!(eq.squared_distance,b(1.)); assert_eq!(eq.squared_sum,b(1.)); assert!(!eq.passes);
        // One ULP at 0.5 produces a halfway sum that rounds back to 1.
        let halfway=Output { scalar:b(0.5)+1,..c };
        assert!(!evaluate(a,halfway,P).unwrap().passes);
        let larger=Output { scalar:b(0.5)+2,..c };
        assert!(evaluate(a,larger,P).unwrap().passes);
        let smaller=Output { scalar:b(0.5)-1,..c };
        assert!(!evaluate(a,smaller,P).unwrap().passes);
    }
    #[test]
    fn rounding_and_source_sum_order_change_strict_decision() {
        let a=geom(1.,4096.,0.,4096.);let c=geom(0.,0.,0.,0.);
        let g=evaluate(a,c,P).unwrap();
        // 2^24 + 1 rounds to 2^24; equality skips despite nonzero x.
        assert_eq!(g.squared_distance,b(16_777_216.));
        assert_eq!(g.squared_sum,b(16_777_216.));assert!(!g.passes);
    }
    #[test]
    fn subtraction_signed_zero_and_exact_cancellation() {
        let a=Output { center:[b(16_777_216.),0x8000_0000,b(1.)],scalar:b(1.) };
        let c=Output { center:[b(16_777_215.),0,b(1.)],scalar:0 };
        let g=evaluate(a,c,P).unwrap();assert_eq!(g.delta,[b(1.),0x8000_0000,0]);
        assert!(!g.passes);
    }
    #[test]
    fn nonfinite_or_overflow_is_explicit_error() {
        let zero=geom(0.,0.,0.,0.);
        assert_eq!(evaluate(geom(f32::NAN,0.,0.,0.),zero,P),Err(Error::NonFiniteInput));
        assert_eq!(evaluate(geom(f32::MAX,0.,0.,0.),zero,P),Err(Error::NonFiniteIntermediate));
    }
    #[test]
    fn gradual_underflow_and_negative_scalar_cancellation() {
        let a=Output { center:[1,0x8000_0000,0],scalar:b(1.) };
        let c=Output { center:[0,0,0],scalar:b(-1.) };
        let g=evaluate(a,c,P).unwrap();
        // The smallest positive subnormal survives subtraction, but its
        // square rounds to +0. Exact +1 + -1 also becomes +0 in nearest mode.
        assert_eq!(g.delta,[1,0x8000_0000,0]);
        assert_eq!(g.squared_distance,0);assert_eq!(g.squared_sum,0);assert!(!g.passes);
    }
    #[test]
    fn distance_additions_are_not_reassociated() {
        let a=geom(1.,1.,4096.,4096.);let c=geom(0.,0.,0.,0.);
        let g=evaluate(a,c,P).unwrap();
        // (2^24 + 1) + 1 rounds twice to 2^24; reassociating the two
        // unit squares first gives the distinct representable 2^24 + 2.
        assert_eq!(g.squared_distance,0x4b80_0000);
        assert_ne!(g.squared_distance,0x4b80_0001);
        assert_eq!(g.squared_sum,0x4b80_0000);assert!(!g.passes);
    }
    #[test]
    fn scoped_mxcsr_is_restored_on_success_and_arithmetic_error() {
        fn get()->u32 { let mut v=0u32; unsafe { std::arch::asm!("stmxcsr [{v}]",v=in(reg)&mut v,options(nostack,preserves_flags)); } v }
        struct Restore(u32);
        impl Drop for Restore { fn drop(&mut self) { unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&self.0,options(nostack,preserves_flags)); } } }
        if !std::is_x86_feature_detected!("sse2") { return; }
        let _restore=Restore(get()); let hostile=0xdf81u32;
        // SAFETY: valid masked control bits on a detected SSE2 processor;
        // RAII restores the caller's exact saved MXCSR even on panic.
        unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&hostile,options(nostack,preserves_flags)); }
        let zero=geom(0.,0.,0.,0.);
        assert!(evaluate(zero,zero,P).is_ok()); assert_eq!(get(),hostile);
        assert_eq!(evaluate(geom(f32::MAX,0.,0.,0.),zero,P),Err(Error::NonFiniteIntermediate));
        assert_eq!(get(),hostile);
    }
}
