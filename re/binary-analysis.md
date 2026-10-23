# Static analysis of the SPORE executables

Status: original measurements 2026-10-16; environment recovery recorded 2026-10-22 in section 10.
Evidence grades: `[MEASURED]` read directly from bytes, `[INFERRED]` reasoned from
measurements, `[UNKNOWN]` not established.

Raw dumps this report is built from: `re/data/pe-structure-raw.txt`,
`re/data/pe-meta-raw.txt`, `re/data/strings-classified.txt`. Parsers: `tools/pe_meta.py`,
`tools/pe_sections.py`, `tools/pe_strings.py`.

## 1. Targets

| | Base game | Galactic Adventures (EP1) |
|---|---|---|
| Path | `SporeBin\SporeApp.exe` | `SporebinEP1\SporeApp.exe` |
| Size | 20,447,744 bytes | 24,888,320 bytes |
| SHA-256 | `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a` | `922af7c7908af2614b9caca2ffb1ec2353442022fd124866105724acaa24c26e` |
| MD5 | `144613253b97a2e2e2e463936360254a` | `5acb26fc0d340e0cef44419636419d33` |
| Format | PE32 (`0x10b`), x86 | PE32 (`0x10b`), x86 |
| Build timestamp | 2009-07-25 00:58:49 UTC | 2009-07-25 00:58:06 UTC |
| Linker | 9.0 | 9.0 |

`[MEASURED]` Both are 32-bit PE32 images with image base `0x400000`. The build
timestamps are 43 seconds apart on the same day, so these are two outputs of one build
run, not a 2008 release and a later patch.

## 2. Sections

Both binaries have the same 7 sections with the same names.

| Section | Base VA | Base raw size | EP1 raw size | Base entropy |
|---|---|---|---|---|
| `.text` | 0x00001000 | 0x00fcb000 | 0x00fcb000 | 6.6369 |
| `.rdata` | 0x00fcc000 | 0x00140200 | 0x00140200 | 6.2780 |
| `.data` | 0x0110d000 | 0x000c4c00 | 0x000c4c00 | 3.7190 |
| `.tls` | 0x01320000 | 0x00000200 | 0x00000200 | 0.0000 |
| `CONST` | 0x01321000 | 0x00000200 | 0x00000200 | 0.5417 |
| `.rsrc` | 0x01322000 | 0x000cd400 | **0x00506c00** | 5.7916 |
| `.reloc` | 0x013f0000 | 0x000e2800 | 0x000e5200 | 6.2264 |

`[MEASURED]` `.text` virtual size differs by 288 bytes (`0x00fcae0a` vs `0x00fcaf2a`) and
`.rdata` by 168 bytes. The one large difference is `.rsrc`, which grows from 840,704 to
5,270,528 raw bytes. The resource directory holds 22 icons in the base build and 252 in
EP1.

`[INFERRED]` The expansion adds almost no executable code to the shipped engine binary;
its size increase is almost entirely icon resources. This is consistent with SPORE's
expansion content being data-driven, delivered through the `.package` archives rather
than through a new engine build. It is also consistent with the executable being the
2009 patched engine in both cases.

## 3. What the executables depend on

`[MEASURED]` 20 imported DLLs, 495 imported functions in total.

| DLL | Imports | What it implies |
|---|---|---|
| MSVCR90.dll | 159 | Visual C++ 2008 (VC9) runtime, dynamically linked |
| KERNEL32.dll | 148 | base Win32 |
| USER32.dll | 89 | windowing and input plumbing |
| WSOCK32.dll | 23 | legacy Winsock |
| IMM32.dll | 12 | IME, for non-Latin text entry |
| ADVAPI32.dll | 12 | registry and security |
| d3dx9_27.dll | 9 | Direct3D 9 helper library, the August 2007 D3DX build |
| dbghelp.dll | 7 | crash dump generation |
| GDI32.dll | 6 | GDI, used for text and 2D surfaces |
| ole32.dll | 6 | COM |
| VERSION.dll | 5 | version resource queries |
| WINMM.dll | 3 | multimedia timers |
| steam_api.dll | 3 | Steam integration |
| SHELL32.dll | 3 | shell integration |
| WS2_32.dll | 3 | Winsock 2 |
| USP10.dll | 2 | Uniscribe text shaping |
| DSOUND.dll | 2 | DirectSound |
| d3d9.dll | 1 | `Direct3DCreate9` |
| DINPUT8.dll | 1 | `DirectInput8Create` |
| PSAPI.DLL | 1 | process memory queries |

