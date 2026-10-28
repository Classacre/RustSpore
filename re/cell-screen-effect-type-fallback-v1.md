# Cell screen-effect resource type fallback: static trace

This note extends [the screen-effect source trace](cell-screen-effect-source-trace-v1.md)
for the x86 base `SporeApp.exe` SHA-256
`17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
Addresses are specific to that binary. The observations are static; they do
not establish that the candidate effect is active or that a resource loads.

The candidate's screen path constructs the request words
`[BD08BD8A, 2F4E681C, 00000000]`. The separately inventoried installed
record has the matching instance and group but type `2F4E681B`. The following
trace shows a registered way to try that type, rather than proving that the
record wins lookup or is rendered.

Graphics startup function `7A77E0` constructs the object initialized by
`7652C0`, then calls slot `44` of the resource-provider object returned by
`67D7F0`, with the registration flag set and the constructed object as an
argument. The provider implementation `8E1440` calls the object's slot `10`
during registration. The object's primary vtable is `140E02C`; its slot `10`
points to `763F90`. Bounded instruction decoding at `763FC4–763FDC` shows
that initializer calling the provider's slot `24` with key type `2F4E681C`,
pointer `140DFB0`, and count 3. Provider method `8E1320` copies that array
into the type-keyed alternative list. The 12 bytes at `140DFB0` decode as
three little-endian words in this order:

1. `2F4E681C`
2. `2F7D0004`
3. `2F4E681B`

Evidence: `ev_05e68cc50152e318444dc90835ec64580b2f806d3263fe171219c3b55d67c60c`,
`ev_c36a8e9cef0f6a4f47fd803b349036a95ac1d9c87c4a6bf871a9834e4be57ae3`,
`ev_f855a2b15a4397461c7a230d3a608813b2693219fb361a561c0d11391379f183`,
`ev_737f5717b108401f0462ccada251e5db6f0793d342aa653ac7093449f5a2714a`,
`ev_b1aef97ba4e708f6f886a111ca03899f2b3ef8e13f32ff76b82e20a9c0a950ae`,
`ev_8da50280a67548fef86d6ca91256cf6657acfed6dd47961c390176a777856d5a`.
`763F90` was decoded from the known vtable entry because REA did not recover
it as a function; the bounded decode is not a complete control-flow analysis.
Registration assumes the relevant startup path executes and allocation succeeds.

The provider's `8DE8A0` has different paths depending on whether an output
pointer is supplied. On the null-output path, it invokes provider slot `30`
(`8DF1A0`) to locate a source. Bounded decoding at `8DF1A0–8DF306` shows a
lookup of the alternatives keyed by the request's type word, copying the key,
replacing its middle word with each alternative in order, and probing a
provider through slot `34`. If no provider is supplied, it traverses providers
and tests the list for each; a match can also be copied to an output key.
For the output-pointer path, `8DE8A0` similarly iterates the registered
alternative types and attempts a provider call with each substituted key.
REA did not recover `8DF1A0` as a function, so this part rests on bounded
decoding from its known vtable entry. Evidence for the decompiled output path:
`ev_0242f273bf966eb7373e12f4adda41467f792b258e45796a89f2171670fa02fa`.

The registered object's slot `30`, `763350`, accepts source type `2F4E681B`
for a `2F4E681C` request. The binary also has the corresponding conversion
branch in `76A0D0`, and `765B00` tests that same source/request pair. Evidence:
`ev_e6f294709c1db55b6494fe11fdc56341ecfe11cf7472a417f0789763d7644e52`,
`ev_2ab8559c95fb0065ea6418adf7c263b1ecfc44ef9039f635d7e6f3ce3f1c9afe`.
This supports `2F4E681B` as a static fallback candidate and an available
conversion path. Provider ordering, earlier matching alternatives, successful
loading, candidate selection, and final image or shader behavior remain
unverified. No original game payload is included here.
