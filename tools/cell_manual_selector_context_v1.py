"""Prepare and compare a hypothetical selector fixture; no live actor claim.

Requires an explicitly pinned locally built Rust example. Source table bytes and
reports are written only to a new target subdirectory. Does not build the example.
"""
import argparse
import hashlib
import json
import struct
import subprocess
from pathlib import Path
from cell_hierarchy_76f930_compare_v1 import TARGET, digest, expected_hash, inside_target

EXE_PIN = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
TABLE_PIN = 'ac4aa6836350b90495f7cd5e20fb38e64986d89472d33f7b1ba8e5e5dfffcfda'
TABLE_VA, SIZE = 0x1484a20, 131 * 36
EXPECTED = (b'CONTEXT state=0 actor_mode=0 parts=empty context=null predicates=all_true source_creature_association=none rng=explicit_draw\n'
            b'DRAW bits=00000000 rows=0,50,101 sum_bits=41700000 fallback=false key=AAAA0015\n'
            b'DRAW bits=40B00000 rows=0,50,101 sum_bits=41700000 fallback=false key=AAAA0049\n'
            b'DRAW bits=41280000 rows=0,50,101 sum_bits=41700000 fallback=false key=AAAA0097\n')


def table_from_exe(data):
    if hashlib.sha256(data).hexdigest() != EXE_PIN:
        raise ValueError('source executable hash mismatch')
    pe = struct.unpack_from('<I', data, 0x3c)[0]
    if data[:2] != b'MZ' or data[pe:pe+4] != b'PE\0\0':
        raise ValueError('invalid PE header')
    machine, count = struct.unpack_from('<HH', data, pe+4)
    optsize = struct.unpack_from('<H', data, pe+20)[0]
    opt = pe + 24
    if machine != 0x14c or struct.unpack_from('<H', data, opt)[0] != 0x10b:
        raise ValueError('expected x86 PE32')
    rva = TABLE_VA - struct.unpack_from('<I', data, opt+28)[0]
    sections = opt + optsize
    locations = []
    for i in range(count):
        _, va, size, offset = struct.unpack_from('<IIII', data, sections + 40*i + 8)
        if va <= rva and rva + SIZE <= va + size and offset + size <= len(data):
            locations.append(offset + rva - va)
    if len(locations) != 1:
        raise ValueError('table span is not uniquely file-backed')
    table = data[locations[0]:locations[0]+SIZE]
    if hashlib.sha256(table).hexdigest() != TABLE_PIN:
        raise ValueError('source table hash mismatch')
    return table


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source_exe', type=Path)
    parser.add_argument('--example', type=inside_target, required=True)
    parser.add_argument('--example-sha256', type=expected_hash, required=True)
    parser.add_argument('--out', type=inside_target, required=True)
    parser.add_argument('--timeout', type=float, default=30)
    args = parser.parse_args()
    if not 0 < args.timeout <= 60:
        parser.error('timeout must be in (0,60]')
    if not args.example.is_file() or args.example.suffix.lower() != '.exe' or digest(args.example) != args.example_sha256:
        raise ValueError('example hash mismatch')
    source = args.source_exe.resolve(strict=True)
    table = table_from_exe(source.read_bytes())
    args.out.mkdir(parents=True, exist_ok=False)
    local_table = args.out / 'table.bin'
    local_table.write_bytes(table)
    report = {'matched': False, 'scope': 'explicit hypothetical context only; no source creature association or native selection',
              'source_exe_sha256_before': EXE_PIN, 'source_table_sha256': TABLE_PIN,
              'example_sha256_before': args.example_sha256, 'expected_stdout_sha256': hashlib.sha256(EXPECTED).hexdigest()}
    try:
        result = subprocess.run([str(args.example), str(local_table)], cwd=str(TARGET),
                                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                timeout=args.timeout, creationflags=subprocess.CREATE_NO_WINDOW, check=False)
        output = result.stdout.replace(b'\r\n', b'\n')
        report.update(exit_code=result.returncode, stdout_sha256=hashlib.sha256(output).hexdigest(),
                      stderr_bytes=len(result.stderr))
        if result.returncode or result.stderr or output != EXPECTED:
            raise ValueError('selector output mismatch')
        report['matched'] = True
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        report['error'] = str(exc)
    finally:
        try:
            report['source_exe_sha256_after'] = digest(source)
            report['source_table_sha256_after'] = digest(local_table)
            report['example_sha256_after'] = digest(args.example)
            if (report['source_exe_sha256_after'] != EXE_PIN or report['source_table_sha256_after'] != TABLE_PIN
                    or report['example_sha256_after'] != args.example_sha256):
                report['matched'] = False
                report['error'] = 'input changed during comparison'
        except OSError as exc:
            report['matched'] = False
            report['error'] = f'input hash failed: {exc}'
        (args.out / 'report.json').write_bytes((json.dumps(report, indent=2) + '\n').encode('utf-8'))
    if not report['matched']:
        raise ValueError('selector comparison failed; see report')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
