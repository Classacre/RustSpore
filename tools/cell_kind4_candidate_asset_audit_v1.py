"""Audit kind-4 candidate disk keys and explicit PROP-to-GMDL references.

Read only. Same-low-ID joins are hypotheses; PROP key values are references, not
proof of runtime resolution. No asset payloads are written. Requires the pinned
metadata scan and census, and writes a new directory under target.
"""
import argparse
import hashlib
import json
import struct
from collections import Counter
from pathlib import Path

from cell_hierarchy_76f930_compare_v1 import digest, inside_target
from cell_kind4_list_source_verify_v1 import native_rows
from prop_refs import read_value, Bad
from prop_parse import ARRAY_SPECIFIERS
from refpack import _read_header, decompress, RefPackError

CENSUS_SHA = '3de1933446c61546f8afcf44e22c1010a5405baf353d0d5c9259ea6071f4e7c6'
MATCH_SHA = '4fe634a732eb1d7fe8bd115de59ff37abe39873650be1843354d822ed0e7a13f'
TYPES = {'02D5C9AF', '0F43029A', '00B1B104', '00E6BCE5'}
MAX_SIZE = 16 * 1024 * 1024


def read_payload(path, row):
    offset, stored, decoded, compression = row[3:7]
    if min(offset, stored, decoded) < 0 or max(stored, decoded) > MAX_SIZE or offset + stored > path.stat().st_size:
        raise ValueError('invalid payload extent')
    with path.open('rb') as f:
        f.seek(offset)
        raw = f.read(stored)
    if len(raw) != stored:
        raise ValueError('short payload')
    if compression == 0xffff:
        header_end, _, declared = _read_header(raw, 0)
        if header_end > stored or declared != decoded:
            raise ValueError('invalid RefPack header')
        raw, _, info = decompress(raw, out_size=decoded, header=True)
        if info['declared_decompressed'] != decoded:
            raise ValueError('RefPack size disagreement')
    elif compression != 0 or stored != decoded:
        raise ValueError('unsupported compression or size')
    if len(raw) != decoded:
        raise ValueError('decoded size disagreement')
    return raw


