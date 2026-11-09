#!/usr/bin/env python3
"""Compare two explicit PCM16 sources after a caller-specified alignment.

This is a diagnostic comparison, not proof of EALayer3 or playback fidelity.
Usage: python tools/eal31b_pcm_compare_v1.py VGM_PCM16_WAV RAW_S16LE CHANNELS INSERTION_PREFIX DECODER_SHIFT
The raw stream and WAV must have matching channel counts. Skip the given
number of leading WAV frames, then compare WAV[i] to raw[i + DECODER_SHIFT].
"""
import hashlib
import struct
import sys
import wave
from pathlib import Path

if len(sys.argv) != 6:
    raise SystemExit(__doc__)
wav_path, raw_path = map(Path, sys.argv[1:3])
channels, prefix, shift = map(int, sys.argv[3:])
if channels <= 0 or prefix < 0 or shift < 0:
    raise SystemExit("channels must be positive and offsets nonnegative")
with wave.open(str(wav_path), "rb") as source:
    if source.getsampwidth() != 2 or source.getnchannels() != channels or source.getcomptype() != "NONE":
        raise SystemExit("WAV must be PCM16 with the specified channels")
    count = source.getnframes()
    rate = source.getframerate()
    wav_bytes = source.readframes(count)
raw_bytes = raw_path.read_bytes()
if len(raw_bytes) % (channels * 2) or prefix >= count:
    raise SystemExit("invalid PCM length or prefix")
start = (prefix + shift) * channels * 2
sample_bytes = wav_bytes[prefix * channels * 2:]
if start + len(sample_bytes) > len(raw_bytes):
    raise SystemExit("raw PCM is too short for specified alignment")
raw_slice = memoryview(raw_bytes)[start:start + len(sample_bytes)]
max_diff = total_diff = within_one = exact = 0
for (left,), (right,) in zip(struct.iter_unpack("<h", sample_bytes), struct.iter_unpack("<h", raw_slice)):
    delta = abs(left - right)
    max_diff = max(max_diff, delta)
    total_diff += delta
    exact += delta == 0
    within_one += delta <= 1
samples = len(sample_bytes) // 2
print(f"{count - prefix} frames, {channels} channels, {rate} Hz; {samples} compared samples")
print(f"max absolute difference={max_diff}, mean={total_diff / samples:.6f}, exact={exact / samples:.6%}, within_one={within_one / samples:.6%}")
print(f"WAV PCM sha256={hashlib.sha256(wav_bytes).hexdigest()}")
print(f"raw PCM sha256={hashlib.sha256(raw_bytes).hexdigest()}")
