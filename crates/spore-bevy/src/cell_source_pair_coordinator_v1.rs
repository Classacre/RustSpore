//! Controlled E7F170 ordering fixture. Effects are caller scripts, never native
//! callbacks, and all identities are owned fixture labels rather than pointers.
#[path = "cell_source_case10_event_v1.rs"]
mod case10_event;
use super::{Fixture,Handle};
use spore_core::cell_feeding_contact_arbitration_e59b40_v1::ContactOutput;
use spore_core::cell_feeding_b7_selection_v1::{self as b7,Binding,Step};
use spore_core::cell_feature_first_candidate_v1::{PendingCandidate,LocalGeometry};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Definition {reference:u32, object:u64, b4:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Identity {actors:[Handle;2],definitions:[(u32,u64);2]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Phase {NeedA,NeedB,First,Second,PendingEvent,PendingCleanup}
#[derive(Debug)]
struct Coordinator {actors:[Handle;2],refs:[u32;2],defs:[Option<Definition>;2],phase:Phase,selection:ContactOutput,event_float_bits:u32}
#[derive(Debug,PartialEq,Eq)]
struct PendingEvent {selection:ContactOutput,float_bits:u32}
/// The definition token is a caller-admitted result of E4CAA0 for the second
/// selected actor. It is an owned fixture label, never a native pointer.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Selector19Definition {reference:u32,token:u64}
#[derive(Debug,PartialEq,Eq)]
enum Selector19Boundary {
 GateReturned,
 /// E780D0 and the subsequent holder cleanup have not run.
 PendingActorOperation {second:Handle,definition:Selector19Definition,arguments:[u32;4]},
}
impl PendingEvent {
 /// Both handles are freshly checked in selected-record order. `first_bytes`
 /// and the optional definition snapshot are caller-supplied stable state;
 /// this executes neither native lookup, holder management nor actor removal.
 fn selector19(&self,pool:&Fixture<'_>,actors:[Handle;2],first_bytes:[u8;2],second_definition_ref:u32,definition:Option<Selector19Definition>,stable:bool)->Result<Selector19Boundary,String>{
  if self.selection.selector!=19{return Err("unsupported selector".into())}
  if !stable{return Err("actor and definition stability not admitted".into())}
  for (i,actor) in actors.iter().enumerate(){
   if actor.raw!=self.selection.actor_ids[i]{return Err("selected actor mismatch".into())}
   pool.resolve(*actor).ok_or("stale or foreign selected actor")?;
  }
  if first_bytes[0]==1||first_bytes[1]==1{return Ok(Selector19Boundary::GateReturned)}
  let definition=definition.ok_or("definition resolver result not admitted")?;
  if definition.token==0||definition.reference!=second_definition_ref{return Err("definition identity mismatch".into())}
  Ok(Selector19Boundary::PendingActorOperation{second:actors[1],definition,arguments:[actors[1].raw,1,0,0]})
 }
}
impl Coordinator {
 fn new(pool:&Fixture<'_>,actors:[Handle;2],refs:[u32;2],opaque_tail:[u32;6],event_float_bits:u32)->Result<Self,String>{
  for h in actors {pool.resolve(h).ok_or("stale or foreign actor")?;}
  if actors[0]==actors[1]{return Err("equal actor IDs".into())}
  if !f32::from_bits(event_float_bits).is_finite(){return Err("unsupported event float".into())}
  Ok(Self{actors,refs,defs:[None,None],phase:Phase::NeedA,event_float_bits,selection:ContactOutput{actor_ids:[0;2],selector:0,feature_indices:[0;2],depth_bits:0,point_words:[opaque_tail[0],opaque_tail[1],opaque_tail[2]],normal_words:[opaque_tail[3],opaque_tail[4],opaque_tail[5]]}})
 }
 /// Explicit post-resolver snapshot; holder effects and pointer validity are
 /// admitted by the caller. Both resolvers run before either B4 gate.
 fn resolve(&mut self,pool:&Fixture<'_>,actor:Handle,definition:Definition)->Result<(),String>{
  let index=match self.phase {Phase::NeedA=>0,Phase::NeedB=>1,_=>return Err("unexpected resolver phase".into())};
  if actor!=self.actors[index]||definition.reference!=self.refs[index]||definition.object==0{return Err("definition identity mismatch".into())}
  pool.resolve(actor).ok_or("stale or foreign retained actor")?;
  if index==1 {pool.resolve(self.actors[0]).ok_or("stale first actor after resolver")?;}
  self.defs[index]=Some(definition);
  self.phase=if index==0{Phase::NeedB}else if self.defs[0].unwrap().b4==0||definition.b4==0{Phase::PendingCleanup}else{Phase::First};
  Ok(())
 }
 fn identity(&self)->Result<Identity,String>{let a=self.defs[0].ok_or("first unresolved")?;let b=self.defs[1].ok_or("second unresolved")?;Ok(Identity{actors:self.actors,definitions:[(a.reference,a.object),(b.reference,b.object)]})}
 /// Each supplied selection is an admitted effect of the just-completed
 /// scripted selector. Definitions and actor identities must remain retained;
 /// their mutable contents and callbacks are not modeled by this fixture.
 fn finish_pass(&mut self,pool:&Fixture<'_>,retained:Identity,order:[Handle;2],selection:ContactOutput)->Result<(),String>{
  let phase=self.phase;
  if phase!=Phase::First&&phase!=Phase::Second{return Err("unexpected selector phase".into())}
  if retained!=self.identity()?{return Err("retained identity mismatch".into())}
  let expected=if phase==Phase::First{self.actors}else{[self.actors[1],self.actors[0]]};
  if order!=expected{return Err("selector order mismatch".into())}
  for h in retained.actors {pool.resolve(h).ok_or("stale or foreign retained actor")?;}
  self.selection=selection;
  self.phase=if phase==Phase::First{Phase::Second}else if selection.selector==0||selection.selector==0x16{Phase::PendingCleanup}else{Phase::PendingEvent};
  Ok(())
 }
 /// E7E820 is pending; cleanup remains pending after either path.
 fn pending_event(&self)->Option<PendingEvent>{(self.phase==Phase::PendingEvent).then_some(PendingEvent{selection:self.selection,float_bits:self.event_float_bits})}
}

/// Two explicitly scripted B7 candidates share the coordinator's one output.
/// Fixture handles and their word values are synthetic; the caller admits that
/// actors, definitions, player word and candidate data stay stable throughout.
/// Resolver, selector routing, cleanup and E7E820 are not executed here.
#[derive(Debug,PartialEq,Eq)]
struct ScriptedB7Result {before:[ContactOutput;2], selection:ContactOutput, event:Option<PendingEvent>}
fn scripted_b7(pool:&Fixture<'_>,actors:[Handle;2],candidates:&[PendingCandidate;2],player_word:u32,stable:bool,float_bits:u32)->Result<ScriptedB7Result,String>{
 if !stable{return Err("actor, definition and candidate stability not admitted".into())}
 let mut c=Coordinator::new(pool,actors,[11,22],[0xa5a5a5a5;6],float_bits)?;
 c.resolve(pool,actors[0],Definition{reference:11,object:101,b4:7})?;
 c.resolve(pool,actors[1],Definition{reference:22,object:102,b4:7})?;
 let identity=c.identity()?;let mut before=[c.selection;2];
 for i in 0..2 {
  let order=if i==0{actors}else{[actors[1],actors[0]]};let candidate=&candidates[i];
  if candidate.definition_b4!=[7,7]||candidate.actor_tokens!=order.map(|h|h.raw){return Err("candidate actor or definition mismatch".into())}
  let (point,normal)=match &candidate.geometry {LocalGeometry::Nondegenerate{point,normal}=>(*point,*normal),LocalGeometry::OpaqueSmall=>return Err("opaque candidate locals".into())};
  if !std::iter::once(candidate.depth).chain(point).chain(normal).all(|v|f32::from_bits(v).is_finite()){return Err("nonfinite candidate geometry".into())}
  before[i]=c.selection;
  let binding=Binding{actor_tokens:order.map(|h|h.raw),actor_words:order.map(|h|h.raw),current_player_word:player_word,stable_disjoint:true};
  let selection=match b7::apply(candidate,binding,c.selection).map_err(|e|format!("B7 admission: {e:?}"))? {
   Step::Submitted{selection,..}=>selection,
   _=>return Err("candidate did not reach the B7 submission".into()),
  };
  c.finish_pass(pool,identity,order,selection)?;
 }
 Ok(ScriptedB7Result{before,selection:c.selection,event:c.pending_event()})
}

pub(super) fn scripted_b7_oracle(pool:&Fixture<'_>,actors:[Handle;2],player:u32,equal:bool)->Result<Vec<u32>,String>{
 fn words(c:ContactOutput)->[u32;12]{[c.actor_ids[0],c.actor_ids[1],c.selector as u32,c.feature_indices[0],c.feature_indices[1],c.depth_bits,c.point_words[0],c.point_words[1],c.point_words[2],c.normal_words[0],c.normal_words[1],c.normal_words[2]]}
 let mut out=vec![0xcccccccc;57];out[..6].fill(0);
 if equal {if Coordinator::new(pool,[actors[0],actors[0]],[11,22],[0;6],0).is_ok(){return Err("equal actors accepted".into())}return Ok(out)}
 let make=|order:[Handle;2],depth:u32,point:[u32;3],normal:[u32;3]|PendingCandidate{actor_tokens:order.map(|h|h.raw),definition_b4:[7,7],second_definition_b0:0,relation:0,classes:[2,3],indices:[0,1],depth,geometry:LocalGeometry::Nondegenerate{point,normal}};
 let x=scripted_b7(pool,actors,&[make(actors,1f32.to_bits(),[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()],[4f32.to_bits(),5f32.to_bits(),6f32.to_bits()]),make([actors[1],actors[0]],2f32.to_bits(),[7f32.to_bits(),8f32.to_bits(),9f32.to_bits()],[10f32.to_bits(),11f32.to_bits(),12f32.to_bits()])],player,true,0x80000000)?;
 out[..6].copy_from_slice(&[2,0x12,2,0,0,2]);out[6..18].copy_from_slice(&words(x.before[0]));out[18..30].copy_from_slice(&words(x.before[1]));out[45..57].copy_from_slice(&words(x.selection));
 if let Some(event)=x.event{out[4]=1;out[30..42].copy_from_slice(&words(event.selection));out[42]=event.float_bits;}
 Ok(out)
}

/// Counts describe the comparator's admitted stubs, including cleanup; the
/// checked continuation itself executes neither the cleanup nor the operation.
pub(super) fn selector19_oracle(pool:&Fixture<'_>,actors:[Handle;2],bytes:[u8;2])->Result<[u32;10],String>{
 let selection=ContactOutput{actor_ids:actors.map(|h|h.raw),selector:19,feature_indices:[0;2],depth_bits:0,point_words:[0;3],normal_words:[0;3]};
 let event=PendingEvent{selection,float_bits:0};let def=Selector19Definition{reference:22,token:102};
 let mut result=[0u32;10];
 match event.selector19(pool,actors,bytes,22,Some(def),true)? {
  Selector19Boundary::GateReturned=>{},
  Selector19Boundary::PendingActorOperation{arguments,..}=>{result[..4].copy_from_slice(&[1,1,1,1]);result[4..8].copy_from_slice(&arguments);},
 }
 Ok(result)
}

/// Fresh, caller-admitted bytes after both selected IDs have been resolved.
/// Their stability through the gates is a fixture admission, not native locking.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Case10ActorBytes {first_178:u8,second_179:u8,second_111:u8,second_17b:u8}
#[derive(Debug,PartialEq,Eq)]
enum Case10Boundary {
 /// E7E050 returns AL=1 here without constructing an event.
 ReturnedTrueWithoutCreation,
 /// E7E050 returns AL=0; E7E820 fallback E67170 remains pending.
 ReturnedFalsePendingFallback,
 /// Call E572E0(first, second); no relation result or later state is assumed.
 PendingRelation {actors:[Handle;2],selected_word_0c:u32,float_bits:u32},
}
impl PendingEvent {
 /// Only the E7E820 selector-10 branch. The actor handles are explicit
 /// fixture identities, freshly resolved in record order, not native pointers.
 fn case10_gate(&self,pool:&Fixture<'_>,actors:[Handle;2],bytes:Case10ActorBytes)->Result<Case10Boundary,String>{
  if self.selection.selector!=10{return Err("unsupported event selector".into())}
  for i in 0..2 {
   if actors[i].raw!=self.selection.actor_ids[i]{return Err("selected actor identity mismatch".into())}
   pool.resolve(actors[i]).ok_or("stale or foreign selected actor")?;
  }
  if bytes.second_179!=0{return Ok(Case10Boundary::ReturnedTrueWithoutCreation)}
  if bytes.second_111==1{return Ok(Case10Boundary::ReturnedFalsePendingFallback)}
  if bytes.second_17b!=0{return Ok(Case10Boundary::ReturnedFalsePendingFallback)}
  if bytes.first_178!=0{return Ok(Case10Boundary::ReturnedFalsePendingFallback)}
  Ok(Case10Boundary::PendingRelation{actors,selected_word_0c:self.selection.feature_indices[0],float_bits:self.float_bits})
 }
}

pub(super) fn fallback_oracle(pool:&Fixture<'_>,actors:[Handle;2],local:[u8;0x38],source:[u8;0x38])->Result<[u8;0x38],String>{case10_event::fallback_oracle(pool,actors,local,source)}

pub(super) fn local_constructor_oracle(pool:&Fixture<'_>,actors:[Handle;2],words:[u32;13])->Result<[u8;0x38],String>{case10_event::local_constructor_oracle(pool,actors,words)}

pub(super) fn e69b_second_oracle(pool:&Fixture<'_>,actors:[Handle;2],present:bool,first:u8,second:u8)->Result<[u32;4],String>{case10_event::e69b_second_oracle(pool,actors,present,first,second)}

pub(super) fn e69b_oracle(pool:&Fixture<'_>,actors:[Handle;2],state:u32)->Result<[u32;2],String>{case10_event::e69b_oracle(pool,actors,state)}

/// Comparison adapter using only the owned event-construction phase.
pub(super) fn case10_event_oracle(pool:&Fixture<'_>,actors:[Handle;2],words:[u32;11])->Result<[u8;0x7c],String>{case10_event::oracle(pool,actors,words)}

/// Fixed synthetic gate-comparison adapter. Relation result is an admitted stub,
/// not a modeled E572E0 call or an event allocation.
pub(super) fn case10_oracle(pool:&Fixture<'_>,a:Handle,b:Handle,bytes:[u8;4],relation:u8)->Result<[u32;2],String>{
 let mut c=Coordinator::new(pool,[a,b],[11,22],[0;6],0)?;
 c.resolve(pool,a,Definition{reference:11,object:101,b4:2})?;
 c.resolve(pool,b,Definition{reference:22,object:102,b4:2})?;
 let id=c.identity()?;let mut selection=c.selection;selection.selector=10;selection.actor_ids=[a.raw,b.raw];
 c.finish_pass(pool,id,[a,b],selection)?;c.finish_pass(pool,id,[b,a],selection)?;
 let state=c.pending_event().ok_or("missing pending event")?.case10_gate(pool,[a,b],Case10ActorBytes{second_179:bytes[0],second_111:bytes[1],second_17b:bytes[2],first_178:bytes[3]})?;
 Ok(match state {Case10Boundary::ReturnedTrueWithoutCreation=>[1,0],Case10Boundary::ReturnedFalsePendingFallback=>[0,0],Case10Boundary::PendingRelation{..}=>[if relation==0{2}else{0},1]})
}

/// Demonstrate only scripted effects with explicit synthetic definition IDs.
pub(super) fn inspect(pool:&Fixture<'_>,a:Handle,b:Handle)->Result<String,String>{
 let mut c=Coordinator::new(pool,[a,b],[11,22],[0xa5a5a5a5;6],0)?;
 c.resolve(pool,a,Definition{reference:11,object:101,b4:2})?;
 c.resolve(pool,b,Definition{reference:22,object:102,b4:2})?;
 let retained=c.identity()?;
 let mut first=c.selection;first.actor_ids=[a.raw,b.raw];first.selector=19;
 c.finish_pass(pool,retained,[a,b],first)?;
 // The caller explicitly admits no further change for the reverse pass.
 c.finish_pass(pool,retained,[b,a],first)?;
 Ok(format!("scripted_first_then_reverse shared={:?} pending_event={:?}; selector effects supplied, resolver/cleanup/event not executed",c.selection,c.pending_event()))
}

/// Adapter output for the fixed scripted native-host corpus. This runs the
/// production coordinator only; counts describe admitted fixture scripts.
pub(super) fn oracle(pool:&Fixture<'_>,a:Handle,b:Handle,equal:bool,b4:[u32;2],selectors:[i32;2],float_bits:u32)->Result<Vec<u32>,String>{
 fn words(c:ContactOutput)->[u32;12]{[c.actor_ids[0],c.actor_ids[1],c.selector as u32,c.feature_indices[0],c.feature_indices[1],c.depth_bits,c.point_words[0],c.point_words[1],c.point_words[2],c.normal_words[0],c.normal_words[1],c.normal_words[2]]}
 let mut out=vec![0u32;43];out[6..].fill(0xcccccccc);
 if equal {if Coordinator::new(pool,[a,a],[11,22],[0xa5a5a5a5;6],float_bits).is_ok(){return Err("equal fixture actor accepted".into())}return Ok(out)}
 let mut c=Coordinator::new(pool,[a,b],[11,22],[0xa5a5a5a5;6],float_bits)?;
 c.resolve(pool,a,Definition{reference:11,object:101,b4:b4[0]})?;c.resolve(pool,b,Definition{reference:22,object:102,b4:b4[1]})?;
 out[0]=2;out[1]=0x12;out[5]=2;
 if c.phase==Phase::PendingCleanup{return Ok(out)}
 let identity=c.identity()?;out[6..18].copy_from_slice(&words(c.selection));
 let mut first=c.selection;first.selector=selectors[0];c.finish_pass(pool,identity,[a,b],first)?;
 out[18..30].copy_from_slice(&words(c.selection));let mut second=c.selection;second.selector=selectors[1];c.finish_pass(pool,identity,[b,a],second)?;
 out[2]=2;out[3]=0x12;
 if let Some(event)=c.pending_event(){out[4]=1;out[30..42].copy_from_slice(&words(event.selection));out[42]=event.float_bits;}
 Ok(out)
}

#[cfg(test)]
mod tests {
 use super::*;
 use spore_core::cell_part_geometry::NativeCellPart;
 use spore_core::cell_actor_geometry_projection_v1::Output;
 use crate::cell_source_capability_v1::FeatureSnapshot;
 fn row()->NativeCellPart{NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}}
 fn actors<'a>(pool:&mut Fixture<'a>,rows:&'a [NativeCellPart])->[Handle;2]{let s=FeatureSnapshot::new(7,rows,[7,8,9]).unwrap();let g=Output{center:[0;3],scalar:0};[pool.bind([7,8,9],[7,8,9],g,s).unwrap(),pool.bind([7,8,9],[7,8,9],g,s).unwrap()]}
 fn d(reference:u32,object:u64,b4:u32)->Definition{Definition{reference,object,b4}}
 #[test] fn b7_two_pass_shares_selection_and_rejects_unadmitted_inputs(){
  let rows=[row()];let mut pool=Fixture::new().unwrap();let [a,b]=actors(&mut pool,&rows);
  let make=|order:[Handle;2],depth:f32|PendingCandidate{actor_tokens:order.map(|h|h.raw),definition_b4:[7,7],second_definition_b0:0,relation:0,classes:[2,3],indices:[0,1],depth:depth.to_bits(),geometry:LocalGeometry::Nondegenerate{point:[1,2,3],normal:[4,5,6]}};
  let candidates=[make([a,b],1.),make([b,a],2.)];
  let first=scripted_b7(&pool,[a,b],&candidates,a.raw,true,0).unwrap();assert_eq!(first.selection.actor_ids,[a.raw,b.raw]);assert_eq!(first.event.unwrap().selection.selector,19);
  let reverse=scripted_b7(&pool,[a,b],&candidates,b.raw,true,0).unwrap();assert_eq!(reverse.selection.actor_ids,[b.raw,a.raw]);assert_eq!(reverse.event.unwrap().selection.selector,19);
  let suppressed=scripted_b7(&pool,[a,b],&candidates,0x99999,true,0).unwrap();assert_eq!(suppressed.selection.actor_ids,[b.raw,a.raw]);assert!(suppressed.event.is_none());
  assert!(scripted_b7(&pool,[a,b],&candidates,a.raw,false,0).is_err());
  let mut wrong=[make([a,b],1.),make([b,a],2.)];wrong[1].actor_tokens=[a.raw,b.raw];assert!(scripted_b7(&pool,[a,b],&wrong,a.raw,true,0).is_err());
  let mut opaque=[make([a,b],1.),make([b,a],2.)];opaque[1].geometry=LocalGeometry::OpaqueSmall;assert!(scripted_b7(&pool,[a,b],&opaque,a.raw,true,0).is_err());
  pool.release(b).unwrap();assert!(scripted_b7(&pool,[a,b],&candidates,a.raw,true,0).is_err());
 }
 #[test] fn selector19_requires_current_selected_actors_and_explicit_definition(){
  let rows=[row()];let mut pool=Fixture::new().unwrap();let [a,b]=actors(&mut pool,&rows);
  let make=|order:[Handle;2]|PendingCandidate{actor_tokens:order.map(|h|h.raw),definition_b4:[7,7],second_definition_b0:0,relation:0,classes:[2,3],indices:[0,1],depth:1f32.to_bits(),geometry:LocalGeometry::Nondegenerate{point:[1,2,3],normal:[4,5,6]}};
  let candidates=[make([a,b]),make([b,a])];let event=scripted_b7(&pool,[a,b],&candidates,a.raw,true,0).unwrap().event.unwrap();
  let definition=Some(Selector19Definition{reference:22,token:102});
  assert_eq!(event.selector19(&pool,[a,b],[1,0],22,None,true).unwrap(),Selector19Boundary::GateReturned);
  assert_eq!(event.selector19(&pool,[a,b],[0,1],22,None,true).unwrap(),Selector19Boundary::GateReturned);
  assert_eq!(event.selector19(&pool,[a,b],[2,2],22,definition,true).unwrap(),Selector19Boundary::PendingActorOperation{second:b,definition:definition.unwrap(),arguments:[b.raw,1,0,0]});
  assert!(event.selector19(&pool,[a,b],[0,0],22,None,true).is_err());
  assert!(event.selector19(&pool,[a,b],[0,0],23,definition,true).is_err());
  assert!(event.selector19(&pool,[b,a],[0,0],22,definition,true).is_err());
  assert!(event.selector19(&pool,[a,b],[0,0],22,definition,false).is_err());
  pool.release(b).unwrap();assert!(event.selector19(&pool,[a,b],[0,0],22,definition,true).is_err());
 }
 #[test] fn resolver_order_shared_selection_and_pending_event(){
  let rows=[row()];let mut pool=Fixture::new().unwrap();let [a,b]=actors(&mut pool,&rows);
  let mut c=Coordinator::new(&pool,[a,b],[11,22],[1,2,3,4,5,6],0x80000000).unwrap();
  assert_eq!((c.selection.actor_ids,c.selection.selector,c.selection.feature_indices,c.selection.depth_bits),([0;2],0,[0;2],0));
  assert_eq!(c.selection.point_words,[1,2,3]);assert_eq!(c.selection.normal_words,[4,5,6]);
  assert!(c.resolve(&pool,b,d(22,102,2)).is_err());c.resolve(&pool,a,d(11,101,2)).unwrap();c.resolve(&pool,b,d(22,102,2)).unwrap();
  let identity=c.identity().unwrap();let mut altered=identity;altered.definitions[0].1=999;
  assert!(c.finish_pass(&pool,altered,[a,b],c.selection).is_err());
  assert!(c.finish_pass(&pool,identity,[b,a],c.selection).is_err());
  let mut first=c.selection;first.selector=9;c.finish_pass(&pool,identity,[a,b],first).unwrap();assert_eq!(c.phase,Phase::Second);
  let mut second=c.selection;second.selector=10;second.actor_ids=[b.raw,a.raw];c.finish_pass(&pool,identity,[b,a],second).unwrap();
  assert_eq!(c.pending_event(),Some(PendingEvent{selection:second,float_bits:0x80000000}));assert_eq!(c.phase,Phase::PendingEvent);
  assert!(c.finish_pass(&pool,identity,[b,a],second).is_err());
 }
 #[test] fn case10_gates_require_fresh_selected_fixture_actors_and_preserve_order(){
  let rows=[row()];let mut pool=Fixture::new().unwrap();let [a,b]=actors(&mut pool,&rows);
  let mut c=Coordinator::new(&pool,[a,b],[11,22],[0;6],0x80000000).unwrap();
  c.resolve(&pool,a,d(11,101,2)).unwrap();c.resolve(&pool,b,d(22,102,2)).unwrap();let id=c.identity().unwrap();
  let mut x=c.selection;x.selector=10;x.actor_ids=[a.raw,b.raw];x.feature_indices[0]=37;
  c.finish_pass(&pool,id,[a,b],x).unwrap();c.finish_pass(&pool,id,[b,a],x).unwrap();let event=c.pending_event().unwrap();
  let base=Case10ActorBytes{first_178:0,second_179:0,second_111:0,second_17b:0};
  assert_eq!(event.case10_gate(&pool,[a,b],base).unwrap(),Case10Boundary::PendingRelation{actors:[a,b],selected_word_0c:37,float_bits:0x80000000});
  assert_eq!(event.case10_gate(&pool,[a,b],Case10ActorBytes{second_179:1,second_111:1,second_17b:1,first_178:1}).unwrap(),Case10Boundary::ReturnedTrueWithoutCreation);
  for blocked in [Case10ActorBytes{second_111:1,..base},Case10ActorBytes{second_17b:1,..base},Case10ActorBytes{first_178:1,..base}] {assert_eq!(event.case10_gate(&pool,[a,b],blocked).unwrap(),Case10Boundary::ReturnedFalsePendingFallback);}
  assert!(event.case10_gate(&pool,[b,a],base).is_err());
  let mut foreign=Fixture::new().unwrap();let [f,g]=actors(&mut foreign,&rows);assert_eq!([f.raw,g.raw],[a.raw,b.raw]);assert!(event.case10_gate(&pool,[f,g],base).is_err());
  pool.release(b).unwrap();assert!(event.case10_gate(&pool,[a,b],base).is_err());
 }
 #[test] fn zero_definition_and_suppressed_outputs_stop_at_cleanup(){
  let rows=[row()];let mut pool=Fixture::new().unwrap();let [a,b]=actors(&mut pool,&rows);
  let mut c=Coordinator::new(&pool,[a,b],[11,22],[0;6],0).unwrap();c.resolve(&pool,a,d(11,101,0)).unwrap();
  assert_eq!(c.phase,Phase::NeedB);c.resolve(&pool,b,d(22,102,2)).unwrap();assert_eq!(c.phase,Phase::PendingCleanup);
  for selector in [0,0x16]{let mut c=Coordinator::new(&pool,[a,b],[11,22],[0;6],0).unwrap();c.resolve(&pool,a,d(11,101,2)).unwrap();c.resolve(&pool,b,d(22,102,2)).unwrap();let id=c.identity().unwrap();let mut x=c.selection;x.selector=selector;c.finish_pass(&pool,id,[a,b],x).unwrap();c.finish_pass(&pool,id,[b,a],x).unwrap();assert_eq!(c.phase,Phase::PendingCleanup);assert_eq!(c.pending_event(),None);}
 }
 #[test] fn equal_foreign_stale_and_wrong_reference_are_rejected(){
  let rows=[row()];let mut pool=Fixture::new().unwrap();let [a,b]=actors(&mut pool,&rows);
  assert!(Coordinator::new(&pool,[a,a],[11,22],[0;6],0).is_err());
  let mut foreign=Fixture::new().unwrap();let [f,_]=actors(&mut foreign,&rows);assert_eq!(a.raw,f.raw);
  assert!(Coordinator::new(&pool,[a,f],[11,22],[0;6],0).is_err());
  assert!(Coordinator::new(&pool,[a,b],[11,22],[0;6],f32::NAN.to_bits()).is_err());
  let mut c=Coordinator::new(&pool,[a,b],[11,22],[0;6],0).unwrap();assert!(c.resolve(&pool,a,d(12,101,2)).is_err());
  c.resolve(&pool,a,d(11,101,2)).unwrap();c.resolve(&pool,b,d(22,102,2)).unwrap();let id=c.identity().unwrap();pool.release(b).unwrap();assert!(c.finish_pass(&pool,id,[a,b],c.selection).is_err());
 }
}
