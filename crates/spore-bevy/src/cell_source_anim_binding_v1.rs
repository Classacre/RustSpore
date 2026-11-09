//! Controlled source-data-bound binding projection, not a live controller.
//! Provider winner, selected actor, cache lifetime and builder input ownership
//! are caller admissions; no event here mutates an instance or refcount.
use sha2::{Digest, Sha256};
use spore_core::{anim_instance_reference_handoff_v1 as refs,
    anim_nonroot_selector::FinalizedSelectorBlock, anim_nonroot_selector_z::DefinitionZ,
    anim_ungrouped_binding_builder::{self as builder, Channel, ResultFields}, creature_data};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetIdentity { pub package: String, pub instance: u64, pub kind: u32, pub group: u32, pub sha256: [u8;32] }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct ActorTriple(pub [u32;3]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Owner(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Cache(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Binding(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Resource(pub u32);

#[derive(Debug)] pub enum Error { Hash, AssetKind, Creation, Anim, ChannelMismatch, NullIdentity, UnadmittedSnapshots, UnadmittedBindingStage, Builder(builder::Error), Reference(refs::Error) }
impl From<builder::Error> for Error { fn from(x:builder::Error)->Self { Self::Builder(x) } }
impl From<refs::Error> for Error { fn from(x:refs::Error)->Self { Self::Reference(x) } }
#[derive(Debug)]
pub struct SourceCandidate { creation:AssetIdentity, anim:AssetIdentity, actor:ActorTriple,
    channels:Vec<Channel> }

fn word(b:&[u8],off:usize)->Result<u32,Error> { Ok(u32::from_le_bytes(b.get(off..off.checked_add(4).ok_or(Error::Anim)?).ok_or(Error::Anim)?.try_into().map_err(|_|Error::Anim)?)) }
fn span(b:&[u8],off:usize,len:usize)->Result<&[u8],Error> { b.get(off..off.checked_add(len).ok_or(Error::Anim)?).ok_or(Error::Anim) }
/// Validates a narrow ANIM channel/descriptor projection only. It does not
/// decode or validate keyframe streams and does not prove native fixups.
fn channels(b:&[u8])->Result<Vec<Channel>,Error> {
    if b.len()<512 || b.len()>16*1024*1024 || &b[..4]!=b"ANIM" || word(b,4)? as usize!=b.len() { return Err(Error::Anim) }
    let n=word(b,324)? as usize; let table=word(b,328)? as usize;
    if !(1..=14).contains(&n) || table<512 { return Err(Error::Anim) }
    let offsets=span(b,table,n*4)?; let mut result=Vec::with_capacity(n);
    for i in 0..n {
        let h=span(b,word(offsets,i*4)? as usize,228)?;
        if &h[..4]!=b"CHAN" {return Err(Error::Anim)}
        let count=word(h,220)? as usize; if !(1..=24).contains(&count) {return Err(Error::Anim)}
        let raw=span(b,word(h,224)? as usize,count*32)?;
        let descriptors=raw.chunks_exact(32).map(|d|std::array::from_fn(|j|u32::from_le_bytes(d[j*4..j*4+4].try_into().unwrap()))).collect();
        let words_88_d0=std::array::from_fn(|j|u32::from_le_bytes(h[136+j*4..140+j*4].try_into().unwrap()));
        result.push(Channel{words_88_d0,descriptors});
    }
    Ok(result)
}
impl SourceCandidate {
    pub fn creation(&self)->&AssetIdentity {&self.creation}
    pub fn anim(&self)->&AssetIdentity {&self.anim}
    pub fn actor(&self)->ActorTriple {self.actor}
    pub fn matches_channels(&self,expected:&[Channel])->bool {self.channels==expected}
    /// Hashes in the asset identities must come from independently admitted
    /// expectations. The bytes are kept in memory for each parse; a package
    /// reader's stability and provenance are caller duties.
    pub fn admit(creation:AssetIdentity, creation_bytes:&[u8], anim:AssetIdentity, anim_bytes:&[u8], actor:ActorTriple)->Result<Self,Error> {
        if creation_bytes.len()>16*1024*1024 || <[u8;32]>::from(Sha256::digest(creation_bytes))!=creation.sha256 ||
           <[u8;32]>::from(Sha256::digest(anim_bytes))!=anim.sha256 {return Err(Error::Hash)}
        if creation.kind!=0x0f43029a || anim.kind!=0xee17c6ad {return Err(Error::AssetKind)}
        let c=creature_data::parse(creation_bytes).map_err(|_|Error::Creation)?;
        if !c.exact || c.consumed!=creation_bytes.len() || c.validate_part_ranges().is_err() {return Err(Error::Creation)}
        Ok(Self {creation,anim,actor,channels:channels(anim_bytes)?})
    }
}

/// Hit means a caller-admitted, nonzero supplied-binding path. It does not
/// model cache retain/release, cache lifetime, or the null-binding fallback.
/// Miss observations likewise do not prove allocation or provider selection.
#[derive(Clone, Copy, Debug)]
pub enum Lookup { Hit(Binding), MissBuild { binding: Binding }, MissFailed }
/// Separate caller admission: omitted work reached 009A3396 without invalidating
/// the supplied objects. It cannot be inferred from cache lookup or builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingStageAdmission { Unestablished, Reached9a3396 }
/// `Projected` contains only selected 009A3140 reference events, not an
/// installed controller, cache lifecycle, or completed binding transaction.
#[derive(Debug)]
pub enum Outcome { BuilderFailedPending, Projected { cache:Cache, creation:AssetIdentity, anim:AssetIdentity, actor:ActorTriple,
    built:Option<ResultFields>, references:refs::Projection } }

/// `expected_channels` and builder inputs are independently supplied fixture
/// state; equality binds only the metadata projection to the ANIM bytes. It
/// does not bind those blocks to the creation or authenticate cache selection.
pub fn handoff(candidate:SourceCandidate, expected_channels:&[Channel], blocks:&[FinalizedSelectorBlock], bounds:DefinitionZ,
    cache:Cache, lookup:Lookup, old:refs::InstanceRefs, resource:Resource, owner:Owner,
    admission:refs::SlotSnapshotAdmission, stage:BindingStageAdmission)->Result<Outcome,Error> {
    if candidate.channels!=expected_channels {return Err(Error::ChannelMismatch)}
    if cache.0==0 || resource.0==0 {return Err(Error::NullIdentity)}
    if admission!=refs::SlotSnapshotAdmission::StableUntilEachSourceRead {return Err(Error::UnadmittedSnapshots)}
    let (binding,built)=match lookup {
        Lookup::Hit(b)=>(b,None),
        Lookup::MissFailed=>return Ok(Outcome::BuilderFailedPending),
        Lookup::MissBuild{binding}=>{
            if binding.0==0 {return Err(Error::NullIdentity)}
            (binding,Some(builder::build(blocks,expected_channels,bounds)?))
        },
    };
    if binding.0==0{return Err(Error::NullIdentity)}
    if stage!=BindingStageAdmission::Reached9a3396 {return Err(Error::UnadmittedBindingStage)}
    let references=refs::project_supplied_binding(old,refs::Identity(resource.0),refs::Identity(owner.0),refs::Identity(binding.0),admission)?;
    Ok(Outcome::Projected {cache,creation:candidate.creation,anim:candidate.anim,actor:candidate.actor,built,references})
}

#[cfg(test)] mod tests {
    use super::*;
    fn key(d:[u8;32])->AssetIdentity {AssetIdentity{package:"fixture".into(),instance:1,kind:2,group:3,sha256:d}}
    #[test] fn wrong_digest_fails_before_parse() {
        assert!(matches!(SourceCandidate::admit(key([0;32]),b"x",key([0;32]),b"y",ActorTriple([1,2,3])),Err(Error::Hash)));
    }
    #[test] fn anim_metadata_rejects_out_of_range_and_inconsistent_extent() {
        let mut b=vec![0;512];b[..4].copy_from_slice(b"ANIM");b[4..8].copy_from_slice(&512u32.to_le_bytes());
        b[324..328].copy_from_slice(&1u32.to_le_bytes());b[328..332].copy_from_slice(&512u32.to_le_bytes());
        assert!(matches!(channels(&b),Err(Error::Anim)));
    }
    // These intentionally bypass admission to test only the orchestration;
    // they are not an asset-backed or native binding fixture.
    fn synthetic_candidate()->SourceCandidate {
        SourceCandidate{creation:key([1;32]),anim:key([2;32]),actor:ActorTriple([7,8,9]),channels:vec![]}
    }
    fn synthetic_bounds()->DefinitionZ {
        let mut p=[0u8;44];p[4..8].copy_from_slice(&0x7fu32.to_le_bytes());
        DefinitionZ::from_native_publisher_projection(&p,0).unwrap()
    }
    #[test] fn admitted_identity_is_not_changed_by_mutating_an_accessor_clone() {
        let c=synthetic_candidate();let mut copy=c.creation().clone();copy.instance=99;copy.sha256=[99;32];
        assert_eq!(c.creation().instance,1);assert_eq!(c.creation().sha256,[1;32]);
        assert_eq!(c.anim().sha256,[2;32]);assert_eq!(c.actor(),ActorTriple([7,8,9]));
    }
    #[test] fn hit_preserves_distinct_identities_and_ordered_reference_events() {
        let old=refs::InstanceRefs{resource:refs::Identity(11),owner:refs::Identity(12),binding:refs::Identity(13)};
        let out=handoff(synthetic_candidate(),&[],&[],synthetic_bounds(),Cache(41),Lookup::Hit(Binding(44)),old,Resource(42),Owner(43),refs::SlotSnapshotAdmission::StableUntilEachSourceRead,BindingStageAdmission::Reached9a3396).unwrap();
        let Outcome::Projected{cache,creation,anim,actor,built,references}=out else {panic!()};
        assert_eq!(cache,Cache(41));assert_eq!(actor,ActorTriple([7,8,9]));assert_ne!(creation.sha256,anim.sha256);assert!(built.is_none());
        assert_eq!(references.events,vec![refs::Event::RetainResource(refs::Identity(42)),refs::Event::StoreResource(refs::Identity(42)),refs::Event::ReleaseResource(refs::Identity(11)),refs::Event::RetainOwner(refs::Identity(43)),refs::Event::StoreOwner(refs::Identity(43)),refs::Event::ReleaseOwner(refs::Identity(12)),refs::Event::RetainBinding(refs::Identity(44)),refs::Event::StoreBinding(refs::Identity(44)),refs::Event::ReleaseBinding(refs::Identity(13))]);
    }
    #[test] fn failure_stays_pending_and_unadmitted_snapshots_fail() {
        let old=refs::InstanceRefs::default();
        assert!(matches!(handoff(synthetic_candidate(),&[],&[],synthetic_bounds(),Cache(1),Lookup::MissFailed,old,Resource(2),Owner(3),refs::SlotSnapshotAdmission::StableUntilEachSourceRead,BindingStageAdmission::Unestablished),Ok(Outcome::BuilderFailedPending)));
        assert!(matches!(handoff(synthetic_candidate(),&[],&[],synthetic_bounds(),Cache(1),Lookup::Hit(Binding(4)),old,Resource(2),Owner(3),refs::SlotSnapshotAdmission::Unestablished,BindingStageAdmission::Reached9a3396),Err(Error::UnadmittedSnapshots)));
        assert!(matches!(handoff(synthetic_candidate(),&[],&[],synthetic_bounds(),Cache(1),Lookup::Hit(Binding(4)),old,Resource(2),Owner(3),refs::SlotSnapshotAdmission::StableUntilEachSourceRead,BindingStageAdmission::Unestablished),Err(Error::UnadmittedBindingStage)));
    }
    #[test] fn zero_binding_never_becomes_a_supplied_binding() {
        for lookup in [Lookup::Hit(Binding(0)),Lookup::MissBuild{binding:Binding(0)}] {
            assert!(matches!(handoff(synthetic_candidate(),&[],&[],synthetic_bounds(),Cache(1),lookup,refs::InstanceRefs::default(),Resource(2),Owner(3),refs::SlotSnapshotAdmission::StableUntilEachSourceRead,BindingStageAdmission::Reached9a3396),Err(Error::NullIdentity)));
        }
    }
    #[test]
    #[ignore = "requires explicitly selected local installation and retained builder fixture"]
    fn explicitly_selected_installed_payloads_match_independent_fixture_channels() {
        use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
        use std::{io::{Cursor,Read,Seek,SeekFrom},path::PathBuf};
        fn digest_path(p:&std::path::Path)->[u8;32] {
            let mut f=std::fs::File::open(p).unwrap();let mut h=Sha256::new();let mut b=[0;65536];
            loop {let n=f.read(&mut b).unwrap();if n==0{break}h.update(&b[..n]);} h.finalize().into()
        }
        fn expected(hex:&str)->[u8;32] {std::array::from_fn(|i|u8::from_str_radix(&hex[i*2..i*2+2],16).unwrap())}
        fn read(p:&std::path::Path,k:NativeResourceKey)->Vec<u8> {
            let idx=NativePackageIndex::open(p).unwrap();let e=idx.index().find_unique(k).unwrap().unwrap_or_else(||panic!("missing selected key {k} in {}",p.display()));
            assert!(e.stored_size<=16*1024*1024 && e.decompressed_size<=16*1024*1024);idx.read(e).unwrap()
        }
        fn take(r:&mut impl Read,n:usize)->Vec<u8>{let mut b=vec![0;n];r.read_exact(&mut b).unwrap();b}
        let root=PathBuf::from(std::env::var("RUSTSPORE_INSTALL").expect("explicit RUSTSPORE_INSTALL required"));
        let fixture=PathBuf::from(std::env::var("RUSTSPORE_ANIM_BUILDER_FIXTURE").expect("explicit retained fixture path required"));
        let content=root.join("Data/Spore_Content.package");let patch=root.join("Data/PatchData.package");
        let content_hash=expected("0b849bd79d9c13dec5737159655bf7e2b7763cc2b79949c81370fa0490256e8c");
        let patch_hash=expected("cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff");
        assert_eq!(digest_path(&content),content_hash);assert_eq!(digest_path(&patch),patch_hash);
        let mut fixture_bytes=Vec::new();std::fs::File::open(&fixture).unwrap().take(64*1024*1024+1).read_to_end(&mut fixture_bytes).unwrap();
        assert!(fixture_bytes.len()<=64*1024*1024);assert_eq!(<[u8;32]>::from(Sha256::digest(&fixture_bytes)),expected("d9d009821bd97d0217af832b3c90d6d5012a53d4ac80451f31418a9b61696319"));
        let creation_key=NativeResourceKey::from_scoped_instance(0x0f43029a,0x40616200780a9eaf);
        let anim_idx=NativePackageIndex::open(&patch).unwrap();let anim_entry=anim_idx.entries().get(4845).unwrap();let anim_key=anim_entry.key;
        assert_eq!(anim_key.instance_id,0x87cb0177);assert_eq!(anim_key.type_id,0xee17c6ad);assert_eq!(anim_key.group_id,0);
        assert!(anim_entry.stored_size<=16*1024*1024 && anim_entry.decompressed_size<=16*1024*1024);
        println!("selected ANIM ordinal=4845 native key={anim_key}");
        let creation=read(&content,creation_key);let anim=anim_idx.read(anim_entry).unwrap();
        let creation_id=AssetIdentity{package:content.to_string_lossy().into(),instance:0x40616200780a9eaf,kind:0x0f43029a,group:creation_key.group_id,sha256:expected("2181f3b217d4b610ab10d4475412eba7789eb37b5a3bff1f164773026be6d71e")};
        let anim_id=AssetIdentity{package:patch.to_string_lossy().into(),instance:0x87cb0177,kind:0xee17c6ad,group:anim_key.group_id,sha256:expected("837a340a2b4b08e5c69b075abfa01a9a9d1f18195c446cb2cfbe3a06aa90a802")};
        let candidate=SourceCandidate::admit(creation_id, &creation,anim_id,&anim,ActorTriple([0x780a9eaf,0x3d97a8e4,0x40616200])).unwrap();
        let mut cur=Cursor::new(fixture_bytes);let mut matched=None;
        for index in 0..=30 {
            let h=take(&mut cur,32);assert_eq!(word(&h,0).unwrap(),0x31444c42);assert_eq!(word(&h,12).unwrap(),0x27f);
            let n=word(&h,4).unwrap() as usize;let c=word(&h,8).unwrap() as usize;
            assert!((1..=255).contains(&n) && (1..=14).contains(&c));cur.seek(SeekFrom::Current((n*324) as i64)).unwrap();
            let mut out=Vec::new();let mut total=0;
            for _ in 0..c {let h=take(&mut cur,80);let count=word(&h,76).unwrap() as usize;assert!((1..=24).contains(&count));total+=count;
                let mut desc=Vec::new();for _ in 0..count {let d=take(&mut cur,32);desc.push(std::array::from_fn(|i|word(&d,i*4).unwrap()));}
                out.push(Channel{words_88_d0:std::array::from_fn(|i|word(&h,i*4).unwrap()),descriptors:desc});}
            assert_eq!(total,word(&h,24).unwrap() as usize);if index==30{matched=Some(out)}
        }
        let expected_channels=matched.unwrap();assert!(candidate.matches_channels(&expected_channels));
        let mut wrong=expected_channels.clone();wrong[0].words_88_d0[0]^=1;assert!(!candidate.matches_channels(&wrong));
        let mut changed=anim.clone();changed[0]^=1;
        assert_ne!(<[u8;32]>::from(Sha256::digest(&changed)),candidate.anim().sha256);
        assert!(matches!(SourceCandidate::admit(candidate.creation().clone(),&creation,candidate.anim().clone(),&changed,candidate.actor()),Err(Error::Hash)));
        assert!(candidate.matches_channels(&expected_channels));
        assert_eq!(candidate.creation().sha256,expected("2181f3b217d4b610ab10d4475412eba7789eb37b5a3bff1f164773026be6d71e"));
        assert_eq!(candidate.anim().sha256,expected("837a340a2b4b08e5c69b075abfa01a9a9d1f18195c446cb2cfbe3a06aa90a802"));
        assert_eq!(digest_path(&content),content_hash);assert_eq!(digest_path(&patch),patch_hash);
    }
}
