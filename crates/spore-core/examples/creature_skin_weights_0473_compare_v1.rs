//! Synthetic endpoint coordinates; conditional pinned query/row arrays only.
//! These records are not observed runtime skin weights or palette identities.
use std::{env,fs,io::{self,Write}};
use spore_core::{creature_generated_skin_mixed_accum_v1::{accumulate_rank_record,Admission,Query},creature_generated_skin_edge_accum_v1::Endpoint,native_sse::Policy,native_x87::{Context,Precision,Rounding}};
fn word(b:&[u8],p:usize)->u32{u32::from_le_bytes(b[p..p+4].try_into().unwrap())}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),4);let f=fs::read(&a[1]).unwrap();let points=fs::read(&a[2]).unwrap();let q=fs::read(&a[3]).unwrap();assert_eq!(f.len(),472);assert_eq!(points.len(),528);assert_eq!(q.len(),23736);
let members:Vec<Vec<u32>>=(0..11).map(|i|{let p=212+i*20;let n=word(&f,p) as usize;assert!((1..=4).contains(&n));(0..n).map(|j|word(&f,p+4+j*4)).collect()}).collect();
let endpoints:Vec<_>=(0..11).map(|i|Endpoint{position:std::array::from_fn(|j|f32::from_bits(word(&f,i*12+j*4))),members:&members[i]}).collect();
let edges:Vec<_>=(0..10).map(|i|[word(&f,132+i*8),word(&f,136+i*8)]).collect();let remap:Vec<_>=(0..10).map(|i|word(&f,432+i*4)).collect();let mut out=Vec::new();
for i in 0..44{let point=std::array::from_fn(|j|f32::from_bits(word(&points,i*12+j*4)));let base=i*516;let n=word(&q,base) as usize;assert!(n<=64);let requests:Vec<_>=(0..n).map(|j|Query{key:word(&q,base+4+j*8),weight:f32::from_bits(word(&q,base+8+j*8))}).collect();let r=accumulate_rank_record(&[0.;10],point,&requests,&edges,&endpoints,Some(&remap),Policy::Mxcsr1f80,Context{precision:Precision::Bits53,rounding:Rounding::NearestEven},Admission).unwrap();for x in r.weights{out.extend(x.to_bits().to_le_bytes())}for c in r.ranked { out.extend(c.index_word.to_le_bytes()); out.extend(c.weight.to_bits().to_le_bytes()); }out.extend(r.record);}
io::stdout().write_all(&out).unwrap();}


