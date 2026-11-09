# Cell skybox effect-directory candidate

The source Cell initialization passes effect ID `05C7A4D0` and group zero
to its skybox effects world. The bounded lookup trace compares both ID and
group. Whether the candidate directory is actually loaded and selected at
runtime remains unverified.

Six explicit installed EFFDIR v4 payloads contain that export. Each points to
visual 4930, whose type-9 block refers to screen component 186; that record
contains texture group zero and instance `BD08BD8A`. A separately located RW4
with the same low instance contains a 256×256 DXT5 raster with nine mip levels.
The texture provider's group routing and actual rendering are not yet proven.

The public `cell_effdir_framing_v3` reader matched an independent audit on all
36 effect directories in the four inspected base Data packages, including all
six candidate exports; seven focused tests passed. It validates section framing
and selected screen records while retaining other section bodies as opaque bytes.
Unsupported assignment framing is rejected. The comparison report is retained
under ignored `target/source-cell/effdir-full-corpus-compare-v1/report.json`
(SHA-256 `f4f64c78b3b249469d8476ae7420f9c0b636a223cfceb16c4a70d9769966dc4d`).
Expansion and other packages have not been included in that count.

Format facts and their external references are recorded in
`tools/cell_skybox_effdir_audit_v1.py`; no original game payload is distributed.
The independent review of that audit confirms the six candidate records but does
not establish package precedence or validate individual records in opaque sections.

The source loader `7E9730` clears previously registered directories, requests
each explicit ID using type `EA5118B0` and group `40450200 | variant`, and
registers successful directory objects through the manager. It then enumerates
other supplied group templates and registers successful loads. Manager lookup
walks registered directory slots in order and compares the full export ID/group
pair. The runtime configuration variant, provider ordering, and overlay
precedence remain to be established. This trace does not establish
that any one installed candidate is the active skybox.

The loader constructor defaults the explicit list to `4CF9B596` and the
enumeration list to empty. The application startup path subsequently calls the
loader's list setter with five explicit IDs in order: `4CF9B596`, `2C978DA4`,
`BAFC5CA1`, `5F848353`, and `C1195706`; it also supplies four group templates.
Across the inspected base copies and variants 0, 1, and 2, only the six
`C1195706` payloads have the selected screen record. These static startup
observations do not prove successful loads, provider ordering, or an active
Cell scene. The configuration watcher starts at 1 but may obtain another
runtime value before the initial directory load.

The watcher is the 16-byte object at owner +88. In the observed constructor,
`7E7403..7E7450` supplies default `1`, property key `03EBEECC`, and object
address `0163A450` to `7E6FA0`. That initializer stores the default at watcher
+0, the supplied object at +4, the key at +8, and zero at +0C; if the object
is non-null it calls `7E6460`, which queries its virtual +24 and can replace
the default with a property word. Thus default 1 is not an established initial
loaded value. The initialized object, its returned property, and its lifetime
are runtime dependencies.

On the conditional update path `7E9DEE..7E9E03`, `7E63E0` is called on owner
+88 and invokes `7E9730` only when it returns true. `7E63E0` compares a
revision-like sum derived from provider +30 and +34 to watcher +0C; on change
it updates +0C, queries virtual +24 with watcher key +8, resolves a word through
`4E4570`, and writes watcher +0 only when the returned word differs. Failure
to query or an unchanged value returns false. This is conditional update
ordering, not proof that the update path ran, that the first directory load
occurred at this call, or that the property name or value is known.

`7E97B0..7E97BB` constructs each explicit key's group as
`40450200 | (watcher & FF)`, while `7E98C4..7E98D3` replaces the low byte of
each enumerated group template with `watcher & FF`. For example, admitted
watcher values 0, 1 and 2 give explicit groups `40450200`, `40450201` and
`40450202`, and template `408A4300` becomes `408A4300`, `408A4301` and
`408A4302`. Higher watcher bits do not affect those two keys. The fixed-byte
source projection and boundary examples are in
`target/source-cell/skybox-variant-key-audit-20261108-v1.json`, SHA-256
`36248853bb5c46222af571ed31e83824492c6c7173476b86f86dd5f54cf8be90`; the
script is `tools/cell_skybox_variant_key_audit_v1.py`. It is a static arithmetic
audit, not an execution comparison or a selected package. Each successful
explicit lookup is registered before the later enumerated templates; failures,
list content, provider ordering/cache and manager state still affect the
result. The manager's active material and camera remain unresolved.

