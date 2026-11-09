# Controlled fresh source sweep registration

The one-frame fixture now joins its existing checked registry to a fresh,
fixed-capacity `BBAB40`-shaped append model before the modeled sweep. Each
registry actor supplies its own scoped raw handle, tag and admitted bounds;
the model returns an append slot index (the modeled E74A50 +364 value), stores
the handle/tag, and appends opening/closing endpoints for both axes. The frame
reports these fixture slots, rebuilds the existing source-order sweep from the
same appended snapshot, and re-resolves dispatched actor words in the same
registry before the broadphase. A passing pair remains pending E7F170.

This is a newly empty collection each frame, with checked capacity and
finite-normal-or-zero, strict-interval preflight. Those checks are fixture
safety policies: BBAB40 has no equivalent local capacity check. It does not
model native allocation, registration side effects, removal, free-list reuse,
endpoint history, tie ordering, mutation during iteration, or source actor
lifetime. The subsequent sweep rebuild still has its own bounds/tie
admissions. Neither the synthetic registry handle nor the returned fixture
slot establishes a live actor or native collection registration.

## Controlled original-instruction comparison

The independently reviewed host ran the complete unpatched `BBAB40..BBAC2E`
function, 239 bytes, SHA-256
`047980ab870d8c5d32f1d76ddf5f7171a8627300ea05ef349d9bd9da2f22e094`.
Two cases of two sequential fresh appends matched the model's complete 116-byte
snapshot each: active/high-water/endpoint counts, both 16-byte slot records,
both four-entry endpoint arrays and both returned indices. Inputs are hex
`(actor,tag,min-x,min-y,max-x,max-y)`:

```text
00010000 00000000 00000000 c0000000 3f800000 40000000
00020001 00000004 40000000 40400000 40800000 40a00000
80010000 ffffffff 80000000 c0400000 3f800000 bf800000
ffffffff 00000009 c0000000 40000000 bf800000 40c00000
```

The [receipt](../target/cell-fresh-registration-result-v1/receipt.json) SHA-256
is `682dfee2d134ed4d3932336fdf4cf251c891ecb74f7ded7a753bfe9dcbd50fea`.
Model and native stdout SHA-256 are
`928d358794d3b2aefcedf07f8db0f0e8ac33b9bfb6a4cd7a5eedab725723ca12` and
`c425bce899503734ee4f9fe18717176dff3cf046b862ebd4e8a9016e345562e2`;
both exited zero with empty stderr. Host SHA-256 is
`089413e731b7dd19dd51e7cd86620957e20f41b4375a1b961890f1e8d7c65932`, runner
`ca984fa2d402c1074cd2c320156d47289dbbbc40825bf273c072f1e834d068ca`, and
model source `7e91319cf496b515d56ebb19583d07cccbe7c32ddc82e8f64501ecf8f6d3d16f`.
Full linked source bytes, complete instruction/RET boundaries, pre/post pins,
stack and guarded arrays passed. Both rows recorded x87 control 027F, status
0000 and MXCSR 1F80; control/TOP checks passed. The first preparation retained
an assembler failure from reserved label names; only corrected v2 was run.

The two finite, disjoint cases do not test exceptional values, exhausted
capacity, source pointer aliasing, free-list paths, removal, concurrent
mutation or live source registration. The comparison's source hash applies to
the exact model source listed above and not automatically to later changes.
