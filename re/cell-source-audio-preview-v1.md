# Cell source audio inspection preview

The experimental Cell app accepts an explicitly selected, one-pass source audio inspection preview through all three options:

```text
--source-audio-package <package>
--source-audio-key <16-hex-scoped-instance>
--source-audio-sha256 <64-hex-payload-sha256>
```

The specified package is read directly using the native index, with bounded index/resource sizes and an in-memory payload SHA-256 check. Only supported nonlooping RAM PCM16 or XAS1 is accepted; looping, streamed and EAL31b inputs fail closed. The package path must remain stable while the index reader reopens it; the payload hash does not authenticate package identity or prove stability. On normal windowed startup, decoded PCM is queued as one Bevy `AudioPlayer` with `PlaybackSettings::DESPAWN`; the temporary pending resource is removed. The UI labels it a source audio inspection preview and the log reports explicit selection, successful decode, unverified playback and no event mapping. `--offscreen` rejects these options.

The preview is not source-game audio selection, a reconstructed gameplay cue, an automatic Cell soundtrack or evidence of audible output. The chosen key and expected digest are caller inputs. A headless test checks argument gating, PCM iteration, asset handle, despawn setting and pending-resource removal; those facts do not establish eventual audio completion or audio-device playback. A separate opt-in ignored test takes an explicit installed package and checks the selected known XAS1 payload against its retained PCM SHA-256. The underlying XAS1 implementation is derived from vgmstream; matching its reference is not independent codec validation. No GUI or sound device was run for this integration.

The final focused headless run passed three tests, with the installed-data test ignored; its retained log `target/cell-source-audio-preview-headless-v2.log` has SHA-256 `e22b687dada80311f266e53ac6fcf55d00358f3bd106f125343f94a277b5ff82`. A separate explicitly selected installed XAS1 run passed, checking 44,100 Hz mono, 14,043 samples and PCM SHA-256 `650ee6371dc3630ac7fc81404ef481203ea70cdba6fa727df7045d212cd9a879`, and rejecting an incorrect payload digest. This checks decoding and queuing boundaries, not audible playback.
