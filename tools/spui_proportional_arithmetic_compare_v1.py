"""Independent binary32 source-coefficient and synthetic arithmetic comparison.

Requires ordinary round-to-nearest ties-to-even and no FTZ/DAZ. Cases admitting
nonzero subnormal intermediate values are rejected. Never executes native code.
"""
import argparse
import json
import math
import random
import struct
import subprocess
from pathlib import Path


def bits(x):
    try:
        return struct.unpack('<I', struct.pack('<f', x))[0]
    except OverflowError:
        return 0x7f800000 if x > 0 else 0xff800000


def val(x):
    return struct.unpack('<f', struct.pack('<I', x))[0]


def normal_or_zero(x):
    b = bits(x) & 0x7fffffff
    return b == 0 or b >= 0x00800000


def reference(operation, words):
    values = list(map(val, words))
    if not all(map(math.isfinite, values)):
        return 'error'
    edges, parent, child = values[:4], values[4:8], values[8:]
    for i in (0, 2, 1, 3):
        span = val(bits(parent[2] - parent[0] if i in (0, 2) else parent[3] - parent[1]))
        delta = val(bits(span * edges[i]))
        result = val(bits(delta + child[i] if operation == 'add' else child[i] - delta))
        if any(math.isfinite(v) and not normal_or_zero(v) for v in (span, delta, result)):
            raise ValueError('subnormal intermediate is outside this fixture admission')
        child[i] = result
    if not all(map(math.isfinite, child)):
        return 'error'
    return '\t'.join(f'{bits(v):08X}' for v in child)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('graph_report', type=Path)
    p.add_argument('driver', type=Path)
    args = p.parse_args()
    report = json.loads(args.graph_report.read_text(encoding='utf-8'))
    coefficients = [[int(v, 16) for v in row['edge_bits']]
                    for c in report['candidates'] for row in c['proportional_coefficients']]
    if len(coefficients) != 12:
        raise ValueError('expected 12 source records')
    cases = []
    def add(edges, parent, child):
        words = edges + parent + child
        for op in ('add', 'sub'):
            cases.append((op, words, reference(op, words)))
    def f(values):
        return [bits(x) for x in values]
    for edge in coefficients:
        add(edge, f([25., -7., 846., 593.]), f([7., -8., -4.6, 26.]))
    add(f([1.0000001, 0., 0., 0.]), f([0., 0., 1.0000001, 1.]), f([-1.0000002, 5., -3., 1.]))
    add(f([1., -2., 0.5, 3.]), f([-5., 7., -10., 7.]), f([1., 2., 3., 4.]))
    # Nonfinite input and finite overflow are rejected by the Rust admission.
    add(f([1., 1., 1., 1.]), [0xff7fffff, 0, 0x7f7fffff, 0x3f800000], f([0., 0., 0., 0.]))
    add([0x7fc00000, 0, 0, 0], f([0., 0., 1., 1.]), f([0., 0., 0., 0.]))
    rng = random.Random(0x97EA70)
    for _ in range(500):
        add(f([rng.uniform(-2, 2) for _ in range(4)]),
            f([rng.uniform(-1000, 1000) for _ in range(4)]),
            f([rng.uniform(-1000, 1000) for _ in range(4)]))
    payload = ''.join(op + '\t' + '\t'.join(f'{b:08X}' for b in words) + '\n' for op, words, _ in cases)
    run = subprocess.run([str(args.driver.resolve())], input=payload, text=True, capture_output=True, check=True)
    actual = run.stdout.splitlines()
    if len(actual) != len(cases):
        raise ValueError(f'expected {len(cases)} rows, got {len(actual)}')
    for i, (got, (_, _, expected)) in enumerate(zip(actual, cases)):
        if got != expected:
            raise ValueError(f'row {i}: Rust {got}, independent reference {expected}')
    print(f'{len(cases)} bit-exact rows matched; 12 source coefficients, 500 seeded synthetic inputs and boundary cases; no native execution')


if __name__ == '__main__':
    main()
