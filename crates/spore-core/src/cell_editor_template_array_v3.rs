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
    InexactSetup,
    MissingTemplateProperty,
    DuplicateTemplateProperty,
    UnsupportedPropertyShape,
    InvalidKeyValue,
    RawSetupMismatch,
    MissingHeader,
    UnsupportedStride(u32),
    CountExceedsLimit(u32),
    LengthMismatch { expected: usize, actual: usize },
    Empty,
    SelectionCountExceedsLimit(usize),
    IndexOutOfRange { index: usize, count: usize },
}

/// Limit is a defensive reader bound, not a measured native limit.
pub const MAX_TEMPLATE_KEYS: u32 = 1024;
pub const TEMPLATE_PROPERTY_ID: u32 = 0x503D_2F60;

/// Extract this array from an already parsed Cell Editor Setup PROP.
/// The caller must identify the original raw value of this same field; its
/// count, stride, extent and keys are validated against the parsed values.
/// Package selection and the initial-index helper remain caller duties.
pub fn templates_from_setup(tree: &crate::prop::PropTree, raw_value: &[u8]) -> Result<Vec<TemplateKey>, TemplateArrayError> {
    if !tree.exact { return Err(TemplateArrayError::InexactSetup); }
    let mut fields = tree.items.iter().filter(|item| !item.anonymous && item.id == TEMPLATE_PROPERTY_ID);
    let field = fields.next().ok_or(TemplateArrayError::MissingTemplateProperty)?;
    if fields.next().is_some() { return Err(TemplateArrayError::DuplicateTemplateProperty); }
    if field.prop_type != crate::prop::types::KEY || !matches!(field.specifier, crate::prop::ARRAY_MARKER_1 | crate::prop::ARRAY_MARKER_2) {
        return Err(TemplateArrayError::UnsupportedPropertyShape);
    }
    if field.values.len() > MAX_TEMPLATE_KEYS as usize {
        return Err(TemplateArrayError::SelectionCountExceedsLimit(field.values.len()));
    }
    let typed: Vec<_> = field.values.iter().map(|value| match value {
        crate::prop::PropValue::Key(low, type_id, scope) => Ok(TemplateKey { low: *low, type_id: *type_id, scope: *scope }),
        _ => Err(TemplateArrayError::InvalidKeyValue),
    }).collect::<Result<_, _>>()?;
    if parse_template_array(raw_value)? != typed { return Err(TemplateArrayError::RawSetupMismatch); }
    Ok(typed)
}

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
/// matches the native positive-count modulo path within the parser limit; invalid native states and
/// its zero-key fallback are left explicit as errors here.
pub fn next_template(keys: &[TemplateKey], index: usize) -> Result<(usize, TemplateKey), TemplateArrayError> {
    if keys.is_empty() {
        return Err(TemplateArrayError::Empty);
    }
    if keys.len() > MAX_TEMPLATE_KEYS as usize {
        return Err(TemplateArrayError::SelectionCountExceedsLimit(keys.len()));
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
        let too_many = vec![one[0]; MAX_TEMPLATE_KEYS as usize + 1];
        assert_eq!(next_template(&too_many, 0), Err(TemplateArrayError::SelectionCountExceedsLimit(1025)));
    }

    #[test]
    fn setup_lookup_requires_one_exact_key_array() {
        use crate::prop::{PropItem, PropTree, PropValue};
        let item = PropItem { id: TEMPLATE_PROPERTY_ID, anonymous: false, prop_type: crate::prop::types::KEY,
            specifier: crate::prop::ARRAY_MARKER_1, values: vec![PropValue::Key(7, 8, 9)] };
        let mut tree = PropTree { items: vec![item.clone()], exact: true, consumed: 0 };
        let mut raw = vec![0, 0, 0, 1, 0, 0, 0, 12];
        for word in [7u32, 8, 9] { raw.extend_from_slice(&word.to_le_bytes()); }
        assert_eq!(templates_from_setup(&tree, &raw), Ok(vec![TemplateKey { low: 7, type_id: 8, scope: 9 }]));
        let mut wrong_stride = raw.clone(); wrong_stride[7] = 8;
        assert_eq!(templates_from_setup(&tree, &wrong_stride), Err(TemplateArrayError::UnsupportedStride(8)));
        let mut wrong_key = raw.clone(); wrong_key[8] = 3;
        assert_eq!(templates_from_setup(&tree, &wrong_key), Err(TemplateArrayError::RawSetupMismatch));
        tree.items.push(item);
        assert_eq!(templates_from_setup(&tree, &raw), Err(TemplateArrayError::DuplicateTemplateProperty));
        tree.exact = false;
        assert_eq!(templates_from_setup(&tree, &raw), Err(TemplateArrayError::InexactSetup));
    }
}
