//! Conditional E6F641..E6F662 selected-actor target copy.
//!
//! This is a checked, caller-admitted pool snapshot, not mouse handling or a
//! movement update. The caller must bind the selected handle to this pool and
//! supply the actual player and state owners. Raw handles are pool-local.
use crate::cell_actor_pool_checked_v1::CheckedActorPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerTarget {
    /// Actor words +08/+0C/+10, retained as bits (no float arithmetic).
    pub words: [u32; 3],
    /// Actor byte +04.
    pub byte_04: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetResult { Copied, SelectionCleared }

/// Resolve state +C4 through the supplied checked pool. On success, copy the
/// selected actor's +4C/+50/+54 words then clear the player byte; on failure,
/// clear +C4 and leave player fields alone. The accessor must return those
/// exact position words from the resolved object. Checked pool behavior and
/// owner identity are admissions, not a reproduction of native pointer faults.
pub fn update_target<T>(selected_c4: &mut u32, player: &mut PlayerTarget,
    pool: &CheckedActorPool<T>, position_words: impl FnOnce(&T) -> [u32; 3]) -> TargetResult {
    match pool.get(*selected_c4) {
        Some(actor) => {
            player.words = position_words(actor);
            player.byte_04 = 0;
            TargetResult::Copied
        }
        None => { *selected_c4 = 0; TargetResult::SelectionCleared }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copies_exact_words_and_keeps_selected_handle() {
        let mut pool = CheckedActorPool::new(2, 0).unwrap();
        let words = [0x8000_0000, 0x7fc0_1234, 0x3f80_0000];
        let mut selected = pool.allocate(words).unwrap();
        let original = selected;
        let mut player = PlayerTarget { words: [1, 2, 3], byte_04: 7 };
        assert_eq!(update_target(&mut selected, &mut player, &pool, |v| *v), TargetResult::Copied);
        assert_eq!(selected, original);
        assert_eq!(player, PlayerTarget { words, byte_04: 0 });
    }
    #[test]
    fn stale_or_missing_selection_clears_state_without_changing_player() {
        let mut pool = CheckedActorPool::new(1, 0).unwrap();
        let mut selected = pool.allocate([4, 5, 6]).unwrap();
        pool.release(selected).unwrap();
        let replacement = pool.allocate([7, 8, 9]).unwrap();
        assert_ne!(selected, replacement);
        let original = PlayerTarget { words: [1, 2, 3], byte_04: 9 };
        let mut player = original;
        assert_eq!(update_target(&mut selected, &mut player, &pool, |v| *v), TargetResult::SelectionCleared);
        assert_eq!(selected, 0);
        assert_eq!(player, original);
        assert_eq!(update_target(&mut selected, &mut player, &pool, |v| *v), TargetResult::SelectionCleared);
        assert_eq!(player, original);
    }
}
