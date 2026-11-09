//! Controlled source-player startup inspection. No live actor or provider binding.
#[path = "cell_source_player_pool_fixture_v1.rs"]
mod pool_fixture;
#[path = "cell_source_feature_geometry_v1.rs"]
mod feature_geometry;
#[path = "cell_source_contact_call_v1.rs"]
mod contact_call;
#[path = "cell_source_contact_record_v1.rs"]
mod contact_record;
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::{cell_definition, cell_globals, cell_kind4_cache_selection_v1 as selection,
    cell_kind4_list_disk_v1, cell_start_equal_size_v1 as size,
    cell_start_null_orientation_v1 as orientation, cell_start_quaternion_matrix_v1 as matrix,
    cell_start_table_v1 as start_table, native_sse::Policy};
use std::{fs::File, io::Read, path::{Path,PathBuf}};

const PATCH_SHA: &str = "cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff";
const CONTENT_SHA: &str = "0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c";
const EXE_SHA: &str = "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a";
const GLOBALS_SHA: &str = "e8c4b80348fd8f3a1821533c87110315ba584f66f081d46305ff61ab6cbf8c72";
const CELL_SHA: &str = "f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542";
const STRUCTURE_SHA: &str = "a01b950a54889f86429f4e847b7aa7e1ff8d5e354027b017eb80ee8e07a6da87";
const LIST_SHA: &str = "29b0f195b26bf2480586518e385933284819ff360a991ea706a939717a5d5f8a";

/// Bounded E751F1..E75277 copy sequence with stable, disjoint objects and no
/// concurrent writers. The pending call must be executed and its effects
/// admitted before any continuation that depends on live state.
#[derive(Debug, PartialEq, Eq)]
struct PlayerTripleDecision { actor:[u32;3], saved:[u32;3], pending_e829d0:Option<[u32;3]> }
fn decide_player_triple(actor:[u32;3], saved:[u32;3], mode:u32)->PlayerTripleDecision {
    if mode == 1 {
        PlayerTripleDecision { actor:saved, saved, pending_e829d0:None }
    } else {
        let (actor,saved)=if saved[0] != 0 {(saved,saved)} else {(actor,actor)};
        PlayerTripleDecision { actor, saved, pending_e829d0:Some(saved) }
    }
}

/// First stores of 4DF9E0 and its conditional virtual call. The call receives
/// a pointer to the input, so it can change memory subsequently read by source.
#[derive(Debug, PartialEq, Eq)]
struct TripleNotificationPrefix { stored_a4:[u32;3], pending_virtual_58:Option<[u32;3]> }
fn notification_prefix(triple:[u32;3])->TripleNotificationPrefix {
    TripleNotificationPrefix { stored_a4:triple, pending_virtual_58:(triple[0]!=0).then_some(triple) }
}
/// Message words are fresh input read after the conditional virtual call and
/// message constructor. This invokes neither nor asserts input stability.
fn notification_message_words(fresh_triple:[u32;3])->[u32;3] {
    [fresh_triple[1],fresh_triple[0],fresh_triple[2]]
}
fn hash_file(path: &Path) -> Result<String, String> {
    let mut file=File::open(path).map_err(|e|format!("{}: {e}",path.display()))?;
    let mut hasher=Sha256::new();let mut buffer=[0u8;65536];
    loop {let n=file.read(&mut buffer).map_err(|e|e.to_string())?;if n==0 {break} hasher.update(&buffer[..n]);}
    Ok(format!("{:x}",hasher.finalize()))
}
fn read_resource(index:&NativePackageIndex, key:NativeResourceKey, expected:&str)->Result<Vec<u8>,String>{
    let entry=index.index().find_unique(key).map_err(|e|e.to_string())?.ok_or_else(||format!("missing {key}"))?;
    if entry.decompressed_size>16*1024*1024 || entry.stored_size>16*1024*1024{return Err(format!("{key}: oversized resource"));}
    let data=index.read(entry).map_err(|e|e.to_string())?;
    if data.len()!=entry.decompressed_size as usize || format!("{:x}",Sha256::digest(&data))!=expected{return Err(format!("{key}: payload hash/extent mismatch"));}
    Ok(data)
}
fn key(instance:u32,kind:u32,scope:u32)->NativeResourceKey {NativeResourceKey{instance_id:instance,type_id:kind,group_id:scope}}

