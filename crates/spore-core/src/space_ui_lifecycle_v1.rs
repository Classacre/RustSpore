//! Fixed controlled source lifecycle: successful constructor and stable object.
//! Widget retain is supplied; nonfreeing callbacks and fixed capacities [18,8,2]
//! are admissions. No real allocator, destructor lifetime or live UI claim.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Free { LocaleBuffer, TooltipBuffer, Object }
#[derive(Debug,PartialEq,Eq)]
pub struct ResultRow { pub before_source_release:u32, pub after_source_release:u32, pub after_scripted_cleanup:u32, pub frees: [Free;3] }
pub fn lifecycle(widget_retains: bool)->ResultRow {
 ResultRow { before_source_release:1+u32::from(widget_retains), after_source_release:u32::from(widget_retains), after_scripted_cleanup:0, frees:[Free::LocaleBuffer,Free::TooltipBuffer,Free::Object] }
}
#[cfg(test)] mod tests { use super::*; #[test] fn fixed_controls(){ assert_eq!(lifecycle(false).after_source_release,0); assert_eq!(lifecycle(true).after_source_release,1); } }
