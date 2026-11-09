//! Bounded 50B020 face-orientation classifier for explicitly supplied geometry.
//! The native comparison admits x87 control 027F and MXCSR 1F80, finite inputs
//! and intermediates. This does not determine actual groups, UVs or live identity.
use crate::native_sse::{self, Policy, SseError};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidFace, InvalidPosition, NonFinite, UnsupportedArchitecture }
impl From<SseError> for Error { fn from(_: SseError) -> Self { Self::UnsupportedArchitecture } }
fn finite(x: f32) -> Result<f32, Error> { if x.is_finite() { Ok(x) } else { Err(Error::NonFinite) } }

pub fn classify(positions: &[[f32;3]], indices: &[u32], face: usize, policy: Policy) -> Result<u32, Error> {
    let start=face.checked_mul(3).ok_or(Error::InvalidFace)?;
    let end=start.checked_add(3).ok_or(Error::InvalidFace)?;
    let tri=indices.get(start..end).ok_or(Error::InvalidFace)?;
    let a=*positions.get(tri[0] as usize).ok_or(Error::InvalidPosition)?;
    let b=*positions.get(tri[1] as usize).ok_or(Error::InvalidPosition)?;
    let c=*positions.get(tri[2] as usize).ok_or(Error::InvalidPosition)?;
    if a.iter().chain(&b).chain(&c).any(|v| !v.is_finite()) { return Err(Error::NonFinite); }
    native_sse::scoped(policy, || {
        let mut u=[0.0;3];let mut v=[0.0;3];
        for k in 0..3 { v[k]=finite(c[k]-a[k])?; u[k]=finite(b[k]-a[k])?; }
        let x=finite(finite(u[1]*v[2])?-finite(u[2]*v[1])?)?;
        let y=finite(finite(u[2]*v[0])?-finite(u[0]*v[2])?)?;
        let z=finite(finite(u[0]*v[1])?-finite(u[1]*v[0])?)?;
        let n=[x,y,z];let q=[x.abs(),y.abs(),z.abs()];
        let axis=if q[0]>q[1] { if q[0]>q[2] {0} else {2} } else if q[1]>q[2] {1} else {2};
        Ok(axis as u32*2+u32::from(n[axis]<0.0))
    })
}
#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
 use super::*;
 #[test] fn ties_sign_zero_and_bounds() {
  let p=[[0.,0.,0.],[1.,-1.,0.],[1.,0.,-1.],[1.,0.,0.],[0.,-1.,0.],[-0.,0.,-0.]];
  assert_eq!(classify(&p,&[0,1,2],0,Policy::Mxcsr1f80),Ok(4));
  assert_eq!(classify(&p,&[0,3,4],0,Policy::Mxcsr1f80),Ok(5));
  assert_eq!(classify(&p,&[5,5,5],0,Policy::Mxcsr1f80),Ok(4));
  assert_eq!(classify(&p,&[0,1,2],1,Policy::Mxcsr1f80),Err(Error::InvalidFace));
  assert_eq!(classify(&p,&[0,1,6],0,Policy::Mxcsr1f80),Err(Error::InvalidPosition));
  assert_eq!(classify(&[[f32::INFINITY,0.,0.]],&[0,0,0],0,Policy::Mxcsr1f80),Err(Error::NonFinite));
 }
}
