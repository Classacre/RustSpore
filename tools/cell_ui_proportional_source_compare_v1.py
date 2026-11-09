"""Compare Rust explicit-source intake with independent graph-audit records.

Run after building the example and generating a fresh graph report. This is
source decoding comparison, not native execution or a rendered HUD.
"""
import argparse
import json
from pathlib import Path
import subprocess
import struct


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('install', type=Path)
    p.add_argument('report', type=Path)
    p.add_argument('example', type=Path)
    args = p.parse_args()
    candidates = json.loads(args.report.read_text(encoding='utf-8'))['candidates']
    for candidate in candidates:
        run = subprocess.run([str(args.example.resolve()), str(args.install / candidate['package'])],
                             check=True, capture_output=True, text=True)
        records, drawables = {}, {}
        for line in run.stdout.splitlines():
            fields = line.split('\t')
            if fields[0] == 'drawable':
                if len(fields) != 10 or int(fields[1]) in drawables:
                    raise ValueError('malformed or duplicate drawable record')
                drawables[int(fields[1])] = fields[2:]
                continue
            if len(fields) != 5 or int(fields[0]) in records:
                raise ValueError('malformed or duplicate Rust record')
            records[int(fields[0])] = fields[1:]
        for expected in candidate['proportional_coefficients']:
            if records.get(expected['index']) != expected['edge_bits']:
                raise ValueError(f"coefficient mismatch: {candidate['package']} {expected['index']}")
        print(f"{candidate['package']}: {len(candidate['proportional_coefficients'])} progress procedures match")
        objects = {o['index']: o for c in candidate['controls'] for o in c['reachable_objects']
                   if o['class'] == 'B03C196F'}
        if len(objects) != 5:
            raise ValueError('expected five distinct progress drawable objects')
        for index, obj in objects.items():
            props = [p for p in obj['properties'] if p['id'] == '0F3AC75E']
            if len(props) != 1 or props[0]['type'] != '0013' or props[0]['count'] != 8:
                raise ValueError(f'invalid source slot property at {index}')
            words = [f'{word:04X}' for word in struct.unpack('<8H', bytes.fromhex(props[0]['value']))]
            if drawables.get(index) != words:
                raise ValueError(f"drawable slot mismatch: {candidate['package']} {index}")
        print(f"{candidate['package']}: {len(objects)}/5 progress drawable arrays, {8 * len(objects)}/40 words match; pointer resolution not inferred")


if __name__ == '__main__':
    main()
