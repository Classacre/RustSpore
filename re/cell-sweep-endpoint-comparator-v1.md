# Controlled source endpoint comparator

`BBA700..BBA71C` loads the second endpoint's +04 binary32 value, compares it
against the first endpoint's +04 with `COMISS`, and returns -1 only when the
second is strictly greater; all other outcomes return +1. It never returns 0.
For finite normal or signed-zero inputs, `cell_sweep_endpoint_compare_v1`
models that result. The existing fresh-snapshot sweep now first rejects equal
endpoints, treating +0 and -0 as equal, and then uses this source comparator
for its unique-endpoint ordering. Tie rejection before Rust sorting is an
extra safety policy; it is not a source tie-breaking rule. Nonfinite and
subnormal endpoints remain rejected. The model does not expose native sticky
flags or exception behavior.

`BBA7C0` calls an indirect CRT sorting function with comparator BBA700 and then
merges sorted endpoint lists, handles tombstones and rewrites slot ranks. It
uses a 0x8018-byte probed stack frame and history-dependent inputs. The current
fixture does not execute that sort/merge, reconstruct source tie order or
model persistent endpoint mutation; equivalent ordering for fresh, finite,
unique inputs is an admitted bounded interpretation. A full native sorter
comparison would require a reviewed stack-probe/CRT-sort boundary and guarded
old/new endpoint buffers, slot records and mutation/lifetime admissions.

An unmodified original-instruction comparator host has been prepared in
`target/cell-sweep-endpoint-prep-v1`, and independently compared as described below. Its
finite pairs include equality and signed zero; it
cannot validate the sorter, merge, pair production or live source state. The
existing six sweep tests passed after integration. The earlier BBA9C0 filter
receipt remains tied to its earlier source hashes.


## Controlled original-instruction comparison

The reviewed host ran the complete unmodified 29-byte `BBA700..BBA71C`
function, SHA-256 `b05619e6f5ded95238efc063c573fcbaf63b743a1d815931a032cfb25b2f8ee9`.
All eight comparator outputs matched. The
[receipt](../target/cell-sweep-endpoint-result-v1/receipt.json) SHA-256 is
`695553f51d9fc145381f7ecac2ba73a2bf35fbf41f5feb475da82eba75125992`.
Model/native stdout hashes are
`c1e37dc11df03c417c28065afccbec8a8bdab2aafcdd5852cc8ce4383402a184` and
`5d5f06a45cde3039e4ef223045637733553f3f6e5797c7b981bea10e4db780d6`;
both processes exited zero with empty stderr.

The input pairs are hexadecimal binary32 words and outputs are signed i32:

```text
c0000000 3f800000 -> -1
3f800000 c0000000 -> +1
40000000 40000000 -> +1
00000000 80000000 -> +1
80000000 00000000 -> +1
3f800000 40000000 -> -1
7f7fffff 3f800000 -> +1
3f800000 7f7fffff -> -1
```

The checked model source SHA-256 was
`542a5e92bcaca533825439c2604933025dc633077dbf9e18e34354c13266a6f6`,
and the integrated sweep source was
`2d51cb48f97dbba4ed0283f12c1dd81e159761e538a6f60baa5a3c2280fe5279`.
Host SHA-256 was `75b775a3dab25ee38d341abfcd2b71f1f473bc111ffddca2b0eee4c6ea735907`
and runner SHA-256 was
`1abe74c8b424d808f8b1f0a1abe0d1fd8e7b3569e2ad57f1c3fa765b46ffb259`.
All rows recorded x87 control `027F`, status `0000` and MXCSR `1F80` while
linked bytes, complete RET, guard/stack checks and pre/post pins passed.
The guarded finite corpus excludes NaNs, subnormals, unmasked exceptions,
aliasing and concurrent mutation. Equal values are included to characterize
the comparator; the integrated sweep deliberately rejects such ties. These
results do not test CRT sorting, `BBA7C0` merge, persistent collection history,
slot lifetime, source pair production or live gameplay.


## BBA7C0 host timeout and loader investigation

The first fresh-only BBA7C0 comparison did not produce a parity result. Its
model exited zero, while the native host timed out after 10 seconds with no
stdout or stderr. The original [receipt](../target/cell-fresh-sort-result-v1/receipt.json)
SHA-256 is `734946b244b104d3302ab1246642d30147af85497797e9d987a7ee74d9def1c4`;
its raw streams remain retained. No cause for that uninstrumented timeout is
proved.

A separately reviewed instrumented host timed out with only flushed stage
markers `A7000001` (entry) and `A7000002` (SetErrorMode returned). It did not
emit the marker immediately after `LoadLibraryExW`; in this instrumented run
execution therefore remained inside that load call. The diagnostic
[receipt](../target/cell-fresh-sort-diagnostic-result-v1/receipt.json) SHA-256
is `7d4a52c5ae9801aee754476caab907f3bb5382091e2c0a951762040bad900201`
and raw stdout SHA-256
`876ce8b13c9da0b250a333833dad8250bc85cdf56689e6300e88f78043f60253`.
The instrumentation adds WriteFile calls and a qsort wrapper, so it does not
establish the cause of the separate original timeout or establish parity.

