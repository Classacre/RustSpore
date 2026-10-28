# Cell screen color helper: native comparison

The retained comparison covers the packed color returned by original x86 helper `4584E0`, not the full screen-effect draw. The immutable source `SporeBin/SporeApp.exe` has SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. Source bytes `[4584E0, 4585EB)` are 267 bytes with SHA-256 `bcd0b6a5a2d65fcceb4402b691a7304eb3511a703b844309a01138bcf6bc9be1`; the only absolute operand is the four-byte constant `013EC540 = 437F0000` (255.0). Independent bounded decoding found no branch or call and a final plain `RET`; see [the render trace](cell-screen-effect-render-trace-v1.md) and [relocation preparation](cell-screen-color-native-relocation-preparation-v2.md).

The first host attempted to reserve the original code region. The runner console reported `RuntimeError: ('native', 10, 0, b'')`; exit 10 is the host's allocation failure code. Retained `target/cell-screen-color-native-result-v1/` has an empty native reply and `target/cell-screen-color-native-result-v1-hashes.json` records runner exit 1. Those saved files alone do not identify the failed reservation. No comparison result is claimed for that attempt.

The second host copies the unchanged function to `04580000` and maps the constant at its original absolute address. Independent static review verified the source and constant bytes, complete position-independent instruction decode apart from the mapped constant, cdecl argument and return, and stack, input-canary, direction-flag and MXCSR-control guards. The linked x86 PE was independently inspected: fixed base `10000000`, exact original bytes at embedded VA `10013180`, correct relocation/copy/call to `04580000`, and constant copy to `013EC540`. Linked host SHA-256 is `a41dd8a8b39e2e5e632a856dbaab2e8e64ef08828668ebe1a90a42e02cc412ee`. The host uses fixture MXCSR `1F80` and restores the incoming value; its post-call guard masks out sticky status bits.

The retained successful run `target/cell-screen-color-native-result-v2/` has 7,228 inputs: per-channel special values, neighbors of scaled half-integer thresholds, and deterministic random finite bit patterns. Independent re-reading regenerated the exact request and compared each raw reply pair: **7,228 matches, zero adapter rejections**. Both stderr files were empty. The saved hashes are:

| Artifact | SHA-256 |
| --- | --- |
| Request | `442837d1f0ac85967ccd0b8cb627886a6a693fecfe844066b2ee80609abcd287` |
| Native reply | `8ab3847b6775556bf3dd5f8a1c877f9964a6e5db58bc61fa0d0d4b46666dabda` |
| Adapter reply | `5397594d210594b4b014ed08f9a3e53c79a848e3b9ff551b5b72ff6c494a961f` |
| Adapter executable | `ffd711433e063004209934275bc48d45a1bbc4e84f9b1f0e467a390d3c62fe9a` |

`target/cell-screen-color-native-result-v2-hashes.json` records runner exit zero and matching before/after hashes for ten named files, including the original EXE, host and adapter executables, generator, runner, adapter source and `cell_screen_quad_v1.rs`; current files independently matched these records. The runner also checks the two expected executable hashes before and after launch. This continuity evidence does not itself prove the compiler or linker identity or bind the binary to every transitive source; in particular `native_sse.rs` is not in the snapshot. The preparation manifest's `built: false` and `executed: false` describe preparation, not the later linked-host run.

This establishes agreement for the recorded corpus and fixture MXCSR only. It does not compare sticky exception flags, establish the game's active floating-point state, validate quad coordinates, or establish shader execution, visual output, or overall game fidelity. Original installation files were not written by this review.
