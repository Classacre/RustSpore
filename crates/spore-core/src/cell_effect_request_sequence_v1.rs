//! Bounded effect request sequence observed in SporeApp.exe FUN_00627de0.
//! Source: target/rea/20261028-source-runtime-v6/156-analyze_function.json,
//! binary SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! This models the calls after a world has been resolved. It does not load an
//! effect, resolve the null-world provider, or establish which directory wins.

pub const CELL_SKYBOX_EFFECT: u32 = 0x05c7_a4d0;
pub const FALLBACK_EFFECT: u32 = 0xe18d_6423;

/// Call the resolved world's request operation with group zero. The caller
/// supplies the operation and any output state it mutates. The source checks
/// whether the first return byte is exactly one; it ignores the fallback result.
/// Returns whether the fallback was called, not whether an effect was loaded.
pub fn request_with_fallback(mut request: impl FnMut(u32, u32) -> u8, id: u32) -> bool {
    if request(id, 0) == 1 {
        false
    } else {
        let _ = request(FALLBACK_EFFECT, 0);
        true
    }
}
