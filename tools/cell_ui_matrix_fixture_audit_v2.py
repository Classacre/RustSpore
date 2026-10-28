"""Compare explicit source-matrix fixture with an independent f32 reference.

Supply eight eight-digit hexadecimal f32 bit patterns and a captured stdout file from the Rust example.
This tests the identity-local/axis-scale-parent branch only, not a game frame.
"""
import argparse
import math
import struct
from pathlib import Path


def f(v):
    x = struct.unpack('<f', struct.pack('<f', v))[0]
    if not math.isfinite(x):
        raise ValueError('nonfinite')
    return x


def mul(a, b): return f(a * b)
def add(a, b): return f(a + b)
def sub(a, b): return f(a - b)


def reference(values):
    l, t, r, b, sx, sy, tx, ty = [f(v) for v in values]
    cx, cy = mul(add(r, l), 0.5), mul(add(b, t), 0.5)
    a = [[1.,0.,0.,0.], [0.,1.,0.,0.], [0.,0.,1.,0.],
         [add(0.,mul(1.,cx)),add(0.,mul(1.,cy)),add(0.,mul(1.,0.)),add(1.,mul(1.,0.))]]
    p = [[sx,0.,0.,0.], [0.,sy,0.,0.], [0.,0.,1.,0.], [tx,ty,0.,1.]]
    out = [[0.]*4 for _ in range(4)]
    for row in range(4):
        for col in range(4):
            terms = [mul(a[row][k], p[k][col]) for k in range(4)]
            out[row][col] = add(add(add(terms[0], terms[1]), terms[2]), terms[3])
    hx, hy = mul(sub(r,l),0.5), mul(sub(b,t),0.5)
    for col in range(4):
        out[3][col] = sub(out[3][col], add(mul(out[0][col],hx), mul(out[1][col],hy)))
    return [f'{struct.unpack("<I", struct.pack("<f",v))[0]:08X}' for row in out for v in row]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('values', nargs=8)
    p.add_argument('--rust-output', type=Path, required=True)
    a = p.parse_args()
    if any(len(word) != 8 or any(c not in '0123456789abcdefABCDEF' for c in word) for word in a.values):
        raise ValueError('each input must be exactly eight hexadecimal digits')
    values = [struct.unpack('<f', struct.pack('<I', int(word, 16)))[0] for word in a.values]
    lines = [line.split('\t') for line in a.rust_output.read_text(encoding='utf-8').splitlines() if line.startswith('matrix\t')]
    if len(lines) != 1 or lines[0][1:] != reference(values):
        raise ValueError('source fixture matrix differs')
    print('All 16 matrix words agree for the supplied explicit fixture; no retail frame asserted.')

if __name__ == '__main__':
    main()
