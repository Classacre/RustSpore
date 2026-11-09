//! Checked owned projection of Cell spatial-tree construction and updates.
//! Explicit supplied geometry and floating contexts only; no live actor, pool,
//! allocation address, cache, renderer, input, or selection filter is implied.
//! Atomic errors, monotonic Rust IDs and storage limits are boundary policies,
//! not native allocator/fault semantics. Native conformance remains pending.
use crate::{cell_actor_geometry_projection_v1::Output as Geometry,
    cell_pick_tree_v1::{Kind,Node},native_sse::{self,Policy,SseError},
    native_x87::{Context,Extended,X87Error}};
use std::cmp::Ordering;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)] pub struct LeafId(usize);
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error {
    InvalidLeaf, InvalidTree, Capacity, NonFiniteInput, NonFiniteIntermediate,
    Sse(SseError), X87(X87Error),
}
impl From<SseError> for Error { fn from(e:SseError)->Self {Self::Sse(e)} }
impl From<X87Error> for Error { fn from(e:X87Error)->Self {Self::X87(e)} }
#[derive(Clone,Copy,Debug)] enum Shape { Leaf(u32), Branch(usize,usize) }
#[derive(Clone,Copy,Debug)] struct Entry { geom:Geometry, parent:Option<usize>, shape:Shape }
#[derive(Clone,Debug)] pub struct Tree { slots:Vec<Option<Entry>>,root:Option<usize>,pending:Vec<usize>, max_live:usize }
#[derive(Clone,Copy,Debug)] pub struct Environment {pub x87:Context,pub sse:Policy}

#[cfg(any(target_arch="x86",target_arch="x86_64"))]
macro_rules! op { ($name:ident,$ins:literal) => { fn $name(a:u32,b:u32)->Result<u32,Error>{
 let out:u32; unsafe {std::arch::asm!("movd {x}, {a:e}","movd {y}, {b:e}",$ins,"movd {out:e}, {x}",
 x=out(xmm_reg) _,y=out(xmm_reg) _,a=in(reg)a,b=in(reg)b,out=lateout(reg)out,options(nostack,preserves_flags));}
 if f32::from_bits(out).is_finite(){Ok(out)}else{Err(Error::NonFiniteIntermediate)}
} }; }
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(add,"addss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(sub,"subss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(mul,"mulss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))] op!(div,"divss {x}, {y}");
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
macro_rules! unsupported { ($($n:ident),*)=>{$(fn $n(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))})*}; }
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))] unsupported!(add,sub,mul,div);
fn f(v:u32)->f32 {f32::from_bits(v)}
fn checked(g:Geometry)->Result<(),Error> {if g.center.iter().chain([&g.scalar]).any(|&v|!f(v).is_finite()){Err(Error::NonFiniteInput)}else{Ok(())}}
fn ext(v:u32)->Result<Extended,Error>{Ok(Extended::from_f32(f(v))?)}

// E86C20: retained x87 differences for distance, FST32 copies for direction.
fn merge(a:Geometry,b:Geometry,c:Context)->Result<Geometry,Error> {
 let d=[c.sub(ext(b.center[0])?,ext(a.center[0])?)?,c.sub(ext(b.center[1])?,ext(a.center[1])?)?,c.sub(ext(b.center[2])?,ext(a.center[2])?)?];
 let committed=[c.store_f32(d[0])?.to_bits(),c.store_f32(d[1])?.to_bits(),c.store_f32(d[2])?.to_bits()];
 let xx=c.mul(d[0],d[0])?;let yy=c.mul(d[1],d[1])?;let zz=c.mul(d[2],d[2])?;
 let len=c.store_f32(c.sqrt(c.add(zz,c.add(xx,yy)?)?)?)?.to_bits();
 if !f(len).is_finite()||committed.iter().any(|&v|!f(v).is_finite()){return Err(Error::NonFiniteIntermediate);}
 let lr1=add(len,a.scalar)?;
 if f(b.scalar)>f(lr1){return Ok(b);}
 if f(a.scalar)>f(add(len,b.scalar)?){return Ok(a);}
 let half=0.5f32.to_bits();let radius=mul(add(lr1,b.scalar)?,half)?;
 let center=if f(len)<f(f32::from_bits(0x37800000).to_bits()) {
  [mul(add(a.center[0],b.center[0])?,half)?,mul(add(b.center[1],a.center[1])?,half)?,mul(add(a.center[2],b.center[2])?,half)?]
 } else {
  let inv=div(1f32.to_bits(),len)?;let diff=sub(b.scalar,a.scalar)?;
  let tx=mul(mul(inv,committed[0])?,diff)?;let ty=mul(mul(inv,committed[1])?,diff)?;let tz=mul(mul(inv,committed[2])?,diff)?;
  [mul(add(add(a.center[0],b.center[0])?,tx)?,half)?,mul(add(add(b.center[1],a.center[1])?,ty)?,half)?,mul(add(add(a.center[2],b.center[2])?,tz)?,half)?]
 };
 Ok(Geometry{center,scalar:radius})
}

