"""Verify eight explicitly selected source-list projections using a pinned example.

Disk projections do not establish runtime fixup, resource selection or actor identity.
"""
import argparse
import hashlib
import json
import struct
import subprocess
from pathlib import Path

from cell_hierarchy_76f930_compare_v1 import TARGET, digest, expected_hash, inside_target
from dbpf_extract import payload
from gmdl_skin_probe import indexes
from native_archive_index_probe import decode

PACKAGE_SHA = 'cadbff237076b441e6981503f80538705aefa5863bb4c8d1b869f9db98d1adff'
TYPE = 0xF9C3D770
PINS = {
    0xD7E124A7: '791d5049ad6721625fd89b834c6ba1c216bed980b14a1300065c96eb40cff868',
    0xD7E124A4: '97415972c97b11cfc3d17a9f404483ea7f73d6f6c4782866d6896fb7da217045',
    0x4122E50F: '9c1d72d5fe1a7c59e4cbe1313f8326617869d432c05f6eb17cd1e3468a8706ed',
    0x4122E50C: 'ee18ae09a9901677e88fe547cbbee5edbd8aa76e6b7cda37b6a8e1b40f0d187a',
    0xB17DEBCF: '1e3c9756d20dce21fd1c5575f0256e5fb7e8d1eb200d4eba4aea74eb3817af17',
    0xB17DEBCC: '4802ddc259489cf393d0eb357b208e32286827c77e3bcb814bd36644cf3a9c50',
    0xBC092047: '0ce47b95537792d7e780c591e8408da5920593dfba08f71837b895d0953a0c5f',
    0xBC092044: 'd4418610112f75fa7651727b2efdb3cc947fe13656d94fcbca93f3898899e107',
}


def native_rows(path):
    with path.open('rb') as f:
        header = f.read(96)
        if len(header) != 96 or header[:4] != b'DBPF':
            raise ValueError('invalid package header')
        count = struct.unpack_from('<I', header, 0x24)[0]
        length = struct.unpack_from('<I', header, 0x2c)[0]
        offset = struct.unpack_from('<I', header, 0x40)[0]
        if count > 1_000_000 or length > 64 * 1024 * 1024 or offset + length > path.stat().st_size:
            raise ValueError('invalid index bounds')
        f.seek(offset)
        block = f.read(length)
    if len(block) != length:
        raise ValueError('truncated index')
    return decode(block, count)[1]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('package', type=Path)
    p.add_argument('--example', type=inside_target, required=True)
    p.add_argument('--example-sha256', type=expected_hash, required=True)
    p.add_argument('--out', type=inside_target, required=True)
    p.add_argument('--timeout', type=float, default=10)
    a = p.parse_args()
    if not 0 < a.timeout <= 60:
        p.error('timeout must be in (0, 60]')
    if not a.example.is_file() or a.example.suffix.lower() != '.exe' or digest(a.example) != a.example_sha256:
        raise ValueError('example executable missing or hash mismatch')
    if digest(a.package) != PACKAGE_SHA:
        raise ValueError('package hash mismatch')
    a.out.mkdir(parents=True, exist_ok=False)
    report = {'matched': False, 'completed': 0, 'package_sha256_before': PACKAGE_SHA,
              'example_sha256_before': a.example_sha256,
              'scope': 'selected disk projections only; runtime fixup, selection and actor identity unverified'}
    records = []
    try:
        rows = native_rows(a.package)
        legacy = indexes(a.package)
        for ident, pin in PINS.items():
            exact = [r for r in rows if r[0:3] == (ident, TYPE, 0)]
            candidates = [e for e in legacy if e['type'] == TYPE and e['instance'] == ident]
            if len(exact) != 1 or len(candidates) != 1:
                raise ValueError(f'{ident:08X}: missing or ambiguous native key')
            entry = candidates[0]
            row = exact[0]
            if (entry['offset'], entry['stored'], entry['dec']) != (row[3], row[4], row[5]):
                raise ValueError(f'{ident:08X}: legacy/native physical span mismatch')
            data = payload(a.package, entry)
            if hashlib.sha256(data).hexdigest() != pin or len(data) != 36 or struct.unpack_from('<II', data) != (1, 8):
                raise ValueError(f'{ident:08X}: payload mismatch')
            source_path = a.out / f'{ident:08X}.bin'
            with source_path.open('xb') as f:
                f.write(data)
            result = subprocess.run([str(a.example), str(source_path)], cwd=str(TARGET),
                                    stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                    timeout=a.timeout, creationflags=subprocess.CREATE_NO_WINDOW, check=False)
            expected = ('count=1 offset=8 payload_bytes=36\nrow 0:' +
                        ''.join(f' {byte:02x}' for byte in data[8:]) + '\n').encode('ascii')
            if result.returncode or result.stderr or result.stdout.replace(b'\r\n', b'\n') != expected:
                raise ValueError(f'{ident:08X}: example output mismatch or failure')
            if digest(source_path) != pin:
                raise ValueError(f'{ident:08X}: extracted file changed')
            records.append({'native_key': f'{ident:08X}:{TYPE:08X}:00000000',
                            'payload_sha256': pin, 'example_stdout_sha256': hashlib.sha256(expected).hexdigest(),
                            'row_bytes_match': True})
            report['completed'] += 1
        report['matched'] = True
    except (OSError, ValueError, subprocess.SubprocessError, AssertionError, struct.error) as exc:
        report['error'] = str(exc)
        raise
    finally:
        for label, path, expected in (('package', a.package, PACKAGE_SHA), ('example', a.example, a.example_sha256)):
            try:
                after = digest(path)
                report[label + '_sha256_after'] = after
                if after != expected:
                    report['matched'] = False
                    report['error'] = label + ' changed or failed pin verification'
            except OSError as exc:
                report['matched'] = False
                report['error'] = str(exc)
        report['records'] = records
        (a.out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    if not report['matched']:
        raise ValueError(report.get('error', 'verification failed'))
    print(json.dumps(report))


if __name__ == '__main__':
    main()
