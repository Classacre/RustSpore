//! Bounded first-side E6AE6C..E6AF2F feature transform, static model only.
//! Caller supplies stable nonaliasing row/actor fields and MXCSR policy. It
//! neither decides contact nor establishes native actor/cache identity.
use spore_core::native_sse::{self,Policy,SseError};
use spore_core::cell_part_geometry::NativeCellPart;
use spore_core::native_x87::{Context,Extended,X87Error};

pub(super) use spore_core::cell_feature_projection_v1::{Transform,Projected};
#[derive(Debug,PartialEq,Eq)]
pub(super) enum Error { NonFinite, NegativeRadius, Sse(SseError), X87(X87Error) }
impl From<SseError> for Error { fn from(e:SseError)->Self { Self::Sse(e) } }
impl From<X87Error> for Error { fn from(e:X87Error)->Self { Self::X87(e) } }
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
macro_rules! operation {
    ($name:ident,$ins:literal) => {
        fn $name(a:u32,b:u32)->Result<u32,Error> {
            let r:u32;
            // Scoped policy checks SSE2 support and restores MXCSR.
            unsafe { std::arch::asm!("movd {x}, {a:e}","movd {y}, {b:e}",$ins,"movd {r:e}, {x}",
                x=out(xmm_reg) _,y=out(xmm_reg) _,a=in(reg) a,b=in(reg) b,r=lateout(reg) r,options(nostack,preserves_flags)); }
            if !f32::from_bits(r).is_finite() { return Err(Error::NonFinite); }
            Ok(r)
        }
    }
}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
operation!(mul,"mulss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
operation!(add,"addss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
operation!(sub,"subss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
operation!(div,"divss {x}, {y}");
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn mul(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn add(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn sub(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn div(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
fn finite(v:u32)->bool { f32::from_bits(v).is_finite() }

pub(super) fn project(row:&NativeCellPart,actor:Transform)->Result<Projected,Error> {
    project_side(row,actor,false)
}
pub(super) fn project_second(row:&NativeCellPart,actor:Transform)->Result<Projected,Error> {
    project_side(row,actor,true)
}
fn project_side(row:&NativeCellPart,actor:Transform,second:bool)->Result<Projected,Error> {
    spore_core::cell_feature_projection_v1::project(row,actor,second).map_err(|e|match e {
        spore_core::cell_feature_projection_v1::Error::NonFinite=>Error::NonFinite,
        spore_core::cell_feature_projection_v1::Error::Sse(e)=>Error::Sse(e),
    })
}

/// E6B039..E6B08C only. Neither branch constructs a complete contact here.
#[derive(Debug,PartialEq,Eq)]
pub(super) struct PairPrefix { pub delta:[u32;3],pub distance_squared:u32,pub radius_sum:u32,pub small_distance:bool }
pub(super) fn pair_prefix(first:Projected,second:Projected)->Result<PairPrefix,Error> {
    if first.center.iter().chain(second.center.iter()).any(|&v|!finite(v)) || !finite(first.radius) || !finite(second.radius) { return Err(Error::NonFinite); }
    native_sse::scoped(Policy::Mxcsr1f80,||{
        let x=sub(second.center[0],first.center[0])?;
        let y=sub(second.center[1],first.center[1])?;
        let yy=mul(y,y)?;let xx=mul(x,x)?;let yx=add(yy,xx)?;
        let z=sub(second.center[2],first.center[2])?;
        let zz=mul(z,z)?;let distance_squared=add(yx,zz)?;
        let radius_sum=add(second.radius,first.radius)?;
        Ok(PairPrefix {delta:[x,y,z],distance_squared,radius_sum,
            small_distance:f32::from_bits(distance_squared)<f32::from_bits(0x37800000)})
    })
}

/// Checked continuation; carried small-branch locals are opaque caller data.
/// This stops before E6A450 and never asserts a contact or callback.
#[derive(Debug,PartialEq,Eq)]
pub(super) enum Geometry {
    Small { depth:u32, carried:Option<[u32;6]> },
    Separated { distance:u32,depth:u32 },
    Nondegenerate { distance:u32,depth:u32,normal:[u32;3],point:[u32;3] },
}
pub(super) fn continue_pair(first:Projected,second:Projected,carried:Option<[u32;6]>)->Result<Geometry,Error> {
    if f32::from_bits(first.radius)<0. || f32::from_bits(second.radius)<0. {return Err(Error::NegativeRadius)}
    let p=pair_prefix(first,second)?;
    if p.small_distance {return Ok(Geometry::Small{depth:p.radius_sum,carried})}
    let ctx=Context::from_control_word(0x027f)?;
    let distance=ctx.store_f32(ctx.sqrt(Extended::from_f32(f32::from_bits(p.distance_squared))?)?)?.to_bits();
    native_sse::scoped(Policy::Mxcsr1f80,||{
        let depth=sub(p.radius_sum,distance)?;
        if f32::from_bits(depth)<0. {return Ok(Geometry::Separated{distance,depth})}
        let inv=div(1f32.to_bits(),distance)?;
        let nx=mul(inv,p.delta[0])?;let ny=mul(inv,p.delta[1])?;let nz=mul(p.delta[2],inv)?;
        let dr=sub(first.radius,second.radius)?;
        let px=mul(add(add(second.center[0],first.center[0])?,mul(nx,dr)?)?,0.5f32.to_bits())?;
        let py=mul(add(add(first.center[1],second.center[1])?,mul(ny,dr)?)?,0.5f32.to_bits())?;
        let pz=mul(add(add(first.center[2],second.center[2])?,mul(nz,dr)?)?,0.5f32.to_bits())?;
        Ok(Geometry::Nondegenerate{distance,depth,normal:[nx,ny,nz],point:[px,py,pz]})
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bits(x:f32)->u32{x.to_bits()}
    fn row()->NativeCellPart {NativeCellPart{word_06_is_ffff:false,class:2,center:[3.,4.,5.],matrix_bits:[0;9],radius:2.,tail_bits:[0;3]}}
    #[test] fn direct_transform_and_nonfinite_admission() {
        let actor=Transform{position:[bits(10.),bits(20.),bits(30.)],scale:bits(2.),flags:0,matrix:[f32::NAN.to_bits();9]};
        assert_eq!(project(&row(),actor).unwrap(),Projected{center:[bits(16.),bits(28.),bits(40.)],radius:bits(4.)});
        assert_eq!(project(&row(),Transform{flags:2,..actor}),Err(Error::NonFinite));
    }
    #[test] fn rotated_y_uses_feature_order() {
        let mut part=row();part.center=[3.,1e20,-1e20];
        let mut matrix=[bits(0.);9]; matrix[1]=bits(1.);matrix[4]=bits(1.);matrix[7]=bits(1.);
        let actor=Transform{position:[bits(0.);3],scale:bits(1.),flags:2,matrix};
        assert_eq!(project(&part,actor).unwrap().center[1],bits(3.));
        assert_eq!(project_second(&part,actor).unwrap().center[1],bits(3.));
    }
    #[test] fn pair_prefix_keeps_small_and_threshold_branches_pending() {
        let a=Projected{center:[bits(0.);3],radius:bits(2.)};
        let b=Projected{center:[bits(3.),bits(4.),bits(0.)],radius:bits(1.)};
        assert_eq!(pair_prefix(a,b).unwrap(),PairPrefix{delta:[bits(3.),bits(4.),bits(0.)],distance_squared:bits(25.),radius_sum:bits(3.),small_distance:false});
        assert!(pair_prefix(a,a).unwrap().small_distance);
        // sqrt(2^-16) is 2^-8, so this exact squared distance equals the threshold.
        let boundary=Projected{center:[bits(0.00390625),bits(0.),bits(0.)],radius:bits(1.)};
        assert!(!pair_prefix(a,boundary).unwrap().small_distance);
        // Source rounds (y² + x²) before adding z².
        let rounding=Projected{center:[bits(4096.),bits(1.),bits(1.)],radius:bits(1.)};
        assert_eq!(pair_prefix(a,rounding).unwrap().distance_squared,bits(16777216.));
    }
    #[test] fn continuation_preserves_small_locals_and_tangent_and_separation() {
        let a=Projected{center:[bits(0.);3],radius:bits(0.5)};
        let carried=[0x7fc01234,1,2,3,4,5];
        assert_eq!(continue_pair(a,a,Some(carried)).unwrap(),Geometry::Small{depth:bits(1.),carried:Some(carried)});
        assert_eq!(continue_pair(a,a,None).unwrap(),Geometry::Small{depth:bits(1.),carried:None});
        let b=Projected{center:[bits(1.),bits(0.),bits(0.)],radius:bits(0.5)};
        assert_eq!(continue_pair(a,b,None).unwrap(),Geometry::Nondegenerate{distance:bits(1.),depth:bits(0.),normal:[bits(1.),bits(0.),bits(0.)],point:[bits(0.5),bits(0.),bits(0.)]});
        let small=Projected{radius:bits(0.25),..a};let far=Projected{radius:bits(0.25),..b};
        assert_eq!(continue_pair(small,far,None).unwrap(),Geometry::Separated{distance:bits(1.),depth:bits(-0.5)});
        let unequal=Projected{radius:bits(1.),..a};
        assert_eq!(continue_pair(unequal,b,None).unwrap(),Geometry::Nondegenerate{distance:bits(1.),depth:bits(0.5),normal:[bits(1.),bits(0.),bits(0.)],point:[bits(0.75),bits(0.),bits(0.)]});
        assert_eq!(continue_pair(Projected{radius:bits(-1.),..a},b,None),Err(Error::NegativeRadius));
    }
}
