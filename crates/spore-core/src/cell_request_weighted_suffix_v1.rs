//! Bounded model of the weighted-selection suffix at E6D160..E6D1DF.
//!
//! The caller supplies rows already admitted in source order and the binary32
//! draw already committed by the preceding x87 path. This module does not
//! model admission, fallback, the RNG or its state, actor state, or binding.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmittedRow {
    pub key: u32,
    pub weight_bits: u32,
    pub first_byte: u8,
    pub second_byte: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecondByte {
    Written(u8),
    Unchanged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub key: u32,
    pub first_byte: u8,
    pub second_byte: SecondByte,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedArchitecture,
    UnsupportedMxcsr,
    InvalidDraw,
    InvalidWeight,
}

/// Uses the caller's MXCSR, admitting only 1F80 controls: all exceptions
/// masked, round-to-nearest-even, DAZ and FTZ disabled. Existing status bits
/// are allowed; validation and arithmetic may set status bits, which are not
/// modeled or claimed to match native execution. Inputs must be finite and
/// nonnegative; either signed zero and subnormal values are admitted. Under
/// these restrictions subtraction cannot produce a nonfinite value, so the
/// source COMISS unordered case is unreachable. The input slice is validated
/// in full even if an earlier row would be selected; this is an API policy.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn select(rows: &[AdmittedRow], draw_bits: u32) -> Result<Selection, Error> {
    if !std::is_x86_feature_detected!("sse") {
        return Err(Error::UnsupportedArchitecture);
    }
    let mut csr = 0u32;
    // SAFETY: SSE was detected; the pointer refers to a writable u32.
    unsafe { std::arch::asm!("stmxcsr [{p}]", p = in(reg) &mut csr, options(nostack, preserves_flags)); }
    if csr & !0x3f != 0x1f80 {
        return Err(Error::UnsupportedMxcsr);
    }
    let draw = f32::from_bits(draw_bits);
    if !draw.is_finite() || draw < 0.0 {
        return Err(Error::InvalidDraw);
    }
    if rows.iter().any(|row| {
        let weight = f32::from_bits(row.weight_bits);
        !weight.is_finite() || weight < 0.0
    }) {
        return Err(Error::InvalidWeight);
    }
    // SAFETY: SSE was detected and the controls and operands were admitted.
    Ok(unsafe { select_inner(rows, draw) })
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn select(_: &[AdmittedRow], _: u32) -> Result<Selection, Error> {
    Err(Error::UnsupportedArchitecture)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse")]
unsafe fn select_inner(rows: &[AdmittedRow], draw: f32) -> Selection {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut remaining = _mm_set_ss(draw);
    for row in rows {
        remaining = _mm_sub_ss(remaining, _mm_set_ss(f32::from_bits(row.weight_bits)));
        // E6D190 compares zero to the remainder and JAE selects on >=.
        if _mm_comige_ss(_mm_setzero_ps(), remaining) != 0 {
            return Selection {
                key: row.key,
                first_byte: row.first_byte,
                second_byte: SecondByte::Written(row.second_byte),
            };
        }
    }
    Selection { key: 0, first_byte: 0, second_byte: SecondByte::Unchanged }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(key: u32, weight: f32) -> AdmittedRow {
        AdmittedRow { key, weight_bits: weight.to_bits(), first_byte: key as u8, second_byte: 17 }
    }

    #[test]
    fn subtraction_order_and_boundaries_select_expected_rows() {
        let rows = [row(1, 5.0), row(2, 1.0), row(3, 5.0)];
        assert_eq!(select(&rows, 5.0f32.to_bits()).unwrap().key, 1);
        assert_eq!(select(&rows, f32::from_bits(5.0f32.to_bits() + 1).to_bits()).unwrap().key, 2);
        assert_eq!(select(&rows, 11.0f32.to_bits()).unwrap().key, 3);
    }

    #[test]
    fn no_selection_preserves_second_byte_and_zero_key_can_be_selected() {
        let none = select(&[row(1, 1.0)], 2.0f32.to_bits()).unwrap();
        assert_eq!(none, Selection { key: 0, first_byte: 0, second_byte: SecondByte::Unchanged });
        let selected = select(&[row(0, 1.0)], 0.0f32.to_bits()).unwrap();
        assert_eq!(selected.second_byte, SecondByte::Written(17));
        assert_eq!(select(&[], 0).unwrap().second_byte, SecondByte::Unchanged);
    }

    #[test]
    fn signed_zero_and_subnormal_operands_are_admitted() {
        assert_eq!(select(&[row(4, 0.0)], (-0.0f32).to_bits()).unwrap().key, 4);
        let tiny = f32::from_bits(1);
        assert_eq!(select(&[row(5, tiny), row(6, tiny)], 2).unwrap().key, 6);
    }

    #[test]
    fn rejects_unadmitted_inputs_even_after_an_early_match() {
        assert_eq!(select(&[], f32::INFINITY.to_bits()), Err(Error::InvalidDraw));
        assert_eq!(select(&[row(1, 1.0), row(2, f32::NAN)], 0), Err(Error::InvalidWeight));
        assert_eq!(select(&[row(1, -1.0)], 0), Err(Error::InvalidWeight));
    }
}
