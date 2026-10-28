# Jawhead multi-mesh palette trace

This is static source-asset and executable evidence, not a live actor, clip,
shader-upload, or animated-output observation. Executable addresses refer to
`SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.

## Source asset census

[MEASURED] Jawhead LOD0 is `00E6BCE5:0001FFFF:40617100780A9EAF` in
`Data/Spore_Content.package`. Its decompressed payload is 286,918 bytes,
SHA-256 `5a4fcd521ea8a900d880fb9cfc31f7e19b92ae19f9d81d6af50b89559ea2c8a4`.
All offsets below are decompressed payload offsets. Both source mesh records
reference vertex buffer 0: 5,947 vertices, 36-byte stride, bytes
`[60280, 274372)`. Their declaration has stream 0 blend indices at +28,
type 5 UBYTE4, usage 2, and weights at +32, type 8 UBYTE4N, usage 1. Every
vertex's four original weight bytes sum to 255. All nonzero-weight index bytes
are multiples of three.

| Source mesh | Triangle buffer | 16-bit index data | Index count | Referenced vertices | Maximum active index byte |
|---|---:|---:|---:|---:|---:|
| 0 | 0 | `[240, 47976)` | 23,868 | 0–4,800 (4,801) | 189 |
| 1 | 1 | `[47992, 60184)` | 6,096 | 4,801–5,946 (1,146) | 54 |

The two referenced sets partition the shared vertex buffer exactly. The source
8-byte tail descriptors are `(0, 64)` at 274,634 and `(64, 19)` at 274,642;
the count is at 274,630. A hypothetical ordinal association fits the active
indices of each referenced subset (`189 < 64*3`, `54 < 19*3`). It does not
fit mesh 1 if the entire shared buffer is checked against its candidate
19-entry descriptor. These bounds are compatibility observations, not proof
of the association, register remapping, or draw behavior.

The public lossless Rust influence reader now matches the independent census
for both mesh references: 11,894 rows, consisting of the same 5,947-vertex
buffer read once per mesh. The comparison report is
`target/source-cell/gmdl-skin-raw-census-comparison-v1.json` (SHA-256
`f9722d0af01ca1d11201adb371440ac9d452f05e65767d7c2e75a6775f5f04c4`).
This checks all eight stored influence bytes per row, including inactive lanes;
it does not associate a palette or animate the mesh.

The local census `target/source-cell/jawhead-skin-census-v1.json` contains
per-vertex original index and weight bytes, triangle-referenced sets, histograms,
declarations and tail fields. Run
`python target/source-cell/jawhead-skin-referenced-v1.py` from the repository
root to regenerate it using the existing Python package readers. It first runs
`jawhead-skin-census-v1.py` from the same directory. The census and any asset
data remain local under `target/`.

## Native construction and consumer boundary

[MEASURED] `7690A0` reads the descriptor count following material records.
At `769871..769893` it clears and reserves the model vector at `+E0`; at
`7698A8` and `7698BC` it reads the two words, and at `7698DF` it appends the
pair. It then calls the separate part reader `7683E0` at `7698F9`.
This identifies the retained source pair storage; it does not establish its
consumer. REA dossiers: `target/rea/20261028-source-runtime-v6/659` and `650`.

[MEASURED] `73EBA0` can retrieve a GMDL from the resource interface and pass
it to `730090`; `730090` calls `72F6B0`, which forwards five arguments to
`72D520`. The latter groups eligible source mesh records that share a vertex
buffer object into a geometry object and builds draw sections from their index
buffers. Its operation is conditional; no live Jawhead object has been
observed. Dossiers: `648`, `658`, `660`, `661` in the same evidence directory.

[MEASURED] `72D520` calls `729A70` for vertex storage and `729B20` for index
storage. Each returns an existing data pointer when present; its alternative
locks storage and copies the returned bytes into a new allocation. Neither
function visibly remaps index bytes or selects a tail palette descriptor. This
observation is limited to those functions and the bytes exposed by the lock;
it does not establish how the earlier buffer was produced or what later draw
processing does. Dossiers: `662`, `663`.

[MEASURED] At `74328D`, `743250` retrieves geometry objects through `73EBA0`;
at `7432AD`, it gathers a contiguous matrix buffer once through `740530`.
At `743349..743357`, the same stack-buffer base is passed with each object to
`73A100`, which forwards that argument to `731CF0`. `740530` places matrices
at offsets derived from its runtime group records, not an explicit source mesh
ordinal. `71DB60` searches 32-byte geometry stream descriptors: these are
separate from the eight-byte source tail pairs. `734CF0` can perform additional
conditional stream transformations. Dossiers: `622`, `629`, `635`, `636`,
`649`.

[UNKNOWN] The reader establishes `+E0` storage but the relevant draw or shader
consumer of that vector has not been identified. The source pair ordering,
matching bounds, and shared-buffer partition do not independently establish a
per-mesh palette mapping. The unchanged CPU palette base also cannot be
assumed to describe every original GPU draw path. The context at `744A2E`
remains outside a recognized procedure (dossier `651`). Trace readers of the
model `+E0` vector and its connection to draw sections or shader constants
before assigning descriptors. A synthetic explicit-palette diagnostic may
exercise deformation, but cannot establish original palette selection.
