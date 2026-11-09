//! Narrow source-formula approximation for the hash-bound startup candidate.
//! This is not a source shader, lighting, live pose or retail renderer.
use sha2::{Digest, Sha256};
use spore_core::{gmdl, mesh::MeshData, Vec3};

fn unpack(b: [u8;3]) -> Result<Vec3,String> {
    // The source's 127-centred degenerate vector has GPU-specific NRM
    // behaviour. It is excluded rather than assigned an invented direction.
    if b == [127;3] { return Err("source normal is degenerate".into()); }
    let k=f32::from_bits(0x3c010204);
    let v=Vec3::new((b[0] as f32).mul_add(k,-1.0),(b[1] as f32).mul_add(k,-1.0),(b[2] as f32).mul_add(k,-1.0));
    let len=v.length();
    if !len.is_finite() || len<=1e-8 { return Err("source normal is nonfinite or near zero".into()); }
    Ok(Vec3::new(v.x/len,v.y/len,v.z/len))
}

pub(super) fn replacement(raw:&[u8], supplied:&MeshData)->Result<Vec<Vec3>,String>{
    if format!("{:x}",Sha256::digest(raw))!="95ba0f5e5e71761ccf0f2ef919cce9bcc8d40303be4b1b7a9fa954a7c7cb8ff4" {return Err("startup model hash mismatch".into())}
    let model=gmdl::parse(raw).map_err(|e|e.to_string())?;
    if model.meshes.len()!=1{return Err("startup mesh count mismatch".into())}
    let m=&model.meshes[0];let vb=model.vertex_buffers.get(usize::try_from(m.vertex_index).map_err(|_|"vertex index")?).ok_or("vertex reference")?;
    let tri=model.triangle_buffers.get(usize::try_from(m.triangle_index).map_err(|_|"triangle index")?).ok_or("triangle reference")?;
    let fmt=model.vertex_formats.get(usize::try_from(vb.format_index).map_err(|_|"format index")?).ok_or("format reference")?;
    if (vb.vertex_count,vb.stride,tri.indices.len(),tri.primitive_type)!=(2555,36,12732,4){return Err("startup geometry extent mismatch".into())}
    let decl:Vec<_>=fmt.elements.iter().map(|e|(e.stream,e.offset,e.element_type,e.method,e.usage,e.usage_index)).collect();
    if decl!=[(0,0,2,0,0,0),(0,12,5,0,3,0),(0,16,1,0,5,0),(0,24,5,0,6,0),(0,28,5,0,2,0),(0,32,8,0,1,0)] {return Err("startup declaration mismatch".into())}
    let bytes=raw.get(vb.data_offset..vb.data_offset.checked_add(vb.stride.checked_mul(vb.vertex_count as usize).ok_or("vertex extent")?).ok_or("vertex extent")?).ok_or("vertex bytes")?;
    if bytes.len()!=vb.buffer_size as usize || format!("{:x}",Sha256::digest(bytes))!="52304dc47a2ba1097fef45ef21c315aab36e677a7b3883ce8c744e3d5240dcb4" {return Err("startup vertex bytes mismatch".into())}
    let indices:Vec<u8>=tri.indices.iter().flat_map(|i|i.to_le_bytes()).collect();
    if format!("{:x}",Sha256::digest(&indices))!="6891da4e135b3dbd32426d96b663448f6978427df99006d95dee06ea150a2fad" {return Err("startup indices mismatch".into())}
    let canonical=spore_core::mesh::from_gmdl(raw,&model).map_err(|e|e.to_string())?;
    let same_vec=|a:&[Vec3],b:&[Vec3]| a.len()==b.len() && a.iter().zip(b).all(|(a,b)|[a.x.to_bits(),a.y.to_bits(),a.z.to_bits()]==[b.x.to_bits(),b.y.to_bits(),b.z.to_bits()]);
    if !same_vec(&canonical.positions,&supplied.positions)||!same_vec(&canonical.normals,&supplied.normals)||canonical.indices!=supplied.indices||canonical.uvs.len()!=supplied.uvs.len()||!canonical.uvs.iter().zip(&supplied.uvs).all(|(a,b)|a[0].to_bits()==b[0].to_bits()&&a[1].to_bits()==b[1].to_bits())||canonical.sanitised!=0||supplied.sanitised!=0{return Err("startup decoded mesh mismatch".into())}
    bytes.chunks_exact(36).map(|v|unpack([v[12],v[13],v[14]])).collect()
}

/// Raw UBYTE4 components admitted only for the exact checked candidate.
pub(super) fn raw_basis(raw:&[u8],supplied:&MeshData)->Result<(Vec<[f32;3]>,Vec<[f32;4]>),String>{
 let _=replacement(raw,supplied)?;let model=gmdl::parse(raw).map_err(|e|e.to_string())?;let vb=&model.vertex_buffers[model.meshes[0].vertex_index as usize];
 let end=vb.data_offset.checked_add(vb.stride.checked_mul(vb.vertex_count as usize).ok_or("basis extent")?).ok_or("basis extent")?;let bytes=raw.get(vb.data_offset..end).ok_or("basis data")?;
 let mut normals=Vec::with_capacity(2555);let mut tangents=Vec::with_capacity(2555);
 for v in bytes.chunks_exact(36){let nb=[v[12],v[13],v[14]];let tb=[v[24],v[25],v[26]];let n=unpack(nb)?;let t=unpack(tb)?;let dot=n.x*t.x+n.y*t.y+n.z*t.z;if !dot.is_finite()||dot.abs()>0.999999{return Err("startup basis nearly collinear".into())}normals.push([nb[0] as f32,nb[1] as f32,nb[2] as f32]);tangents.push([tb[0] as f32,tb[1] as f32,tb[2] as f32,v[27] as f32]);}
 if normals.len()!=2555{return Err("startup basis count".into())}Ok((normals,tangents))
}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn coefficient_boundaries_and_asymmetry(){
  let n=unpack([0,0,0]).unwrap();assert!((n.x+0.57735026).abs()<1e-6 && n.x==n.y && n.y==n.z);
  let n=unpack([255,255,255]).unwrap();assert!((n.x-0.57735026).abs()<1e-6);
  let n=unpack([64,203,205]).unwrap();assert!((n.x+0.50074172).abs()<1e-5 && (n.y-0.6040694).abs()<1e-5 && (n.z-0.619966).abs()<1e-5);
  assert!(unpack([127,127,127]).is_err());
  let n=unpack([127,127,128]).unwrap();assert!(n.z>0.99999 && n.x.abs()<1e-4 && n.y.abs()<1e-4);
 }
}
