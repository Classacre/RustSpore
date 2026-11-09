//! Checked direct-name B1DF30 branches with synthetic service admission.
//! Generic string construction and actual service methods remain pending.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Name{CellFromEditor,CreatureFromEditor,TribeFromEditor,CivilisationFromEditor,SpaceFromEditor,LoadGame}
impl Name{pub fn source_literal(self)->u32{match self{Self::CellFromEditor=>0x145e58c,Self::CreatureFromEditor=>0x145e570,Self::TribeFromEditor=>0x145e554,Self::CivilisationFromEditor=>0x145e534,Self::SpaceFromEditor=>0x145e518,Self::LoadGame=>0x145e5a4}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unstable,Unsupported}
/// Stable service/vtable and callback lifetime are caller admissions. The
/// returned enum identifies a source literal, not an executed service call.
/// Unsupported mode combinations include the dynamic string fallback.
pub fn direct_name(old:u32,new:u32,stable:bool)->Result<Name,Error>{if !stable{return Err(Error::Unstable)}if new==0x1654c08{return Ok(Name::LoadGame)}if old!=0x1654c06{return Err(Error::Unsupported)}match new{0x1654c00=>Ok(Name::CellFromEditor),0x1654c01=>Ok(Name::CreatureFromEditor),0x1654c02=>Ok(Name::TribeFromEditor),0x1654c04=>Ok(Name::CivilisationFromEditor),0x1654c05=>Ok(Name::SpaceFromEditor),_=>Err(Error::Unsupported)}}
#[cfg(test)]mod tests{use super::*;#[test]fn direct_and_unhandled(){assert_eq!(direct_name(0x1654c06,0x1654c01,true),Ok(Name::CreatureFromEditor));assert_eq!(direct_name(0,0x1654c08,true),Ok(Name::LoadGame));assert_eq!(direct_name(0x1654c00,0x1654c01,true),Err(Error::Unsupported));assert_eq!(direct_name(0x1654c06,0x1654c03,true),Err(Error::Unsupported));assert_eq!(direct_name(0,0x1654c08,false),Err(Error::Unstable));}}
