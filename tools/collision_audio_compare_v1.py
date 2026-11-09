#!/usr/bin/env python3
"""Independently compare the exploratory Rust collision-event report to package bytes.

The source-bound index parser and event walker are separate Python implementations.
This compares the report's observations; it does not prove event reference semantics.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

from native_archive_index_probe import decode
from refpack import decompress
from audio_event_v4 import walk


def index(path):
    with path.open('rb') as f:
        header = f.read(96)
        if header[:4] != b'DBPF':
            raise ValueError('not a DBPF package')
        count, size, offset = (struct.unpack_from('<I', header, n)[0] for n in (0x24, 0x2c, 0x40))
        f.seek(offset)
        block = f.read(size)
    if len(block) != size:
        raise ValueError('truncated index')
    return decode(block, count)


def payload(path, row):
    with path.open('rb') as f:
        f.seek(row[3])
        data = f.read(row[4])
    if len(data) != row[4]:
        raise ValueError('truncated payload')
    if row[6] == 0xffff:
        data = decompress(data, out_size=None, header=True)[0]
    elif row[6] != 0:
        raise ValueError('unsupported compression')
    if len(data) != row[5]:
        raise ValueError('decoded size mismatch')
    return data


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    if not __debug__:
        raise RuntimeError('comparison requires Python assertions enabled')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('receipt', type=Path, help='new JSON file under target')
    args = parser.parse_args()
    rust = json.loads(args.report.read_text(encoding='utf-8'))
    sources = {row['role']: Path(row['path']) for row in rust['inputs']}
    if set(sources) != {'audio1', 'patch_event', 'audio2_event'}:
        raise ValueError('unexpected input roles')
    tool_dir = Path(__file__).resolve().parent
    implementation = [Path(__file__).resolve()] + [tool_dir / name for name in ('native_archive_index_probe.py', 'refpack.py', 'audio_event_v4.py', 'dbpf_extract.py')]
    implementation_before = {str(path): file_sha(path) for path in implementation}
    before = {role: file_sha(path) for role, path in sources.items()}
    for row in rust['inputs']:
        if row['sha256_before'] != row['sha256_after'] or before[row['role']] != row['sha256_before']:
            raise ValueError('input hash does not match Rust report')
    flags, rows = index(sources['audio1'])
    census = rust['audio1']
    assert census['flags'] == f'{flags:08X}'
    assert census['records'] == len(rows)
    assert census['unique_native_keys'] == len({r[:3] for r in rows})
    assert census['type_counts'] == dict(Counter(f'{r[1]:08X}' for r in rows))
    checked = []
    for variant, role in zip(rust['variants'], ('patch_event', 'audio2_event'), strict=True):
        path = sources[role]
        if Path(variant['package']) != path:
            raise ValueError('variant input path differs')
        _, event_rows = index(path)
        matching = [r for r in event_rows if r[:3] == (0x34c2ff0b, 0x02b9f662, 0x021407ee)]
        if len(matching) != 1:
            raise ValueError('event key not unique')
        event = payload(path, matching[0])
        assert len(event) == variant['event_length'] and sha(event) == variant['event_sha256']
        assert walk(event)[0] == len(event)
        marker = bytes.fromhex('701ed91e')
        if event.count(marker) != 1:
            raise ValueError('sample marker not unique in event bytes')
        start = event.index(marker)
        typ, field_flags, count, stride = struct.unpack_from('>HHII', event, start + 4)
        assert variant['samples_type'] == f'{typ:04X}' and variant['samples_flags'] == f'{field_flags:04X}'
        if stride != 12 or len(event[start+16:start+16+count*stride]) != count*stride:
            raise ValueError('unexpected sample array span')
        assert len(variant['rows']) == count
        for i, result in enumerate(variant['rows']):
            raw = event[start+16+i*stride:start+16+(i+1)*stride]
            instance = int.from_bytes(raw[:4], 'little')
            assert raw.hex() == result['raw_hex'] and f'{instance:08X}' == result['candidate_instance_le']
            matches = [r for r in rows if r[0] == instance]
            assert len(matches) == len(result['matches'])
            for source, actual in zip(matches, result['matches'], strict=True):
                data = payload(sources['audio1'], source)
                assert actual['key_words'] == [f'{v:08X}' for v in source[:3]]
                assert actual['decoded_bytes'] == len(data) and actual['sha256'] == sha(data)
                assert actual['metadata_word'] == (None if source[8] is None else f'{source[8]:08X}')
        checked.append({'role': role, 'event_sha256': sha(event), 'rows_compared': count,
                        'distinct_candidate_instances': len({row['candidate_instance_le'] for row in variant['rows']}),
                        'match_counts': dict(Counter(str(len(row['matches'])) for row in variant['rows'])),
                        'candidate_types': dict(Counter(match['key_words'][1] for row in variant['rows'] for match in row['matches']))})
    after = {role: file_sha(path) for role, path in sources.items()}
    if before != after:
        raise ValueError('input hash changed during Python comparison')
    implementation_after = {str(path): file_sha(path) for path in implementation}
    if implementation_before != implementation_after:
        raise ValueError('comparison implementation changed during inspection')
    receipt = {'implementation': [{'path': path, 'sha256_before': value, 'sha256_after': implementation_after[path]} for path, value in implementation_before.items()], 'inputs': [{'role': role, 'path': str(sources[role]), 'sha256_before': before[role], 'sha256_after': after[role]} for role in sources], 'schema': 'collision-audio-independent-comparison-v1', 'rust_report': str(args.report),
               'rust_report_sha256': sha(args.report.read_bytes()), 'variants': checked,
               'audio1_records_censused': len(rows), 'audio1_unique_keys_observed': len({r[:3] for r in rows}), 'audio1_type_counts_observed': dict(Counter(f'{r[1]:08X}' for r in rows)), 'result': 'pass',
               'scope': 'Comparison of observations only; no source reference semantics or stable-file admission. Local imports occur before the first implementation hash.'}
    with args.receipt.open('x', encoding='utf-8') as f:
        json.dump(receipt, f, indent=2)
        f.write('\n')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()

