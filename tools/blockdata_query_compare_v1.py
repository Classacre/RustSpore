"""Prepare/compare bounded synthetic production query cases; never run Rust.

Reproduce from the repo root, using a fresh output directory:
  python -I -S tools/blockdata_query_compare_v1.py prepare target/blockdata-query-production-v1
  tools\build.bat run -p spore-core --example blockdata_query_compare_v1 -- target/blockdata-query-production-v1/input.bin target/blockdata-query-production-v1/rust.bin
  python -I -S tools/blockdata_query_compare_v1.py verify target/blockdata-query-production-v1 target/blockdata-query-production-v1/rust.bin

The comparison is model-vs-model and does not execute the original program.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
import blockdata_query_reference_v1 as ref

FILES = {
    'generator': Path(__file__),
    'reference': ROOT / 'tools/blockdata_query_reference_v1.py',
    'example': ROOT / 'crates/spore-core/examples/blockdata_query_compare_v1.rs',
    'query': ROOT / 'crates/spore-core/src/blockdata_query_v1.rs',
    'blockdata': ROOT / 'crates/spore-core/src/blockdata.rs',
}

def sha(data): return hashlib.sha256(data).hexdigest()
def pins(): return {name: sha(path.read_bytes()) for name, path in FILES.items()}

def prepare(out):
    rng = random.Random(0x42445131)
    cases, reversed_witness, reassociated_witness = [], None, None
    for _ in range(20_000):
        raw, coords = ref.make_case(rng)
        expected = b''.join(map(ref.bits, ref.query(raw, coords)))
        if reversed_witness is None and expected != b''.join(map(ref.bits, ref.query(raw, coords, reverse=True))):
            reversed_witness = len(cases)
        if reassociated_witness is None and expected != b''.join(map(ref.bits, ref.query(raw, coords, reassociate=True))):
            reassociated_witness = len(cases)
        cases.append((raw, coords, expected))
        if len(cases) >= 128 and reversed_witness is not None and reassociated_witness is not None: break
    if reversed_witness is None or reassociated_witness is None: raise RuntimeError('rounding witnesses absent')
    request, expected = bytearray(b'BDQIN001'), bytearray(b'BDQOUT01')
    for raw, coords, row in cases:
        request.extend(struct.pack('<II', len(raw), len(coords)))
        request.extend(raw)
        request.extend(struct.pack('<%df' % len(coords), *coords))
        expected.extend(row)
    request.extend(struct.pack('<II', 0, 0))
    out.resolve().relative_to((ROOT / 'target').resolve())
    out.mkdir(parents=True, exist_ok=False)
    (out / 'input.bin').write_bytes(request)
    (out / 'expected.bin').write_bytes(expected)
    report = {'schema':'blockdata-query-production-comparison-v1', 'cases':len(cases),
              'seed':'0x42445131', 'input_sha256':sha(request), 'expected_sha256':sha(expected),
              'pins':pins(), 'python_version':sys.version,
              'reverse_recursion_witness_case':reversed_witness,
              'reassociation_witness_case':reassociated_witness,
              'scope':'synthetic finite [2,2] grids; independently step-rounded model; not native execution',
              'original_program_executed':False}
    (out / 'manifest.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(report, indent=2))

def verify(directory, reply):
    manifest = json.loads((directory / 'manifest.json').read_text(encoding='utf-8'))
    if manifest['pins'] != pins(): raise ValueError('source pins differ')
    request = (directory / 'input.bin').read_bytes()
    expected = (directory / 'expected.bin').read_bytes()
    actual = reply.read_bytes()
    if sha(request) != manifest['input_sha256'] or sha(expected) != manifest['expected_sha256']:
        raise ValueError('corpus hash mismatch')
    if len(expected) != 8 + 24 * manifest['cases'] or len(actual) != len(expected): raise ValueError('reply extent mismatch')
    if expected[:8] != b'BDQOUT01' or actual[:8] != b'BDQOUT01': raise ValueError('reply magic')
    mismatches = [i for i in range(manifest['cases']) if actual[8+i*24:32+i*24] != expected[8+i*24:32+i*24]]
    print(json.dumps({'cases':manifest['cases'], 'mismatches':mismatches, 'reply_sha256':sha(actual),
                      'reply_provenance_authenticated':False, 'original_program_executed':False}))
    if mismatches: raise SystemExit(1)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    p = sub.add_parser('prepare'); p.add_argument('directory', type=Path)
    p = sub.add_parser('verify'); p.add_argument('directory', type=Path); p.add_argument('reply', type=Path)
    args = parser.parse_args()
    if args.action == 'prepare': prepare(args.directory)
    else: verify(args.directory, args.reply)

if __name__ == '__main__': main()
