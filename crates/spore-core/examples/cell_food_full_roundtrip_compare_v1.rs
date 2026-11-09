//! Compare writer frame and host-guarded reader outcomes for four controlled rows.
use spore_core::cell_food_full_roundtrip_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),224);for i in 0..4{let v=(i%2) as u32;let x=project(0x11223344,v,v+1000,true).unwrap();let r=&b[i*56..(i+1)*56];assert_eq!((x.food,x.plant),(v,v+1000));assert_eq!((w(r,0),w(r,4),w(r,8)),(1,36,3));assert_eq!(&r[12..48],&x.frame);let fp=w(r,48);assert_eq!(fp&0xffff,if i>=2{0x37f}else{0x27f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,52)&!0x3f,0x1f80);}println!("matched four writer frames; native reader fields and descriptor restoration are host-guarded, synthetic only");}
