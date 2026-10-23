"""Independent length-prefixed RASTER mip framing and compressed-content check."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from dbpf_extract import payload
from gmdl_skin_probe import indexes, key


def mip_chain(data):
    version, width, height, levels, unknown, fmt = struct.unpack_from('<6I', data)
    assert version == 1 and width > 0 and height > 0
    assert 0 < levels <= max(width, height).bit_length()
    result = []
    at = 24
    for level in range(levels):
        size, = struct.unpack_from('<I', data, at)
        at += 4
        w, h = max(1, width >> level), max(1, height >> level)
        if fmt in [0x31545844, 0x33545844, 0x35545844]:
            expected = ((w+3)//4) * ((h+3)//4) * (8 if fmt == 0x31545844 else 16)
        else:
            assert fmt in [21, 50], hex(fmt)
            expected = w*h*(4 if fmt == 21 else 1)
        assert size == expected, (level, size, expected, hex(fmt))
        end = at + size
        assert end <= len(data), (level, at, size, len(data))
        result.append((w, h, data[at:end]))
        at = end
    assert at == len(data), (at, len(data))
    return (width, height, levels, fmt), result


def digest(parts):
    value = 0xcbf29ce484222325
    for part in parts:
        for byte in part:
            value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return value


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('--rust', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    expected = []
    packages = levels = pixels = 0
    formats = {}
    for path in sorted(args.install.rglob('*.package')):
        packages += 1
        for entry in indexes(path):
            if entry['type'] != 0x2f4e681c:
                continue
            header, chain = mip_chain(payload(path, entry))
            chunks = [struct.pack('<4I', *header)]
            for i, (w, h, source) in enumerate(chain):
                chunks.extend([struct.pack('<4I', i, w, h, len(source)), source])
                pixels += len(source)
            expected.append(f'{path}\t{key(entry)}\t{len(chain)}\t{digest(chunks):016X}')
            levels += len(chain)
            formats[f'{header[-1]:08X}'] = formats.get(f'{header[-1]:08X}', 0) + 1
    assert expected
    expected.sort()
    actual = sorted(args.rust.read_text().splitlines())
    report = dict(packages=packages, resources=len(expected), levels=levels,
        pixel_bytes=pixels, formats=formats, rust_records=len(actual), exact=actual == expected,
        sorted_lf_sha256=hashlib.sha256(('\n'.join(expected)+'\n').encode()).hexdigest())
    args.out.write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))
    assert report['exact'], 'source mip digest mismatch'


if __name__ == '__main__':
    main()
