"""Audit exact copies of the GlobalUICell-3 SPUI candidate and image links.

Names are hash leads, not proof of screen selection. Output stays in a new
directory under target; installed packages are only opened for reading.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import struct

from dbpf_extract import payload, png_report
from gmdl_skin_probe import indexes
from spui_layout_probe import container, elements, fnv

INSTANCE = 0x4046420026B068D2
SPUI = 0x250FE9A2
IMAGE_CLASS = 0x01BE6B15
CALLER_CONTROL_IDS = {0x05AF4AF0, 0x05AF4BF0, 0x05AF4AF1,
                      0x05AF4BF1, 0x05AF4AF2, 0x05AF4BF2}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    args.out.resolve().relative_to(target)
    if args.out.exists():
        raise FileExistsError(args.out)
    if fnv('GlobalUICell-3') != INSTANCE & 0xffffffff:
        raise ValueError('candidate name hash mismatch')

    packages = sorted(args.install.rglob('*.package'))
    providers = defaultdict(list)
    candidates = []
    for package in packages:
        for entry in indexes(package):
            providers[(entry['type'], entry['instance'])].append((package, entry))
            if entry['type'] == SPUI and entry['instance'] == INSTANCE:
                candidates.append((package, entry))
    if not candidates:
        raise ValueError('candidate not found')
    records = []
    failures = []
    for package, entry in candidates:
        raw = payload(package, entry)
        if len(raw) != entry['dec']:
            raise ValueError('layout decoded size mismatch')
        header = container(raw)
        body = elements(raw, header['body_offset'])
        resources = header['images'] + header['atlases']
        base = len(resources) + header['hit_mask_count']
        by_index = {e['index']: e for e in body['records']}
        if len(by_index) != len(body['records']) or set(by_index) != set(range(base, base + len(header['proxies']))):
            raise ValueError('incomplete or duplicate object indices')
        graph = []
        images = []
        caller_controls = []
        for obj in body['records']:
            cls = header['proxies'][obj['index'] - base]
            refs = []
            for prop in obj['properties']:
                if prop['id'] == 'EEC1B001' and prop['type'] == '0009' and prop['count'] == 1:
                    value = struct.unpack('<I', bytes.fromhex(prop['value']))[0]
                    if value in CALLER_CONTROL_IDS:
                        caller_controls.append({'id': f'{value:08X}', 'object_index': obj['index'],
                                                'class': f'{cls:08X}'})
                if prop['type'] == '0013':
                    values = struct.unpack('<' + 'H' * prop['count'], bytes.fromhex(prop['value']))
                    refs.append({'property': prop['id'], 'indices': list(values)})
            graph.append({'index': obj['index'], 'class': f'{cls:08X}', 'root': obj['root'], 'references': refs})
            if cls != IMAGE_CLASS:
                continue
            props = {p['id']: p for p in obj['properties']}
            p = props['01BE0001']
            if p['type'] != '0013' or p['count'] != 1:
                raise ValueError('unexpected image resource property')
            index = struct.unpack('<H', bytes.fromhex(p['value']))[0]
            if index >= len(resources):
                raise ValueError('image reference outside resource table')
            low, typ, high = resources[index]
            instance = high << 32 | low
            images.append({'object_index': obj['index'], 'resource_index': index,
                           'type': f'{typ:08X}', 'instance': f'{instance:016X}'})
        resolved = []
        for typ, instance in sorted({(int(x['type'], 16), int(x['instance'], 16)) for x in images}):
            copies = []
            for image_package, image_entry in providers[(typ, instance)]:
                data = payload(image_package, image_entry)
                if len(data) != image_entry['dec']:
                    raise ValueError('image decoded size mismatch')
                ok, detail = png_report(data)
                if not ok:
                    failures.append(f'{typ:08X}:{instance:016X}:invalid_png')
                copies.append({'package': image_package.relative_to(args.install).as_posix(),
                    'legacy_archive_group_word': f"{image_entry['group']:08X}",
                    'sha256': digest(data), 'bytes': len(data), 'png_valid': ok, 'png_detail': detail})
            if not copies:
                failures.append(f'{typ:08X}:{instance:016X}')
            resolved.append({'type': f'{typ:08X}', 'instance': f'{instance:016X}', 'copies': copies})
        records.append({'package': package.relative_to(args.install).as_posix(),
            'legacy_archive_group_word': f"{entry['group']:08X}", 'sha256': digest(raw),
            'bytes': len(raw), 'resource_table_count': len(resources),
            'hit_mask_count': header['hit_mask_count'], 'objects': len(graph),
            'class_counts': dict(sorted(Counter(x['class'] for x in graph).items())),
            'caller_control_id_matches': caller_controls,
            'image_objects': images, 'image_providers': resolved, 'graph': graph})
    result = {'scope': 'Name-hash candidate only. References and archive copies do not establish active screen, overlay precedence, visual roles, event behavior, or rendered appearance.',
              'candidate': f'{SPUI:08X}:{INSTANCE:016X}', 'packages_scanned': len(packages),
              'records': records, 'image_failures': sorted(set(failures))}
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / 'report.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'packages_scanned': len(packages), 'copies': len(records),
        'objects': [r['objects'] for r in records], 'image_objects': [len(r['image_objects']) for r in records],
        'unique_image_keys': [len(r['image_providers']) for r in records], 'image_failures': result['image_failures']}, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