// E86EF0: the first child has SSE-subtracted deltas, the second x87 deltas.
fn child_outside(parent:Geometry,child:Geometry,first:bool,c:Context)->Result<bool,Error>{
 let mut d=[Extended::from_i32(0);3];
 for (i,slot) in d.iter_mut().enumerate(){*slot=if first {ext(sub(child.center[i],parent.center[i])?)?} else {c.sub(ext(child.center[i])?,ext(parent.center[i])?)?};}
 let x=c.mul(d[0],d[0])?;let y=c.mul(d[1],d[1])?;let z=c.mul(d[2],d[2])?;
 let sum=c.add(c.add(z,y)?,x)?;let reach=c.add(c.sqrt(sum)?,ext(child.scalar)?)?;
 Ok(c.compare(reach,ext(parent.scalar)?)?==Ordering::Greater)
}
fn update_outside(parent:Geometry,child:Geometry,c:Context)->Result<bool,Error>{
 let x=c.sub(ext(child.center[0])?,ext(parent.center[0])?)?;let y=c.sub(ext(child.center[1])?,ext(parent.center[1])?)?;let z=c.sub(ext(child.center[2])?,ext(parent.center[2])?)?;
 let xx=c.mul(x,x)?;let yy=c.mul(y,y)?;let zz=c.mul(z,z)?;
 let len=c.sqrt(c.add(c.add(zz,xx)?,yy)?)?;
 Ok(c.compare(c.add(len,ext(child.scalar)?)?,ext(parent.scalar)?)?==Ordering::Greater)
}