Read-only PE inspection found that SporeApp.exe embeds an RT_MANIFEST
dependency on `Microsoft.VC90.CRT`, x86 version `9.0.21022.8`, public key
token `1fc8b3b9a1e18e3b`. The standalone comparison host has no resource
directory/embedded manifest. Installed `SporeBin/Microsoft.VC90.CRT.manifest`
has the matching assembly identity (SHA-256
`9e0220511d4ebdb014cc17ecb8319d57e3b0fea09681a80d8084aa8647196582`).
The pinned installed `msvcr90.dll` imports only KERNEL32 at PE import-table
level, but contains `R6034` incorrect-runtime-loading, `MessageBoxA`, USER32
and activation-context-related strings. Those strings and manifest mismatch
make activation context a concrete hypothesis, not an observed error dialog
or proof of the hang's cause. `SetErrorMode` and CREATE_NO_WINDOW do not prove
that a runtime-created dialog cannot wait. No installed DLL or manifest was
modified.


## Activation-context diagnostic on private desktop

A separately reviewed instrumented host created and activated a context from
SporeApp.exe's embedded CRT manifest (resource ordinal 1) with SporeBin as
assembly directory, before loading the pinned installed CRT. Its runner created
a unique private desktop without switching to it, launched the bounded child
there, and closed its desktop handle after the child was reaped. The child
exited zero: markers show successful context creation/activation and loader
return, six qsort entries and exits, and all six source-sort returns. The
[diagnostic receipt](../target/cell-fresh-sort-activation-private-result-v1/receipt.json)
SHA-256 is `29140905603ec983240e264cdb308a0b35509c229d28bcb5bb4a33299ba3ad02`;
raw stdout SHA-256 is
`dc704d9da8ac7a9fa5678c15fddde789f1d3983a7b487ac950d585e2f4d708c9`.
The receipt records desktop access `00C3`, no switch, child reaped before
close, and successful CloseDesktop.

This shows that activation permitted this *instrumented* run to progress. It
does not prove the cause of the earlier uninstrumented timeout or establish
sort parity: the marker calls and qsort wrapper alter execution and call
topology. The installed executable, DLL and manifest were not modified.


## Controlled fresh-only BBA7C0 comparison

The reviewed, uninstrumented activated host ran the complete original
`BBA7C0..BBA9BE` (511 bytes, SHA-256
`46084f94dc1d8fb34cedf4f8f79b8912300bbf25e7ef8c033ed8b0f6e178a908`),
original stack probe `11E04F0..11E051A` (43 bytes, SHA-256
`f8675b4976f7b8efc0bd917aebab734d11ca90222d20bcd2a166ce9914ed2b6e`)
and original comparator `BBA700..BBA71C`, with the three documented address
relocations and direct installed MSVCR90 qsort. All three fresh-only cases
matched the model's 104 bytes: both 16-byte slot records, both four-entry
sorted endpoint arrays, and both returned counts. The
[receipt](../target/cell-fresh-sort-activated-result-v1/receipt.json) SHA-256
is `a638aafa53f3c5061fbd3ba70daa02d514776791e256066f216e245997ee85bd`.
Model/native stdout SHA-256 values are
`6c39eb4a200813b3b8bef797d95bb6e6d0a8acd0c0527039a9c70a3db1cd7c18` and
`5f0407208dc88d9fedb639ba60a723b29b12a448d1ae9e22c46add3bb4a0ef87`;
both processes exited zero with no stderr.

Each pair below is `(actor, tag, min-x, min-y, max-x, max-y)` in hex; each
two rows form one fresh collection. Both axes have unique finite endpoints.

```text
00010000 00000000 00000000 00000000 40400000 40400000
00020001 00000004 3f800000 3f800000 40000000 40000000
00010000 00000001 00000000 c0400000 3f800000 bf800000
00020001 00000002 40000000 40000000 40800000 40c00000
80010000 ffffffff c0000000 40000000 bf800000 40c00000
00020001 00000000 00000000 c0400000 3f800000 bf800000
```

The host activated the pinned source manifest and used the exact installed
`msvcr90.dll` (SHA-256
`ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185`).
It checked the loaded module path and qsort address owner; Windows loader and
in-memory module integrity remain explicit admissions. Both model and native
children ran on separate private, non-switched desktops that were closed after
child reaping. Host SHA-256 was
`bc2d9d5b1b5146bd3e8ed1accbadc76c954d0321ee050b23717be32d17c92965`;
reviewed runner SHA-256 was
`ab6eb6a903fd5fa6a70e61fa9ae5b5bacb761faa0dfe4074b9a650dec09ee922`.
The model source SHA-256 was
`78d820b3406253cf15b0d374d55f941ab3f344893ddc103e21378df25316732e`.
Original instruction coverage, relocated targets, guarded buffers, stack,
pre/post pins and x87/MXCSR controls passed; each row recorded x87 `027F`,
status `0000`, and MXCSR `1F80`.

Only old-count-zero, two-slot, four-endpoint-per-axis, finite-unique, no-
tombstone cases are covered. This does not validate incremental merging,
old entries, ties, free-list/reuse, capacity overflow, callbacks, live actor
identity, or persistent collection lifetime. Earlier failed unactivated hosts
and the instrumented activation diagnostic remain distinct observations.
