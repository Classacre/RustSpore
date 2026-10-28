# Cell editor setup template keys

The original Cell Editor Setup property `503D2F60` contains an ordered array of
12-byte resource keys. Native consumer `00574680` chooses a starting record via
an unresolved helper; `00574710` advances a stored valid index modulo the
positive array count. The starting-index algorithm and other caller effects are
not modeled here.

The public bounded reader checks the original field's big-endian count/stride
header, little-endian key words, exact extent, and equality with the parsed PROP
tree. Its caller must supply the raw value belonging to the same selected field.
The cycle operation admits only a valid index and at most 1,024 entries; that
limit is defensive rather than an original engine limit. Invalid native states
are intentionally not modeled as this API's errors.

Independent source analysis found 12 keys in each explicit PatchData and
Spore_Game setup copy. The Rust reader and next-index outputs match the
independent source audit for all 12 keys in each package, and each named
template resource is present in its respective package. The package copies'
setup payload hashes differ, although their template payload hashes agree.
Evidence: `target/source-cell/editor-template-rust-v2-comparison.json` and
`target/source-cell/editor-template-array-v1/`. The verification example
conservatively rejects an ambiguous raw property-header match.

These records are model templates, not an established part catalog, list of
available parts, unlock sequence, or proof of active package precedence.
