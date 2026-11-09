# Controlled E51760 component-provider join

The reviewed original fixture extends the earlier camera-vector and dispatch
path through original E51760 and B71D00 on a separately allocated synthetic
component pool. The eight fixed cases include zero actor +248, zero actor +FC,
and two nonzero words with the same low index and different high portions.
Original B71D00 ignores the high portion; both project the supplied slot. The
component +24 request pointer and its aggregate are synthetic, stable fixture
inputs. The test does not establish live ownership or validate generations.

The original run matched its controlled output checks, exited zero and closed
the non-switched desktop after reaping. Receipt
`target/cell-camera-provider-ownership-result-v1/receipt.json` SHA-256
`2b3aa5e9ec03b68b18426d493a7dffaa9a0c5effc8cee1f7768ba4da397ac8f9`.
The host retained the earlier ramp as a measurement rather than an independent
exact oracle. Mode/selector alternatives and E60B0D remain bounded as described
in `cell-camera-dispatch-v1.md`; no gameplay camera or frame is observed.

`cell_camera_component_provider_v1` separately composes the checked dispatch
with the source gates and a low-16-bit component lookup. It requires readable,
stable, disjoint component slots and admitted nonwrapping base/stride address
arithmetic; it intentionally does not add a generation check. Its comparison
checks all eight retained vector, lookup, callback-count, scalar and FP rows.
The request pointer, post-callback snapshots and aggregate are supplied, and
the model does not establish their real source lifetime or identity.
