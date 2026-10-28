# SpaceGame candidate references, 2026-10-28

`tools/space_game_reference_candidate_v1.py` inspects the resource key
`2DB6DAD3:00B1B104:CC489C6F` (instance:type:scope) in an explicitly supplied
installed DBPF package. The instance matches lowercase FNV-1 of the ASCII string
`SpaceGame` found in the installed base executable. This is a name candidate,
not proof of resource identity or stage use.

The decoded bytes in both `Data/Spore_Game.package` and
`Data/PatchData.package` have SHA-256
`306be594fb294905b6d48342fbf71c1b34956f183b392814abdcb92a89ec864a`.
The respective index hashes are
`abd86cb2718f3e7ff4a2e1550e25f4c88ea06191c3c945ac8898cfb9c99188db`
and `075698b95018ce0115d1178e2070ad3b1a2155232be815cf9200a71b7e961ee9`.
The parser admits the observed two-field framing, keeps the first record opaque,
and exposes the second property's five 12-byte triples as raw little-endian
words. Its last first word, `B09ABBDA`, matches lowercase FNV-1 of executable
string `Planets` and a PROP instance in the same package and scope. This is a
candidate connection, not a resolved reference or a gameplay rule. The parser
never decides patch precedence or changes the Space prototype.

Reproduce with `python tools/space_game_reference_candidate_v1.py <package>`
and synthetic framing checks with `python -m unittest discover -s tools -p
test_space_game_reference_candidate_v1.py`. Further work should trace native
consumers of field ID `7435A2D3` and the SpaceGame lookup, then establish key
resolution and active stage use before integrating values into runtime.

## Static native consumers

REA Ghidra dossiers q271–275 identify functions `0065FBF0`, `00661490`,
`006616A0`, `00661790`, and `006618A0` containing the property ID. The
pseudocode calls `006A0550` with the ID; `0065FBF0` and `006618A0` traverse
12-byte entries and recursively pass each first word, while `006616A0` passes
first words to `006615A0`. In `006618A0`, nearby strings include `Sporepedia`
and `UI/Tooltip`. This supports a candidate UI/category tree interpretation,
not Space gameplay or serialization. Dossiers contain stack/decompiler
uncertainty and unreachable-block warnings; callers, accessor contract, and
active runtime paths remain unresolved. Analysis target is base `SporeApp.exe`,
SHA-256 `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The source-bound parser consequently remains an audit, not runtime integration.

Dossier q287 shows `006A0550` checks type `0x20` and an array flag;
its decompilation warns of stack analysis problems, so its precise ABI is not
claimed. Dossier q288 shows `006615A0` comparing generated `0x34`-byte
records against a collection; it does not establish Space-stage semantics.
