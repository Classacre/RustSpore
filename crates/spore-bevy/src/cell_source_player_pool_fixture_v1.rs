//! Owned inspection fixture only. Checked pool identity and epoch are additional
//! safety policies, not claims about the source's unchecked B71D00 lookup.
#[path = "cell_source_pair_coordinator_v1.rs"]
mod pair_coordinator;
use std::sync::atomic::{AtomicU64, Ordering};
use spore_core::cell_actor_pool_checked_v1::{CheckedActorPool, PoolError};
use spore_core::cell_actor_geometry_projection_v1::Output;
use crate::cell_source_capability_v1::FeatureSnapshot;
use super::feature_geometry::{self,Transform};
use spore_core::cell_feeding_producer_prefix_v1::{self as producer,PrefixInput,PrefixResult};
use spore_core::cell_feeding_contact_arbitration_e59b40_v1::{self as arbitration,ArbitrationWrites,ContactOutput};

static NEXT_POOL: AtomicU64 = AtomicU64::new(1);
static NEXT_POINT: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Handle { pool:u64, raw:u32 }
struct Actor<'a> {
    constructor_triple:[u32;3], actor_triple:[u32;3],
    // This is the separately admitted pre-override result, not an updated leaf.
    pre_override_geometry:Output, features:FeatureSnapshot<'a>,
}
pub(super) struct Fixture<'a> { pool_id:u64, pool:CheckedActorPool<Actor<'a>> }
impl<'a> Fixture<'a> {
    pub(super) fn new()->Result<Self, String> {
        let pool_id=NEXT_POOL.fetch_update(Ordering::Relaxed,Ordering::Relaxed,|n|n.checked_add(1))
            .map_err(|_|"fixture pool identity exhausted")?;
        Ok(Self { pool_id,pool:CheckedActorPool::new(2,0).map_err(|e|format!("pool: {e:?}"))? })
    }
    fn bind(&mut self, constructor_triple:[u32;3], actor_triple:[u32;3],
        pre_override_geometry:Output, features:FeatureSnapshot<'a>)->Result<Handle,String> {
        if !features.matches_actor(actor_triple) { return Err("actor/cache first-word mismatch".into()); }
        let raw=self.pool.allocate(Actor { constructor_triple,actor_triple,pre_override_geometry,features }).map_err(|e|format!("pool: {e:?}"))?;
        Ok(Handle{pool:self.pool_id,raw})
    }
    fn resolve(&self, h:Handle)->Option<&Actor<'a>> {
        if h.pool != self.pool_id { return None; }
        self.pool.get(h.raw)
    }
    fn release(&mut self, h:Handle)->Result<(),PoolError> {
        if h.pool != self.pool_id { return Err(PoolError::InvalidHandle); }
        self.pool.release(h.raw).map(drop)
    }
    /// Fixture-only join. Inputs are caller admissions, and the producer model
    /// stops at its pending external-call boundary without dispatching effects.
    fn producer_prefix(&self, a:Handle,b:Handle,player:Handle, indices:[usize;2],
        transforms:[Transform;2], definitions:[i32;2], carried:Option<[u32;6]>) -> Result<FixtureOutcome,String> {
        let first=self.resolve(a).ok_or("stale or foreign first actor")?;
        let second=self.resolve(b).ok_or("stale or foreign second actor")?;
        self.resolve(player).ok_or("stale or foreign player")?;
        if a==b {return Err("identical fixture actor handles".into())}
        let row_a=first.features.candidates().find(|(i,_)|*i==indices[0]).ok_or("missing or skipped first feature")?.1;
        let row_b=second.features.candidates().find(|(i,_)|*i==indices[1]).ok_or("missing or skipped second feature")?.1;
        let shape_a=feature_geometry::project(row_a,transforms[0]).map_err(|e|format!("first geometry: {e:?}"))?;
        let shape_b=feature_geometry::project_second(row_b,transforms[1]).map_err(|e|format!("second geometry: {e:?}"))?;
        let result=feature_geometry::continue_pair(shape_a,shape_b,carried).map_err(|e|format!("geometry: {e:?}"))?;
        let (depth,point,normal)=match result {
            feature_geometry::Geometry::Separated{..}=>return Ok(FixtureOutcome::Separated),
            feature_geometry::Geometry::Small{depth,carried:Some(words)}=>(depth,[words[3],words[4],words[5]],[words[0],words[1],words[2]]),
            feature_geometry::Geometry::Small{carried:None,..}=>return Err("small-distance local words unavailable".into()),
            feature_geometry::Geometry::Nondegenerate{depth,point,normal,..}=>(depth,point,normal),
        };
        let feature_indices=[u32::try_from(indices[0]).map_err(|_|"first index overflow")?,u32::try_from(indices[1]).map_err(|_|"second index overflow")?];
        let input=PrefixInput {feature_a:row_a.class as i32,feature_b:row_b.class as i32,
            definition_a:definitions[0],definition_b:definitions[1],actor_ids:[a.raw,b.raw],current_player_id:player.raw,
            feature_indices,depth_bits:depth,point_words:point,normal_words:normal};
        let result=producer::evaluate_prefix(input).map_err(|e|format!("prefix: {e:?}"))?;
        let point_id=NEXT_POINT.fetch_update(Ordering::Relaxed,Ordering::Relaxed,|n|n.checked_add(1)).map_err(|_|"fixture point identity exhausted")?;
        Ok(FixtureOutcome::Prefix(result,point_id))
    }
    /// Models only the admitted E59B40 writes on a disjoint owned output.
    /// Player state is explicitly stable through the source's later read.
    fn arbitrate(&self, outcome:FixtureOutcome,player:Handle,mut output:ContactOutput)->Result<FixtureArbitration,String> {
        self.resolve(player).ok_or("stale or foreign player")?;
        let (candidate,continuation,point_id)=match outcome {
            FixtureOutcome::Prefix(PrefixResult::PendingSubmission(c),point_id)=>(c,false,point_id),
            FixtureOutcome::Prefix(PrefixResult::PendingContinuationSubmission(c),point_id)=>(c,true,point_id),
            _=>return Err("no pending submission".into()),
        };
        for raw in candidate.actor_ids {
            if self.resolve(Handle{pool:self.pool_id,raw}).is_none() {return Err("stale candidate actor".into())}
        }
        let writes=arbitration::submit_contact_candidate(&mut output,candidate,player.raw);
        let continuation=if continuation {
            Some(ContinuationToken {actors:candidate.actor_ids.map(|raw|Handle{pool:self.pool_id,raw}),point_id})
        } else {None};
        Ok(FixtureArbitration{output,writes,continuation})
    }
    /// Fresh reads after arbitration; token is consumed and identities must
    /// match the retained fixture actors and synthetic point identity.
    fn resume_to_e673(&self, token:ContinuationToken, fresh:FreshRead)->Result<PendingE673,String> {
        if fresh.actors!=token.actors || fresh.point_id!=token.point_id {return Err("retained fixture identity mismatch".into())}
        for actor in token.actors {self.resolve(actor).ok_or("stale or foreign retained actor")?;}
        let distance=super::contact_call::distance(fresh.positions,fresh.control)?;
        Ok(PendingE673{actor_ids:token.actors.map(|a|a.raw),point_id:token.point_id,point:fresh.point,distance,current_word:fresh.current_word})
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum FixtureOutcome {Separated,Prefix(PrefixResult,u64)}
#[derive(Debug,PartialEq,Eq)]
struct FixtureArbitration {output:ContactOutput,writes:ArbitrationWrites,continuation:Option<ContinuationToken>}
#[derive(Debug,PartialEq,Eq)]
struct ContinuationToken {actors:[Handle;2],point_id:u64}
struct FreshRead {actors:[Handle;2],point_id:u64,positions:[[u32;3];2],point:[u32;3],current_word:u32,control:u16}
#[derive(Debug,PartialEq,Eq)]
struct PendingE673 {actor_ids:[u32;2],point_id:u64,point:[u32;3],distance:u32,current_word:u32}

/// Fixed-corpus adapter; creates only synthetic fixture actors.
pub(crate) fn coordinator_oracle(equal:bool,b4:[u32;2],selectors:[i32;2],float_bits:u32)->Result<Vec<u32>,String>{
    let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
    let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;
    let mut pool=Fixture::new()?;let geometry=Output{center:[0;3],scalar:0};
    let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
    let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
    pair_coordinator::oracle(&pool,a,b,equal,b4,selectors,float_bits)
}

/// Fixed synthetic corpus adapter; no native actor binding or event dispatch.
pub(crate) fn scripted_b7_oracle(player:u32,equal:bool)->Result<Vec<u32>,String>{
    let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
    let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;
    let geometry=Output{center:[0;3],scalar:0};
    let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
    pair_coordinator::scripted_b7_oracle(&pool,[a,b],player,equal)
}

pub(crate) fn selector19_oracle(bytes:[u8;2])->Result<[u32;10],String>{
    let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
    let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;
    let geometry=Output{center:[0;3],scalar:0};
    let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
    pair_coordinator::selector19_oracle(&pool,[a,b],bytes)
}

/// Independent synthetic fixture for the bounded native gate comparator.
pub(crate) fn case10_gate_oracle(bytes:[u8;4],relation:u8)->Result<[u32;2],String>{
 let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
 let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;
 let geometry=Output{center:[0;3],scalar:0};
 let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
 pair_coordinator::case10_oracle(&pool,a,b,bytes,relation)
}

/// Fixed synthetic actors for the event-store comparator.
pub(crate) fn case10_event_oracle(words:[u32;11])->Result<[u8;0x7c],String>{
 let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
 let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;let geometry=Output{center:[0;3],scalar:0};
 let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
 pair_coordinator::case10_event_oracle(&pool,[a,b],words)
}

/// Synthetic first/second actor fixture for E69B00 state-table comparison.
pub(crate) fn e69b_oracle(state:u32)->Result<[u32;2],String>{
 let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
 let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;let geometry=Output{center:[0;3],scalar:0};
 let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
 pair_coordinator::e69b_oracle(&pool,[a,b],state)
}

pub(crate) fn e69b_second_oracle(present:bool,first:u8,second:u8)->Result<[u32;4],String>{
 let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
 let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;let geometry=Output{center:[0;3],scalar:0};
 let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
 pair_coordinator::e69b_second_oracle(&pool,[a,b],present,first,second)
}

pub(crate) fn local_constructor_oracle(words:[u32;13])->Result<[u8;0x38],String>{
 let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
 let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;let geometry=Output{center:[0;3],scalar:0};
 let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
 pair_coordinator::local_constructor_oracle(&pool,[a,b],words)
}

pub(crate) fn fallback_oracle(local:[u8;0x38],source:[u8;0x38])->Result<[u8;0x38],String>{
 let rows=[spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}];
 let snapshot=FeatureSnapshot::new(7,&rows,[7,8,9])?;let mut pool=Fixture::new()?;let geometry=Output{center:[0;3],scalar:0};
 let a=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;let b=pool.bind([7,8,9],[7,8,9],geometry,snapshot)?;
 pair_coordinator::fallback_oracle(&pool,[a,b],local,source)
}

