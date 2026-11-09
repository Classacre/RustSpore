"""Read-only SPUI census for static Cell progress-route control IDs.

Serialized ID co-occurrence is a candidate lead, not proof of a runtime layout,
package precedence, visibility, control behavior, or a rendered HUD.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from dbpf_extract import payload
from gmdl_skin_probe import indexes
from spui_layout_probe import container, elements

IDS = {0x051B54C0 + i for i in range(5)} | {0x051B5168, 0x051B7E48, 0xABCD1001}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    args.out.resolve().relative_to(target)
    if args.out.exists():
        raise FileExistsError(args.out)
    packages = sorted(args.install.rglob('*.package'))
    results, errors = [], []
    count = 0
    for package in packages:
        for entry in indexes(package):
            if entry['type'] != 0x250FE9A2:
                continue
            count += 1
            try:
                raw = payload(package, entry)
                if len(raw) != entry['dec']:
                    raise ValueError('decoded length mismatch')
                header = container(raw)
                body = elements(raw, header['body_offset'])
                base = len(header['images']) + len(header['atlases']) + header['hit_mask_count']
                matches = []
                for obj in body['records']:
                    for prop in obj['properties']:
                        if prop['id'] != 'EEC1B001' or prop['type'] != '0009' or prop['count'] != 1:
                            continue
                        value = struct.unpack('<I', bytes.fromhex(prop['value']))[0]
                        if value in IDS:
                            matches.append({'id': f'{value:08X}', 'object_index': obj['index'],
                                            'class': f"{header['proxies'][obj['index'] - base]:08X}"})
                if matches:
                    results.append({'package': package.relative_to(args.install).as_posix(),
                        'type': f"{entry['type']:08X}", 'instance': f"{entry['instance']:016X}",
                        'archive_group_word': f"{entry['group']:08X}",
                        'sha256': hashlib.sha256(raw).hexdigest(), 'matches': matches})
            except (ValueError, AssertionError, IndexError, KeyError, struct.error) as exc:
                errors.append({'package': package.relative_to(args.install).as_posix(),
                               'instance': f"{entry['instance']:016X}", 'error': str(exc)})
    report = {'scope': __doc__, 'ids': [f'{i:08X}' for i in sorted(IDS)],
              'packages_scanned': len(packages), 'spui_entries_scanned': count,
              'matching_copies': results, 'errors': errors}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open('x', encoding='utf-8') as output:
        json.dump(report, output, indent=2)
        output.write('\n')
    print(json.dumps(report, indent=2))
    if errors:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
