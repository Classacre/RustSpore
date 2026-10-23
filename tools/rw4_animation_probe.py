"""Measure RW4 animation objects from caller-supplied packages; no external code.

This Python discovery pass uses the existing wide-index helper. The Rust verifier
is the authority for full-install coverage, including other DBPF index layouts.
"""
import argparse
import json
import math
import csv
from pathlib import Path
import struct
from collections import Counter

from dbpf_extract import parse_index, payload

TYPES = {0x70001: 'keyframe', 0x70002: 'skeleton', 0x70003: 'animation_skin',
         0x7000B: 'unknown_7000b', 0x7000C: 'skins_in_k',
         0x7000F: 'skin_matrix', 0xFF0001: 'animations'}

def sections(data):
    if data[:12] != bytes.fromhex('89525734773332000d0a1a0a'):
        raise ValueError('not RW4')
    count, table = struct.unpack_from('<I', data, 36)[0], struct.unpack_from('<I', data, 48)[0]
    for i in range(count):
        at, _, size, align, _, typ = struct.unpack_from('<6I', data, table + i*24)
        if typ == 0x10030:
            at += struct.unpack_from('<I', data, 68)[0]
        yield i, typ, at, size, align

def inspect(data, base, size):
    vals = struct.unpack_from('<' + 'I' * (min(size, 128)//4), data, base)
    floats = struct.unpack_from('<' + 'f' * (min(size, 128)//4), data, base)
    return {'words': [f'{v:08x}' for v in vals],
            'floats': [round(f, 6) if math.isfinite(f) and abs(f) < 1e10 else None for f in floats]}

def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f'{value:016X}'

def digest_records(data):
    """Independent canonical typed values, rather than a hash of source slices."""
    ss = list(sections(data))
    by_index = {i: (t, at, size) for i, t, at, size, _ in ss}
    records = {}
    word = lambda at: struct.unpack_from('<I', data, at)[0]
    words = lambda at, count: list(struct.unpack_from('<'+'I'*count, data, at))
    def emit(kind, index, count, values):
        records[(kind, str(index))] = (count, fnv64(struct.pack('<'+'I'*len(values), *values)))
    for i, typ, at, size, _ in ss:
        if typ == 0x70002:
            flags, parents, ids, count, sid, sflags = words(at, 6)
            values = [sid, sflags, count]
            for j in range(count):
                values.extend([word(ids+j*4), word(flags+j*4), word(parents+j*4)])
            emit('skeleton', i, count, values)
        elif typ in (0x70003, 0x7000f):
            ptr, count = words(at, 2)
            stride = 16 if typ == 0x70003 else 12
            values = [count]
            for j in range(count):
                raw = words(ptr+j*stride*4, stride)
                values.extend(raw)
                if typ == 0x70003:
                    values.extend(raw[k] for k in (0,4,8,12,1,5,9,13,2,6,10,14))
            emit('inverse_bind' if typ == 0x70003 else 'matrix_slots', i, count, values)
        elif typ == 0x70001:
            h = words(at, 12)
            descriptors = [words(h[11]+j*12, 3) for j in range(h[1])]
            kinds = {(d[2], d[1]) for d in descriptors}
            factor = {(0x101, 32): 2, (0x601, 48): 3}.get(next(iter(kinds)), 1) if len(kinds) == 1 else 1
            allocated = h[5]-h[4]
            assert allocated % factor == 0
            end = h[4]-at+allocated//factor
            assert not any(data[at+end:h[5]])
            values = [h[2], h[8], h[10], h[5]-at-end, h[1]]
            total = 0
            for j, (off, stride, kind) in enumerate(descriptors):
                stop = descriptors[j+1][0] if j+1<h[1] else end
                assert stop >= off and (stop-off) % stride == 0
                count = (stop-off)//stride
                total += count
                values.extend([word(h[0]+j*4), kind, stride, count])
                for k in range(count):
                    v = words(at+off+k*stride, stride//4)
                    if kind == 0x100 and stride == 8:
                        values.extend([v[1], v[0]])
                    elif kind == 0x101 and stride == 32:
                        values.extend([v[7]]+v[:7]+[0x3f800000]*3)
                    elif kind == 0x601 and stride == 48:
                        values.extend([v[11]]+v[:10])
                    else:
                        raise ValueError(f'unknown track {kind:x}/{stride}')
            emit('clip', i, total, values)

    def influences(mesh_at, joint_count):
        # Read the declaration storage directly. No Rust parser or mesh helper
        # participates in this reference calculation.
        source = None
        for ref in words(mesh_at+36, word(mesh_at+16)):
            if ref >> 22 or ref not in by_index:
                continue
            _, vb, _ = by_index[ref]
            desc_ref = word(vb)
            if desc_ref >> 22 or desc_ref not in by_index:
                continue
            _, desc, _ = by_index[desc_ref]
            n = struct.unpack_from('<H', data, desc+12)[0]
            elements = [struct.unpack_from('<hhbbbbI', data, desc+24+j*12) for j in range(n)]
            if not any(el[4] == 0 for el in elements):
                continue
            indices = next((el for el in elements if el[4] == 2 and el[0] == 0), None)
            weights = next((el for el in elements if el[4] == 1 and el[0] == 0), None)
            if indices is None or weights is None:
                raise ValueError('vertex declaration has no joint indices or weights')
            count = word(vb+12)
            stride = word(vb+20) or data[desc+15]
            _, base, _ = by_index[word(vb+24)]
            source = ('vertex', count, stride, base, indices, weights)
            break
        if source is None:
            blend = next(((at, size) for _, typ, at, size, _ in ss if typ == 0x200af and word(at) == 1), None)
            if blend is None:
                raise ValueError('no blend influences')
            base, _ = blend
            h = words(base, 16)
            count, n = h[13], h[15]
            if not 1 <= n <= 4 or not h[10] or not h[11]:
                raise ValueError('unsupported blend influences')
            source = ('blend', count, (n*2+3)&~3, base+h[10], base+h[11], n)
        values = [source[1]]
        for j in range(source[1]):
            if source[0] == 'blend':
                _, _, stride, ip, wp, n = source
                raw = list(struct.unpack_from('<'+'H'*n, data, ip+j*stride)) + [0]*(4-n)
                weights = list(struct.unpack_from('<'+'f'*n, data, wp+j*n*4)) + [0.]*(4-n)
                bits = words(wp+j*n*4, n)+[0]*(4-n)
            else:
                _, _, stride, base, ie, we = source
                ip, wp = base+j*stride+ie[1], base+j*stride+we[1]
                if ie[2] == 5:
                    raw = list(data[ip:ip+4])
                elif ie[2] == 6:
                    raw = list(struct.unpack_from('<2H', data, ip))+[0,0]
                elif ie[2] in (7, 12):
                    raw = list(struct.unpack_from('<4H', data, ip))
                else:
                    raise ValueError(f'unsupported index declaration type {ie[2]}')
                if 0 <= we[2] <= 3:
                    n = we[2]+1
                    bits = words(wp, n)+[0]*(4-n)
                    weights = list(struct.unpack('<4f', struct.pack('<4I', *bits)))
                elif we[2] == 8:
                    weights = [v/255 for v in data[wp:wp+4]]
                    bits = list(struct.unpack('<4I', struct.pack('<4f', *weights)))
                else:
                    raise ValueError(f'unsupported weight declaration type {we[2]}')
            if abs(sum(weights)-1) > .002:
                raise ValueError('unnormalized weights')
            joints = [raw[k]//3 if weights[k] > 0 else 0 for k in range(4)]
            if any(weights[k] > 0 and (raw[k] % 3 or joints[k] >= joint_count) for k in range(4)):
                raise ValueError('invalid joint register index')
            values.extend(joints+bits)
        return source[1], values

    for i, typ, at, _, _ in ss:
        if typ != 0x7000c:
            continue
        _, skeleton_at, _ = by_index[word(at+12)]
        count = word(skeleton_at+12)
        for ordinal, (_, mesh_typ, mesh_at, _, _) in enumerate(s for s in ss if s[1] == 0x20009):
            try:
                vertices, values = influences(mesh_at, count)
            except ValueError:
                continue  # Missing typed Rust rows remain explicit verifier gaps.
            emit('influences', f'{i}:{ordinal}', vertices, values)
    return records

def compare_digests(path):
    expected = {}
    with path.open(newline='', encoding='utf-8') as handle:
        for row in csv.DictReader(handle, delimiter='\t'):
            key = (row['package'], row['resource_key'])
            expected.setdefault(key, {})[(row['kind'], row['section'])] = (int(row['count']), row['fnv64'])
    grouped = {}
    for (package, key), records in expected.items():
        grouped.setdefault(package, {})[key] = records
    checked, mismatches, rows = 0, [], 0
    for package, targets in grouped.items():
        package = Path(package)
        found = set()
        for entry in parse_index(package)[0]:
            key = f'{entry["type"]:08X}:{entry["group"]:08X}:{entry["instance"]:016X}'
            if key not in targets:
                continue
            found.add(key)
            actual = digest_records(payload(package, entry))
            checked += 1
            rows += len(targets[key])
            if actual != targets[key]:
                differences = [f'{k}: rust={targets[key].get(k)} python={actual.get(k)}' for k in set(actual)|set(targets[key]) if actual.get(k) != targets[key].get(k)]
                mismatches.append({'package': package.name, 'resource': key, 'differences': differences[:10]})
        if found != set(targets):
            raise ValueError(f'missing resources: {set(targets)-found}')
        print(f'digest_package={package.name} checked={len(found)}', flush=True)
    result = {'resources': checked, 'object_or_influence_rows': rows, 'mismatches': mismatches}
    print(json.dumps(result, indent=2), flush=True)
    return result

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('packages', nargs='*', type=Path)
    ap.add_argument('--compare-digests', type=Path)
    ap.add_argument('--samples', type=int, default=2)
    ap.add_argument('--out', type=Path)
    args = ap.parse_args()
    if args.compare_digests:
        result = compare_digests(args.compare_digests)
        if args.out:
            args.out.parent.mkdir(parents=True, exist_ok=True)
            args.out.write_text(json.dumps(result, indent=2), encoding='utf-8')
        raise SystemExit(bool(result['mismatches']))
    if not args.packages:
        ap.error('supply packages or --compare-digests')
    counts, samples = Counter(), []
    seen = Counter()
    for package in args.packages:
        entries, _ = parse_index(package)
        for entry in entries:
            if entry['type'] != 0x2F4E681B:
                continue
            data = payload(package, entry)
            for index, typ, at, size, alignment in sections(data):
                if typ not in TYPES:
                    continue
                counts[TYPES[typ]] += 1
                if seen[typ] >= args.samples:
                    continue
                seen[typ] += 1
                row = {'package': package.name, 'instance': f'{entry["instance"]:016x}',
                       'index': index, 'type': TYPES[typ], 'at': at, 'size': size,
                       'alignment': alignment, **inspect(data, at, size)}
                samples.append(row)
            if args.samples and all(seen[t] >= args.samples for t in TYPES):
                break
    result = {'scope': 'Sample discovery with early stopping; wide 28-byte DBPF indices only.',
              'observed_objects': dict(counts), 'samples': samples}
    text = json.dumps(result, indent=2)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(text, encoding='utf-8')
    print(text)

if __name__ == '__main__':
    main()
