//! Bounded E61390 successful-construction projection with scripted helpers.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, UnsupportedControl }
#[derive(Clone,Debug,PartialEq,Eq)] pub struct Outcome { pub object:[u8;0xec], pub root_51da:u8 }
/// Caller admits successful allocation of a distinct stable object/root, and that
/// helper1/helper2 perform only the explicitly supplied dword writes at +78/+34.
/// This does not run real allocation/helpers, install P or describe loaded state.
pub fn construct(mut object:[u8;0xec],helper_78:u32,helper_34:u32,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 fn put(b:&mut [u8;0xec],o:usize,v:u32){b[o..o+4].copy_from_slice(&v.to_le_bytes())}
 for (o,v) in [(0,0x1486048),(4,0x140e924),(8,0),(0x10,0),(0x14,0),(0x18,0),(0x1c,0),(0x20,0),(0x24,0),(0x28,0),(0x2c,0),(0x30,10),(0x6c,0),(0x70,0),(0x74,0),(0x7c,1),(0xe4,0)]{put(&mut object,o,v)}
 for (o,v) in [(0x68,0),(0x69,0),(0xe0,0),(0xe1,0),(0xe2,1),(0xe8,0),(0xe9,0)]{object[o]=v}
 put(&mut object,0x78,helper_78);put(&mut object,0x34,helper_34);object[0x80..0xe0].fill(0);
 Ok(Outcome{object,root_51da:0})
}
#[cfg(test)]mod tests{use super::*;#[test]fn defaults_and_unwritten_bytes(){let x=construct([0xa5;0xec],1,2,0x27f,0x1f80,true).unwrap();assert_eq!(&x.object[0x1c..0x30],&[0;20]);assert_eq!(x.object[0x0c],0xa5);assert_eq!(x.object[0xe2],1);assert_eq!(x.root_51da,0);assert_eq!(construct([0;0xec],0,0,0,0x1f80,true),Err(Error::UnsupportedControl));}}
