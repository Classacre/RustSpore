//! Bounded base Window state write and draw-tree eligibility.
//! Object offsets refer to the allocation base, not its +4 interface pointer.
//! Actual construction, property application, callbacks, attachment and
//! mutation must be established by the caller; no live visibility is inferred.
use crate::spui_std_drawable::select_slot;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct WindowSnapshot { pub authored_2c:u32, pub private_30:u8, pub state_ac:u32 }
impl WindowSnapshot {
    /// Relevant fields directly initialized by 9629F0; later writes are separate.
    pub fn constructor_fields() -> Self { Self { authored_2c:3, private_30:0, state_ac:0 } }
    /// The raw 95A690 eligibility branch, not a full tree or final draw result.
    pub fn tree_eligible(self) -> bool {
        self.private_30 & 4 == 0 && (self.private_30 & 1 != 0 || self.authored_2c & 1 != 0)
    }
    /// A snapshot's +AC state and explicitly resolved slot pointers only.
    /// Does not assert the window was visited or the drawable was invoked.
    pub fn selected_slot(self, resolved:[bool;8]) -> Option<usize> { select_slot(self.state_ac,resolved) }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct StateWrite { pub old:u32, pub new:u32, pub event_13_pending:bool }
/// 95FE90 receives the Window +4 interface: interface +A8 is base +AC.
/// Returns the immediate write and pending event only. The virtual callback and
/// conditional invalidation may mutate state; reacquire a snapshot after them.
pub fn write_state_before_callbacks(old:u32,new:u32) -> StateWrite {
    StateWrite { old, new, event_13_pending:old!=new }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn constructor_and_tree_gate() {
        let mut w=WindowSnapshot::constructor_fields();assert_eq!(w.state_ac,0);assert!(w.tree_eligible());
        w.authored_2c=2;assert!(!w.tree_eligible());w.private_30=1;assert!(w.tree_eligible());w.private_30=5;assert!(!w.tree_eligible());
    }
    #[test] fn changed_write_is_only_a_pending_callback_boundary() {
        assert_eq!(write_state_before_callbacks(7,7).event_13_pending,false);
        assert_eq!(write_state_before_callbacks(0,9),StateWrite {old:0,new:9,event_13_pending:true});
        let fresh=WindowSnapshot {authored_2c:1,private_30:0,state_ac:9};
        assert_eq!(fresh.selected_slot([true,false,false,false,false,true,false,false]),Some(5));
        assert_eq!(fresh.selected_slot([false;8]),None);
    }
}
