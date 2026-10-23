"""Compare source creature assembly conventions against shipped merged geometry.

No image normalization or alignment is applied. Only metadata/distances are
written. Unsupported geometry is counted explicitly; install bytes are read
through indexes. The existing Python helper's compact-index limitation remains.
"""
import argparse
from collections import Counter, defaultdict
from functools import lru_cache
import json
from pathlib import Path
import struct

import numpy as np
from scipy.spatial import cKDTree

from dbpf_extract import parse_index, payload
from anim_rig_mapping_probe import inspect_creature, properties, key_values, MODEL_SLOTS, resource_id
from rw4_animation_probe import sections
from gmdl_full import DECL_SIZE

PROP, CREATURE, RW4, GMDL = 0x00B1B104, 0x0F43029A, 0x2F4E681B, 0x00E6BCE5


def gmdl_positions(data):
    at = 8+struct.unpack_from('>I', data, 4)[0]*12
    meshes = struct.unpack_from('<I', data, at)[0]
    at += 32
    count = struct.unpack_from('<I', data, at)[0]
    at += 4
    for _ in range(count):
        _, n, bits, _ = struct.unpack_from('<4I', data, at)
        at += 16+n*(2 if bits == 16 else 4)
    formats = []
    count = struct.unpack_from('<I', data, at)[0]
    at += 4
    for _ in range(count):
        n = struct.unpack_from('<I', data, at)[0]
        at += 4
        elems = [struct.unpack_from('<hhbbbbi', data, at+i*12) for i in range(n)]
        at += n*12
        formats.append(elems)
    buffers = []
    count = struct.unpack_from('<I', data, at)[0]
    at += 4
    for _ in range(count):
        fmt, n, _ = struct.unpack_from('<3I', data, at)
        at += 12
        stride = sum(DECL_SIZE[e[2]] for e in formats[fmt])
        pos = next(e for e in formats[fmt] if e[0] == 0 and e[4] == 0)
        if pos[2] != 2:
            raise ValueError(f'unsupported gmdl position type {pos[2]}')
        if at+n*stride > len(data):
            raise ValueError('gmdl vertex bounds')
        buffers.append(np.ndarray((n, 3), dtype='<f4', buffer=data, offset=at+pos[1], strides=(stride, 4)).astype(np.float64))
        at += n*stride
    links = [struct.unpack_from('<2I', data, at+i*8) for i in range(meshes)]
    return np.concatenate([buffers[v] for _, v in links])


