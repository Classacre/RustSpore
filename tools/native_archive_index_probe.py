"""Independent flag-driven index census from the installed archive bytes.

Original reader8DAFE0 supplies the key/metadata semantics. This tool does not
use the historical record-shape parser, execute the game or extract payloads.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct


def decode(block, count):
    cursor = 0

    def take():
        nonlocal cursor
        value, = struct.unpack_from('<I', block, cursor)
        cursor += 4
        return value

    flags = take()
    assert flags & 4, 'native common-high flag is absent'
    common_type = take() if flags & 1 else None
    common_group = take() if flags & 2 else None
    assert take() == 0, 'native reader rejects nonzero common-high'
    rows = []
    for _ in range(count):
        type_id = take() if common_type is None else common_type
        group = take() if common_group is None else common_group
        instance, offset, size, decoded_size = take(), take(), take(), take()
        metadata = take() if size & 0x80000000 else None
        size &= 0x7fffffff
        compression = metadata & 0xffff if metadata is not None else (0 if size == decoded_size else 0xffff)
        committed = int(metadata is not None and bool(metadata & 0x10000))
        rows.append((instance, type_id, group, offset, size, decoded_size, compression, committed, metadata))
    assert cursor == len(block), (cursor, len(block))
    return flags, rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    paths = sorted(args.install.rglob('*.package'), key=lambda path: path.relative_to(args.install).as_posix().lower())
    packages = []
    totals = Counter()
    flag_counts = Counter()
    compression_counts = Counter()
    failures = []
    with (args.out / 'records.tsv').open('x', encoding='utf-8', newline='\n') as records:
        for path in paths:
            relative = path.relative_to(args.install).as_posix()
            try:
                file_bytes = path.stat().st_size
                with path.open('rb') as source:
                    head = source.read(96)
                    assert head[:4] == b'DBPF'
                    count, = struct.unpack_from('<I', head, 0x24)
                    length, = struct.unpack_from('<I', head, 0x2c)
                    offset, = struct.unpack_from('<I', head, 0x40)
                    source.seek(offset)
                    block = source.read(length)
                assert len(block) == length
                flags, rows = decode(block, count)
                records.write(f'P\t{relative}\t{flags:08X}\t{count}\t{length}\n')
                seen = set()
                for ordinal, row in enumerate(rows):
                    instance, type_id, group, at, size, decoded, compression, committed, metadata = row
                    key = instance, type_id, group
                    assert key not in seen, ('duplicate key', key)
                    seen.add(key)
                    assert at + size <= file_bytes
                    totals['records'] += 1
                    totals['metadata_words'] += metadata is not None
                    totals['coded_size_rule'] += decoded > size
                    totals['coded_metadata'] += compression == 0xffff
                    totals['compression_vs_size_mismatches'] += ((compression == 0xffff) != (decoded > size))
                    compression_counts[f'{compression:04X}'] += 1
                    meta = '-' if metadata is None else f'{metadata:08X}'
                    records.write(f'R\t{relative}\t{ordinal}\t{instance:08X}\t{type_id:08X}\t{group:08X}\t{at}\t{size}\t{decoded}\t{compression:04X}\t{committed}\t{meta}\n')
                flag_counts[f'{flags:08X}'] += 1
                packages.append(dict(package=relative, records=count, flags=f'{flags:08X}',
                    index_bytes=length, index_sha256=hashlib.sha256(block).hexdigest(),
                    header_sha256=hashlib.sha256(head).hexdigest(), unique_keys=len(seen)))
            except Exception as error:
                failures.append(dict(package=relative, error=str(error)))
    report = dict(schema='native-archive-index-census-v1', packages_discovered=len(paths),
        packages_parsed=len(packages), totals=dict(totals), flag_package_counts=dict(flag_counts),
        compression_record_counts=dict(compression_counts), failures=failures, packages=packages,
        records_sha256=hashlib.sha256((args.out/'records.tsv').read_bytes()).hexdigest(),
        source_reader='SporeApp.exe8DAFE0; exact native dossier803; no game execution',
        scope='Canonical low/type/scope and per-record metadata; historical shape labels are excluded.')
    (args.out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({key: value for key, value in report.items() if key != 'packages'}, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
