//! Compare checked provider/continuation composition with four original rows.
use spore_core::cell_camera_provider_continuation_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native stdout")).unwrap();assert_eq!(b.len(),208);
 let old=[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()];let global=[5f32.to_bits(),8f32.to_bits(),11f32.to_bits()];let direction=[2f32.to_bits(),(-3f32).to_bits(),4f32.to_bits()];
 for i in 0..4{let mut aggregate=[0u8;24];let count=u32::from(i!=0);aggregate[..4].copy_from_slice(&count.to_le_bytes());aggregate[4..8].copy_from_slice(&0x98c942b3u32.to_le_bytes());let n=match i{2=>2f32,3=>-1f32,_=>1f32};let d=if i==2{1f32}else{2f32};aggregate[12..16].copy_from_slice(&n.to_bits().to_le_bytes());aggregate[20..24].copy_from_slice(&d.to_bits().to_le_bytes());
  let x=project(&aggregate,old,global,if i==0{0}else{2},direction,0x027f,0x1f80,true).unwrap();let o=i*52;assert_eq!(w(&b,o),x.provider.value);for j in 0..3{assert_eq!(w(&b,o+4+4*j),x.provider.vector[j],"provider {i}/{j}");assert_eq!(w(&b,o+20+4*j),x.continuation.output[j],"continuation {i}/{j}");}assert_eq!(w(&b,o+16),x.continuation.helper_value,"ramp {i}");assert_eq!(w(&b,o+32),x.continuation.flags);let fp=w(&b,o+36);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+40)&!0x3f,0x1f80);assert_eq!(w(&b,o+44),0xc0dec0de);assert_eq!(w(&b,o+48),0xa5a5a5a5);
 }println!("matched four retained provider-to-continuation rows; state supplied and capture thunk admitted");}
