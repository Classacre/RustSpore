#!/usr/bin/env python3
"""Prepare and independently verify bounded NativeSamples corpus data.

prepare scans only immediate *.package files in Data using dbpf_extract's index
and RefPack reader. Unsupported index layouts are reported by filename and
excluded; all other read/decode/size errors fail. verify compares every Rust
output float bit pattern to explicit rounded binary32 Python arithmetic.
No native executable, interpolation, resource override, or gameplay claim.
"""
import argparse
import hashlib
import json
import math
import struct
from collections import Counter
from pathlib import Path
from dbpf_extract import parse_index
from refpack import decompress

TARGET = 0x5CAA5E28
MAX_RECORDS = 100_000
MAX_TOTAL = 1_000_000_000

def u32(n): return struct.pack('<I', n)
def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''): h.update(chunk)
    return h.hexdigest()
def read_exact(f, n):
    data = f.read(n)
    if len(data) != n: raise ValueError('truncated stream')
    return data

def shape(raw):
    if not 36 <= len(raw) < 100_000: raise ValueError('native stream size outside [36,100000)')
    dims = raw[28:36].split(b'\0', 1)[0]
    count = math.prod(dims) if dims else 0
    if len(raw) != 36 + 3 * count: raise ValueError('non-exact sample extent')
    return dims, count

def prepare(data_dir, output, manifest):
    packages = sorted(data_dir.glob('*.package'))
    if not packages: raise ValueError('no immediate Data/*.package files')
    excluded, included, counts = [], [], Counter()
    records = total = 0
    with output.open('xb') as out:
        out.write(b'BDRAW001')
        for pkg in packages:
            try: entries, file_size = parse_index(pkg)
            except ValueError as e:
                if 'index layout not supported:' not in str(e): raise
                excluded.append({'package':pkg.name, 'reason':str(e)})
                continue
            selected = [e for e in entries if e['type'] == TARGET]
            included.append({'package':pkg.name, 'sha256':sha(pkg), 'entries':len(entries), 'selected':len(selected)})
            with pkg.open('rb') as f:
                for e in selected:
                    if e['offset'] + e['stored'] > file_size: raise ValueError(f'{pkg.name}: payload outside file')
                    f.seek(e['offset']); stored = read_exact(f, e['stored'])
                    raw = decompress(stored, out_size=None, header=True)[0] if e['dec'] > e['stored'] else stored
                    if len(raw) != e['dec']: raise ValueError(f"{pkg.name}: declared decoded length mismatch at {e['offset']}")
                    dims, count = shape(raw)
                    records += 1; total += len(raw)
                    if records > MAX_RECORDS or total > MAX_TOTAL: raise ValueError('corpus budget exceeded')
                    counts[dims.hex()] += 1
                    out.write(u32(len(raw))); out.write(raw)
        if records == 0: raise ValueError('no matching resources')
        out.write(u32(0))
    report = {'target_type':f'{TARGET:08X}', 'scope':'immediate Data/*.package, no override resolution',
              'included':included, 'excluded':excluded, 'records':records, 'raw_bytes':total,
              'dimensions_hex':dict(sorted(counts.items())), 'input_sha256':sha(output),
              'bounds':'36 <= decoded length < 100000; exact 36 + 3*product; <=100000 records; <=1GB raw'}
    with manifest.open('x', encoding='utf-8') as f: json.dump(report, f, indent=2); f.write('\n')
    print(json.dumps(report, indent=2))

def f32(x): return struct.unpack('<f', struct.pack('<f', x))[0]
def verify(source, output, manifest):
    report = json.loads(manifest.read_text(encoding='utf-8'))
    if sha(source) != report['input_sha256']: raise ValueError('source stream hash differs from manifest')
    records = samples = 0
    with source.open('rb') as src, output.open('rb') as out:
        if read_exact(src,8) != b'BDRAW001' or read_exact(out,8) != b'BDOUT001': raise ValueError('bad magic')
        while True:
            length = struct.unpack('<I', read_exact(src,4))[0]
            if not length: break
            if not 36 <= length < 100_000 or records >= MAX_RECORDS: raise ValueError('input bounds exceeded')
            raw = read_exact(src,length); dims, count = shape(raw)
            out_dims = read_exact(out,1)[0]
            if out_dims != len(dims) or read_exact(out,out_dims) != dims: raise ValueError(f'dimension mismatch record {records}')
            if struct.unpack('<I',read_exact(out,4))[0] != count: raise ValueError(f'count mismatch record {records}')
            if read_exact(out,24) != raw[:24]: raise ValueError(f'header mismatch record {records}')
            head = struct.unpack_from('<7f',raw)
            if not all(math.isfinite(x) for x in head): raise ValueError(f'nonfinite input record {records}; not covered')
            for i in range(count):
                packed = raw[36+3*i:39+3*i]
                first = [f32(head[j] - f32((packed[j] >> 4) * head[6])) for j in range(3)]
                second = [f32(head[j+3] + f32((packed[j] & 15) * head[6])) for j in range(3)]
                expected = struct.pack('<6f', *(first+second))
                actual = read_exact(out,24)
                if actual != expected: raise ValueError(f'float bits differ record {records} sample {i}: expected {expected.hex()} got {actual.hex()}')
                samples += 1
            records += 1
        if src.read(1) or out.read(1): raise ValueError('trailing data')
    if records != report['records']: raise ValueError('manifest record count mismatch')
    print(json.dumps({'records':records, 'samples':samples, 'all_sample_bits_equal':True,
                      'input_sha256':sha(source), 'rust_output_sha256':sha(output),
                      'exclusions':report['excluded'], 'original_spore_execution':False}, indent=2))

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest='action', required=True)
    p = subs.add_parser('prepare'); p.add_argument('data_dir',type=Path); p.add_argument('source',type=Path); p.add_argument('manifest',type=Path)
    p = subs.add_parser('verify'); p.add_argument('source',type=Path); p.add_argument('rust_output',type=Path); p.add_argument('manifest',type=Path)
    a = parser.parse_args()
    if a.action == 'prepare': prepare(a.data_dir,a.source,a.manifest)
    else: verify(a.source,a.rust_output,a.manifest)
if __name__ == '__main__': main()
