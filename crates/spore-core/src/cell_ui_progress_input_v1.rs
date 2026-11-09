//! Explicit snapshot bridge for E5BE20 -> E283A0; no live actor or UI binding.
//! P is the save object read through the Cell global, not the actor object.
use crate::{cell_progression_v1::ProgressStateV1,
    cell_ui_progress_selection_v1::{self, Selection}, native_x87::{Context, Extended, X87Error}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressInput { pub value: f32, pub selection: Selection }
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error { OutOfScope, X87(X87Error), Selection(cell_ui_progress_selection_v1::Error) }
impl From<X87Error> for Error { fn from(e:X87Error)->Self { Self::X87(e) } }
impl From<cell_ui_progress_selection_v1::Error> for Error { fn from(e:cell_ui_progress_selection_v1::Error)->Self { Self::Selection(e) } }

/// Caller supplies the freshly read save snapshot at the UI call boundary.
/// Pointer identity, absence of intervening mutation and lifecycle are caller
/// admissions. Negative source values are outside this selector's safe domain.
pub fn from_snapshot(state: ProgressStateV1, x87: Context) -> Result<ProgressInput,Error> {
    let p=state.progress_1c;
    if !(0..=1000).contains(&p) { return Err(Error::OutOfScope); }
    let value=if p==1000 { 1.0 } else {
        let interval=p/200;
        let low=interval*200;
        let numerator=x87.sub(Extended::from_i32(p),Extended::from_i32(low))?;
        let denominator=x87.add(Extended::from_i32(200),Extended::from_f32(f32::from_bits(0x37800000))?)?;
        let ratio=x87.div(numerator,denominator)?;
        let scale=Extended::from_f32(f32::from_bits(0x3e4ccccd))?;
        let result=match interval {
            0 => x87.mul(ratio,scale)?,
            1 => x87.mul(x87.add(ratio,Extended::from_i32(1))?,scale)?,
            2 => x87.add(x87.mul(ratio,scale)?,Extended::from_f32(f32::from_bits(0x3ecccccd))?)?,
            3 => x87.add(x87.mul(ratio,scale)?,Extended::from_f32(f32::from_bits(0x3f19999a))?)?,
            4 => x87.add(x87.mul(ratio,scale)?,Extended::from_f32(f32::from_bits(0x3f4ccccd))?)?,
            _ => return Err(Error::OutOfScope),
        };
        // E6CAEB stores the returned x87 value before E29810 receives it.
        x87.store_f32(result)?
    };
    Ok(ProgressInput { value, selection:cell_ui_progress_selection_v1::select(value,x87)? })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_progression_v1::account_progress_v1;
    fn state(p:i32)->ProgressStateV1 { ProgressStateV1 { progress_1c:p,counter_20:0,counter_24:0,counter_28:0 } }
    #[test]
    fn source_intervals_and_endpoints_produce_bounded_selection() {
        for cw in [0x027f,0x037f] {
            let ctx=Context::from_control_word(cw).unwrap();
            for p in [0,1,199,200,201,399,400,401,599,600,601,799,800,801,999,1000] {
                let r=from_snapshot(state(p),ctx).unwrap();
                assert!((0.0..=1.0).contains(&r.value) && r.selection.segment<5 && r.selection.bucket<2,"{cw:04X} {p} {r:?}");
            }
            assert_eq!(from_snapshot(state(0),ctx).unwrap().value,0.);
            assert_eq!(from_snapshot(state(1000),ctx).unwrap().value,1.);
        }
    }
    #[test]
    fn caller_can_supply_post_accounting_snapshot_without_assuming_call_order() {
        let ctx=Context::from_control_word(0x037f).unwrap();
        let post=account_progress_v1(state(990),25,1).state;
        assert_eq!(post.progress_1c,1000);
        assert_eq!(from_snapshot(post,ctx).unwrap().selection.segment,4);
        assert_eq!(from_snapshot(state(-1),ctx),Err(Error::OutOfScope));
        assert_eq!(from_snapshot(state(1001),ctx),Err(Error::OutOfScope));
    }
}
