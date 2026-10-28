//! Bounded transcription of SporeApp.exe 761D60 and 4584E0.
//!
//! Evidence: `re/cell-screen-effect-render-trace-v1.md`, REA dossiers 815
//! and 818, executable SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! This models only successful-allocation vertex contents and the color helper.
//! It does not model the allocator, state changes, submission, shader, blending,
//! texture, viewport lookup, or the providers of the two offset globals.
//!
//! The caller supplies the arithmetic implementation. Matching the original
//! requires x86 scalar SSE operand ordering, MXCSR rounding, denormal handling,
//! exceptions and NaN behavior; this module does not provide or establish those.
//! Width and height are the observed u16 fields, not verified viewport semantics.

use crate::native_sse::{self, Policy, SseError};

/// Scalar operations with SSE destination and source operand ordering.
/// Implementations must provide the desired floating-point environment.
pub trait ScalarOps {
    fn maxss(&mut self, dst: f32, src: f32) -> f32;
    fn minss(&mut self, dst: f32, src: f32) -> f32;
    fn mulss(&mut self, dst: f32, src: f32) -> f32;
    fn divss(&mut self, dst: f32, src: f32) -> f32;
    fn subss(&mut self, dst: f32, src: f32) -> f32;
    fn cvtss2si(&mut self, src: f32) -> i32;
}

/// Four 24-byte source records: position, packed word, then UV.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub packed_color: u32,
    pub uv: [f32; 2],
}

/// Transcribe 4584E0: each result's low byte occupies bits 16, 8, 0, 24.
/// Color-space meaning and the active MXCSR are not established.
pub fn pack_color<O: ScalarOps>(ops: &mut O, components: [f32; 4]) -> u32 {
    let mut word = 0u32;
    for (component, shift) in components.into_iter().zip([16, 8, 0, 24]) {
        let lower = ops.maxss(0.0, component);
        let scaled = ops.mulss(lower, 255.0);
        let upper = ops.minss(scaled, 255.0);
        let integer = ops.cvtss2si(upper);
        word |= ((integer as u32) & 0xff) << shift;
    }
    word
}

/// Transcribe the writes after 6DD7B0 succeeds, using caller-supplied globals.
/// Zero dimensions and non-finite inputs are intentionally passed to `ops`;
/// source exception behavior depends on its SSE environment. This does not
/// assert that allocation succeeds or that 6DD800 submits a particular primitive.
pub fn successful_quad<O: ScalarOps>(
    ops: &mut O,
    width: u16,
    height: u16,
    offset_x: f32,
    offset_y: f32,
    packed_color: u32,
) -> [Vertex; 4] {
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    std::array::from_fn(|i| {
        // The source repeats each multiply and divide for each vertex.
        let x2 = ops.mulss(offset_x, 2.0);
        let x_ratio = ops.divss(x2, width as f32);
        let x = ops.subss(corners[i].0, x_ratio);
        let y2 = ops.mulss(offset_y, 2.0);
        let y_ratio = ops.divss(y2, height as f32);
        let z = ops.subss(corners[i].1, y_ratio);
        Vertex { position: [x, 0.0, z], packed_color, uv: uv[i] }
    })
}