def prop_gmdl_refs(data):
    # Strict bounds and exact consumption; only key records are interpreted.
    def take(fmt, pos):
        size = struct.calcsize(fmt)
        if pos + size > len(data):
            raise ValueError('truncated PROP framing')
        return struct.unpack_from(fmt, data, pos)[0], pos + size
    count, pos = take('>I', 0)
    if count > 200000:
        raise ValueError('PROP item limit')
    refs = []
    for _ in range(count):
        ident, pos = take('>I', pos)
        typ, pos = take('>H', pos)
        spec, pos = take('>H', pos)
        if spec in ARRAY_SPECIFIERS:
            n, pos = take('>I', pos)
            stride, pos = take('>I', pos)
            if n > 100000 or stride > 8192 or n * stride > len(data) - pos:
                raise ValueError('invalid PROP array extent')
            if typ == 0x20:
                if n and stride < 12:
                    raise ValueError('short PROP key stride')
                keys = [struct.unpack_from('<3I', data, pos + i * stride) for i in range(n)]
            else:
                keys = []
            pos += n * stride
        else:
            key, pos = read_value(data, pos, typ, spec, False)
            if pos > len(data):
                raise ValueError('PROP scalar extent')
            keys = [key] if key else []
        for instance_low, key_type, instance_high in keys:
            if key_type == 0x00E6BCE5:
                refs.append({'property_id': f'{ident:08X}',
                             'raw_words_le': [f'{instance_low:08X}', f'{key_type:08X}', f'{instance_high:08X}'],
                             'native_key': f'{instance_low:08X}:{key_type:08X}:{instance_high:08X}',
                             'scope_inherits_prop_context': instance_high == 0xFFFFFFFF})
    if pos != len(data):
        raise ValueError('PROP trailing bytes')
    return refs


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('install', type=Path)
    p.add_argument('--census', required=True, type=inside_target)
    p.add_argument('--matches', required=True, type=inside_target)
    p.add_argument('--out', required=True, type=inside_target)
    a = p.parse_args()
    install = a.install.resolve(strict=True)
    for file, sha in ((a.census, CENSUS_SHA), (a.matches, MATCH_SHA)):
        if not file.is_file() or file.stat().st_size > 128 * 1024 * 1024 or digest(file) != sha:
            raise ValueError('missing, oversized, or unpinned input')
    matches = json.loads(a.matches.read_bytes())
    if len(matches) != 42:
        raise ValueError('unexpected candidate count')
    pins = {}
    for match in matches:
        low, typ, scope = match['native_key'].split(':')
        if typ != '02D5C9AF' or low in pins:
            raise ValueError('invalid or duplicate candidate')
        pins[low] = (match['package'], scope, match['payload_sha256'])
    selected = []
    for line in a.census.read_text(encoding='utf-8').splitlines():
        r = line.split('\t')
        if r[0] != 'R':
            continue
        if len(r) != 12:
            raise ValueError('malformed census')
        if r[3] not in pins or r[4] not in TYPES:
            continue
        # Metadata is selected by the full pinned identity. Other types are
        # deliberately broader same-low-ID census joins, not associations.
        if r[4] == '02D5C9AF' and (r[1], r[5]) != pins[r[3]][:2]:
            continue
        selected.append(r)
    if len(selected) > 1000:
        raise ValueError('candidate record limit')
    paths = {}
    for r in selected:
        path = (install / r[1]).resolve(strict=True)
        if not path.is_relative_to(install) or not path.is_file():
            raise ValueError('package missing or outside install')
        paths[r[1]] = path
    before = {name: digest(path) for name, path in paths.items()}
    a.out.mkdir(parents=True, exist_ok=False)
    report = {'success': False, 'scope': 'disk keys and PROP references only; provider precedence, registration, selection, fixup and actors unverified',
              'census_sha256': CENSUS_SHA, 'matches_sha256': MATCH_SHA, 'package_sha256_before': before}
    try:
        indexes = {}
        for name, path in paths.items():
            idx = {}
            for row in native_rows(path):
                if row[:3] in idx:
                    raise ValueError('duplicate key in ' + name)
                idx[row[:3]] = row
            indexes[name] = idx
        results = []
        for r in selected:
            key = tuple(int(r[i], 16) for i in (3, 4, 5))
            row = indexes[r[1]].get(key)
            expected = (int(r[6]), int(r[7]), int(r[8]), int(r[9], 16))
            if row is None or row[3:7] != expected:
                raise ValueError('live index disagrees with census: ' + ':'.join(r[3:6]))
            raw = read_payload(paths[r[1]], row)
            sha = hashlib.sha256(raw).hexdigest()
            if r[4] == '02D5C9AF' and (r[1], r[5], sha) != pins[r[3]]:
                raise ValueError('metadata candidate disagrees with pinned scan')
            entry = {'package': r[1], 'native_key': ':'.join(r[3:6]),
                     'selection_basis': 'pinned_metadata_key' if r[4] == '02D5C9AF' else 'same_low_id_census_join',
                     'payload_sha256': sha,
                     'stored_bytes': row[4], 'decoded_bytes': row[5]}
            if r[4] == '00B1B104':
                refs = prop_gmdl_refs(raw)
                entry['explicit_gmdl_references'] = refs
            results.append(entry)
        found_metadata = [r for r in results if r['native_key'].split(':')[1] == '02D5C9AF']
        found_ids = [r['native_key'].split(':')[0] for r in found_metadata]
        if len(found_ids) != 42 or set(found_ids) != set(pins):
            raise ValueError('candidate metadata missing or ambiguous')
        by_key = {}
        for r in results:
            by_key.setdefault(r['native_key'], []).append(r['package'])
        for r in results:
            for ref in r.get('explicit_gmdl_references', []):
                ref['candidate_census_packages_with_exact_key'] = (
                    [] if ref['scope_inherits_prop_context'] else by_key.get(ref['native_key'], []))
        output = (json.dumps(results, indent=2) + '\n').encode()
        (a.out / 'candidates.json').write_bytes(output)
        report['output_sha256'] = hashlib.sha256(output).hexdigest()
        report['records_by_type'] = dict(Counter(r['native_key'].split(':')[1] for r in results))
        report['explicit_gmdl_references'] = sum(len(r.get('explicit_gmdl_references', [])) for r in results)
        report['exact_candidate_census_reference_hits'] = sum(bool(x['candidate_census_packages_with_exact_key']) for r in results for x in r.get('explicit_gmdl_references', []))
        report['success'] = True
    except (OSError, ValueError, Bad, RefPackError, IndexError, AssertionError, struct.error) as exc:
        report['error'] = str(exc)
    finally:
        try:
            after = {name: digest(path) for name, path in paths.items()}
            report['package_sha256_after'] = after
            report['census_sha256_after'] = digest(a.census)
            report['matches_sha256_after'] = digest(a.matches)
            if after != before or report['census_sha256_after'] != CENSUS_SHA or report['matches_sha256_after'] != MATCH_SHA:
                report['success'] = False
                report['error'] = 'input changed during audit'
        except OSError as exc:
            report['success'] = False
            report['error'] = 'input rehash failed: ' + str(exc)
        (a.out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    if not report['success']:
        raise ValueError('audit failed; see report')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
