# Blockdata lookup wrapper projection

`spore_core::blockdata_lookup_v1` is a bounded projection of the result behavior
of `414DC0` in the installed base `SporeBin/SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The module is registered in `spore-core`; it is not a resource-map implementation
or a gameplay integration. The underlying raw-buffer query is documented in
`blockdata-query-interpolation-static-v1.md` and its model comparisons in
`blockdata-query-model-comparison-v1.md`.

## Static source boundary

`414DC0` forms a lookup key from its arguments and invokes `41F890` on the
owner's map at `+30` (`414DD1..414DFD`). The returned iterator is compared
with the map's end (`414E02..414E65`). This establishes the existence of the
lookup step, not how an actual gameplay actor selects a key or resource.

* For a found entry with no first output pointer, `414E6B..414E6F` skips
  evaluation and `414EDE..414EFF` returns true. A missing entry with no first
  output pointer returns false at `414F06..414FAA`. Neither branch needs a
  validated payload or examines coordinates.
* With a found entry and requested outputs, the wrapper initializes its local
  index-pair state and calls `414FC0` at `414E71..414ED9`. It compares only
  second triple component zero with first triple component zero using
  `COMISS` and `JBE` at `414EF4..414EFD`. Strictly greater returns true;
  equal, lower or unordered reaches the fallback. The other two axes are not
  acceptance checks.
* A missing entry with requested outputs, or a failed component-zero
  comparison, returns false and writes three copies each of the source
  constants `0xBDCCCCCD` (-0.1f) and `0x3DCCCCCD` (+0.1f), respectively
  (`414F06..414FAA`). The constants are located at `013EB974` and `01489324`.

## Rust projection and limits

The caller supplies `Lookup::Missing` or `Lookup::Found`, independently from
the optional validated `NativeSamples` view. `Request::PresenceOnly` does not
need that view. A values request for `Found(None)` returns the explicit
`UnvalidatedResource` error; a found validated view invokes the bounded
`blockdata_query_v1::query`. Query errors propagate separately from an ordinary
false wrapper result. Missing values and a non-accepted evaluated first axis
produce the observed fallback bit patterns.

The typed return models distinct, valid output buffers for a values request.
It does not model pointer aliasing, an invalid second output pointer, map
construction, reference lifetime, caller key selection or runtime resource
precedence. The query admits a bounded exact-size payload and finite inputs and
results; it rejects some situations the original instructions do not reject.
NaNs, exceptional arithmetic, and original floating-point control-state parity
are not established by this projection. This is a static instruction mapping,
not an execution of the original wrapper or evidence of live game behavior.

## Tests

Seven focused module tests passed as part of the full all-target workspace run:
797 passed, zero failed, and two ignored. They cover supplied presence without
validation, missing outputs, first-axis-only acceptance, equal and reversed
first-axis fallback, defensive query errors, and a nonzero two-sample grid
whose interpolated first-axis bounds cross. The retained log is
`target/workspace-tests-20261030-resume-v58.log`, SHA-256
`18a693a991ad8dd3976ac7c0f714d4c5cc5775a9415aea6fd7d31bd0420c2e85`.
The integration agent independently compared the implementation statically to
the source; the original game wrapper was not executed. These tests and review
do not establish live resource-map selection or original runtime behavior.
