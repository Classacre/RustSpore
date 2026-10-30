"""Independent finite binary32 query arithmetic used by comparison generators.

No draft adapter, command line, native execution, or FP-control validation.
"""
import math
import random
import struct

def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]

def bits(x):
    return struct.pack('<f', x)

def query(raw, coords, reverse=False, reassociate=False):
    dims = list(raw[28:36].split(b'\0', 1)[0])
    header = struct.unpack_from('<7f', raw)
    if not all(math.isfinite(v) for v in header): raise ValueError('nonfinite header')
    if not dims: return header[:6]
    assert len(raw) == 36 + 3 * math.prod(dims)
    strides = [math.prod(dims[i+1:]) for i in range(len(dims))]
    pairs = []
    for i, dim in enumerate(dims):
        x = coords[i] if i < len(coords) else 0.0
        if not math.isfinite(x): raise ValueError('nonfinite coordinate')
        x = min(max(x, 0.0), 1.0)
        scaled = f32(float(dim - 1) * x)
        lo = int(scaled)
        weight = f32(scaled - f32(float(lo)))
        hi = lo + 1 if weight > 0 and lo < dim - 1 else lo
        pairs.append((lo * strides[i], hi * strides[i], weight))

    def leaf(idx):
        if not 0 <= idx < math.prod(dims): raise ValueError('bad sample index')
        sample = raw[36 + idx * 3:39 + idx * 3]
        first = [f32(header[i] - f32(float(sample[i] >> 4) * header[6])) for i in range(3)]
        second = [f32(f32(float(sample[i] & 15) * header[6]) + header[i + 3]) for i in range(3)]
        result = first + second
        if not all(math.isfinite(x) for x in result): raise ValueError('nonfinite leaf')
        return result

    order = list(range(len(dims)))
    if reverse: order.reverse()
    def visit(level, idx):
        if level == len(order): return leaf(idx)
        axis = order[level]
        lo, hi, weight = pairs[axis]
        left = visit(level + 1, idx + lo)
        if lo == hi: return left
        right = visit(level + 1, idx + hi)
        if reassociate:
            result = [f32(f32(a * f32(1.0 - weight)) + f32(b * weight)) for a, b in zip(left, right)]
        else:
            result = [f32(a + f32(f32(b - a) * weight)) for a, b in zip(left, right)]
        if not all(math.isfinite(x) for x in result): raise ValueError('nonfinite blend')
        return result
    return visit(0, 0)

def make_case(rng):
    # Non-dyadic decimal values are deliberately rounded before storage.
    h = [f32(rng.uniform(-1000, 1000)) for _ in range(6)] + [f32(rng.uniform(0.001, 50))]
    raw = bytearray(struct.pack('<7f', *h) + bytes([2, 2, 0, 0, 0, 0, 0, 0]))
    raw.extend(rng.randrange(256) for _ in range(12))
    coords = [f32(rng.uniform(0.01, 0.99)), f32(rng.uniform(0.01, 0.99))]
    return bytes(raw), coords
