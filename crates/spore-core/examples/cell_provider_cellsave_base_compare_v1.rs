//! Compare base-object projection with all four retained original rows.
use spore_core::cell_provider_cellsave_base_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native stdout")).unwrap();assert_eq!(b.len(),1008);for i in 0..4{let cw=if i>=2{0x37f}else{0x27f};let x=project([0xa5;0xec],i%2==0,cw,0x1f80,true).unwrap();let row=&b[i*252..(i+1)*252];assert_eq!(&row[..0xec],&x.storage);assert_eq!(w(row,0xec),u32::from(x.returned_object));assert_eq!(w(row,0xf0),1);let fp=w(row,0xf4);assert_eq!(fp&0xffff,cw as u32);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(row,0xf8)&!0x3f,0x1f80);}println!("matched four retained CellSaveGame base callback rows; synthetic allocator admitted");}
