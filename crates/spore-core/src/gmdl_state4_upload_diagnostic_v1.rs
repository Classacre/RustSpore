//! Controlled state-4 upload argument arithmetic for the +8 == 0 branch.
//!
//! This computes relative offsets, not absolute pointer additions or their
//! mapping/lifetime, and does not call a device or prove descriptor-to-mesh
//! selection, source pose, runtime state, or animation.
//! Native x86 arithmetic wraps; this checked fixture rejects overflow instead.
use crate::gmdl_skin::GmdlPalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { UnsupportedMode, ArithmeticOverflow, PaletteExtent }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UploadArguments {
    /// Offset from the explicitly admitted original matrix base, in bytes.
    pub source_offset_bytes: u32,
    /// Rows passed to the receiver, three per descriptor second word.
    pub row_count: u32,
    /// Explicit caller-supplied stack argument plus three times supplied start.
    pub first_row: u32,
}

/// `descriptor` must come from the caller's admitted direct-draw selection;
/// neither that association nor the supplied state is authenticated here.
/// `palette_len_bytes` bounds a stable caller-owned synthetic matrix buffer.
/// Reject overflow and unsupported modes rather than emulating wrap or native
/// unchecked reads. Zero count requires only that the start offset be in bounds.
pub fn compute(descriptor: GmdlPalette, payload_start: u32, payload_mode: u32,
    caller_base: u32, palette_len_bytes: u32) -> Result<UploadArguments, Error> {
    if payload_mode != 0 { return Err(Error::UnsupportedMode); }
    let descriptor_offset = descriptor.first_word.checked_mul(48).ok_or(Error::ArithmeticOverflow)?;
    let start_offset = payload_start.checked_mul(48).ok_or(Error::ArithmeticOverflow)?;
    let source_offset_bytes = descriptor_offset.checked_add(start_offset).ok_or(Error::ArithmeticOverflow)?;
    let source_length = descriptor.bone_count.checked_mul(48).ok_or(Error::ArithmeticOverflow)?;
    let end = source_offset_bytes.checked_add(source_length).ok_or(Error::ArithmeticOverflow)?;
    let row_count = descriptor.bone_count.checked_mul(3).ok_or(Error::ArithmeticOverflow)?;
    let first_row = caller_base.checked_add(payload_start.checked_mul(3).ok_or(Error::ArithmeticOverflow)?)
        .ok_or(Error::ArithmeticOverflow)?;
    // Ensure the full row range itself is representable even though the source
    // passes first row and count separately to the device.
    first_row.checked_add(row_count).ok_or(Error::ArithmeticOverflow)?;
    if end > palette_len_bytes { return Err(Error::PaletteExtent); }
    Ok(UploadArguments { source_offset_bytes, row_count, first_row })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_jawhead_descriptor_values_and_nonzero_start() {
        // Values copied from the separately checked Jawhead census, not loaded
        // or authenticated by this test.
        let a = GmdlPalette { first_word: 0, bone_count: 64 };
        let b = GmdlPalette { first_word: 64, bone_count: 19 };
        assert_eq!(compute(a, 2, 0, 7, 85 * 48), Ok(UploadArguments { source_offset_bytes: 96, row_count: 192, first_row: 13 }));
        assert_eq!(compute(b, 2, 0, 7, 85 * 48), Ok(UploadArguments { source_offset_bytes: 3168, row_count: 57, first_row: 13 }));
        assert_eq!(compute(b, 2, 0, 7, 85 * 48 - 1), Err(Error::PaletteExtent));
    }
    #[test]
    fn unsupported_modes_and_checked_integer_boundaries() {
        let p = GmdlPalette { first_word: 0, bone_count: 1 };
        assert_eq!(compute(p, 0, 1, 0, 48), Err(Error::UnsupportedMode));
        assert_eq!(compute(p, 0, u32::MAX, 0, 48), Err(Error::UnsupportedMode));
        assert_eq!(compute(GmdlPalette { first_word: u32::MAX, bone_count: 0 }, 0, 0, 0, u32::MAX), Err(Error::ArithmeticOverflow));
        assert_eq!(compute(p, u32::MAX, 0, 0, u32::MAX), Err(Error::ArithmeticOverflow));
        assert_eq!(compute(p, 0, 0, u32::MAX, 48), Err(Error::ArithmeticOverflow));
        assert_eq!(compute(p, 0, 0, 0, 47), Err(Error::PaletteExtent));
    }
}
