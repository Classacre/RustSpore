//! Compare pre-open argument preparation against twelve retained source rows.
use spore_core::cell_stream_preopen_arguments_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),384);for i in 0..12{let c=i%6;let bit=(c&1)*2;let mode=if c<4{6}else{5};let mapped=if mode==6{if bit==0{3}else{4}}else{mode};
 // Fixture callback changes +14 to 2222 and preserves the argument stack.
 let x=project(bit as u32,mode,mapped,0x2222,true).unwrap();assert_eq!(x.remapped_before_callback,mapped);let r=&b[i*32..(i+1)*32];assert_eq!((w(r,0),w(r,4),w(r,8),w(r,12)),(0,x.pending_arguments[1],x.pending_arguments[0],x.preparatory_calls));let fp=w(r,16);assert_eq!(fp&0xffff,if i>=6{0x37f}else{0x27f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,20)&!0x3f,0x1f80);assert_eq!((w(r,24),w(r,28)),(0xc0dec0de,0xa5a5a5a5));}println!("matched twelve pre-open source rows; +4C trapped");}
