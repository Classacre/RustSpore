//! Controlled comparison of start-table -> effect curves; source snapshot is caller-authenticated.
use spore_core::cell_effect_curve_v1::from_start_table;
use spore_core::cell_start_table_v1::Row;
use spore_core::native_sse::Policy;
use spore_core::native_x87::Context;
fn word(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let a:Vec<_>=std::env::args().collect();assert_eq!(a.len(),3);let b=std::fs::read(&a[1]).unwrap();assert_eq!(b.len(),20447744);
 let off=0x1083ac4;let rows:Vec<_>=(0..22).map(|i|Row{value_bits:word(&b,off+i*28+16),threshold:word(&b,off+i*28+4) as i32}).collect();let special=word(&b,0x1083d20);let numerator=word(&b,0x10855d0);
 let text=std::fs::read_to_string(&a[2]).unwrap();let mut count=0;
 for line in text.lines(){let p:Vec<_>=line.split_whitespace().collect();assert_eq!(p.len(),4);let progress:i32=p[0].parse().unwrap();let want:Vec<_>=p[1..].iter().map(|v|u32::from_str_radix(v,16).unwrap()).collect();let ctx=Context::from_control_word(0x037f).unwrap();let (x,near,far)=from_start_table(progress,&rows,special,numerator,Policy::Mxcsr1f80,ctx).unwrap();assert_eq!(vec![x.to_bits(),near.to_bits(),far.to_bits()],want,"progress {progress}");count+=1;}
 assert_eq!(count,1001);println!("matched {count} controlled source-table/curve values; no active runtime state");}
