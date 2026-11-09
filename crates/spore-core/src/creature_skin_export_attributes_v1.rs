//! Bounded 46E717..46E775 attribute selection with available-capacity appends.
//! Inputs are supplied same-object snapshots; copied normal indices, lifecycle,
//! first four groups and live material/identity are not established here.
//! Only the first three words of each six-word tangent/bitangent pair are emitted.
#[derive(Debug, PartialEq, Eq)]
pub enum Error { IndexOutOfRange, UnsupportedFloat }
#[derive(Debug, PartialEq, Eq)]
pub struct Attributes { pub positions: Vec<[u32;3]>, pub normals: Vec<[u32;3]>, pub uvs: Vec<[u32;2]>, pub tangents: Vec<[u32;3]> }
fn admitted(x:u32)->bool { let e=x&0x7f80_0000; e!=0x7f80_0000 && (e!=0 || x&0x7fff_ffff==0) }
/// The source helpers use x87 loads and stores. This bounded projection admits
/// normal finite values and signed zeros, whose bits survive those copies.
pub fn project(triples:&[[u32;3]], positions:&[[u32;3]], normals:&[[u32;3]], uvs:&[[u32;2]], pairs:&[[u32;6]]) -> Result<Attributes,Error> {
 let mut o=Attributes{positions:Vec::with_capacity(triples.len()),normals:Vec::with_capacity(triples.len()),uvs:Vec::with_capacity(triples.len()),tangents:Vec::with_capacity(triples.len())};
 for &[pi,ni,ui] in triples {
  let p=*positions.get(pi as usize).ok_or(Error::IndexOutOfRange)?;
  let n=*normals.get(ni as usize).ok_or(Error::IndexOutOfRange)?;
  let u=*uvs.get(ui as usize).ok_or(Error::IndexOutOfRange)?;
  let pair=pairs.get(ui as usize).ok_or(Error::IndexOutOfRange)?;
  let t=[pair[0],pair[1],pair[2]];
  if !p.iter().chain(&n).chain(&u).chain(&t).copied().all(admitted){return Err(Error::UnsupportedFloat)}
  o.positions.push(p);o.normals.push(n);o.uvs.push(u);o.tangents.push(t);
 }
 Ok(o)
}
#[cfg(test)] mod tests { use super::*;
 #[test] fn selects_independent_indices_and_only_tangent_half(){let p=[[0x3f800000,0,0x80000000],[0x40000000,0,0]];let n=[[0x40400000,0,0],[0x40800000,0,0]];let uv=[[0x3f000000,0],[0,0x3f800000]];let pairs=[[0x3f800000,0,0,0x7fc00000,0,0],[0,0x40000000,0,1,2,3]];let o=project(&[[1,0,1],[0,1,0]],&p,&n,&uv,&pairs).unwrap();assert_eq!(o.positions,[p[1],p[0]]);assert_eq!(o.normals,[n[0],n[1]]);assert_eq!(o.uvs,[uv[1],uv[0]]);assert_eq!(o.tangents,[[0,0x40000000,0],[0x3f800000,0,0]]);}
 #[test] fn rejects_bad_index_and_unsupported_float(){let p=[[0,0,0]];let uv=[[0,0]];let t=[[0;6]];assert_eq!(project(&[[1,0,0]],&p,&p,&uv,&t),Err(Error::IndexOutOfRange));assert_eq!(project(&[[0,0,0]],&[[1,0,0]],&p,&uv,&t),Err(Error::UnsupportedFloat));}
}
