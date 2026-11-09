//! Supplied-tree projection of E87170 and its 5A9C40 ray/sphere helper.
//!
//! This is a checked finite-domain model, not an admitted live spatial tree,
//! actor lookup, selection filter, source execution or gameplay integration.
//! Child order is significant; no distance sorting or tree construction occurs.
//! Tests are analytic model checks and are not a native conformance comparison.
use crate::{cell_actor_geometry_projection_v1::Output as Geometry,
    native_sse::{self, Policy, SseError}, native_x87::{Context, Extended, X87Error}};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { Leaf(u32), Branch { first: usize, second: usize } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node { pub geometry: Geometry, pub kind: Kind }
#[derive(Clone, Copy, Debug)]
pub struct Query {
    pub origin: [u32; 3], pub direction: [u32; 3], pub limit: u32,
    pub capacity: usize, pub x87: Context, pub sse: Policy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidNode, RepeatedNode, NonFiniteInput, NonFiniteIntermediate,
    Sse(SseError), X87(X87Error) }
impl From<SseError> for Error { fn from(e:SseError)->Self { Self::Sse(e) } }
impl From<X87Error> for Error { fn from(e:X87Error)->Self { Self::X87(e) } }

#[cfg(any(target_arch="x86",target_arch="x86_64"))]
macro_rules! op { ($name:ident,$instruction:literal) => {
    fn $name(a:u32,b:u32)->Result<u32,Error> {
        let out:u32;
        // SAFETY: invoked only inside SSE2-checked scoped MXCSR1F80.
        unsafe { std::arch::asm!("movd {x}, {a:e}","movd {y}, {b:e}",$instruction,"movd {out:e}, {x}",
            x=out(xmm_reg) _,y=out(xmm_reg) _,a=in(reg)a,b=in(reg)b,
            out=lateout(reg)out,options(nostack,preserves_flags)); }
        if f32::from_bits(out).is_finite() { Ok(out) } else { Err(Error::NonFiniteIntermediate) }
    }
}; }
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(add,"addss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(sub,"subss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(mul,"mulss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(div,"divss {x}, {y}");
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
macro_rules! unsupported { ($($name:ident),*) => { $(fn $name(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))})* }; }
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))] unsupported!(add,sub,mul,div);

fn finite(x:u32)->bool { f32::from_bits(x).is_finite() }
fn f(x:u32)->f32 { f32::from_bits(x) }

// Native 5A9C40, admitted finite domain. Return the committed parameter only
// on success. The first root is committed while the other remains x87.
fn ray_sphere(q:&Query, g:Geometry)->Result<Option<u32>,Error> {
    let [x,y,z]=q.direction;
    let dx=sub(q.origin[0],g.center[0])?;
    let dy=sub(q.origin[1],g.center[1])?;
    let dz=sub(q.origin[2],g.center[2])?;
    let norm=add(add(mul(x,x)?,mul(y,y)?)?,mul(z,z)?)?;
    let dot=add(add(mul(x,dx)?,mul(y,dy)?)?,mul(z,dz)?)?;
    let dist=add(add(mul(dx,dx)?,mul(dy,dy)?)?,mul(dz,dz)?)?;
    let disc=sub(mul(dot,dot)?,mul(sub(dist,mul(g.scalar,g.scalar)?)?,norm)?)?;
    if f(disc)<0.0 { return Ok(None); }
    let c=q.x87;
    let root=c.sqrt(Extended::from_f32(f(disc))?)?;
    let dot_x=Extended::from_f32(f(dot))?;
    let near=c.store_f32(c.sub(dot_x.negated(),root)?)?;
    let far=c.sub(root,dot_x)?;
    if c.compare(Extended::from_i32(0),far)?==Ordering::Greater { return Ok(None); }
    if !near.is_finite() { return Err(Error::NonFiniteIntermediate); }
    if near <= 0.0 { Ok(Some(0)) } else { Ok(Some(div(near.to_bits(),norm)?)) }
}

