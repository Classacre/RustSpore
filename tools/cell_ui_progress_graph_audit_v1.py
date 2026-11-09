"""Inventory explicit Cell SPUI progress controls and serialized references.

Serialized child order is not native paint order. Authored rectangles are not
runtime screen positions. No provider, visibility, or animation is selected.
"""
import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path
import struct

from dbpf_extract import payload
from gmdl_skin_probe import indexes
from spui_layout_probe import container, elements

KEY = (0x250FE9A2, 0x4046420026B068D2)
IDS = {0x051B54C0 + i for i in range(5)} | {0x051B5168, 0x051B7E48, 0xABCD1001}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent / 'target').resolve())
    if args.out.exists():
        raise FileExistsError(args.out)
    result = []
    for relative in ('Data/PatchData.package', 'Data/Spore_Graphics.package'):
        package = args.install / relative
        entries = [e for e in indexes(package) if (e['type'], e['instance']) == KEY]
        if len(entries) != 1:
            raise ValueError(f'{relative}: expected exactly one candidate')
        raw = payload(package, entries[0])
        if len(raw) != entries[0]['dec']:
            raise ValueError('decoded size mismatch')
        header = container(raw)
        body = elements(raw, header['body_offset'])
        resources = header['images'] + header['atlases']
        base = len(resources) + header['hit_mask_count']
        objects = {o['index']: o for o in body['records']}
        if set(objects) != set(range(base, base + len(header['proxies']))):
            raise ValueError('object index mismatch')
        def refs(o, prop_id):
            matches = [p for p in o['properties'] if p['id'] == prop_id]
            if len(matches) > 1:
                raise ValueError('duplicate references')
            if not matches:
                return []
            p = matches[0]
            if p['type'] != '0013':
                raise ValueError('unexpected reference type')
            return list(struct.unpack('<' + 'H' * p['count'], bytes.fromhex(p['value'])))
        parents = defaultdict(list)
        for i, o in objects.items():
            for position, child in enumerate(refs(o, 'EEC1B00B')):
                if child not in objects:
                    raise ValueError('missing child object')
                parents[child].append({'parent': i, 'serialized_ordinal': position})
        roots = [i for i, o in objects.items() if o['root']]
        if len(roots) != 1:
            raise ValueError('expected one root')
        paths = {}
        def walk(i, path):
            if i in paths:
                raise ValueError('cycle or multiple child paths')
            paths[i] = path + [i]
            for child in refs(objects[i], 'EEC1B00B'):
                walk(child, paths[i])
        walk(roots[0], [])
        def obj_record(i):
            obj = objects[i]
            return {'index': i, 'class': f"{header['proxies'][i-base]:08X}",
                    'properties': obj['properties'], 'child_parents': parents[i],
                    'serialized_root_path': paths.get(i)}
        controls = []
        for i, obj in objects.items():
            for p in obj['properties']:
                if p['id'] != 'EEC1B001' or p['type'] != '0009' or p['count'] != 1:
                    continue
                value = struct.unpack('<I', bytes.fromhex(p['value']))[0]
                if value not in IDS:
                    continue
                # Follow all serialized element references; preserve their property
                # IDs and raw fields to support later class-specific interpretation.
                pending, seen = [i], set()
                while pending:
                    n = pending.pop()
                    if n in seen:
                        continue
                    seen.add(n)
                    for prop in objects[n]['properties']:
                        if prop['type'] == '0013':
                            for ref in refs(objects[n], prop['id']):
                                if ref in objects:
                                    pending.append(ref)
                controls.append({'id': f'{value:08X}', 'index': i,
                                 'reachable_objects': [obj_record(n) for n in sorted(seen)]})
        if len(controls) != len(IDS):
            raise ValueError('expected one occurrence of each requested ID')
        proportional = {o['index']: o for c in controls for o in c['reachable_objects']
                        if o['class'] == 'AF3DF411'}
        coefficients = []
        for i, obj in sorted(proportional.items()):
            props = [p for p in obj['properties'] if p['id'] == '0F3D0000']
            if len(props) != 1 or props[0]['type'] != '000B' or props[0]['count'] != 4:
                raise ValueError(f'bad proportional property on {i}')
            bits = list(struct.unpack('<4I', bytes.fromhex(props[0]['value'])))
            coefficients.append({'index': i, 'edge_bits': [f'{x:08X}' for x in bits]})
        if len(coefficients) != 6:
            raise ValueError('expected six progress proportional procedures')
        result.append({'package': relative, 'type': f'{KEY[0]:08X}',
                       'instance': f'{KEY[1]:016X}', 'archive_group_word': f"{entries[0]['group']:08X}",
                       'sha256': hashlib.sha256(raw).hexdigest(), 'root': roots[0],
                       'resources_low_type_high': [[f'{x:08X}' for x in r] for r in resources],
                       'controls': controls, 'proportional_coefficients': coefficients,
                       'ancestor_objects': [obj_record(i) for i in sorted({n for c in controls for n in (paths.get(c['index']) or [])})]})
    report = {'scope': __doc__, 'candidates': result}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open('x', encoding='utf-8') as out:
        json.dump(report, out, indent=2)
        out.write('\n')
    print(f'Wrote {len(result)} explicit candidates to {args.out}')


if __name__ == '__main__':
    main()
