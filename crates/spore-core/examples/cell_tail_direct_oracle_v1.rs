//! Hex-word adapter for a controlled direct-tail comparison.
use spore_core::{cell_feeding_contact_arbitration_e59b40_v1::ContactOutput,
    cell_feeding_tail_e6a2b0_v1::{evaluate_tail, TailInput}};
fn main() {
    let words: Vec<u32> = std::env::args().skip(1).map(|s| {
        assert!(s.len()==8 && s.bytes().all(|b|b.is_ascii_hexdigit()));
        u32::from_str_radix(&s,16).unwrap()
    }).collect();
    assert_eq!(words.len(),21);
    let mut out=ContactOutput { actor_ids:[words[0],words[1]],selector:words[2] as i32,
        feature_indices:[words[3],words[4]],depth_bits:words[5],
        point_words:[words[6],words[7],words[8]],normal_words:[words[9],words[10],words[11]] };
    let input=TailInput { actor_ids:[words[12],words[13]],point_words:[words[14],words[15],words[16]],
        normal_words:[words[17],words[18],words[19]],depth_bits:words[20],relation:0,word_20:0,relation_five:None };
    evaluate_tail(&mut out,input).unwrap();
    let result=[out.actor_ids[0],out.actor_ids[1],out.selector as u32,out.feature_indices[0],out.feature_indices[1],out.depth_bits,
        out.point_words[0],out.point_words[1],out.point_words[2],out.normal_words[0],out.normal_words[1],out.normal_words[2]];
    println!("{}",result.iter().map(|v|format!("{v:08x}")).collect::<Vec<_>>().join(" "));
}