/// Traverse an explicitly supplied snapshot in native child order. Geometry is
/// directly compatible with `cell_actor_geometry_projection_v1::Output`.
/// Bounds/cycle/finite checks are Rust admission policy, not native behavior.
/// Reached nodes must form a tree; malformed or overflowing reached data errors
/// without returning a partial result. Capacity zero consumes no node inputs.
pub fn traverse(nodes:&[Node],root:Option<usize>,q:Query)->Result<Vec<u32>,Error> {
    native_sse::scoped(q.sse,|| {
        let mut result=Vec::new();
        let mut stack=Vec::new();
        let mut visited=vec![false;nodes.len()];
        if q.capacity==0 { return Ok(result); }
        if let Some(i)=root { stack.push(i); }
        while let Some(i)=stack.pop() {
            let node=*nodes.get(i).ok_or(Error::InvalidNode)?;
            if std::mem::replace(&mut visited[i],true) { return Err(Error::RepeatedNode); }
            let g=node.geometry;
            if q.origin.iter().chain(g.center.iter()).any(|&v|!finite(v)) || !finite(g.scalar) {
                return Err(Error::NonFiniteInput);
            }
            let dx=sub(g.center[0],q.origin[0])?;
            let dy=sub(g.center[1],q.origin[1])?;
            let dz=sub(g.center[2],q.origin[2])?;
            let d2=add(add(mul(dx,dx)?,mul(dy,dy)?)?,mul(dz,dz)?)?;
            let r2=mul(g.scalar,g.scalar)?;
            if f(d2)>f(r2) {
                if q.direction.iter().any(|&v|!finite(v)) || !finite(q.limit) { return Err(Error::NonFiniteInput); }
                match ray_sphere(&q,g)? { None=>continue, Some(t) if f(t)>f(q.limit)=>continue, _=>{} }
            }
            match node.kind {
                Kind::Leaf(handle)=>{ result.push(handle); if result.len()==q.capacity { break; } },
                Kind::Branch{first,second}=>{ stack.push(second); stack.push(first); }
            }
        }
        Ok(result)
    })
}

#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision,Rounding};
    fn bits(a:[f32;3])->[u32;3] { a.map(f32::to_bits) }
    fn node(x:f32,r:f32,kind:Kind)->Node { Node { geometry:Geometry { center:bits([x,0.,0.]),scalar:r.to_bits() },kind } }
    fn query()->Query { Query { origin:bits([0.,0.,0.]),direction:bits([1.,0.,0.]),limit:100f32.to_bits(),capacity:512,x87:Context {precision:Precision::Bits64,rounding:Rounding::NearestEven},sse:Policy::Mxcsr1f80 } }
    #[test] fn ordering_and_capacity() {
        let n=[node(0.,20.,Kind::Branch{first:1,second:2}),node(8.,1.,Kind::Leaf(8)),node(3.,1.,Kind::Leaf(3))];
        assert_eq!(traverse(&n,Some(0),query()).unwrap(),vec![8,3]);
        assert_eq!(traverse(&n,Some(0),Query{capacity:1,..query()}).unwrap(),vec![8]);
        assert_eq!(traverse(&n,Some(99),Query{capacity:0,..query()}).unwrap(),Vec::<u32>::new());
    }
    #[test] fn hit_miss_tangent_limit_and_inside() {
        let leaf=node(3.,1.,Kind::Leaf(7));
        assert_eq!(traverse(&[leaf],Some(0),query()).unwrap(),vec![7]);
        assert!(traverse(&[leaf],Some(0),Query{limit:1f32.to_bits(),..query()}).unwrap().is_empty());
        assert!(traverse(&[node(-3.,1.,Kind::Leaf(7))],Some(0),query()).unwrap().is_empty());
        let tangent=Node{geometry:Geometry{center:bits([3.,1.,0.]),scalar:1f32.to_bits()},kind:Kind::Leaf(2)};
        assert_eq!(traverse(&[tangent],Some(0),query()).unwrap(),vec![2]);
        assert_eq!(traverse(&[node(0.,1.,Kind::Leaf(4))],Some(0),Query{direction:[f32::NAN.to_bits();3],limit:f32::NAN.to_bits(),..query()}).unwrap(),vec![4]);
    }
    #[test] fn checked_tree_and_overflow() {
        let n=[node(0.,1.,Kind::Branch{first:0,second:0})];
        assert_eq!(traverse(&n,Some(0),query()),Err(Error::RepeatedNode));
        assert_eq!(traverse(&[node(f32::MAX,1.,Kind::Leaf(1))],Some(0),query()),Err(Error::NonFiniteIntermediate));
        assert_eq!(traverse(&[],None,query()).unwrap(),Vec::<u32>::new());
    }
}
