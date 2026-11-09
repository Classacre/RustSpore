//! Bounded projection of the CDAFEF -> C8DE50 selector and null-provider path.
//!
//! Earlier caller state and source lifetimes are supplied; this does not select
//! a live Tribe object or resource. The consumer's provider returns null and
//! both caller-owned output vectors have enough disjoint writable capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub provider_key: Option<u32>,
    pub appended_selector: Option<u32>,
    pub appended_position_bits: Option<[u32; 3]>,
}

/// The input bits must be finite, normal binary32 values or zero on the two
/// dispatch paths, with x87 control 027F and MXCSR 1F80. The provider receiver,
/// callback and vectors must remain valid and non-mutating through the call.
pub fn project(record_word: u32, position_bits: [u32; 3]) -> Result<Outcome, &'static str> {
    let (selector, key) = match record_word {
        1 => (0, 0xa100_1c58),
        2 => (1, 0xbcb8_a199),
        _ => return Ok(Outcome { provider_key: None, appended_selector: None, appended_position_bits: None }),
    };
    if position_bits.iter().any(|&v| {
        let exponent = (v >> 23) & 0xff;
        exponent == 0xff || (exponent == 0 && v & 0x7f_ffff != 0)
    }) { return Err("position must be finite normal or zero"); }
    Ok(Outcome { provider_key: Some(key), appended_selector: Some(selector), appended_position_bits: Some(position_bits) })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_and_skipped_branch() {
        let p = [1.0f32.to_bits(), (-2.0f32).to_bits(), (-0.0f32).to_bits()];
        assert_eq!(project(1,p).unwrap().provider_key, Some(0xa100_1c58));
        assert_eq!(project(2,p).unwrap().appended_position_bits, Some(p));
        assert_eq!(project(3,[u32::MAX;3]).unwrap().provider_key, None);
        assert!(project(1,[1,0,0]).is_err());
    }
}
