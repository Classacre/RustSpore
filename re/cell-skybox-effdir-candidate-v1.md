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