The watcher's supplied address has a concrete conditional implementation.
Static initializer `12F1DD0` calls `6A1750` on `0163A450` with `52h`, and
registers `13C0030` as a cleanup callback; `6A1750` installs table `01408818`,
initializes +30 and +34 to zero, and allocates the supplied number of direct
slots. On the startup path `7E88FD` the same object is published through
`67DB70` to `015FE918`. Under that installed table and unchanged object,
virtual +24 is `6A2370`: keys below the direct-slot count go through virtual
+28, while `03EBEECC` exceeds the initial `52h` count and goes to `6A1FE0`.
That routine searches the local 24-byte-entry range +18..+1C first; on a miss
it calls the object at +30 through virtual +24 if non-null, otherwise returns
failure. This establishes local-before-parent lookup for this implementation,
conditional on the current count, parent pointer, object identity and intact
table. It does not establish which source populated either collection, the
parent's implementation, or the value returned at construction or update.
The watcher's constructor retains the supplied interface and the registered
cleanup path invokes `6A1680`; dynamic replacement and destruction order are
not resolved. A bounded decoded evidence file is
`target/source-cell/skybox-property-provider-audit-20261108-v1.json`, SHA-256
`482151862fa2cc3b2822891ffbaa4af0fbccb77268ef81bdb4652262636af9ba`.
Table +38 reaches `6A2A10`: on one conditional input branch it releases the
old +30 interface, calls the object returned by `67D950` at virtual +2C with
the address of +30, and then walks the local 24-byte entries while calling
`93A3F0` and `693EC0`. The supplied input and factory outcome are not yet
identified. These static operations indicate a possible parent/local
population route; they do not establish that this global object took it.

An explicit scan of installed `Data/Config/Properties.txt`,
`ConfigManager.txt` and `VideoCards.txt` found neither a literal `03EBEECC`
nor an ASCII identifier token whose lowercased FNV-1 multiply-then-XOR hash is
that key. The file hashes and search boundaries are recorded in
`target/source-cell/skybox-property-config-audit-20261108-v1.json`, SHA-256
`71108696739090424aa39c8f94ee83cec4eee6ea5abefed7435b319e26965e2b`.
An exact textual search of the retained 82-package index census
`target/native-archive-index-census-v1/records.tsv` (SHA-256
`3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6`)
also found no `03EBEECC` row. Neither check searches property IDs inside
package payloads, transformed names, other config files, user preferences or
dynamically populated values; they cannot select a background variant.

A subsequent bounded payload scan of all indexed `00B1B104` PROP resources in
`PatchData`, `Spore_Game`, and `Spore_EP1_Data` examined respectively 11,460,
7,030, and 4,738 decoded payloads, with no errors and equal per-package hashes
before and after. Each package contains four byte-identical rows in index group
`00010000` whose low instance word is `046170A0`: upper words `40470101`,
`40470102`, `40470103`, and `4047010B` contain scalar INT32 property
`03EBEECC` values 0, 1, 2, and 4 respectively. Each also contains an
`08EA246B` row with this scalar set to 1; those three copies are not identical.
All matches were parsed as property entries rather than accepted on raw bytes
alone. Full keys, decoded hashes, offsets, and package hashes are retained in
`target/source-cell/skybox-property-payload-scan-20261108-v1.json`, SHA-256
`6cc02b4db5e28c27558d73288748b0d63ff3da7adc7bcef5f755153288af47b7`.
The reader reopens package paths and requires them to remain stable; the
three-package scope does not prove completeness or overlay order.

Installed `Data/Config/Properties.txt` explicitly declares `OptionEffects`
as `046170A0`. Its `ConfigManager.txt` initially sets `OptionEffects` to
`$High`, later sets it to `$cpuLevel`, and contains conditional Medium/Low
overrides. These texts and the matching candidate resource keys connect the
option name and records, but they do not by themselves establish an effective
selected option or active property value.

Bounded x86 decoding gives a conditional merge route. `7CCB10` starts with
type `00B1B104` and group-like word `40470000`, requests a resource through
`67D7F0`, then calls the `67D950` property manager's +2C. On a path through
its option arrays, `7CCC5C..7CCC6C` carries an array-supplied option ID into
the request, while `7CCCD4..7CCD0C` forms `40470100 | (index & FF)` and asks
that manager's +2C for a corresponding list. Thus an admitted option ID
`046170A0` and indices 1, 2, 3 or 0B match the four explicit installed keys
above. `7CCF89..7CCFB7` can attach and retain the current global `015FE918`
at the option object's +B0. `7C9120` uses each option's current selection
word +8, and calls `7C8370`; when the selected index is in bounds and its
list pointer is non-null, `7C83AE..7C83BA` invokes +30 on the attached +B0
object with that selected list. Under the separately established global
table `01408818`, +30 is `6A10B0`, which iterates the supplied list's local
24-byte entries, calls receiver virtual +14 for each, and increments the
receiver's +34 revision. This supports a source-backed **conditional** route
from a loaded option list into the watcher provider. It still requires the
option array contents, index, successful loads, receiver identity/table,
application order, parent/overlays and live configuration; none of these
traces alone identifies the active Cell variant or camera.
The bounded instruction listings and source bytes for this merge are retained
in `target/source-cell/skybox-option-merge-audit-20261108-v1.json`, SHA-256
`ce7e4b0b9190d3f6eae8306e2ec7a128346bd5a7e91f31df1aab321d63fd13b9`.

