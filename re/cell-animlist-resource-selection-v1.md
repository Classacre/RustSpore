# Cell AnimList and keyed resource selection

Static inspection of the base executable SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`
and the installed packages. These are data and code connections, not evidence
that any Cell actor submitted a request, installed a clip, or played it.

## Executable chain

[MEASURED] `A0EF30` uses resource enumeration with `4AEB6BC6`, opens returned
items, and compares the stream word to `74736C61`; on a match it calls `A0E8C0`.
The latter accepts versions 5 through 10, reads request records and their
variants, and calls `A0E3E0` with the record key to populate the global tree
at `01552A68`. It copies record fields and variant vectors into the node.
Additional entries can compete by the loader's key/float comparison; enumeration
and replacement order have not been established. See REA dossiers 677–678.

[MEASURED] `A0C120` looks up a caller-supplied request key there, selects a
variant and makes triples from each 28-byte entry's +8, +4, +0 words. Its slot
path passes the first triple word to `9A3680`, a lookup in `0166D038`; that path
can use a separate fallback and binding can fail. `9A4310` populates the latter
cache on a miss by `99ED00`, then `9A41F0` inserts or locates a node and stores
the returned pointer. `9A3F00` removes entries and `9A4070` evicts them.
`A08110` offers another concrete path: first variant, first entry +8 is passed
to `9A4310` with a null filename. The selection and installation of an actual
Cell actor are not established by these paths. Dossiers 613–618, 626–628,
693–695.

[MEASURED] When `9A4310` is passed a null filename and nonzero key, `99ED00`
forwards that key to `A088E0`. `A088E0` constructs a three-word request with
key, `EE17C6AD`, zero, and tries a manager's virtual +60 before its resource
manager +58/+34 fallback; it passes a returned stream to `99EAF0`. The latter
reads the stream, accepts `ANIM` magic and versions 6–25, and validates and
prepares the animation object. This establishes a keyed ANIM loading path, not
which mounted package wins. When a filename is supplied instead, `99ED00`
opens it directly; if `9A4310` must derive a key from that filename, `9A9980`
uses `GetFullPathNameA` and `932B30` computes lowercase FNV-1. Dossiers 696–699,
703–705. Resource-type interpretation is also supported by `anim-channels.md`.

[MEASURED] Bounded PE decoding at `A0C4F0` and `A0C520` shows wrappers forward
their two stack arguments and object to `A0C120`; the former additionally calls
`A01B70` with the returned slot and object+184. Their addresses appear as
adjacent 32-bit values at `1449664` and `1449668`. REA did not resolve a
containing procedure for the two call sites or xrefs to the first table entry;
the dispatch table's owner and Cell actor association are unknown. These entry
boundaries follow INT3 padding, and are bounded raw decoding rather than an
independent provider's function discovery.

## Installed source records

[MEASURED] A read-only index scan of 82 installed `.package` files found four
entries of type `4AEB6BC6`, including a PatchData entry with key
`4AEB6BC6:0001FFFF:000082D0142CBA20`, decompressed SHA-256
`df812c5d070e2c3efe6985198e3499f4f74212c9bb901bc0041a3a4bc2e64868`.
It begins with bytes `tsla`, has big-endian version 10 and 1,875 records. A
bounded reader following the version-10 sequence in `A0E8C0` consumed all
342,835 bytes. The other type entries were version 7, version 7 and version 10;
the latter's 656 records also consumed its 201,348 bytes. The version-7 entries
were inventoried but not decoded by this probe. Report and reproducible Python
probe remain local under `target/source-cell/animlist-static-census-v1.{json,py}`;
no source game files were modified. The index reader recognizes three measured
index layouts; zero-type entries cannot establish type-based absence.

The public Rust version-10 reader now matches an independent Python parse of
every ordered field in both installed version-10 resources: 2,531 records
across a scan of 82 package files, with zero reported indexing or parsing
errors. Two version-7 resources were inventoried but not decoded. The report is
`target/source-cell/animlist-v10-compare-v1.json`, SHA-256
`efa652fd7ef7c228ad9ed2b6d4cb1e1b589e65b1f3ca0ec3b5f7c3b29e14066f`.
Four focused Rust tests pass. Strict end-of-payload checks and configurable
resource ceilings are Rust parser policies, not claims about native rejection.

The separate public version-7 reader also matches all ordered fields in the
two installed version-7 resources, totaling 1,888 records in the same 82-package
scan. Its report is `target/source-cell/animlist-v7-compare-v1.json`, SHA-256
`7c30d513a5159c7bb0ecf4a25be3bad00ee49bec0f96f70b6418458e8bf11976`;
four focused Rust tests pass. The source-guided v8/v9 reader passes five
synthetic tests; this installation has no v8 or v9 resource for corpus
comparison. Both scans are subject to the index helper's documented layout coverage and establish
file contents, not mounted-resource precedence or actor playback.

The PatchData data provides concrete candidate mappings; each shown record has
one variant and one animation entry. Offsets are in the decompressed AnimList.

| Record label | Request key | Record offset | Entry +8 / ANIM instance | Entry path |
|---|---:|---:|---:|---|
| `cell_idle` | `046917B8` | 75308 | `59D2678A` | `_final/cell/cell_idle` |
| `cell_idle` | `AAAA0015` | 302800 | `59D2678A` | `_final/cell/cell_idle` |
| `cell_move_jet` | `AAAA0021` | 303646 | `7D23AB3B` | `_final/cell/cell_move_jet` |
| `Cell_mov_cil` | `AAAA0060` | 309671 | `87CB0177` | `_final/cell/Cell_mov_cil` |
| `cell_mov_cil` | `AAAA0091` | 314504 | `87CB0177` | `_final/cell/cell_mov_cil` |
| `cell_move_jet` | `AAAA0092` | 314635 | `7D23AB3B` | `_final/cell/cell_move_jet` |

Each entry's other two words are float bits `3F800000` and `BF800000` and
the variant threshold is `3F800000`. Package indexes contain matching
`EE17C6AD` instance IDs in both PatchData and Spore_Game. For idle and cilia
the indexed group is `0001FFFF`; for jet it is `00010000`. These duplicate
package copies have different sizes; this inventory does not prove either
resource-manager precedence or that the keyed manager returns a particular copy.
The list records alone do not show that their keys reach `A0C120` from an actor.
A byte scan of executable sections found none of the six request-key constants
as little-endian four-byte sequences, which does not exclude computed, indirect,
or data-driven requests.

## Remaining boundary

Trace the owner of the `1449664`/`1449668` dispatch slots and the Cell-side
caller or data-driven state transitions to identify its request argument. Then
resolve AnimList enumeration/priority and resource-manager selection, including
all relevant mounted packages, and confirm the chosen resource reaches binding
and a controller slot. Runtime scheduling and playback remain separate gates.
