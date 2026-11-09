//! Compare retained native E50080 output from a caller-supplied TSV, no host execution.
use spore_core::cell_effect_curve_v1::{evaluate,Curve};
use spore_core::native_x87::Context;
fn main(){
 let path=std::env::args().nth(1).expect("expected controlled native TSV");
 let text=std::fs::read_to_string(path).expect("read TSV");let mut count=0;
 for line in text.lines(){let p:Vec<_>=line.split_whitespace().collect();assert_eq!(p.len(),4);let word=|s:&str|u32::from_str_radix(s,16).unwrap();
 let curve=match p[0]{"near"=>Curve::Near,"far"=>Curve::Far,_=>panic!("bad curve")};let x=f32::from_bits(word(p[1]));let ctx=Context::from_control_word(word(p[2]) as u16).unwrap();let want=word(p[3]);let got=evaluate(curve,x,ctx).unwrap().to_bits();assert_eq!(got,want,"{line}");count+=1;}
 assert_eq!(count,92);println!("matched {count} retained native outputs; caller-provided input and no live state");
}
