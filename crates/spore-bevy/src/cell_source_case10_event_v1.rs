//! Synthetic, inspection-only continuation of the E7E050 selector-10 path.
//! External operations and fresh identity/snapshot admissions remain explicit.
use super::{Case10Boundary,Fixture,Handle};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct EventIdentity { handle:u32, object:u64 }
#[derive(Clone,Copy,Debug)]
struct FreshInputs {
 actors:[Handle;2], second_definition:u32, duration_bits:u32,
 // Current data behind the source's two default pointers. The vector is
 // independently read twice; a caller admits it remains stable for both reads.
 default_vector:[u32;3], default_quaternion:[u32;4],
}
#[derive(Debug,PartialEq,Eq)]
enum Phase { NeedCallback,NeedFloat,PendingE69B00 }
#[derive(Debug)]
struct EventFixture { identity:EventIdentity, bytes:[u8;0x7c], phase:Phase }
fn put(bytes:&mut [u8;0x7c],offset:usize,value:u32){bytes[offset..offset+4].copy_from_slice(&value.to_le_bytes())}
impl EventFixture {
 /// Caller admits relation=0 and completion of the first-actor lookup/E6D260,
 /// including their effects, before supplying fresh actors and duration bits.
 /// Allocation and its first lookup are explicit admissions. Object tokens and
 /// the no-alias/stability conditions are fixture safety rules, not native checks.
 fn initialize(pool:&Fixture<'_>,pending:Case10Boundary,relation:u8,fresh:FreshInputs,allocation:EventIdentity,first_lookup:EventIdentity,mut bytes:[u8;0x7c])->Result<Self,String>{
  let (actors,selected)=match pending {Case10Boundary::PendingRelation{actors,selected_word_0c,..}=>(actors,selected_word_0c),_=>return Err("relation is not pending".into())};
  if relation!=0{return Err("nonzero relation does not create this event".into())}
  if fresh.actors!=actors{return Err("fresh actor identity mismatch".into())}
  for actor in fresh.actors {pool.resolve(actor).ok_or("stale or foreign actor")?;}
  if allocation.object==0||allocation!=first_lookup{return Err("event allocation lookup mismatch".into())}
  for (off,v) in [(0x28,actors[0].raw),(0x24,1),(0x30,selected),(0x2c,actors[1].raw),(0x1c,fresh.duration_bits),(0x20,fresh.duration_bits),(0x38,2),(0x34,u32::MAX),(0x40,0),(0x3c,fresh.second_definition),(0x44,0)]{put(&mut bytes,off,v)}
  for (i,v) in fresh.default_vector.iter().enumerate(){put(&mut bytes,0x48+4*i,*v)}
  for (i,v) in fresh.default_vector.iter().enumerate(){put(&mut bytes,0x54+4*i,*v)}
  for (i,v) in fresh.default_quaternion.iter().enumerate(){put(&mut bytes,0x60+4*i,*v)}
  put(&mut bytes,0x70,0);bytes[0x74]=0;put(&mut bytes,0x78,0);put(&mut bytes,4,0);bytes[8]=0;
  Ok(Self{identity:allocation,bytes,phase:Phase::NeedCallback})
 }
 /// Caller admits the new event-pool lookup resolves the same live object.
 fn install_callback(&mut self,lookup:EventIdentity)->Result<(),String>{
  if self.phase!=Phase::NeedCallback||lookup!=self.identity{return Err("callback lookup or phase mismatch".into())}
  put(&mut self.bytes,4,0xe7dbb0);self.phase=Phase::NeedFloat;Ok(())
 }
 /// The float is a fresh admitted post-callback-setter caller-slot snapshot.
 fn install_float(&mut self,lookup:EventIdentity,float_bits:u32)->Result<(),String>{
  if self.phase!=Phase::NeedFloat||lookup!=self.identity{return Err("float lookup or phase mismatch".into())}
  put(&mut self.bytes,0x0c,float_bits);self.phase=Phase::PendingE69B00;Ok(())
 }
}

