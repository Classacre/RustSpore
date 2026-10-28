"""Compare empty-caller verifier TSV to segmented original instruction reference.

The reference and verifier outputs stay under target. This does not execute Rust.
"""
import hashlib
import json
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
GIDS = (5, 132, 219, 230)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main(reference, tsv, destination):
    reference, tsv, destination = map(Path, (reference, tsv, destination))
    for path in (reference, tsv, destination):
        path.resolve().relative_to((ROOT / 'target').resolve())
    destination.mkdir(exist_ok=False)
    actual = {}
    for line in tsv.read_text().splitlines():
        fields = line.split('\t')
        gid, kind = int(fields[0]), fields[1]
        if gid not in GIDS: raise ValueError(f'unexpected gid {gid}')
        if kind in ('helper', 'point', 'alpha'):
            key = (gid, kind, int(fields[2])); values = tuple(map(int, fields[3:]))
        elif kind in ('advance', 'metrics'):
            key = (gid, kind); values = tuple(map(int, fields[2:]))
        else: raise ValueError(f'unexpected kind {kind}')
        if key in actual: raise ValueError(f'duplicate {key}')
        actual[key] = values
    expected = {}
    for gid in GIDS:
        caller = json.loads((reference / f'gid{gid}.json').read_text())
        raster = json.loads((reference / f'gid{gid}-raster.json').read_text())
        assert caller['gid'] == gid and caller['branch'] == 'Bypassed' and not caller['vm_entered'] and raster['blocker'] is None
        for kind, field in [('helper','helper_output_points'), ('point','output_points')]:
            for i, point in enumerate(caller[field]): expected[gid, kind, i] = tuple(point)
        for i, v in enumerate(bytes.fromhex(raster['alpha_hex'])): expected[gid, 'alpha', i] = (v,)
        m = raster['metrics']; meta = caller['metadata']
        expected[gid, 'metrics'] = (m['0C']-m['08'], m['14']-m['10'], m['20'], meta['0E4'], meta['0E0'])
        expected[gid, 'advance'] = (caller['original_advance16dot16'],)
    errors = [{'key': list(k), 'expected': expected.get(k), 'actual': actual.get(k)} for k in sorted(expected.keys() | actual.keys()) if expected.get(k) != actual.get(k)]
    report = {'schema':'font0-empty-caller-rust-comparison-v1', 'tool_sha256':sha(Path(__file__)), 'reference_manifest_sha256':sha(reference/'manifest.json'), 'tsv_sha256':sha(tsv), 'numeric_rows':len(expected), 'error_count':len(errors), 'errors':errors, 'scope':'font0 15ppem identity 0x8b mode0; caller helper/postcaller points, advance, raster alpha/dimensions/pitch and final aligned origins; not VM frame, graphics state, cache or lifetime'}
    (destination/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print('rows',len(expected),'errors',len(errors))
    if errors: raise SystemExit(1)

if __name__ == '__main__': main(*sys.argv[1:])
