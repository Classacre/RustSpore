//! Cell editor setup property `503D2F60`: ordered source template keys.
//!
//! Native `00574680` selects a 12-byte record using an unresolved helper;
//! `00574710` advances the stored index and copies the next record. This
//! module accepts an explicit valid index; it does not implement RNG, archive
//! precedence, template loading, or part catalog/eligibility semantics.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateKey {
    pub low: u32,
    pub type_id: u32,
    pub scope: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateArrayError {
    MissingHeader,
    UnsupportedStride(u32),
    CountExceedsLimit(u32),
    LengthMismatch { expected: usize, actual: usize },
    Empty,
    IndexOutOfRange { index: usize, count: usize },
}

/// Limit is a defensive reader bound, not a measured native limit.
pub const MAX_TEMPLATE_KEYS: u32 = 1024;

/// Read the raw value of a previously identified PROP array field. The caller
/// must establish the field identity and type. The header is big-endian;
/// each key's low, type and scope words are little-endian.
pub fn parse_template_array(value: &[u8]) -> Result<Vec<TemplateKey>, TemplateArrayError> {
    if value.len() < 8 {
        return Err(TemplateArrayError::MissingHeader);
    }
    let count = u32::from_be_bytes(value[0..4].try_into().unwrap());
    let stride = u32::from_be_bytes(value[4..8].try_into().unwrap());
    if stride != 12 {
        return Err(TemplateArrayError::UnsupportedStride(stride));
    }
    if count > MAX_TEMPLATE_KEYS {
        return Err(TemplateArrayError::CountExceedsLimit(count));
    }
    let expected = 8 + count as usize * 12;
    if value.len() != expected {
        return Err(TemplateArrayError::LengthMismatch { expected, actual: value.len() });
    }
    Ok(value[8..].chunks_exact(12).map(|row| TemplateKey {
        low: u32::from_le_bytes(row[0..4].try_into().unwrap()),
        type_id: u32::from_le_bytes(row[4..8].try_into().unwrap()),
        scope: u32::from_le_bytes(row[8..12].try_into().unwrap()),
    }).collect())
}

/// Advance from a valid stored index, wrapping to zero. This bounded subset
/// matches the native positive-count modulo path; invalid native states and
/// its zero-key fallback are left explicit as errors here.
pub fn next_template(keys: &[TemplateKey], index: usize) -> Result<(usize, TemplateKey), TemplateArrayError> {
    if keys.is_empty() {
        return Err(TemplateArrayError::Empty);
    }
    if index >= keys.len() {
        return Err(TemplateArrayError::IndexOutOfRange { index, count: keys.len() });
    }
    let next = if index == keys.len() - 1 { 0 } else { index + 1 };
    Ok((next, keys[next]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value() -> Vec<u8> {
        let mut raw = vec![0, 0, 0, 2, 0, 0, 0, 12];
        for word in [0x0625a974u32, 0x3d97a8e4, 0x40616200,
                     0x0625a978, 0x3d97a8e4, 0x40616200] {
            raw.extend_from_slice(&word.to_le_bytes());
        }
        raw
    }

    #[test]
    fn parses_mixed_endian_and_cycles_in_source_order() {
        let keys = parse_template_array(&value()).unwrap();
        assert_eq!(keys[0], TemplateKey { low: 0x0625a974, type_id: 0x3d97a8e4, scope: 0x40616200 });
        assert_eq!(next_template(&keys, 0), Ok((1, keys[1])));
        assert_eq!(next_template(&keys, 1), Ok((0, keys[0])));
        assert_eq!(next_template(&keys, 2), Err(TemplateArrayError::IndexOutOfRange { index: 2, count: 2 }));
    }

    #[test]
    fn rejects_bad_header_stride_count_and_exact_extent() {
        assert_eq!(parse_template_array(&[0; 7]), Err(TemplateArrayError::MissingHeader));
        let mut raw = value(); raw[7] = 8;
        assert_eq!(parse_template_array(&raw), Err(TemplateArrayError::UnsupportedStride(8)));
        raw = value(); raw[0..4].copy_from_slice(&1025u32.to_be_bytes());
        assert_eq!(parse_template_array(&raw), Err(TemplateArrayError::CountExceedsLimit(1025)));
        raw = value(); raw.pop();
        assert_eq!(parse_template_array(&raw), Err(TemplateArrayError::LengthMismatch { expected: 32, actual: 31 }));
        raw = value(); raw.push(0);
        assert_eq!(parse_template_array(&raw), Err(TemplateArrayError::LengthMismatch { expected: 32, actual: 33 }));
    }

    #[test]
    fn empty_array_is_valid_framing_but_cannot_select() {
        let keys = parse_template_array(&[0, 0, 0, 0, 0, 0, 0, 12]).unwrap();
        assert_eq!(next_template(&keys, 0), Err(TemplateArrayError::Empty));
        let one = [TemplateKey { low: 1, type_id: 2, scope: 3 }];
        assert_eq!(next_template(&one, 0), Ok((0, one[0])));
    }
}
