//! Compare bounded food writer dispatch with eight retained source rows.
use spore_core::cell_food_writer_dispatch_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let mut args=std::env::args().skip(1);let b=std::fs::read(args.next().expect("native rows")).unwrap();let map=std::fs::read_to_string(args.next().expect("host map")).unwrap();assert_eq!(b.len(),256);
 let symbol=|name:&str|{let v:Vec<_>=map.lines().filter(|l|l.split_whitespace().nth(1)==Some(name)).collect();assert_eq!(v.len(),1);u32::from_str_radix(v[0].split_whitespace().nth(2).unwrap(),16).unwrap()};let a=symbol("ObjectA");let alt=symbol("ObjectB");
 for i in 0..8{let preexisting=i%2==1;let x=project(if preexisting{alt}else{0},a,0x1c,if preexisting{1000}else{0},true).unwrap();assert_eq!(x.field_address,x.descriptor_base+0x1c);let r=&b[i*32..(i+1)*32];assert_eq!((w(r,0),w(r,4),w(r,8),w(r,12),w(r,16)),(1,if preexisting{2}else{1},u32::from_le_bytes(x.encoded),1,2));let fp=w(r,20);assert_eq!(fp&0xffff,if i>=4{0x37f}else{0x27f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,24)&!0x3f,0x1f80);assert_eq!(w(r,28),0x10001);}
 println!("matched eight synthetic food descriptor writer rows; no live traversal or file");}
