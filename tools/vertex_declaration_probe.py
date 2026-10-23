"""Independent native selector feature census using original declaration bytes."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
from dbpf_extract import payload
from gmdl_skin_probe import indexes, key


def declarations(data):
    refs, = struct.unpack_from('>I', data, 4)
    at = 8 + refs * 12 + 4 + 28
    triangles, = struct.unpack_from('<I', data, at)
    at += 4
    for _ in range(triangles):
        _, count, bits, _ = struct.unpack_from('<4I', data, at)
        at += 16 + count * (2 if bits == 16 else 4)
    count, = struct.unpack_from('<I', data, at)
    at += 4
    formats = []
    for _ in range(count):
        size, = struct.unpack_from('<I', data, at)
        at += 4
        rows = [struct.unpack_from('<hhbbbbi', data, at + i * 12) for i in range(size)]
        at += 12 * size
        formats.append(rows)
    return formats


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('--rust', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    expected, semantics = [], Counter()
    packages = models = elements = 0
    # Native semantic table has 24 real entries, then the declaration END marker.
    native_table = {i: 2 ** i for i in range(24)}
    for path in sorted(args.install.rglob('*.package')):
        packages += 1
        for entry in indexes(path):
            if entry['type'] != 0x00E6BCE5:
                continue
            models += 1
            for i, rows in enumerate(declarations(payload(path, entry))):
                flags = 0
                for row in rows:
                    semantic = row[-1]
                    assert semantic in native_table, (str(path), key(entry), semantic)
                    flags |= native_table[semantic]
                    semantics[semantic] += 1
                elements += len(rows)
                expected.append(f'{path}\t{key(entry)}\t{i}\t{len(rows)}\t{flags:08X}')
    assert expected
    expected.sort()
    actual = sorted(args.rust.read_text().splitlines())
    result = dict(packages=packages, models=models, declarations=len(expected), elements=elements,
        semantics=dict(semantics), rust_rows=len(actual), exact=expected == actual,
        sorted_lf_sha256=hashlib.sha256(('\n'.join(expected)+'\n').encode()).hexdigest())
    args.out.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    assert result['exact'], 'native declaration feature mismatch'


if __name__ == '__main__':
    main()
