//! Bounded `661D99..661DFE` presentation handoff.
//!
//! Admits the set local flag, stable scripted wrapper/receiver, normally returning
//! callbacks, a supplied opaque locale pointer and no mutation between calls.
//! The clear-flag projection additionally scripts allocation and construction.
//! Actual locale service, constructor behavior, object lifetime and UI are excluded.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event { GetReceiver, Locale, SetText, SetId, Finish }

/// Ordered callback events; property failure does not inspect the receiver.
pub fn events(property_succeeded: bool) -> &'static [Event] {
    use Event::*;
    if property_succeeded {
        &[GetReceiver, Locale, SetText, GetReceiver, SetId, GetReceiver, Finish]
    } else {
        &[]
    }
}

/// Calls in the clear-flag branch with a scripted allocator and constructor.
/// A successful constructor returns the admitted stable synthetic object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearEvent { Allocate, Locale, Construct, AddRef, SetChild }

pub fn clear_events(allocation_succeeded: bool) -> &'static [ClearEvent] {
    use ClearEvent::*;
    if allocation_succeeded {
        &[Allocate, Locale, Construct, AddRef, SetChild]
    } else {
        &[Allocate, SetChild]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn success_and_failure_paths() {
        assert!(events(false).is_empty());
        assert_eq!(events(true).len(), 7);
        assert_eq!(clear_events(true).len(), 5);
        assert_eq!(clear_events(false), &[ClearEvent::Allocate, ClearEvent::SetChild]);
    }
}
