//! Compare source record initialization with retained native output.
use spore_core::cell_transition_record_init_v1::initialize;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),1152);let fs=[0,0x80000000,0x3f800000,0x3eaaaaab];
 for i in 0..8{let cw=if i<4{0x27f}else{0x37f};let x=initialize([0xa5;0x7c],fs[i%4],[0x11223344,0x55667788,0x99aabbcc],[0x10203040,0x50607080,0x90a0b0c0,0xd0e0f001],0x12345678,cw,0x1f80,true).unwrap();let row=&b[i*144..(i+1)*144];assert_eq!(&x.record[..],&row[..124],"record {i}");assert_eq!(x.root_5194,w(row,124));assert_eq!(x.pending_notification as u32,w(row,128));let fp=w(row,132);assert_eq!(fp&0xffff,cw as u32);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(row,136)&!0x3f,0x1f80);}
 println!("matched 8 retained record snapshots; allocation and scheduling unobserved");}
