# First-feature fragment comparison: prepared, not executed

Preparation created a synthetic x86 PE containing the exact 200-byte
`E6AE6C..E6AF33` fragment from the pinned executable. The preparation step
did not execute it; the subsequent reviewed run and six-case result are
recorded in [the projection note](cell-controlled-feature-projection-v1.md).
`fragment.bin` SHA-256 is
`addffaaee26f1051ffeaf1fca15f1fb463449e72c1a14e7c2e2fa50d6dbaefe2`.
The JE is internal; the fragment has no calls. Review the linked host and
wrapper, including the copied bytes and destination, before execution.

Six fixed finite cases cover bypass and identity transforms, asymmetric
matrix, Y cancellation/order, subnormal rounding and signed zeros. The host
sets MXCSR 1F80, supplies separate guarded actor and row memory, reserves stack
scratch, captures x/y/z/radius and post-fragment MXCSR, then validates guards.
The Rust adapter uses the same fixed corpus. The runner will preserve raw
stdout/stderr and a success or failure receipt in an exclusively created run
directory; a failed attempt must not be overwritten. It verifies pins before
and after, host machine and imports, source fragment and linked byte occurrence.
It cannot exclude transient file replacement or independently prove executable
provenance. Recorded MXCSR words must have control bits 1F80 after masking the
six sticky flags; those flags are recorded but not compared. Execution is conditional
on separate review of assembly, emitted PE, ABI, memory layout, corpus and runner.

Preparation sources: `tools/cell_feature_fragment_prepare_v1.py`,
`tools/cell_feature_fragment_run_v1.py`,
`crates/spore-bevy/examples/cell_feature_fragment_model_v1.rs`.
Artifacts and pins: `target/cell-feature-fragment-prep-v2/manifest.json`.
The earlier preparation failure (MASM line length) remains under
`target/cell-feature-fragment-prep-v1/build-failure.json`. Host build and model
adapter build did not execute copied original instructions. No live game was run.

The proposed reviewed invocation is:

```powershell
python tools/cell_feature_fragment_run_v1.py 1325ffd6bd8ee5f641beb9b03069b2053f98a014812ae1c50d3c160163156f2f
```

A pass would establish only agreement for the six supplied cases under the
stated controls, not runtime actor identity, actual cache state, feature pair
geometry, contact acceptance, or gameplay behavior.