`[INFERRED]` The rendering stack is Direct3D 9 through `D3DX9_27` (D3DX from the
September 2007 DirectX SDK). There is no D3D10 or D3D11 import, so the shipped renderer
is D3D9 only.

`[MEASURED]` Combined with the asset evidence in `docs/dbpf-format.md`, where type
`0x2F4E681B` (RenderWare 4) dominates `Spore_Graphics.package`, the graphics layer is
D3D9 underneath a RenderWare 4 scene and asset layer. Two independent sources, the import
table and the asset type histogram, point the same way.

`[INFERRED]` Audio is DirectSound plus the WinMM timer, with no Miles Sound System or
Bink import. If those libraries are used they are loaded dynamically, which the import
table cannot show.

`[MEASURED]` Networking is present (WSOCK32, WS2_32) and Steam is integrated
(steam_api.dll). `[INFERRED]` This matches SPORE's online Sporepedia and asset-sharing
backend.

## 4. Security and runtime properties

`[MEASURED]` `DllCharacteristics = 0x8140`, which is `DYNAMIC_BASE` (0x40) plus
`NX_COMPAT` (0x100) plus `TERMINAL_SERVER_AWARE` (0x8000). So ASLR and DEP are requested
and Control Flow Guard is not.

`[MEASURED]` There is no `LOAD_CONFIG` data directory (RVA 0, size 0), so there is no CFG
function table and no SEH table in the load config.

`[MEASURED]` There is no `EXCEPTION` data directory. This is normal for x86, which uses
frame-based SEH rather than the table-driven unwinding that x64 uses.

`[MEASURED]` There is no export directory, so the executable exports nothing.

`[MEASURED]` A TLS directory is present with callbacks. A Ghidra import log from an
earlier run reported callback addresses `0x13eb0b4`, `0x1000e4a4`, `0x100057cc` and
`0x100295f8`. `[UNKNOWN]` What those callbacks do.

`[MEASURED]` `steam_api.dll` in the install directory is 15,360 bytes. `[INFERRED]` A
real Steamworks `steam_api.dll` is roughly 200 KB or more, so this is very likely a stub
or an emulator shim rather than the genuine Valve library. `[UNKNOWN]` Its contents were
not analysed. This matters if Steam functionality is ever to be reproduced.

## 5. Toolchain fingerprint from the Rich header

`[MEASURED]` Both binaries carry a Rich header with the same XOR key `0xd05b2bcd` and the
same 16 entries with the same counts. The entries name several MSVC tool generations:

| Product | Build | Count |
|---|---|---|
| Utc1500_CPP (VC9, VS2008) | 30729 | 1880 |
| Import0 | 0 | 558 |
| Utc1310_CPP (VC7.1, VS2003) | 3077 | 546 |
| Utc1500_C (VC9) | 30729 | 461 |
| Implib800 | 50727 | 33 |
| Masm900 | 30729 | 24 |
| Implib710 | 4035 | 8 |
| AliasObj900 | 20413 | 5 |
| Utc1400_C (VC8) | 50727 | 5 |
| Utc1400_CPP (VC8) | 50727 | 5 |
| Implib900 | 30729 | 2 |
| Cvtomf600 | 8447 | 1 |
| Utc1310_C | 4035 | 1 |
| Cvtres900 | 21022 | 1 |
| Linker900 | 30729 | 1 |
| Utc1310_CPP | 4035 | 1 |

