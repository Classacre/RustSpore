"""Independent header-0 SMT/state association and installed-key census.

Only measured data framing is decoded. Compiled-state payloads remain opaque;
source shader text is not copied to the repository. Reports stay under target/.
"""
import argparse
from collections import Counter, defaultdict
import csv
import json
from pathlib import Path
import struct

from gmdl_skin_probe import indexes, key
from dbpf_extract import payload
from rw4_animation_probe import sections


def links(data):
    assert data[:4] == b'\0'*4
    at, offset, records = 4, 0, []
    while True:
        ident, = struct.unpack_from('>I', data, at)
        at += 4
        if ident == 0xFFFFFFFF:
            break
        count, textures = data[at:at+2]
        at += 2
        refs = [struct.unpack_from('>2I', data, at+i*8) for i in range(textures)]
        at += textures*8
        records.append(dict(material_id=ident, state_count=count, textures=refs,
                            state_start=offset, state_end=offset+count))
        offset += count
    assert at == len(data)
    return records


def states(data):
    result = []
    for i, typ, at, size, align in sections(data):
        assert typ == 0x2000B and at % 16 == 0 and size >= 32
        values = struct.unpack_from('<7I', data, at)
        assert values[0] == size
        result.append(dict(index=i, primitive_type=values[1], soft_dirty=values[2],
                           soft_delta=values[3], hard_dirty=values[4], hard_delta=values[5],
                           shader_id=values[6], padding=data[at+28:at+32],
                           payload=data[at+32:at+size]))
    return result


def canonical(records, compiled):
    b = bytearray()
    def word(v):
        b.extend(struct.pack('<I', v))
    word(len(records))
    for r in records:
        for v in (r['material_id'], r['state_count'], len(r['textures'])):
            word(v)
        for instance, group in r['textures']:
            word(instance)
            word(group)
        word(r['state_start'])
        word(r['state_end'])
    word(len(compiled))
    for s in compiled:
        for name in ('index', 'primitive_type', 'soft_dirty', 'soft_delta', 'hard_dirty', 'hard_delta', 'shader_id'):
            word(s[name])
        word(len(s['payload']))
        b.extend(s['padding'])
        b.extend(s['payload'])
    h = 0xCBF29CE484222325
    for v in b:
        h = ((h ^ v) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f'{h:016X}'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('--out', required=True, type=Path)
    ap.add_argument('--rust', type=Path)
    args = ap.parse_args()
    packages, errors, records = [], [], []
    wanted = set()
    for pkg in sorted(args.install.rglob('*.package')):
        es = indexes(pkg)
        packages.append((pkg, es))
        for e in es:
            if e['type'] != 0x0469A3F7 or e['instance'] >> 32 != 0x40212000:
                continue
            rs = links(payload(pkg, e))
            instance = 0x4021200100000000 | (e['instance'] & 0xFFFFFFFF)
            matches = [v for v in es if v['type'] == 0x2F4E681B and v['instance'] == instance]
            assert len(matches) == 1
            cs = states(payload(pkg, matches[0]))
            assert sum(r['state_count'] for r in rs) == len(cs)
            digest = canonical(rs, cs)
            for r in rs:
                r['shader_ids'] = [f"{s['shader_id']:08X}" for s in cs[r['state_start']:r['state_end']]]
                for lo, hi in r['textures']:
                    wanted.add((hi << 32)|lo)
            # Preserve offsets and field values, not opaque saved pointers/source
            # fragments accidentally resident in compiled-state padding.
            cs = [{k:v for k,v in s.items() if k not in ('padding', 'payload')} for s in cs]
            records.append(dict(package=str(pkg), key=key(e), fnv64=digest,
                                materials=rs, compiled_states=cs))
    found, low_candidates = defaultdict(list), defaultdict(list)
    wanted_lows = {v & 0xFFFFFFFF for v in wanted}
    for pkg, es in packages:
        for e in es:
            if e['instance'] in wanted:
                found[e['instance']].append(dict(package=str(pkg), key=key(e)))
            if (e['instance'] & 0xFFFFFFFF) in wanted_lows:
                low_candidates[e['instance'] & 0xFFFFFFFF].append(dict(package=str(pkg), key=key(e)))
    references = [dict(instance=f'{v:016X}', exact_resources=found[v],
                       same_low_half_only=low_candidates[v & 0xFFFFFFFF]) for v in sorted(wanted)]
    mismatches = []
    if args.rust:
        rust = {(str(Path(r['package'])).lower(), r['resource_key']):r
                for r in csv.DictReader(args.rust.open(newline=''), delimiter='\t')}
        for r in records:
            peer = rust.get((str(Path(r['package'])).lower(), r['key']))
            if peer is None or peer['fnv64'] != r['fnv64']:
                mismatches.append(dict(package=r['package'], key=r['key'], python=r['fnv64'], rust=peer))
        assert len(records) == len(rust)
    summary = dict(packages=len(packages), libraries=len(records), materials=sum(len(r['materials']) for r in records),
                   states=sum(len(r['compiled_states']) for r in records),
                   texture_references=sum(len(m['textures']) for r in records for m in r['materials']),
                   unique_texture_keys=len(wanted), exact_key_hits=sum(bool(found[v]) for v in wanted),
                   same_low_half_hits=sum(bool(low_candidates[v & 0xFFFFFFFF]) for v in wanted),
                   digest_mismatches=len(mismatches))
    args.out.write_text(json.dumps(dict(summary=summary, records=records, references=references, mismatches=mismatches), indent=2))
    print(json.dumps(summary, indent=2))
    if mismatches:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
