//! Explicit hash-bound effect-map plane asset; caller supplies placement and runtime state.
use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
use sha2::{Digest,Sha256};
use spore_core::{material_links,mesh,prop,raster,rw4};
use std::{path::Path,io::Read};
use super::cell_effectmap_plane_v1::PlaneArt;
fn key(id:u32,typ:u32)->NativeResourceKey{NativeResourceKey{instance_id:id,type_id:typ,group_id:0}}
fn open(path:&Path)->Result<NativePackageIndex,String>{
 let mut h=[0;dbpf::HEADER_SIZE];std::fs::File::open(path).and_then(|mut f|f.read_exact(&mut h)).map_err(|e|e.to_string())?;
 let h=dbpf::DbpfHeader::parse(&h).map_err(|e|e.to_string())?;if h.index_size>64*1024*1024||h.index_entry_count>2_000_000{return Err("package index exceeds bound".into())}NativePackageIndex::open(path).map_err(|e|e.to_string())
}
fn read(p:&NativePackageIndex,k:NativeResourceKey)->Result<Vec<u8>,String>{let e=p.index().find_unique(k).map_err(|e|e.to_string())?.ok_or("source key missing")?;if e.stored_size>16*1024*1024||e.decompressed_size>16*1024*1024{return Err("resource exceeds bound".into())}let b=p.read(e).map_err(|e|e.to_string())?;if b.len()!=e.decompressed_size as usize{return Err("resource extent mismatch".into())}Ok(b)}
pub fn load(install:&Path)->Result<PlaneArt,String>{
 let props=open(&install.join("Data/PatchData.package"))?;let graphics=open(&install.join("Data/Spore_Graphics.package"))?;
 let pb=read(&props,key(0x154c862b,0x00b1b104))?;
 if format!("{:x}",Sha256::digest(&pb))!="24320dfe4660ac730a979d39a5d11e9956cd484b9a51a174bf517680d21c1bb8"{return Err("PROP hash mismatch".into())}
 let tree=prop::parse(&pb).map_err(|e|e.to_string())?;if !tree.exact||tree.consumed!=pb.len()||tree.key(prop::idents::MODEL_LOD0)!=Some((0x154c862b,0x2f4e681b)){return Err("PROP LOD0 mismatch".into())}
 let bytes=read(&graphics,key(0x154c862b,0x2f4e681b))?;if format!("{:x}",Sha256::digest(&bytes))!="7b54e34f27d07c1526f475f37fa259c10aa8fe4d0ae5683dc149b7398e4f4d5e"{return Err("RW4 hash mismatch".into())}
 let model=rw4::parse(&bytes).map_err(|e|e.to_string())?;let link=model.object(&bytes,9).ok_or("mesh-state link missing")?;if link!=[7u32,1,8].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>(){return Err("mesh-state link mismatch".into())}
 let state=material_links::compiled_state(&model,&bytes,8).map_err(|e|e.to_string())?;if state.shader_id!=0x80000002{return Err("saved shader mismatch".into())}
 let bindings=state.bindings().map_err(|e|e.to_string())?;let stage=bindings.textures.iter().find(|s|s.stage==0).ok_or("stage zero missing")?;
 if bindings.textures.iter().filter(|s|s.stage==0).count()!=1||stage.raster_reference!=11||stage.list_b.iter().map(|p|(p.state,p.value)).collect::<Vec<_>>()!=[(1,1),(2,1),(5,2),(6,2),(7,1)]{return Err("saved texture or sampler mismatch".into())}
 let r=model.raster(&bytes,11).ok_or("raster missing")?;if (r.width,r.height,r.mipmap_levels,r.format,r.data)!=(256,256,9,raster::FORMAT_DXT5,10){return Err("raster metadata mismatch".into())}
 let blocks=model.object(&bytes,r.data).ok_or("raster bytes missing")?;let mut offset=0;let mut pixels=Vec::new();
 for level in 0..9{let w=(256u32>>level).max(1);let h=(256u32>>level).max(1);let size=(w as usize).div_ceil(4)*(h as usize).div_ceil(4)*16;let end=offset+size;let data=blocks.get(offset..end).ok_or("truncated mip")?;pixels.extend(raster::decode_pixels(r.format,w,h,data).map_err(|e|e.to_string())?);offset=end;}
 if offset!=blocks.len(){return Err("trailing mip data".into())}
 let stage1=bindings.textures.iter().find(|s|s.stage==1).ok_or("stage one missing")?;
 if bindings.textures.len()!=2||stage1.raster_reference!=13||stage1.list_b.iter().map(|p|(p.state,p.value)).collect::<Vec<_>>()!=[(1,1),(2,1),(5,2),(6,2),(7,1)]{return Err("stage one state mismatch".into())}
 let r1=model.raster(&bytes,13).ok_or("stage one raster")?;if(r1.width,r1.height,r1.mipmap_levels,r1.format,r1.data)!=(256,256,9,raster::FORMAT_DXT5,12){return Err("stage one metadata".into())}
 let b1=model.object(&bytes,12).ok_or("stage one data")?;let mut at=0;for level in 0..9{let side=(256u32>>level).max(1);let n=(side as usize).div_ceil(4)*(side as usize).div_ceil(4)*16;let end=at+n;raster::decode_pixels(r1.format,side,side,b1.get(at..end).ok_or("stage one mip")?).map_err(|e|e.to_string())?;at=end;}if at!=b1.len(){return Err("stage one trailing".into())}
 let geometry=mesh::from_rw4(&bytes,&model).map_err(|e|e.to_string())?;geometry.validate().map_err(|e|e.to_string())?;
 if geometry.positions.len()!=1701||geometry.uvs.len()!=1701||geometry.indices.len()!=9600||geometry.indices.iter().any(|&i|i>=1701){return Err("source geometry mismatch".into())}
 Ok(PlaneArt{geometry,pixels})
}
