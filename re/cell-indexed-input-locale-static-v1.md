# Cell indexed-input layout selector

This extends [the indexed-input target trace](cell-indexed-input-player-target-static-v1.md)
using static x86 inspection of the installed `SporeApp.exe`, SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
It does not establish the live provider, current locale, physical key codes,
callback order, actor identity or a running player controller.

At `E844A0..E844BC`, the routine calls `67D960`, which returns the pointer at
`015FE8F8`. Assuming that pointer and its vtable are valid, it calls the object's
virtual slot `+14`, pushes the returned pointer, calls `930BA0`, and then adds
eight bytes to the stack. Before the virtual call it pushes a pointer to the
literal at `01403C78`. The literal is the six UTF-16 little-endian words
`0066 0072 002D 0066 0072 0000`, i.e. the terminated value `fr-fr`.
Whether that pointer remains the comparison's second argument depends on the
dynamic callee's stack behavior; it could also be an argument to the virtual
slot. The callee's calling convention is not established here.

If the literal remains the second comparison argument, and pointers and the
UTF-16 word span are valid and stable, `930BA0` compares the returned object's
begin/end span with that literal:
the length must be five words and all ten bytes must match. It returns integer
one for equality and zero otherwise. The inspected comparison is exact and
case-sensitive; no locale normalization is visible here. Therefore each of
the four `E844A0` results in the indexed-input trace can be derived from the
separately returned span if the comparison's pointer and lifetime conditions
hold. The caller uses only AL, and the first and third calls select alternate
bit indices when that byte is nonzero. The four virtual calls may return
different spans or have side effects, so a single shared locale value is an
additional frozen-state admission, not a native scheduling fact.

One candidate constructed provider and its `+14` implementation have since
been traced in [the provider note](cell-indexed-input-locale-provider-static-v1.md).
For that same live object and unmodified table, the slot returns its embedded
span without popping the literal, closing the comparison argument condition.
The active runtime object and locale remain unobserved. Matching `fr-fr`
conditionally selects different input indices; their correspondence to a
particular keyboard layout or an observed
key press remains a separate event-routing question. Comparing a supplied
`[u16]` with the five literal words is a small testable inspection operation,
not proof of the selected runtime provider.

## Reproduction

Read the hash-checked PE with `pefile` and decode `E844A0..E844BD`,
`67D960..67D966` and `930BA0..930C19` in x86-32 mode with Capstone, mapping
virtual addresses by image base `00400000`. Read 12 bytes at `01403C78`.
No original code, GUI or live game process was executed.