The three explicit installed catalogs indexed as type `00B1B104`, instance
`4047000000000000` were independently decoded. The `05DAAFFF` option-ID array
contains `046170A0` at index 3 in PatchData and index 4 in Game and EP1 Data;
the corresponding words of `05DAB000` and `05DAB001` are 1 and 12 in all
three. Their payloads and array lengths differ, so those array positions must
not be shared across an unchosen catalog. For this option, the source loop at
`7CCC78..7CCD0C` uses the supplied catalog words as the lower start and upper
exclusive bound when it attempts option-list lookups. The installed catalog
projection and package endpoint hashes are recorded in
`target/source-cell/skybox-option-catalog-audit-20261108-v1.json`, SHA-256
`d301fd85f8516c19ac5827664b3ae4677c5bd1ba5d4a87d80019f20947336b54`.

On the identified setup path `7C9120` reads `015FE91C`: if that object is non-null and has a
successful +24 lookup for the option ID with type `000A`, the extracted word
becomes option +8; on lookup/type failure the catalog's option +4 default is
used and written through the same preference object's +14. It then calls
`7C8370` to conditionally merge the selected loaded list. `7E6AD6..7E6ADD`
publishes owner +CC to `015FE91C` and `7E943F..7E945D` clears the global and
releases owner +CC on the cleanup path. Earlier `7E684A..7E68AF` can populate
owner +CC from a successful conditional load; `7E6982..7E69E0` constructs an
empty list if none remains. Actual file/source identity, version checks,
successful path, preference contents, and later writes are not established.
The catalog default 1 is a fallback admission, not the active setting.

For a future controlled source integration, an explicit option value may be
joined to one pinned catalog and same-key list, merged with caller-supplied
provider state, and projected to the low-byte directory key. It must preserve
lookup failures and list ordering and label configuration, provider precedence,
screen material and camera as controlled inputs until observed.

On 2026-11-08 a read-only, single-file inspection found line 3
`OptionEffects 2` in
`%APPDATA%\Spore\Preferences\Preferences.prop`.
The 896-byte file SHA-256 was
`cc1bb5dc49d2bfa7c475abf5b5b08b213d59f04ea408ac76b887fabb4917fa6a`;
its last-write timestamp was 2026-10-22 12:31:36.599728 UTC. The selective
audit retained only this value and metadata in
`target/source-cell/skybox-persisted-option-audit-20261108-v1.json`, SHA-256
`a37fa015d2051c17372443deca368c305a4acc729db598804e57e9ed36f1a72c`;
file size and timestamp agreed across the read, but this does not prove
continuous stability. No live game was observed. This file may be stale and
must not be represented as the current in-memory setting.

The retained `7E64E0` native dossier
`target/rea/20261022-source-runtime-v3/546-analyze_function.json` shows a
conditional path preparing resource words `D9BD3CA1:00B1B104:011AC192`,
attempting a load outside the `safe` path, checking preference version
`04754439` against 9, and publishing the surviving or constructed object via
`67DA30` to `015FE91C`. Its fallback requests default preferences. `D9BD3CA1`
also equals the source's lowercased FNV-1 hash of `Preferences`. This supports
a preferences-loader connection but does not establish that this particular
filesystem path was the loaded object, that its saved value survived validation
or later overrides, or that the conditionally selected list was applied.
If explicitly admitted as the current option index, **2** joins to the
installed `40470102046170A0` list's watcher value 1 and hence directory group
`40450201`; that is a controlled projection, not a runtime observation.

`tools/cell_skybox_controlled_option_join_v1.py` composes this projection from
an explicitly hash-admitted preferences file, option value, source executable,
ordered package list, and per-resource index groups. It rejects ambiguous
preference entries, duplicate exact keys within a provider, unsupported option
bounds, non-unique or non-scalar watcher properties, ambiguous screen
components, wrong candidate raster bytes and changed input endpoint hashes.
It requires explicit stable-path admission because package index and payload
reads reopen files. It does not apply this result to gameplay.

One controlled headless run used the saved value 2, caller-selected first-match
order PatchData, Spore_Game, then Spore_Graphics, and explicit catalog/option/
directory/raster index groups `0001FFFF`, `00010000`, `0001FFFF`, `00010000`.
It selected the PatchData catalog and option-2 list, watcher value 1, PatchData
directory `40450201C1195706`, screen component 186 and request
`BD08BD8A:2F4E681C:00000000`; the separately requested B-type candidate came
from Spore_Graphics. The receipt is
`target/source-cell/skybox-controlled-option-join-20261108-v1.json`, SHA-256
`011261460996a032bbbf694874f738f1b3ef3267526f146eeffffbf75848548d`.
This direct B-type lookup does not exercise or override the original C-type
request's alternative ordering, native converter or cache. Supplied ordering,
successful merge, lack of later watcher changes and file stability remain
admissions; current runtime selection, actual material and camera remain
unresolved. `python -m unittest discover -s tools -p
test_cell_skybox_controlled_option_join_v1.py` passed two focused tests.
