//! Experimental single-active-child 009E7C80 mode-2 arithmetic.
//! Synthetic state must be explicitly admitted; no live-animation claim.
#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub definition_quaternion: [u32;4],
    pub parent_definition_position: [u32;3],
    pub child_definition_position: [u32;3],
    pub stage_quaternion: [u32;4],
    pub stage_position: [u32;3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { UnsupportedArchitecture, UnsupportedMxcsr, NonFinite }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Output {
    pub position: [u32;3], pub quaternion: [u32;4],
    pub counters: [u32;3], pub node_flags: [u8;4], pub node_210: u32,
}
/// Runs only with default SSE exception masks, round-to-nearest, and FTZ/DAZ
/// disabled. Arithmetic status flags may change. Caller owns this environment.
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
pub fn evaluate_explicit_synthetic_mode2(input: &Input) -> Result<Output, Error> {
    if !std::is_x86_feature_detected!("sse2") { return Err(Error::UnsupportedArchitecture); }
    let mut csr=0u32;
    unsafe { std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut csr,options(nostack,preserves_flags)); }
    if csr & !0x3f != 0x1f80 { return Err(Error::UnsupportedMxcsr); }
    if input.definition_quaternion.iter().chain(&input.parent_definition_position).chain(&input.child_definition_position).chain(&input.stage_quaternion).chain(&input.stage_position).any(|&x| !f32::from_bits(x).is_finite()) { return Err(Error::NonFinite); }
    // SAFETY: SSE2 detection and MXCSR admission above; local scalar values only.
    unsafe { evaluate_inner(input) }
}
#[cfg(not(any(target_arch="x86", target_arch="x86_64")))]
pub fn evaluate_explicit_synthetic_mode2(_: &Input) -> Result<Output, Error> { Err(Error::UnsupportedArchitecture) }

#[cfg(any(target_arch="x86", target_arch="x86_64"))]
#[target_feature(enable="sse2")]
unsafe fn evaluate_inner(input: &Input) -> Result<Output, Error> {
    #[cfg(target_arch="x86")] use std::arch::x86::*;
    #[cfg(target_arch="x86_64")] use std::arch::x86_64::*;
    #[inline] unsafe fn add(a:f32,b:f32)->f32 { _mm_cvtss_f32(_mm_add_ss(_mm_set_ss(a),_mm_set_ss(b))) }
    #[inline] unsafe fn sub(a:f32,b:f32)->f32 { _mm_cvtss_f32(_mm_sub_ss(_mm_set_ss(a),_mm_set_ss(b))) }
    #[inline] unsafe fn mul(a:f32,b:f32)->f32 { _mm_cvtss_f32(_mm_mul_ss(_mm_set_ss(a),_mm_set_ss(b))) }
    #[inline] fn neg(a:f32)->f32 { f32::from_bits(a.to_bits() ^ 0x80000000) }
    let [dx,dy,dz,dw]=input.definition_quaternion.map(f32::from_bits);
    let [x,y,z,w]=input.stage_quaternion.map(f32::from_bits);
    let qx=sub(add(sub(mul(x,dw),mul(dx,w)),mul(dy,z)),mul(dz,y));
    let qy=add(sub(sub(mul(y,dw),mul(z,dx)),mul(dy,w)),mul(dz,x));
    let qz=sub(sub(add(mul(y,dx),mul(z,dw)),mul(dy,x)),mul(dz,w));
    let qw=add(add(add(mul(y,dy),mul(dz,z)),mul(dx,x)),mul(w,dw));
    // Caller 009E8930 starts the accumulator at positive zero. Source dot
    // expression uses (y*qy + z*qz) + w*qw + x*qx.
    let dot=add(add(add(mul(0.0,qy),mul(0.0,qz)),mul(0.0,qw)),mul(0.0,qx));
    if !dot.is_finite() { return Err(Error::NonFinite); }
    let q=[qx,qy,qz,qw];
    let accumulated=if 0.0 <= dot { q.map(|v| add(0.0,v)) } else { q.map(|v| sub(0.0,v)) };
    let pp=input.parent_definition_position.map(f32::from_bits);
    let cp=input.child_definition_position.map(f32::from_bits);
    let v=std::array::from_fn::<_,3,_>(|i| sub(mul(pp[i],1.0),mul(cp[i],1.0)));
    let [vx,vy,vz]=v;
    let xx=neg(mul(qx,qx));let yy=neg(mul(qy,qy));let zz=neg(mul(qz,qz));
    let xw=mul(qx,qw);let yw=mul(qy,qw);let zw=mul(qz,qw);
    let yx=mul(qy,qx);let zx=mul(qz,qx);let zy=mul(qz,qy);
    let rx=add(mul(add(add(mul(add(zz,yy),vx),mul(add(zx,yw),vz)),mul(sub(yx,zw),vy)),2.0),vx);
    let ry=add(mul(add(add(mul(add(yx,zw),vx),mul(sub(zy,xw),vz)),mul(add(zz,xx),vy)),2.0),vy);
    let rz=add(mul(add(add(mul(sub(zx,yw),vx),mul(add(yy,xx),vz)),mul(add(zy,xw),vy)),2.0),vz);
    let sp=input.stage_position.map(f32::from_bits);
    let position=[add(0.0,add(sp[0],rx)),add(0.0,add(sp[1],ry)),add(add(sp[2],rz),0.0)];
    if position.iter().chain(&accumulated).any(|v| !v.is_finite()) { return Err(Error::NonFinite); }
    Ok(Output { position: position.map(f32::to_bits), quaternion: accumulated.map(f32::to_bits), counters:[1,1,1],node_flags:[4,0,255,255],node_210:0xbf800000 })
}

#[cfg(all(test, any(target_arch="x86",target_arch="x86_64")))]
mod tests {
    use super::*;
    struct Restore(u32);
    impl Drop for Restore { fn drop(&mut self) { unsafe { std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&self.0,options(nostack,preserves_flags)); } } }
    #[test]
    fn identity_quaternion_and_simple_translation() {
        if !std::is_x86_feature_detected!("sse2") { return; }
        let mut old=0u32; unsafe { std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut old,options(nostack,preserves_flags)); }
        let _restore=Restore(old); let csr=0x1f80u32; unsafe { std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&csr,options(nostack,preserves_flags)); }
        let bits=|v:f32| v.to_bits();
        let input=Input { definition_quaternion:[0,0,0,bits(1.0)], parent_definition_position:[bits(1.0),bits(2.0),bits(3.0)], child_definition_position:[0;3], stage_quaternion:[0,0,0,bits(1.0)], stage_position:[bits(1.0);3] };
        let out=evaluate_explicit_synthetic_mode2(&input).unwrap();
        assert_eq!(out.position,[bits(2.0),bits(3.0),bits(4.0)]);
        assert_eq!(out.quaternion,[0,0,0,bits(1.0)]);
        assert_eq!(out.counters,[1,1,1]); assert_eq!(out.node_flags,[4,0,255,255]); assert_eq!(out.node_210,bits(-1.0));
    }
}
