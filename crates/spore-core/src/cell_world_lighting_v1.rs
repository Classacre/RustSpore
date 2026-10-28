//! Saved CellGame lighting inputs from one explicitly selected package.
//! These raw values are not draw-ready shader uniforms.
use std::collections::BTreeSet;
use crate::prop::{self, types, PropValue, ARRAY_MARKER_1, ARRAY_MARKER_2};

#[derive(Debug)]
pub struct CellWorldLightingPreset {
    pub coefficients: [[f32; 3]; 25],
    pub exposure: f32,
}

pub fn parse(bytes: &[u8]) -> Result<CellWorldLightingPreset, String> {
    let declared_bytes = bytes.get(..4).ok_or("missing PROP count")?;
    let declared = u32::from_be_bytes(declared_bytes.try_into().map_err(|_| "PROP count")?) as usize;
    let tree = prop::parse(bytes).map_err(|e| e.to_string())?;
    if !tree.exact || tree.consumed != bytes.len() || tree.items.len() != declared
        || tree.items.iter().any(|item| item.anonymous)
        || tree.items.iter().map(|item| item.id).collect::<BTreeSet<_>>().len() != declared
    {
        return Err("expected a complete, uniquely named PROP".into());
    }
    let sh = tree.items.iter().find(|item| item.id == 0x0100_EAC5)
        .ok_or("missing SH coefficients")?;
    if sh.prop_type != types::CRGB || !matches!(sh.specifier, ARRAY_MARKER_1 | ARRAY_MARKER_2)
        || sh.values.len() != 25
    {
        return Err("expected 25 RGB coefficients".into());
    }
    let mut coefficients = [[0.0; 3]; 25];
    for (out, value) in coefficients.iter_mut().zip(&sh.values) {
        match value {
            PropValue::ColorRGB(r, g, b) if r.is_finite() && g.is_finite() && b.is_finite() => {
                *out = [*r, *g, *b];
            }
            _ => return Err("non-finite or invalid SH coefficient".into()),
        }
    }
    let exposure = tree.items.iter().find(|item| item.id == 0x0100_EAC4)
        .ok_or("missing exposure")?;
    if exposure.prop_type != types::FLOAT || matches!(exposure.specifier, ARRAY_MARKER_1 | ARRAY_MARKER_2)
        || exposure.values.len() != 1
    {
        return Err("expected one FLOAT exposure".into());
    }
    match &exposure.values[0] {
        PropValue::Float(value) if value.is_finite() => Ok(CellWorldLightingPreset { coefficients, exposure: *value }),
        _ => Err("non-finite or invalid exposure".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_rgb_array_that_overlaps_the_following_property() {
        let bytes = [
            0, 0, 0, 2, 0x01, 0x00, 0xea, 0xc5, 0, 0x32, 0, 0x9c,
            0, 0, 0, 25, 0, 0, 0, 0, 0x01, 0x00, 0xea, 0xc4,
            0, 0x0d, 0, 0, 0x3e, 0x99, 0x99, 0x9a,
        ];
        assert!(parse(&bytes).is_err());
    }
}
