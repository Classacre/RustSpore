"""Retain exact source camera PROP records for a local install (no art export).

This named/count-bounded inspection uses the corrected primitive sizes, retains
raw bytes, and never assigns camera meanings from a plausible numeric value.
Native field and matrix evidence is recorded separately in re/cell-camera.md.
"""
import argparse
from functools import lru_cache
import json
from pathlib import Path
import struct

from dbpf_extract import payload
from prop_numeric_probe import index, PROP
from prop_v5 import value_sizes, is_separator

CAMERAS = {0x4041010002AD26C9, 0x4041010097F5B65C,
           0x404101003F37CA0F, 0x40410100D1A0765D}
ARRAYS = {0x1C, 0x9C}


def properties(data):
    count = struct.unpack_from('>I', data)[0]
    if count > 10000:
        raise ValueError('implausible property count')
    choices = {}

    @lru_cache(None)
    def solve(at, remaining):
        if remaining == 0:
            return at == len(data)
        if at + 8 > len(data):
            return False
        ident, typ, spec = struct.unpack_from('>IHH', data, at)
        for size in value_sizes(data, at + 8, typ, spec):
            end = at + 8 + size
            if end <= len(data) and solve(end, remaining - 1):
                choices[(at, remaining)] = (ident, typ, spec, end)
                return True
        # A separator may be considered only after a real named entry fails.
        if is_separator(data, at) and solve(at + 4, remaining):
            choices[(at, remaining)] = None
            return True
        return False

    if not solve(4, count):
        raise ValueError('named/count-bounded list does not consume exactly')
    rows = []
    at, remaining = 4, count
    while remaining:
        choice = choices[(at, remaining)]
        if choice is None:
            at += 4
            continue
        ident, typ, spec, end = choice
        start, number, stride = at + 8, 1, end - at - 8
        if spec in ARRAYS:
            number, stride = struct.unpack_from('>II', data, start)
            start += 8
        row = dict(offset=at, identifier=f'{ident:08X}', type=f'{typ:04X}',
                   specifier=f'{spec:04X}', count=number, stride=stride,
                   raw=data[at + 8:end].hex(' '))
        if typ in (9, 10, 13):
            words = [data[start + i * stride:start + i * stride + 4]
                     for i in range(number)]
            row['bits'] = [f'{struct.unpack(">I", word)[0]:08X}' for word in words]
            row['values'] = [struct.unpack({9: '>i', 10: '>I', 13: '>f'}[typ], word)[0]
                             for word in words]
        elif typ in (0x12, 0x13) and spec not in ARRAYS:
            chars = struct.unpack_from('>I', data, start)[0]
            row['text'] = data[start + 4:start + 4 + chars * (1 if typ == 0x12 else 2)].decode(
                'ascii' if typ == 0x12 else 'utf-16le', errors='strict')
        elif typ == 0x20:
            row['keys'] = []
            for i in range(number):
                low, key_type, high = struct.unpack_from('<III', data, start + i * stride)
                row['keys'].append(dict(type=f'{key_type:08X}', instance=f'{high:08X}{low:08X}'))
        elif typ in (0x30, 0x31, 0x32, 0x33, 0x34):
            dimension = {0x30: 2, 0x31: 3, 0x32: 3, 0x33: 4, 0x34: 4}[typ]
            row['raw_le_components'] = [list(struct.unpack_from('<' + 'f' * dimension,
                                          data, start + i * stride)) for i in range(number)]
        rows.append(row)
        at, remaining = end, remaining - 1
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--references', action='store_true', help='scan named PROP keys that reference the selected cameras')
    args = parser.parse_args()
    by_id, exclusions, references = {}, [], []
    for package in sorted(args.install.rglob('*.package')):
        try:
            entries = index(package)
        except Exception as error:
            exclusions.append(dict(package=str(package), error=str(error)))
            continue
        for entry in entries:
            if entry['type'] == PROP:
                by_id.setdefault(entry['instance'], []).append((package, entry))
                if args.references:
                    data = payload(package, entry)
                    if any(struct.pack('<I', camera & 0xFFFFFFFF) in data for camera in CAMERAS):
                        row = dict(package=str(package.relative_to(args.install)),
                                   key=f"{entry['group']:08X}:{PROP:08X}:{entry['instance']:016X}")
                        try:
                            props = properties(data)
                            row['labels'] = [prop.get('text') for prop in props if prop['identifier'] == '00B2CCCA']
                            row['camera_keys'] = [prop for prop in props if any(
                                int(key['instance'], 16) & 0xFFFFFFFF in {camera & 0xFFFFFFFF for camera in CAMERAS}
                                for key in prop.get('keys', []))]
                        except Exception as error:
                            row['error'] = str(error)
                        references.append(row)
    todo, done = list(CAMERAS), {}
    while todo:
        identity = todo.pop()
        if identity in done:
            continue
        records = []
        for package, entry in by_id.get(identity, []):
            data = payload(package, entry)
            row = dict(package=str(package.relative_to(args.install)),
                       key=f"{entry['group']:08X}:{PROP:08X}:{identity:016X}",
                       bytes=len(data), declared_count=struct.unpack_from('>I', data)[0],
                       properties=properties(data))
            for prop in row['properties']:
                if prop['identifier'] == '00B2CCCB':
                    for key in prop.get('keys', []):
                        parent = int(key['instance'], 16)
                        todo.append(parent)
                        if parent not in by_id:
                            # Retain candidate identities, not an invented native
                            # namespace-resolution rule. This is an inventory.
                            candidates = [value for value in by_id
                                          if value & 0xFFFFFFFF == parent & 0xFFFFFFFF]
                            key['same_low_half_prop_candidates'] = [f'{value:016X}' for value in candidates]
                            todo.extend(candidates)
            records.append(row)
        done[identity] = records
    report = dict(method='named-first/count-bounded raw source camera PROP inspection',
                  package_exclusions=exclusions,
                  camera_references=references if args.references else None,
                  resources={f'{key:016X}': value for key, value in sorted(done.items())})
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2, allow_nan=False), encoding='utf-8')
    print(f'{len(done)} identities, {sum(map(len, done.values()))} records; {len(exclusions)} index exclusions')
    for identity, records in sorted(done.items()):
        for record in records:
            label = [prop.get('text') for prop in record['properties'] if prop['identifier'] == '00B2CCCA']
            values = [(prop['identifier'], prop.get('values', prop.get('raw_le_components', prop.get('keys'))))
                      for prop in record['properties'] if any(key in prop for key in ('values', 'raw_le_components', 'keys'))]
            print(f'{identity:016X}', record['package'], label, values)


if __name__ == '__main__':
    main()
