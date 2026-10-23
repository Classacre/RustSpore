"""Independent rational/integer oracle for controlled x87 arithmetic.

No native floating arithmetic supplies the expected results. Fractions retain
exact operands; integer division/square-root rounding supplies each precision
and rounding lane. The process control word of retail SPORE remains unknown.
"""
import argparse
from fractions import Fraction
import hashlib
from math import isqrt
from pathlib import Path
import struct
import json


def power(exponent):
    return Fraction(1 << exponent) if exponent >= 0 else Fraction(1, 1 << -exponent)


def operand(bits):
    exponent, fraction = (bits >> 23) & 255, bits & 0x7fffff
    value = Fraction(fraction if exponent == 0 else fraction | 0x800000)
    value *= power(-149 if exponent == 0 else exponent - 150)
    return -value if bits >> 31 else value


def floor_log2(value):
    n, d = value.numerator, value.denominator
    exponent = n.bit_length() - d.bit_length()
    return exponent if value >= power(exponent) else exponent - 1


def round_integer(n, d, mode, negative):
    q, r = divmod(n, d)
    if mode == 0:
        up = r * 2 > d or (r * 2 == d and q & 1)
    else:
        up = bool(r) and ((mode == 1 and negative) or (mode == 2 and not negative))
    return q + int(up)


def quantize(value, precision, mode, negative_zero=False):
    negative = value < 0 or (value == 0 and negative_zero)
    if not value:
        return value, bytes(8) + struct.pack('<H', 0x8000 if negative else 0)
    magnitude = abs(value)
    exponent = floor_log2(magnitude)
    quantum = exponent - precision + 1
    scaled = magnitude / power(quantum)
    q = round_integer(scaled.numerator, scaled.denominator, mode, negative)
    if q == 1 << precision:
        q >>= 1
        exponent += 1
        quantum += 1
    exact = q * power(quantum)
    raw = struct.pack('<QH', q << (64 - precision), exponent + 16383 | (0x8000 if negative else 0))
    return -exact if negative else exact, raw


def sqrt_quantize(value, precision, mode):
    assert value > 0
    exponent = floor_log2(value) // 2
    quantum = exponent - precision + 1
    scaled = value / power(quantum * 2)
    n, d = scaled.numerator, scaled.denominator
    q = isqrt(n // d)
    if mode == 0:
        comparison = n * 4 - d * (q * 2 + 1) ** 2
        up = comparison > 0 or (comparison == 0 and q & 1)
    else:
        up = mode == 2 and q * q * d != n
    q += int(up)
    if q == 1 << precision:
        q >>= 1
        exponent += 1
        quantum += 1
    return q * power(quantum), struct.pack('<QH', q << (64 - precision), exponent + 16383)


def commit(value, mode, negative_zero=False):
    negative = value < 0 or (value == 0 and negative_zero)
    sign = 0x80000000 if negative else 0
    if not value:
        return sign
    magnitude = abs(value)
    exponent = floor_log2(magnitude)
    quantum = max(exponent - 23, -149)
    scaled = magnitude / power(quantum)
    q = round_integer(scaled.numerator, scaled.denominator, mode, negative)
    result = q * power(quantum)
    if result > operand(0x7f7fffff):
        if mode == 3 or (mode == 1 and not negative) or (mode == 2 and negative):
            return sign | 0x7f7fffff
        return None
    if not q:
        return sign
    exponent = floor_log2(result)
    if exponent < -126:
        return sign | int(result / power(-149))
    significand = int(result / power(exponent - 23))
    return sign | (exponent + 127) << 23 | (significand & 0x7fffff)


def compare(left, right):
    return int(left > right) - int(left < right)


def records():
    pairs = [
        (0x3f800000, 0x21800000), (0x3f800000, 0x33800000),
        (0x3f800000, 0xbf800000), (0x40000000, 0x40400000),
        (0x3f800001, 0x3f7fffff), (0x3f000001, 0xbf000000),
        (0x7f7fffff, 0x7f7fffff), (1, 0x7f7fffff),
        (0x80000001, 0x7f7fffff), (0x00800001, 0x00800000),
        (0x3b83126f, 0x3f800000), (0x3f7fffff, 0x34000001),
        (0xbf800001, 0x34000000), (0x4b800000, 0x3f800000),
        (0xcb800000, 0x3f800000), (0x3eaaaaab, 0x3f2aaaab),
    ]
    state = 0xc0ffee
    for _ in range(32):
        values = []
        for _ in range(2):
            state = (state * 1664525 + 1013904223) & 0xffffffff
            # Every sampled input is finite, nonzero and normal.
            values.append((state & 0x807fffff) | ((1 + ((state >> 23) % 254)) << 23))
        pairs.append(tuple(values))
    for precision, pc in [(24, 0), (53, 0x200), (64, 0x300)]:
        for mode in range(4):
            control = 0x7f | pc | (mode << 10)
            for a, b in pairs:
                for operation in 'ASMDQ':
                    aa = a & 0x7fffffff if operation == 'Q' else a
                    x, y = operand(aa), operand(b)
                    if operation == 'Q':
                        value, raw = sqrt_quantize(x, precision, mode)
                    else:
                        exact = {'A': lambda: x+y, 'S': lambda: x-y,
                                 'M': lambda: x*y, 'D': lambda: x/y}[operation]()
                        value, raw = quantize(exact, precision, mode, not exact and mode == 1)
                    negative_zero = raw[-1] & 0x80 != 0
                    stored = commit(value, mode, negative_zero)
                    fields = [f'{control:04X}', operation, f'{aa:08X}', f'{b:08X}', raw.hex().upper(),
                              '-' if stored is None else f'{stored:08X}', str(compare(value, x)),
                              '-' if stored is None else str(compare(value, operand(stored)))]
                    yield '\t'.join(fields)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixtures', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--rust', type=Path)
    args = parser.parse_args()
    rows = list(records())
    text = '\n'.join(rows) + '\n'
    if args.fixtures.exists():
        assert args.fixtures.read_text() == text, 'existing fixture differs'
    else:
        args.fixtures.parent.mkdir(parents=True, exist_ok=True)
        with args.fixtures.open('x', newline='\n') as stream:
            stream.write(text)
    report = dict(oracle='exact Fraction arithmetic and integer sqrt/rounding',
                  precisions=[24,53,64], rounding_modes=4, records=len(rows),
                  sha256=hashlib.sha256(text.encode()).hexdigest(), retail_control_word_known=False)
    if args.rust:
        actual = args.rust.read_text().splitlines()
        assert actual == rows, next(((i, a, b) for i, (a, b) in enumerate(zip(rows, actual)) if a != b),
                                   ('record count',len(rows),len(actual)))
        report.update(rust_records=len(actual), exact=True)
    with args.out.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
