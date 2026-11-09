use spore_core::cell_mode_generic_name_v1::cell_to_creature;
fn w(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native stdout")).unwrap();assert_eq!(b.len(),32);let p=cell_to_creature(0x1654c00,0x1654c01,1,32,true).unwrap();
 // Counts and opaque return are properties of the inert native comparator stubs.
 let got=[1,1,p.pending_service as u32,p.pending_service as u32,p.pending_cleanup as u32,0x12345678];for (i,x) in got.into_iter().enumerate(){assert_eq!(x,w(&b,i*4),"word {i}")}let fp=w(&b,24);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,28)&!0x3f,0x1f80);println!("matched retained scripted generic path; helper and service effects excluded");}
