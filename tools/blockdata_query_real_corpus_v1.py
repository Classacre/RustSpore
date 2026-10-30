"""Prepare/verify finite query models on the retained raw resource corpus.

Reproduce from the repository root with a fresh output directory:
  python -I -S tools/blockdata_query_real_corpus_v1.py prepare target/blockdata-corpus-v1/input.bin target/blockdata-corpus-v1/manifest.json target/blockdata-query-real-repro
  tools\build.bat run -p spore-core --example blockdata_query_compare_v1 -- target/blockdata-query-real-repro/input.bin target/blockdata-query-real-repro/rust.bin
  python -I -S tools/blockdata_query_real_corpus_v1.py verify target/blockdata-query-real-repro target/blockdata-query-real-repro/rust.bin

No Rust or original-game program is launched here. Inputs are package records,
not distinct assets or proof of runtime selection. Replies are model comparisons.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
import blockdata_query_reference_v1 as ref

SOURCE_SHA = 'faa9b17ea564a60e85496568a2b993bf4907566a108e2879805b303d9ab83907'
MAX_CASES = 20_000
FILES = {'generator': Path(__file__), 'reference': ROOT / 'tools/blockdata_query_reference_v1.py',
         'example': ROOT / 'crates/spore-core/examples/blockdata_query_compare_v1.rs',
         'query': ROOT / 'crates/spore-core/src/blockdata_query_v1.rs',
         'blockdata': ROOT / 'crates/spore-core/src/blockdata.rs'}

def sha(data): return hashlib.sha256(data).hexdigest()
def pins(): return {name: sha(path.read_bytes()) for name, path in FILES.items()}

def read_records(blob):
    if blob[:8] != b'BDRAW001': raise ValueError('bad input magic')
    pos, records = 8, []
    while True:
        if pos + 4 > len(blob): raise ValueError('truncated framing')
        length = struct.unpack_from('<I', blob, pos)[0]; pos += 4
        if length == 0: break
        if not 36 <= length < 100_000 or pos + length > len(blob): raise ValueError('invalid resource extent')
        raw = blob[pos:pos + length]; pos += length
        dims = raw[28:36].split(b'\0', 1)[0]
        count = math.prod(dims) if dims else 0
        if len(raw) != 36 + 3 * count: raise ValueError('resource shape mismatch')
        records.append((raw, len(dims)))
        if len(records) > MAX_CASES: raise ValueError('record limit')
    if pos != len(blob): raise ValueError('trailing source bytes')
    return records

def prepare(source, source_manifest, out):
    source_raw, manifest_raw = source.read_bytes(), source_manifest.read_bytes()
    manifest = json.loads(manifest_raw)
    if sha(source_raw) != SOURCE_SHA or manifest['input_sha256'] != SOURCE_SHA:
        raise ValueError('retained source corpus hash differs')
    records = read_records(source_raw)
    if len(records) != manifest['records']: raise ValueError('record count differs')
    request, expected = bytearray(b'BDQIN001'), bytearray(b'BDQOUT01')
    counts = Counter()
    excluded = Counter({'nonfinite_header_or_scale_records':0, 'nonfinite_or_overflow_cases':0})
    mapping = []
    for ordinal, (raw, dimensions) in enumerate(records):
        # A stricter policy than the source zero-dimensional path: all seven
        # header floats must be finite even if the scale will not be read.
        if not all(math.isfinite(x) for x in struct.unpack_from('<7f', raw)):
            excluded['nonfinite_header_or_scale_records'] += 1
            continue
        counts['finite_records'] += 1
        if dimensions == 0:
            counts['zero_dimension_records'] += 1
            coord_sets = [('header_only', [])]
        else:
            counts['nonzero_dimension_records'] += 1
            # Last sample, first sample and an interior asymmetric coordinate.
            coord_sets = [('lower', [0.0] * dimensions), ('upper', [1.0] * dimensions),
                          ('interior', [ref.f32(0.137 + i * 0.097) for i in range(dimensions)])]
        for label, coords in coord_sets:
            try:
                row = b''.join(map(ref.bits, ref.query(raw, coords)))
            except (ValueError, OverflowError) as error:
                excluded['nonfinite_or_overflow_cases'] += 1
                mapping.append({'source_ordinal':ordinal, 'policy':label, 'case_index':None, 'excluded':str(error)})
                continue
            if len(mapping) >= MAX_CASES: raise ValueError('case limit')
            request.extend(struct.pack('<II', len(raw), len(coords)))
            request.extend(raw)
            request.extend(struct.pack('<%df' % len(coords), *coords))
            expected.extend(row)
            mapping.append({'source_ordinal':ordinal, 'policy':label, 'case_index':counts['admitted_cases'], 'excluded':None})
            counts['admitted_cases'] += 1
    request.extend(struct.pack('<II', 0, 0))
    mapping_bytes = (json.dumps(mapping, separators=(',', ':')) + '\n').encode()
    out.resolve().relative_to((ROOT / 'target').resolve())
    out.mkdir(parents=True, exist_ok=False)
    (out / 'input.bin').write_bytes(request)
    (out / 'expected.bin').write_bytes(expected)
    (out / 'case-map.json').write_bytes(mapping_bytes)
    report = {'schema':'blockdata-query-real-corpus-model-v1', 'source_records':len(records),
              'counts':dict(counts), 'excluded':dict(excluded), 'cases':counts['admitted_cases'],
              'input_sha256':sha(request), 'expected_sha256':sha(expected), 'case_map_sha256':sha(mapping_bytes),
              'source_sha256':sha(source_raw), 'source_manifest_sha256':sha(manifest_raw),
              'source_package_exclusions':manifest['excluded'], 'pins':pins(), 'python_version':sys.version,
              'scope':'retained package records, finite header/scale and finite evaluated cases, three deterministic coordinates for nonzero dimensions; independent model, not original runtime',
              'original_program_executed':False}
    (out / 'manifest.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(report, indent=2))

def verify(directory, reply):
    manifest = json.loads((directory / 'manifest.json').read_text(encoding='utf-8'))
    if manifest['pins'] != pins(): raise ValueError('source pins differ')
    request, expected, mapping = ((directory / 'input.bin').read_bytes(),
                                  (directory / 'expected.bin').read_bytes(),
                                  (directory / 'case-map.json').read_bytes())
    if (sha(request), sha(expected), sha(mapping)) != (manifest['input_sha256'], manifest['expected_sha256'], manifest['case_map_sha256']):
        raise ValueError('corpus hash mismatch')
    actual = reply.read_bytes()
    if len(expected) != 8 + 24 * manifest['cases'] or len(actual) != len(expected): raise ValueError('reply extent mismatch')
    if expected[:8] != b'BDQOUT01' or actual[:8] != b'BDQOUT01': raise ValueError('reply magic')
    mismatches = [i for i in range(manifest['cases']) if actual[8+i*24:32+i*24] != expected[8+i*24:32+i*24]]
    print(json.dumps({'cases':manifest['cases'], 'mismatches':mismatches, 'reply_sha256':sha(actual),
                      'reply_provenance_authenticated':False, 'original_program_executed':False}))
    if mismatches: raise SystemExit(1)

def main():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest='action', required=True)
    a = sub.add_parser('prepare'); a.add_argument('source', type=Path); a.add_argument('source_manifest', type=Path); a.add_argument('directory', type=Path)
    a = sub.add_parser('verify'); a.add_argument('directory', type=Path); a.add_argument('reply', type=Path)
    args = p.parse_args()
    if args.action == 'prepare': prepare(args.source, args.source_manifest, args.directory)
    else: verify(args.directory, args.reply)

if __name__ == '__main__': main()
