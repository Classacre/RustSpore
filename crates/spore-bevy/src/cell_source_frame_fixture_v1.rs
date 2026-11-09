//! One scheduled frame using one owned fixture registry. Inputs and player
//! selection are synthetic admissions; no native actors or gameplay are joined.
use super::*;

#[derive(Debug,PartialEq,Eq)]
enum PairBoundary { RejectedBroadphase, PendingE7F170, UnsupportedBroadphase(String) }
#[derive(Debug,PartialEq,Eq)]
struct PairDiagnostic { first:ActorHandle, second:ActorHandle, boundary:PairBoundary }
#[derive(Debug,PartialEq,Eq)]
struct FrameDiagnostic { player:ActorHandle, fresh_registration_slots:Vec<(ActorHandle,u32)>, fresh_sorted:spore_core::cell_fresh_sweep_registration_v1::SortedFresh, pairs:Vec<PairDiagnostic>, suppressed:usize, pending:&'static str }
#[derive(Resource)]
struct FrameFixture { registry:Registry, pair_capacity:usize, requested:bool, runs:u32, result:Option<Result<FrameDiagnostic,String>> }
impl FrameFixture {
 /// All snapshots and the index selecting the player are explicitly supplied.
 fn new(actors:Vec<ActorSnapshot>,player_index:usize,pool_counter:u32,pair_capacity:usize)->Result<Self,String>{
  if player_index>=actors.len(){return Err("explicit player index is outside fixture actors".into())}
  // Admit endpoint ordering/ties for the whole supplied frame before creating
  // any registry identities. This is not persistent source registration.
  let slots:Vec<_>=actors.iter().enumerate().map(|(i,a)|SweepSlot{id:(i+1) as i32,tag:a.tag,min:a.min,max:a.max}).collect();
  sweep::rebuild(&slots,pair_capacity).map_err(|e|format!("bounds: {e:?}"))?;
  let mut registry=Registry::new(actors.len(),pool_counter).map_err(|e|format!("registry: {e:?}"))?;
  let mut handles=Vec::with_capacity(actors.len());
  for actor in actors {handles.push(registry.insert(actor).map_err(|e|format!("actor: {e:?}"))?)}
  registry.set_player(handles[player_index]).map_err(|e|format!("player: {e:?}"))?;
  Ok(Self{registry,pair_capacity,requested:true,runs:0,result:None})
 }
}
fn one_frame(mut state:ResMut<FrameFixture>){
 if !state.requested{return}
 state.requested=false;state.runs+=1;
 let result=(||->Result<FrameDiagnostic,String>{
  let player=state.registry.player.ok_or("fixture player not admitted")?;
  state.registry.resolve(player).map_err(|e|format!("player: {e:?}"))?;
  let (fresh_registration_slots,ordered,suppressed,fresh_sorted)=state.registry.update_fresh_registration(state.pair_capacity)?;
  let pairs=ordered.into_iter().map(|(first,second,gate)| PairDiagnostic{first,second,boundary:match gate {
   Ok(false)=>PairBoundary::RejectedBroadphase,
   Ok(true)=>PairBoundary::PendingE7F170,
   Err(e)=>PairBoundary::UnsupportedBroadphase(format!("{e:?}")),
  }}).collect();
  Ok(FrameDiagnostic{player,fresh_registration_slots,fresh_sorted,pairs,suppressed,pending:"source actor creation, persistent sweep lifecycle, E7F170 selector/contact producers, callbacks and event dispatch"})
 })();
 state.result=Some(result);
}
fn app_for(state:FrameFixture)->App {let mut app=App::new();app.insert_resource(state).add_systems(Update,one_frame);app}

pub(super) fn run(args:&[String])->Result<(),String>{
 if !args.is_empty(){return Err("source frame fixture takes no arguments; all actors are synthetic".into())}
 let mut app=app_for(FrameFixture::new(vec![fixture_actor(1),fixture_actor(2)],0,0,4)?);
 app.update();let frame=app.world_mut().resource_mut::<FrameFixture>().result.take().ok_or("scheduled frame did not run")??;
 println!("SOURCE_FRAME_FIXTURE controlled synthetic frame={frame:?}; one registry; actor snapshots and player index supplied; no native or gameplay identity");Ok(())
}

/// Builds only a checked fixture actor. The cache key and empty saved state are
/// explicit admissions; neither is evidence of a live actor/cache or E829D0.
fn staged_actor(definition:DefinitionIdentity,source:ControlledStagedSource,saved:[u32;3],geometry:spore_core::cell_actor_geometry_projection_v1::Output,tag:u32,min:[f32;2],max:[f32;2])->Result<ActorSnapshot,String>{
 if saved!=[0;3]{return Err("saved-player override requires fresh source actor/cache; unsupported".into())}
 if definition.package.is_empty()||definition.raw.is_empty()||<[u8;32]>::from(Sha256::digest(&definition.raw))!=definition.digest{return Err("definition provenance or digest mismatch".into())}
 if source.creation_sha.len()!=64||!source.creation_sha.bytes().all(|b|b.is_ascii_hexdigit()){return Err("creation digest malformed".into())}
 crate::cell_source_capability_v1::FeatureSnapshot::new(source.cache_first,&source.rows,source.triple).map_err(|e|e.to_string())?;
 // Check finite intervals and endpoint admission before creating a registry.
 sweep::rebuild(&[SweepSlot{id:1,tag,min,max}],1).map_err(|e|format!("bounds: {e:?}"))?;
 Ok(ActorSnapshot{staged:Some(source),definition,health_244:6,tag,min,max,geometry,position_words:[100f32.to_bits(),100f32.to_bits(),0]})
}
#[derive(Clone,Copy,Debug)]
struct PeerAdmission {position:[f32;3],center:[f32;3],scalar:f32,tag:u32,min:[f32;2],max:[f32;2]}
fn peer_actor(first:&ActorSnapshot,input:PeerAdmission)->Result<ActorSnapshot,String>{
 if input.position.iter().chain(input.center.iter()).chain([&input.scalar]).any(|v|!v.is_finite()||v.is_subnormal())||input.scalar<=0.0{return Err("peer position/geometry must be finite normal or zero; scalar must be positive".into())}
 let mut src=first.staged.clone().ok_or("first actor has no staged provenance")?;src.geometry_admission="caller_supplied";
 let geometry=spore_core::cell_actor_geometry_projection_v1::Output{center:input.center.map(f32::to_bits),scalar:input.scalar.to_bits()};
 let mut actor=staged_actor(first.definition.clone(),src,[0;3],geometry,input.tag,input.min,input.max)?;
 actor.position_words=input.position.map(f32::to_bits);Ok(actor)
}

/// A separately admitted synthetic two-actor manager snapshot. Registry handle
/// resolution is checked, but the actor-to-source-manager lifetime is only an
/// explicit caller admission. No native actor or cache identity is inferred.
fn actor_multiframe(registry:&Registry,old_handle:ActorHandle,new_handle:ActorHandle,stable:bool)->Result<spore_core::cell_sweep_multiframe_diagnostic_v1::Diagnostic,String>{
 if !stable{return Err("stable actor/manager ownership admission required".into())}
 if old_handle==new_handle||registry.player!=Some(old_handle){return Err("distinct player and peer required".into())}
 let old=registry.resolve(old_handle).map_err(|e|format!("old actor: {e:?}"))?;let new=registry.resolve(new_handle).map_err(|e|format!("new actor: {e:?}"))?;
 let mut first=spore_core::cell_fresh_sweep_registration_v1::FreshCollection::new(1).map_err(|e|format!("first: {e:?}"))?;
 let mut added=spore_core::cell_fresh_sweep_registration_v1::FreshCollection::new(1).map_err(|e|format!("added: {e:?}"))?;
 first.append(old_handle.raw,old.tag,old.min.map(f32::to_bits),old.max.map(f32::to_bits)).map_err(|e|format!("first append: {e:?}"))?;
 added.append(new_handle.raw,new.tag,new.min.map(f32::to_bits),new.max.map(f32::to_bits)).map_err(|e|format!("peer append: {e:?}"))?;
 let bounds=|slot,geometry|->Result<([u32;2],[u32;2]),String>{
  let input=spore_core::cell_actor_frame_update_v1::Input{slot,manager_token:1,geometry:Some(geometry),stable_disjoint:true};
  match spore_core::cell_actor_frame_update_v1::evaluate(input).map_err(|e|format!("actor update: {e:?}"))?{
   spore_core::cell_actor_frame_update_v1::Handoff::PendingUpdate{manager_token:1,slot:actual,min,max} if actual==slot=>Ok((min,max)),_=>Err("unexpected actor update handoff".into())
  }
 };
 let (old_min,old_max)=bounds(0,old.geometry)?;let (new_min,new_max)=bounds(1,new.geometry)?;
 let changes=[spore_core::cell_sweep_two_frame_diagnostic_v1::PendingBounds{slot:0,min:new_min,max:new_max}];
 spore_core::cell_sweep_multiframe_diagnostic_v1::inspect(&first,&added,0,old_min,old_max,&changes,4,spore_core::cell_sweep_pending_update_v1::X87Admission::Default027f).map_err(|e|format!("multiframe: {e:?}"))
}

/// Route-only handoff on a previously passing synthetic broadphase pair. The
/// four actor words are caller-supplied stable snapshots, not pointer proof.
fn selector_routes(registry:&Registry,pair:&PairDiagnostic,words:[spore_core::cell_selector_route_v1::ActorWords;2],stable:bool)->Result<[spore_core::cell_selector_route_v1::Route;2],String>{
 if !stable{return Err("selector actor lifetime and word stability admission required".into())}
 if pair.boundary!=PairBoundary::PendingE7F170||pair.first==pair.second{return Err("passing distinct pair required".into())}
 registry.resolve(pair.first).map_err(|e|format!("first actor: {e:?}"))?;registry.resolve(pair.second).map_err(|e|format!("second actor: {e:?}"))?;
 Ok([spore_core::cell_selector_route_v1::route(words[0],words[1]),spore_core::cell_selector_route_v1::route(words[1],words[0])])
}

/// Classes are separately supplied synthetic cache-row words. No retained
/// feature rows or package bytes are asserted to be native cache contents.
fn selector_feature_pairs(registry:&Registry,pair:&PairDiagnostic,words:[spore_core::cell_selector_route_v1::ActorWords;2],classes:[&[u32];2],actor_stable:bool,cache_stable:bool)->Result<[Vec<(u32,u32)>;2],String>{
 let routes=selector_routes(registry,pair,words,actor_stable)?;
 if routes!=[spore_core::cell_selector_route_v1::Route::BothFeatures;2]{return Err("both-feature routes required".into())}
 let counts=[i32::try_from(classes[0].len()).map_err(|_|"first count overflow")?,i32::try_from(classes[1].len()).map_err(|_|"second count overflow")?];
 let a=spore_core::cell_feature_loop_v1::pairs(counts,classes,cache_stable).map_err(|e|format!("feature loop: {e:?}"))?;
 let b=spore_core::cell_feature_loop_v1::pairs([counts[1],counts[0]],[classes[1],classes[0]],cache_stable).map_err(|e|format!("feature loop reverse: {e:?}"))?;
 Ok([a,b])
}
fn parse_feature_classes(value:&str)->Result<Vec<u32>,String>{if value=="-"{return Ok(Vec::new())}let v:Vec<u32>=value.split(',').map(|x|x.parse().map_err(|_|"invalid feature class".to_string())).collect::<Result<_,_>>()?;if v.len()>64{return Err("feature class count exceeds 64".into())}Ok(v)}

/// Exact staged inspection and its same-read CELL bytes are owned by the frame.
/// Only the explicitly controlled empty-save branch is admitted.
pub(super) fn run_installed(args:&[String])->Result<(),String>{
 let mut install=None;let mut seed=None;let mut progress=None;let mut tag=None;let mut bounds=None;let mut peer=None;let mut stable=false;let mut source_registration=false;let mut registration_stable=false;let mut multiframe=false;let mut actor_manager_stable=false;let mut selector_words=None;let mut selector_stable=false;let mut feature_classes:Option<[Vec<u32>;2]>=None;let mut feature_cache_stable=false;let mut i=0;
 while i<args.len(){
  match args[i].as_str(){
   "--feature-classes"=>{if feature_classes.is_some()||i+2>=args.len(){return Err("--feature-classes requires player and peer comma-separated class words once (- for empty)".into())}feature_classes=Some([parse_feature_classes(&args[i+1])?,parse_feature_classes(&args[i+2])?]);i+=3;},
   "--admit-feature-cache-stability"=>{if feature_cache_stable{return Err("duplicate --admit-feature-cache-stability".into())}feature_cache_stable=true;i+=1;},
   "--selector-route-words"=>{if selector_words.is_some()||i+4>=args.len(){return Err("--selector-route-words requires player +248/+FC and peer +248/+FC once".into())}let mut w=[0u32;4];for j in 0..4{w[j]=args[i+1+j].parse().map_err(|_|"invalid selector actor word")?;}selector_words=Some(w);i+=5;},
   "--admit-selector-actor-lifetime-and-word-stability"=>{if selector_stable{return Err("duplicate --admit-selector-actor-lifetime-and-word-stability".into())}selector_stable=true;i+=1;},
   "--multiframe-update"=>{if multiframe{return Err("duplicate --multiframe-update".into())}multiframe=true;i+=1;},
   "--admit-actor-manager-stability"=>{if actor_manager_stable{return Err("duplicate --admit-actor-manager-stability".into())}actor_manager_stable=true;i+=1;},
   "--source-registration"=>{if source_registration{return Err("duplicate --source-registration".into())}source_registration=true;i+=1;},
   "--admit-registration-stability"=>{if registration_stable{return Err("duplicate --admit-registration-stability".into())}registration_stable=true;i+=1;},
   "--admit-stable-files"=>{if stable{return Err("duplicate --admit-stable-files".into())}stable=true;i+=1;},
   "--peer"=>{
    if peer.is_some()||i+12>=args.len(){return Err("--peer requires position(3), center(3), scalar, tag, bounds(4) once".into())}
    let mut v=[0f32;11];for j in 0..7{v[j]=args[i+1+j].parse().map_err(|_|"invalid peer position or geometry")?;}
    let peer_tag=args[i+8].parse::<u32>().map_err(|_|"invalid peer tag")?;
    for j in 0..4{v[7+j]=args[i+9+j].parse().map_err(|_|"invalid peer bounds")?;}
    peer=Some(PeerAdmission{position:[v[0],v[1],v[2]],center:[v[3],v[4],v[5]],scalar:v[6],tag:peer_tag,min:[v[7],v[8]],max:[v[9],v[10]]});i+=13;
   },
   "--bounds"=>{if bounds.is_some()||i+4>=args.len(){return Err("--bounds requires four values once".into())}let mut v=[0f32;4];for j in 0..4 {v[j]=args[i+1+j].parse().map_err(|_|"invalid bounds")?;}bounds=Some(v);i+=5;},
   "--install"|"--seed"|"--progress"|"--tag"=>{
    let value=args.get(i+1).ok_or("missing option value")?;
    match args[i].as_str(){
     "--install"=>{if install.replace(value.clone()).is_some(){return Err("duplicate --install".into())}},
     "--seed"=>{if seed.replace(value.parse::<u32>().map_err(|_|"invalid seed")?).is_some(){return Err("duplicate --seed".into())}},
     "--progress"=>{if progress.replace(value.parse::<i32>().map_err(|_|"invalid progress")?).is_some(){return Err("duplicate --progress".into())}},
     _=>{if tag.replace(value.parse::<u32>().map_err(|_|"invalid tag")?).is_some(){return Err("duplicate --tag".into())}},
    }i+=2;
   },
   _=>return Err(format!("unsupported option {} (saved overrides are not admitted)",args[i])),
  }
 }
 if feature_classes.is_some()&&!feature_cache_stable{return Err("--feature-classes requires --admit-feature-cache-stability".into())}
 if feature_cache_stable&&feature_classes.is_none(){return Err("--admit-feature-cache-stability requires --feature-classes".into())}
 if feature_classes.is_some()&&selector_words.is_none(){return Err("--feature-classes requires --selector-route-words".into())}
 if selector_words.is_some()&&!selector_stable{return Err("--selector-route-words requires --admit-selector-actor-lifetime-and-word-stability".into())}
 if selector_stable&&selector_words.is_none(){return Err("--admit-selector-actor-lifetime-and-word-stability requires --selector-route-words".into())}
 if selector_words.is_some()&&peer.is_none(){return Err("--selector-route-words requires --peer".into())}
 if multiframe&&!actor_manager_stable{return Err("--multiframe-update requires --admit-actor-manager-stability".into())}
 if actor_manager_stable&&!multiframe{return Err("--admit-actor-manager-stability requires --multiframe-update".into())}
 if multiframe&&peer.is_none(){return Err("--multiframe-update requires --peer".into())}
 if !stable{return Err("--admit-stable-files required; before/after hashes are not an atomic snapshot".into())}
 let install=install.ok_or("--install required")?;let seed=seed.ok_or("--seed required")?;let progress=progress.ok_or("--progress required")?;
 if source_registration {if !registration_stable{return Err("--admit-registration-stability required for source registration".into())}if tag.is_some()||bounds.is_some(){return Err("--source-registration conflicts with --tag/--bounds".into())}}
 else if registration_stable{return Err("--admit-registration-stability requires --source-registration".into())}
 if !source_registration&&(tag.is_none()||bounds.is_none()){return Err("caller mode requires --tag and --bounds".into())}
 let staged=crate::cell_source_player_v1::inspect(std::path::Path::new(&install),seed,progress)?;
 // This validates the modeled first-word binding before transferring owned rows.
 staged.capability.feature_snapshot(staged.triple).map_err(|e|e.to_string())?;
 let source=ControlledStagedSource{triple:staged.triple,cache_first:staged.triple[0],creation_sha:staged.creation_sha.into(),rows:staged.capability.rows().to_vec(),geometry_admission:"modeled_pre_override"};
 let provenance=staged.definition;
 let definition=DefinitionIdentity{package:provenance.package.to_string_lossy().into_owned(),key:provenance.key,digest:provenance.sha256,raw:provenance.bytes};
 let (tag,b)=if source_registration {
  let input=spore_core::cell_registration_prefix_v1::Input{original_manager:0,current_manager:0,argument10:0,definition_b4:2,actor_handle:0,geometry:staged.projected,post_e86960_stable_disjoint:registration_stable};
  match spore_core::cell_registration_prefix_v1::evaluate(input).map_err(|e|format!("registration prefix: {e:?}"))? {
   spore_core::cell_registration_prefix_v1::ResultV1::Register{tag,min,max}=>(tag,[f32::from_bits(min[0]),f32::from_bits(min[1]),f32::from_bits(max[0]),f32::from_bits(max[1])]),
   spore_core::cell_registration_prefix_v1::ResultV1::Skipped=>return Err("registration manager mismatch".into()),
  }
 }else{(tag.unwrap(),bounds.unwrap())};
 let actor=staged_actor(definition,source,[0;3],staged.projected,tag,[b[0],b[1]],[b[2],b[3]])?;
 let mut actors=vec![actor];if let Some(input)=peer {actors.push(peer_actor(&actors[0],input)?)}
 let mut app=app_for(FrameFixture::new(actors,0,0,4)?);app.update();
 let mut state=app.world_mut().resource_mut::<FrameFixture>();
 let result=state.result.take().ok_or("scheduled frame did not run")??;
 let actor=state.registry.resolve(result.player).map_err(|e|format!("fixture player: {e:?}"))?;
 let source=actor.staged.as_ref().ok_or("missing staged source")?;
 println!("SOURCE_INSTALLED_FRAME controlled synthetic_registry_handle={:08X} seed={} progress={} cell_package={} cell_key={:08X?} cell_sha256={:x} cell_bytes={} modeled_creation_triple={:08X?} creation_sha256={} retained_rows={} cache_first={:08X} modeled_pre_override_center={:08X?} modeled_pre_override_scalar={:08X} synthetic_position=[100,100,0] tag={} bounds={:?} registration_mode={} frame={:?}; saved=[0,0,0], E829D0 pending; file stability explicitly admitted; source actor/cache identity, persistent registration, selector/contact producer, callback and event pending; no live gameplay claim",result.player.raw,seed,progress,actor.definition.package,actor.definition.key,Sha256::digest(&actor.definition.raw),actor.definition.raw.len(),source.triple,source.creation_sha,source.rows.len(),source.cache_first,actor.geometry.center,actor.geometry.scalar,tag,b,if source_registration{"controlled_E74F_prefix_same_manager_post_E86960_stable_disjoint"}else{"caller_supplied"},result);
 if let Some(w)=selector_words {
  let pair=result.pairs.iter().find(|p|p.boundary==PairBoundary::PendingE7F170).ok_or("selector route requires a passing broadphase pair")?;
  let player=spore_core::cell_selector_route_v1::ActorWords{word_248:w[0],word_fc:w[1]};let peer=spore_core::cell_selector_route_v1::ActorWords{word_248:w[2],word_fc:w[3]};
  let ordered=if pair.first==result.player{[player,peer]}else if pair.second==result.player{[peer,player]}else{return Err("selector pair does not contain fixture player".into())};
  let routes=selector_routes(&state.registry,pair,ordered,selector_stable)?;
  println!("SOURCE_INSTALLED_SELECTOR_ROUTES controlled synthetic_pair=[{:08X},{:08X}] words=caller_supplied routes={routes:?}; actor lifetime and word stability explicitly admitted; route callees/contact/feeding/callbacks not executed; no live actor claim",pair.first.raw,pair.second.raw);
  if let Some(classes)=feature_classes.as_ref(){
   let ordered_classes=if pair.first==result.player{[classes[0].as_slice(),classes[1].as_slice()]}else{[classes[1].as_slice(),classes[0].as_slice()]};
   let pairs=selector_feature_pairs(&state.registry,pair,ordered,ordered_classes,selector_stable,feature_cache_stable)?;
   println!("SOURCE_INSTALLED_FEATURE_LOOP controlled synthetic_pair=[{:08X},{:08X}] classes=caller_supplied cache_stability=admitted ordered_indices={pairs:?}; geometry/contact/feeding/callbacks not executed; no live cache or actor claim",pair.first.raw,pair.second.raw);
  }
 }
 if multiframe {
  let peer_handle=result.fresh_registration_slots.iter().map(|(h,_)|*h).find(|h|*h!=result.player).ok_or("multiframe peer missing")?;
  let diagnostic=actor_multiframe(&state.registry,result.player,peer_handle,actor_manager_stable)?;
  println!("SOURCE_INSTALLED_MULTIFRAME controlled synthetic_player={:08X} synthetic_peer={:08X} manager_token=1 geometry=admitted_actor_snapshots diagnostic={diagnostic:?}; actor/manager stability explicitly admitted; sorted/pending checked models, independent pair sort; no live identity or lifecycle claim",result.player.raw,peer_handle.raw);
 }
 for (raw,actor) in state.registry.pool.iter(){
  let src=actor.staged.as_ref().ok_or("missing actor provenance")?;
  println!("SOURCE_INSTALLED_FRAME_ACTOR controlled synthetic_handle={raw:08X} definition=shared_same_inspection cell_key={:08X?} cell_sha256={:x} creation_triple={:08X?} creation_sha256={} geometry_admission={} actor_position_bits={:08X?} center_bits={:08X?} scalar_bits={:08X} tag={} bounds=[{:?},{:?}]; no live actor or distinct installed creature claim",actor.definition.key,Sha256::digest(&actor.definition.raw),src.triple,src.creation_sha,src.geometry_admission,actor.position_words,actor.geometry.center,actor.geometry.scalar,actor.tag,actor.min,actor.max);
 }
 Ok(())
}

#[cfg(test)]
mod tests {
 #[test]fn feature_loop_requires_both_routes_stable_caches_and_live_pair(){use super::*;use spore_core::cell_selector_route_v1::ActorWords;let mut r=Registry::new(2,0).unwrap();let a=r.insert(fixture_actor(1)).unwrap();let b=r.insert(fixture_actor(2)).unwrap();let pair=PairDiagnostic{first:a,second:b,boundary:PairBoundary::PendingE7F170};let yes=ActorWords{word_248:1,word_fc:1};let no=ActorWords{word_248:0,word_fc:1};assert!(selector_feature_pairs(&r,&pair,[yes,yes],[&[0,1,2],&[1,0]],true,false).is_err());assert!(selector_feature_pairs(&r,&pair,[yes,no],[&[0],&[0]],true,true).is_err());let out=selector_feature_pairs(&r,&pair,[yes,yes],[&[0,1,2],&[1,0]],true,true).unwrap();assert_eq!(out[0],[(0,1),(2,1)]);assert_eq!(out[1],[(1,0),(1,2)]);r.release(b).unwrap();assert!(selector_feature_pairs(&r,&pair,[yes,yes],[&[0],&[0]],true,true).is_err());}
 #[test]fn feature_loop_cli_requires_explicit_classes_and_stability(){use super::*;let a=|v:&[&str]|v.iter().map(|x|x.to_string()).collect::<Vec<_>>();assert!(run_installed(&a(&["--feature-classes","1,2","-"])).unwrap_err().contains("admit-feature-cache-stability"));assert!(run_installed(&a(&["--admit-feature-cache-stability"])).unwrap_err().contains("requires --feature-classes"));assert!(parse_feature_classes("0,bad").is_err());assert!(parse_feature_classes(&vec!["0";65].join(",")).is_err());}
 #[test] fn selector_routes_require_passing_pair_stable_words_and_live_checked_handles(){use super::*;use spore_core::cell_selector_route_v1::{ActorWords,Route};let mut registry=Registry::new(2,0).unwrap();let a=registry.insert(fixture_actor(1)).unwrap();let b=registry.insert(fixture_actor(2)).unwrap();let mut pair=PairDiagnostic{first:a,second:b,boundary:PairBoundary::PendingE7F170};let words=[ActorWords{word_248:1,word_fc:0},ActorWords{word_248:u32::MAX,word_fc:1}];assert!(selector_routes(&registry,&pair,words,false).is_err());assert_eq!(selector_routes(&registry,&pair,words,true).unwrap(),[Route::SecondFeatures,Route::FirstFeatures]);pair.boundary=PairBoundary::RejectedBroadphase;assert!(selector_routes(&registry,&pair,words,true).is_err());pair.boundary=PairBoundary::PendingE7F170;registry.release(b).unwrap();assert!(selector_routes(&registry,&pair,words,true).is_err());}
 #[test] fn selector_route_cli_requires_words_stability_and_peer(){use super::*;let a=|v:&[&str]|v.iter().map(|x|x.to_string()).collect::<Vec<_>>();assert!(run_installed(&a(&["--selector-route-words","1","0","1","1"])).unwrap_err().contains("admit-selector-actor-lifetime-and-word-stability"));assert!(run_installed(&a(&["--admit-selector-actor-lifetime-and-word-stability"])).unwrap_err().contains("requires --selector-route-words"));assert!(run_installed(&a(&["--selector-route-words","1","0","1","1","--admit-selector-actor-lifetime-and-word-stability"])).unwrap_err().contains("requires --peer"));}
 #[test] fn multiframe_admits_checked_synthetic_actors_and_rejects_stale_or_unadmitted(){
  use super::*;
  let mut r=Registry::new(2,0).unwrap();let mut a=fixture_actor(1);a.min=[0.,0.];a.max=[5.,6.];a.geometry=spore_core::cell_actor_geometry_projection_v1::Output{center:[2f32.to_bits(),2f32.to_bits(),0],scalar:1f32.to_bits()};
  let mut b=fixture_actor(2);b.min=[8.,8.];b.max=[9.,9.];b.geometry=spore_core::cell_actor_geometry_projection_v1::Output{center:[2.5f32.to_bits(),2.5f32.to_bits(),0],scalar:0.25f32.to_bits()};
  let first=r.insert(a).unwrap();let peer=r.insert(b).unwrap();r.set_player(first).unwrap();assert!(actor_multiframe(&r,first,peer,false).is_err());let out=actor_multiframe(&r,first,peer,true).unwrap();assert_eq!(out.first.axes[0][0].value_bits,0f32.to_bits());assert_eq!(out.updated_old.axes[0][0].value_bits,1f32.to_bits());assert_eq!(out.combined.axes[0][1].value_bits,2.25f32.to_bits());assert_eq!(out.pairs.stored_order().len(),1);r.release(peer).unwrap();assert!(actor_multiframe(&r,first,peer,true).is_err());
 }
 #[test] fn multiframe_cli_requires_explicit_admissions(){use super::*;let args=|x:&[&str]|x.iter().map(|s|s.to_string()).collect::<Vec<_>>();assert!(run_installed(&args(&["--multiframe-update"])).unwrap_err().contains("admit-actor-manager-stability"));assert!(run_installed(&args(&["--admit-actor-manager-stability"])).unwrap_err().contains("requires --multiframe-update"));assert!(run_installed(&args(&["--multiframe-update","--admit-actor-manager-stability"])).unwrap_err().contains("requires --peer"));}
 use super::*;
 #[test] fn scheduled_frame_runs_once_and_stops_at_source_producer(){
  let mut app=app_for(FrameFixture::new(vec![fixture_actor(1),fixture_actor(2)],0,0,4).unwrap());
  app.update();let state=app.world().resource::<FrameFixture>();assert_eq!(state.runs,1);
  let result=state.result.as_ref().unwrap().as_ref().unwrap();assert_eq!(result.pairs.len(),1);assert_eq!(result.pairs[0].boundary,PairBoundary::PendingE7F170);
  assert_eq!(result.fresh_sorted.slots.len(),2);assert_eq!(result.fresh_sorted.axes[0].len(),4);assert_eq!(result.fresh_registration_slots.len(),2);assert_eq!(result.fresh_registration_slots[0].1,0);assert_eq!(result.fresh_registration_slots[1].1,1);
  assert_eq!(result.player.registry,result.pairs[0].first.registry);assert_eq!(result.player.registry,result.pairs[0].second.registry);
  assert!(result.pending.contains("selector/contact producers"));app.update();assert_eq!(app.world().resource::<FrameFixture>().runs,1);
 }
 #[test] fn capacity_and_strict_gate_are_explicit(){
  let mut app=app_for(FrameFixture::new(vec![fixture_actor(1),fixture_actor(2)],0,0,0).unwrap());app.update();
  let result=app.world().resource::<FrameFixture>().result.as_ref().unwrap().as_ref().unwrap();assert!(result.pairs.is_empty());assert_eq!(result.suppressed,1);
  let mut far=fixture_actor(2);far.geometry.center[0]=4f32.to_bits();
  let mut app=app_for(FrameFixture::new(vec![fixture_actor(1),far],0,0,4).unwrap());app.update();assert_eq!(app.world().resource::<FrameFixture>().result.as_ref().unwrap().as_ref().unwrap().pairs[0].boundary,PairBoundary::RejectedBroadphase);
 }
 #[test] fn stale_foreign_and_reused_player_do_not_run_a_frame(){
  let mut state=FrameFixture::new(vec![fixture_actor(1),fixture_actor(2)],0,0,4).unwrap();
  let old=state.registry.player.unwrap();state.registry.release(old).unwrap();let newer=state.registry.insert(fixture_actor(3)).unwrap();assert_ne!(old.raw,newer.raw);
  state.registry.player=Some(old);let mut app=app_for(state);app.update();assert!(app.world().resource::<FrameFixture>().result.as_ref().unwrap().is_err());
  let mut state=FrameFixture::new(vec![fixture_actor(1),fixture_actor(2)],0,0,4).unwrap();let foreign=FrameFixture::new(vec![fixture_actor(1)],0,0,1).unwrap();
  state.registry.player=foreign.registry.player;let mut app=app_for(state);app.update();assert!(app.world().resource::<FrameFixture>().result.as_ref().unwrap().is_err());
  assert!(FrameFixture::new(vec![fixture_actor(1)],1,0,1).is_err());
 }
 fn admitted_parts()->(DefinitionIdentity,ControlledStagedSource,spore_core::cell_actor_geometry_projection_v1::Output){
  let raw=vec![1,2,3];let definition=DefinitionIdentity{package:"controlled".into(),key:[1,2,3],digest:Sha256::digest(&raw).into(),raw};
  let row=spore_core::cell_part_geometry::NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]};
  let source=ControlledStagedSource{triple:[7,8,9],cache_first:7,creation_sha:"ab".repeat(32),rows:vec![row],geometry_admission:"modeled_pre_override"};
  let geometry=spore_core::cell_actor_geometry_projection_v1::Output{center:[100f32.to_bits(),100f32.to_bits(),0],scalar:1f32.to_bits()};(definition,source,geometry)
 }
 #[test] fn installed_adapter_preserves_owned_identity_and_rejects_inconsistent_admissions(){
  let (def,src,geometry)=admitted_parts();let actor=staged_actor(def.clone(),src.clone(),[0;3],geometry,4,[0.,0.],[2.,2.]).unwrap();
  let staged=actor.staged.as_ref().unwrap();assert_eq!(staged.triple,[7,8,9]);assert_eq!(staged.rows.len(),1);assert_eq!(actor.definition,def);assert_eq!(actor.geometry,geometry);
  let mut app=app_for(FrameFixture::new(vec![actor],0,0,4).unwrap());app.update();let state=app.world().resource::<FrameFixture>();let result=state.result.as_ref().unwrap().as_ref().unwrap();assert!(result.pairs.is_empty());assert_eq!(state.registry.resolve(result.player).unwrap().staged.as_ref().unwrap().triple,[7,8,9]);
  assert!(staged_actor(def.clone(),src.clone(),[7,8,9],geometry,0,[0.,0.],[2.,2.]).is_err());
  let mut wrong=src.clone();wrong.cache_first=8;assert!(staged_actor(def.clone(),wrong,[0;3],geometry,0,[0.,0.],[2.,2.]).is_err());
  let mut wrong=def.clone();wrong.raw[0]^=1;assert!(staged_actor(wrong,src.clone(),[0;3],geometry,0,[0.,0.],[2.,2.]).is_err());
  assert!(staged_actor(def,src,[0;3],geometry,0,[0.,0.],[0.,2.]).is_err());
 }
 #[test] fn two_source_backed_fixture_actors_share_provenance_but_not_identity(){
  let (def,src,geometry)=admitted_parts();let first=staged_actor(def.clone(),src.clone(),[0;3],geometry,0,[0.,0.],[2.,2.]).unwrap();
  let peer=PeerAdmission{position:[200.,201.,0.],center:[100.5,100.,0.],scalar:1.,tag:0,min:[1.,1.],max:[3.,3.]};
  let second=peer_actor(&first,peer).unwrap();assert_eq!(second.definition,first.definition);assert_eq!(second.staged.as_ref().unwrap().triple,first.staged.as_ref().unwrap().triple);assert_eq!(second.staged.as_ref().unwrap().geometry_admission,"caller_supplied");
  assert_eq!(second.position_words,peer.position.map(f32::to_bits));assert_eq!(second.geometry.center,peer.center.map(f32::to_bits));
  let mut app=app_for(FrameFixture::new(vec![first.clone(),second],0,0,4).unwrap());app.update();let state=app.world().resource::<FrameFixture>();let result=state.result.as_ref().unwrap().as_ref().unwrap();assert_eq!(result.pairs.len(),1);assert_eq!(result.pairs[0].boundary,PairBoundary::PendingE7F170);assert_ne!(result.pairs[0].first.raw,result.pairs[0].second.raw);assert_eq!(result.pairs[0].first.registry,result.pairs[0].second.registry);
  let mut nonfinite=peer;nonfinite.center[0]=f32::NAN;assert!(peer_actor(&first,nonfinite).is_err());
  let mut invalid=peer;invalid.scalar=0.;assert!(peer_actor(&first,invalid).is_err());
  let mut tie=peer;tie.min=[0.,0.];let second=peer_actor(&first,tie).unwrap();assert!(FrameFixture::new(vec![first,second],0,0,4).is_err());
 }
 #[test] fn source_registration_options_require_explicit_consistent_admissions(){
  let prefix=["--admit-stable-files","--install","missing","--seed","1","--progress","0"];
  let args=|tail:&[&str]|prefix.iter().chain(tail).map(|x|x.to_string()).collect::<Vec<_>>();
  assert!(run_installed(&args(&["--source-registration"])).unwrap_err().contains("admit-registration-stability"));
  assert!(run_installed(&args(&["--admit-registration-stability"])).unwrap_err().contains("requires --source-registration"));
  assert!(run_installed(&args(&["--source-registration","--admit-registration-stability","--tag","0"])).unwrap_err().contains("conflicts"));
 }
 #[test] fn installed_options_require_stability_and_reject_overrides(){
  assert!(run_installed(&[]).unwrap_err().contains("admit-stable-files"));
  assert!(run_installed(&["--saved".into(),"1".into()]).unwrap_err().contains("unsupported"));
 }
 #[test]
 #[ignore = "requires exact installed packages and executable; stable paths are explicitly admitted"]
 fn installed_player_provenance_and_projection_enter_one_frame(){
  let install=std::env::var("RUSTSPORE_INSTALL").unwrap_or_else(|_|"D:/Games/SPORE Collection".into());
  let staged=crate::cell_source_player_v1::inspect(std::path::Path::new(&install),123,0).unwrap();
  let expected=staged.projected;let triple=staged.triple;let rows=staged.capability.rows().len();let d=staged.definition;
  let definition=DefinitionIdentity{package:d.package.to_string_lossy().into_owned(),key:d.key,digest:d.sha256,raw:d.bytes};
  assert_eq!(definition.key,[0xcbcd1287,0xdfad9f51,0]);
  let source=ControlledStagedSource{triple,cache_first:triple[0],creation_sha:staged.creation_sha.into(),rows:staged.capability.rows().to_vec(),geometry_admission:"modeled_pre_override"};
  let actor=staged_actor(definition.clone(),source,[0;3],expected,0,[0.,0.],[2.,2.]).unwrap();
  let peer=PeerAdmission{position:[300.,300.,0.],center:[100.4,100.1,0.],scalar:1.,tag:0,min:[1.,1.],max:[3.,3.]};let second=peer_actor(&actor,peer).unwrap();
  let mut app=app_for(FrameFixture::new(vec![actor,second],0,0,4).unwrap());app.update();
  let state=app.world().resource::<FrameFixture>();let result=state.result.as_ref().unwrap().as_ref().unwrap();let actor=state.registry.resolve(result.player).unwrap();assert_eq!(actor.geometry,expected);assert_eq!(actor.definition,definition);assert_eq!(actor.staged.as_ref().unwrap().rows.len(),rows);assert_eq!(actor.staged.as_ref().unwrap().triple,triple);
  assert_eq!(result.fresh_registration_slots.len(),2);assert_eq!(result.fresh_registration_slots[0].1,0);assert_eq!(result.fresh_registration_slots[1].1,1);
  assert_eq!(result.pairs.len(),1);assert_eq!(result.pairs[0].boundary,PairBoundary::PendingE7F170);assert_ne!(result.pairs[0].first.raw,result.pairs[0].second.raw);
  for h in [result.pairs[0].first,result.pairs[0].second] {assert_eq!(state.registry.resolve(h).unwrap().definition,definition);}

 }
 #[test]
 #[ignore = "requires exact installed packages and explicit stable file/manager admissions"]
 fn installed_source_registration_frame_runs_from_modeled_geometry(){
  let install=std::env::var("RUSTSPORE_INSTALL").unwrap_or_else(|_|"D:/Games/SPORE Collection".into());
  let args=vec!["--install".into(),install,"--seed".into(),"123".into(),"--progress".into(),"0".into(),"--source-registration".into(),"--admit-registration-stability".into(),"--admit-stable-files".into()];
  run_installed(&args).unwrap();
 }
 #[test]
 #[ignore = "requires installed packages and explicit file/actor/manager stability admissions"]
 fn installed_multiframe_uses_same_source_inspection_and_synthetic_peer(){
  let install=std::env::var("RUSTSPORE_INSTALL").unwrap_or_else(|_|"D:/Games/SPORE Collection".into());
  let staged=crate::cell_source_player_v1::inspect(std::path::Path::new(&install),123,0).unwrap();let g=staged.projected;let r=f32::from_bits(g.scalar).abs();let x=f32::from_bits(g.center[0])+10.+2.*r;let y=f32::from_bits(g.center[1])+10.+2.*r;
  let args=vec!["--install".into(),install,"--seed".into(),"123".into(),"--progress".into(),"0".into(),"--source-registration".into(),"--admit-registration-stability".into(),"--admit-stable-files".into(),"--multiframe-update".into(),"--admit-actor-manager-stability".into(),"--peer".into(),"0".into(),"0".into(),"0".into(),x.to_string(),y.to_string(),"0".into(),"0.5".into(),"0".into(),(x-1.).to_string(),(y-1.).to_string(),(x+1.).to_string(),(y+1.).to_string()];
  run_installed(&args).unwrap();
 }
 #[test]
 #[ignore = "requires installed packages and explicit file/actor-word lifetime admissions"]
 fn installed_selector_route_uses_passing_synthetic_pair(){
  let install=std::env::var("RUSTSPORE_INSTALL").unwrap_or_else(|_|"D:/Games/SPORE Collection".into());let staged=crate::cell_source_player_v1::inspect(std::path::Path::new(&install),123,0).unwrap();let g=staged.projected;let r=f32::from_bits(g.scalar);let x=f32::from_bits(g.center[0])+0.25*r;let y=f32::from_bits(g.center[1])+0.125*r;let half=0.6*r;
  let args=vec!["--install".into(),install,"--seed".into(),"123".into(),"--progress".into(),"0".into(),"--source-registration".into(),"--admit-registration-stability".into(),"--admit-stable-files".into(),"--selector-route-words".into(),"1".into(),"0".into(),"1".into(),"1".into(),"--admit-selector-actor-lifetime-and-word-stability".into(),"--peer".into(),"0".into(),"0".into(),"0".into(),x.to_string(),y.to_string(),"0".into(),(0.5*r).to_string(),"0".into(),(x-half).to_string(),(y-half).to_string(),(x+half).to_string(),(y+half).to_string()];run_installed(&args).unwrap();
 }
 #[test]
 #[ignore = "requires installed packages and explicit synthetic class/cache stability admissions"]
 fn installed_feature_loop_uses_supplied_classes_only(){
  let install=std::env::var("RUSTSPORE_INSTALL").unwrap_or_else(|_|"D:/Games/SPORE Collection".into());let staged=crate::cell_source_player_v1::inspect(std::path::Path::new(&install),123,0).unwrap();let g=staged.projected;let r=f32::from_bits(g.scalar);let x=f32::from_bits(g.center[0])+0.25*r;let y=f32::from_bits(g.center[1])+0.125*r;let half=0.6*r;
  let args=vec!["--install".into(),install,"--seed".into(),"123".into(),"--progress".into(),"0".into(),"--source-registration".into(),"--admit-registration-stability".into(),"--admit-stable-files".into(),"--selector-route-words".into(),"1".into(),"1".into(),"1".into(),"1".into(),"--admit-selector-actor-lifetime-and-word-stability".into(),"--feature-classes".into(),"0,1,2".into(),"1,0".into(),"--admit-feature-cache-stability".into(),"--peer".into(),"0".into(),"0".into(),"0".into(),x.to_string(),y.to_string(),"0".into(),(0.5*r).to_string(),"0".into(),(x-half).to_string(),(y-half).to_string(),(x+half).to_string(),(y+half).to_string()];run_installed(&args).unwrap();
 }
 #[test] fn invalid_geometry_is_reported_not_promoted_to_contact(){
  let mut bad=fixture_actor(2);bad.geometry.center[0]=f32::NAN.to_bits();
  let mut app=app_for(FrameFixture::new(vec![fixture_actor(1),bad],0,0,4).unwrap());app.update();
  assert!(matches!(app.world().resource::<FrameFixture>().result.as_ref().unwrap().as_ref().unwrap().pairs[0].boundary,PairBoundary::UnsupportedBroadphase(_)));
 }
}
