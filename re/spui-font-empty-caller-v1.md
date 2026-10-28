# Font0 empty-program simple glyphs

The bounded Rust caller handles the four simple glyphs in font0 whose hint
program is empty: glyphs 5, 132, 219 and 230. It scales their real and phantom
points, follows the original helper's early return without entering the hint VM,
then applies the observed caller's horizontal grid alignment and advance.

For the retained font0 square-15-ppem identity profile, the public Rust caller
and raster path match all 898 source-reference values checked: helper and final
points, advances, alpha bytes, bitmap dimensions, pitch and final aligned origins.
The source reference also agrees with 16 previously retained native observations.
The Rust output SHA-256 is
`ebada708f2d3e8e26107f436f552c8dd1fa0c64a634a27ddb4f89f6a3bddd91a`.
Evidence is retained under ignored `target/source-ui/`, including
`font-empty-rust-public-comparison-v1/manifest.json` and
`font-empty-caller-retained-compare-v3/manifest.json`.

The existing hint VM continues to reject an empty program; this is a distinct
caller path. The evidence does not establish graphics-state contents, cache
lifetime, composite glyphs, other font faces, other transforms, or complete UI
behavior. Original font data remains local to the installed game.
