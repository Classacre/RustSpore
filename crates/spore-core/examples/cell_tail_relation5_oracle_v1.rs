//! Controlled nonzero-E6A250 relation-5 comparison adapter.
use spore_core::{cell_feeding_contact_arbitration_e59b40_v1::ContactOutput,
    cell_feeding_tail_e6a2b0_v1::{evaluate_tail, RelationFiveOutcome, TailInput}};
fn main() {
    let w:Vec<u32>=std::env::args().skip(1).map(|s| {
        assert!(s.len()==8 && s.bytes().all(|b|b.is_ascii_hexdigit()));
        u32::from_str_radix(&s,16).unwrap()
    }).collect();
    assert_eq!(w.len(),21);
    let mut out=ContactOutput{actor_ids:[w[0],w[1]],selector:w[2] as i32,feature_indices:[w[3],w[4]],
        depth_bits:w[5],point_words:[w[6],w[7],w[8]],normal_words:[w[9],w[10],w[11]]};
    let input=TailInput{actor_ids:[w[12],w[13]],point_words:[w[14],w[15],w[16]],normal_words:[w[17],w[18],w[19]],
        depth_bits:w[20],relation:5,word_20:0,relation_five:Some(RelationFiveOutcome::DirectPriority22)};
    evaluate_tail(&mut out,input).unwrap();
    let r=[out.actor_ids[0],out.actor_ids[1],out.selector as u32,out.feature_indices[0],out.feature_indices[1],out.depth_bits,
        out.point_words[0],out.point_words[1],out.point_words[2],out.normal_words[0],out.normal_words[1],out.normal_words[2]];
    println!("{}",r.iter().map(|v|format!("{v:08x}")).collect::<Vec<_>>().join(" "));
}
