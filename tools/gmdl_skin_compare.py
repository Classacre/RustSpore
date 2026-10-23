"""Compare independent Python GMDL tail values with Rust canonical digests.

Complete tail records come from gmdl_skin_probe's source-byte population pass.
Influences are independently re-read from source buffers for every Rust row by
default; --influence-limit is an explicit bounded diagnostic, not full coverage.
"""
import argparse
import csv
import json
from pathlib import Path
import struct

from dbpf_extract import payload
from gmdl_skin_probe import indexes, declarations


def fnv(raw):
    h = 0xCBF29CE484222325
    for b in raw:
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f'{h:016X}'


def tail_bytes(t):
    b = bytearray()
    def word(v):
        b.extend(struct.pack('<I', v))
    def transform(v):
        word(v['flags'])
        word(v['count'])
        b.extend(struct.pack('<13f', *v['translation'], v['scale'], *v['rotation']))
    word(t['source_at'])
    word(len(t['materials']))
    for m in t['materials']:
        word(int(m['version9_prefix'] is not None))
        if m['version9_prefix'] is not None:
            word(m['version9_prefix'])
        word(len(m['shader_data']))
        for ident, raw in m['shader_data']:
            raw = bytes.fromhex(raw)
            word(ident)
            word(len(raw))
            b.extend(raw)
    word(len(t['pairs']))
    for a, c in t['pairs']:
        word(a)
        word(c)
    word(len(t['parts']))
    for p in t['parts']:
        word(p['source_at'])
        transform(p['transform_a'])
        transform(p['transform_b'])
        word(p['field'])
        word(p['model_instance'] & 0xFFFFFFFF)
        word(p['model_type'])
        word(p['model_instance'] >> 32)
        word(p['present'])
        word(p['bones_at'])
        word(len(p['bone_words']))
        for row in p['bone_words']:
            b.extend(struct.pack('<40I', *row))
    for v in t['footer']:
        word(v)
    return b


def influence_bytes(data, mesh_index):
    buffers, meshes = declarations(data)
    fmt, count, stride, at = buffers[meshes[mesh_index][1]]
    joint = next(e for e in fmt if e[0] == 0 and e[4] == 2 and e[5] == 0)
    weight = next(e for e in fmt if e[0] == 0 and e[4] == 1 and e[5] == 0)
    assert joint[2] == 5 and weight[2] == 8
    out = bytearray(struct.pack('<I', count))
    for i in range(count):
        js = data[at+i*stride+joint[1]:at+i*stride+joint[1]+4]
        ws = data[at+i*stride+weight[1]:at+i*stride+weight[1]+4]
        assert all(j % 3 == 0 for j, w in zip(js, ws) if w)
        # Python computes double then rounds to f32; exactly the same correctly
        # rounded result as the Rust f32 byte/255 division for all256 byte values.
        out.extend(struct.pack('<4I4f', *(j//3 if w else 0 for j, w in zip(js, ws)),
                               *(w/255 for w in ws)))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('probe', type=Path)
    ap.add_argument('rust', type=Path)
    ap.add_argument('--out', required=True, type=Path)
    ap.add_argument('--influence-limit', type=int)
    args = ap.parse_args()
    report = json.loads(args.probe.read_text())
    by_key = {(str(Path(r['package'])).lower(), r['key']): r for r in report['records']}
    rows = list(csv.DictReader(args.rust.open(newline=''), delimiter='\t'))
    index_cache = {}
    checked = dict(tails=0, influences=0, influence_vertices=0)
    mismatches = []
    for row in rows:
        package = str(Path(row['package'])).lower()
        if row['kind'] == 'tail':
            source = by_key.get((package, row['resource_key']))
            actual = fnv(tail_bytes(source['tail'])) if source else 'source-record-missing'
            checked['tails'] += 1
        else:
            if args.influence_limit is not None and checked['influences'] >= args.influence_limit:
                continue
            path = Path(row['package'])
            if package not in index_cache:
                index_cache[package] = {f"{e['type']:08X}:{e['group']:08X}:{e['instance']:016X}": e for e in indexes(path)}
            entry = index_cache[package][row['resource_key']]
            raw = influence_bytes(payload(path, entry), int(row['kind'].split(':')[1]))
            actual = fnv(raw)
            checked['influences'] += 1
            checked['influence_vertices'] += struct.unpack_from('<I', raw)[0]
        if actual != row['fnv64']:
            mismatches.append(dict(package=row['package'], key=row['resource_key'], kind=row['kind'],
                                   python=actual, rust=row['fnv64']))
    result = dict(checked=checked, rust_records=len(rows), mismatches=mismatches,
                  population_complete=args.influence_limit is None,
                  probe_packages=report['packages'], probe_errors=report['errors'])
    args.out.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    if mismatches:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
