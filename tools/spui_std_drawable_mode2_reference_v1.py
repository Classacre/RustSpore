"""Independent source-instruction arithmetic comparison for 95D120.

Explicit by-value test inputs only; no native execution. Assumes binary32
round-to-nearest ties-to-even, no FTZ/DAZ, and no pointer aliasing. Rejects
nonzero subnormal intermediates. Output is geometry before atlas mapping.
"""
import argparse
import math
import random
import struct
import subprocess
from pathlib import Path


def b(x):
    try:
        return struct.unpack('<I', struct.pack('<f', x))[0]
    except OverflowError:
        return 0x7f800000 if x > 0 else 0xff800000


def v(x):
    return struct.unpack('<f', struct.pack('<I', x))[0]


def q(x):
    result = v(b(x))
    bits = b(result) & 0x7fffffff
    if 0 < bits < 0x00800000:
        raise ValueError('subnormal outside fixture admission')
    return result


def reference(words):
    d = list(map(v, words[:4])); size = [struct.unpack('<i', struct.pack('<I', n))[0] for n in words[4:6]]
    uv = list(map(v, words[6:10])); split = list(map(v, words[10:14])); scale = list(map(v, words[14:16]))
    if not all(map(math.isfinite, d + uv + split + scale)):
        return 'error'
    # Source register order: SUBSS, CVTSI2SS, MULSS, MULSS, then
    # MULSS/ADDSS or SUBSS. Do not combine into fused operations.
    du = q(uv[2] - uv[0]); dv = q(uv[3] - uv[1])
    sx = q(q(q(float(size[0])) * du) * scale[0]); sy = q(q(q(float(size[1])) * dv) * scale[1])
    left = q(q(split[0] * sx) + d[0]); top = q(q(split[1] * sy) + d[1])
    right = q(d[2] - q(q(1.0 - split[2]) * sx)); bottom = q(d[3] - q(q(1.0 - split[3]) * sy))
    if left > right:
        right = left = q(q(right + left) * 0.5)
    if top > bottom:
        bottom = top = q(q(bottom + top) * 0.5)
    u1 = q(q(du * split[0]) + uv[0]); v1 = q(q(dv * split[1]) + uv[1])
    u2 = q(q(du * split[2]) + uv[0]); v2 = q(q(dv * split[3]) + uv[1])
    if not all(map(math.isfinite, [du, dv, sx, sy, left, top, right, bottom, u1, v1, u2, v2])):
        return 'error'
    # These are the individual argument stores at the nine indirect callsites,
    # written explicitly rather than deriving a generic nine-slice loop.
    calls = [([d[0], d[1], left, top], [uv[0], uv[1], u1, v1]),
             ([right, d[1], d[2], top], [u2, uv[1], uv[2], v1]),
             ([d[0], bottom, left, d[3]], [uv[0], v2, u1, uv[3]]),
             ([right, bottom, d[2], d[3]], [u2, v2, uv[2], uv[3]])]
    gap_x = q(right - left); gap_y = q(bottom - top)
    if not math.isfinite(gap_x) or not math.isfinite(gap_y):
        return 'error'
    if gap_x > 0:
        calls += [([left, d[1], right, top], [u1, uv[1], u2, v1]),
                  ([left, bottom, right, d[3]], [u1, v2, u2, uv[3]])]
    if gap_y > 0:
        calls += [([d[0], top, left, bottom], [uv[0], v1, u1, v2]),
                  ([right, top, d[2], bottom], [u2, v1, uv[2], v2])]
    if left != right and top != bottom:
        calls.append(([left, top, right, bottom], [u1, v1, u2, v2]))
    return '\t'.join([str(len(calls))] + [f'{b(x):08X}' for dest, src in calls for x in dest + src])


def main():
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('driver', type=Path); args = p.parse_args()
    cases = []
    def add(d, sz, uv, split, scale):
        words = [b(x) for x in d] + [x & 0xffffffff for x in sz] + [b(x) for x in uv + split + scale]
        cases.append((words, reference(words)))
    third = v(0x3eaaaaab); upper = q(1.0 - third)
    for dest in ([0., 0., 90., 60.], [0., 0., 2., 2.], [0., 0., 2., 60.], [0., 0., 90., 2.]):
        add(list(dest), [180, 29], [0., 0., 1., 1.], [third, third, upper, upper], [1., 1.])
    add([0., 0., 1., 1.], [1, 1], [0., 0., float('nan'), 1.], [0., 0., 1., 1.], [1., 1.])
    largest = v(0x7f7fffff)
    add([-largest, -1., largest, 1.], [0, 0], [0., 0., 1., 1.], [0., 0., 1., 1.], [1., 1.])
    rng = random.Random(0x95d120)
    for _ in range(500):
        x, y = rng.uniform(-100, 100), rng.uniform(-100, 100)
        add([x, y, x + rng.uniform(1, 500), y + rng.uniform(1, 500)], [rng.randrange(1, 1024), rng.randrange(1, 1024)],
            [0., 0., 1., 1.], [third, third, upper, upper], [1., 1.])
    for _ in range(200):
        x, y = rng.uniform(-100, 100), rng.uniform(-100, 100)
        u, vv = rng.uniform(-1, 1), rng.uniform(-1, 1)
        add([x, y, x + rng.uniform(1, 500), y + rng.uniform(1, 500)],
            [rng.randrange(1, 1024), rng.randrange(1, 1024)],
            [u, vv, u + rng.uniform(0.1, 2), vv + rng.uniform(0.1, 2)],
            [0.1, 0.2, 0.7, 0.8], [0.75, 1.25])
    text = ''.join('\t'.join(f'{x:08X}' for x in words) + '\n' for words, _ in cases)
    output = subprocess.run([str(args.driver.resolve())], input=text, text=True, capture_output=True, check=True).stdout.splitlines()
    if len(output) != len(cases): raise ValueError('wrong output count')
    for i, (got, (_, expected)) in enumerate(zip(output, cases)):
        if got != expected: raise ValueError(f'case {i}: expected {expected}, got {got}')
    print(f'{len(cases)} explicit fixture rows matched; source-derived split constants, synthetic rectangles; no native execution')


if __name__ == '__main__': main()
