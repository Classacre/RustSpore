//! Bounded E5D8F0 player gate. All player and root values are caller-supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerFlags { pub byte_112: u8, pub byte_113: u8 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome { pub call_before: bool, pub lookup: bool, pub root_5158: u32, pub pending_after_call: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unadmitted;
/// Caller admits reaching E5D8F0, successful return from its preceding helper,
/// stable root and lookup lifetimes. The initial root value is the snapshot after
/// the preceding helper returns; player flags are the subsequent lookup snapshot. The pending
/// after-call does not perform the service action or establish a stage switch.
pub fn gate(initial_root_5158: u32, player: Option<PlayerFlags>, admitted: bool) -> Result<Outcome, Unadmitted> {
    if !admitted { return Err(Unadmitted); }
    let pass = player.is_some_and(|p| p.byte_113 == 0 && p.byte_112 == 0);
    Ok(Outcome { call_before: true, lookup: true, root_5158: if pass { 5 } else { initial_root_5158 }, pending_after_call: pass })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tests_gate_and_preserves_initial_root_on_block() {
        let initial = 0x12345678;
        assert!(!gate(initial, None, true).unwrap().pending_after_call);
        assert_eq!(gate(initial, Some(PlayerFlags { byte_112: 0, byte_113: 0 }), true).unwrap().root_5158, 5);
        for flags in [PlayerFlags { byte_112: 0xff, byte_113: 0 }, PlayerFlags { byte_112: 0, byte_113: 0x80 }] {
            let out = gate(initial, Some(flags), true).unwrap();
            assert_eq!(out.root_5158, initial);
            assert!(!out.pending_after_call);
        }
        assert_eq!(gate(initial, None, false), Err(Unadmitted));
    }
}
