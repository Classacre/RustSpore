# Controlled contact geometry continuation

The opt-in owned pair fixture now invokes a checked continuation before
E6A450. The small-distance result carries either six caller-supplied opaque
words unchanged or `None`; the fixture supplies `None` and does not invent the
source frame's contents. For finite nonnegative radii and the normal path,
it uses x87 control 027F for square root and binary32 store, separately rounded
SSE under MXCSR 1F80 for depth/normal/point, and returns separated or
nondegenerate output. Negative radii, nonfinite values and unsupported
intermediates are checked policy. This does not invoke E6A450, establish native
actor/cache identity, or assert a gameplay contact. A later six-case
original-instruction comparison is recorded below. Earlier first-side and
pair-prefix comparisons apply to their separately pinned earlier source files,
not this expanded model source.

Focused tests including installed source inspection passed 11/11 with
`tools\build.bat test -p spore-bevy cell_source_player_v1 -- --include-ignored`
and `RUSTSPORE_INSTALL` set to the pinned installation. Those checks establish
controlled model behavior, not live gameplay.

The preparation step built an original-instruction host without running it. The 184-byte
fragment `E6B099..E6B150`, SHA-256
`c16003e09ec659ab4f2d729301fa51a4dbd5ce9cfa90756776149069e8f5ef74`, has a
redirected external `JA` operand and two relocated constant addresses. Static
PE inspection found all other bytes intact, a separated trampoline at
`00401990` (`mov eax,1; ret`), fallthrough `xor eax,eax; ret`, and constants
`0040300A=3F800000` and `0040300E=3F000000`. The host initializes 027F x87 and
1F80 MXCSR, uses guarded synthetic inputs and frame, and captures distance,
depth, six local words and control/status. Separated cases retain the sentinel
locals. Cases include tangent, separated, unequal radii, 3-4-5 direction,
sqrt(2), and the exact nondegenerate threshold. The source prefix inputs are
cross-checked by the Rust adapter before comparison.

Review `tools/cell_contact_geometry_prepare_v1.py`, generated
`target/cell-contact-geometry-prep-v2/host-x86.asm` and linked PE, the case
bytes, model adapter and `tools/cell_contact_geometry_run_v1.py` before any
execution. The v1 build failed on MASM operand widths and remains preserved;
v2 built without running original instructions. Manifest pins are in the v2
directory. Runner SHA-256 is
`93cae81778c32a58b062eb7fcdc590299dd4734cb3bf66f131ce9202622b237a`.
The proposed reviewed invocation is:

```powershell
python tools/cell_contact_geometry_run_v1.py 93cae81778c32a58b062eb7fcdc590299dd4734cb3bf66f131ce9202622b237a
```

The runner uses an exclusive fresh output directory, captures raw outputs and
failure receipt, checks source/host/model pins, patched byte boundaries,
trampolines, constants and imports, and checks x87/MXCSR control fields. It
records but does not compare sticky status. Hash checks do not establish atomic
file stability. Even a pass would cover only these fixed synthetic normal-path
inputs, not the small branch's inherited locals, callbacks or gameplay.

## Reviewed comparison result

The later reviewed host run matched the Rust model's branch, binary32
distance/depth, and normal/point words in all six cases. Both processes exited
zero. The host supplied first and second centers/radii and admitted prefix
values directly; it did not reproduce actor or feature lookup. Each case used
first center (0,0,0):

| Case | Second center; first and second radii | Observed result |
|---|---|---|
| 0 | (1,0,0); 0.5, 0.5 | tangent; depth `00000000`, point X `3F000000` |
| 1 | (1,0,0); 0.25, 0.25 | separated; depth `BF000000`; six locals remained `CCCCCCCC` |
| 2 | (1,0,0); 1, 0.5 | depth `3F000000`, point X `3F400000` |
| 3 | (3,4,0); 3, 3 | distance `40A00000`, depth `3F800000`, normal X/Y `3F19999A 3F4CCCCD` |
| 4 | (1,1,0); 1, 1 | distance `3FB504F3`, depth `3F15F61A`, normal X/Y `3F3504F3 3F3504F3` |
| 5 | (`3B800000`,0,0); 1, 1 | exact threshold boundary; distance `3B800000`, depth `3FFF8000` |

The runner checked x87 control word `027F` and MXCSR controls `1F80` after
masking sticky flags. It recorded the x87 control/status words
`0000027F 0000027F 0000027F 0000027F 0020027F 0000027F` and MXCSR words
`1F80 1F80 1F80 1FA0 1FA0 1F80`; sticky statuses were not compared. Raw
normal and point words are retained in the native and model stdout files.

The compared expanded model source SHA-256 is
`04bebc36266ba420f700c5004c2ebcaac070e17c7d7908ca794ada6e5b80f3aa`.
The receipt SHA-256 is
`f050bf37a7a4db224f044d5c618dd03308f9ab24500fa205c0ef8dfeaffc465b`;
native stdout SHA-256
`92bc41586427cfe58f2d4af9a29329c0d1bafbb3b8791832e827ff0179f606a5`;
model stdout SHA-256
`1eca20a8b3c4909affe903ac6dae31eb8feeacb1f7414238aa85b38699c66c35`.
They are retained under `target/cell-contact-geometry-run-v1/`. This agreement
covers only these six finite normal-path cases and checked output words. It
does not validate native exception-status parity, the small-distance carryover,
E6A450 or callback effects, actual identity/lifetimes, stable-file admission
against transient replacement, or a live contact.
