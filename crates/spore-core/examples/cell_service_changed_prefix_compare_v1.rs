use spore_core::{cell_service_action_gate_v1::Branch,cell_service_changed_prefix_v1::{project,MapResult}};
fn w(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native stdout")).unwrap();assert_eq!(b.len(),120);for i in 0..3{let fresh=if i==2{0x44444444}else{0x11111111};let found=i!=0;let p=project(Branch::Changed,0x11111111,0x33333333,fresh,if found{MapResult::Found(0x12345678)}else{MapResult::Missing},true).unwrap();
 // Inert fixture stubs preserve +8C after its source write; counts do not imply live calls.
 let got=[p.old_for_first,p.old_for_pair,p.requested,1,p.pending_found_callback.is_some() as u32,fresh,p.service_8c_before_pair,if found{4}else{3}];for(j,x)in got.into_iter().enumerate(){assert_eq!(x,w(&b,i*40+j*4),"case {i} word {j}")}let fp=w(&b,i*40+32);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*40+36)&!0x3f,0x1f80);}println!("matched three retained changed-prefix rows; callbacks and transition pending");}


