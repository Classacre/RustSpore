"""Compare original ordinary glyf packets with FontTools design data.

Writes one canonical row per glyph for the independent Rust verifier. It
does not execute a native scaler, glyph hints, component flattening or a
replacement rasterizer. Original payloads and generated rows stay target/.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

import fontTools
from fontTools.ttLib import TTFont
from native_archive_index_probe import decode as native_index
from refpack import decompress
from refpack_bulk import install as checked_bulk_copy

ROOT = Path(__file__).resolve().parent.parent
FACES = {
    '511B49CA': 'f6ad6f1e12e2d2a7c1e28b0ea3cb93a653482840b2b6340fb8682f3eb4c75f26',
    '61639DFB': 'c86cdd68a8de8e2ae2df6e3e3fc7bb49f6f1853c2d2ebb9336c5f43c19a224c6',
    '6BBC4E0E': '30f4c0f2b86cb997e53107956f7991d0a85fb2e6b80861ba8ffd76de6e9e9c37',
    'BACAAAA5': '011841006478c327a8b9b3639701b3758b2d18738dfe1a22e047e28398ab3e3d',
    'FFE355DF': 'de9963a8da834c0e573215fe8edbb2f9926d9ef6dc642f2bfc21f29c2e748764',
}


def hexadecimal(data):
    return data.hex() if data else '-'


class Packet:
    def __init__(self, data):
        self.data, self.at = data, 0

    def take(self, n):
        assert 0 <= n <= len(self.data) - self.at
        result = self.data[self.at:self.at + n]
        self.at += n
        return result

    def u8(self):
        return self.take(1)[0]

    def u16(self):
        return struct.unpack('>H', self.take(2))[0]

    def s16(self):
        return struct.unpack('>h', self.take(2))[0]

    def instructions(self):
        n = self.u16()
        assert n <= 32767
        return self.take(n)


def decode(data, glyph_count):
    p = Packet(data)
    if not data:
        return dict(kind='E', bounds=None, end_points=[], instructions=b'', points=[], components=[], remainder=b'')
    contours = p.s16()
    result = dict(bounds=[p.s16() for _ in range(4)], end_points=[], instructions=b'', points=[], components=[])
    if contours == 0:
        result['kind'] = 'Z'
    elif contours > 0:
        result['kind'] = 'S'
        ends = [p.u16() for _ in range(contours)]
        assert all(a < b for a, b in zip(ends, ends[1:]))
        count = ends[-1] + 1
        assert count <= 32767
        result['end_points'] = ends
        result['instructions'] = p.instructions()
        flags = []
        while len(flags) < count:
            flag = p.u8()
            repeat = p.u8() + 1 if flag & 8 else 1
            assert repeat <= count - len(flags)
            flags.extend([flag] * repeat)
        coordinates = []
        for short, same in [(2, 16), (4, 32)]:
            values, position = [], 0
            for flag in flags:
                if flag & short:
                    magnitude = p.u8()
                    delta = magnitude if flag & same else -magnitude
                else:
                    delta = 0 if flag & same else p.s16()
                position = ((position + delta + 32768) & 0xFFFF) - 32768
                values.append(position)
            coordinates.append(values)
        result['points'] = list(zip(*coordinates, flags))
    else:
        result['kind'] = f'C:{contours}'
        while True:
            flags, glyph = p.u16(), p.u16()
            assert glyph < glyph_count
            if flags & 1:
                a, b = p.u16(), p.u16()
                args = [a - 65536 if a & 0x8000 else a, b - 65536 if b & 0x8000 else b] if flags & 2 else [a, b]
            else:
                a, b = p.u8(), p.u8()
                args = [a - 256 if a & 128 else a, b - 256 if b & 128 else b] if flags & 2 else [a, b]
            if flags & 8:
                scale = p.s16()
                transform = [scale, 0, 0, scale]
            elif flags & 64:
                x, y = p.s16(), p.s16()
                transform = [x, 0, 0, y]
            elif flags & 128:
                transform = [p.s16() for _ in range(4)]
            else:
                transform = [16384, 0, 0, 16384]
            result['components'].append(dict(flags=flags, glyph=glyph, args=args,
                                             args_kind='xy' if flags & 2 else 'match', transform=transform))
            if not flags & 32:
                break
        if flags & 256:
            result['instructions'] = p.instructions()
    result['remainder'] = data[p.at:]
    return result


def row(key, gid, packet):
    bounds = ','.join(map(str, packet['bounds'])) if packet['bounds'] is not None else '-'
    ends = ','.join(map(str, packet['end_points'])) or '-'
    points = ';'.join(','.join(map(str, p)) for p in packet['points']) or '-'
    components = ';'.join(','.join(map(str, [c['flags'], c['glyph'], c['args_kind'], *c['args'], *c['transform']]))
                          for c in packet['components']) or '-'
    return '\t'.join([key, 'glyph', str(gid), packet['kind'], bounds, ends,
                       hexadecimal(packet['instructions']), points, components, hexadecimal(packet['remainder'])]) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fonts', type=Path, required=True)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    for p in (args.out, args.report):
        p.resolve().relative_to((ROOT / 'target').resolve())
        assert not p.exists()
    with args.package.open('rb') as archive:
        header = archive.read(96)
        assert header[:4] == b'DBPF'
        count, index_bytes, index_offset = (struct.unpack_from('<I', header, at)[0] for at in (0x24, 0x2C, 0x40))
        archive.seek(index_offset)
        index_block = archive.read(index_bytes)
        assert len(index_block) == index_bytes
        flags, records = native_index(index_block, count)
        selected = {f'{r[0]:08X}': r for r in records if r[1] == 0x027C5CEF and r[2] == 0x0248E873}
        assert set(selected) == set(FACES)
        checked_bulk_copy()
        payloads = {}
        for suffix, record in selected.items():
            _, _, _, offset, size, decoded, compression, _, _ = record
            assert compression in (0, 0xFFFF)
            archive.seek(offset)
            stored = archive.read(size)
            assert len(stored) == size
            raw = stored if compression == 0 else decompress(stored, out_size=decoded, header=True)[0]
            assert len(raw) == decoded and hashlib.sha256(raw).hexdigest() == FACES[suffix]
            payloads[suffix] = raw
    inputs = []
    for suffix, digest in FACES.items():
        matches = list(args.fonts.glob(f'font-file-*-027C5CEF-0248E873{suffix}.bin'))
        assert len(matches) == 1
        path = matches[0]
        data = path.read_bytes()
        assert hashlib.sha256(data).hexdigest() == digest
        assert data == payloads[suffix]
        # Historical cache file names contain a compression-metadata label.
        # Native key fields come only from the flag-driven archive index.
        instance, type_id, scope = selected[suffix][:3]
        key = f'{type_id:08X}:{scope:08X}:{instance:08X}'
        inputs.append((key, path, data, digest))
    totals, faces, state = Counter(), [], hashlib.sha256()
    with args.out.open('xb') as output:
        for key, path, data, digest in sorted(inputs):
            font = TTFont(path)
            order = font.getGlyphOrder()
            locations = font['loca'].locations
            glyf = bytes(font.reader['glyf'])
            count = Counter()
            for gid, name in enumerate(order):
                packet = decode(glyf[locations[gid]:locations[gid + 1]], len(order))
                glyph = font['glyf'][name]
                assert packet['kind'] == ('E' if locations[gid] == locations[gid + 1] else
                                          'Z' if glyph.numberOfContours == 0 else
                                          'S' if glyph.numberOfContours > 0 else f'C:{glyph.numberOfContours}')
                if packet['bounds'] is not None:
                    assert packet['bounds'] == [glyph.xMin, glyph.yMin, glyph.xMax, glyph.yMax]
                if packet['kind'] == 'S':
                    assert packet['end_points'] == glyph.endPtsOfContours
                    assert [(x, y) for x, y, _ in packet['points']] == list(glyph.coordinates)
                    assert [flag & 1 for _, _, flag in packet['points']] == [flag & 1 for flag in glyph.flags]
                    assert packet['instructions'] == glyph.program.getBytecode()
                elif packet['kind'].startswith('C:'):
                    assert len(packet['components']) == len(glyph.components)
                    for raw, component in zip(packet['components'], glyph.components):
                        assert raw['glyph'] == font.getGlyphID(component.glyphName)
                        assert raw['flags'] & 0x1E04 == component.flags & 0x1E04
                        expected_args = [component.x, component.y] if hasattr(component, 'x') else [component.firstPt, component.secondPt]
                        assert raw['args'] == expected_args
                        transform = getattr(component, 'transform', [[1.0, 0.0], [0.0, 1.0]])
                        fixed = [v * 16384 for pair in transform for v in pair]
                        assert all(v == int(v) for v in fixed)
                        assert raw['transform'] == [int(v) for v in fixed]
                    assert packet['instructions'] == (glyph.program.getBytecode() if hasattr(glyph, 'program') else b'')
                encoded = row(key, gid, packet).encode('ascii')
                output.write(encoded)
                state.update(encoded)
                count.update(glyphs=1, points=len(packet['points']), components=len(packet['components']),
                             instructions=len(packet['instructions']))
                count[packet['kind'].split(':')[0]] += 1
            totals.update(count)
            record = selected[f'{int(key.split(":")[2], 16):08X}']
            faces.append(dict(path=str(path), key=key, legacy_cache_label=path.stem.split('-')[3],
                              bytes=len(data), sha256=digest, counts=dict(count),
                              source_offset=record[3], source_stored_size=record[4],
                              source_compression=f'{record[6]:04X}', source_metadata=record[8]))
    assert totals['glyphs'] == 51680 and len(faces) == 5
    report = dict(schema='source-font-outline-packet-fonttools-comparison-v1',
                  FontTools_version=fontTools.__version__, faces=faces, totals=dict(totals),
                  canonical_key_order='type:scope:instance', source_package=str(args.package),
                  source_index_flags=f'{flags:08X}', source_index_sha256=hashlib.sha256(index_block).hexdigest(),
                  cached_payloads_equal_fresh_canonical_records=True,
                  canonical_rows=totals['glyphs'], canonical_bytes=args.out.stat().st_size,
                  canonical_sha256=state.hexdigest(), native_host_executed=False,
                  scope='All five installed original ordinary glyf faces. Packet header/loca, contour endpoints, '
                  'simple coordinates/on-curve flags, original hint bytes and composite descriptors are checked '
                  'against FontTools. Full expanded source flags, raw component flags, packet tails and source '
                  'transform choice are retained in canonical rows. No component flattening, phantom points, '
                  'hint execution, native scaled outline capture or raster equivalence claim.')
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
