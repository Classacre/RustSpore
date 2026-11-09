# Controlled Cell feature snapshot

The controlled source capability retains its first-word cache key and ordered
`NativeCellPart` rows. `feature_snapshot` accepts an independently supplied
post-override actor triple, requires a nonzero matching first word, and exposes
rows in order while skipping class 1 without renumbering. The second and third
triple words deliberately do not participate in the check: the traced E67A50
lookup uses the first word. This models only an admitted stable hit; it neither
creates a cache nor validates the original full triple or provider.

The source-player inspection prints these eligible row indices using its selected
triple as an explicit post-override admission. The saved-player override, actor
identity, source cache lifetime, transformed feature geometry, ordered contact
producer, and callbacks remain unresolved. In particular this output is not a
contact or an observation of retail gameplay. See
[cell contact provenance](cell-contact-input-provenance-static-v1.md) and
[cell actor geometry](cell-actor-geometry-consumer-static-v1.md).