impl Tree {
 pub fn new(max_live:usize)->Self {Self{slots:Vec::new(),root:None,pending:Vec::new(),max_live}}
 fn get(&self,i:usize)->Result<Entry,Error>{self.slots.get(i).and_then(|x|*x).ok_or(Error::InvalidTree)}
 fn set(&mut self,i:usize,e:Entry)->Result<(),Error>{*self.slots.get_mut(i).ok_or(Error::InvalidTree)?=Some(e);Ok(())}
 fn allocate(&mut self,e:Entry)->Result<usize,Error>{if self.slots.iter().filter(|x|x.is_some()).count()>=self.max_live{return Err(Error::Capacity)};let i=self.slots.len();self.slots.push(Some(e));Ok(i)}
 fn atomic<T>(&mut self,env:Environment,work:impl FnOnce(&mut Self)->Result<T,Error>)->Result<T,Error>{
  native_sse::scoped(env.sse,||{let mut next=self.clone();let out=work(&mut next)?;*self=next;Ok(out)})
 }
 /// Queue one leaf; `geometry` can come directly from E6D7F0's checked projection.
 pub fn insert(&mut self,geometry:Geometry,handle:u32,env:Environment)->Result<LeafId,Error>{
  self.atomic(env,|s|{checked(geometry)?;let i=s.allocate(Entry{geom:geometry,parent:None,shape:Shape::Leaf(handle)})?;s.pending.push(i);Ok(LeafId(i))})
 }
 /// Drain queued leaves in source LIFO order and preserve child tie behavior.
 pub fn flush(&mut self,env:Environment)->Result<(),Error>{self.atomic(env,|s|s.flush_inner(env.x87))}
 fn flush_inner(&mut self,c:Context)->Result<(),Error>{
  while let Some(new)=self.pending.pop(){
   let root=match self.root{Some(v)=>v,None=>{self.root=Some(new);continue}};
   let mut chosen=root;
   loop {let e=self.get(chosen)?;match e.shape {Shape::Leaf(_)=>break,Shape::Branch(a,b)=>{
    let p=self.get(new)?.geom.center;let aa=self.get(a)?.geom.center;let bb=self.get(b)?.geom.center;
    // First sums y²+x² then z²; second sums z²+y² then x².
    let distance=|v:[u32;3],first:bool|->Result<u32,Error>{let x=sub(v[0],p[0])?;let y=sub(v[1],p[1])?;let z=sub(v[2],p[2])?;
     if first {add(add(mul(y,y)?,mul(x,x)?)?,mul(z,z)?)} else {add(add(mul(z,z)?,mul(y,y)?)?,mul(x,x)?)}};
    chosen=if f(distance(bb,false)?)>f(distance(aa,true)?){a}else{b};
   }}}
   let old=self.get(chosen)?;let mut ng=merge(self.get(new)?.geom,old.geom,c)?;
   ng.scalar=mul(ng.scalar,0x3f8ccccd)?;
   let parent=self.allocate(Entry{geom:ng,parent:old.parent,shape:Shape::Branch(new,chosen)})?;
   let mut n=self.get(new)?;n.parent=Some(parent);self.set(new,n)?;
   let mut o=old;o.parent=Some(parent);self.set(chosen,o)?;
   if let Some(ancestor)=old.parent {let mut a=self.get(ancestor)?;a.shape=match a.shape{Shape::Branch(x,y) if x==chosen=>Shape::Branch(parent,y),Shape::Branch(x,y) if y==chosen=>Shape::Branch(x,parent),_=>return Err(Error::InvalidTree)};self.set(ancestor,a)?;self.refit(ancestor,c)?;} else {self.root=Some(parent);}
  } Ok(())
 }
 fn refit(&mut self,mut i:usize,c:Context)->Result<(),Error>{loop{
  let mut p=self.get(i)?;let (a,b)=match p.shape{Shape::Branch(a,b)=>(a,b),_=>return Err(Error::InvalidTree)};
  let ga=self.get(a)?.geom;let gb=self.get(b)?.geom;
  if !child_outside(p.geom,ga,true,c)? && !child_outside(p.geom,gb,false,c)?{return Ok(());}
  p.geom=merge(ga,gb,c)?;self.set(i,p)?;match p.parent{Some(x)=>i=x,None=>return Ok(())}
 }}
 fn detach(&mut self,i:usize)->Result<(),Error>{
  let mut leaf=self.get(i)?;let p=leaf.parent.ok_or(Error::InvalidTree)?;let parent=self.get(p)?;
  let sibling=match parent.shape{Shape::Branch(a,b) if a==i=>b,Shape::Branch(a,b) if b==i=>a,_=>return Err(Error::InvalidTree)};
  let mut sib=self.get(sibling)?;sib.parent=parent.parent;self.set(sibling,sib)?;
  if let Some(g)=parent.parent{let mut grand=self.get(g)?;grand.shape=match grand.shape{Shape::Branch(a,b) if a==p=>Shape::Branch(sibling,b),Shape::Branch(a,b) if b==p=>Shape::Branch(a,sibling),_=>return Err(Error::InvalidTree)};self.set(g,grand)?;}else{self.root=Some(sibling)}
  self.slots[p]=None;leaf.parent=None;self.set(i,leaf)?;Ok(())
 }
 /// E870B0's supplied leaf update; outside-parent leaves are requeued.
 pub fn update(&mut self,id:LeafId,geometry:Geometry,env:Environment)->Result<(),Error>{self.atomic(env,|s|{
  checked(geometry)?;let mut e=s.get(id.0).map_err(|_|Error::InvalidLeaf)?;
  if !matches!(e.shape,Shape::Leaf(_)){return Err(Error::InvalidLeaf)};e.geom=geometry;s.set(id.0,e)?;
  if let Some(p)=e.parent{if update_outside(s.get(p)?.geom,geometry,env.x87)?{s.detach(id.0)?;s.pending.push(id.0);}}Ok(())
 })}
 /// Remove a known leaf from root, pending list or attached tree.
 pub fn remove(&mut self,id:LeafId,env:Environment)->Result<(),Error>{self.atomic(env,|s|{
  let e=s.get(id.0).map_err(|_|Error::InvalidLeaf)?;if !matches!(e.shape,Shape::Leaf(_)){return Err(Error::InvalidLeaf)}
  if s.root==Some(id.0){s.root=None;}else if e.parent.is_some(){s.detach(id.0)?;}else if let Some(i)=s.pending.iter().position(|&x|x==id.0){s.pending.remove(i);}else{return Err(Error::InvalidTree)};
  s.slots[id.0]=None;Ok(())
 })}
 /// Return a traversable snapshot of the already-flushed portion. Pending
 /// entries remain separate until `flush`, just as before the source query.
 pub fn snapshot(&self)->Result<(Vec<Node>,Option<usize>),Error>{
  let mut map=vec![None;self.slots.len()];let mut nodes=Vec::new();
  for (i,e) in self.slots.iter().enumerate(){if let Some(e)=e {map[i]=Some(nodes.len());nodes.push(Node{geometry:e.geom,kind:Kind::Leaf(0)});}}
  for (i,e) in self.slots.iter().enumerate(){if let Some(e)=e{let k=map[i].ok_or(Error::InvalidTree)?;nodes[k].kind=match e.shape{Shape::Leaf(h)=>Kind::Leaf(h),Shape::Branch(a,b)=>Kind::Branch{first:map.get(a).and_then(|x|*x).ok_or(Error::InvalidTree)?,second:map.get(b).and_then(|x|*x).ok_or(Error::InvalidTree)?}};}}
  Ok((nodes,match self.root{Some(i)=>Some(map[i].ok_or(Error::InvalidTree)?),None=>None}))
 }
 /// Pending handles in the source list's head-to-tail order.
 pub fn pending_handles(&self)->Result<Vec<u32>,Error>{self.pending.iter().rev().map(|&i|match self.get(i)?.shape{Shape::Leaf(h)=>Ok(h),_=>Err(Error::InvalidTree)}).collect()}
}