`[INFERRED]` The bulk of the code was compiled with VC9 (VS2008), matching the linker
version 9.0. The VC7.1 and VC8 objects are almost certainly prebuilt static libraries
(third-party middleware and older EA internal code) linked in without recompilation,
which is a normal pattern.

`[MEASURED]` The Rich header checksum does not validate in either file (`computed
0xc6b82d0c`), so the header may have been altered after linking. `[UNKNOWN]` Whether that
is benign or a sign of tampering.

## 6. Do the two executables share code?

This question matters because the cheap answer, "the section sizes match, so they share
code", turns out to be wrong, and it is worth recording why.

`[MEASURED]` `.text` compared byte for byte over its full 16,560,128 bytes:
**15,560,362 bytes differ, which is 93.96%**. Only 425 of 131,072 sixteen-byte windows
match at the same offset.

`[MEASURED]` Testing byte shifts of +1, +2, +3, +4, +8, +16, -1, -2 and -4 does not
reduce the difference; every shift gives between 95.70% and 96.18%. So the difference is
not an alignment artefact.

`[INFERRED]` 93.96% differing means 6.04% of bytes match. Two unrelated byte streams
would match about 0.39% of the time by chance. So the two `.text` sections are far more
alike than random but nowhere near identical: the builds are related, but the code is not
the same code.

`[MEASURED]` At the same time the two files have identical import tables (same 20 DLLs,
same per-DLL import counts) and byte-identical Rich headers.

`[INFERRED]` The combination points to two builds produced by one build configuration
from related but not identical sources, or from the same sources compiled with different
settings. The metadata cannot distinguish those cases.

`[UNKNOWN]` Whether the code difference is source-level or codegen-level. Resolving it
needs symbol or decompilation evidence that this environment could not produce, see
section 8.

## 7. Resource directory

`[MEASURED]`

| Type | Meaning | Base entries | EP1 entries |
|---|---|---|---|
| 3 | RT_ICON | 22 | 252 |
| 14 | RT_GROUP_ICON | 2 | 2 |
| 16 | RT_VERSION | 1 | 1 |
| 24 | RT_MANIFEST | 1 | 1 |
| (named) | DATA | 2 | 2 |

`[UNKNOWN]` The exact product version string in RT_VERSION was not extracted. That would
be the cleanest way to state which patch level each executable is.

## 8. Limits of this analysis

The following records the limits of the earlier runs. Section 10 records the later
environment investigation; the 330-second deadline is configurable, not a fixed limit.

1. **No decompilation.** The reverse-engineering MCP backend runs Ghidra headless and its
   startup deadline is 330 seconds. Auto-analysis of a 20 MB MSVC binary exceeded that on
   three attempts, each ending in `Ghidra startup deadline elapsed` while still mid
   analysis. The third attempt used the MCP directly rather than a headless script:
   `open_binary` accepted the file and returned its identity (PE, x86, SHA-256
   `17cf32b3...`), then `binary_overview` timed out at 330000 ms with
   `failure_kind: timeout`, `failure_message: "Ghidra startup deadline elapsed"`. The log
   shows import and library linking had completed and `ANALYZING all memory and code` had
   begun. So the limit is the provider's own deadline during auto-analysis, not the calling
   script, and it is not something a smaller request can work around: the first request
   after `open_binary` is what triggers analysis. The import table in section 3 came from
   the import log of those runs, which is why it is detailed. No pseudocode, no call graph
   and no function list was obtained.
2. **Sparse RTTI.** The separate byte scan in `tools/pe_rtti.py` found only two distinct
   RTTI names per binary (`re/data/rtti-types.json`), rather than a useful engine-class
   inventory. Engine names were instead mapped from registration strings in
   `docs/engine-architecture.md`. Completed Ghidra analysis must not be assumed to recover
   a class hierarchy that the executable does not supply.
