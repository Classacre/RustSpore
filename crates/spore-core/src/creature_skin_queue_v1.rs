//! Bounded 53EFE0 queue traversal; source finalization and editor mesh omitted.
use crate::{creature_skin_corner_lookup_v1::corner_hash,creature_skin_corner_sample_v1 as corner,creature_skin_cube_walk_v1 as cube,creature_skin_edge_diagnostic_v1 as diagnostic,native_sse::Policy};
const CAP:usize=0x4000;
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Entry{pub packed:u32,pub faces:u8,pub known:u8,pub class:u8}
#[derive(Debug)]pub enum Error{Capacity,Corner,Field,Status,StepLimit,Edge(diagnostic::Error)}
#[derive(Debug,PartialEq,Eq)]pub enum Step{Processed,Drained}
/// Fresh synthetic worker state. Queue/corner/edge storage is nonaliasing;
/// source receiver lifetime, callback success/finite behavior and FP controls
/// are admitted. Failure can retain earlier writes and callback effects.
pub struct Queue {pub entries:Vec<Entry>,pub cursor:usize,pub keys:Vec<u32>,pub indices:Vec<u32>,pub geometry:diagnostic::Diagnostic,pub triangles:Vec<[u32;3]>,capacity:usize}
fn add(p:u32,d:&[u8])->u32{let mut v=0;for i in 0..3{v|=((((p>>(i*8))&255)+(d[i] as u32))&255)<<(i*8);}v}
fn translate(face:&[u8;32],bits:u8)->u8{let value=bits&face[12];let shift=1<<(face[0]/2);if face[0]&1==0{value<<shift}else{value>>shift}}
fn lookup(keys:&[u32],key:u32)->u32{let h=corner_hash(key);let mut i=(h&0x3fff) as usize;let stride=(((h>>16)&0x3fff)|1) as usize;if keys[i]>>24==255||keys[i]==key{return i as u32}let mut n=1;loop{i=(i+stride)&0x3fff;n+=1;if keys[i]==key||keys[i]>>24==255{return i as u32}if n>0x4000{return u32::MAX}}}
impl Queue{
 /// `seed` is an explicit source-style packed seed, e.g. from seed_v1.
 /// Native seed insertion uses cache slot zero irrespective of its hash.
 pub fn from_seed(seed:u32,capacity:usize,corners:usize,positions:usize)->Result<Self,Error>{if capacity==0||capacity>CAP||seed>>24!=0{return Err(Error::Capacity)}let mut keys=vec![u32::MAX;CAP];let mut indices=vec![0;CAP];keys[0]=seed;indices[0]=0;Ok(Self{entries:vec![Entry{packed:seed,..Entry::default()}],cursor:0,keys,indices,geometry:diagnostic::Diagnostic::new(corners,positions),triangles:Vec::new(),capacity})}
 /// Run at most `max_items` queued cubes. A drained queue is not finalized
 /// source geometry; budget exhaustion retains all preceding state and output.
 pub fn run<F:FnMut([f32;3])->f32>(&mut self,tables:&cube::Tables,faces:&[[u8;32];6],origin:[f32;3],step:f32,threshold:f32,limit:f32,policy:Policy,max_items:usize,mut field:F)->Result<usize,Error>{
  let mut done=0;loop{if self.cursor==self.entries.len(){return Ok(done)}if done==max_items{return Err(Error::StepLimit)}match self.step(tables,faces,origin,step,threshold,limit,policy,|p|field(p))?{Step::Processed=>done+=1,Step::Drained=>return Ok(done)}}
 }
 /// Processes exactly one pending cube; no native completion/finalization work.
 /// An explicit field and query snapshot must stay stable through every call.
 pub fn step<F:FnMut([f32;3])->f32>(&mut self,tables:&cube::Tables,faces:&[[u8;32];6],origin:[f32;3],step:f32,threshold:f32,limit:f32,policy:Policy,mut field:F)->Result<Step,Error>{
  if faces!=&cube::face_records()||self.keys.len()!=CAP||self.indices.len()!=CAP||self.cursor>self.entries.len()||self.entries.len()>self.capacity{return Err(Error::Capacity)}
  if self.cursor==self.entries.len(){return Ok(Step::Drained)}let current=self.cursor;self.cursor+=1;let mut entry=self.entries[current];
  if entry.known!=255 {for i in 0..8u8{let bit=1u8<<i;if entry.known&bit==0{let offsets=[i&1,(i>>1)&1,(i>>2)&1];let p=add(entry.packed,&offsets);let mut out=[0;2];let mut invalid=false;corner::sample(p,&mut self.geometry.corner_keys,&mut self.geometry.corner_values,&mut self.geometry.records,&mut self.geometry.status,&mut self.geometry.corner_count,&mut out,origin,step,policy,|p,o|{let v=field(p);if !v.is_finite(){invalid=true}o[0]=v.to_bits()}).map_err(|_|Error::Corner)?;if invalid||!f32::from_bits(out[0]).is_finite(){return Err(Error::Field)}if f32::from_bits(out[0])<0.{entry.class|=bit;}}}entry.known=255;self.entries[current].known=255;self.entries[current].class=entry.class;}
  for (i,face) in faces.iter().enumerate(){let bit=1u8<<i;if entry.faces&bit!=0{continue}let neighbour=add(entry.packed,&face[8..11]);let slot=lookup(&self.keys,neighbour);let hit=slot<0x4000&&self.keys[slot as usize]==neighbour;
   if hit{let index=self.indices[slot as usize] as usize;if index>=self.entries.len(){return Err(Error::Capacity)}if self.cursor<=index{let target=&mut self.entries[index];target.faces|=face[3];target.known|=translate(face,entry.known&face[12]);target.class|=translate(face,entry.known&entry.class);}entry.faces|=bit;}
   else {self.geometry.status|=slot&0xffffc000;let selected=entry.class&face[12];if selected!=0&&selected!=face[12]{if self.entries.len()>=self.capacity{return Err(Error::Capacity)}let index=self.entries.len();self.keys[(slot&0x3fff) as usize]=neighbour;self.indices[(slot&0x3fff) as usize]=index as u32;self.entries.push(Entry{packed:neighbour,faces:face[3],known:face[13],class:translate(face,entry.class)});entry.faces|=bit;}}
  }
  self.entries[current].faces=entry.faces;if self.geometry.status!=0{return Err(Error::Status)}
  self.geometry.cube(tables,entry.packed,entry.class,&mut self.triangles,origin,step,threshold,limit,policy,|p|field(p)).map_err(Error::Edge)?;Ok(Step::Processed)
 }
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn empty_class_drains_without_neighbours(){let mut q=Queue::from_seed(0,16,32,32).unwrap();let t=cube::tables();let f=cube::face_records();assert_eq!(q.step(&t,&f,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|_|1.).unwrap(),Step::Processed);assert_eq!(q.entries.len(),1);assert_eq!((q.entries[0].known,q.entries[0].class),(255,0));assert_eq!(q.step(&t,&f,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|_|panic!()).unwrap(),Step::Drained);}
 #[test]fn mixed_face_enqueues_and_propagates(){let mut q=Queue::from_seed(0,16,64,64).unwrap();let t=cube::tables();let f=cube::face_records();q.step(&t,&f,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|p|p[0]-0.6).unwrap();assert!(q.entries.len()>1);assert!(q.entries[1..].iter().all(|e|e.known!=0&&e.faces!=0));assert!(!q.triangles.is_empty());}
 #[test]fn capacity_failure_keeps_prior_corner_work(){let mut q=Queue::from_seed(0,1,16,16).unwrap();let t=cube::tables();let f=cube::face_records();assert!(matches!(q.step(&t,&f,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|p|p[0]-0.6),Err(Error::Capacity)));assert_eq!(q.cursor,1);assert_eq!(q.geometry.corner_count,8);assert_eq!(q.entries[0].known,255);}
 #[test]fn pending_neighbour_receives_masks_but_consumed_does_not(){let t=cube::tables();let f=cube::face_records();let mut q=Queue::from_seed(0,8,8,8).unwrap();q.entries[0].known=255;let neighbour=255;let slot=lookup(&q.keys,neighbour) as usize;q.keys[slot]=neighbour;q.indices[slot]=1;q.entries.push(Entry{packed:neighbour,..Entry::default()});q.step(&t,&f,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|_|panic!()).unwrap();assert_eq!((q.entries[1].faces,q.entries[1].known,q.entries[1].class),(2,0xaa,0));let mut q=Queue::from_seed(neighbour,8,8,8).unwrap();let slot=lookup(&q.keys,neighbour) as usize;q.keys[slot]=neighbour;q.indices[slot]=0;q.entries.push(Entry{packed:0,known:255,..Entry::default()});q.cursor=1;q.step(&t,&f,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|_|panic!()).unwrap();assert_eq!(q.entries[0],Entry{packed:neighbour,..Entry::default()});assert_ne!(q.entries[1].faces&1,0);}
 #[test]fn seeded_sphere_runs_multiple_cubes_and_drains(){let origin=[-2.;3];let field=|p:[f32;3]|1.5-((p[0]*p[0]+p[1]*p[1])+p[2]*p[2]);let groups=[[[0.;3];4]];let seed=crate::creature_skin_seed_v1::seed([4.;3],origin,1.,1.,&groups,1,Policy::Mxcsr1f80,field).unwrap();let mut q=Queue::from_seed(seed,256,1024,1024).unwrap();let done=q.run(&cube::tables(),&cube::face_records(),origin,1.,0.01,0.001,Policy::Mxcsr1f80,256,field).unwrap();assert!(done>1);assert_eq!(done,q.entries.len());assert_eq!(q.cursor,q.entries.len());assert!(!q.triangles.is_empty());assert!(q.triangles.iter().flatten().all(|&x|(x as usize)<q.geometry.position_count));}
 #[test]fn work_budget_preserves_pending_items(){let mut q=Queue::from_seed(0,16,64,64).unwrap();let e=q.run(&cube::tables(),&cube::face_records(),[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,1,|p|p[0]-0.6);assert!(matches!(e,Err(Error::StepLimit)));assert_eq!(q.cursor,1);assert!(q.entries.len()>q.cursor);}
 #[test]fn full_lookup_returns_sentinel(){let keys=vec![0;CAP];assert_eq!(lookup(&keys,1),u32::MAX);}
}
