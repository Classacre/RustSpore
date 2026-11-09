//! Checked E5E100 existing-object virtual-call handoff.
//! Caller supplies fixture identities and admits stable, disjoint storage; no
//! native pointer is dereferenced and no callback or live effect is executed.
use crate::{cell_effect_placement_prefix_v1::Projection as Placement, cell_effect_pool_route_v1::Route};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { UnstableOrAliased, WrongRoute, AbsentObject, InvalidIdentity, InvalidWeight }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingCall {
    /// Nonzero synthetic token for the admitted object, not a native pointer.
    pub receiver: u64,
    /// Nonzero synthetic token for the caller-owned weight local, not an address.
    pub weight_local: u64,
    pub weight_bits: u32,
    pub first_argument: u32,
    pub last_argument: u32,
}
/// The same nonzero object slot must remain valid from the placement branch
/// through the source's re-read and virtual-table lookup. The weight-local
/// token names the same stable local read by the pending method; the call may
/// mutate arbitrary state, so this projection makes no post-call claim.
/// Finite nonzero weight and stable, disjoint lifetimes are checked admissions.
pub fn project(route: Route, placement: Placement, receiver: u64, weight_local: u64, weight_bits: u32, stable_disjoint: bool) -> Result<PendingCall, Error> {
    if !stable_disjoint { return Err(Error::UnstableOrAliased); }
    if route != Route::PendingNonzeroPlacement { return Err(Error::WrongRoute); }
    if !placement.pending_existing { return Err(Error::AbsentObject); }
    if receiver == 0 || weight_local == 0 { return Err(Error::InvalidIdentity); }
    if !f32::from_bits(weight_bits).is_finite() || weight_bits & 0x7fff_ffff == 0 { return Err(Error::InvalidWeight); }
    Ok(PendingCall { receiver, weight_local, weight_bits, first_argument: 4, last_argument: 1 })
}
#[cfg(test)] mod tests {
 use super::*;
 fn existing() -> Placement { Placement { corner_words:[0;6], pending_existing:true } }
 #[test] fn captures_only_pending_call_with_explicit_local(){ let p=project(Route::PendingNonzeroPlacement,existing(),11,22,0x3f40009f,true).unwrap(); assert_eq!((p.receiver,p.weight_local,p.weight_bits,p.first_argument,p.last_argument),(11,22,0x3f40009f,4,1)); }
 #[test] fn rejects_unavailable_or_unstable_inputs(){ let r=Route::PendingNonzeroPlacement; assert_eq!(project(r,existing(),1,2,1,false),Err(Error::UnstableOrAliased)); assert_eq!(project(Route::NoMatch,existing(),1,2,1,true),Err(Error::WrongRoute)); let mut absent=existing(); absent.pending_existing=false; assert_eq!(project(r,absent,1,2,1,true),Err(Error::AbsentObject)); for (a,b) in [(0,2),(1,0)] { assert_eq!(project(r,existing(),a,b,1,true),Err(Error::InvalidIdentity)); } for w in [0,0x80000000,0x7f800000,0x7fc00000] { assert_eq!(project(r,existing(),1,2,w,true),Err(Error::InvalidWeight)); } }
}
