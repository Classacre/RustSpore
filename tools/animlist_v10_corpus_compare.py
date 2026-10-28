"""Compare all v10 resource records against the standalone Rust dump example.

Reads installed packages without modifying them. Decompressed payloads and dump
JSON stay in memory; the optional report must be placed under target/. The
package index helper has known layout coverage limits; failures are reported.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gmdl_skin_probe import indexes
from dbpf_extract import payload


def independent_parse(data):
    """Separate cursor and representation from Rust; preserve UTF-16 code units."""
    pos = 0
    def take(n):
        nonlocal pos
        if n < 0 or n > len(data) - pos:
            raise ValueError('truncated payload')
        result = data[pos:pos + n]
        pos += n
        return result
    def word():
        return struct.unpack('>I', take(4))[0]
    def string():
        return take(word() * 2).hex()
    if take(4) != b'tsla' or word() != 10:
        raise ValueError('not AnimList v10')
    count = word()
    if count > (len(data) - pos) // 35:
        raise ValueError('impossible record count')
    records = []
    for _ in range(count):
        key = word()
        name = string()
        before = [word(), word()]
        flags = list(take(3))
        after = [word(), word(), word()]
        nvariants = word()
        if nvariants > (len(data) - pos) // 8:
            raise ValueError('impossible variant count')
        variants = []
        for _ in range(nvariants):
            threshold = word()
            nentries = word()
            if nentries > (len(data) - pos) // 16:
                raise ValueError('impossible entry count')
            entries = []
            for _ in range(nentries):
                words = [word(), word(), word()]
                entries.append({'words': words, 'path_utf16le_hex': string()})
            variants.append({'threshold_bits': threshold, 'entries': entries})
        records.append({'key': key, 'name_utf16le_hex': name,
                        'words_before_flags': before, 'flags': flags,
                        'words_after_flags': after, 'variants': variants})
    if pos != len(data):
        raise ValueError('trailing bytes')
    return {'records': records}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('package_root', type=Path)
    p.add_argument('dump_executable', type=Path)
    p.add_argument('--report', type=Path, required=True)
    args = p.parse_args()
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    report = args.report.resolve()
    if not report.is_relative_to(target):
        p.error('report must be under target/')
    if not args.package_root.is_dir() or not args.dump_executable.is_file():
        p.error('package root and dump executable must exist')
    packages = sorted(args.package_root.rglob('*.package'))
    rows, errors, versions = [], [], {}
    for package in packages:
        try:
            entries = indexes(package)
        except Exception as exc:
            errors.append({'package': str(package), 'stage': 'index', 'error': str(exc)})
            continue
        for entry in entries:
            if entry['type'] != 0x4aeb6bc6:
                continue
            try:
                data = payload(package, entry)
                if len(data) < 8 or data[:4] != b'tsla':
                    raise ValueError('invalid header')
                version = int.from_bytes(data[4:8], 'big')
                versions[str(version)] = versions.get(str(version), 0) + 1
                if version != 10:
                    continue
                expected = independent_parse(data)
                run = subprocess.run([str(args.dump_executable.resolve())], input=data,
                                     capture_output=True, timeout=60, check=False)
                if run.returncode:
                    raise ValueError(f'dump exit {run.returncode}: {run.stderr.decode(errors="replace")[:500]}')
                actual = json.loads(run.stdout)
                rows.append({'package': str(package), 'key': entry,
                             'sha256': hashlib.sha256(data).hexdigest(),
                             'records': len(expected['records']), 'equal': actual == expected})
            except Exception as exc:
                errors.append({'package': str(package), 'key': entry, 'stage': 'resource', 'error': str(exc)})
    result = {'packages': len(packages), 'versions': versions, 'v10': rows, 'errors': errors}
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(f'packages={len(packages)} v10={len(rows)} matches={sum(r["equal"] for r in rows)} errors={len(errors)} report={report}')
    return 0 if packages and rows and not errors and all(r['equal'] for r in rows) else 1


if __name__ == '__main__':
    raise SystemExit(main())
