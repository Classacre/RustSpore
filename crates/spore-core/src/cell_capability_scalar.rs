//! Scalar/flag/speed-mask tail of original Cell cache builder `E51EC0`.
//!
//! Counts, globals, the mutable curve identifier and property-curve lookup are
//! explicit inputs. This does not load packages, bind actors, or presume the
//! retail entry's floating-point state. Scalar SSE requires MXCSR control1F80.

use crate::{
    cell_capability_curve::CellCapabilityCurveError,
    native_sse::{self, Policy, SseError},
    native_x87::Context,
};
use std::fmt;

/// The measured static initializer for the mutable source global `016B4C64`.
/// A live caller must still supply its observed identifier explicitly.
pub const INITIAL_ELECTRIC_RECHARGE_CURVE: u32 = 0x5a38_c4ba;

#[derive(Debug, Clone, Copy)]
pub struct CellCapabilityScalarContext {
    pub x87: Context,
    pub sse: Policy,
}

#[derive(Debug, Clone, Copy)]
pub struct CellCapabilityScalarGlobals {
    /// Native CellGlobals field `74`.
    pub cilia_speed_as_jet: f32,
    /// Native CellGlobals field `78`.
    pub flagella_speed_as_jet: f32,
    /// Native CellGlobals field `7C`, multiplied by literal positive zero.
    pub flagella_speed_as_cilia: f32,
    /// Native CellGlobals field `80`.
    pub cilia_speed_as_flagella: f32,
    /// Native CellGlobals field `90`, read only for an empty propulsion mask.
    pub no_part_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellCapabilityCurveQuery {
    pub identifier: u32,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct CellCapabilityScalars {
    pub flag_1248: bool,
    pub value_124c: f32,
    pub flag_1250: bool,
    pub flag_1251: bool,
    pub propulsion_mask_1254: u32,
    pub value_1258: f32,
    pub value_125c: f32,
    pub value_1260: f32,
    pub value_1264: f32,
    pub value_1268: f32,
    pub integer_126c: i32,
    pub value_1270: f32,
    queries: Vec<CellCapabilityCurveQuery>,
}
impl CellCapabilityScalars {
    pub fn queries(&self) -> &[CellCapabilityCurveQuery] {
        &self.queries
    }

    /// Write exactly the native scalar fields in cache `1204..128F`.
    /// Three bytes after1248, two after1251, geometry/counts and all fields
    /// at1274 or later retain the caller's original bytes.
    pub fn write_native_tail(&self, tail: &mut [u8; 140]) {
        tail[0x1248 - 0x1204] = u8::from(self.flag_1248);
        tail[0x1250 - 0x1204] = u8::from(self.flag_1250);
        tail[0x1251 - 0x1204] = u8::from(self.flag_1251);
        for (offset, bits) in [
            (0x124c, self.value_124c.to_bits()),
            (0x1254, self.propulsion_mask_1254),
            (0x1258, self.value_1258.to_bits()),
            (0x125c, self.value_125c.to_bits()),
            (0x1260, self.value_1260.to_bits()),
            (0x1264, self.value_1264.to_bits()),
            (0x1268, self.value_1268.to_bits()),
            (0x126c, self.integer_126c as u32),
            (0x1270, self.value_1270.to_bits()),
        ] {
            tail[offset - 0x1204..offset - 0x1204 + 4].copy_from_slice(&bits.to_le_bytes());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellCapabilityScalarError {
    ClassCountOutsideNativePartBound,
    NonFiniteInput,
    NonFiniteIntermediate,
    IntegerConversionOutsideFiniteDomain,
    Curve(CellCapabilityCurveError),
    Sse(SseError),
}
impl From<CellCapabilityCurveError> for CellCapabilityScalarError {
    fn from(value: CellCapabilityCurveError) -> Self {
        Self::Curve(value)
    }
}
impl From<SseError> for CellCapabilityScalarError {
    fn from(value: SseError) -> Self {
        Self::Sse(value)
    }
}
impl fmt::Display for CellCapabilityScalarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native Cell capability scalars: {self:?}")
    }
}
impl std::error::Error for CellCapabilityScalarError {}

fn input(value: f32) -> Result<f32, CellCapabilityScalarError> {
    value
        .is_finite()
        .then_some(value)
        .ok_or(CellCapabilityScalarError::NonFiniteInput)
}
fn intermediate(value: f32) -> Result<f32, CellCapabilityScalarError> {
    value
        .is_finite()
        .then_some(value)
        .ok_or(CellCapabilityScalarError::NonFiniteIntermediate)
}

/// Follow `E524A4..E52726` with resolved original counts and curve lookup.
///
/// Native first64 geometry yields counters in0..128, exactly representable by
/// CVTSI2SS. Each callback is an original E837F0 lookup: it receives the literal
/// identifier, binary32 query and explicit x87 policy, then returns the original
/// curve's committed binary32 result. Missing source properties are resolved by
/// the callback, not by an invented scalar fallback. Only consumed global
/// fields must be finite. Raw nonfinite native results and CVTSS2SI indefinite
/// values are outside this checked finite-domain API.
pub fn derive_cell_capability_scalars<F>(
    counts: &[u32; 13],
    globals: CellCapabilityScalarGlobals,
    electric_recharge_curve: u32,
    context: CellCapabilityScalarContext,
    curve: F,
) -> Result<CellCapabilityScalars, CellCapabilityScalarError>
where
    F: FnMut(u32, f32, Context) -> Result<f32, CellCapabilityCurveError>,
{
    native_sse::scoped(context.sse, || {
        derive_scoped(counts, globals, electric_recharge_curve, context.x87, curve)
    })
}

fn derive_scoped<F>(
    counts: &[u32; 13],
    globals: CellCapabilityScalarGlobals,
    electric_recharge_curve: u32,
    context: Context,
    mut curve: F,
) -> Result<CellCapabilityScalars, CellCapabilityScalarError>
where
    F: FnMut(u32, f32, Context) -> Result<f32, CellCapabilityCurveError>,
{
    if counts.iter().any(|count| *count > 128) {
        return Err(CellCapabilityScalarError::ClassCountOutsideNativePartBound);
    }
    let mut queries = Vec::with_capacity(7);
    let mut sample = |identifier, value| {
        let value = intermediate(value)?;
        queries.push(CellCapabilityCurveQuery { identifier, value });
        intermediate(curve(identifier, value, context)?)
    };
    let value_124c = sample(0x7754_0c34, counts[11] as f32)?;
    let mut speed = if counts[10] != 0 {
        let flagella = intermediate(counts[9] as f32 * input(globals.flagella_speed_as_jet)?)?;
        let cilia = intermediate(counts[8] as f32 * input(globals.cilia_speed_as_jet)?)?;
        let mixed = intermediate(flagella + cilia)?;
        let query = intermediate(mixed + counts[10] as f32)?;
        sample(0x2c57_4746, query)?
    } else if counts[9] != 0 {
        let cilia = intermediate(counts[8] as f32 * input(globals.cilia_speed_as_flagella)?)?;
        let query = intermediate(cilia + counts[9] as f32)?;
        sample(0xbbab_1bf5, query)?
    } else if counts[8] != 0 {
        let unused_flagella = intermediate(input(globals.flagella_speed_as_cilia)? * 0.)?;
        let query = intermediate(unused_flagella + counts[8] as f32)?;
        sample(0x2ef6_8471, query)?
    } else {
        0.
    };
    let mut mask = (u32::from(counts[10] > 0) << 2)
        | (u32::from(counts[8] > 0) << 1)
        | u32::from(counts[9] > 0);
    if mask == 0 {
        mask = 1;
        speed = input(globals.no_part_speed)?;
    }
    let value_1260 = sample(0xe501_5d0c, counts[6] as f32)?;
    let value_125c = sample(electric_recharge_curve, counts[6] as f32)?;
    let value_1268 = sample(0x03ab_589e, counts[6] as f32)?;
    let conversion = sample(0xaee2_835e, counts[6] as f32)?.round_ties_even();
    if !(-2_147_483_648.0..2_147_483_648.0).contains(&conversion) {
        return Err(CellCapabilityScalarError::IntegerConversionOutsideFiniteDomain);
    }
    let value_1270 = sample(0x92ba_ae19, counts[5] as f32)?;
    Ok(CellCapabilityScalars {
        flag_1248: counts[7] > 0,
        value_124c,
        flag_1250: counts[2] > 0 || counts[4] > 0,
        flag_1251: counts[3] > 0 || counts[4] > 0,
        propulsion_mask_1254: mask,
        value_1258: speed,
        value_125c,
        value_1260,
        value_1264: 0.,
        value_1268,
        integer_126c: conversion as i32,
        value_1270,
        queries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn context() -> CellCapabilityScalarContext {
        CellCapabilityScalarContext {
            x87: Context {
                precision: Precision::Bits24,
                rounding: Rounding::Up,
            },
            sse: Policy::Mxcsr1f80,
        }
    }
    #[test]
    fn zero_mask_reads_only_default_and_retains_unwritten_cache_bytes() {
        let globals = CellCapabilityScalarGlobals {
            cilia_speed_as_jet: f32::NAN,
            flagella_speed_as_jet: f32::NAN,
            flagella_speed_as_cilia: f32::NAN,
            cilia_speed_as_flagella: f32::NAN,
            no_part_speed: -0.,
        };
        let value = derive_cell_capability_scalars(
            &[0; 13],
            globals,
            INITIAL_ELECTRIC_RECHARGE_CURVE,
            context(),
            |_, _, _| Ok(0.),
        )
        .unwrap();
        assert_eq!(value.propulsion_mask_1254, 1);
        assert_eq!(value.value_1258.to_bits(), 0x8000_0000);
        assert_eq!(value.queries().len(), 6);
        let mut tail = [0xa5; 140];
        value.write_native_tail(&mut tail);
        for range in [0..68, 69..72, 78..80, 112..140] {
            assert!(tail[range].iter().all(|byte| *byte == 0xa5));
        }
    }
    #[test]
    fn mixed_propulsion_uses_literal_query_order_and_nearest_integer_commit() {
        let mut counts = [0; 13];
        for (class, count) in [(4, 1), (5, 7), (6, 3), (7, 2), (8, 3), (9, 2), (10, 1)] {
            counts[class] = count;
        }
        let globals = CellCapabilityScalarGlobals {
            cilia_speed_as_jet: 0.5,
            flagella_speed_as_jet: 0.25,
            flagella_speed_as_cilia: f32::NAN,
            cilia_speed_as_flagella: f32::NAN,
            no_part_speed: f32::NAN,
        };
        let value = derive_cell_capability_scalars(
            &counts,
            globals,
            INITIAL_ELECTRIC_RECHARGE_CURVE,
            context(),
            |id, query, _| Ok(if id == 0xaee2_835e { 2.5 } else { query }),
        )
        .unwrap();
        assert!(value.flag_1248 && value.flag_1250 && value.flag_1251);
        assert_eq!(value.propulsion_mask_1254, 7);
        assert_eq!(value.value_1258, 3.);
        assert_eq!(value.integer_126c, 2);
        assert_eq!(value.queries()[1].identifier, 0x2c57_4746);
        assert_eq!(value.queries()[1].value, 3.);
        assert_eq!(value.value_1270, 7.);
    }
}
