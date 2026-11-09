//! Compare checked provider/camera continuation against four retained rows.
use spore_core::cell_camera_provider_join_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("retained native stdout")).unwrap();assert_eq!(b.len(),144);
 let old=[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()];let global=[5f32.to_bits(),8f32.to_bits(),11f32.to_bits()];
 for i in 0..4 {let mut aggregate=[0u8;24];let count=u32::from(i!=0);aggregate[..4].copy_from_slice(&count.to_le_bytes());aggregate[4..8].copy_from_slice(&0x98c942b3u32.to_le_bytes());let n=match i{2=>2f32,3=>-1f32,_=>1f32};let d=if i==2{1f32}else{2f32};aggregate[12..16].copy_from_slice(&n.to_bits().to_le_bytes());aggregate[20..24].copy_from_slice(&d.to_bits().to_le_bytes());
  let x=project(&aggregate,old,global,0x027f,0x1f80,true).unwrap();assert_eq!(x.value,w(&b,i*36));for j in 0..3{assert_eq!(x.vector[j],w(&b,i*36+4+j*4),"case {i}/{j}");}let fp=w(&b,i*36+16);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*36+20)&!0x3f,0x1f80);assert_eq!(w(&b,i*36+24),count);assert_eq!(w(&b,i*36+28),0xc0dec0de);assert_eq!(w(&b,i*36+32),0xa5a5a5a5);assert_eq!(x.found,i!=0);
 }println!("matched four retained data4/camera rows; synthetic aggregate and globals admitted");}
