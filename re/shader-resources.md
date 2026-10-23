# Source shader libraries and packs, 2026-10-22

## Measured population and limits

`[MEASURED]` All 82 installed package indexes expose 36 resources of type
`0469A3F7` (.smt). There are 12 files in each of three distinct header families:
header 0 material links, header 1 HLSL fragment libraries, and header 7 compiled
shader packs/builders. Their instance prefixes are respectively `40212000`,
`40212002` and `40212004`. Each family has four profile files in Graphics,
PatchData and EP1 Data. These are stored resources, including overlay/profile
duplicates, not counts of unique shader programs used at runtime.

`[MEASURED]` The independent Python probe and Rust decoder agree on all 12
fragment library records and all 12 shader pack records. Re-encoding each typed
resource reproduces the entire original payload. The fragment digest SHA-256
(sorted records, LF endings) is
`f7b4c30062f73f40caacae6bea421b7439d30279102ea58e0dbac777a233f5d3`.
The pack digest SHA-256 is
`1595eb958a876132e60b6ed09f64f9c064e188e9b6663588ac4a7ace2a5b38d6`.
Detailed corpus results are retained in ignored `target/source-shaders/`.
Raw source HLSL and D3D9 programs stay there or in the
user's immutable install; none is embedded in the public Rust implementation.

`[UNKNOWN]` Exact framing does not establish correct shader selection,
conditional fragment evaluation, sampler/render-state binding, or the original
material's rendered appearance. The source Cell preview still uses Bevy PBR and
is explicitly an asset inspection path. It cannot close a game fidelity gate.

## Header 1 fragment libraries

`[MEASURED]` The stream is big-endian except for uninterpreted payload bytes:

```text
u32 header = 1
255 vertex records
256 pixel records
end of file

vertex record:
  15 raw metadata bytes
  text code, text declarations
  u32 uniform_count
  uniform_count * { text declaration, 12 raw metadata bytes }
  text name if metadata_flags & 2

pixel record:
  14 raw metadata bytes
  text code, text declarations
  u32 uniform_count
  uniform_count * { text declaration, 12 raw metadata bytes }
  36 raw texture metadata bytes
  text name if metadata_flags & 2

text: u32 UTF-8 byte length, that many bytes
```

`[MEASURED]` The 255 vertex slots versus 256 pixel slots are observed in all 12
files. Empty slots are retained. There are 2,044 named vertex records and 1,832
named pixel records across that stored population. Slot indexes exposed by the
Rust API are zero-based storage positions. Native vertex IDs start at 1, while
pixel storage IDs start at 0; the lookup methods preserve that difference.
See `re/shader-selection.md`. Uniform metadata can be read as four big-endian u16 words
followed by a u32; individual fields remain raw until corroborated.

`[MEASURED]` Native readers `6FF150` and `6FC370` independently confirm the
255/256 fixed arrays, uniform metadata widths, and the optional name flag in the
last metadata word. The source records all have that flag set, including empty
slots. Rust and Python also support its absence, which is exercised by synthetic
fixtures. Native Evidence queries 193 and 194 retain the original measurements.

`[MEASURED]` The source skinning fragments declare 192 float4 entries for bone
data and access three rows per matrix. This independently supports the observed
three-register bone indexing in source vertex declarations and the native
palette writer. A source uniform with ID `021C` declares two float4 entries,
corroborating the measured 32-byte GMDL shader-data payload. These are binding
constraints, not proof that generated creature palettes are ready for playback.

## Header 7 shader packs

`[MEASURED]` The full population contains 3,136 standard shader records, 1,040
builders, 11,064 compiled vertex programs, and 3,291 compiled pixel programs.
These counts include all package/profile copies. All 12 fit this framing:

```text
u32 header = 7
u32 standard_count
standard_count * standard_shader
u32 builder_count
builder_count * builder
compiled vertex table
compiled pixel table
text pack_name
end of file

shader_header: seven big-endian u32 words, text name if last_word & 0x10
standard_shader:
  shader_header
  repeat { u8 render_type, blob VS, blob PS,
           u32 VS_uniform_count, VS_uniform_count * 12 raw bytes,
           u32 PS_uniform_count, PS_uniform_count * 12 raw bytes }
  u8 FF terminator
builder:
  shader_header
  repeat { u8 render_type,
           u32 vertex_count, vertex_count * 25 raw selector bytes,
           u32 pixel_count, pixel_count * 25 raw selector bytes }
  u8 FF terminator
compiled table:
  u32 header_word, u32 program_count
  program_count * {
    32 raw signature bytes
    blob D3D9_bytecode
    u32 uniform_count, uniform_count * 12 raw metadata bytes
    uniform_count * u32 associated_words
    u32 flags_word
  }
blob: u32 byte length, that many bytes
```

