"""Bounded source metadata census; no runtime registration or selection claim."""
import argparse
import hashlib
import json
import struct
from pathlib import Path

from cell_hierarchy_76f930_compare_v1 import digest, inside_target
from cell_kind4_list_source_verify_v1 import native_rows
from refpack import RefPackError, _read_header, decompress

CENSUS_SHA = '3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6'
TARGET_TYPE = '02D5C9AF'
FIELDS = {0x02DD90AF, 0xDDDD0000, 0xDDDD0001}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('install', type=Path)
    p.add_argument('--census', required=True, type=inside_target)
    p.add_argument('--out', required=True, type=inside_target)
    a = p.parse_args()
    install = a.install.resolve(strict=True)
    if not a.census.is_file() or a.census.stat().st_size > 128 * 1024 * 1024 or digest(a.census) != CENSUS_SHA:
        raise ValueError('census missing, oversized or hash mismatch')
    data = a.census.read_bytes()
    if hashlib.sha256(data).hexdigest() != CENSUS_SHA:
        raise ValueError('census changed while reading')
    records = []
    for line in data.decode('utf-8').splitlines():
        fields = line.split('\t')
        if fields[0] == 'R':
            if len(fields) != 12:
                raise ValueError('invalid census row')
            records.append(fields)
    if len(records) > 1_000_000:
        raise ValueError('census record limit')
    selected = [r for r in records if r[4] == TARGET_TYPE]
    if len(selected) != 2590:
        raise ValueError('unexpected metadata record count')
    packages = {}
    for r in selected:
        path = (install / r[1]).resolve(strict=True)
        if not path.is_relative_to(install) or not path.is_file():
            raise ValueError('package outside install or missing')
        packages[r[1]] = path
    before = {name: digest(path) for name, path in packages.items()}
    a.out.mkdir(parents=True, exist_ok=False)
    matches, failures = [], []
    report = {'matched': False, 'census_sha256': CENSUS_SHA, 'package_sha256_before': before,
              'scope': 'disk metadata and key joins only; registration, fixup and runtime selection unverified'}
    try:
        live = {}
        for name, path in packages.items():
            index = native_rows(path)
            by_key = {}
            for row in index:
                key = row[:3]
                if key in by_key:
                    raise ValueError(f'duplicate native key in {name}')
                by_key[key] = row
            live[name] = by_key
        for r in selected:
            try:
                path = packages[r[1]]
                offset, stored, decoded = (int(r[i]) for i in (6, 7, 8))
                if min(offset, stored, decoded) < 0 or max(stored, decoded) > 1024 * 1024 or offset + stored > path.stat().st_size:
                    raise ValueError('invalid payload span')
                key = tuple(int(r[i], 16) for i in (3, 4, 5))
                row = live[r[1]].get(key)
                if row is None or row[3:7] != (offset, stored, decoded, 0xffff) or int(r[9], 16) != 0xffff:
                    raise ValueError('live native key, span or compression tag mismatch')
                with path.open('rb') as source:
                    source.seek(offset)
                    encoded = source.read(stored)
                if len(encoded) != stored:
                    raise ValueError('truncated compressed payload')
                header_end, _, declared = _read_header(encoded, 0)
                if header_end > stored or declared != decoded:
                    raise ValueError('invalid RefPack header or declared size')
                raw, _, info = decompress(encoded, out_size=decoded, header=True)
                if info['declared_decompressed'] != decoded:
                    raise ValueError('RefPack declared size mismatch')
                if len(raw) != decoded or len(raw) < 12 or struct.unpack_from('>I', raw)[0] != 4:
                    raise ValueError('invalid version or decoded size')
                cursor = 4
                found = []
                for section in range(2):
                    if cursor + 4 > len(raw): raise ValueError('truncated count')
                    count = struct.unpack_from('>I', raw, cursor)[0]
                    cursor += 4
                    if count > 4096 or count * 12 > len(raw) - cursor:
                        raise ValueError('invalid section extent')
                    for i in range(count):
                        key, marker, value = struct.unpack_from('>III', raw, cursor + i * 12)
                        if key in FIELDS:
                            found.append({'section': section, 'key': f'{key:08X}',
                                          'marker': f'{marker:08X}', 'value': f'{value:08X}'})
                    cursor += count * 12
                if cursor != len(raw): raise ValueError('trailing data')
                if any(f['key'] == '02DD90AF' and f['marker'] == '02E1A75D' and f['value'] == '3D97A8E4' for f in found):
                    joins = [{'package': q[1], 'native_key': ':'.join((q[3], q[4], q[5]))}
                             for q in records if q[3] == r[3] and q[4] == '3D97A8E4' and q[5] == r[5]]
                    matches.append({'package': r[1], 'native_key': ':'.join((r[3], r[4], r[5])),
                                    'payload_sha256': hashlib.sha256(raw).hexdigest(), 'fields': found,
                                    'same_instance_scope_3d97a8e4': joins})
            except (OSError, ValueError, struct.error, RefPackError, IndexError) as exc:
                failures.append({'package': r[1], 'native_key': ':'.join((r[3], r[4], r[5])), 'error': str(exc)})
        report.update(records=len(selected), packages=len(packages), matches=len(matches), failures=failures)
        report['matched'] = not failures
    except (OSError, ValueError, struct.error, RefPackError, IndexError, AssertionError) as exc:
        report['error'] = str(exc)
    finally:
        after = {}
        for name, path in packages.items():
            try:
                after[name] = digest(path)
            except OSError as exc:
                report['error'] = f'package digest failed: {name}: {exc}'
        report['package_sha256_after'] = after
        try:
            report['census_sha256_after'] = digest(a.census)
        except OSError as exc:
            report['census_sha256_after'] = None
            report['error'] = f'census digest failed: {exc}'
        if after != before or report['census_sha256_after'] != CENSUS_SHA:
            report['matched'] = False
            report['error'] = 'input changed during scan'
        match_data = (json.dumps(matches, indent=2) + '\n').encode('utf-8')
        (a.out / 'matches.json').write_bytes(match_data)
        report['matches_sha256'] = hashlib.sha256(match_data).hexdigest()
        (a.out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    if not report['matched']:
        raise ValueError('metadata scan failed; see report')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
