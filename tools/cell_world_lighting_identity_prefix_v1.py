"""Audit a conditional 16-vector CellGame SH route, not a retail frame.

Uses saved PROP values from the explicitly supplied package. It independently
rounds each proven DIVSS/MULPS/ADDPS operation to binary32. The 16-vector
identity frame and fresh accumulation are explicit assumptions; actual frame,
program register count and overlay selection remain unverified.
"""
import argparse
from decimal import Decimal, localcontext
import hashlib
import json
from pathlib import Path
import struct

from dbpf_extract import payload
from native_archive_index_probe import decode
from source_camera_probe import properties

KEY = (0xA426730B, 0x00B1B104, 0x40200100)
RATIONAL = ((1,4),(3,4),(3,4),(3,4),(15,4),(15,4),(5,16),(15,4),
            (15,16),(35,32),(105,4),(21,32),(7,16),(21,32),(105,16),(35,32))
EXPECTED_IDS = {'0100EAB6','0100EAB7','0100EAB8','0100EAB9',
                '0100EABA','0100EAC4','0100EAC5'}


def f32(x):
    return struct.unpack('<f', struct.pack('<f', float(x)))[0]


def bits(x):
    return f'{struct.unpack("<I", struct.pack("<f", x))[0]:08X}'


def inspect(package):
    with package.open('rb') as source:
        header = source.read(96)
        if header[:4] != b'DBPF':
            raise ValueError('not a DBPF package')
        count, size, offset = (struct.unpack_from('<I', header, p)[0] for p in (36,44,64))
        if offset + size > package.stat().st_size:
            raise ValueError('index out of bounds')
        source.seek(offset)
        block = source.read(size)
    _, rows = decode(block, count)
    matches = [row for row in rows if row[:3] == KEY]
    if len(matches) != 1:
        raise ValueError('expected exactly one exact native key')
    _, _, _, off, stored, decoded, compression, _, _ = matches[0]
    if off + stored > package.stat().st_size or compression not in (0, 0xffff):
        raise ValueError('unsupported extent or compression')
    data = payload(package, dict(offset=off, stored=stored, dec=decoded))
    if len(data) != decoded:
        raise ValueError('decoded length mismatch')
    fields = properties(data)
    if {f['identifier'] for f in fields} != EXPECTED_IDS or len(fields) != 7:
        raise ValueError('unsupported preset properties or parent')
    by_id = {f['identifier']: f for f in fields}
    sh, exp = by_id['0100EAC5'], by_id['0100EAC4']
    if sh['type'] != '0032' or sh['count'] != 25 or sh['stride'] != 12:
        raise ValueError('unsupported coefficient layout')
    if exp['type'] != '000D' or exp['count'] != 1 or exp['specifier'] in ('001C','009C'):
        raise ValueError('unsupported exposure')
    exposure = exp['values'][0]
    pi = Decimal.from_float(f32(3.141592653589793))
    result = []
    with localcontext() as context:
        context.prec = 100
        for i, (numerator, denominator) in enumerate(RATIONAL):
            factor = f32((Decimal(numerator) / (Decimal(denominator) * pi)).sqrt())
            inverse = f32(1.0 / factor)
            rgb = sh['raw_le_components'][i]
            vector = []
            for value in (*rgb, 1.0):
                unpacked = f32(value * inverse)
                accumulated = f32(unpacked + 0.0)
                packed = f32(accumulated * factor)
                vector.append(bits(f32(packed * exposure)))
            result.append(vector)
    return dict(schema='cell-world-lighting-identity-prefix-v1', package=str(package),
                native_key=[f'{word:08X}' for word in KEY],
                payload_sha256=hashlib.sha256(data).hexdigest(), exposure_bits=exp['bits'][0],
                coefficient_count=25, output_vector_bits=result,
                limitations='Conditional first 16 only: identity frame, empty accumulation, no optional fields; no runtime or overlay claim.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    report = inspect(args.package)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    if args.out.exists():
        raise FileExistsError(args.out)
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('16 conditional SH vectors retained; identity frame and output count not established for retail')


if __name__ == '__main__':
    main()