`[MEASURED]` Both compiled table header words are zero in all source packs.
They are not stage enums. D3D9 token words inside blobs are little-endian. The
source population contains vertex profiles 2.0 and 3.0, and pixel profiles 2.0,
2.x (`FFFF0201`) and 3.0. The trailing names identify a profile and pack hash.
Do not drop signature padding or reconstruct it from presumed fragment IDs.

`[MEASURED]` Named source shader IDs include `70000060` (`cellGameShader`) and
builder `80000004` (`generic_skinning`). `CellSkinDrawShaderSH` is `700000B8`
in PatchData/EP1 but `700000B3` in Graphics; Graphics `700000B8` instead names
`genericTileAdditiveParticle_shader`. Therefore the package/profile provenance
must accompany every shader ID. These names identify source records, not the
shader chosen for every Cell draw. That requires the actual material/state path.

`[MEASURED]` Focused native reader `711880` accepts versions 6 and 7, then invokes
`6E5710` for standard shaders, `6FD170` for builders, and `6FEDF0`/`6FBDE0` for
compiled tables. The shared header reader `7776F0` confirms the optional name
flag. Selector reader `6FC9C0` confirms a byte, three big-endian u16 words, four
big-endian u32 words and a trailing byte; the caller reads the fragment byte
separately. Version 6 consumes additional discarded fields, so its selector
framing differs. The native byte-swapping reader corroborates big-endian outer
framing. Evidence is in ignored `target/rea/20261022-anim-rig-finalize-v2/`,
queries 163 through 166 and 173 through 174. Version 6 is deliberately
unsupported by the new Rust parser because no installed fixture was measured.
The executable SHA-256 is
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

## Reproduction and remaining work

```powershell
python tools/shader_resource_probe.py '<install>' --out target/source-shaders
python tools/shader_fragment_probe.py '<extracted header-1 file>' --out '<new JSON>'
python tools/shader_pack_probe.py '<extracted header-7 file>' --out '<new JSON>'
python tools/shader_pack_probe.py target/source-shaders --out target/source-shaders/python-pack-corpus-v2.json
tools/build.bat test -p spore-core shader_fragments
tools/build.bat test -p spore-core shader_pack
tools/build.bat run -p spore-bevy --example shader_fragments_verify -- '<install>' target/source-shaders/rust-fragment-digests-v1.tsv
tools/build.bat run -p spore-bevy --example shader_pack_verify -- '<install>' target/source-shaders/rust-pack-digests-v1.tsv
```

`[MEASURED]` Three fragment tests and four pack tests pass, including mixed
stages/tables, exact preservation, malformed strings, count bounds, every
truncated prefix, optional native names, repeated render types and trailing bytes.
Fresh Rust v2 corpus digests after the native optional-name correction agree with
the same independent source records and hashes. The verifiers refuse
to overwrite their output paths; choose a fresh numbered path when repeating.

`[MEASURED]` Header-0 material records now account for all companion RW4 state
sections: 5,120 material records, 5,132 states and 6,170 texture references across
12 libraries, with independent Python/Rust digest agreement. This preserves their
association and opaque payloads; it does not yet execute a state on the GPU.

`[MEASURED]` A separate native Rust probe now executes the original standard UI
programs on the RTX, with exact RGBA agreement for two directly read source PNGs.
That is a controlled draw, not execution of the material-state library or a full
SPUI screen. See `re/shader-execution.md` and `re/material-links.md`.

`[MEASURED]` The eight selector conditions found in this install now have a
native-backed evaluator and focused regression checks. Its caller must still
supply actual draw bindings; see `re/shader-selection.md`.

`[UNKNOWN]` Next: bind actual Cell GMDL materials to their runtime states,
recover input/constant layouts, and implement the actual
source Cell draw path. Faithful lighting, painted channels, transparency,
effects and the game camera require separate source comparisons.

Public format terminology was cross-checked against the community's
[fragment documentation](https://github.com/Spore-Community/SporeModder-FX/wiki/Shader-Fragments-(.vertex_fragment,-.pixel_fragment)),
[builder documentation](https://github.com/Spore-Community/SporeModder-FX/wiki/Shader-Builders-(.shader_builder)),
and [compiled-state documentation](https://github.com/Spore-Community/SporeModder-FX/wiki/Materials-and-Compiled-States-(.smt_t-and-.rw4)).
No third-party implementation was copied. The population, byte framing and
corroborating native evidence above were checked against the user's install.
