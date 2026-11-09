//! Observed stores in the shared constructor and selected Creature actor methods.
//!
//! Static evidence: SporeApp.exe SHA-256
//! 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a;
//! target/rea/20261028-source-runtime-v6/{80,119,269,270,276,277,292}-analyze_function.json.
//! The actor type is supported by the registry/factory-to-constructor chain.
//! This is a partial memory-write model: method invocation and native floating-
//! point evaluation remain external observations, not a gameplay state machine.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatureActorClass {
    Animal,
    Citizen,
}

/// Two 32-bit slots from an actor receiver rooted at the shared constructor ECX.
/// These are retained as words to avoid changing NaN payloads or inventing
/// native x87 rounding. They are not asserted to be current and maximum health.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureActorFieldWords {
    pub class: CreatureActorClass,
    pub offset_5d4: u32,
    pub offset_5e0: u32,
}

impl CreatureActorFieldWords {
    /// C1F940 stores its externally evaluated and binary32-rounded return at
    /// both offsets. The caller must obtain these bits through native-equivalent
    /// evaluation; raw property words are not always equivalent after x87.
    pub fn after_constructor(class: CreatureActorClass, rounded_word: u32) -> Self {
        Self { class, offset_5d4: rounded_word, offset_5e0: rounded_word }
    }

    /// CBCAA0 stores its 32-bit argument at subobject +0x38. This method models
    /// only the call with receiver equal to whole-object +0x5A8 observed in
    /// C222B0; it does not compute the argument or decide when to call it.
    pub fn apply_observed_subobject_store(&mut self, argument_word: u32) {
        self.offset_5e0 = argument_word;
    }

    /// C080F0 writes +0.0 to +0x5E0 on its animal receiver. Other effects of
    /// C080F0 are deliberately omitted; None means the actor class is excluded
    /// from this bounded method, not that its runtime can never reach it.
    pub fn apply_animal_zero_store(&mut self) -> Option<()> {
        if self.class != CreatureActorClass::Animal {
            return None;
        }
        self.offset_5e0 = 0;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_preserves_the_provided_bits_in_both_slots() {
        let actor = CreatureActorFieldWords::after_constructor(CreatureActorClass::Citizen, 0x7fc0_1234);
        assert_eq!((actor.offset_5d4, actor.offset_5e0), (0x7fc0_1234, 0x7fc0_1234));
    }

    #[test]
    fn stores_touch_only_the_observed_slot() {
        let mut actor = CreatureActorFieldWords::after_constructor(CreatureActorClass::Animal, 0x42c8_0000);
        actor.apply_observed_subobject_store(0x8000_0000);
        assert_eq!(actor.offset_5e0, 0x8000_0000);
        assert_eq!(actor.apply_animal_zero_store(), Some(()));
        assert_eq!((actor.offset_5d4, actor.offset_5e0), (0x42c8_0000, 0));
    }

    #[test]
    fn animal_only_operation_is_not_inferred_for_citizens() {
        let mut actor = CreatureActorFieldWords::after_constructor(CreatureActorClass::Citizen, 7);
        assert_eq!(actor.apply_animal_zero_store(), None);
        assert_eq!(actor.offset_5e0, 7);
    }
}
