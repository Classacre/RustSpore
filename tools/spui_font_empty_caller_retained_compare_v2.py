"""Compare empty-program caller/raster evidence to retained mode-0 captures.

Read-only: no native execution and no reliance on the historical fake VM frame.
"""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GIDS = (5, 132, 219, 230)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main(reference, destination):
    reference = Path(reference)
    destination = Path(destination)
    destination.resolve().relative_to((ROOT / 'target').resolve())
    destination.mkdir(exist_ok=False)
    rows = []
    for gid in GIDS:
        caller_path = reference / f'gid{gid}.json'
        raster_path = reference / f'gid{gid}-raster.json'
        caller = json.loads(caller_path.read_text())
        raster = json.loads(raster_path.read_text())
        assert caller['gid'] == gid and caller['branch'] == 'Bypassed'
        assert not caller['vm_entered'] and raster['blocker'] is None
        m = raster['metrics']
        expected = (m['0C'] - m['08'], m['14'] - m['10'], m['20'], caller['metadata']['0E4'] & 0xffffffff, caller['metadata']['0E0'] & 0xffffffff, caller['original_advance16dot16'] & 0xffffffff)
        observations = []
        for path in sorted((ROOT / 'target/source-ui/font-glyph-corpus-v1/results').glob('*/attempt-*/parsed.json')):
            parsed = json.loads(path.read_text())
            if parsed['mode'] != 0:
                continue
            for glyph in parsed['glyphs']:
                if glyph['glyph_index'] != gid:
                    continue
                alpha = path.parent / 'active' / f"glyph-{glyph['ordinal']:02d}-{gid:03d}.alpha"
                actual = tuple(glyph[k] for k in ('width', 'height', 'pitch', 'offset_e4_raw', 'offset_e0_raw', 'advance_d0_raw'))
                observations.append({'parsed': str(path.relative_to(ROOT)), 'parsed_sha256': digest(path), 'alpha_sha256': digest(alpha), 'alpha_exact': bytes.fromhex(raster['alpha_hex']) == alpha.read_bytes(), 'metrics_advance_exact': actual == expected})
        assert observations, f'no retained observations for {gid}'
        rows.append({'gid': gid, 'caller_sha256': digest(caller_path), 'raster_sha256': digest(raster_path), 'expected_metrics_advance': expected, 'observations': observations})
    report = {'schema': 'font0-empty-caller-retained-comparison-v2', 'tool_sha256': digest(Path(__file__)), 'reference_manifest_sha256': digest(reference / 'manifest.json'), 'new_native_execution': False, 'rust_execution': False, 'scope': 'retained font0 mode0 square15 fresh captures; alpha, final caller dimensions/offsets and advance only; helper bypass and graphics state inferred independently from source, not these captures', 'rows': rows}
    (destination / 'manifest.json').write_text(json.dumps(report, indent=2) + '\n')
    print([(r['gid'], len(r['observations']), all(x['alpha_exact'] and x['metrics_advance_exact'] for x in r['observations'])) for r in rows])

if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