/// The snapshot was already admitted against the caller's post-override triple.
/// Pool creation, seed and lifetime are owned here and do not identify retail.
pub(super) fn inspect(constructor_triple:[u32;3], actor_triple:[u32;3],
    pre_override_geometry:Output, features:FeatureSnapshot<'_>, transform:Transform,
    supplied_second:Option<Transform>)->Result<String,String> {
    let mut fixture=Fixture::new()?;
    let second_features=features;
    let handle=fixture.bind(constructor_triple,actor_triple,pre_override_geometry,features)?;
    let actor=fixture.resolve(handle).ok_or("fresh fixture handle rejected")?;
    let eligible_rows:Vec<_>=actor.features.candidates().map(|(i,_)|i).collect();
    let projected:Vec<_>=actor.features.candidates().map(|(i,row)| feature_geometry::project(row,transform)
        .map(|shape|(i,shape)).map_err(|e|format!("feature {i}: {e:?}"))).collect::<Result<_,_>>()?;
    let pair=match (projected.first(),supplied_second) {
        (Some((index,first)),Some(second_transform)) => {
            let row=second_features.candidates().find(|(i,_)|i==index).ok_or("second feature missing")?.1;
            let second=feature_geometry::project_second(row,second_transform).map_err(|e|format!("second feature: {e:?}"))?;
            let prefix=feature_geometry::pair_prefix(*first,second).map_err(|e|format!("pair prefix: {e:?}"))?;
            let continuation=feature_geometry::continue_pair(*first,second,None).map_err(|e|format!("pair continuation: {e:?}"))?;
            Some((*index,second,prefix,continuation))
        },
        _=>None,
    };
    let first_index=eligible_rows.first().copied();
    let report=format!("owned_pool={} handle={:08X} constructor={:08X?} post_override={:08X?} pre_override_geometry={:?} eligible_rows={:?} statically_modeled_features={:?} supplied_second_pair_prefix={:?}",
        handle.pool,handle.raw,actor.constructor_triple,actor.actor_triple,actor.pre_override_geometry,eligible_rows,projected,pair);
    let producer=if let (Some(index),Some(second_transform))=(first_index,supplied_second) {
        let second_handle=fixture.bind(constructor_triple,actor_triple,pre_override_geometry,second_features).map_err(|e|format!("second fixture actor: {e}"))?;
        // Explicit synthetic definition codes and fixture player; source state is not inferred.
        let outcome=fixture.producer_prefix(handle,second_handle,handle,[index,index],[transform,second_transform],[2,7],None);
        let initial=ContactOutput{actor_ids:[0;2],selector:-1,feature_indices:[0;2],depth_bits:0,point_words:[0;3],normal_words:[0;3]};
        let arbitration=match &outcome {
            Ok(FixtureOutcome::Prefix(PrefixResult::PendingSubmission(_)|PrefixResult::PendingContinuationSubmission(_),_))=>
                Some(fixture.arbitrate(outcome.as_ref().unwrap().clone(),handle,initial)),
            _=>None,
        };
        // A separate B=6 fixture route, with newly supplied +90 words and a
        // fresh point snapshot, stops before E673A0 and all its effects.
        let pending=match fixture.producer_prefix(handle,second_handle,handle,[index,index],[transform,second_transform],[2,6],None) {
            Ok(o @ FixtureOutcome::Prefix(PrefixResult::PendingContinuationSubmission(_),_))=>Some((||->Result<_,String>{
                let mut result=fixture.arbitrate(o,handle,initial)?;
                let token=result.continuation.take().ok_or("missing continuation token")?;
                let fresh=FreshRead{actors:[handle,second_handle],point_id:token.point_id,
                    positions:[transform.position,second_transform.position],point:[101f32.to_bits(),100f32.to_bits(),0],current_word:0,control:0x027f};
                fixture.resume_to_e673(token,fresh)
            })()),
            _=>None,
        };
        let record_gate=pending.as_ref().and_then(|p|p.as_ref().ok()).map(|p| {
            let input=super::contact_record::Input{ids:p.actor_ids,point_id:p.point_id,point:p.point,distance:p.distance,current:p.current_word};
            // Caller-admitted synthetic bounds; stop before native provider.
            super::contact_record::bounds(input,[0;3],[200f32.to_bits();3])
        });
        let scripted_coordinator=pair_coordinator::inspect(&fixture,handle,second_handle)?;
        fixture.release(second_handle).map_err(|e|format!("second release: {e:?}"))?;
        Some((outcome,arbitration,pending,record_gate,scripted_coordinator))
    } else {None};
    fixture.release(handle).map_err(|e|format!("release: {e:?}"))?;
    if fixture.resolve(handle).is_some() { return Err("released fixture handle remained valid".into()); }
    Ok(format!("{report} fixture_producer_arbitration_and_pending={producer:?}; separate synthetic definitions=[2,7] and [2,6], first fixture actor is player, initial selector=-1, fresh +90 positions supplied from fixture transforms, fresh point=[101,100,0], current_word=0, synthetic bounds=[0,200]"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spore_core::cell_part_geometry::NativeCellPart;
    fn row(class:u32)->NativeCellPart { NativeCellPart { word_06_is_ffff:false,class,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3] } }
    fn geometry()->Output { Output{center:[1,2,3],scalar:4} }
    fn transform(x:f32)->Transform {Transform{position:[x.to_bits(),0,0],scale:1f32.to_bits(),flags:0,matrix:[0;9]}}
    #[test]
    fn checked_pool_rejects_cross_pool_and_stale_handles_and_retains_pre_override_geometry() {
        let rows=[row(1),row(2),row(3)];
        let snapshot=||FeatureSnapshot::new(7,&rows,[7,8,9]).unwrap();
        let mut a=Fixture::new().unwrap(); let mut b=Fixture::new().unwrap();
        let old=a.bind([6,1,2],[7,8,9],geometry(),snapshot()).unwrap();
        assert!(a.bind([6,1,2],[8,8,9],geometry(),snapshot()).is_err());
        let foreign=b.bind([6,1,2],[7,8,9],geometry(),snapshot()).unwrap();
        assert_eq!(old.raw,foreign.raw); assert!(a.resolve(foreign).is_none());
        let actor=a.resolve(old).unwrap();
        assert_eq!(actor.constructor_triple,[6,1,2]); assert_eq!(actor.actor_triple,[7,8,9]);
        assert_eq!(actor.pre_override_geometry,geometry()); assert_eq!(actor.features.candidates().map(|(i,_)|i).collect::<Vec<_>>(),vec![1,2]);
        a.release(old).unwrap();
        let replacement=a.bind([6,1,2],[7,8,9],geometry(),snapshot()).unwrap();
        assert_ne!(old.raw,replacement.raw); assert!(a.resolve(old).is_none());
        assert_eq!(a.release(old),Err(PoolError::InvalidHandle));
        assert_eq!(a.release(foreign),Err(PoolError::InvalidHandle));
        let second=a.bind([6,1,2],[7,8,9],geometry(),snapshot()).unwrap();
        assert!(a.bind([6,1,2],[7,8,9],geometry(),snapshot()).is_err());
        assert!(a.resolve(replacement).is_some()); assert!(a.resolve(second).is_some());
    }
    #[test]
    fn two_actor_prefix_preserves_identity_indices_and_stops_at_pending_calls() {
        let rows=[row(1),row(2),row(9)];
        let snapshot=||FeatureSnapshot::new(7,&rows,[7,8,9]).unwrap();
        let mut r=Fixture::new().unwrap();
        let a=r.bind([7,8,9],[7,8,9],geometry(),snapshot()).unwrap();
        let b=r.bind([7,8,9],[7,8,9],geometry(),snapshot()).unwrap();
        let outcome=r.producer_prefix(a,b,a,[1,1],[transform(0.),transform(1.)],[2,7],None).unwrap();
        let FixtureOutcome::Prefix(PrefixResult::PendingSubmission(candidate),_)=outcome else {panic!()};
        assert_eq!(candidate.actor_ids,[a.raw,b.raw]);assert_eq!(candidate.feature_indices,[1,1]);assert_eq!(candidate.selector,19);
        assert_eq!(candidate.point_words,[0.5f32.to_bits(),0,0]);assert_eq!(candidate.normal_words,[1f32.to_bits(),0,0]);
        let initial=ContactOutput{actor_ids:[0;2],selector:-1,feature_indices:[0;2],depth_bits:0,point_words:[0;3],normal_words:[0;3]};
        let selected=r.arbitrate(outcome,a,initial).unwrap();assert_eq!(selected.output,candidate);
        assert!(selected.writes.priority_or_depth);assert!(selected.continuation.is_none());
        let continuing=r.producer_prefix(b,a,a,[1,1],[transform(0.),transform(1.)],[2,6],None).unwrap();
        assert!(matches!(continuing,FixtureOutcome::Prefix(PrefixResult::PendingContinuationSubmission(c),_) if c.selector==1));
        let mut unchanged=r.arbitrate(continuing,a,candidate).unwrap();
        assert_eq!(unchanged.output,candidate);assert!(!unchanged.writes.priority_or_depth);
        let token=unchanged.continuation.take().unwrap();let point_id=token.point_id;
        let fresh=FreshRead{actors:[b,a],point_id,positions:[[3f32.to_bits(),4f32.to_bits(),0],[0;3]],point:[11,22,33],current_word:77,control:0x027f};
        let pending=r.resume_to_e673(token,fresh).unwrap();
        assert_eq!(pending,PendingE673{actor_ids:[b.raw,a.raw],point_id,point:[11,22,33],distance:5f32.to_bits(),current_word:77});
        let token=r.arbitrate(continuing,a,candidate).unwrap().continuation.unwrap();
        let fresh=FreshRead{actors:[a,b],point_id:token.point_id,positions:[[0;3];2],point:[0;3],current_word:0,control:0x027f};
        assert!(r.resume_to_e673(token,fresh).unwrap_err().contains("identity mismatch"));
        let token=r.arbitrate(continuing,a,candidate).unwrap().continuation.unwrap();
        let fresh=FreshRead{actors:[b,a],point_id:token.point_id.wrapping_add(1),positions:[[0;3];2],point:[0;3],current_word:0,control:0x027f};
        assert!(r.resume_to_e673(token,fresh).unwrap_err().contains("identity mismatch"));
        let token=r.arbitrate(continuing,a,candidate).unwrap().continuation.unwrap();
        let fresh=FreshRead{actors:[b,a],point_id:token.point_id,positions:[[0;3];2],point:[0;3],current_word:0,control:0x037f};
        assert!(r.resume_to_e673(token,fresh).unwrap_err().contains("027F"));
        assert!(matches!(r.producer_prefix(a,b,a,[2,1],[transform(0.),transform(1.)],[2,7],None).unwrap(),FixtureOutcome::Prefix(PrefixResult::FeatureNineExit,_)));
        assert!(matches!(r.producer_prefix(a,b,a,[1,1],[transform(0.),transform(1.)],[7,2],None).unwrap(),FixtureOutcome::Prefix(PrefixResult::DefinitionASevenExit,_)));
        assert!(r.producer_prefix(a,a,a,[1,1],[transform(0.),transform(1.)],[2,7],None).is_err());
        assert!(r.producer_prefix(a,b,a,[1,1],[transform(0.),transform(0.)],[2,7],None).unwrap_err().contains("local words"));
        let words=[1,2,3,4,5,6];
        let FixtureOutcome::Prefix(PrefixResult::PendingSubmission(c),_)=r.producer_prefix(a,b,a,[1,1],[transform(0.),transform(0.)],[2,7],Some(words)).unwrap() else {panic!()};
        assert_eq!(c.normal_words,[1,2,3]);assert_eq!(c.point_words,[4,5,6]);
        let mut foreign=Fixture::new().unwrap();let f=foreign.bind([7,8,9],[7,8,9],geometry(),snapshot()).unwrap();
        assert!(r.producer_prefix(a,f,a,[1,1],[transform(0.),transform(1.)],[2,7],None).is_err());
        let token=r.arbitrate(continuing,a,candidate).unwrap().continuation.unwrap();
        let fresh=FreshRead{actors:[b,a],point_id:token.point_id,positions:[[0;3];2],point:[0;3],current_word:0,control:0x027f};
        r.release(b).unwrap();assert!(r.resume_to_e673(token,fresh).unwrap_err().contains("stale"));
        assert!(r.producer_prefix(a,b,a,[1,1],[transform(0.),transform(1.)],[2,7],None).is_err());
        assert!(r.arbitrate(outcome,a,initial).unwrap_err().contains("stale candidate"));
    }
}
