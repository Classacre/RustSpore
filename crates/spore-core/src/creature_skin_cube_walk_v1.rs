//! Bounded constructor-derived tables and one-cube 540070 directed walk.
//! Not a queue, finalized mesh, native allocation or runtime identity model.
const EDGES:[[u8;2];12]=[[0,4],[4,6],[2,6],[0,2],[0,1],[4,5],[6,7],[2,3],[1,5],[5,7],[3,7],[1,3]];
const FACES:[[u8;4];6]=[[0,2,6,4],[1,3,7,5],[0,4,5,1],[2,6,7,3],[0,1,3,2],[4,5,7,6]];
/// Constructor 53E410's six face records, admitting zero-initialized
/// unwritten bytes in the synthetic constructor frame.
pub fn face_records()->[[u8;32];6]{
 let mut records=[[0u8;32];6];
 for (i,face) in FACES.iter().enumerate(){let r=&mut records[i];r[0]=i as u8;r[1]=(i^1) as u8;r[2]=1<<i;r[3]=1<<(i^1);let axis=i/2;r[8+axis]=if i%2==0{255}else{1};for (j,&corner) in face.iter().enumerate(){r[4+j]=1<<corner;for k in 0..3{r[16+4*j+k]=(corner>>k)&1;}}r[12]=r[4]|r[5]|r[6]|r[7];let shift=1<<(i/2);r[13]=if i%2==0{r[12]<<shift}else{r[12]>>shift};}
 records
}
#[derive(Debug,Clone)]pub struct Tables{pub masks:[u16;256],pub orientation:[u8;24],pub endpoints:[u8;24],pub successor:[[[u8;4];24];3]}
/// Admits zero initialization of each successor entry's unwritten fourth byte.
pub fn tables()->Tables{
 let mut t=Tables{masks:[0;256],orientation:[0;24],endpoints:[0;24],successor:[[[0;4];24];3]};let mut map=[[u8::MAX;8];8];
 for (i,e) in EDGES.iter().enumerate(){let d=(i as u8)*2+u8::from(e[0]>e[1]);map[e[0] as usize][e[1] as usize]=d;map[e[1] as usize][e[0] as usize]=d^1;t.endpoints[2*i]=e[0];t.endpoints[2*i+1]=e[1];t.orientation[2*i]=1<<e[0];t.orientation[2*i+1]=1<<e[1];}
 for (face,f) in FACES.iter().enumerate(){let dir=if face&1==0{-1}else{1};for j in 0..4i32{let at=|x:i32|f[x.rem_euclid(4) as usize] as usize;let a=at(j);let b=at(j+dir);let c=at(j+2*dir);let d=at(j+3*dir);let current=map[a][b] as usize;for(k,next) in [map[c][b],map[d][c],map[a][d]].into_iter().enumerate(){let m=1u16<<(next>>1);t.successor[k][current]=[m as u8,(m>>8) as u8,next,0];}}}
 for c in 0..256{for (i,e) in EDGES.iter().enumerate(){if ((c>>e[0])&1)!=((c>>e[1])&1){t.masks[c]|=1<<i;}}}t
}
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidTables,StepLimit,Callback}
/// `cube`'s low three bytes are coordinates in native byte order; additions
/// wrap each byte and clear the fourth. The callback receives ordered cache key,
/// then the directed pair of packed corner coordinates. Already-emitted triangles
/// and callback effects survive failure. No claim is made for arbitrary tables.
pub fn walk<F>(t:&Tables,cube:u32,class:u8,triangles:&mut Vec<[u32;3]>,mut edge:F)->Result<(),Error>where F:FnMut([u32;2],[u32;2])->Option<u32>{
 let mut bits=t.masks[class as usize];let mut steps=0usize;
 while bits!=0 {let e=bits.trailing_zeros() as usize;if e>=12{return Err(Error::InvalidTables)}let start=2*e+usize::from(class&t.orientation[2*e]!=0);let mut d=start;let(mut first,mut prev)=(u32::MAX,u32::MAX);
 loop{steps+=1;if steps>256{return Err(Error::StepLimit)}if d>=24{return Err(Error::InvalidTables)}let a=t.endpoints[d] as u32;let b=t.endpoints[d^1] as u32;if a>=8||b>=8{return Err(Error::InvalidTables)}
 let corner=|n:u32|{let mut p=0u32;for k in 0..3{let x=((cube>>(8*k))&255).wrapping_add((n>>k)&1)&255;p|=x<<(8*k);}p};let p=[corner(a),corner(b)];let key=if d&1==0{p}else{[p[1],p[0]]};let v=edge(key,p).ok_or(Error::Callback)?;
 if first==u32::MAX{first=v}else if prev==u32::MAX{prev=v}else{if first!=prev&&first!=v&&prev!=v{triangles.push([first,prev,v]);}prev=v}
 let x=t.successor[0][d];let y=t.successor[1][d];let z=t.successor[2][d];let mask=|q:[u8;4]|u16::from_le_bytes([q[0],q[1]]);let choice=if mask(x)&bits!=0{x}else if mask(y)&bits!=0{y}else{z};bits^=mask(choice);d=choice[2] as usize;if d==start{break}
 }
 }Ok(())
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn face_records_have_source_masks(){let f=face_records();assert_eq!(&f[0][0..16],&[0,1,1,2,1,4,64,16,255,0,0,0,0x55,0xaa,0,0]);assert_eq!(f[1][12..14],[0xaa,0x55]);}
 #[test]fn all_classes_finish_and_uniform_empty(){let t=tables();for c in 0..=255u8{let mut tris=vec![];let mut calls=0;walk(&t,0,c,&mut tris,|_,_|{calls+=1;Some(calls)}).unwrap();if c==0||c==255{assert_eq!((calls,tris.len()),(0,0));}else{assert!(calls>=3);}}}
 #[test]fn degeneracy_and_failure_preserve_prior_output(){let t=tables();let mut tr=vec![[9,8,7]];walk(&t,0,1,&mut tr,|_,_|Some(3)).unwrap();assert_eq!(tr,vec![[9,8,7]]);let mut n=0;assert_eq!(walk(&t,0,3,&mut tr,|_,_|{n+=1;if n==2{None}else{Some(4)}}),Err(Error::Callback));assert_eq!(n,2);assert_eq!(tr,vec![[9,8,7]]);}
 #[test]fn source_index_sentinel_is_not_a_fan_anchor(){let t=tables();let mut n=0u32;let mut triangles=vec![];walk(&t,0,3,&mut triangles,|_,_|{n+=1;Some(if n==1{u32::MAX}else{n-1})}).unwrap();assert_eq!(triangles,vec![[1,2,3]]);}
 #[test]fn coordinate_bytes_wrap_and_clear_high_byte(){let t=tables();let mut seen=vec![];walk(&t,0xaaffffff,1,&mut vec![],|k,p|{seen.push((k,p));Some(seen.len() as u32)}).unwrap();assert!(seen.iter().flat_map(|(k,p)|k.iter().chain(p)).all(|v|v>>24==0));assert!(seen.iter().flat_map(|(_,p)|p).any(|p|p&0xff==0));}
}
