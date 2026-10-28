"""Audit candidate Cell skybox effect exports in installed EFFDIR v4 payloads.

Structural interpretation follows the independent SporeModder FX format reader:
https://github.com/Spore-Community/SporeModder-FX/blob/master/src/sporemodder/file/effects/EffectDirectory.java
https://github.com/Spore-Community/SporeModder-FX/blob/master/src/sporemodder/file/effects/VisualEffect.java
https://github.com/Spore-Community/SporeModder-FX/blob/master/src/sporemodder/file/effects/VisualEffectBlock.java
https://github.com/Spore-Community/SporeModder-FX/blob/master/src/sporemodder/file/effects/ImportEffect.java
https://github.com/Spore-Community/SporeModder-FX/blob/master/src/sporemodder/file/effects/ScreenEffect.java
https://github.com/Spore-Community/SporeModder-FX/blob/master/src/sporemodder/file/effects/ResourceID.java

This read-only audit does not establish which package is loaded, whether the
runtime lookup succeeds, or how to render any component. It emits metadata,
not game payload bytes. Input/output locations are explicit.
"""
import argparse
import hashlib
import json
from pathlib import Path

from dbpf_extract import payload
from gmdl_skin_probe import indexes

TYPE = 0xEA5118B0
TARGET = 0x05C7A4D0
PACKAGES = ('PatchData.package', 'Spore_Game.package', 'Spore_Graphics.package', 'Spore_Content.package')


class Reader:
    def __init__(self, data):
        self.data = data
        self.pos = 0

    def take(self, n):
        if n < 0 or n > len(self.data) - self.pos:
            raise ValueError(f'truncated payload at {self.pos}, requested {n}')
        start = self.pos
        self.pos += n
        return self.data[start:self.pos]

    def uint(self, n):
        return int.from_bytes(self.take(n), 'big')

    def count(self, limit):
        n = self.uint(4)
        if n > limit:
            raise ValueError(f'count {n} exceeds limit {limit} at {self.pos - 4}')
        return n


def screens(data, section):
    if section['version'] != 1:
        raise ValueError('unsupported screen component version')
    r = Reader(data)
    r.pos = section['offset'] + 4
    result = []
    for i in range(section['count']):
        start = r.pos
        mode, flags = r.uint(1), r.uint(4)
        for width in (12, 4, 4):
            r.take(r.count(100000) * width)
        r.take(16)
        group, instance = r.uint(4), r.uint(4)
        r.take(2)
        for _ in range(r.count(100000)):
            r.take(10)
            r.take(r.count(100000))
        for width in (8, 4, 12, 8, 8):
            r.take(r.count(100000) * width)
        result.append(dict(index=i, offset=start, size=r.pos-start, mode=mode,
                           flags=f'{flags:08X}', texture_group=f'{group:08X}',
                           texture_instance=f'{instance:08X}'))
    if r.pos != section['offset'] + section['size']:
        raise ValueError('screen component section length mismatch')
    return result


def audit(data):
    r = Reader(data)
    if r.uint(2) != 4:
        raise ValueError('unsupported directory version')
    sections = []
    counts = {}
    for kind in ('component', 'resource'):
        seen = set()
        while True:
            typ = r.uint(2)
            if typ == 0xFFFF:
                break
            if typ in seen:
                raise ValueError(f'duplicate {kind} type {typ}')
            seen.add(typ)
            version, size = r.uint(2), r.uint(4)
            if size < 4:
                raise ValueError('section too short for count')
            start = r.pos
            count = r.uint(4)
            r.take(size - 4)
            sections.append(dict(kind=kind, type=typ, version=version, count=count,
                                 offset=start, size=size))
            counts[kind, typ] = count
    version, total = r.uint(2), r.count(100000)
    if version != 1:
        raise ValueError(f'unsupported visual version {version}')
    visuals = []
    for i in range(total):
        start = r.pos
        flags = r.uint(4)
        r.take(21)
        lod = r.count(32)
        r.take(4 * lod + 12 + 4)
        blocks = r.count(10000)
        refs = []
        for _ in range(blocks):
            block_type = r.uint(1)
            r.take(4 + 2 + 4 + 36 + 12 + 2)
            lod_scales = r.count(10000)
            r.take(lod_scales * 12 + 24 + 8 + 4 + 4)
            index = r.uint(4)
            refs.append(dict(type=block_type, index=index))
        visuals.append(dict(index=i, offset=start, size=r.pos-start, flags=f'{flags:08X}', blocks=refs))
    export_start = r.pos
    matches = []
    export_count = 0
    while True:
        index = r.uint(4)
        if index == 0xFFFFFFFF:
            break
        group, name = r.uint(4), r.uint(4)
        if name == TARGET:
            matches.append(dict(offset=r.pos-12, index=index, group=f'{group:08X}', name=f'{name:08X}'))
        if (index & 0x7F000000) != 0x7F000000 and index >= total:
            raise ValueError(f'export index {index} out of range')
        export_count += 1
    assignments = r.uint(4)
    imports = r.count(100000)
    r.take(imports * 8)
    if r.pos != len(data):
        raise ValueError(f'unparsed trailing bytes: {len(data)-r.pos}')
    for match in matches:
        index = match['index']
        if (index & 0x7F000000) == 0x7F000000:
            match['kind'] = 'import reference'
        else:
            match['kind'] = 'visual effect'
            match['visual'] = visuals[index]
            for ref in match['visual']['blocks']:
                idx, typ = ref['index'], ref['type']
                if idx == 0xFFFFFFFF:
                    ref['validity'] = 'null'
                elif (idx & 0x7F000000) == 0x7F000000:
                    ref['validity'] = 'import' if (idx & 0x80FFFFFF) < imports else 'invalid import'
                else:
                    bound = total if typ == 0 else counts.get(('component', typ), 0)
                    ref['validity'] = 'in range' if idx < bound else 'out of range'
    screen_section = next((x for x in sections if x['kind'] == 'component' and x['type'] == 9), None)
    if matches and screen_section:
        screen_records = screens(data, screen_section)
        for match in matches:
            for ref in match.get('visual', {}).get('blocks', []):
                if ref['type'] == 9 and ref['validity'] == 'in range':
                    ref['screen'] = screen_records[ref['index']]
    return dict(sections=sections, visual_count=total, export_offset=export_start,
                export_count=export_count, string_assignments_word=f'{assignments:08X}',
                import_count=imports, matches=matches)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('install', type=Path)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    records = []
    for name in PACKAGES:
        path = args.install / 'Data' / name
        for entry in indexes(path):
            if entry['type'] != TYPE:
                continue
            data = payload(path, entry)
            if len(data) != entry['dec']:
                raise ValueError(f'{name}: decoded size mismatch')
            result = audit(data)
            if result['matches']:
                records.append(dict(package=name, key=f"{entry['group']:08X}:{TYPE:08X}:{entry['instance']:016X}",
                                    decoded_bytes=len(data), payload_sha256=hashlib.sha256(data).hexdigest(), **result))
    report = dict(schema='cell-skybox-effdir-audit-v1', target=f'{TARGET:08X}', records=records,
                  status='Format-based candidate exports only; runtime lookup and package precedence unverified.')
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open('x', encoding='utf-8') as f:
        json.dump(report, f, indent=2)
        f.write('\n')
    print(f'{len(records)} directories contain the target export')


if __name__ == '__main__':
    main()