#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests {
 use super::*;
 use crate::{cell_pick_tree_v1::{self,Query},native_x87::{Precision,Rounding}};
 fn env()->Environment{Environment{x87:Context{precision:Precision::Bits64,rounding:Rounding::NearestEven},sse:Policy::Mxcsr1f80}}
 fn g(x:f32,r:f32)->Geometry{Geometry{center:[x.to_bits(),0,0],scalar:r.to_bits()}}
 fn handles(t:&Tree)->Vec<u32>{let(n,root)=t.snapshot().unwrap();cell_pick_tree_v1::traverse(&n,root,Query{origin:[0;3],direction:[1f32.to_bits(),0,0],limit:100f32.to_bits(),capacity:512,x87:env().x87,sse:env().sse}).unwrap()}
 #[test] fn pending_lifo_and_ordered_children(){
  let mut t=Tree::new(8);t.insert(g(2.,1.),2,env()).unwrap();t.insert(g(4.,1.),4,env()).unwrap();
  assert!(handles(&t).is_empty());t.flush(env()).unwrap();assert_eq!(handles(&t),vec![2,4]);
 }
 #[test] fn update_requeues_and_remove_handles_root_and_pending(){
  let mut t=Tree::new(16);let a=t.insert(g(2.,1.),2,env()).unwrap();let b=t.insert(g(4.,1.),4,env()).unwrap();t.flush(env()).unwrap();
  t.update(a,g(30.,1.),env()).unwrap();assert_eq!(handles(&t),vec![4]);t.flush(env()).unwrap();assert_eq!(handles(&t),vec![2,4]);
  t.remove(a,env()).unwrap();assert_eq!(handles(&t),vec![4]);t.remove(b,env()).unwrap();assert!(handles(&t).is_empty());
  let c=t.insert(g(0.,1.),9,env()).unwrap();t.remove(c,env()).unwrap();t.flush(env()).unwrap();assert!(handles(&t).is_empty());
 }
 #[test] fn capacity_and_invalid_geometry_are_atomic(){
  let mut t=Tree::new(2);let a=t.insert(g(1.,1.),1,env()).unwrap();t.insert(g(2.,1.),2,env()).unwrap();
  assert_eq!(t.flush(env()),Err(Error::Capacity));assert!(handles(&t).is_empty());
  assert_eq!(t.update(a,g(f32::NAN,1.),env()),Err(Error::NonFiniteInput));assert!(handles(&t).is_empty());
  t.remove(a,env()).unwrap();t.flush(env()).unwrap();assert_eq!(handles(&t),vec![2]);
 }
 #[test] fn merge_containment_tie_and_coincident_centers(){
  assert_eq!(merge(g(0.,5.),g(1.,1.),env().x87).unwrap(),g(0.,5.));
  assert_eq!(merge(g(0.,1.),g(0.,1.),env().x87).unwrap(),g(0.,1.));
  assert_eq!(merge(g(0.,1.),g(2.,1.),env().x87).unwrap(),g(1.,2.));
 }
}
