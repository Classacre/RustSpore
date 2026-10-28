"""Audit the two installed lighting-PROP candidates for native CellGame ID.

E5D9E0 passes A426730B through an unresolved virtual method. The matching
40200100 namespace is a candidate, not a proven result of that lookup. No game
assets are exported and no executable code is run.
"""
import argparse
import hashlib
import json
from pathlib import Path

from dbpf_extract import payload
from prop_numeric_probe import PROP, index
from source_camera_probe import properties

INSTANCE = 0x40200100A426730B
PACKAGES = ('PatchData.package', 'Spore_Graphics.package')


def inspect(install):
    rows = []
    for name in PACKAGES:
        package = install / 'Data' / name
        matches = [x for x in index(package) if x['type'] == PROP and x['instance'] == INSTANCE]
        if len(matches) != 1:
            raise ValueError(f'{name}: expected exactly one matching PROP; got {len(matches)}')
        entry = matches[0]
        data = payload(package, entry)
        if len(data) != entry['dec']:
            raise ValueError(f'{name}: decoded size mismatch')
        fields = properties(data)
        ids = [x['identifier'] for x in fields]
        if len(ids) != len(set(ids)):
            raise ValueError(f'{name}: duplicate property ID')
        sh = [x for x in fields if x['identifier'] == '0100EAC5']
        if len(sh) != 1 or sh[0]['type'] != '0032' or sh[0]['count'] != 25:
            raise ValueError(f'{name}: unexpected SH field')
        exposure = [x for x in fields if x['identifier'] == '0100EAC4']
        if len(exposure) != 1 or exposure[0]['type'] != '000D' or len(exposure[0]['bits']) != 1:
            raise ValueError(f'{name}: unexpected exposure field')
        # Include actual parsed typed values, not unused padding in compound fields.
        semantic = [{key: value for key, value in field.items()
                     if key not in ('offset', 'raw', 'stride')} for field in fields]
        rows.append(dict(package=name, key=f"{entry['group']:08X}:{PROP:08X}:{INSTANCE:016X}",
                         payload_sha256=hashlib.sha256(data).hexdigest(), fields=semantic))
    return dict(schema='cell-world-lighting-candidate-v1', records=rows,
                parsed_fields_equal=rows[0]['fields'] == rows[1]['fields'],
                status='Candidate only: native virtual lookup and overlay precedence unresolved.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    result = inspect(args.install)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    if args.out.exists():
        raise FileExistsError(args.out)
    args.out.write_text(json.dumps(result, indent=2, allow_nan=False) + '\n', encoding='utf-8')
    print(f"{len(result['records'])} candidates, parsed fields equal: {result['parsed_fields_equal']}")


if __name__ == '__main__':
    main()
