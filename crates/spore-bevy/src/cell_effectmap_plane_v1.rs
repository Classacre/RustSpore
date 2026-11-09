//! Explicit hash-bound effect-map plane asset; caller supplies placement and runtime state.
use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
use sha2::{Digest,Sha256};
use spore_core::{material_links,mesh,prop,raster,rw4};
use std::{path::Path,io::Read};
pub struct PlaneArt {pub geometry:mesh::MeshData,pub pixels:Vec<u8>}
fn key(id:u32,typ:u32)->NativeResourceKey{NativeResourceKey{instance_id:id,type_id:typ,group_id:0}}
fn open(path:&Path)->Result<NativePackageIndex,String>{
 let mut h=[0;dbpf::HEADER_SIZE];std::fs::File::open(path).and_then(|mut f|f.read_exact(&mut h)).map_err(|e|e.to_string())?;
 let h=dbpf::DbpfHeader::parse(&h).map_err(|e|e.to_string())?;if h.index_size>64*1024*1024||h.index_entry_count>2_000_000{return Err("package index exceeds bound".into())}NativePackageIndex::open(path).map_err(|e|e.to_string())
}
fn read(p:&NativePackageIndex,k:NativeResourceKey)->Result<Vec<u8>,String>{let e=p.index().find_unique(k).map_err(|e|e.to_string())?.ok_or("source key missing")?;if e.stored_size>16*1024*1024||e.decompressed_size>16*1024*1024{return Err("resource exceeds bound".into())}let b=p.read(e).map_err(|e|e.to_string())?;if b.len()!=e.decompressed_size as usize{return Err("resource extent mismatch".into())}Ok(b)}
pub fn load(install:&Path)->Result<PlaneArt,String>{
 let props=open(&install.join("Data/PatchData.package"))?;let graphics=open(&install.join("Data/Spore_Graphics.package"))?;
 let pb=read(&props,key(0x44c489b7,0x00b1b104))?;
 if format!("{:x}",Sha256::digest(&pb))!="61d1eeefa544d0ddaf134cfea8bc9f8c4f6233abd58ea042506de7aa3add6da9"{return Err("PROP hash mismatch".into())}
 let tree=prop::parse(&pb).map_err(|e|e.to_string())?;if !tree.exact||tree.consumed!=pb.len()||tree.key(prop::idents::MODEL_LOD0)!=Some((0x44c489b7,0x2f4e681b)){return Err("PROP LOD0 mismatch".into())}
 let bytes=read(&graphics,key(0x44c489b7,0x2f4e681b))?;if format!("{:x}",Sha256::digest(&bytes))!="bbb3fca35d99c40de940a6d5bf9f6b59717661c79be3f7eae164cbcec2844ec2"{return Err("RW4 hash mismatch".into())}
 let model=rw4::parse(&bytes).map_err(|e|e.to_string())?;let link=model.object(&bytes,9).ok_or("mesh-state link missing")?;if link!=[7u32,1,8].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>(){return Err("mesh-state link mismatch".into())}
 let state=material_links::compiled_state(&model,&bytes,8).map_err(|e|e.to_string())?;if state.shader_id!=0x80000033{return Err("saved shader mismatch".into())}
 let bindings=state.bindings().map_err(|e|e.to_string())?;let stage=bindings.textures.iter().find(|s|s.stage==0).ok_or("stage zero missing")?;
 if bindings.textures.iter().filter(|s|s.stage==0).count()!=1||stage.raster_reference!=11||stage.list_b.iter().map(|p|(p.state,p.value)).collect::<Vec<_>>()!=[(1,1),(2,1),(5,2),(6,2),(7,1)]{return Err("saved texture or sampler mismatch".into())}
 let r=model.raster(&bytes,11).ok_or("raster missing")?;if (r.width,r.height,r.mipmap_levels,r.format,r.data)!=(256,256,9,raster::FORMAT_DXT1,10){return Err("raster metadata mismatch".into())}
 let blocks=model.object(&bytes,r.data).ok_or("raster bytes missing")?;let mut offset=0;let mut pixels=Vec::new();
 for level in 0..9{let w=(256u32>>level).max(1);let h=(256u32>>level).max(1);let size=(w as usize).div_ceil(4)*(h as usize).div_ceil(4)*8;let end=offset+size;let data=blocks.get(offset..end).ok_or("truncated mip")?;pixels.extend(raster::decode_pixels(r.format,w,h,data).map_err(|e|e.to_string())?);offset=end;}
 if offset!=blocks.len(){return Err("trailing mip data".into())}
 if pixels.chunks_exact(4).any(|p|p[3]!=255){return Err("plane correction requires opaque mip pixels".into())}
 let geometry=mesh::from_rw4(&bytes,&model).map_err(|e|e.to_string())?;geometry.validate().map_err(|e|e.to_string())?;
 let pos:Vec<_>=geometry.positions.iter().map(|p|[p.x,p.y,p.z]).collect();
 if pos!=[[-0.5,-0.5,0.],[0.5,-0.5,0.],[0.5,0.5,0.],[-0.5,0.5,0.]]||geometry.uvs!=[[0.,0.],[1.,0.],[1.,-1.],[0.,-1.]]||geometry.indices!=[0,1,3,1,2,3]{return Err("source geometry mismatch".into())}
 Ok(PlaneArt{geometry,pixels})
}
