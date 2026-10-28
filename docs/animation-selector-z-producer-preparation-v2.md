# Z producer v2 guard correction

V1 remains preserved and is blocked from execution. Its untouched upper-stack guard incorrectly began at SourceS+24. The actual caller pushes ESI, takes outMax at S+24, pushes it, takes outMin at S+12, then pushes it. The helper writes min at S+12..23 and max at S+24..35. Later caller Y/Z scratch writes S+4..11. Therefore S..3 and S+36..end are untouched.

`tools/anim_selector_z_bounds_native_probe_v2.py` pins and wraps frozen v1, changing exactly one owned `ADD ESI,24` to `ADD ESI,36`. All original bytes, constants, source entry/exit, input corpus and Rust model remain unchanged. New complete preparation is `target/animation/selector-Z-producer-native-v2-complete/`; an earlier source-only failed preparation at `...native-v2/` is retained (line-ending-sensitive text hash assertion, before ASM output, no build or native execution).

`tools/anim_selector_z_bounds_audit_v2.py` pins and wraps frozen independent v1 auditor, additionally requiring the corrected linked guard instruction and predecessor SourceStack load. Root must build a fresh v2 host and run this v2 auditor. Existing v1 runner admission is on hold; a new runner version will pin actual v2 identity after root's build.

No original code, compilation, Cargo or GPU execution was performed by this worker. Independent real-stack/ESI/operation-order peer and numerical acceptance remain open.
