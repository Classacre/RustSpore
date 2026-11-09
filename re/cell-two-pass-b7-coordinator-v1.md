# Controlled two-pass B7 coordinator

The checked fixture composes the existing E7F170 coordinator with two explicitly
supplied early-B7 E6A450/E59B40 candidates and a single selection snapshot. It
checks the retained fixture handles and pass order, requires stable disjoint
synthetic inputs, `[7, 7]` definition B4 words, and finite nonopaque geometry.
The result stops at pending E7E820 or pending cleanup. Resolver, selector route,
cleanup, event dispatch, live actor identity and real cache ownership remain
outside the model. Equal actor IDs are rejected by the checked fixture; the
comparison adapter records that rejection as the source host's early return.

The independently reviewed source-instruction host was run for four controlled
cases: first actor is player, second actor is player, neither is player (event
suppressed), and equal IDs (early return). Its event stub captures arguments and
does not execute dispatch. The retained native receipt is
`target/cell-two-pass-result-v2/receipt.json`, SHA-256
`f64cfb93c2ae53cb448a7bb2bac57a7dd039f2f78b7c84d400b51ac0f3842ef4`.
The Rust adapter matched all four retained output rows, excluding the separately
recorded native floating-point control/status words. Its separate receipt is
`target/cell-two-pass-rust-result-v1/receipt.json`, SHA-256
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
The comparison does not rerun source instructions and is bounded to the pinned
fixture, executable and source revisions in its comparator.
