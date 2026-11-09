//! Compare the synthetic actor interpolation with four retained native rows.
use spore_core::cell_camera_actor_branch_v1::{project,Slot};
fn w(b:&[u8],o:usize)->u32 { u32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn main() {
 let mut args=std::env::args().skip(1);let b=std::fs::read(args.next().expect("native stdout")).unwrap();let map=std::fs::read_to_string(args.next().expect("pinned host map")).unwrap();assert_eq!(b.len(),144);
 let lines:Vec<_>=map.lines().filter(|l|l.split_whitespace().nth(1)==Some("Actors")).collect();assert_eq!(lines.len(),1);let actor=u32::from_str_radix(lines[0].split_whitespace().nth(2).unwrap(),16).unwrap();
 let p=[5f32.to_bits(),8f32.to_bits(),11f32.to_bits()];let slots=[Slot{handle:0x10000,position:p},Slot{handle:0x10001,position:p}];let old=[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()];
 for (i,handle) in [0,0x10000,0x20000,0x10001].into_iter().enumerate() {
  let x=project(handle,1,&slots,old,0.5f32.to_bits(),0x1f80,true).unwrap();
  let pointer=x.resolved_index.map_or(0,|j|actor+(j as u32)*0x80);assert_eq!(w(&b,i*36),pointer);
  for j in 0..3 { assert_eq!(w(&b,i*36+4+j*4),x.vector[j],"case {i}/{j}"); }
  let fp=w(&b,i*36+16);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*36+20)&!0x3f,0x1f80);assert_eq!(w(&b,i*36+24),handle);assert_eq!(w(&b,i*36+28),0xc0dec0de);assert_eq!(w(&b,i*36+32),0xa5a5a5a5);
 }
 println!("matched four retained actor-camera interpolation rows; synthetic pool and scalar admitted");
}
