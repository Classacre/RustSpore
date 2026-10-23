"""Measure shipped GMDL skeleton tails and declaration-stored skin influences.

Uses the repository's independently measured core-geometry boundary only. All
tail fields below are read directly from package bytes. No retail source or
third-party implementation is copied. Diagnostics belong under ignored target/.
"""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import struct

from dbpf_extract import payload
from gmdl_full import parse as geometry_boundary, Reader, DECL_SIZE
from anim_rig_mapping_probe import inspect_creature, properties, key_values
from refpack_bulk import install as bulk_backreferences

bulk_backreferences()

GMDL = 0x00E6BCE5
CREATURE = 0x0F43029A
PROP = 0x00B1B104
MODEL_SLOTS = {0x00F9EFBB, 0x00F9EFBC, 0x00F9EFBD, 0x00F9EFBE}


def indexes(path):
    """The three measured DBPF index shapes; non-fitting layouts fail visibly."""
    with path.open('rb') as f:
        h = f.read(0x60)
        assert h[:4] == b'DBPF'
        count, size, offset = (struct.unpack_from('<I', h, at)[0] for at in (0x24, 0x2C, 0x40))
        f.seek(offset)
        blob = f.read(size)
    shapes = [(4, 28, 4), (12, 24, 0), (16, 20, 0)]
    candidates = []
    for prefix, width, suffix in shapes:
        if prefix + count * width + suffix != size:
            continue
        out = []
        for i in range(count):
            v = struct.unpack_from('<'+'I'*(width//4), blob, prefix+i*width)
            if width == 28:
                g, t, hi, lo, off, stored, dec = v
                instance = (hi << 32) | lo
            elif width == 24:
                t, instance, off, stored, dec, g = v
            else:
                instance, off, stored, dec, g = v
                t = 0
            out.append(dict(type=t, group=g, instance=instance, offset=off,
                            stored=stored & 0x7FFFFFFF, dec=dec))
        spans = sorted((e['offset'], e['stored']) for e in out
                       if e['stored'] and e['offset'] >= 0x60 and e['offset']+e['stored'] <= path.stat().st_size)
        score = sum(a[0]+a[1] == b[0] for a, b in zip(spans, spans[1:])) / max(1, len(spans)-1)
        candidates.append((score, out))
    if not candidates:
        raise ValueError('unsupported index shape')
    return max(candidates, key=lambda c: c[0])[1]


def key(entry):
    return f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"


def transform(data, at):
    flags, count = struct.unpack_from('<2H', data, at)
    offset = struct.unpack_from('<3f', data, at+4)
    scale, = struct.unpack_from('<f', data, at+16)
    rotation = struct.unpack_from('<9f', data, at+20)
    return dict(flags=flags, count=count, translation=offset, scale=scale, rotation=rotation)


def tail(data, shader_sizes=None):
    kind, at, end, error = geometry_boundary(data)
    version = struct.unpack_from('<I', data)[0]
    assert version in (8, 9)
    if kind == 'core':
        return None
    if (error or end != len(data)) and not (version == 9 and kind == 'fail_trailing'):
        raise ValueError(f'non-exact GMDL: {kind}/{error}')
    start = at
    zero, materials = struct.unpack_from('<2I', data, at)
    assert zero == 0
    at += 8
    # Retain raw shader payloads without assigning diffuse/normal meanings.
    material_data = []
    for _ in range(materials):
        prefix = None
        if version == 9:
            prefix, = struct.unpack_from('>I', data, at)
            at += 4
            if prefix == 0:
                material_data.append(dict(version9_prefix=prefix, shader_data=[]))
                continue
        count, = struct.unpack_from('<I', data, at)
        at += 4
        records = []
        for _ in range(count):
            ident, = struct.unpack_from('<I', data, at)
            at += 4
            if ident == 0x20D:
                textures, = struct.unpack_from('<I', data, at)
                length = 4 + textures * 24
            else:
                length = {0x210: 20, 0x218: 0, 0x21C: 32, 0x238: 1, **(shader_sizes or {})}[ident]
            records.append((ident, data[at:at+length].hex()))
            at += length
        material_data.append(dict(version9_prefix=prefix, shader_data=records))
    count, = struct.unpack_from('<I', data, at)
    at += 4
    pairs = list(struct.iter_unpack('<2I', data[at:at+count*8]))
    at += count*8
    count, = struct.unpack_from('<I', data, at)
    at += 4
    parts = []
    for i in range(count):
        source_at = at
        a, b = transform(data, at), transform(data, at+56)
        at += 112
        field, low, typ, high, present = struct.unpack_from('<5I', data, at)
        at += 20
        bones = []
        bones_at = at
        if present:
            n, = struct.unpack_from('<I', data, at)
            at += 4
            bones_at = at
            for _ in range(n):
                bones.append(struct.unpack_from('<40I', data, at))
                at += 160
        parts.append(dict(index=i, source_at=source_at, transform_a=a, transform_b=b,
                          field=field, model_type=typ, model_instance=(high << 32)|low,
                          present=present, bones_at=bones_at, bone_words=bones))
    footer = list(struct.unpack_from('<3I', data, at))
    assert at+12 == len(data)
    return dict(source_at=start, materials=material_data, pairs=pairs, parts=parts, footer=footer)


def declarations(data):
    r = Reader(data)
    r.i32()
    n = r.i32(be=True)
    r.take(n*12)
    mesh_count = r.i32()
    r.take(28)
    for _ in range(r.i32()):
        primitive, count, bits, length = [r.i32() for _ in range(4)]
        r.take(count*(2 if bits == 16 else 4))
    formats = []
    for _ in range(r.i32()):
        n = r.i32()
        formats.append([struct.unpack_from('<hhbbbbi', data, r.p+j*12) for j in range(n)])
        r.take(n*12)
    buffers = []
    for _ in range(r.i32()):
        fmt, count, size = [r.i32() for _ in range(3)]
        stride = sum(DECL_SIZE[e[2]] for e in formats[fmt])
        buffers.append((formats[fmt], count, stride, r.p))
        r.take(count*stride)
    meshes = [(r.i32(), r.i32()) for _ in range(mesh_count)]
    return buffers, meshes


def skin_stats(data):
    buffers, meshes = declarations(data)
    result = []
    for mesh, (_, vb) in enumerate(meshes):
        fmt, count, stride, at = buffers[vb]
        indices = next((e for e in fmt if e[0] == 0 and e[4] == 2), None)
        weights = next((e for e in fmt if e[0] == 0 and e[4] == 1), None)
        row = dict(mesh=mesh, vertices=count, index_type=indices[2] if indices else None,
                   weight_type=weights[2] if weights else None)
        if indices and weights and indices[2] == 5 and weights[2] == 8:
            sums = Counter()
            active = []
            for i in range(count):
                js = data[at+i*stride+indices[1]:at+i*stride+indices[1]+4]
                ws = data[at+i*stride+weights[1]:at+i*stride+weights[1]+4]
                sums[sum(ws)] += 1
                active.extend(j for j, w in zip(js, ws) if w)
            row.update(weight_sums=dict(sums), max_active_index=max(active, default=0),
                       indices_divisible_by_three=all(j % 3 == 0 for j in active),
                       active_influences=len(active))
        result.append(row)
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('root', nargs='?', type=Path, default=Path('D:/Games/SPORE Collection'))
    ap.add_argument('--out', required=True, type=Path)
    ap.add_argument('--reuse', type=Path, help='reuse prior successful source records; re-read failures')
    args = ap.parse_args()
    reuse = {}
    if args.reuse:
        old = json.loads(args.reuse.read_text())
        reuse = {(str(Path(r['package'])).lower(), r['key']): r for r in old['records']}
        for r in reuse.values():
            r['tail']['materials'] = [m if isinstance(m, dict) else dict(version9_prefix=None, shader_data=m)
                                      for m in r['tail']['materials']]
    report = dict(packages=0, index_errors=[], resources=0, errors=[], records=[], summary={})
    counts = Counter()
    pair_ids, pair_counts, fields, flags, bone_finite = Counter(), Counter(), Counter(), Counter(), Counter()
    creature_rows = {}
    prop_models = {}
    packages = []
    for pkg in sorted(args.root.rglob('*.package')):
        try:
            entries = indexes(pkg)
        except Exception as e:
            report['index_errors'].append([str(pkg), str(e)])
            continue
        report['packages'] += 1
        packages.append((pkg, entries))
        for entry in entries:
            if entry['type'] == CREATURE:
                try:
                    rows, _, _, parts, _ = inspect_creature(payload(pkg, entry))
                    creature_rows.setdefault(entry['instance'] & 0xFFFFFFFF, []).append(
                        (str(pkg), key(entry), [(p['instance'], row.hex()) for p, row in zip(parts, rows)]))
                except Exception as e:
                    report['errors'].append([str(pkg), key(entry), f'creature {e}'])
    wanted_props = {int(inst, 16) for variants in creature_rows.values()
                    for _, _, rows in variants for inst, _ in rows}
    for pkg, entries in packages:
        for entry in entries:
            if entry['type'] == PROP and entry['instance'] in wanted_props:
                # Resolve only exact typed source model references; no field-name guessing.
                try:
                    models = []
                    for ident, typ, spec, value in properties(payload(pkg, entry)):
                        if ident in MODEL_SLOTS:
                            models.extend((mt, inst) for inst, mt in key_values(typ, spec, value))
                    if models:
                        prop_models.setdefault(entry['instance'], set()).update(models)
                except Exception:
                    pass
    for pkg, entries in packages:
        for entry in entries:
            if entry['type'] != GMDL:
                continue
            report['resources'] += 1
            try:
                cached = reuse.get((str(pkg).lower(), key(entry)))
                if cached:
                    t, rows = cached['tail'], cached['skin']
                else:
                    data = payload(pkg, entry)
                    t = tail(data)
                    rows = skin_stats(data) if t else []
                if t is None:
                    counts['core_only'] += 1
                    continue
                counts['with_tail'] += 1
                parts = t['parts']
                counts['part_records'] += len(parts)
                counts['bone_records'] += sum(len(p['bone_words']) for p in parts)
                declared = sum(p['field'] for p in parts if p['field'] != 0xFFFFFFFF)
                total_bones = declared + sum(len(p['bone_words']) for p in parts)
                pairs = t['pairs']
                pair_ids.update(p[0] for p in pairs)
                pair_counts.update(p[1] for p in pairs)
                counts['pair_total_equals_part_plus_bones'] += int(sum(p[1] for p in pairs) == total_bones)
                for p in parts:
                    fields.update([p['field']])
                    flags.update([p['transform_a']['flags'], p['transform_b']['flags']])
                    for bone in p['bone_words']:
                        floats = struct.unpack('<40f', struct.pack('<40I', *bone))
                        bone_finite.update(i for i, f in enumerate(floats) if not math.isfinite(f))
                for row in rows:
                    counts['mesh_records'] += 1
                    if 'weight_sums' in row:
                        counts['skinned_meshes'] += 1
                        counts['skinned_vertices'] += row['vertices']
                        counts['active_influences'] += row['active_influences']
                        counts['weight_sum_255_vertices'] += row['weight_sums'].get(255, 0)
                        counts['register_indices_meshes'] += row['indices_divisible_by_three']
                        applicable = [n for m, n in pairs if m == row['mesh']]
                        row['pair_palette_counts'] = applicable
                        row['indices_within_pair_palette'] = bool(applicable) and row['max_active_index'] < max(applicable)*3
                        counts['indices_within_pair_palette_meshes'] += row['indices_within_pair_palette']
                joins = []
                for cpkg, ck, source in creature_rows.get(entry['instance'] & 0xFFFFFFFF, []):
                    matches = []
                    for p in parts:
                        if p['model_type'] == 0xFFFFFFFF:
                            continue
                        source_matches = [i for i, (inst, raw) in enumerate(source)
                                          if (p['model_type'], p['model_instance']) in prop_models.get(int(inst,16), set())]
                        low_matches = [i for i, (inst, raw) in enumerate(source)
                                       if any(mt == p['model_type'] and (mi & 0xFFFFFFFF) == (p['model_instance'] & 0xFFFFFFFF)
                                              for mt, mi in prop_models.get(int(inst,16), set()))]
                        matches.append(dict(part=p['index'], source_rows=source_matches,
                                            source_rows_same_low_half_only=low_matches))
                    joins.append(dict(package=cpkg, creature=ck, parts=len(source), matches=matches))
                report['records'].append(dict(package=str(pkg), key=key(entry), tail=t, skin=rows, creature_joins=joins))
            except Exception as e:
                report['errors'].append([str(pkg), key(entry), str(e)])
        if any(e['type'] == GMDL for e in entries):
            print(f"package={pkg.name} gmdl_seen={report['resources']} decoded={len(report['records'])} errors={len(report['errors'])}", flush=True)
    report['summary'] = dict(counts=counts, pair_first_words=pair_ids, pair_second_words=pair_counts,
                             part_field_values=fields, transform_flags=flags, nonfinite_bone_float_words=bone_finite,
                             prop_model_keys=len(prop_models), creature_low_halves=len(creature_rows))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2, allow_nan=False))
    print(json.dumps({k:v for k,v in report.items() if k not in ('records', 'errors')}, indent=2))
    print('errors=', len(report['errors']), 'first_errors=', report['errors'][:5])


if __name__ == '__main__':
    main()
