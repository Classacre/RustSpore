# Candidate kind-0 RW4 shader evidence

This is a read-only disk and static-source trace, not an observed actor or render. The candidate asset chain and its provider limitations are in [the intake note](cell-kind0-asset-intake-v1.md). The exact inspected source is `Data/Spore_Graphics.package`, resource `2F4E681B:00010000:00000000DAE8CE17`; the 95,536-byte RW4 payload has SHA-256 `bbd99a741d41a3606a310f242e14e5f85dc7b49af454972ff02124b16e318592`.

## Saved mesh, state and raster links

RW4 mesh section 13 links index buffer section 12 and vertex buffer section 4. The latter references declaration section 7 and buffer data section 3: 454 vertices, stride 36. Section 12 references data section 11, format `65` (D3D INDEX16), and primitive value 4. Section 13 stores 896 triangles and 2,688 indices. Mesh-to-state section 15 contains `[13, 1, 14]`. State section 14 is compiled-state type `2000B`, has primitive word 4, shader ID `80000004`, and soft-dirty word `0010801C`. These are saved links and words; actor choice and runtime fixup remain unverified.

State 14's `0x8` variable region contains records `(ID, offset, length)` of `(3, 0, 4)`, `(4, 0, 20)`, and `(210, 0, 20)`, followed by the zero-ID terminator. Bounded static decodes in retained dossiers q205 (`11EE810`) and q207 (`11EE650`) support the record traversal. After the other declared regions and state list, the saved texture stage 0 references section 17 and stage 1 references section 19. These section indices identify two `20003` raster sections, each 128 x 128 DXT5 with eight mip levels, using data sections 16 and 18 respectively. Their complete compressed-data hashes are `6722dedde263a9c159878da41188d2b6be5345046410766a4a00e760986778ba` and `92798f39cd5b2228e97dc82598891e3bfa6dd9ac320c57cedb81c58da0bfae1f`. The stored references do not prove successful live pointer fixup or binding; draw-time state may also alter them.

The state embeds a 36-byte declaration with six 12-byte entries. Each row below is `(stream, offset, D3D type, method, usage, usage index, saved semantic)`:

```text
(0,  0, 2, 0, 0, 0,  0)
(0, 12, 5, 0, 3, 0,  2)
(0, 16, 5, 0, 6, 0, 19)
(0, 20, 1, 0, 5, 0,  6)
(0, 28, 5, 0, 2, 0, 14)
(0, 32, 8, 0, 1, 0, 15)
```

Under the native semantic-to-feature mapping documented in [shader selection](shader-selection.md), these semantic IDs yield feature word `0008C045`. Section 7 contains the same declaration bytes; the current runtime declaration still needs to be established.

The stored data records are:

```text
ID 003: 03000000
ID 004: 0100000008000000000000000000400000008000
ID 210: 025c4426000020410000803f0000803f00000000
```

The recovered `210` callback documented in [material links](material-links.md) treats the first byte as a separate selector and, with the observed `5C 44 26` marker, uploads the following four float values `[10, 1, 1, 0]`. This is one saved input, not a complete list of uniforms or a verified live binding.

## Shader and controlled-render boundary

In all twelve inspected installed profile packs, ID `80000004` identifies builder `generic_skinning`. The pack resource has type `0469A3F7` and packed instance `4021200400000000` through `4021200400000003`, in each of `Spore_Graphics`, `PatchData`, and `Spore_EP1_Data`. The `target/source-shaders/<package>-0469A3F7-402120040000000<profile>.pack-layout.json` files retain exact parsed pack evidence. Profile 3's render-type-zero builder contains 37 vertex and 27 pixel selectors in Graphics, versus 38 and 28 in PatchData and EP1. Thus a builder ID does not select one compiled vertex/pixel program by itself: selector evaluation requires the current declaration, shader-data presence and values, sampler availability, and other runtime inputs. Package ordering, profile choice, render type, selected recipe, cached program, complete uniforms and runtime overrides are unresolved.

The existing `native_cell_renderer.rs` offscreen renderer parses GMDL, uses the profile-3 `generic_static` builder `80000002`, and derives texture inputs from GMDL material data. It does not directly implement this RW4 `generic_skinning` state. A controlled candidate draw would need to validate the saved mesh/index ranges and topology, use the exact declaration and both referenced raster mip chains, decode and apply the saved state and sampler lists, and evaluate the profile-matched builder against an explicit complete context. It must provide and identify a test bone palette, transforms, camera and lighting inputs; select only a corresponding cached program or a separately justified compilation; and record the resulting recipes, program hashes and all assumed inputs. An identity palette or inspection camera would be a controlled fixture, not evidence of the original pose or scene.

No original shader render, actual runtime pointer fixup, active candidate, provider precedence, texture overrides, real pose, or rendered-image fidelity is established by this note. No Cargo, native code, GPU or GUI was run for this investigation.
