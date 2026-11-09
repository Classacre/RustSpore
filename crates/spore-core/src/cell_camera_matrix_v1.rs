//! Bounded 10436F0 camera basis projection for controlled finite inputs.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Unadmitted, UnsupportedControl, UnsupportedFloat }

/// Projects the original helper's SSE ordering and x87 PC53 normalization.
///
/// `eye`, `target`, and `up` are explicit disjoint snapshots. The caller admits
/// stable storage, no intervening mutation, and host binary64 square-root and
/// arithmetic matching the source x87 PC53 operations. Only finite binary32
/// normals and zeros, including intermediate SSE results, are accepted; CW
/// 027F and MXCSR 1F80 are required. This is a controlled approximation whose
/// bit agreement has only been checked on the retained fixture, not a general
/// x87 equivalence or an observed gameplay camera.
pub fn project(eye:[u32;3], target:[u32;3], up:[u32;3], cw:u16, mxcsr:u32, admitted:bool) -> Result<[u32;16],Error> {
    if !admitted { return Err(Error::Unadmitted); }
    if cw!=0x027f || mxcsr!=0x1f80 { return Err(Error::UnsupportedControl); }
    fn valid(x:f32)->bool { x.is_finite() && (x==0.0 || x.is_normal()) }
    fn op(x:f32)->Result<f32,Error> { if valid(x) { Ok(x) } else { Err(Error::UnsupportedFloat) } }
    fn mul(a:f32,b:f32)->Result<f32,Error> { op(a*b) }
    fn sub(a:f32,b:f32)->Result<f32,Error> { op(a-b) }
    fn factor(x:f32,y:f32,z:f32, second:bool)->Result<f32,Error> {
        let x=x as f64; let y=y as f64; let z=z as f64;
        let epsilon=f32::from_bits(0x322bcc77) as f64;
        let squared=if second { (z*z+y*y)+x*x } else { (x*x+z*z)+y*y };
        let value=(1.0f64/(squared+epsilon).sqrt()) as f32;
        op(value)
    }
    let e=eye.map(f32::from_bits); let t=target.map(f32::from_bits); let u=up.map(f32::from_bits);
    if !e.iter().chain(t.iter()).chain(u.iter()).all(|&x|valid(x)) { return Err(Error::UnsupportedFloat); }
    let x=sub(t[0],e[0])?; let y=sub(t[1],e[1])?; let z=sub(t[2],e[2])?;
    let k=factor(x,y,z,false)?;
    let fx=mul(k,x)?; let fy=mul(k,y)?; let fz=mul(z,k)?;
    let rx=sub(mul(u[2],fy)?,mul(u[1],fz)?)?;
    let ry=sub(mul(u[0],fz)?,mul(u[2],fx)?)?;
    let rz=sub(mul(u[1],fx)?,mul(u[0],fy)?)?;
    let q=factor(rx,ry,rz,true)?;
    let rx=mul(q,rx)?; let ry=mul(ry,q)?; let rz=mul(rz,q)?;
    let ux=sub(mul(ry,fz)?,mul(rz,fy)?)?;
    let uy=sub(mul(rz,fx)?,mul(fz,rx)?)?;
    let uz=sub(mul(rx,fy)?,mul(ry,fx)?)?;
    Ok([rx.to_bits(),ry.to_bits(),rz.to_bits(),0,fx.to_bits(),fy.to_bits(),fz.to_bits(),0,
        ux.to_bits(),uy.to_bits(),uz.to_bits(),0,eye[0],eye[1],eye[2],0x3f80_0000])
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn admissions_and_degenerate() {
        let up=[0,0x3f80_0000,0];
        assert_eq!(project([0;3],[0;3],up,0x027f,0x1f80,false),Err(Error::Unadmitted));
        assert_eq!(project([0;3],[0;3],up,0x037f,0x1f80,true),Err(Error::UnsupportedControl));
        assert_eq!(project([1,0,0],[0;3],up,0x027f,0x1f80,true),Err(Error::UnsupportedFloat));
        let m=project([0;3],[0;3],up,0x027f,0x1f80,true).unwrap();
        assert!(m[..12].iter().all(|w|w&0x7fff_ffff==0)); assert_eq!(m[15],0x3f80_0000);
    }
}
