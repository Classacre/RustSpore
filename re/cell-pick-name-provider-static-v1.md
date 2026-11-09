# Cell cursor selection name provider: source-bound conditional join

Static x86 trace against installed base `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The resource join uses retained, hash-pinned package-derived bytes. No original
code, live actor, input event, locale provider or game process ran here.

`E620D0` resolves the candidate actor's definition through `E4CAA0` and a
temporary reference. `E4CAA0` jumps to `E823D0`, which increments the supplied
reference's `+0C`, calls `E822F0` if its `+08` is null, then calls the
resolved object's virtual `+0C` with `02269ED1` and returns result `+14`.
It is not an integer-index lookup into an arbitrary definition array. The
actual actor `+108` binding, reference lifetime and provider remain gates.

At `E620FC..E62110`, `E620D0` calls `6B4EE0` on state `+920` with fixed
table ID `04DC2F94`, the definition's word `+A4`, and zero. `6B4EE0` is
stateful: it may retain/reuse a loaded table reference, otherwise requests a
resource with type `02FAC0B6` and scope `02FABF01` via `67D960` virtual
`+24`; it invokes the table object's virtual `+14` for the requested ID and
checks that the first returned UTF-16 unit is nonzero. The success path also
invokes `6B4D40`; output processing, cached state, selected locale and mounted
resource priority cannot be inferred from a package file alone. On success
`E620D0` copies `6B4FF0`'s terminated text into its output and returns true.
Otherwise it checks the embedded definition units at `+04`; a nonzero first
unit causes a terminated-string copy and true. If that is empty it calls
`E845D0` with the actor triple at `+FC` and the output string object.

`E845D0` immediately returns false if the first triple word is zero. Otherwise
it preserves the first and third words and substitutes resource type
`030BDEE3` for the middle word, then calls the object returned by `67D7F0`
through virtual `+0C` (`E845D0..E8463C`). On success it asks the returned
object's virtual `+0C` for interface `030BDEE3`, calls `414DA0`, reads the
runtime object's `+78` pointer and copies its terminated UTF-16 units; it
returns true after the successful provider path. The current
`content_manifest` parser validates the disk framing for that resource type,
but does not expose the fields that produce runtime `+78` or establish which
specific object a live actor triple resolves to. The producer branches are
detailed below. Resource success does not prove a nonempty name in this final
branch.

## Reproducible retained-source join

`tools/cell_pick_locale_join_v1.py` joins all 489 definition copies in
`target/source-cell/definition-corpus-v4/corpus.bin` (SHA-256
`f0a940e9c6a2a6baa994f5df3b2b21b19bd316920a43cc9e77649547ed168673`)
with the explicit en-US `04DC2F94` locale table in the retained locale corpus
(SHA-256 `a6fbed4f4e2954689735149d2cf86cf1b1785eb43d4780d655fbad8bf3d4641c`).
The metadata-bound Python tokenizer from `locale_table_probe.py` retains
last-entry duplicate priority. Per-copy package/key/payload digest, raw row
ID and category are in `target/source-cell/pick-locale-join-v1/report.json`.
The retained 82-package census contains 23 language copies of this table;
this join chooses en-US explicitly, not an observed runtime selection.

| Conditional en-US outcome | Definition copies |
|---|---:|
| Nonempty locale lookup candidate | 128 |
| No nonempty locale candidate; nonempty embedded label | 12 |
| Neither; creation-provider fallback required | 349 |

All 140 nonempty embedded labels have a zero terminator within their 80-unit
field. The 349 remaining definitions have zero `+A4` and empty embedded text;
the selected en-US table has no ID zero. Counts are resource **copies**, not
unique definitions, live actors or measured game selections. The 128/12
outcomes apply only if the supplied en-US lookup result and preceding runtime
reference conditions are admitted; in particular a different selected locale,
overlay or cached table can change the branch and visible text.

The next practical bounded bridge can combine a caller-admitted locale table
result and source definition bytes, preserving the embedded-label fallback and
raw UTF-16, and require an explicit creation-provider result only when reached.
For a genuine application path, first bind the actual actor's `+108` and `+FC`
to resolved source resources, determine active locale/overlay and cache
behavior, reconstruct the `030BDEE3` runtime `+78` producer, and connect those
providers and output-string side effects to the existing ordered pick tree.
The existing prototype `CellWorld` does not establish these facts.

## Locale processing and manifest name producer

`6B4D40` stores the supplied raw string pointer when its copy argument is zero,
then obtains the object returned by `67D970` virtual `+10` and calls its
virtual `+24` on the raw string, storing the resulting classification byte.
`6B4FF0` returns the raw pointer only when its cached-output flag is clear,
that classification is zero, and global byte `01606354` is zero. Otherwise it
uses a cached or processed output, including a virtual `+2C` call on that
service object. Thus a parsed locale lookup supplies raw units and the
`6B4EE0` nonempty branch decision, but supplies the final units only under
explicitly admitted raw-return state. Runtime token processing is unresolved.

The `030BDEE3` reader `54C620` uses `93A3F0` and `93A470` for 32-bit and
64-bit words, respectively, with conversion controlled by stream mode. On the
version-10 path, assuming ordinary big-endian package-stream conversion and
no exceptional seek in `54CA5C..54CBD4`, a discriminator is at offset 60.
When it is zero, `54CC93..54CFA6` reads a 64-bit field, then three
length-prefixed raw UTF-16 buffers into object `+58`, `+78`, and `+88`;
the first two lengths must each be below 256. The `+78` bytes follow the
`+58` length and its `2*length` bytes. The stream virtual `+30` supplies
the bytes directly. A safe implementation must additionally check bounds,
termination and native reader's other success conditions.

When that discriminator is nonzero, `54CFAB..54D281` instead reads four
32-bit words: a table ID and three IDs. The first ID must not equal
`FFFFFFFF` and its `6B4EE0` lookup must succeed; only then does it try the
second ID, also requiring a successful lookup, and copy `6B4FF0`'s processed
UTF-16 result into object `+78`. If the third ID is not `FFFFFFFF`, its lookup must also succeed and
its processed result is copied to `+88` (`54D2BD..54D3A7`). Failure
returns false from the reader. These words occur at offsets 64, 68, 72 and 76 under the stated framing.
The parser currently skips all these bytes, so its ASCII tag and later string
list are not the `+78` name. The remaining fields and final reader success
still matter for resource admission.

An explicit installed example from `Data/Spore_Content.package`, group
`0001FFFF`, instance `40666200D1B4BB56`, decompressed payload SHA-256
`9849d099c437bb722a8fe7a7b9bd55ab50a9b6f743828668c75ef3b53183c936`,
has discriminator `FFFFFFFF`, table `04DC2F94`, first ID `00000001`,
and +78 ID `E24369AD`. The retained explicit en-US table above maps those
IDs to raw strings `Maxis` and `Expolosif`, respectively. This illustrates
the two required lookups; it does not show that any selected live actor
resolves to this manifest, that this table wins runtime provider resolution,
or that locale output processing leaves either string unchanged. The exact
locale parser can support a bounded explicit-resource bridge once manifest
framing, prerequisite lookup and postprocessing inputs are represented; it
cannot by itself supply a verified live selection label.

## Bounded Rust implementation and explicit sample

`content_manifest::localized_name_prefix` exposes the version-10 nonzero
discriminator prefix only when the caller admits ordinary framing; it reports
the raw-string branch, truncation and other versions separately. Its ordered
lookup takes an explicitly selected parsed locale table, reports unresolved
provider or mismatched table, follows required first/name and optional third
lookup order, and returns raw candidates. It does not establish a complete
reader result, native provider selection, output processing or actor binding.

The six focused `content_manifest` tests passed through `tools\build.bat`.
A separately built example `manifest_locale_sample_v1` was run with retained
`target/source-cell/manifest-locale-sample-v1/manifest.bin` and
`target/source-locale/corpus-v2/sources/01169.locale`; it confirmed the raw
`Maxis` / `Expolosif` pair and optional sentinel using the Rust APIs. The
example source SHA-256 is
`31fd142be0c2f62edc40563900451507a96ebe0b8c86a0fa39c84c78aed2fa87`,
and the executable is
`007dd6638f7321005e80930cbde175d56412c9c5479a14ca9a7b718f66d07c46`.
This is a supplied-package integration check, not an original-instruction
comparison or runtime measurement.

The successful `6B4EE0` call clears state byte `+11` at `6B4EED`;
`6B4D40` then assigns raw pointer `+08` and classifier byte `+10`. Provided
no intervening mutation occurs, `6B4FF0` returns that pointer unchanged when
`+10` and `01606354` are both zero (`6B5031..6B503E`, `6B50EC`). A
separate preexisting-cache path returns raw when `+11` is nonzero and `+04`
is null (`6B5014..6B5019`); nonnull `+04` returns its stored pointer. A
bounded branch model could expose these bytes and pointer-presence explicitly
and report processed output unresolved. A source-instruction comparison could
exercise the raw-return branches using a reviewed x86 host and explicit
global/state inputs; stubs would not establish real classifier or processor
behavior. The bounded comparison below covers only the raw-return cases.

## Raw-return source-instruction comparison

The parent reviewed and executed `tools/cell_locale_raw_host_run_v1.py` once.
The host copied unchanged original ranges `6B4FF0..6B501E`,
`6B5031..6B5043`, and `6B50EC..6B50FB` to one dynamically allocated page,
preserving their relative branch displacements and the original absolute
global address `01606354`. Other bytes were `INT3`, including the cached-output
and processing paths. Six fixed input tuples cover both cached-flag/absent-cache
and uncached zero-classifier/zero-global raw returns; no input represents a
real classifier result. The source prologue pushes an original SEH handler
address; any exception is fatal and is not a comparison result.

The source-free fixed-address preflight first found `006B0000` occupied and
exited 11; its output remains in `target/source-cell/locale-raw-host-v2`.
The reviewed successor preflight allocated a relocated code page and fixed
global page and released them successfully; its result is retained in
`target/source-cell/locale-raw-host-v3`. Neither preflight executed source.

The one-shot relocated host exited 0 with 78 output bytes SHA-256
`943415f3f1efc5996496c9ac4d1769f4141594c435036f982776ad6fb5a8d965`.
The pinned Rust comparator (SHA-256
`af786dc398d36076c6d10a5eac992e0f4ebf36195ee89b2d57df28a99cf2c274`)
matched all six rows. Host SHA-256
`8812847375a3a361b0f605f7cca67aec5365afe38cde5fa11b03a7f29cc5b11a`;
runner SHA-256
`923c5140761a754584202d838ef94e6e5c5b98811a5cc559e6752ea7c0ad1855`.
The exact disassembly and allocation assumptions are in the host v3 audit;
output and run record are in `target/source-cell/locale-raw-native-run-v1`.
This establishes only the supplied raw-return branches, not processed output,
actual classifier values, provider or game runtime behavior.

## Zero-discriminator raw-prefix subset

The separate `content_manifest::raw_name_prefix` handles the ordinary
version-10 zero-discriminator branch under explicitly admitted big-endian
integer framing and no exceptional seek. It preserves the exact little-endian
raw string bytes and exposes name units before the first NUL. Lengths for
`+58` and `+78` are each strictly below 256; `+88` is below 4096. The helper
`552690` resizes the string container, and growth helper `42C750` stores a
zero unit at the new end (`42C840..42C848`), so the later terminated copy can
stop at an embedded NUL or that allocated terminator. This conclusion
assumes successful allocation and a valid constructed container.

The model intentionally rejects short reads even though the source tests
raw stream return only for nonzero, and reports zero lengths for `+58` or
`+78` as unresolved because the source still calls stream virtual `+30`
and its zero-byte return behavior has not been established. The zero-length
`+88` read is skipped by source. These checks do not model the remainder of
the reader or prove resource-provider success. Synthetic tests exercise the
prefix and embedded-NUL behavior. A static scan of all 1,641 resources from
the parseable base `Data/*.package` census found version 10 and nonzero
discriminator at offset 60 in every copy; no installed zero-branch record
was validated or claimed.
The reproducible read-only census script is
`tools/cell_manifest_discriminator_census_v1.py` (SHA-256
`ba7b52b275506a9307381af7f64882c8106f4e6d06d885436bd500e8841b870b`);
the per-resource report SHA-256 is
`c9c2adc9aaba5de9454326a28ce259fff3375fc782a8ffe5fdd7c0cbd59271b3`.
Six package indexes were parsed; `Spore_Audio1.package` was skipped because
its index layout is unsupported. This is not an exhaustive installed-resource
or live-provider census.

## Typed Cell-definition cohort comparison

The separate `cell_definition::conditional_name` models the `E620D0`
definition path; it is not the `030BDEE3` manifest-prefix parser. It takes
an explicitly selected table `04DC2F94` and output-state admission, returns
raw locale units only for the admitted `6B4FF0` raw-return branch, and
otherwise reports processing unresolved. On lookup failure it checks the
fixed embedded UTF-16 label and then reports creation-provider unresolved.
It rejects a missing terminator within the embedded field rather than
following the native copy beyond it. Unknown provider/cache state remains
unresolved. With an explicitly null cached table and ID `FFFFFFFF`,
`6B4F23..6B4F33` skips the provider lookup; a nonnull cached table takes a
different path, so that shortcut is not applied to unknown cache state.

The independent Python reference `tools/cell_definition_name_reference_v1.py`
(SHA-256 `d4f5561e8ceb4036e933a8943088fefe39ba29e699de5f50ba1e8aa1eb246998`)
emits ordinal, lookup ID, branch and exact raw units from the retained
489-copy definition corpus and explicit en-US locale table pinned above.
Its result `target/source-cell/definition-name-compare-v1/python-expected.tsv`
has SHA-256 `034ebecce5bce87f0620b803e26a8455fa4b43b60806852a2ff51e92a32ad3fd`.
The wrapper-built Rust example `cell_definition_name_compare_v1`
(executable SHA-256 `09ac9db3d35c7c31ef32b43a43d40155602062ead71da1080aa264d49c9be4b1`)
matched every row and unit: 489/489 resource copies, comprising 128
conditional locale results, 12 terminated embedded fallbacks and 349
creation-provider-unresolved outcomes. No records were omitted from this
retained corpus. This compares two source-derived parsers on pinned package
bytes, not original instructions, full installed coverage, active locale
selection, processing, or live actor binding. The five focused
`cell_definition` tests passed through `tools\build.bat`.

## Explicit conditional definition-to-manifest candidates

Retained definition copy 46 is the `Data/PatchData.package`
`CBCD1287:DFAD9F51:00000000` record, SHA-256
`f12d98c0f59af6d4d58174016f1e04d709647480310103341193df9b0fa76542`.
It is one of the 349 creation-provider fallbacks under the admitted en-US
table. The separately traced PatchData structure `CBCD1287:4B9EF6DC:00000000`
first has a kind-4 row with raw list ID `9A391385`; the explicit
`9A391385:F9C3D770:00000000` list has four type-zero rows. The construction
route and source hashes for structure/list are in
[cell-kind4-cache-selection-static-v1.md](cell-kind4-cache-selection-static-v1.md).

On an admitted cache miss and selection of one of those type-zero rows,
`E5C283..E5C2A0` supplies `(row ID, 3D97A8E4, [015A8D0C])`. The executable's
initial image word at `015A8D0C` is `40616200`; this is not a measurement of
its mutable runtime value. If that value is still present and the triple
reaches `E845D0` without later replacement, its type substitution yields the
following exact `030BDEE3` low-instance/scope pairs in the explicitly
examined `Data/Spore_Content.package`. The table and raw locale strings below
use the pinned en-US table, not an observed provider or final processed text.

| Selected row / manifest instance | Manifest SHA-256 | Locale name ID | Raw en-US units as text |
|---|---|---|---|
| `A1C46DF2` | `e4517f1e72a6f868691c9635b3329dffa2fe7ffdf266e0b653c14bc9e4e7b62a` | `356360F1` | Oog |
| `E219E5D0` | `66c8b8deb3efa44e6c1364dc5d0f502e198018031f2e05903d355b4a1a1422f3` | `FC40DEC7` | Yurgh |
| `E219E5D3` | `01747f124f8891a8d5fac8f4c49f1ea99af2b823024216cfada016366d47df41` | `414FC279` | Noog |
| `E219E5D2` | `f042d1ba56436d38deb0a31f059a7a390849ad1522f0108f80153905f70b532d` | `84B6B3F3` | Glarg |

All four manifest prefixes use table `04DC2F94`, required first ID
`00000001` (raw en-US `Maxis`), and optional third ID `FFFFFFFF`. This is
a candidate join for one definition copy, not 349 resolved names. The
actual actor definition/structure/list handle fixups and row ordering, cache
hit or miss and cached value, RNG state and selection, current `015A8D0C`,
possible save-state override, actor `+FC` triple, mounted manifest winner,
full manifest reader/provider success, locale selection, and `6B4FF0`
processing are all separate admission gates. A cache hit or another actor
construction path can yield a different triple. No actor triple or live
selection is inferred from a disk match.

The four payloads and independent per-stage raw-unit report are retained in
`target/source-cell/manifest-four-candidates-v1` (report SHA-256
`0f2e690bfb98d6a943b632c8b3de84d416a79855960b3d5d08078fa1b4f61309`).
A separately wrapper-built `manifest_four_compare_v1` Rust example
(executable SHA-256 `ec90d37aae28c7a8a7d694cff4866d96499f03d8ab0364efbd90b00c7baeea3e`)
matched all 4/4 prefixes, required IDs and raw units against that report. The
comparison verified the package, table, report, executable and each decoded
payload hash before execution; `rust-comparison.json` records the pins. The
example accepts external payloads and contains no embedded game names. This
is a supplied-input parser comparison, not execution of the native reader.

Reproduce from the workspace with a fresh output directory (the command below
was verified to regenerate the report hash above):

```powershell
python tools\cell_manifest_four_prepare_v1.py --package 'D:\Games\SPORE Collection\Data\Spore_Content.package' --locale target\source-locale\corpus-v2\sources\01169.locale --out target\source-cell\manifest-four-candidates-v1-repro --scope 40616200 A1C46DF2 E219E5D0 E219E5D3 E219E5D2
tools\build.bat build -p spore-core --example manifest_four_compare_v1
python tools\cell_manifest_four_compare_v1.py
```

The compare runner uses the retained v1 report and checks its pinned
executable, so rebuilding must reproduce the pinned executable for that
runner; it does not automatically admit a different build. The preparer SHA
is `eb655ab0ce09de1ecede5f2572c67dbfd452ac9111b5af695e7376b55c1d400c`;
the comparison runner SHA is
`1d2a99161619785c854394e570cdb309d5dc774aef7354b5e529b9f4144128a0`.

After that recorded sample run, the example was changed to require expected
raw strings as external arguments, removing copied game names from source.
The new wrapper-built executable SHA-256 is
`3480046f90137e62da9281d6fd3d78e5f8c8ddfc3931a9ada5fb0a5035749d57`; it
passed against the same retained sample with expectations independently read
from the explicit locale table. The earlier example hash above describes the
previous build and is historical. The unit test now uses invented strings;
all eight focused manifest tests passed after the change.
