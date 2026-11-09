//! Compare the bounded tier model with retained native records; no host is run.
use spore_core::{cell_effect_tier_v1::{helper,placement_with_helper,TierRow},cell_effect_placement_prefix_v1::{Input,Projection},cell_effect_pool_route_v1::Route,native_sse::Policy,native_x87::Context};
fn u(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn main(){let a:Vec<_>=std::env::args().collect();assert_eq!(a.len(),5);let t=std::fs::read(&a[1]).unwrap();let n=std::fs::read(&a[2]).unwrap();let spacing=std::fs::read(&a[3]).unwrap();let placement=std::fs::read(&a[4]).unwrap();assert_eq!((t.len(),n.len(),spacing.len(),placement.len()),(616,4004*12,64,176));
 let rows:Vec<_>=(0..22).map(|i|TierRow{tier:u(&t,i*28),threshold:u(&t,i*28+4) as i32}).collect();let special=rows[21].tier;
 for i in 0..4004{let q=i/1001;let progress=(i%1001)as i32;let level=if q%2==0{3}else{11};let cw=if q<2{0x027f}else{0x037f};let actual=u(&n,i*12);assert_eq!(u(&n,i*12+4)&0xffff,cw);assert_eq!((u(&n,i*12+4)>>16)&0x3840,0);assert_eq!(u(&n,i*12+8)&!0x3f,0x1f80);assert_eq!(helper(progress,level,&rows,special,true).unwrap(),actual,"case {i}");}
 let spacing:[u32;16]=std::array::from_fn(|i|u(&spacing,i*4));let ctx=Context::from_control_word(0x027f).unwrap();
 // Retained placement cases 0 and 2 have scripted helper returns 1 and 0.5.
 // Explicit progress values below produce those returns, while +514C is an
 // independently supplied snapshot; no same-state or native joined call claimed.
 for (case,progress,grid,global,present) in [(0,300,[2,-3],1000f32,false),(2,200,[16777217,-16777217],10000f32,true)]{
  let input=Input{route:Route::PendingNonzeroPlacement,level:3,grid,spacing_bits:&spacing,helper_return_bits:0,global_514c_bits:global.to_bits(),effect_present:present,stable_disjoint:true};let actual=placement_with_helper(input,progress,&rows,special,true,Policy::Mxcsr1f80,ctx).unwrap();let expected=Projection{corner_words:std::array::from_fn(|j|u(&placement,case*44+4+j*4)),pending_existing:u(&placement,case*44)!=0};assert_eq!(actual,expected,"placement {case}");
 }
 println!("matched 4004 retained native helper outputs and 2 separately retained placement records; synthetic stable-state composition");}
