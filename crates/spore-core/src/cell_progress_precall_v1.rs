//! Pre-accounting call gate in retail Cell procedure E7CE40 (SDK x86).
//!
//! Evidence: `target/source-cell/native-cell-progression-add-sdk.asm`,
//! E7CE40..E7CE8C, SporeApp.exe SHA-256
//! 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! This models only whether E79750 is called, not the call's meaning or effects.
//! It observes the state before that call; it must not be assumed unchanged
//! when subsequently entering the integer progression accounting slice.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressPrecallStateV1 {
    pub byte_68: u8,
    pub progress_1c: i32,
    /// Thirteen consecutive raw 32-bit words starting at offset 0x34.
    pub words_34: [u32; 13],
    pub word_7c: u32,
}

/// Whether the observed E7CE40 branch calls E79750.
///
/// The progress comparisons are signed, endpoints included; the byte is
/// tested for zero, and every nonzero word contributes one to the count.
pub fn calls_e79750_v1(state: &ProgressPrecallStateV1) -> bool {
    state.byte_68 == 0
        && (120..=150).contains(&state.progress_1c)
        && state.words_34.iter().filter(|&&word| word != 0).count() < 5
        && state.word_7c != 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ProgressPrecallStateV1 {
        ProgressPrecallStateV1 { byte_68: 0, progress_1c: 120, words_34: [0; 13], word_7c: 0 }
    }

    #[test]
    fn progress_range_is_inclusive_and_signed() {
        for (progress, expected) in [(i32::MIN, false), (119, false), (120, true), (150, true), (151, false), (i32::MAX, false)] {
            let mut s = state();
            s.progress_1c = progress;
            assert_eq!(calls_e79750_v1(&s), expected, "progress {progress}");
        }
    }

    #[test]
    fn four_nonzero_words_pass_and_five_fail_including_last_slot() {
        let mut s = state();
        for i in [0, 3, 7, 12] { s.words_34[i] = u32::MAX; }
        assert!(calls_e79750_v1(&s));
        s.words_34[11] = 1;
        assert!(!calls_e79750_v1(&s));
    }

    #[test]
    fn byte_is_zero_tested_and_word_7c_is_equal_to_two() {
        let mut s = state();
        s.byte_68 = 2;
        assert!(!calls_e79750_v1(&s));
        s.byte_68 = 0;
        s.word_7c = 2;
        assert!(!calls_e79750_v1(&s));
        for raw in [0, 1, 3, u32::MAX] {
            s.word_7c = raw;
            assert!(calls_e79750_v1(&s));
        }
    }
}
