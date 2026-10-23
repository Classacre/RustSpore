"""Measured CompiledState variable-region framing, independent of Rust.

This does not reconstruct native runtime code or interpret saved pointer values.
It parses source-observed extent tags and sentinel lists; unknown flags fail.
"""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
import struct
from material_link_probe import states, links


def bindings(s):
    data = s['payload']
    at = 0
    def take(n):
        nonlocal at
        if n < 0 or at+n > len(data):
            raise ValueError(f'truncated at {at}: {n} > {len(data)}')
        b = data[at:at+n]
        at += n
        return b
    def word():
        return struct.unpack('<I', take(4))[0]
    def pairs():
        result = []
        while True:
            ident = word()
            if ident == 0xFFFFFFFF:
                return result
            result.append((ident, word()))
    soft = s['soft_dirty']
    # Source library coverage currently has only these nonempty regions. Bit4
    # exists on every object but does not add bytes in the observed getter.
    if soft & ~0x108034:
        raise ValueError(f'unknown soft extent flags {soft:08X}')
    regions = []
    if soft & 0x100000:
        if len(data) < at+14:
            raise ValueError('truncated description header')
        n, = struct.unpack_from('<H', data, at+12)
        start = at
        take(12*(n+2))
        regions.append(dict(flag=0x100000, start=start, end=at))
    for flag, size in ((0x10,16),(0x20,12),(0x8000,17)):
        if soft & flag:
            start = at
            take(size)
            regions.append(dict(flag=flag,start=start,end=at))
    if s['hard_delta']:
        raise ValueError('unknown hard delta extent flags')
    groups = []
    if s['hard_dirty'] & 0x20000:
        while True:
            ident = word()
            if ident == 0xFFFFFFFF:
                break
            groups.append(dict(id=ident, pairs=pairs()))
    lead = word()
    textures = []
    while True:
        stage = word()
        if stage == 0xFFFFFFFF:
            break
        raster = word()
        a_flags = word()
        a = pairs() if a_flags else []
        b_flags = word()
        b = pairs() if b_flags else []
        textures.append(dict(stage=stage, raster_reference=raster,
                             a_flags=a_flags, a=a, b_flags=b_flags, b=b))
    if at != len(data):
        raise ValueError(f'trailing bytes {at}/{len(data)}')
    return dict(regions=regions, groups=groups, texture_leading_word=lead,
                textures=textures, consumed=at)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('directory', type=Path)
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--rust', type=Path)
    args = ap.parse_args()
    records, errors, libraries = [], [], []
    for p in sorted(args.directory.glob('*-2F4E681B-40212001*.bin')):
        ss = states(p.read_bytes())
        parsed = []
        for s in ss:
            try:
                b = bindings(s)
                parsed.append(b)
                records.append(dict(file=p.name,index=s['index'],shader_id=s['shader_id'],**b))
            except Exception as ex:
                errors.append(dict(file=p.name,index=s['index'],error=str(ex)))
        if len(parsed) != len(ss): continue
        ls = links(p.with_name(p.name.replace('-2F4E681B-40212001','-0469A3F7-40212000')).read_bytes())
        canonical = bytearray()
        def word(v): canonical.extend(struct.pack('<I',v))
        def pairs(ps):
            word(len(ps))
            for a,b in ps: word(a); word(b)
        word(len(ss))
        for s,b in zip(ss,parsed):
            word(s['index']); word(len(b['regions']))
            for r in b['regions']:
                word(r['flag']); word(r['end']-r['start'])
                canonical.extend(s['payload'][r['start']:r['end']])
            word(bool(s['hard_dirty'] & 0x20000)); word(len(b['groups']))
            for g in b['groups']: word(g['id']); pairs(g['pairs'])
            word(b['texture_leading_word']); word(len(b['textures']))
            for t in b['textures']:
                word(t['stage']); word(t['raster_reference'])
                word(t['a_flags']); pairs(t['a'])
                word(t['b_flags']); pairs(t['b'])
        word(len(ls))
        for l in ls:
            slots = [(i,t['stage']) for i in range(l['state_start'],l['state_end'])
                     for t in parsed[i]['textures'] if t['raster_reference']==0x400000]
            assert len(slots)==len(l['textures'])
            word(len(slots))
            for i,stage in slots: word(i); word(stage)
        h = 0xCBF29CE484222325
        for v in canonical: h=((h^v)*0x100000001B3)&0xFFFFFFFFFFFFFFFF
        libraries.append(dict(file=p.name,bindings_fnv64=f'{h:016X}'))
    c = Counter(f'{r["texture_leading_word"]:08X}' for r in records)
    mismatches = []
    if args.rust:
        peers = {}
        for r in csv.DictReader(args.rust.open(newline=''),delimiter='\t'):
            stem=Path(r['package']).stem
            inst=int(r['resource_key'].split(':')[2],16)
            peers[f'{stem}-2F4E681B-{inst+0x100000000:016X}.bin']=r['bindings_fnv64']
        assert len(peers)==len(libraries)
        for r in libraries:
            if peers.get(r['file'])!=r['bindings_fnv64']:
                mismatches.append(dict(**r,rust=peers.get(r['file'])))
    summary = dict(states=len(records),errors=len(errors),libraries=len(libraries),
                   textures=sum(len(r['textures']) for r in records),
                   groups=sum(len(r['groups']) for r in records),leading=dict(c),
                   digest_mismatches=len(mismatches))
    args.out.write_text(json.dumps(dict(summary=summary,errors=errors,records=records,
                                      libraries=libraries,mismatches=mismatches),indent=2))
    print(json.dumps(summary,indent=2))
    print(json.dumps(errors[:8],indent=2))
    if errors or mismatches: raise SystemExit(1)


if __name__ == '__main__': main()
