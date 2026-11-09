# Controlled E4EE10 factory wrapper and request store

The independently reviewed original fixture extends the conditional request
gate through the complete original E4EE10 wrapper and original A04D20/A04D40
setters, then reads the stored request through E51760. Five cases use guarded
synthetic world callbacks: gate skips, a null request, a stable nonnull request,
and a case where world +40 replaces request +17C before both original setters.
The replacement case writes the supplied second backing; all actors, component
pool, request and backings are synthetic. Real A0B110 creation, asset/provider
selection, allocation and runtime lifetimes are not exercised.

The original run matched its controlled output checks, exited zero and closed
the non-switched desktop after reaping. Receipt
`target/cell-camera-factory-result-v1/receipt.json` SHA-256
`5528441e76c6536b672792ce8721bd1a59b1f488ebf768b15f5d825ac389543d`.

`cell_camera_factory_v1` separately composes the bounded gate/store/provider
projection with explicit factory result, fresh post-attach backing snapshot
and admitted backing stability between source setters. Its comparator checks
all five retained provider/store and both backing/FP rows. This does not
establish the actual request, resource winner or live actor/camera ownership.

The follow-on world-prefix fixture executes the original A0B830 wrapper and
A0B110 prefix, trapping immediately before cache lookup. Five controlled rows
matched, including the key words 1 and 40616200 on the three entered paths.
Cache lookup, allocation and the request return are scripted or excluded; the
world, request and ownership are synthetic. The first attempted invocation
omitted the required manifest argument and stopped before creating a result or
executing the host. The corrected externally pinned invocation succeeded;
`target/cell-camera-world-prefix-result-v1/receipt.json` SHA-256 is
`52521e59d29acf0e55402e07970ef049a7fd3d4220b9135d319f667adade1d37`.

The standalone `cell_camera_world_prefix_compare_v1` comparator reuses the
bounded factory projection and checks all five retained rows and key captures.
Its successful comparison does not establish real cache selection, allocation,
request identity or runtime lifetime.
