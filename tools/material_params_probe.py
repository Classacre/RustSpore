"""Independent conversion of native 0x210 material constants from a tail report."""
import argparse
import hashlib
import json
from pathlib import Path
import struct


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('skin_report', type=Path)
    ap.add_argument('--rust', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--textures', type=Path, help='optional Rust texture binding TSV')
    args = ap.parse_args()
    report = json.loads(args.skin_report.read_text())
    expected, texture_rows, markers = [], [], 0
    for record in report['records']:
        for i, material in enumerate(record['tail']['materials']):
            for ident, raw in material['shader_data']:
                if ident == 0x20d:
                    data = bytes.fromhex(raw)
                    count, = struct.unpack_from('<I', data)
                    assert count <= 6 and len(data) == 4 + count*24
                    for j in range(count):
                        at = 4 + j*24
                        sampler, = struct.unpack_from('<I', data, at)
                        instance, = struct.unpack_from('<Q', data, at+16)
                        flag = data[at+8]
                        fields = [record['package'], record['key'], str(i), str(j),
                                  str(sampler), f'{instance:016X}']
                        for bit, byte in [(1, 9), (2, 10), (4, 11)]:
                            fields.append(str(struct.unpack_from('b', data, at+byte)[0]) if flag & bit else '-')
                        fields.append(str(data[at+12]) if flag & 8 else '-')
                        texture_rows.append('\t'.join(fields))
                if ident != 0x210:
                    continue
                data = bytes.fromhex(raw)
                assert len(data) >= 16
                has_w = data[1:4] == bytes([0x5c, 0x44, 0x26])
                assert not has_w or len(data) >= 20
                words = list(struct.unpack_from('<3I', data, 4))
                words.append(struct.unpack_from('<I', data, 16)[0] if has_w else 0x3f800000)
                expected.append('\t'.join([record['package'], record['key'], str(i),
                    str(data[0]), str(int(has_w))] + [f'{v:08X}' for v in words]))
                markers += has_w
    assert expected, 'no source parameters'
    actual = args.rust.read_text().splitlines()
    # Preserve duplicates: both resource identity and material slot are part of
    # the comparison, not merely a set of float values.
    expected.sort()
    actual.sort()
    result = dict(models=len(report['records']), parameters=len(expected),
        four_component_markers=markers, rust_rows=len(actual), exact=actual == expected,
        sorted_lf_sha256=hashlib.sha256(('\n'.join(expected)+'\n').encode()).hexdigest())
    if args.textures:
        texture_rows.sort()
        actual_textures = sorted(args.textures.read_text().splitlines())
        result['textures'] = dict(rows=len(texture_rows), rust_rows=len(actual_textures),
            exact=texture_rows == actual_textures,
            sorted_lf_sha256=hashlib.sha256(('\n'.join(texture_rows)+'\n').encode()).hexdigest())
    args.out.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    assert result['exact'], 'native parameter conversion disagrees'
    if args.textures:
        assert result['textures']['exact'], 'native texture bindings disagree'


if __name__ == '__main__':
    main()
