//! Conditional source category prefix→explicit locale table→raw-return bridge.
//! Caller supplies a matching stable table, normal successful service behavior,
//! and output state; no actual provider, live locale or UI is inferred.
use crate::content_manifest::{locale_output_decision, LocaleOutputDecision, LocaleOutputState};
use crate::locale_table::LocaleTable;
use crate::space_locale_prefix_handoff_v1;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome<'a> {
    ProviderUnresolved,
    LookupOrOutputUnresolved,
    ProcessingUnresolved { raw: &'a [u16] },
    Raw(&'a [u16]),
}

/// Pointer-backed type-22 record + selected table; callbacks and lifetime are
/// admitted stable. `6B5100` ignores the `6B4EE0` return, so a missing/empty
/// lookup is unresolved rather than falling back to the record's inline text.
pub fn project<'a>(record:&[u8], table_id:u32, table:&'a LocaleTable,
                   state:Option<LocaleOutputState>) -> Result<Outcome<'a>, &'static str> {
    let prefix=space_locale_prefix_handoff_v1::project(true,0x22,0x9c,record)?;
    let [requested,id]=prefix.receiver_words.ok_or("missing source prefix")?;
    if requested!=table_id {return Ok(Outcome::ProviderUnresolved);}
    let Some(raw)=table.lookup(id).filter(|s|s.first().is_some_and(|v|*v!=0)) else {
        return Ok(Outcome::LookupOrOutputUnresolved);
    };
    match state {
        Some(s) if !s.cached_flag && locale_output_decision(s)==LocaleOutputDecision::Raw => Ok(Outcome::Raw(raw)),
        _ => Ok(Outcome::ProcessingUnresolved{raw}),
    }
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn raw_state_only_and_no_inline_fallback(){
  let table=crate::locale_table::parse(b"0x20012 All\n").unwrap();
  let prefix=[0x45,0x17,0x67,0x21,0x12,0,2,0];
  let state=LocaleOutputState{cached_flag:false,cached_pointer_present:false,classifier_nonzero:false,global_nonzero:false};
  assert_eq!(project(&prefix,0x21671745,&table,Some(state)).unwrap(),Outcome::Raw(&[65,108,108]));
  assert_eq!(project(&prefix,0,&table,Some(state)).unwrap(),Outcome::ProviderUnresolved);
  let empty=crate::locale_table::parse(b"0x20013 Planet\n").unwrap();
  assert_eq!(project(&prefix,0x21671745,&empty,Some(state)).unwrap(),Outcome::LookupOrOutputUnresolved);
  assert!(matches!(project(&prefix,0x21671745,&table,None).unwrap(),Outcome::ProcessingUnresolved{..}));
 }
}
