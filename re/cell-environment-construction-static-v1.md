# Cell environment construction and all-slot scan boundary

Static PE analysis of installed `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`. This complements [the flow producer trace](cell-flow-field-producer-static-v1.md) and does not observe initialization, a live environment, its allocator's returned contents, or a player tick. All claims below require the native allocations, pointers, calls and bounds to be valid.

## Construction

At `E80F87`, one caller invokes `E4EB20`; after return, `E80F8C..E80F92` stores EAX at `[016B4BE4]+4108`. Earlier instructions in this caller populate bound globals including `016B4C68/6C` and `016B4C74/78`. This is one static initialization path, not proof that it is the only writer or that it ran for an observed game session.

`E4EB20..E4EC86` requests `30580h` bytes through `F472B0` and uses the returned pointer without a local null check. It constructs a four-word helper from `016B4994/98`, zero words and argument 2 via `A7C270`, then runs exactly `5DCh` (1500) iterations. Each calls `A7C1F0`, computes X and Y with the resulting two local values and current global bound pairs, and writes them to the indexed 12-byte entries beginning at `+4`; Z is written as positive zero. It calls `E56BC0` on each resulting position and copies its returned three words into the indexed `+4654` vector array. The bounds, helper values, global lookup state, floating controls and all called functions are inputs; the resulting coordinates are not recovered from a fixed creation resource.

After the loop, `E4EC0A..E4EC5F` explicitly calls the `11E052E` zero-fill thunk for these ranges:

| Offset | Bytes | Observed effect |
|---|---:|---|
| `+8CA4` | `4650h` | 1500 three-word entries cleared |
| `+4654` | `4650h` | 1500 three-word entries cleared; this overwrites the earlier `E56BC0` results in that array |
| `+11944` | `4650h` | 1500 three-word entries cleared |
| `+D2F4` | `4650h` | 1500 three-word entries cleared |
| `+18E74` | `1770h` | 1500 four-byte words cleared |

This overwrite does not prove `E56BC0` or its callees have no other effects. `E4EC67..E4EC73` also stores zero to `+1BD54/+1BD58` and `FFFFFFFFh` to `+1BD5C`. These are construction-time writes, not a live-state snapshot.

Within the inspected `E4EB20` body, there is no explicit initialization of the 1500 words at `+15F94` or `+17704`, the grid outputs `+1BD60`, `+1C210`, `+1D020`, the free-link array `+1A5E4`, or scratch beginning `+1D4D0`. This bounded negative observation is **not** proof that these memory locations contain garbage or zero. The allocation wrapper `F472B0` delegates via `928750` to `927940`; a complete allocator zero-fill guarantee and all intervening writers have not been established. In particular, the first reached `E4EA40` calls `E4DC40` to copy the then-current `+15F94` words into accepted records **before** `E4E730` clears/repopulates that scalar array. One cannot infer first-rebuild copied scalar values from the later clear.

## Indexed allocation, release and scan

Starting from the just-described counter words, `E4DA20` computes old `+1BD54 + 1` and returns `-1` when that equals 1500. Otherwise it updates the count and, when the old count equals `+1BD58`, increments `+1BD58` and uses the computed index; under the untouched initial state this first index is 1. On the other branch it consumes the head at `+1BD5C` and follows a word in the `+1A5E4` array. It writes the supplied point and vector to the indexed `+4` and `+4654` arrays, copies three globals into indexed `+D2F4`, and stores a supplied word in indexed `+18E74`. These observations do not establish an invalid-state check, ownership, or validity of an arbitrary free-list index.

`E4EA90`'s release-like path obtains a replacement position from `DF5AE0` using the global bounds, calls `E56BC0` for that position, stores its three returned words to the same indexed `+4654` array, decrements `+1BD54`, zeroes the indexed `+18E74` word, links the index into `+1A5E4` and installs it as `+1BD5C`. It does not simply erase the slot or leave its position/vector necessarily zero. The validity and exact distribution of `DF5AE0` and the values returned by `E56BC0` remain external to this trace.

`E4EA40` passes literal count 1500 to `E4DC40`; the bounded loop reads each slot's XY and, if its computed bin is valid and not already full, copies that slot's position, vector and scalar. It does not consult the `+18E74` word or the two counters in this binning loop. A zero `+18E74` word therefore does not by itself mean the slot is excluded: all-slot background and released entries can contribute if they pass the bin and eight-record gates. Calling this a complete or accurate runtime field requires, at minimum, valid initialized 1500-slot position/vector/scalar storage, admitted coordinate conversion and FP state, correct environment identity, appropriate sequencing with allocation/release and other writers, and the intervening `E4E730/E4E4C0` effects. There is no proof here of first-frame field values, a zero world, or which live slots contributed.

## Reproduction

The retained PE-byte disassemblies are `target/environment-init-disassembly-20261105.txt`, `target/environment-init-prereqs-20261105.txt` and `target/lifecycle-independent-review-20261105.txt`. Reproduce with `pefile` and x86-32 Capstone on the hash-checked PE, using `get_data(start - 0x400000, end - start)` for instruction-aligned ranges `E4EB20..E4EC87`, `E4EA90..E4EB19`, `E4DA20..E4DAD3`, `E80F87..E80F98`, `E56BC0..E56C32`, `A7C1F0..A7C270`, `F472B0..F472DA`, and `928750..92878E`; ends are exclusive. Allocator target `927940` requires its own full analysis for allocation-content guarantees. An independent static review checked the constructor, zero-fill extents and allocation/release paths; it did not establish allocator contents or live values. No Cargo, GUI, game execution or runtime capture was used.
