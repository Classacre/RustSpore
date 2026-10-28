"""Independently compare every conditional visitation and local rectangle row.

Both source reports must represent the same explicit candidate package. The
fixture assumes fresh child lists, successful attachment, zero private flags,
no runtime mutation and the authored root rectangle. It is not a retail frame.
"""
import argparse
import json
import math
import struct
from pathlib import Path


def f32(v):
    return struct.unpack('<f', struct.pack('<f', v))[0]


def floor32(v):
    return v if v == 0.0 else f32(math.floor(v))


def bits(rect):
    if rect is None:
        return 'unknown'
    return ','.join(f'{struct.unpack("<I", struct.pack("<f", x))[0]:08X}' for x in rect)


def simple_layout(parent, authored, procedures):
    r = list(authored)
    for flag in procedures:
        if flag & 8:
            span = f32(parent[2] - parent[0])
            r[2] = f32(span + r[2])
            if not flag & 4:
                r[0] = f32(span + r[0])
        if flag & 2:
            span = f32(parent[3] - parent[1])
            r[3] = f32(span + r[3])
            if not flag & 1:
                r[1] = f32(span + r[1])
        if r[0] > r[2]:
            r[2] = r[0]
        if r[1] > r[3]:
            r[3] = r[1]
    if not all(math.isfinite(v) for v in r):
        raise ValueError('nonfinite layout')
    return tuple(floor32(v) for v in r)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('candidate_report', type=Path)
    p.add_argument('element_report', type=Path)
    p.add_argument('rust_tsv', type=Path)
    p.add_argument('--package', required=True)
    args = p.parse_args()
    report = json.loads(args.candidate_report.read_text(encoding='utf-8'))
    records = [r for r in report['records'] if r['package'] == args.package]
    if len(records) != 1:
        raise ValueError('package must identify exactly one audit record')
    graph = {e['index']: e for e in records[0]['graph']}
    elements = json.loads(args.element_report.read_text(encoding='utf-8'))
    objects = {e['index']: e for e in elements['records']}
    if set(graph) != set(objects):
        raise ValueError('source graph and element indices differ')

    def property_value(n, prop_id, typ):
        ps = [p for p in objects[n]['properties'] if p['id'] == prop_id]
        if len(ps) != 1 or ps[0]['type'] != typ or ps[0]['count'] != 1:
            raise ValueError(f'missing or invalid {prop_id} on {n}')
        return bytes.fromhex(ps[0]['value'])

    def flags(n):
        return struct.unpack('<I', property_value(n, 'EEC1B000', '0009'))[0]

    def rectangle(n):
        rect = struct.unpack('<4f', property_value(n, 'EEC1B005', '0010'))
        if not all(math.isfinite(v) for v in rect):
            raise ValueError('nonfinite rectangle')
        return rect

    def refs(n, prop_id):
        ps = [p for p in graph[n]['references'] if p['property'] == prop_id]
        if len(ps) > 1:
            raise ValueError('duplicate reference property')
        return ps[0]['indices'] if ps else []

    roots = [n for n, obj in graph.items() if obj['root']]
    if len(roots) != 1 or refs(roots[0], 'EEC1B00C'):
        raise ValueError('unsupported root')
    todo = [(roots[0], None, 0, rectangle(roots[0]))]
    seen = set()
    expected = []
    while todo:
        n, parent, depth, local = todo.pop()
        if n in seen:
            raise ValueError('duplicate or cyclic reachable reference')
        seen.add(n)
        if not flags(n) & 1:
            continue
        descendants = refs(n, 'EEC1B00B')
        if any(flags(c) & 0x40 for c in descendants):
            raise ValueError('unsupported insertion partition')
        expected.append((n, parent, depth, graph[n]['class'], flags(n), bits(local)))
        for c in descendants:
            procs = refs(c, 'EEC1B00C')
            if len(procs) != len(set(procs)):
                raise ValueError('duplicate procedure reference')
            proc_flags = []
            supported = True
            for proc in procs:
                if graph[proc]['class'] != '2F418D72':
                    supported = False
                    break
                raw = property_value(proc, '0F3D0000', '0007')
                if len(raw) != 1:
                    raise ValueError('invalid procedure flags')
                proc_flags.append(raw[0])
            child_local = simple_layout(local, rectangle(c), proc_flags) if local is not None and supported else None
            todo.append((c, n, depth + 1, child_local))

    lines = [s for s in args.rust_tsv.read_text(encoding='utf-8').splitlines() if not s.startswith('#')]
    header = lines.pop(0).split('\t')
    if header != ['ordinal','object','parent','depth','class','flags','local_rect_f32_bits']:
        raise ValueError('unexpected header')
    actual = []
    for ordinal, line in enumerate(lines):
        num, obj, parent, depth, cls, flag, rect = line.split('\t')
        if int(num) != ordinal:
            raise ValueError('unexpected ordinal')
        actual.append((int(obj,16), None if parent == '-' else int(parent,16), int(depth), cls, int(flag,16), rect))
    if expected != actual:
        for i, (want, got) in enumerate(zip(expected, actual)):
            if want != got:
                raise ValueError(f'row {i} differs: expected {want}, got {got}')
        raise ValueError(f'row counts differ: {len(expected)} vs {len(actual)}')
    print(f'{len(actual)} rows agree on conditional visitation, flags and every local rectangle or unknown')
    print('No screen position, runtime frame, or active package established.')

if __name__ == '__main__':
    main()
