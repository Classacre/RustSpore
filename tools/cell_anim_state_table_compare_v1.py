"""Prepare and compare a Rust table projection with the hash-bound source audit.

Prepare writes a binary fixture only under target. Compare consumes stdout from
cell_anim_state_table_verify_v1 and checks every word and both accessors. This
script does not invoke Cargo; see --help for commands.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

SOURCE_SHA = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
TABLE_SHA = 'ac4aa6836350b90495f7cd5e20fb38e64986d89472d33f7b1ba8e5e5dfffcfda'
COUNT, STRIDE = 131, 36
TARGET = (Path(__file__).resolve().parent.parent / 'target').resolve()


def target_path(path):
    path = path.resolve()
    path.relative_to(TARGET)
    return path


def read_audit(path):
    audit = json.loads(target_path(path).read_text(encoding='utf-8'))
    if (audit['source_sha256'] != SOURCE_SHA or audit['table_sha256'] != TABLE_SHA
            or audit['start_va'] != '01484A20' or audit['end_va_exclusive'] != '01485C8C'
            or audit['record_count'] != COUNT or audit['record_size'] != STRIDE
            or len(audit['records']) != COUNT):
        raise ValueError('audit provenance or extent mismatch')
    raw = bytearray()
    expected = []
    for i, row in enumerate(audit['records']):
        record = bytes.fromhex(row['bytes_hex'])
        if len(record) != STRIDE or row['ordinal'] != i or int(row['va'], 16) != 0x01484A20 + i * STRIDE:
            raise ValueError(f'invalid record extent or ordinal {i}')
        words = list(struct.unpack('<9I', record))
        if ([f'{x:08X}' for x in words] != row['words_le_hex']
                or row['state_word'] != f'{words[0]:08X}'
                or row['request_key'] != f'{words[1]:08X}'):
            raise ValueError(f'audit projection mismatch at {i}')
        raw.extend(record)
        expected.append([i, *words, words[0], words[1]])
    if hashlib.sha256(raw).hexdigest() != TABLE_SHA:
        raise ValueError('table byte hash mismatch')
    return bytes(raw), expected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['prepare', 'compare'])
    parser.add_argument('--audit', type=Path, required=True)
    parser.add_argument('--file', type=Path, required=True,
                        help='fixture path for prepare, Rust stdout path for compare; under target')
    args = parser.parse_args()
    raw, expected = read_audit(args.audit)
    path = target_path(args.file)
    if args.mode == 'prepare':
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open('xb') as out:
            out.write(raw)
        print(f'Prepared {len(raw)} verified bytes at {path}')
    else:
        lines = path.read_text(encoding='utf-8-sig').splitlines()
        if len(lines) != COUNT:
            raise ValueError(f'expected {COUNT} lines, got {len(lines)}')
        for i, line in enumerate(lines):
            fields = line.split()
            if len(fields) != 12:
                raise ValueError(f'expected 12 fields at record {i}')
            try:
                actual = [int(fields[0], 10), *(int(x, 16) for x in fields[1:])]
            except ValueError as exc:
                raise ValueError(f'invalid field at record {i}') from exc
            if actual != expected[i]:
                raise ValueError(f'projection mismatch at record {i}')
        print(f'PASS: {COUNT} records, {COUNT * 9} raw words and {COUNT * 2} accessor values')


if __name__ == '__main__':
    main()
