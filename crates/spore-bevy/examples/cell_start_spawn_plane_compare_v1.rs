//! Compare guarded original first-candidate corners to the existing plane path.
#[path = "../src/cell_effect_plane_grid_v1.rs"]
mod cell_effect_plane_grid_v1;
use spore_core::{cell_effect_placement_prefix_v1::Input,cell_effect_pool_route_v1::Route,cell_effect_tier_v1::{placement_with_helper,TierRow},cell_effect_transform_v1,native_sse::Policy,native_x87::Context};
fn word(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){
 let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),3072);
 let spacing=[10f32,30.,100.,300.,1000.,3000.,10000.,30000.,100000.,300000.,1000000.,3000000.,10000000.,30000000.,100000000.,300000000.].map(f32::to_bits);
 let table:Vec<_>=(0..22).map(|i|TierRow{tier:if i==0{0}else{(((i+1)/2) as u32).min(10)},threshold:if i<2{0}else{(i as i32-1)*50}}).collect();
 let ctx=Context::from_control_word(0x027f).unwrap();let one=1f32.to_bits();let basis=[one,0,0,0,one,0,0,0,one];
 let plane=cell_effect_plane_grid_v1::parse_and_project("3,0,2,2,3f800000").unwrap().map(f32::to_bits);
 for i in 0..8 {let o=i*360;let kind=(i%2) as u32;assert_eq!((word(&b,o),word(&b,o+4),word(&b,o+8)),(4,kind,(i/2) as u32));
  assert_eq!([word(&b,o+16),word(&b,o+20),word(&b,o+24),word(&b,o+28)],[kind,3,2,2]);
  let input=Input{route:Route::PendingNonzeroPlacement,level:3,grid:[2,2],spacing_bits:&spacing,helper_return_bits:0,global_514c_bits:one,effect_present:false,stable_disjoint:true};
  let corners=placement_with_helper(input,0,&table,10,true,Policy::Mxcsr1f80,ctx).unwrap().corner_words;
  for (j,x) in corners.iter().enumerate(){assert_eq!(word(&b,o+336+j*4),*x,"row {i} corner {j}");}
  let t=cell_effect_transform_v1::project(corners,basis,Policy::Mxcsr1f80,true).unwrap();assert_eq!(plane,[t[1],t[2],t[4],t[4]]);
 }
 println!("matched eight original first-candidate corner rows to the existing controlled Bevy plane projection; synthetic state");
}
