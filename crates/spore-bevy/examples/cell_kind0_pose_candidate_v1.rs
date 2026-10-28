//! Check a model/clip source pair before producing an explicit non-retail pose.
//! No runtime clip selection or original interpolation is inferred.
//!
//! Usage: cell_kind0_pose_candidate_v1 <model.package> <clip.package>
//!     <clip-instance-hex> <clip-binding-section> <clip-section> <seconds> <new-target.tsv>
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{rw4, rw4_animation::{self, AffineTransform, SampleValue}};
use std::{collections::BTreeSet, error::Error, io::{Read,Write}, path::Path};

type Failure = Box<dyn Error>;
fn package(path: &Path) -> Result<NativePackageIndex, Failure> {
    let mut f=std::fs::File::open(path)?;
    let mut bytes=[0;dbpf::HEADER_SIZE]; f.read_exact(&mut bytes)?;
    let header=dbpf::DbpfHeader::parse(&bytes)?;
    if header.index_size>64*1024*1024 || header.index_entry_count>2_000_000 {
        return Err("package index exceeds preflight bounds".into());
    }
    // Reader reopens the path: caller must supply stable trusted packages.
    Ok(NativePackageIndex::open(path)?)
}
fn resource(p:&NativePackageIndex,key:NativeResourceKey)->Result<Vec<u8>,Failure>{
    let e=p.index().find_unique(key)?.ok_or("exact resource missing")?;
    if e.stored_size.max(e.decompressed_size)>32*1024*1024 {return Err("resource exceeds bounds".into());}
    Ok(p.read(e)?)
}
fn main()->Result<(),Failure>{
    let args:Vec<_>=std::env::args().skip(1).collect();
    if args.len()!=7 {return Err("usage: cell_kind0_pose_candidate_v1 <model.package> <clip.package> <clip-scoped-hex> <clip-binding-section> <clip-section> <seconds> <new-target.tsv>".into());}
    let scoped=u64::from_str_radix(args[2].trim_start_matches("0x"),16)?;
    let binding_index:u32=args[3].parse()?; let clip_index:u32=args[4].parse()?; let time:f32=args[5].parse()?;
    if !time.is_finite(){return Err("sample time must be finite".into());}
    let out=Path::new(&args[6]); if out.exists(){return Err("output already exists".into());}
    let target=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target").canonicalize()?;
    let parent=out.parent().ok_or("output needs a parent")?.canonicalize()?;
    if !parent.starts_with(target){return Err("output must be under target".into());}
    let mp=package(Path::new(&args[0]))?; let cp=package(Path::new(&args[1]))?;
    let mk=NativeResourceKey{instance_id:0xdae8ce17,type_id:0x2f4e681b,group_id:0};
    let ck=NativeResourceKey::from_scoped_instance(0x2f4e681b,scoped);
    let mb=resource(&mp,mk)?; let cb=resource(&cp,ck)?;
    let mr=rw4::parse(&mb)?;let cr=rw4::parse(&cb)?;
    let ma=rw4_animation::parse(&mr,&mb)?;let ca=rw4_animation::parse(&cr,&cb)?;
    let model_binding=ma.bindings.iter().find(|v|v.section_index==10).ok_or("candidate binding 10 missing")?.value;
    let clip_binding=ca.bindings.iter().find(|v|v.section_index==binding_index).ok_or("clip binding missing")?.value;
    let model_skeleton=&ma.skeletons.iter().find(|v|v.section_index==model_binding.skeleton).ok_or("model skeleton missing")?.value;
    let clip_skeleton=&ca.skeletons.iter().find(|v|v.section_index==clip_binding.skeleton).ok_or("clip skeleton missing")?.value;
    let model_skin=&ma.animation_skins.iter().find(|v|v.section_index==model_binding.animation_skin).ok_or("model skin missing")?.value;
    let clip_skin=&ca.animation_skins.iter().find(|v|v.section_index==clip_binding.animation_skin).ok_or("clip skin missing")?.value;
    let clip=&ca.clips.iter().find(|v|v.section_index==clip_index).ok_or("clip section missing")?.value;
    if model_skeleton.id!=0x379cc22e || model_skeleton.bones.len()!=9 || clip_skeleton.id!=model_skeleton.id || clip_skeleton.bones!=model_skeleton.bones || clip.id!=model_skeleton.id {return Err("skeleton identity/hierarchy mismatch".into());}
    if clip.tracks.len()!=model_skeleton.bones.len() || clip.tracks.iter().zip(&model_skeleton.bones).any(|(t,b)|t.bone_id!=b.id) || clip.tracks.iter().map(|t|t.bone_id).collect::<BTreeSet<_>>().len()!=clip.tracks.len() {return Err("clip track identity/order mismatch".into());}
    if model_skin.raw_columns.len()!=clip_skin.raw_columns.len(){return Err("inverse-bind count mismatch".into());}
    let mut differing=0usize; let mut max_delta=0f32;
    for (a,b) in model_skin.raw_columns.iter().zip(&clip_skin.raw_columns) {
        for (&x,&y) in a.iter().zip(b) {
            if !x.is_finite()||!y.is_finite(){return Err("nonfinite inverse bind".into());}
            if x.to_bits()!=y.to_bits(){differing+=1;}
            max_delta=max_delta.max((x-y).abs());
        }
    }
    println!("POSE_CHECK model_package={} model_key={} clip_package={} clip_key={} model_binding=10 clip_binding={} clip_section={} skeleton={:08X} joints={} inverse_bind_word_differences={} inverse_bind_max_abs={} runtime_selection=unverified interpolation=rustspore_linear_slerp",args[0],mk,args[1],ck,binding_index,clip_index,model_skeleton.id,model_skeleton.bones.len(),differing,max_delta);
    if differing!=0 {return Err("inverse binds differ; source retargeting must be established before pose transfer".into());}
    let samples=clip.sample(time,false)?;
    let mut global=Vec::<AffineTransform>::with_capacity(samples.len());let mut skin=Vec::with_capacity(samples.len());
    for (i,sample) in samples.iter().enumerate(){
        if sample.bone_id!=model_skeleton.bones[i].id{return Err("sample order mismatch".into());}
        let local=match sample.value {SampleValue::Transform(t)=>t.affine()?,_=>return Err("non-transform channel".into())};
        let posed=if let Some(p)=model_skeleton.bones[i].parent {if p>=i{return Err("unsupported parent ordering".into());}global[p].then_local(local)}else{local};
        let matrix=posed.then_local(model_skin.inverse_bind[i]);
        if matrix.rows.iter().flatten().any(|x|!x.is_finite()){return Err("nonfinite skin matrix".into());}
        global.push(posed);skin.push(matrix);
    }
    let mesh=mr.mesh(&mb,13).ok_or("candidate mesh missing")?;
    let influences=rw4_animation::vertex_skinning(&mr,&mb,&mesh,skin.len())?;
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(out)?;
    writeln!(file,"joint\tbone_id\trow\tx\ty\tz\tw\tmax_identity_delta")?;
    let mut max_identity=0f32;
    for (i,m) in skin.iter().enumerate(){let delta=m.rows.iter().flatten().zip(AffineTransform::IDENTITY.rows.iter().flatten()).map(|(a,b)|(a-b).abs()).fold(0f32,f32::max);max_identity=max_identity.max(delta);
        for (r,v) in m.rows.iter().enumerate(){writeln!(file,"{i}\t{:08X}\t{r}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",model_skeleton.bones[i].id,v[0],v[1],v[2],v[3],delta)?;}}
    let effective_time=time.clamp(0.,clip.duration.max(0.));
    println!("POSE_OUTPUT requested_time={} effective_time={} clip_duration={} joints={} vertices={} max_identity_delta={} retarget_applied=false pose_transfer_validity=unverified runtime_selection=unverified interpolation=rustspore_linear_slerp palette_upload=not_performed output={}",time,effective_time,clip.duration,skin.len(),influences.len(),max_identity,out.display());
    Ok(())
}