/// Exact CELL bytes from the same hash-bound staged inspection. Package
/// stability remains the caller's explicit admission, not an atomic snapshot.
#[derive(Debug)]
pub struct StagedDefinition {pub package:PathBuf,pub key:[u32;3],pub sha256:[u8;32],pub bytes:Vec<u8>}
/// Values are a staged controlled computation; no actor is published because
/// the production creation/property/cache provider is not yet implemented.
#[derive(Debug)]
pub struct StagedPlayer {
    pub seed:u32,pub progress:i32,pub table_output:start_table::Output,pub after_size:u32,pub after_orientation:u32,pub after_selection:u32,
    pub scale_bits:u32,pub quaternion:[u32;4],pub matrix:[u32;9],pub triple:[u32;3],
    pub creation_sha:&'static str,
    pub definition:StagedDefinition,
    pub parts:Vec<(usize, crate::cell_source_part_provider_v1::Classification)>,
    pub capability:crate::cell_source_capability_v1::Capability,
    pub projected:spore_core::cell_actor_geometry_projection_v1::Output,
}
fn creation_digest(id:u32)->Option<&'static str>{match id {
    0xa1c46df2=>Some("6852040fa9a9376ed9025f4823e05024a22685a147e00e64d1b0f536b33e8cd8"),
    0xe219e5d0=>Some("8d1a88a9eddbcabbe4fe5eb36a6cf1d1c24a88ed38affb9e1ec02e9556aae48e"),
    0xe219e5d2=>Some("cafd2cd270d676834143d85f252d634b51c73fcf12250f765c12680a65fb2c1e"),
    0xe219e5d3=>Some("dbbb55830f7f75eb5c45afd470a8bd0716cb08134e26f62a82674e28cd697137"),
    _=>None,
}}
/// The application owns a fresh empty kind-4 cache and controls the resolver
/// snapshots; callbacks and other threads are excluded by construction. This
/// controlled continuity is an admission, not retail evidence.
pub fn inspect(install:&Path,seed:u32,progress:i32)->Result<StagedPlayer,String>{
    if seed==0 {return Err("zero seed needs native RDTSC; supply a nonzero controlled seed".into())}
    let patch=install.join("Data/PatchData.package");let content=install.join("Data/Spore_Content.package");let exe=install.join("SporeBin/SporeApp.exe");
    for (p,expected) in [(&patch,PATCH_SHA),(&content,CONTENT_SHA),(&exe,EXE_SHA)] {
        if hash_file(p)?!=expected{return Err(format!("{}: source hash mismatch",p.display()));}
    }
    if !(0..=1000).contains(&progress) {return Err("progress must be in 0..=1000 for the bounded table scan".into())}
    // These file offsets are bound to EXE_SHA; hash the actual in-memory bytes
    // before reading them, independent of the before/after file hash checks.
    const EXE_LEN:u64=20_447_744;
    let exe_file=File::open(&exe).map_err(|e|e.to_string())?;
    if exe_file.metadata().map_err(|e|e.to_string())?.len()!=EXE_LEN {return Err("executable length mismatch".into())}
    let mut exe_bytes=Vec::with_capacity(EXE_LEN as usize);
    exe_file.take(EXE_LEN+1).read_to_end(&mut exe_bytes).map_err(|e|e.to_string())?;
    if exe_bytes.len()!=EXE_LEN as usize {return Err("executable snapshot length mismatch".into())}
    if format!("{:x}",Sha256::digest(&exe_bytes))!=EXE_SHA {return Err("executable snapshot hash mismatch".into())}
    let table=exe_bytes.get(0x1083ac4..0x1083ac4+22*28).ok_or("executable table truncated")?;
    let mut rows=Vec::with_capacity(22);
    for row in table.chunks_exact(28) {
        rows.push(start_table::Row{threshold:i32::from_le_bytes(row[4..8].try_into().unwrap()),value_bits:u32::from_le_bytes(row[16..20].try_into().unwrap())});
    }
    let numerator=u32::from_le_bytes(exe_bytes.get(0x10855d0..0x10855d4).ok_or("executable scalar truncated")?.try_into().unwrap());
    let table_output=start_table::evaluate(progress,&rows,rows[21].value_bits,numerator,Policy::Mxcsr1f80).map_err(|e|format!("start table: {e:?}"))?;
    let index=NativePackageIndex::open(&patch).map_err(|e|e.to_string())?;
    let globals=read_resource(&index,key(0xa426730b,0x2a3ce5b7,0),GLOBALS_SHA)?;
    let cell=read_resource(&index,key(0xcbcd1287,0xdfad9f51,0),CELL_SHA)?;
    let structure=read_resource(&index,key(0xcbcd1287,0x4b9ef6dc,0),STRUCTURE_SHA)?;
    let list=read_resource(&index,key(0x9a391385,0xf9c3d770,0),LIST_SHA)?;
    let curve=read_resource(&index,key(0xc951eb63,0x00b1b104,0),"c00576f276d3e3daa9f898985775ba13e173d66b492bc8e8fcf0b7b3b25a6462")?;
    if structure.len()!=108{return Err("unexpected structure extent".into())}
    let globals_view=cell_globals::parse(&globals).map_err(|e|e.to_string())?;
    let def=cell_definition::parse(&cell).map_err(|e|e.to_string())?;
    if globals_view.game_mode()!=0 || def.raw()[0xac]==1 {return Err("unsupported constructor branch".into())}
    let sample=size::candidate_start_size(size::AdmittedCellKey{instance:0xcbcd1287,type_id:0xdfad9f51,scope:0},def,globals_view,f32::from_bits(table_output.global_514c_bits),f32::from_bits(table_output.actor_argument_bits),seed,0x027f).map_err(|e|format!("size: {e:?}"))?;
    let orient=orientation::null_orientation(sample.rng_state_after,0x40c90fdb,[0,0,0x3f800000],0x027f,Policy::Mxcsr1f80).map_err(|e|format!("orientation: {e:?}"))?;
    let mat=matrix::matrix_words(orient.quaternion_bits,Policy::Mxcsr1f80).map_err(|e|format!("matrix: {e:?}"))?;
    let rows=cell_kind4_list_disk_v1::parse(&list).map_err(|e|format!("list: {e:?}"))?;
    let mut cache=selection::Cache::default();
    let selected=selection::select(&mut cache,1,Some(&rows),orient.rng_state_after,0x027f,0x40616200,selection::SnapshotAdmission);
    let (triple,after_selection)=match selected {selection::ResultV1::Selected{triple,rng_state,..}=>(triple,rng_state),other=>return Err(format!("selection: {other:?}"))};
    let creation_sha=creation_digest(triple[0]).ok_or("unsupported selected creation")?;
    let content_index=NativePackageIndex::open(&content).map_err(|e|e.to_string())?;
    let creation=read_resource(&content_index,key(triple[0],0x0f43029a,triple[2]),creation_sha)?;
    let parsed=spore_core::creature_data::parse(&creation).map_err(|e|e.to_string())?;
    if !parsed.exact || parsed.consumed!=creation.len(){return Err("creation framing unresolved".into())}
    parsed.validate_part_ranges().map_err(|e|e.to_string())?;
    let mut parts=Vec::new();
    for row in 0..parsed.parts.len().min(64) {
        let view=parsed.part_view(row).ok_or("missing creation part")?;
        let part_key=NativeResourceKey::from_scoped_instance(0x00b1b104,view.part.part_instance);
        let class=crate::cell_source_part_provider_v1::classify(&index,part_key)?;
        parts.push((row,class));
    }
    let classes:Vec<u32>=parts.iter().map(|(_,class)|class.class).collect();
    let capability=crate::cell_source_capability_v1::build(&creation,triple[0],&classes,globals_view,&curve).map_err(|e|e.0)?;
    use spore_core::cell_actor_geometry_projection_v1::{self as projection, ActorInput, AdmittedCacheSnapshot};
    let actor=ActorInput{handle_fc:triple[0],position:[100f32.to_bits(),100f32.to_bits(),0],
        scalar:sample.actor_scale_bits,flags_48:2,matrix:mat};
    let cache=AdmittedCacheSnapshot{key_fc:triple[0],center:capability.geometry.center.map(f32::to_bits),scalar:capability.geometry.radius.to_bits()};
    let projected=projection::project(actor,Some(cache),Policy::Mxcsr1f80).map_err(|e|format!("projection: {e:?}"))?;
    for (p,expected) in [(&patch,PATCH_SHA),(&content,CONTENT_SHA),(&exe,EXE_SHA)] {
        if hash_file(p)?!=expected{return Err(format!("{}: source changed",p.display()));}
    }
    Ok(StagedPlayer{seed,progress,table_output,after_size:sample.rng_state_after,after_orientation:orient.rng_state_after,
        after_selection,scale_bits:sample.actor_scale_bits,quaternion:orient.quaternion_bits,matrix:mat,triple,creation_sha,
        definition:StagedDefinition{package:patch,key:[0xcbcd1287,0xdfad9f51,0],sha256:Sha256::digest(&cell).into(),bytes:cell},parts,capability,projected})
}

