"""Compare Rust ordered-local output against retained independent candidate audit.

The inputs must describe the SAME explicitly selected source layout. This is a
conditional fresh-list fixture, not proof of retail draw order or screen output.
"""
import argparse
import json
import struct
from pathlib import Path


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
        raise ValueError('independent source graph and element indices differ')
    def flags(n):
        props = [p for p in objects[n]['properties'] if p['id'] == 'EEC1B000']
        if len(props) != 1 or props[0]['type'] != '0009' or props[0]['count'] != 1:
            raise ValueError('missing explicit flags')
        return struct.unpack('<I', bytes.fromhex(props[0]['value']))[0]
    def children(n):
        props = [p for p in graph[n]['references'] if p['property'] == 'EEC1B00B']
        if len(props) > 1:
            raise ValueError('duplicate reference property')
        return props[0]['indices'] if props else []
    roots = [n for n, obj in graph.items() if obj['root']]
    if len(roots) != 1:
        raise ValueError('not exactly one root')
    todo = [(roots[0], None, 0)]
    seen = set()
    expected = []
    while todo:
        n, parent, depth = todo.pop()
        if n in seen:
            raise ValueError('duplicate or cyclic reachable reference')
        seen.add(n)
        if not flags(n) & 1:
            continue
        descendants = children(n)
        if any(flags(c) & 0x40 for c in descendants):
            raise ValueError('unsupported insertion partition')
        expected.append((n, parent, depth, graph[n]['class'], flags(n)))
        todo.extend((c, n, depth + 1) for c in descendants)
    lines = [s for s in args.rust_tsv.read_text(encoding='utf-8').splitlines() if not s.startswith('#')]
    header = lines.pop(0).split('\t')
    assert header == ['ordinal','object','parent','depth','class','flags','local_rect_f32_bits']
    actual = []
    by_obj = {}
    for ordinal, line in enumerate(lines):
        num, obj, parent, depth, cls, flag, rect = line.split('\t')
        assert int(num) == ordinal
        n = int(obj,16)
        actual.append((n, None if parent == '-' else int(parent,16), int(depth), cls, int(flag,16)))
        by_obj[n] = rect
    if expected != actual:
        raise ValueError('conditional source order or flags differ')
    # This fixture is an independently hand-derived SimpleLayout result for
    # the named PatchData copy only; other local rows are not certified here.
    if records[0]['sha256'] == '5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda':
        bits = ','.join(f'{struct.unpack("<I", struct.pack("<f", x))[0]:08X}' for x in (-1.,553.,88.,601.))
        if by_obj.get(456) != bits:
            raise ValueError('PatchData Window 456 local fixture differs')
    print(f'{len(actual)} rows agree on conditional source ordering and flags; PatchData 456 local fixture checked when applicable')
    print('No screen position, runtime frame, or active package established.')

if __name__ == '__main__':
    main()
