"""Reproduce a bounded disk census of hash-pair record middle words.

Only aggregate counts and hashes are written to a new target directory. This
projects framing and raw words, not native runtime semantics or selection.
"""
import argparse
import collections
import hashlib
import json
import struct
from pathlib import Path

from cell_hierarchy_76f930_compare_v1 import digest, inside_target
from cell_kind4_list_source_verify_v1 import native_rows
from refpack import _read_header, decompress

PIN = '3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6'
MAX_PAYLOAD = 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--census', type=inside_target, required=True)
    parser.add_argument('--out', type=inside_target, required=True)
    args = parser.parse_args()
    root = args.install.resolve(strict=True)
    require(args.census.is_file() and args.census.stat().st_size <= 128 * 1024 * 1024,
            'invalid census path or size')
    data = args.census.read_bytes()
    require(sha(data) == PIN, 'census hash mismatch')
    records = []
    for line in data.decode('utf-8').splitlines():
        row = line.split('\t')
        if row[0] == 'R':
            require(len(row) == 12, 'invalid census row')
            if row[4] == '02D5C9AF':
                records.append(row)
    require(len(records) == 2590, 'unexpected corpus size')
    paths = {r[1]: (root / r[1]).resolve(strict=True) for r in records}
    require(len(paths) == 4 and all(p.is_relative_to(root) and p.is_file() for p in paths.values()),
            'invalid package paths')
    before = {name: digest(path) for name, path in paths.items()}
    indexes = {}
    for name, path in paths.items():
        rows = native_rows(path)
        indexes[name] = {row[:3]: row for row in rows}
        require(len(indexes[name]) == len(rows), 'duplicate native key')
    args.out.mkdir(parents=True, exist_ok=False)
    markers = [collections.Counter(), collections.Counter()]
    count_dist = [collections.Counter(), collections.Counter()]
    totals = [0, 0]
    combinations = collections.Counter()
    failures = []
    for r in records:
        try:
            path = paths[r[1]]
            offset, stored, decoded = map(int, r[6:9])
            require(min(offset, stored, decoded) >= 0 and max(stored, decoded) <= MAX_PAYLOAD
                    and offset + stored <= path.stat().st_size, 'invalid payload bounds')
            key = tuple(int(r[i], 16) for i in (3, 4, 5))
            row = indexes[r[1]].get(key)
            require(row is not None and row[3:7] == (offset, stored, decoded, 0xffff)
                    and int(r[9], 16) == 0xffff, 'live index mismatch')
            with path.open('rb') as source:
                source.seek(offset)
                encoded = source.read(stored)
            require(len(encoded) == stored, 'truncated encoded payload')
            _, _, declared = _read_header(encoded, 0)
            require(declared == decoded, 'RefPack size mismatch')
            raw, _, _ = decompress(encoded, out_size=decoded, header=True)
            require(len(raw) == decoded and len(raw) >= 12 and struct.unpack_from('>I', raw)[0] == 4,
                    'invalid decoded payload')
            cursor = 4
            sections = []
            for _ in range(2):
                require(cursor + 4 <= len(raw), 'truncated section count')
                n = struct.unpack_from('>I', raw, cursor)[0]
                cursor += 4
                require(n <= 4096 and cursor + 12 * n <= len(raw), 'invalid section extent')
                words = [struct.unpack_from('>III', raw, cursor + 12 * i) for i in range(n)]
                cursor += 12 * n
                sections.append(words)
            require(cursor == len(raw), 'trailing data')
            for i, words in enumerate(sections):
                count_dist[i][len(words)] += 1
                totals[i] += len(words)
                markers[i].update(f'{word[1]:08X}' for word in words)
            combinations[f'{len(sections[0])},{len(sections[1])}'] += 1
        except Exception as exc:
            failures.append({'package': r[1], 'key': ':'.join((r[3], r[4], r[5])), 'error': repr(exc)})
    after = {name: digest(path) for name, path in paths.items()}
    census_after = digest(args.census)
    report = {'scope': 'disk payload framing and marker census only; no runtime semantics or provider selection',
              'records_requested': len(records), 'records_success': len(records) - len(failures), 'failures': failures,
              'section_total_pairs': totals, 'section_counts_per_record': [dict(sorted(x.items())) for x in count_dist],
              'markers_per_section': [dict(sorted(x.items())) for x in markers],
              'section_count_combinations': dict(sorted(combinations.items())), 'package_sha256_before': before,
              'package_sha256_after': after, 'census_sha256': sha(data), 'census_sha256_after': census_after,
              'stable': before == after and census_after == PIN}
    encoded = (json.dumps(report, indent=2) + '\n').encode('utf-8')
    (args.out / 'report.json').write_bytes(encoded)
    print(json.dumps({'report_sha256': sha(encoded), 'records_success': report['records_success'],
                      'failures': len(failures), 'stable': report['stable'], 'section_total_pairs': totals,
                      'markers_per_section': report['markers_per_section'],
                      'section_counts_per_record': report['section_counts_per_record'],
                      'section_count_combinations': report['section_count_combinations']}))
    if failures or not report['stable']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
