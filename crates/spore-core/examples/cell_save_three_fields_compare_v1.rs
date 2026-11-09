//! Controlled three-field source writer/reader comparison. The sorted 22-entry
//! descriptor table, matching branch and successful synthetic stream are fixture
//! admissions. Reader destinations, other object words and descriptor restoration
//! are checked inside the native host; only the writer frame is emitted.
use spore_core::cell_save_descriptor_order_v1::sorted_fixture_lower_bound;
fn frame(key:[u32;3],food:u32,plant:u32)->[u8;56]{
 let fields=spore_core::cell_save_three_field_codec_v1::Fields{player_key:key,food,plant_food:plant};
 let bytes=spore_core::cell_save_three_field_codec_v1::encode(fields);
 assert_eq!(spore_core::cell_save_three_field_codec_v1::decode(&bytes),Ok(fields));
 bytes[4..].try_into().unwrap()
}
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),304);assert_eq!([sorted_fixture_lower_bound(0xabcd0000),sorted_fixture_lower_bound(0xabcd0001),sorted_fixture_lower_bound(0xabcd0011)],[(3,true),(4,true),(10,true)]);for i in 0..4{let v=(i%2)as u32;let expected=frame([0x11223344,0x55667788,0x99aabbcc],v,v+1000);let r=&b[i*76..(i+1)*76];assert_eq!((w(r,0),w(r,4),w(r,8)),(1,60,4));assert_eq!(&r[12..68],&expected);let fp=w(r,68);assert_eq!(fp&0xffff,if i>=2{0x37f}else{0x27f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,72)&!0x3f,0x1f80);}println!("matched four synthetic three-field frames; reader outputs are host guarded");}
#[cfg(test)]mod tests{use super::*;#[test]fn key_order(){let b=frame([1,2,3],4,5);assert_eq!(&b[36..48],&[0,0,0,3,0,0,0,2,0,0,0,1]);}}