def rw4_positions(data):
    ss = {i: (typ, at, size) for i, typ, at, size, _ in sections(data)}
    blend = next(((at, size) for typ, at, size in ss.values() if typ == 0x200AF), None)
    result = []
    for typ, at, _ in ss.values():
        if typ != 0x20009:
            continue
        count = struct.unpack_from('<I', data, at+16)[0]
        chosen = None
        for index in struct.unpack_from('<'+'I'*count, data, at+36):
            # RW4's no-object/subreference indices are not section ordinals.
            # Match the existing typed reader: they cannot select a VB here.
            if index not in ss:
                continue
            _, vb, _ = ss[index]
            desc_index = struct.unpack_from('<I', data, vb)[0]
            if desc_index not in ss:
                continue
            _, desc, _ = ss[desc_index]
            n = struct.unpack_from('<H', data, desc+12)[0]
            elems = [struct.unpack_from('<hhbbbbi', data, desc+24+i*12) for i in range(n)]
            pos = next((e for e in elems if e[0] == 0 and e[4] == 0), None)
            if pos is not None:
                chosen = vb, desc, pos
                break
        if chosen:
            vb, desc, pos = chosen
            n = struct.unpack_from('<I', data, vb+12)[0]
            stride, index = struct.unpack_from('<2I', data, vb+20)
            stride = stride or data[desc+15]
            _, base, size = ss[index]
            if pos[2] != 2 or stride == 0 or n*stride > size:
                raise ValueError('unsupported rw4 position layout')
            result.append(np.ndarray((n, 3), dtype='<f4', buffer=data, offset=base+pos[1], strides=(stride, 4)).astype(np.float64))
        elif blend:
            base, size = blend
            offset = struct.unpack_from('<I', data, base+4)[0]
            n = struct.unpack_from('<I', data, base+52)[0]
            if not offset or offset+n*16 > size:
                raise ValueError('blend position bounds')
            result.append(np.ndarray((n, 3), dtype='<f4', buffer=data, offset=base+offset, strides=(16, 4)).astype(np.float64))
        else:
            raise ValueError('rw4 has no supported position source')
    return np.concatenate(result)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('root', nargs='?', type=Path, default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    resources, creatures, skipped = defaultdict(list), [], []
    for path in sorted(args.root.rglob('*.package')):
        try:
            entries, _ = parse_index(path)
        except ValueError as error:
            skipped.append({'package': str(path), 'error': str(error)})
            continue
        for entry in entries:
            if entry['type'] in (PROP, RW4, GMDL):
                resources[entry['type'], entry['instance']].append((path, entry))
            elif entry['type'] == CREATURE and path.name == 'Spore_Content.package':
                creatures.append((path, entry))
    def find(kind, ident):
        candidates = resources.get((kind, ident), [])
        local = [c for c in candidates if c[0].name == 'Spore_Content.package']
        return (local or candidates)[0] if candidates else None
    @lru_cache(maxsize=2048)
    def prop_mesh(ident):
        source = find(PROP, ident)
        if not source:
            raise ValueError('no prop')
        records = properties(payload(*source))
        for slot in MODEL_SLOTS:
            models = [key for i, t, s, v in records if i == slot for key in key_values(t, s, v)]
            if not models:
                continue
            model, kind = models[0]
            source = find(kind, model)
            if not source:
                raise ValueError('first model absent')
            data = payload(*source)
            positions = {GMDL: gmdl_positions, RW4: rw4_positions}.get(kind)
            if positions is None:
                raise ValueError(f'unsupported model type {kind:08X}')
            vertices = positions(data)
            if not len(vertices) or not np.isfinite(vertices).all():
                raise ValueError('empty/nonfinite vertices')
            return vertices
        raise ValueError('no direct model')
    results, excluded = [], []
    for path, entry in creatures:
        try:
            merged = prop_mesh(entry['instance'])
            rows, *_ = inspect_creature(payload(path, entry))
            variants = {key: [] for key in ('stored', 'stored_scale', 'transpose', 'transpose_scale')}
            for row in rows:
                ident = (struct.unpack_from('<I', row, 132)[0] << 32) | struct.unpack_from('<I', row, 136)[0]
                vertices = prop_mesh(ident)
                rotation = np.asarray(struct.unpack_from('<9f', row, 48)).reshape((3, 3))
                translation = np.asarray(struct.unpack_from('<3f', row, 84))
                scale = struct.unpack_from('<f', row, 44)[0]
                for key in variants:
                    linear = rotation if key.startswith('transpose') else rotation.T
                    factor = scale if key.endswith('_scale') else 1.0
                    variants[key].append((vertices*factor) @ linear+translation)
            diagonal = np.linalg.norm(merged.max(axis=0)-merged.min(axis=0))
            tree = cKDTree(merged)
            metrics = {}
            for key, parts in variants.items():
                assembled = np.concatenate(parts)
                forward = tree.query(assembled)[0]
                reverse = cKDTree(assembled).query(merged)[0]
                metrics[key] = {'forward_mean': float(forward.mean()/diagonal), 'reverse_mean': float(reverse.mean()/diagonal),
                                'forward_max': float(forward.max()/diagonal), 'reverse_max': float(reverse.max()/diagonal)}
            results.append({'key': resource_id(entry), 'parts': len(rows), 'merged_vertices': len(merged), 'metrics': metrics})
        except (ValueError, StopIteration, KeyError, struct.error) as error:
            excluded.append({'key': resource_id(entry), 'reason': str(error)})
    summary = {variant: {direction: dict(zip(('min', 'p10', 'median', 'p90', 'max'),
                map(float, np.quantile([r['metrics'][variant][direction] for r in results], [0, .1, .5, .9, 1]))))
                for direction in ('forward_mean', 'reverse_mean')} for variant in ('stored', 'stored_scale', 'transpose', 'transpose_scale')} if results else {}
    report = {'scope': {'creatures_attempted': len(creatures), 'compared': len(results), 'excluded': len(excluded),
                        'compact_indexes_excluded': len(skipped)}, 'summary': summary, 'results': results, 'excluded': excluded}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2))
    print(json.dumps({'scope': report['scope'], 'summary': summary, 'exclusion_reasons': Counter(r['reason'] for r in excluded)}, indent=2))
    if not results:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