fn parse_options(args:&[String])->Result<(&Path,u32,i32),String>{
    let mut install=None;let mut seed=None;let mut progress=None;let mut i=0;
    while i<args.len(){let v=args.get(i+1).ok_or_else(||format!("missing value for {}",args[i]))?;
        match args[i].as_str(){
            "--install" if install.is_none()=>install=Some(Path::new(v)),
            "--seed" if seed.is_none()=>seed=Some(v.parse::<u32>().map_err(|_|"seed must be a nonzero decimal u32")?),
            "--progress" if progress.is_none()=>progress=Some(v.parse::<i32>().map_err(|_|"progress must be an integer")?),
            _=>return Err(format!("unknown or repeated option {}",args[i]))
        } i+=2;}
    Ok((install.ok_or("--install is required")?,seed.ok_or("--seed is required")?,progress.unwrap_or(0)))
}

pub fn run(args:&[String])->Result<(),String>{
    let (path,seed,progress)=parse_options(args)?;
    let result=inspect(path,seed,progress)?;
    println!("SOURCE_PLAYER_STAGED controlled_seed={} progress={} fp=027F/mxcsr1f80",result.seed,result.progress);
    println!("assumptions: fresh empty owned selection cache; disk rows admitted as runtime snapshot; no intervening RNG mutation; scope=40616200");
    println!("prior actor allocation is assumed; source reference callbacks and full constructor are not executed");
    println!("inputs: selected={:?} +514C={:08X} arg6={:08X} division={:08X} angle=40C90FDB axis=[00000000,00000000,3F800000]",result.table_output.selected,result.table_output.global_514c_bits,result.table_output.actor_argument_bits,result.table_output.division_bits);
    println!("table assumptions: caller-admitted progress; stable nonaliasing owners; executable snapshot authenticated; no source FP sticky effects");
    println!("provenance: PatchData.package sha256={PATCH_SHA}; Spore_Content.package sha256={CONTENT_SHA}; SporeApp.exe sha256={EXE_SHA}");
    println!("package paths must remain trusted and stable; before/after hashes cannot establish atomic stability");
    println!("rng_after_size={:08X} rng_after_orientation={:08X} rng_after_selection={:08X}",result.after_size,result.after_orientation,result.after_selection);
    println!("scale_bits={:08X} quaternion={:08X?} matrix={:08X?}",result.scale_bits,result.quaternion,result.matrix);
    println!("modeled_selected_triple={:08X}:{:08X}:{:08X}",result.triple[0],result.triple[1],result.triple[2]);
    println!("explicit_creation_candidate={:08X}:0F43029A:{:08X} payload_sha256={}",result.triple[0],result.triple[2],result.creation_sha);
    for (row,classification) in &result.parts {
        println!("part_row={row} class={} parent_nodes={}",classification.class,classification.chain.len());
        for node in &classification.chain {
            println!("property_package=Data/PatchData.package key={} payload_sha256={} parent={:?}",node.key,node.sha256,node.parent);
        }
    }
    println!("controlled_cache: rows={} empty_center=[0,0,0] electric_curve=5A38C4BA center_bits={:08X?} radius_bits={:08X} flags_1250={} flags_1251={} speed_bits={:08X} mask={:08X}",result.capability.row_count,result.capability.geometry.center.map(f32::to_bits),result.capability.geometry.radius.to_bits(),result.capability.scalars.flag_1250,result.capability.scalars.flag_1251,result.capability.scalars.value_1258.to_bits(),result.capability.scalars.propulsion_mask_1254);
    // Explicitly controlled empty saved state; this does not establish the
    // actual +5190 object or execute the non-1 branch's E829D0 call.
    let decision=decide_player_triple(result.triple,[0;3],0);
    println!("controlled_saved_triple decision={decision:?}; E829D0 remains pending and no downstream native state is claimed");
    if let Some(triple)=decision.pending_e829d0 {
        let prefix=notification_prefix(triple);
        println!("controlled_notification_prefix={prefix:?}; virtual +58 and message dispatch remain unexecuted");
        // Only an explicitly unchanged input admission permits this preview.
        println!("controlled_message_preview id=03795725 words={:08X?}; input unchanged across virtual call and constructor assumed, no dispatch",notification_message_words(triple));
    }
    let features=result.capability.feature_snapshot(decision.actor)?;
    let eligible:Vec<_>=features.candidates().map(|(index,row)|(index,row.class)).collect();
    println!("controlled_feature_rows={eligible:?}; selected triple independently admitted as post-override actor triple; class-1 rows skipped without renumbering; no contact geometry or live actor asserted");
    let transform=feature_geometry::Transform {position:[100f32.to_bits(),100f32.to_bits(),0],scale:result.scale_bits,flags:2,matrix:result.matrix};
    // Explicit synthetic second actor input: copy one modeled row and translate
    // its supplied actor position. No second actor/cache is inferred.
    let second=features.candidates().next().map(|_|feature_geometry::Transform {position:[102f32.to_bits(),100f32.to_bits(),0],..transform});
    let pool_report=pool_fixture::inspect(result.triple,decision.actor,result.projected,features,transform,second)?;
    println!("pair assumption: independently supplied synthetic second feature copies first eligible row at position [102,100,0]; statically inferred prefix only, no contact acceptance");
    println!("CONTROLLED_PLAYER_POOL_FIXTURE {pool_report}; owned synthetic identity; no gameplay actor or updated leaf");
    println!("projection assumptions: controlled position=[100,100,0], flags_48=02, modeled matrix; actor +FC and cache key assigned from selected creation ID; local cache association is not source-proven");
    println!("controlled_projected_geometry center_bits={:08X?} scalar_bits={:08X}",result.projected.center,result.projected.scalar);
    println!("curve_package=Data/PatchData.package key=C951EB63:00B1B104:00000000 payload_sha256=c00576f276d3e3daa9f898985775ba13e173d66b492bc8e8fcf0b7b3b25a6462");
    Err("UNRESOLVED native actor handle, provider/cache binding, registration and notification services; player not published; no gameplay or retail runtime claim".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_player_triple_branches_preserve_words_and_pending_call() {
        let actor=[1,2,3];
        let empty=[0,4,5];
        assert_eq!(decide_player_triple(actor,empty,1),PlayerTripleDecision{actor:empty,saved:empty,pending_e829d0:None});
        assert_eq!(decide_player_triple(actor,empty,2),PlayerTripleDecision{actor,saved:actor,pending_e829d0:Some(actor)});
        let saved=[7,8,9];
        assert_eq!(decide_player_triple(actor,saved,0),PlayerTripleDecision{actor:saved,saved,pending_e829d0:Some(saved)});
    }
    #[test]
    fn notification_prefix_and_fresh_message_reads_are_separate() {
        assert_eq!(notification_prefix([0,2,3]),TripleNotificationPrefix{stored_a4:[0,2,3],pending_virtual_58:None});
        assert_eq!(notification_prefix([1,2,3]),TripleNotificationPrefix{stored_a4:[1,2,3],pending_virtual_58:Some([1,2,3])});
        // The callback can mutate its input: message reads must use a fresh snapshot.
        assert_eq!(notification_message_words([4,5,6]),[5,4,6]);
    }
    #[test]
    fn controlled_options_reject_ambiguous_repeated_inputs() {
        let args = |values: &[&str]| values.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
        let valid=args(&["--install","root","--seed","1"]);
        assert_eq!(parse_options(&valid).unwrap(),(Path::new("root"),1,0));
        for option in ["--install", "--seed", "--progress"] {
            let repeated=args(&["--install","root","--seed","1","--progress","0",option,"1"]);
            assert!(parse_options(&repeated).unwrap_err().contains("repeated"));
        }
        assert!(parse_options(&args(&["--install","root","--seed"])).is_err());
    }
    #[test]
    fn zero_seed_is_rejected_before_any_resource_access() {
        let error=inspect(Path::new("missing-install"),0,0).unwrap_err();
        assert!(error.contains("RDTSC"));
    }
    #[test]
    #[ignore = "requires exact installed packages and executable; controlled inspection only"]
    fn installed_staged_rows_enter_owned_player_pool_fixture() {
        let install=std::env::var_os("RUSTSPORE_INSTALL").expect("set RUSTSPORE_INSTALL");
        let staged=inspect(Path::new(&install),123,0).unwrap();
        let decision=decide_player_triple(staged.triple,[0;3],0);
        let snapshot=staged.capability.feature_snapshot(decision.actor).unwrap();
        let transform=feature_geometry::Transform {position:[100f32.to_bits(),100f32.to_bits(),0],scale:staged.scale_bits,flags:2,matrix:staged.matrix};
        let second=snapshot.candidates().next().map(|_|feature_geometry::Transform{position:[102f32.to_bits(),100f32.to_bits(),0],..transform});
        let report=pool_fixture::inspect(staged.triple,decision.actor,staged.projected,snapshot,transform,second).unwrap();
        assert!(report.contains("eligible_rows="));
    }
    #[test]
    #[ignore = "requires exact installed packages and executable; controlled inspection only"]
    fn installed_progress_changes_table_and_preserves_default_values() {
        let install=std::env::var_os("RUSTSPORE_INSTALL").expect("set RUSTSPORE_INSTALL");
        let zero=inspect(Path::new(&install),123,0).unwrap();
        assert_eq!(zero.table_output.selected,start_table::Selected::Row(1));
        assert_eq!(zero.table_output.global_514c_bits,0x3f000000);
        assert_eq!(zero.table_output.actor_argument_bits,0x3f800000);
        let fifty=inspect(Path::new(&install),123,50).unwrap();
        assert_eq!(fifty.table_output.selected,start_table::Selected::Row(2));
        assert_eq!(fifty.table_output.global_514c_bits,0x3f000000);
        let hundred=inspect(Path::new(&install),123,100).unwrap();
        assert_eq!(hundred.table_output.selected,start_table::Selected::Row(3));
        assert_eq!(hundred.table_output.global_514c_bits,0x3fc00000);
        assert_eq!(hundred.table_output.actor_argument_bits,0x40400000);
    }

}
