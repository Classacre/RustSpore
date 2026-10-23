"""Prepare synthetic and retained-source ROT100 requests, never execute code."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import random
import struct

import anim_quaternion_probe as quat
from anim_rotation_native_probe import INPUT_BYTES

ROOT = Path(__file__).resolve().parent.parent


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def rows(frames, qp=0, qn=0, wp=0, wn=0, seed=0, variant='ordinary'):
    result = bytearray(1920)
    rng = random.Random(seed)
    quaternions = [[0., 0., 0., 1.], [0., .6, 0., .8], [.3, -.4, .5, .70710677], [-.3, .4, -.5, -.70710677]]
    for i, frame in enumerate(frames):
        at = i*120
        struct.pack_into('<5I', result, at, frame, 0x11220000+i, 0x33440000+i, 0x55660000+i, 0x77880000+i)
        raw = bytearray(rng.randrange(256) for _ in range(100))
        q = quaternions[i % 4]
        if variant == 'nonunit':
            q = [v*2 for v in q]
        elif variant == 'zero':
            q = [-0., 0., -0., 0.]
        elif variant == 'large-finite':
            q = [1e28, -1e28, 1e28, -1e28] if i % 2 == 0 else [-1e28, 1e28, -1e28, 1e28]
        struct.pack_into('<5f', raw, 0, *q, f32(rng.uniform(-2, 3)))
        raw[28], raw[29] = qp | 0xC0, qn | 0x80
        struct.pack_into('<2f', raw, 84, f32(rng.uniform(-5, 5)), f32(rng.uniform(-5, 5)))
        raw[92], raw[93] = wp | 0xC0, wn | 0x80
        if variant == 'unused-nonfinite':
            struct.pack_into('<2I', raw, 20, 0x7FC00000, 0x7F800000)
        result[at+20:at+120] = raw
    return bytes(result)


def families(source_samples):
    out = []
    for name, frames, samples, duration in [
        ('empty', [], [0.], 1.), ('single', [0], [-1., 0., .5, 2.], 4.),
        ('two', [0, 4], [-1., 0., .5, 2., 4., 5.], 6.),
        ('four', [0, 3, 11, 32], [-1., 0., 1., 3., 7., 11., 17., 32., 33.], 40.),
        ('duplicate', [0, 0, 4, 4], [-1., 0., 2., 4., 5.], 6.),
        ('sixteen', [i*i for i in range(16)], [0., 5., 10., 111., 224., 225.], 240.)]:
        out.append((name, frames, samples, duration, rows(frames, seed=0x99F6A0+len(out)), None))
    frames, samples = [0, 3, 11, 32], [0., 1., 3., 7., 11., 17., 32.]
    for previous in range(4):
        for following in range(4):
            out.append((f'quaternion-modes-{previous}-{following}', frames, samples, 40.,
                        rows(frames, qp=previous, qn=following, seed=previous*16+following), None))
            out.append((f'weight-modes-{previous}-{following}', frames, samples, 40.,
                        rows(frames, wp=previous, wn=following, seed=256+previous*16+following), None))
    for variant in ['nonunit', 'zero', 'large-finite', 'unused-nonfinite']:
        out.append((variant, frames, [1., 7., 17.], 40., rows(frames, seed=42, variant=variant), None))
    # Each retained channel contributes literal source rows/controls. Truncated
    # channels are explicitly windows; no full original runtime is asserted.
    for sample in source_samples[:16]:
        frames = sample['frames'][:16]
        if not frames or any(frame > 0x1000000 for frame in frames):
            continue
        duration = struct.unpack('<f', struct.pack('<I', int(sample['duration_bits'], 16)))[0]
        if not 1 <= duration <= 0x1000000 or duration-1 < frames[-1]:
            continue
        data = bytearray(1920)
        for i, (frame, raw) in enumerate(zip(frames, sample['row_bytes'])):
            struct.pack_into('<5I', data, i*120, frame, 0, 0, 0, 0)
            data[i*120+20:i*120+120] = bytes.fromhex(raw)
        samples = [float(frames[0]), float(frames[-1])]
        for left, right in zip(frames, frames[1:]):
            if left < right:
                samples.append(f32((left+right)/2))
                break
        out.append((f'source-{len(out)}', frames, samples, duration, bytes(data), sample))
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-samples', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    out.relative_to((ROOT/'target').resolve())
    source_bytes = args.source_samples.read_bytes()
    groups = families(json.loads(source_bytes))
    out.mkdir(parents=True, exist_ok=False)
    records, counts, sha = [], Counter(), hashlib.sha256()
    with (out/'inputs.bin').open('xb') as stream:
        for blend in [0, 1]:
            start = len(records)
            for name, frames, samples, duration, data, source in groups:
                for frame in samples:
                    for wrap in [0, 8]:
                        for enabled in [0, 1]:
                            for control in quat.CONTROLS:
                                request = struct.pack('<fIf5I', f32(frame), len(frames), f32(duration), wrap, control, blend, enabled, 0)+data
                                assert len(request) == INPUT_BYTES
                                stream.write(request)
                                sha.update(request)
                                records.append(dict(row=len(records), family=name, source=source is not None,
                                    blend=blend, frame_bits=f'{quat.fbits(frame):08X}', count=len(frames),
                                    wrap=wrap, enabled=enabled, cw=f'{control:04X}'))
                                counts['source_rows' if source else 'synthetic_rows'] += 1
            counts[f'blend{blend}_rows'] = len(records)-start
    (out/'input-index.json').write_text(json.dumps(records, indent=2)+'\n')
    (out/'source-provenance.json').write_text(json.dumps([g[5] for g in groups if g[5]], indent=2)+'\n')
    manifest = dict(schema='original-info-rot100-input-cohort-v1', input_bytes=INPUT_BYTES,
        rows=len(records), input_sha256=sha.hexdigest(), counts=dict(counts),
        source_samples_path=str(args.source_samples.resolve()), source_samples_sha256=hashlib.sha256(source_bytes).hexdigest(),
        families=[dict(name=n, frames=f, sample_frames=s, duration=d, source_window=source is not None) for n, f, s, d, raw, source in groups],
        scope='Explicit bounded synthetic contexts plus literal retained source ROT100 row windows. Both blend policies, both neighbor-wrap contexts, enabled/disabled descriptor, all12CWs. No normalization/filtering of source rows. Whole source census remains separate5022/731383 denominator.',
        host_executed=False, next_step='Root reviews this exact input cohort together with the separate rotation-native host before hidden execution.')
    (out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps({k: v for k, v in manifest.items() if k != 'families'}, indent=2))


if __name__ == '__main__':
    main()
