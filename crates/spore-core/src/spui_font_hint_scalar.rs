//! Draft source integer scalar state operations for the original font VM.
//!
//! Source 8ADC20/40 storage, 8AD790 instruction-control bit updates and
//! 8AB710 grid rounding. Deliberately unexported; no compiled acceptance yet.
//! Checked storage addressing is an owned API guard. Source native handlers
//! have no corresponding bounds check. All integer additions wrap as x86 DWORDs.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarError { StorageIndex(i32) }

pub fn read_storage(storage: &[i32], index: i32) -> Result<i32, ScalarError> {
    usize::try_from(index).ok().and_then(|i| storage.get(i).copied())
        .ok_or(ScalarError::StorageIndex(index))
}

pub fn write_storage(storage: &mut [i32], index: i32, value: i32) -> Result<(), ScalarError> {
    let slot = usize::try_from(index).ok().and_then(|i| storage.get_mut(i))
        .ok_or(ScalarError::StorageIndex(index))?;
    *slot = value;
    Ok(())
}

/// 8AD790 pops selector last: only selectors 1/2 act, only in PREP C8 != 0.
/// A selector is a full DWORD comparison, with only that selected bit copied.
pub fn instruction_control(old: u32, value: i32, selector: i32, prep: bool) -> u32 {
    if prep && (selector == 1 || selector == 2) {
        old ^ ((old ^ value as u32) & selector as u32)
    } else { old }
}

/// Exact 8AB710 integer kernel, including wraparound and sign-change clamp.
/// `distance` and `compensation` are the source callback DWORD arguments.
/// No floating-point operation is admitted or substituted.
pub fn round_grid(distance: i32, compensation: i32) -> i32 {
    let rounded = if distance >= 0 {
        distance.wrapping_add(compensation).wrapping_add(32) & !63
    } else {
        (compensation.wrapping_sub(distance).wrapping_add(32) & !63).wrapping_neg()
    };
    if distance != 0 && (rounded ^ distance) < 0 { 0 } else { rounded }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_ties_and_sign_clamp() {
        assert_eq!((-33..=33).map(|x| round_grid(x, 0)).collect::<Vec<_>>(),
            (-33..=33).map(|x: i32| if x.abs() >= 32 { 64 * x.signum() } else { 0 }).collect::<Vec<_>>());
        assert_eq!(round_grid(1, -65), 0);
        assert_eq!(round_grid(-1, -65), 0);
        assert_eq!(round_grid(i32::MIN, 0), i32::MIN);
        assert_eq!(round_grid(i32::MAX, 0), 0);
    }
    #[test]
    fn instruction_control_full_selector_and_selected_bits() {
        assert_eq!(instruction_control(0x12345678, -1, 1, true), 0x12345679);
        assert_eq!(instruction_control(3, 0, 2, true), 1);
        for selector in [0, 3, -1, 65537] {
            assert_eq!(instruction_control(7, 0, selector, true), 7);
        }
        assert_eq!(instruction_control(7, 0, 1, false), 7);
    }
    #[test]
    fn storage_guards_preserve_data() {
        let mut data = [7, 9];
        assert_eq!(write_storage(&mut data, -1, 0), Err(ScalarError::StorageIndex(-1)));
        assert_eq!(read_storage(&data, 2), Err(ScalarError::StorageIndex(2)));
        write_storage(&mut data, 1, -7).unwrap();
        assert_eq!(read_storage(&data, 1), Ok(-7));
        assert_eq!(data[0], 7);
    }
}