// Private: instances can only be used inside the SSE2-checked scope below.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct ScopedSse;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn binary_sse(a: f32, b: f32, opcode: u8) -> f32 {
    let (a, b) = (a.to_bits(), b.to_bits());
    let result: u32;
    // SAFETY: called only within native_sse::scoped, which checks SSE2 and
    // installs the explicitly selected, masked-exception MXCSR policy.
    unsafe {
        macro_rules! binary {
            ($op:literal) => {
                std::arch::asm!("movd {x}, {a:e}", "movd {y}, {b:e}", $op,
                    "movd {result:e}, {x}",
                    x = out(xmm_reg) _, y = out(xmm_reg) _,
                    a = in(reg) a, b = in(reg) b, result = lateout(reg) result,
                    options(nostack, preserves_flags))
            };
        }
        match opcode {
            0 => binary!("maxss {x}, {y}"),
            1 => binary!("minss {x}, {y}"),
            2 => binary!("mulss {x}, {y}"),
            3 => binary!("divss {x}, {y}"),
            _ => binary!("subss {x}, {y}"),
        }
    }
    f32::from_bits(result)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl ScalarOps for ScopedSse {
    fn maxss(&mut self, a: f32, b: f32) -> f32 { binary_sse(a, b, 0) }
    fn minss(&mut self, a: f32, b: f32) -> f32 { binary_sse(a, b, 1) }
    fn mulss(&mut self, a: f32, b: f32) -> f32 { binary_sse(a, b, 2) }
    fn divss(&mut self, a: f32, b: f32) -> f32 { binary_sse(a, b, 3) }
    fn subss(&mut self, a: f32, b: f32) -> f32 { binary_sse(a, b, 4) }
    fn cvtss2si(&mut self, src: f32) -> i32 {
        let bits = src.to_bits();
        let result: i32;
        // SAFETY: as for binary_sse. CVTSS2SI observes the scoped MXCSR.
        unsafe {
            std::arch::asm!("movd {x}, {bits:e}", "cvtss2si {result:e}, {x}",
                x = out(xmm_reg) _, bits = in(reg) bits,
                result = lateout(reg) result, options(nostack, preserves_flags));
        }
        result
    }
}

/// Evaluate 4584E0 using an explicit fixture policy, not a verified game MXCSR.
/// The incoming thread's complete MXCSR is restored after the call.
pub fn pack_color_mxcsr1f80(components: [f32; 4]) -> Result<u32, SseError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { Ok(pack_color(&mut ScopedSse, components)) }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        { let _ = components; Err(SseError::UnsupportedArchitecture) }
    })
}

/// Evaluate successful vertex writes using the explicit 1F80 fixture policy.
/// This neither observes the original runtime MXCSR nor models allocation.
pub fn successful_quad_mxcsr1f80(
    width: u16, height: u16, offset_x: f32, offset_y: f32, packed_color: u32,
) -> Result<[Vertex; 4], SseError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        { Ok(successful_quad(&mut ScopedSse, width, height, offset_x, offset_y, packed_color)) }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        { let _ = (width, height, offset_x, offset_y, packed_color); Err(SseError::UnsupportedArchitecture) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test double is only used with exactly representable finite values;
    // it is not a replacement for the unknown source floating-point environment.
    struct Exact;
    impl ScalarOps for Exact {
        fn maxss(&mut self, a: f32, b: f32) -> f32 { a.max(b) }
        fn minss(&mut self, a: f32, b: f32) -> f32 { a.min(b) }
        fn mulss(&mut self, a: f32, b: f32) -> f32 { a * b }
        fn divss(&mut self, a: f32, b: f32) -> f32 { a / b }
        fn subss(&mut self, a: f32, b: f32) -> f32 { a - b }
        fn cvtss2si(&mut self, x: f32) -> i32 { x as i32 }
    }

    #[test]
    fn component_order_and_low_byte() {
        assert_eq!(pack_color(&mut Exact, [1.0, 0.0, 1.0, 0.0]), 0x00ff_00ff);
        assert_eq!(pack_color(&mut Exact, [0.0, 1.0, 0.0, 1.0]), 0xff00_ff00);
    }

    #[test]
    fn source_order_and_offsets() {
        let q = successful_quad(&mut Exact, 8, 4, 1.0, 1.0, 0x1234_5678);
        assert_eq!(q.map(|v| v.position), [[-1.25, 0.0, -1.5], [0.75, 0.0, -1.5], [0.75, 0.0, 0.5], [-1.25, 0.0, 0.5]]);
        assert_eq!(q.map(|v| v.uv), [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
        assert!(q.iter().all(|v| v.packed_color == 0x1234_5678));
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn scoped_sse_rounding_and_nan_operand_order() {
        if !std::is_x86_feature_detected!("sse2") { return; }
        // 0.5 * 255 = 127.5, rounded to even; MAXSS selects its NaN
        // source and the following MINSS selects its 255 source.
        let qnan = f32::from_bits(0x7fc0_1234);
        assert_eq!(pack_color_mxcsr1f80([0.5, qnan, -1.0, 1.0]), Ok(0xff80_ff00));
        assert_eq!(successful_quad_mxcsr1f80(8, 4, 1.0, 1.0, 7).unwrap()[0].position, [-1.25, 0.0, -1.5]);
    }
}