/// E69B00 is a separate actor-state path, not event-record dispatch.
/// The checked first lookup and its fresh actor bytes are explicit admissions.
#[derive(Debug,PartialEq,Eq)]
enum E69BPrefix {
 ReturnedFalseFirstMissing,
 ReturnedFalseActor112,
 PendingE57890{first:Handle,selected_word:u32,written_18c:u32},
 PendingSecondLookup{first:Handle,selected_word:u32},
}
impl EventFixture {
 /// The caller supplies current words after E52750 and the pending lookup's
 /// result. Passing None admits B71CC0 returned null; Some must be current in
 /// this owned fixture. The input snapshots are not inferred from event bytes.
 fn e69b_prefix(&self,pool:&Fixture<'_>,first_lookup:Option<Handle>,actor_112:u8,actor_18c:u32,selected_word:u32)->Result<E69BPrefix,String>{
  if self.phase!=Phase::PendingE69B00{return Err("E69B00 not pending".into())}
  let first=match first_lookup {None=>return Ok(E69BPrefix::ReturnedFalseFirstMissing),Some(h)=>h};
  pool.resolve(first).ok_or("stale or foreign first actor")?;
  if actor_112!=0{return Ok(E69BPrefix::ReturnedFalseActor112)}
  // E69C4C byte table: entries indexing the E69B61 arm; type 2 sets 12.
  const WRITING_STATES:[u32;18]=[0,1,2,3,18,19,20,21,22,24,25,26,27,29,40,48,50,51];
  if WRITING_STATES.contains(&actor_18c) {
   Ok(E69BPrefix::PendingE57890{first,selected_word,written_18c:12})
  }else{Ok(E69BPrefix::PendingSecondLookup{first,selected_word})}
 }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum CallbackAdmission {Skipped,Completed}
#[derive(Debug,PartialEq,Eq)]
enum E69BSecond {
 /// B71CC0 returned null; E69B00 returns AL=1 without actor-byte stores.
 ReturnedTrueSecondMissing,
 /// Ordered stores are first +178, second +179, second +17F; 4098A0 pending.
 PendingLocalConstructor{first:Handle,second:Handle,selected_word:u32,ordered_values:[u8;3]},
}
/// The stack handle is freshly supplied after any E57890 effects. A returned
/// Some handle admits a successful new B71CC0 result; None admits null. The
/// retained first pointer must still be valid before the non-null branch writes.
fn e69b_second(pool:&Fixture<'_>,prefix:E69BPrefix,callback:CallbackAdmission,fresh_stack_handle:u32,second_lookup:Option<Handle>)->Result<E69BSecond,String>{
 let (first,selected,expected)=match prefix {
  E69BPrefix::PendingE57890{first,selected_word,..}=>(first,selected_word,CallbackAdmission::Completed),
  E69BPrefix::PendingSecondLookup{first,selected_word}=>(first,selected_word,CallbackAdmission::Skipped),
  _=>return Err("second lookup is not pending".into()),
 };
 if callback!=expected{return Err("E57890 completion admission mismatch".into())}
 let second=match second_lookup {None=>return Ok(E69BSecond::ReturnedTrueSecondMissing),Some(h)=>h};
 if second.raw!=fresh_stack_handle{return Err("fresh second handle mismatch".into())}
 pool.resolve(first).ok_or("retained first actor became stale or foreign")?;
 pool.resolve(second).ok_or("resolved second actor is stale or foreign")?;
 if first==second{return Err("fixture requires distinct actor snapshots".into())}
 Ok(E69BSecond::PendingLocalConstructor{first,second,selected_word:selected,ordered_values:[1,1,1]})
}

#[derive(Clone,Copy,Debug)]
struct LocalDefaults {vector:[u32;3],scalar:u32,matrix:[u32;9]}
#[derive(Debug,PartialEq,Eq)]
struct PendingE68070 {first:Handle,second:Handle,selected_word:u32,local:[u8;0x38]}
/// 4098A0/41CAE0 writes all 56 local bytes before pending E68070. Global
/// snapshots, their stability across ordered reads and nonaliasing are explicit
/// caller admissions; the eventual E68070 effects are not executed here.
fn e69b_local_constructor(pool:&Fixture<'_>,pending:E69BSecond,defaults:LocalDefaults)->Result<PendingE68070,String>{
 let (first,second,selected_word)=match pending {E69BSecond::PendingLocalConstructor{first,second,selected_word,..}=>(first,second,selected_word),_=>return Err("local constructor not pending".into())};
 for actor in [first,second]{pool.resolve(actor).ok_or("stale or foreign actor before local constructor")?;}
 let mut local=[0u8;0x38];
 for (i,v) in defaults.vector.iter().enumerate(){local[4+i*4..8+i*4].copy_from_slice(&v.to_le_bytes())}
 local[0x10..0x14].copy_from_slice(&defaults.scalar.to_le_bytes());
 for (i,v) in defaults.matrix.iter().enumerate(){local[0x14+i*4..0x18+i*4].copy_from_slice(&v.to_le_bytes())}
 Ok(PendingE68070{first,second,selected_word,local})
}

/// Controlled 537EE0 fallback result. Identity and snapshots are fixture
/// admissions; no native helper, cache or live actor is accessed.
#[derive(Debug,PartialEq,Eq)]
struct FallbackResult {local:[u8;0x38],updated:bool}
#[derive(Clone,Copy)]
struct FallbackAdmission {source_actor:Handle,disjoint_stable:bool,x87_control:u16,mxcsr:u32}
fn fallback_537ee0(pool:&Fixture<'_>,pending:PendingE68070,source:[u8;0x38],admission:FallbackAdmission)->Result<FallbackResult,String>{
 if pending.first!=admission.source_actor{return Err("fallback actor identity mismatch".into())}
 for actor in [pending.first,pending.second] {pool.resolve(actor).ok_or("stale or foreign fallback actor")?;}
 if !admission.disjoint_stable{return Err("stable disjoint buffers required".into())}
 if admission.x87_control!=0x027f||admission.mxcsr!=0x1f80{return Err("unsupported FP controls or sticky flags".into())}
 let mut local=pending.local;
 let read16=|b:&[u8;0x38],i:usize|u16::from_le_bytes(b[i..i+2].try_into().unwrap());
 let readf=|b:&[u8;0x38],i:usize|f32::from_bits(u32::from_le_bytes(b[i..i+4].try_into().unwrap()));
 // Restrict all float slots, even skipped matrix slots, to avoid modeling
 // exceptional operand/status behavior and x87 special-value conversion.
 for b in [&local,&source] {for i in (4..0x38).step_by(4){let v=readf(b,i);if !(v.is_normal()||v==0.0){return Err("finite normal or zero required".into())}}}
 let df=read16(&local,0);let sf=read16(&source,0);
 let mut changed=df!=sf;
 if !changed {for i in [4,8,12] {if readf(&local,i)!=readf(&source,i){changed=true;break}}}
 if !changed && readf(&local,0x10)!=readf(&source,0x10){changed=true}
 if !changed && df&2!=0 {for i in (0x14..0x38).step_by(4){if readf(&local,i)!=readf(&source,i){changed=true;break}}}
 if changed {
  let rev=read16(&local,2).wrapping_add(1);local[2..4].copy_from_slice(&rev.to_le_bytes());
  local[0..2].copy_from_slice(&source[0..2]);local[4..0x38].copy_from_slice(&source[4..0x38]);
 }
 Ok(FallbackResult{local,updated:changed})
}

pub(super) fn fallback_oracle(pool:&Fixture<'_>,actors:[Handle;2],local:[u8;0x38],source:[u8;0x38])->Result<[u8;0x38],String>{
 let pending=PendingE68070{first:actors[0],second:actors[1],selected_word:u32::MAX,local};
 Ok(fallback_537ee0(pool,pending,source,FallbackAdmission{source_actor:actors[0],disjoint_stable:true,x87_control:0x027f,mxcsr:0x1f80})?.local)
}

pub(super) fn local_constructor_oracle(pool:&Fixture<'_>,actors:[Handle;2],words:[u32;13])->Result<[u8;0x38],String>{
 let pending=E69BSecond::PendingLocalConstructor{first:actors[0],second:actors[1],selected_word:37,ordered_values:[1;3]};
 let defaults=LocalDefaults{vector:[words[0],words[1],words[2]],scalar:words[3],matrix:words[4..13].try_into().unwrap()};
 Ok(e69b_local_constructor(pool,pending,defaults)?.local)
}

/// Fixed-corpus adapter for the second lookup result and byte stores.
pub(super) fn e69b_second_oracle(pool:&Fixture<'_>,actors:[Handle;2],present:bool,first:u8,second:u8)->Result<[u32;4],String>{
 let prefix=E69BPrefix::PendingE57890{first:actors[0],selected_word:37,written_18c:12};
 let second_lookup=present.then_some(actors[1]);
 let outcome=e69b_second(pool,prefix,CallbackAdmission::Completed,actors[1].raw,second_lookup)?;
 Ok(match outcome {E69BSecond::ReturnedTrueSecondMissing=>[first as u32,second as u32,second as u32,1],E69BSecond::PendingLocalConstructor{ordered_values,..}=>[ordered_values[0] as u32,ordered_values[1] as u32,ordered_values[2] as u32,0]})
}

/// Controlled E69B00 type-2 branch adapter; no E57890 execution.
pub(super) fn e69b_oracle(pool:&Fixture<'_>,actors:[Handle;2],state:u32)->Result<[u32;2],String>{
 let pending=Case10Boundary::PendingRelation{actors,selected_word_0c:0,float_bits:0};
 let fresh=FreshInputs{actors,second_definition:0,duration_bits:0,default_vector:[0;3],default_quaternion:[0;4]};
 let id=EventIdentity{handle:1,object:1};let mut event=EventFixture::initialize(pool,pending,0,fresh,id,id,[0;0x7c])?;
 event.install_callback(id)?;event.install_float(id,0)?;
 match event.e69b_prefix(pool,Some(actors[0]),0,state,37)?{
  E69BPrefix::PendingE57890{written_18c,..}=>Ok([written_18c,1]),
  E69BPrefix::PendingSecondLookup{..}=>Ok([state,0]),
  _=>Err("unexpected first actor branch".into()),
 }
}

/// Fixed-corpus adapter over the owned construction phase, not an event pool.
pub(super) fn oracle(pool:&Fixture<'_>,actors:[Handle;2],words:[u32;11])->Result<[u8;0x7c],String>{
 if words[10]>255{return Err("initial fill byte out of range".into())}
 let pending=Case10Boundary::PendingRelation{actors,selected_word_0c:words[0],float_bits:0};
 let fresh=FreshInputs{actors,second_definition:words[1],duration_bits:words[2],default_vector:[words[3],words[4],words[5]],default_quaternion:[words[6],words[7],words[8],words[9]]};
 let id=EventIdentity{handle:7,object:101};
 Ok(EventFixture::initialize(pool,pending,0,fresh,id,id,[words[10] as u8;0x7c])?.bytes)
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::cell_source_capability_v1::FeatureSnapshot;
 use spore_core::{cell_part_geometry::NativeCellPart,cell_actor_geometry_projection_v1::Output};
 fn setup<'a>(rows:&'a [NativeCellPart])->(Fixture<'a>,[Handle;2]) {
  let mut pool=Fixture::new().unwrap();let snap=FeatureSnapshot::new(7,rows,[7,8,9]).unwrap();let geometry=Output{center:[0;3],scalar:0};
  let a=pool.bind([7,8,9],[7,8,9],geometry,snap).unwrap();let b=pool.bind([7,8,9],[7,8,9],geometry,snap).unwrap();(pool,[a,b])
 }
 fn row()->NativeCellPart{NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}}
 fn word(b:&[u8;0x7c],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
 #[test] fn admitted_creation_and_setters_keep_boundaries_and_untouched_bytes(){
  let rows=[row()];let (pool,actors)=setup(&rows);
  let make=||Case10Boundary::PendingRelation{actors,selected_word_0c:37,float_bits:0x80000000};
  let fresh=FreshInputs{actors,second_definition:0x1234,duration_bits:0x3f800000,default_vector:[10,11,12],default_quaternion:[20,21,22,23]};
  let id=EventIdentity{handle:17,object:99};let other=EventIdentity{handle:17,object:98};
  assert!(EventFixture::initialize(&pool,make(),1,fresh,id,id,[0xa5;0x7c]).is_err());
  assert!(EventFixture::initialize(&pool,make(),0,fresh,id,other,[0xa5;0x7c]).is_err());
  let mut e=EventFixture::initialize(&pool,make(),0,fresh,id,id,[0xa5;0x7c]).unwrap();
  for (offset,value) in [(0x24,1),(0x28,actors[0].raw),(0x2c,actors[1].raw),(0x30,37),(0x34,u32::MAX),(0x38,2),(0x3c,0x1234),(0x1c,0x3f800000),(0x20,0x3f800000),(0x40,0),(0x44,0),(0x48,10),(0x54,10),(0x60,20),(0x6c,23),(0x70,0),(0x78,0),(4,0)]{assert_eq!(word(&e.bytes,offset),value,"offset {offset:x}")}
  assert_eq!(e.bytes[8],0);assert_eq!(&e.bytes[0x75..0x78],&[0xa5;3]);assert_eq!(word(&e.bytes,0x0c),0xa5a5a5a5);
  assert!(e.install_float(id,0).is_err());assert!(e.install_callback(other).is_err());
  e.install_callback(id).unwrap();assert_eq!(word(&e.bytes,4),0xe7dbb0);assert!(e.install_float(other,0).is_err());
  e.install_float(id,0x80000000).unwrap();assert_eq!(word(&e.bytes,0x0c),0x80000000);assert_eq!(e.phase,Phase::PendingE69B00);
 }
 #[test] fn e69b_prefix_orders_missing_gate_state_write_and_pending_calls(){
  let rows=[row()];let (pool,actors)=setup(&rows);
  let pending=Case10Boundary::PendingRelation{actors,selected_word_0c:7,float_bits:0};
  let fresh=FreshInputs{actors,second_definition:0,duration_bits:0,default_vector:[0;3],default_quaternion:[0;4]};
  let id=EventIdentity{handle:1,object:1};let mut event=EventFixture::initialize(&pool,pending,0,fresh,id,id,[0;0x7c]).unwrap();
  assert!(event.e69b_prefix(&pool,Some(actors[0]),0,0,99).is_err());event.install_callback(id).unwrap();event.install_float(id,0).unwrap();
  assert_eq!(event.e69b_prefix(&pool,None,0,0,99).unwrap(),E69BPrefix::ReturnedFalseFirstMissing);
  assert_eq!(event.e69b_prefix(&pool,Some(actors[0]),1,0,99).unwrap(),E69BPrefix::ReturnedFalseActor112);
  for state in [0,3,18,22,24,27,29,40,48,50,51]{assert_eq!(event.e69b_prefix(&pool,Some(actors[0]),0,state,99).unwrap(),E69BPrefix::PendingE57890{first:actors[0],selected_word:99,written_18c:12});}
  for state in [4,17,23,28,30,39,41,49,52,u32::MAX]{assert_eq!(event.e69b_prefix(&pool,Some(actors[0]),0,state,99).unwrap(),E69BPrefix::PendingSecondLookup{first:actors[0],selected_word:99});}
  let (_other,foreign)=setup(&rows);assert!(event.e69b_prefix(&pool,Some(foreign[0]),0,0,99).is_err());
 }
 #[test] fn e69b_second_requires_fresh_lookup_and_callback_admissions(){
  let rows=[row()];let (mut pool,actors)=setup(&rows);
  let called=||E69BPrefix::PendingE57890{first:actors[0],selected_word:37,written_18c:12};
  let skipped=||E69BPrefix::PendingSecondLookup{first:actors[0],selected_word:37};
  assert!(e69b_second(&pool,called(),CallbackAdmission::Skipped,actors[1].raw,Some(actors[1])).is_err());
  assert!(e69b_second(&pool,skipped(),CallbackAdmission::Completed,actors[1].raw,Some(actors[1])).is_err());
  assert_eq!(e69b_second(&pool,called(),CallbackAdmission::Completed,123,None).unwrap(),E69BSecond::ReturnedTrueSecondMissing);
  assert!(e69b_second(&pool,skipped(),CallbackAdmission::Skipped,99,Some(actors[1])).is_err());
  assert_eq!(e69b_second(&pool,called(),CallbackAdmission::Completed,actors[1].raw,Some(actors[1])).unwrap(),E69BSecond::PendingLocalConstructor{first:actors[0],second:actors[1],selected_word:37,ordered_values:[1,1,1]});
  let (_other,foreign)=setup(&rows);assert!(e69b_second(&pool,skipped(),CallbackAdmission::Skipped,foreign[1].raw,Some(foreign[1])).is_err());
  pool.release(actors[0]).unwrap();assert!(e69b_second(&pool,skipped(),CallbackAdmission::Skipped,actors[1].raw,Some(actors[1])).is_err());
  assert_eq!(e69b_second(&pool,skipped(),CallbackAdmission::Skipped,0,None).unwrap(),E69BSecond::ReturnedTrueSecondMissing);
 }
 #[test] fn local_constructor_copies_admitted_words_and_stops_at_e68070(){
  let rows=[row()];let (mut pool,actors)=setup(&rows);
  let pending=||E69BSecond::PendingLocalConstructor{first:actors[0],second:actors[1],selected_word:37,ordered_values:[1;3]};
  let defaults=LocalDefaults{vector:[0x80000000,0x7fc00001,3],scalar:4,matrix:[5,6,7,8,9,10,11,12,13]};
  let out=e69b_local_constructor(&pool,pending(),defaults).unwrap();
  let words:Vec<_>=out.local.chunks_exact(4).map(|v|u32::from_le_bytes(v.try_into().unwrap())).collect();
  assert_eq!(words,vec![0,0x80000000,0x7fc00001,3,4,5,6,7,8,9,10,11,12,13]);
  assert_eq!((out.first,out.second,out.selected_word),(actors[0],actors[1],37));
  assert!(e69b_local_constructor(&pool,E69BSecond::ReturnedTrueSecondMissing,defaults).is_err());
  pool.release(actors[1]).unwrap();assert!(e69b_local_constructor(&pool,pending(),defaults).is_err());
 }
 #[test] fn finite_fallback_compares_short_circuits_and_wraps_revision(){
  let rows=[row()];let (pool,actors)=setup(&rows);let make=|local|PendingE68070{first:actors[0],second:actors[1],selected_word:u32::MAX,local};
  let admission=FallbackAdmission{source_actor:actors[0],disjoint_stable:true,x87_control:0x027f,mxcsr:0x1f80};
  let mut a=[0u8;0x38];a[2..4].copy_from_slice(&u16::MAX.to_le_bytes());let mut b=a;b[4..8].copy_from_slice(&0x80000000u32.to_le_bytes());
  let equal=fallback_537ee0(&pool,make(a),b,admission).unwrap();assert!(!equal.updated);assert_eq!(equal.local,a);
  b[0x14..0x18].copy_from_slice(&0x3f800000u32.to_le_bytes());assert!(!fallback_537ee0(&pool,make(a),b,admission).unwrap().updated);
  a[0..2].copy_from_slice(&2u16.to_le_bytes());b[0..2].copy_from_slice(&2u16.to_le_bytes());let changed=fallback_537ee0(&pool,make(a),b,admission).unwrap();assert!(changed.updated);assert_eq!(&changed.local[2..4],&[0,0]);assert_eq!(&changed.local[4..],&b[4..]);
  let mut bad=b;bad[4..8].copy_from_slice(&0x7fc00000u32.to_le_bytes());assert!(fallback_537ee0(&pool,make(a),bad,admission).is_err());
  let mut no=admission;no.disjoint_stable=false;assert!(fallback_537ee0(&pool,make(a),b,no).is_err());no=admission;no.mxcsr|=1;assert!(fallback_537ee0(&pool,make(a),b,no).is_err());
 }
 #[test] fn stale_or_foreign_fresh_actors_are_rejected(){
  let rows=[row()];let (mut pool,actors)=setup(&rows);let (_other,foreign)=setup(&rows);
  let make=||Case10Boundary::PendingRelation{actors,selected_word_0c:0,float_bits:0};
  let id=EventIdentity{handle:1,object:1};let mut fresh=FreshInputs{actors:foreign,second_definition:0,duration_bits:0,default_vector:[0;3],default_quaternion:[0;4]};
  assert!(EventFixture::initialize(&pool,make(),0,fresh,id,id,[0;0x7c]).is_err());
  fresh.actors=actors;pool.release(actors[1]).unwrap();assert!(EventFixture::initialize(&pool,make(),0,fresh,id,id,[0;0x7c]).is_err());
 }
}
