//! Bounded E7E160 callback and selector-1 dispatch boundary, version 2.
//!
//! Static evidence: ev_a66d55179cd6b40f2611da8f3a031053ff7b39284a7b155d1d236a67d268ff58,
//! `target/source-cell/feeding-event-dispatch-linear-v1.json`, and
//! `re/cell-feeding-event-dispatch-v1.md`. This is an adapter for an already
//! admitted live record, not a scheduler, event producer, or gameplay model.
//! The caller must guarantee record lifetime across the callback, and perform
//! actor resolution and player checks against fresh state downstream.

/// Admitted finite normal binary32 values or signed zero. Subnormals,
/// infinities, and NaNs are excluded; no all-bit-pattern fidelity is claimed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdmittedFloat(f32);

impl AdmittedFloat {
    pub fn new(value: f32) -> Option<Self> {
        if value.is_normal() || value == 0.0 { Some(Self(value)) } else { None }
    }

    pub fn get(self) -> f32 { self.0 }
}

/// Caller guarantees masked x87 exceptions and a usable x87 stack/control
/// environment at entry and after normal callback return; the callback
/// preserves that environment and the incoming float stack slot. This admits
/// value forwarding for the restricted domain, not floating status or exception
/// behavior. The adapter neither observes nor reproduces x87 status flags.
#[derive(Debug, Clone, Copy)]
pub struct FloatEnvironmentAdmission;

/// Only fields read by this bounded boundary. IDs and handles are opaque words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventRecord {
    pub callback_present: bool,
    pub gate_08: u8,
    pub selector_24: u32,
    pub actor_28: u32,
    pub word_38: u32,
    pub definition_3c: u32,
}

/// Arguments forwarded to E7D830, after any callback has returned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectorOne {
    pub actor: u32,
    pub raw_word: u32,
    pub definition: u32,
    pub gate: u8,
    pub value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DispatchBoundary {
    SelectorOne(SelectorOne),
    /// Other selectors are deliberately left to the unmodeled dispatch table.
    OtherSelector(u32),
}

/// Replays only the observed conditional callback and selector-1 forwarding.
/// `third_argument` is tested by its low byte, as in the x86 `test bl, bl`.
/// Callback mutations are retained, except that +08 is set to one after return.
/// Its return value is intentionally ignored. The callback may also mutate
/// external world state; no snapshot of that state is taken here. The caller
/// must also admit that the incoming float stack slot is unchanged by aliasing
/// during the callback, because the native code reloads it after the call.
pub fn dispatch_boundary<F, R>(
    record: &mut EventRecord,
    value: AdmittedFloat,
    third_argument: u32,
    _float_environment: FloatEnvironmentAdmission,
    callback: F,
) -> DispatchBoundary
where
    F: FnOnce(&mut EventRecord, f32) -> R,
{
    let value = value.get();
    if record.gate_08 == 0 && record.callback_present && third_argument as u8 == 0 {
        let _ = callback(record, value);
        record.gate_08 = 1;
    }
    if record.selector_24 == 1 {
        DispatchBoundary::SelectorOne(SelectorOne {
            actor: record.actor_28,
            raw_word: record.word_38,
            definition: record.definition_3c,
            gate: record.gate_08,
            value,
        })
    } else {
        DispatchBoundary::OtherSelector(record.selector_24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> EventRecord {
        EventRecord { callback_present: true, gate_08: 0, selector_24: 1,
            actor_28: 10, word_38: 2, definition_3c: 20 }
    }

    #[test]
    fn callback_changes_are_read_after_return_and_gate_is_forced_to_one() {
        let mut event = event();
        let result = dispatch_boundary(&mut event, AdmittedFloat::new(0.5).unwrap(), 0x100, FloatEnvironmentAdmission, |r, _| {
            r.actor_28 = 11; r.word_38 = 3; r.definition_3c = 21; r.gate_08 = 7;
            123
        });
        assert_eq!(result, DispatchBoundary::SelectorOne(SelectorOne {
            actor: 11, raw_word: 3, definition: 21, gate: 1, value: 0.5,
        }));
    }

    #[test]
    fn callback_can_redirect_selector() {
        let mut event = event();
        assert_eq!(dispatch_boundary(&mut event, AdmittedFloat::new(0.0).unwrap(), 0, FloatEnvironmentAdmission, |r, _| r.selector_24 = 18),
            DispatchBoundary::OtherSelector(18));
    }

    #[test]
    fn every_callback_gate_prevents_call_without_changing_gate() {
        for (gate, callback_present, third) in [(1, true, 0), (0, false, 0), (0, true, 1)] {
            let mut event = event(); event.gate_08 = gate; event.callback_present = callback_present;
            let result = dispatch_boundary(&mut event, AdmittedFloat::new(1.0).unwrap(), third, FloatEnvironmentAdmission, |_, _| panic!("unexpected callback"));
            assert_eq!(event.gate_08, gate);
            assert!(matches!(result, DispatchBoundary::SelectorOne(SelectorOne { gate: g, .. }) if g == gate));
        }
    }

    #[test]
    fn float_domain_rejects_non_normal_nonzero_values() {
        for value in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN, f32::from_bits(1), f32::from_bits(0x80000001)] {
            assert!(AdmittedFloat::new(value).is_none());
        }
        for value in [0.0_f32, -0.0, f32::MIN_POSITIVE, -f32::MIN_POSITIVE, f32::MAX] {
            assert_eq!(AdmittedFloat::new(value).unwrap().get().to_bits(), value.to_bits());
        }
    }

    #[test]
    fn callback_observes_initial_values_and_can_select_one() {
        let mut event = event(); event.selector_24 = 0;
        let result = dispatch_boundary(&mut event, AdmittedFloat::new(-0.0).unwrap(), 0,
            FloatEnvironmentAdmission, |r, value| {
                assert_eq!(r.gate_08, 0);
                assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
                r.selector_24 = 1;
            });
        assert!(matches!(result, DispatchBoundary::SelectorOne(SelectorOne { gate: 1, .. })));
    }

    #[test]
    fn noncanonical_gates_and_other_selector_boundaries() {
        for (gate, third, selector) in [(0x80, 0, 0), (0, 0xff, u32::MAX)] {
            let mut event = event(); event.gate_08 = gate; event.selector_24 = selector;
            assert_eq!(dispatch_boundary(&mut event, AdmittedFloat::new(1.0).unwrap(), third,
                FloatEnvironmentAdmission, |_, _| panic!("unexpected callback")),
                DispatchBoundary::OtherSelector(selector));
            assert_eq!(event.gate_08, gate);
        }
    }

}
