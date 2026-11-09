//! Bounded cUITimeline E45AF5 sender suffix; caller supplies synthetic observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HelperResult { pub accepted: bool, pub flag_after: u8 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route { EqualObjectCut, BusyCut, NoSend, PendingSend }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome { pub route: Route, pub helper_called: bool, pub pending_hide: bool, pub pending_send: bool, pub flag_at_cut: u8 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Unadmitted, MissingHelper }
/// The caller admits reaching this suffix and stable receiver/service lifetimes.
/// For the helper path, `flag_after` is a fresh post-helper observation; callbacks
/// are represented as pending, not executed or delivered; flag_at_cut is before them. Equal and busy routes
/// are cut before their unrelated native actions.
pub fn sender(equal:bool,busy:u8,flag:u8,helper:Option<HelperResult>,admitted:bool)->Result<Outcome,Error> {
 if !admitted{return Err(Error::Unadmitted)}
 let result=|route,helper_called,pending_hide,pending_send,flag_at_cut| Outcome{route,helper_called,pending_hide,pending_send,flag_at_cut};
 if equal{return Ok(result(Route::EqualObjectCut,false,false,false,flag))}
 if busy!=0{return Ok(result(Route::BusyCut,false,false,false,flag))}
 if flag==0{return Ok(result(Route::NoSend,false,false,false,flag))}
 let h=helper.ok_or(Error::MissingHelper)?;
 if !h.accepted{return Ok(result(Route::NoSend,true,false,false,h.flag_after))}
 Ok(result(Route::PendingSend,true,true,true,0))
}
#[cfg(test)]mod tests{use super::*;
 #[test] fn boundary_and_mutating_helper(){
  assert_eq!(sender(false,0,0,None,true).unwrap().route,Route::NoSend);
  assert_eq!(sender(false,0,1,None,true),Err(Error::MissingHelper));
  let h=Some(HelperResult{accepted:false,flag_after:7});assert_eq!(sender(false,0,1,h,true).unwrap().flag_at_cut,7);
  let h=Some(HelperResult{accepted:true,flag_after:127});let x=sender(false,0,1,h,true).unwrap();assert_eq!(x.flag_at_cut,0);assert!(x.pending_hide&&x.pending_send);
  assert_eq!(sender(false,0,1,h,false),Err(Error::Unadmitted));
 }
}
