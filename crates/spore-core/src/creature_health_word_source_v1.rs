//! Bounded raw-word selection observed at SporeApp C1F940 -> BCE6B0.
//!
//! Evidence: target/rea/20261028-source-runtime-v6/{80,81}-analyze_function.json;
//! executable SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! The caller performs both lookups and supplies the runtime global word. This
//! module does not resolve a property, establish a resource, execute the x87
//! load, or interpret the selected bits as an active Creature health value.

/// The externally observed outcome of the two successive lookups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthLookupOutcome {
    NamespaceMissing,
    PropertyMissing,
    PropertyFound { type_code: u32, selected_word: u32 },
}

/// A selected word and the observed branch that produced it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthWordSelection {
    CallerFallback(u32),
    VariantWord(u32),
    RuntimeGlobalWord(u32),
}

/// Raw IEEE-754 binary32 bits of the caller fallback, before the x87 load.
pub const CALLER_FALLBACK_WORD: u32 = 0x42c8_0000;

/// Select raw bits only. The global's runtime contents must be supplied by the
/// caller; a zero in the static executable image is not evidence of its value.
pub fn select_health_word(
    outcome: HealthLookupOutcome,
    wrong_type_runtime_global: u32,
) -> HealthWordSelection {
    match outcome {
        HealthLookupOutcome::NamespaceMissing | HealthLookupOutcome::PropertyMissing => {
            HealthWordSelection::CallerFallback(CALLER_FALLBACK_WORD)
        }
        HealthLookupOutcome::PropertyFound { type_code: 0x0d | 0x10, selected_word } => {
            HealthWordSelection::VariantWord(selected_word)
        }
        HealthLookupOutcome::PropertyFound { .. } => {
            HealthWordSelection::RuntimeGlobalWord(wrong_type_runtime_global)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_failures_select_the_caller_word() {
        for outcome in [HealthLookupOutcome::NamespaceMissing, HealthLookupOutcome::PropertyMissing] {
            assert_eq!(select_health_word(outcome, 7), HealthWordSelection::CallerFallback(0x42c8_0000));
        }
    }

    #[test]
    fn variant_types_select_raw_bits_without_float_conversion() {
        for type_code in [0x0d, 0x10] {
            let outcome = HealthLookupOutcome::PropertyFound { type_code, selected_word: 0x7fc0_1234 };
            assert_eq!(select_health_word(outcome, 7), HealthWordSelection::VariantWord(0x7fc0_1234));
        }
    }

    #[test]
    fn other_types_select_supplied_runtime_global() {
        let outcome = HealthLookupOutcome::PropertyFound { type_code: 9, selected_word: 1 };
        assert_eq!(select_health_word(outcome, 0x8000_0000), HealthWordSelection::RuntimeGlobalWord(0x8000_0000));
    }
}
