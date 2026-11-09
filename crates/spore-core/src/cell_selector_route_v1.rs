//! E73E10 route predicate over explicitly admitted actor words only.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct ActorWords{pub word_248:u32,pub word_fc:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Route{BothFeatures,SecondFeatures,FirstFeatures,NeitherFeatures}
/// Presence is the conjunction of nonzero raw words, not a proof of pointer
/// validity or feature ownership. Route callees and contact are not executed.
pub fn route(first:ActorWords,second:ActorWords)->Route{let a=first.word_248!=0&&first.word_fc!=0;let b=second.word_248!=0&&second.word_fc!=0;match(a,b){(true,true)=>Route::BothFeatures,(false,true)=>Route::SecondFeatures,(true,false)=>Route::FirstFeatures,(false,false)=>Route::NeitherFeatures}}
#[cfg(test)]mod tests{use super::*;#[test]fn forward_and_reverse_preserve_order_and_raw_nonzero(){let absent=ActorWords{word_248:1,word_fc:0};let present=ActorWords{word_248:0x80000000,word_fc:u32::MAX};assert_eq!(route(absent,present),Route::SecondFeatures);assert_eq!(route(present,absent),Route::FirstFeatures);assert_eq!(route(present,present),Route::BothFeatures);assert_eq!(route(absent,absent),Route::NeitherFeatures);}}
