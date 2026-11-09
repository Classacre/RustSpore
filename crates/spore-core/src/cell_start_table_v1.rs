//! Controlled E802EF..E803CE start-table and scalar projection.
//! Caller supplies source words, stable nonaliasing memory, and masked MXCSR.
//! No source player, runtime table ownership, or callback scheduling is implied.
use crate::native_sse::{self, Policy, SseError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row { pub value_bits: u32, pub threshold: i32 }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selected { Row(usize), Special1000 }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Output { pub selected: Selected, pub global_514c_bits: u32, pub division_bits: u32, pub actor_argument_bits: u32 }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { MissingRow, MissingThreshold, NonFinite, ZeroDivisor, Sse(SseError) }
impl From<SseError> for Error { fn from(e:SseError)->Self { Self::Sse(e) } }

/// `rows[n].threshold` is the word at base + n*28 + 4. The scan's
/// first test uses row 2's threshold while selecting row 1. `special_bits`
/// is the float at 01484920 (special base +10), supplied separately.
/// `numerator_bits` is the already-produced XMM1 input for DIVSS.
/// The caller admits stable row contents and nonaliasing of the intervening
/// +4C store with +514C; this is stronger than native behavior. Results and
/// intermediates must be finite and the divisor nonzero. MXCSR sticky flags
/// are restored by the scoped policy and are not reported as source effects.
pub fn evaluate(progress:i32, rows:&[Row], special_bits:u32, numerator_bits:u32, policy:Policy)->Result<Output,Error> {
    let (selected,bits)=if progress==1000 {(Selected::Special1000,special_bits)} else {
        let mut index=0usize;
        if progress>=0 { loop {
            index=index.checked_add(1).ok_or(Error::MissingThreshold)?;
            let threshold=rows.get(index.checked_add(1).ok_or(Error::MissingThreshold)?).ok_or(Error::MissingThreshold)?.threshold;
            if progress<threshold { break; }
        }}
        (Selected::Row(index),rows.get(index).ok_or(Error::MissingRow)?.value_bits)
    };
    let value=f32::from_bits(bits); let numerator=f32::from_bits(numerator_bits);
    if !value.is_finite() || !numerator.is_finite() { return Err(Error::NonFinite); }
    native_sse::scoped(policy,|| {
        let global=std::hint::black_box(value)*std::hint::black_box(0.5f32);
        let divisor=std::hint::black_box(global)*std::hint::black_box(2.0f32);
        if !global.is_finite() || !divisor.is_finite() {return Err(Error::NonFinite)}
        if divisor==0.0 {return Err(Error::ZeroDivisor)}
        let division=std::hint::black_box(numerator)/std::hint::black_box(divisor);
        let argument=std::hint::black_box(global)*std::hint::black_box(2.0f32);
        if !division.is_finite() || !argument.is_finite() {return Err(Error::NonFinite)}
        Ok(Output{selected,global_514c_bits:global.to_bits(),division_bits:division.to_bits(),actor_argument_bits:argument.to_bits()})
    })
}
#[cfg(test)] mod tests {
 use super::*;
 fn rows()->Vec<Row>{vec![Row{value_bits:8f32.to_bits(),threshold:-99},Row{value_bits:2f32.to_bits(),threshold:-98},Row{value_bits:4f32.to_bits(),threshold:10},Row{value_bits:6f32.to_bits(),threshold:20},Row{value_bits:8f32.to_bits(),threshold:30}]}
 fn run(p:i32,r:&[Row])->Result<Output,Error>{evaluate(p,r,16f32.to_bits(),12f32.to_bits(),Policy::Mxcsr1f80)}
 #[test] fn scan_uses_next_row_threshold_and_strict_boundary(){let r=rows(); assert_eq!(run(-1,&r).unwrap().selected,Selected::Row(0)); assert_eq!(run(0,&r).unwrap().selected,Selected::Row(1)); assert_eq!(run(9,&r).unwrap().selected,Selected::Row(1)); assert_eq!(run(10,&r).unwrap().selected,Selected::Row(2)); assert_eq!(run(20,&r).unwrap().selected,Selected::Row(3)); assert_eq!(run(30,&r),Err(Error::MissingThreshold));}
 #[test] fn special_path_does_not_need_rows(){let o=run(1000,&[]).unwrap();assert_eq!(o.selected,Selected::Special1000);assert_eq!(o.global_514c_bits,8f32.to_bits());assert_eq!(o.division_bits,0.75f32.to_bits());assert_eq!(o.actor_argument_bits,16f32.to_bits());}
 #[test] fn finite_and_zero_boundaries_fail_closed(){let mut r=rows();r[1].value_bits=0;r[2].threshold=10;assert_eq!(run(0,&r),Err(Error::ZeroDivisor));r[1].value_bits=f32::NAN.to_bits();assert_eq!(run(0,&r),Err(Error::NonFinite));assert_eq!(evaluate(1000,&[],1f32.to_bits(),f32::INFINITY.to_bits(),Policy::Mxcsr1f80),Err(Error::NonFinite));}
 #[test] fn subnormal_rounding_and_division_overflow_are_rejected(){
  assert_eq!(evaluate(1000,&[],1,1f32.to_bits(),Policy::Mxcsr1f80),Err(Error::ZeroDivisor));
  assert_eq!(evaluate(1000,&[],f32::MIN_POSITIVE.to_bits(),f32::MAX.to_bits(),Policy::Mxcsr1f80),Err(Error::NonFinite));
 }
 #[test] fn negative_finite_values_and_special_precedence(){
  let o=evaluate(1000,&[],(-4f32).to_bits(),8f32.to_bits(),Policy::Mxcsr1f80).unwrap();
  assert_eq!(o.global_514c_bits,(-2f32).to_bits());assert_eq!(o.division_bits,(-2f32).to_bits());assert_eq!(o.actor_argument_bits,(-4f32).to_bits());
  assert_eq!(run(-1,&[]),Err(Error::MissingRow));assert_eq!(run(0,&rows()[..2]),Err(Error::MissingThreshold));
 }

}
