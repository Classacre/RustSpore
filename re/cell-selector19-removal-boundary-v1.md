# Selector 19: pending actor operation

Static trace of installed `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
The validated B7 composition can select decimal 19. E7E820 resolves the selected
record's first and second words in that order via B71D00, then subtracts one
from the selector and indexes the table at E7F0D8. Entry 18, corresponding to
selector 19, is E7E90C. It pushes the second and first resolved actor pointers,
calls E7B600, and returns. This is a different route from selector 10's
E7E050/E51D40 construction and must not be composed as that event.

E7B600 checks the first actor's +112 byte for equality to 1 and returns if so;
otherwise it checks +113 for equality to 1 and returns if so. On the remaining
path it initializes a holder in the reused first-argument stack slot, resolves
the second actor's +108 reference through E4CAA0, and calls E780D0 with that
resolver result in EAX, the second actor's first word, literal 1, +0.0f and
literal 0, then tail-calls holder cleanup E82160. Values other than 1 do not
trigger either gate. All actors, references, holder lifetime and definition
resolution require fresh admissions; the source does not establish a safe
null path here.

E780D0 begins with B71CC0 actor validity, can clear the current-player word at
global +411C when it equals the actor word, then resolves the actor via B71D00
and enters further state and manager operations. This is a consequential
operation boundary: the controlled fixture must stop before E780D0 unless its
mutations, ownership and lifetime are explicitly modeled. No live removal,
callback, event allocation, actor identity or feeding claim follows.

A guarded build-only source host for the complete E7B600 function is under
`target/cell-selector19-gates-prep-v1`. It stubs holder initialization,
definition resolution, E780D0 and holder cleanup, and exercises byte pairs
(0,0), (1,0), (0,1), (2,2). Its static-only preflight intentionally stops
before launching the executable and checks source bytes and all four call/jump
relocations. After independent review the host ran and matched all four cases;
`target/cell-selector19-gates-result-v1/receipt.json` has SHA-256
`c3ec1af99709c4f0b21daef9a267db9b69ef21786099e920061a9faf2db17b84`.
The original x87/MXCSR controls passed and the private desktop was closed after
child reaping. The host does not run E7E820's pool lookups or establish runtime
actor lifetime. Its handler stubs admit stable synthetic holders and definitions;
E780D0 and holder cleanup themselves do not execute.

The checked Rust continuation consumes a pending selector-19 selection and
freshly verifies both ordered fixture handles. It takes the two gate bytes, the
second actor's definition reference and a matching admitted definition token as
explicit synthetic snapshots, then stops at the E780D0 arguments and pending
holder cleanup. A focused test covers its composition with the checked
two-pass B7 selector, the two gates, missing or mismatched definition,
stale/reversed actors and stability admission. The fixed comparator adapter
matched all four retained native rows; its holder and cleanup counts describe
the native host's admitted stubs, not real effects. The separate receipt
`target/cell-selector19-gates-rust-result-v1/receipt.json` has SHA-256
`0e5cf5fef2639740e68a6c08de7a78845e821a8408dfcbfdef5337345dcca111`.
