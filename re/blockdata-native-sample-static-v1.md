# Blockdata native sample layout and lookup

Static trace of `D:\Games\SPORE Collection\SporeBin\SporeApp.exe`, SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`; addresses are virtual addresses. No original executable, UI or GPU code was executed. This supersedes the earlier interpretation of 12-byte groups as records: the measured size relationship holds for the measured corpus, but native code accesses three-byte samples and does not establish a quadtree or terrain purpose.

## Loading and key lookup

At `404050..4041BF`, code enumerates resources using type `5CAA5E28`, obtains each stream and checks its size is at least `0x24` and strictly below `0x186A0` (100,000). It allocates space, requests a whole-stream read, and stores the resulting buffer in the owner +30 lookup structure under a 64-bit key assembled from two words of the enumerated resource key. The stream-read return value and all error paths were not independently verified. `414DC0..414ED9` assembles a key from its first two arguments, looks it up through `41F890` on that same owner +30 structure, and passes the found buffer to `414FC0` when output is requested. This establishes a consumer path to these buffers; it does not establish the full gameplay context of callers.

At `41C790..41C917`, a separate path calls `41A9E0` with an object and an output buffer and, on success, creates a resource key containing type `5CAA5E28` and passes buffer pointer and length to a stream virtual +38. This is evidence of a producer/write path; the full construction algorithm and stream implementation are not established here.

## Dimensions and sample decoding

`414FC0..41503A` scans bytes at offsets +28 through +35 in order, stopping at the first zero or after eight bytes. It multiplies the preceding nonzero unsigned bytes to get a sample-count bound. It does **not** count all `04` bytes anywhere in the context. For no dimensions, `415184..4151B0` copies the first and second float triples directly to the two outputs; no payload sample is read. For one or more dimensions, each dimension's stride is the product of the dimensions after it (`41507B..4150AF`), and the last dimension changes fastest.

At `415440..4155DA`, indexed sample `i` is read from `buffer + 36 + 3*i`. For axis `j=0..2`, with `b` the corresponding sample byte and `s` the little-endian f32 at +24, the two output components are:

```text
out_first[j]  = f32_at(4*j)      - float(b >> 4) * s
out_second[j] = f32_at(12 + 4*j) + float(b & 15) * s
```

The upper and lower nibbles are unsigned. This establishes the arithmetic by which the two triples are expanded; identifying them as a particular kind of bound or as a terrain primitive requires caller evidence. Floating-point results should follow f32 native operations; no claim of exact behavior for nonfinite inputs is made.

`414FC0` clamps supplied coordinate components to `[0,1]`; dimensions beyond the supplied coordinate count use zero. `4151C0` scales each coordinate by `dimension-1`, truncates to obtain the lower grid index, records the fractional component and selects the adjacent upper index when possible. `4152A0` traverses those indexed combinations recursively and `413C50` combines the two triples by a weighted vector difference/addition. This supports multidimensional linear interpolation of the two decoded triples on ordinary finite coordinates, conditional on valid data. It does not prove the meaning of the input coordinates, runtime timing, or any physics/gameplay role.

Native code's scan and index arithmetic shown here do not themselves validate the resource length, multiplication overflow, coordinate finiteness, or a suffix after the first zero. A safe implementation must check these independently. Native loading's upper size check is not proof of a complete payload-shape check.

## Corpus boundary and size relationship

A fresh read-only CPU scan using the repository's `dbpf_extract.parse_index` and `refpack.decompress` read 3,098 type-matched resources from the indexed `Data` packages. One package (`Spore_Audio1.package`) had an unsupported index layout and was excluded, so this is not a claim over every installed package. All 3,098 selected records had zero length mismatches under: `36` bytes with no leading nonzero dimensions, else `36 + 3*product(dimensions)` bytes. Their observed contexts were exactly zero through seven leading `04` bytes followed by zero bytes, with counts `424, 660, 930, 738, 256, 74, 14, 2`, respectively. For those specific contexts, `3*4^c = 12*4^(c-1)` when `c>0`, explaining the empirical rule; that identity does not generalize the old count-of-04 parser to arbitrary contexts.

The next gate is to trace callers of `414DC0`, producer `41A9E0`, and independently compare decoded samples against saved source data. Neither the old empirical grouping nor this static mapping proves terrain, quadtree, collision behavior, or which assets are active in a given stage.

## Independent corpus comparison

The bounded Rust `NativeSamples` implementation was compared against a separately written Python formula for all 296,176 packed samples in the 3,098 selected records. Every decoded output float bit pattern matched; dimensions, sample counts, header bits and mixed-radix index round trips were also checked. The input stream SHA-256 is `faa9b17ea564a60e85496568a2b993bf4907566a108e2879805b303d9ab83907`; the Rust output SHA-256 is `40c5c820917d2f61fc6bc42883abdb9417d8899894d26a4e472e4b2c08473a80`. The ignored retained evidence is `target/blockdata-corpus-v1/input.bin`, `rust.bin`, `manifest.json`, and `prepare.log`. The manifest records package hashes and the unsupported and excluded `Spore_Audio1.package`. This is a comparison of independently implemented formulas over the selected finite corpus, not original-game execution or proof of native floating-point control-state parity.

From the repository root, the comparison can be reproduced using a new empty output directory (the writers refuse to overwrite existing files):

```powershell
New-Item -ItemType Directory target/blockdata-corpus-repro
python tools/blockdata_native_corpus_v1.py prepare "D:/Games/SPORE Collection/Data" target/blockdata-corpus-repro/input.bin target/blockdata-corpus-repro/manifest.json
cargo run -p spore-core --example blockdata_native_corpus_v1 -- target/blockdata-corpus-repro/input.bin target/blockdata-corpus-repro/rust.bin
python tools/blockdata_native_corpus_v1.py verify target/blockdata-corpus-repro/input.bin target/blockdata-corpus-repro/rust.bin target/blockdata-corpus-repro/manifest.json
```

The final `verify` command was rerun read-only against the retained `target/blockdata-corpus-v1` files and confirmed the counts and hashes above. Cargo was not run during this documentation update.

The comparison has additional provenance limits from independent review: the 3,098 package records contain 1,432 distinct payload digests, so they are not 3,098 unique payloads. The retained stream does not record each resource's TGI or ordinal and does not resolve package overrides. Python independently checks the decoded formula and outputs but does not independently validate index-to-coordinate mapping; the Rust adapter checks its own index round trips. Package hashes were computed before a separate extraction pass and were not recomputed afterwards. The excluded Audio1 package's target-resource presence is unknown. Original floating-point control state and original native runtime behavior remain unverified.