3. **No symbol or PDB data.** Ghidra reported no PDB file found in its search paths, and
   neither binary has a symbol table (`PointerToSymbolTable = 0`, `NumberOfSymbols = 0`).
4. **No dynamic analysis.** Neither executable was run. Everything here is static.
5. **The string inventory** in `re/data/strings-classified.txt` is extracted and is the
   only route into the engine that works at this file size: `tools/pe_engine_map.py` turns
   it into `simulator-classes.txt`, `argscript-functions.txt`, `console-commands.txt` and
   the RenderWare names. Section 4 stands on that, not on decompilation.

The honest summary: the container format in `docs/dbpf-format.md` is on solid measured
ground, but the engine internals are largely unexplored, and the blocker is tooling
throughput rather than absence of method.

## 9. Reproduction

```bash
PY=C:/Users/nieuw/.workbuddy-ai/binaries/python/envs/default/Scripts/python.exe
$PY tools/pe_meta.py        # headers, Rich header, imports, resources
$PY tools/pe_sections.py    # section table and entropy
$PY tools/pe_strings.py     # classified string inventory
```

## 10. Environment recovery, 2026-10-22

`[MEASURED]` REA MCP 6.5.0 was already installed and answered `binary_session`.
Ghidra 12.1.4 exists at `C:\Program Files\ghidra_12.1.4_PUBLIC`, with its Windows
decompiler and headless launcher; Java 21 meets its declared requirement. The running
desktop environment lacked `GHIDRA_INSTALL_DIR`, although the user-scope value existed.

`[MEASURED]` Installed REA 6.5.0 supports `REA_GHIDRA_STARTUP_TIMEOUT_MS`; 330000 is
the default, not a fixed cap. The Codex REA environment now explicitly sets the installed
Ghidra path and a 1200000 ms startup budget. The two-setting change was backed up and
validated as TOML, preserving all other settings. Existing MCP processes need reconnection
to receive new environment values; `tools/rea-spore.ps1` starts a fresh CLI process with
the values immediately.

The runner keeps complete JSON Evidence and query snapshots under ignored `target/rea/`.
These snapshots cache observations, not a persistent Ghidra database. Retain an MCP session
for successive queries when possible, because separate CLI calls can repeat auto-analysis.

```powershell
.\tools\rea-spore.ps1 -Operation trace -Query 'SPSkinPaintFloodEffect'
```

`[MEASURED]` A new full-default-profile run against the same base-game SHA-256 still
timed out at **1200000 ms**, during the first `search_strings` inside `trace_feature`.
The error was `provider_timeout` / `Ghidra startup deadline elapsed`. Profile digest:
`3c06738e45e144d5f073edbd3244c6bff46222f389525fe659bc92dca2ccdece`.
The complete error is local-only at `target/rea/20261022-paint-trace/evidence.json`.
Thread dumps showed active work progressing through constant-reference, scalar-operand
and stack-variable analysis. Raising the deadline alone did not establish readiness.

`[MEASURED]` A subsequent **focused profile completed**, with `Stack`,
`x86 Constant Reference Analyzer` and `Decompiler Switch Analysis` disabled. Ghidra's
log reports 676 seconds for the configured analysis. A persistent stdio REA MCP session
then traced two flood-effect strings to `0x535900` and returned pseudocode/assembly for
seven selected functions, without repeating startup. Complete Evidence was exported and
the native session closed. The original executable's SHA-256 remained unchanged.

This partially unblocks task #2: native function inspection works with the focused
profile, while the full default profile remains unverified beyond its timeout. Automatic
stack-variable, propagated-reference and switch recovery are reduced; three indirect call
sites in the selected functions remain unresolved. No game runtime was observed.
See `re/paint-entrypoints.md` for the exact target/profile, Evidence IDs, observed path and
remaining unknowns. The corresponding runner command is:

```powershell
.\tools\rea-spore.ps1 -Focused -Operation trace -Query 'SPSkinPaintFloodEffect'
```
