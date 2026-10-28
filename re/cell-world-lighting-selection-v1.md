# Cell world lighting selection: source trace boundary

This is a static trace of the installed base executable SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The original program was not executed for this investigation.

The Cell initialization at `E5D9E0` calls the object obtained from `67D8B0`
through virtual slot `18` with `(04A4D97F, 0, 0)`, then calls the returned
world's virtual slot `30` with `A426730B`. It also supplies that same world to
the background and main model worlds through their virtual slot `140`. The
call identities do not establish that a frame was drawn.

Startup `7A77E0` obtains the manager through `709990`, initializes it via
virtual `0C` and puts it in global `015FE8D0` using `67DAC0`. Its constructor
`709590` installs primary vtable `140C6C4`; slot `0C` is `70DBD0` and slot
`18` is `70D5E0`. The former calls `70DAB0`; the latter constructs a world via
`70CC10`, which installs vtable `140C708`. Its slot `30` is `709EB0`. These
are static data and control-flow observations, conditional on the objects and
calls succeeding.

The original bytes at `709EB0` call `7053C0` with the selection ID and the
manager stored at world `+0C`; the returned pointer is written to world `+10`,
the ID to `+14`, and `708E10` processes the result. `7053C0` searches 0x144-byte
manager entries and compares the object referenced by each entry's first word
at object `+8` with the selection ID. It may return null. Ghidra does not define
a function at `709EB0`; the bounded linear decode is retained under ignored
`target/source-cell/world-preset-00709eb0-linear-v1.json`.

`70DAB0` passes the word at `1536B38`, `40200100`, to property-manager slot
`48`; its assembly then calls slot `2C` with each returned vector word, that
same namespace, and an output pointer. On success it passes the returned
object and a 0x144-byte entry to `70A820`. Instruction `70A8D0` stores the
returned object as the entry's first word. Property-manager slot `48` is
`6AA3F0`, and slot `2C` is `6A5B70`. The latter constructs a PROP request
with type `00B1B104` and the input low and namespace words. Its successful
uncached path uses the manager's resource provider and `6A3890`; cached paths
must also be accounted for.

The property-list producer `6A2DE0` allocates a list via `6A1640`, calls the
source resource object's virtual `10`, and copies the returned three words into
the list at `+8`, `+0C`, and `+10`. The fresh provider path closes the identity
link: `6A5B70` passes a PROP key and four zero optional arguments through
`8DEC20` to `8DE8A0`. After successful creation, instructions `8DEAE2..8DEAF9`
copy the original request's three words to the returned object's `+8..+10`.
Thus a fresh successful request for `A426730B` produces a matching list entry.
Property-manager cache lookup `41F890` compares both low ID and namespace;
the resource-provider cache can use registered callbacks (`8DD8A0`), whose
behavior remains unverified. Source package precedence and live state remain
unproven, as do success and selection when the manager is not fresh.

The installed packages contain candidate PROP instance
`40200100A426730B`: PatchData scope `0001FFFF`, SHA-256
`4aba7085198c157065ce1b68be18dba163f718d3a328aa27a91aa9b0715ba6c5`, and
Spore_Graphics scope `00010000`, SHA-256
`268b794bf78c58faee9a954ebb72abedf2334f127408b8e1eb50c044c4043fbf`.
Both have 25 RGB entries at property `0100EAC5` and float property `0100EAC4`
with bits `3E99999A`; parsed fields agree. Their distinct payload hashes are
not treated as evidence of overlay choice.

Reproduce the independent installed-candidate read with:

```powershell
python tools/cell_world_lighting_candidate_v1.py 'D:/Games/SPORE Collection' --out target/source-cell/world-lighting-candidate-v1.json
```

The ignored report's SHA-256 is
`82295b59d0ea2cf2a8c40222e5c72cade0a99f25dbc4b8d14d577cbf355696b5`.
REA dossiers in `target/rea/20261028-source-runtime-v6/` include 20, 142,
146, 154, 157–161, 165–166, 169–171, and 179–183. Source fields do not alone
establish shader uniform state, scene framing or visual equivalence.

The standalone Rust example `cell_world_lighting_intake_v1` accepts an
explicit package and validates the complete named PROP, 25 finite RGB values,
and a finite scalar exposure. It has not been compiled or executed in this
trace. Its selection of a file is a caller choice, not retail overlay evidence.
