"""Independently check explicit authored creation/LOD0/material/raster inputs.

Matches the bounded inspection loader's local PatchData then path-order lookup.
This does not establish retail package precedence or original shader rendering.
Only geometry counts and base-level source RGBA are compared, not GPU pixels.
Keep reports and source-derived digests under ignored target/.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import struct

from creature_model_links_probe import selected_keys
from dbpf_extract import payload
from gmdl_full import Reader
from gmdl_skin_probe import declarations, tail
from prop_numeric_probe import index
from raster_digest import fnv1a64
from raster_extract import parse_raster_header, decode_dxt, decode_raw_rgba, FMT_DXT1, FMT_DXT5, FMT_RAW

CREATURE, PROP, GMDL, RASTER = 0x0F43029A, 0x00B1B104, 0x00E6BCE5, 0x2F4E681C
DEFAULTS = (0x40626200067C5390, 0x40666200D1B4BB56, 0x406B62000660EACA)


def key(entry):
    return f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"


def geometry_counts(data):
    reader = Reader(data)
    reader.i32()
    reader.take(reader.i32(be=True) * 12)
    reader.i32()
    reader.take(28)
    triangles = []
    for _ in range(reader.i32()):
        reader.i32()
        count, bits, _length = [reader.i32() for _ in range(3)]
        assert count % 3 == 0 and bits in (16, 32)
        triangles.append(count // 3)
        reader.take(count * (bits // 8))
    buffers, meshes = declarations(data)
    return sum(buffers[vb][1] for _, vb in meshes), sum(triangles[tb] for tb, _ in meshes), len(meshes)


def source_sampler(data, meshes):
    skin = tail(data)
    assert skin and len(skin['materials']) == meshes and meshes
    selected = []
    for material in skin['materials']:
        fields = [bytes.fromhex(raw) for field, raw in material['shader_data'] if field == 0x20D]
        assert fields
        raw = fields[-1]
        count, = struct.unpack_from('<I', raw)
        assert count <= 6 and len(raw) == 4 + count * 24
        bindings = []
        for ordinal in range(count):
            at = 4 + ordinal * 24
            sampler, = struct.unpack_from('<I', raw, at)
            state = raw[at+8:at+16]
            instance, = struct.unpack_from('<Q', raw, at+16)
            if sampler == 0:
                bindings.append((instance, state))
        assert bindings
        instance, state = bindings[-1]
        assert state[0] & 15 == 15 and state[1] in (1, 2, 3)
        assert state[2] in (1, 2) and state[3] in (0, 1, 2) and state[4] == 1
        selected.append((instance, state))
    # Irrelevant reserved state bytes need not be shared by separate materials.
    assert len({(instance, state[1:5]) for instance, state in selected}) == 1
    return selected[-1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--digest-out', type=Path, required=True)
    parser.add_argument('--rust-digest', type=Path)
    parser.add_argument('--instances', nargs='+', type=lambda value: int(value, 16), default=DEFAULTS)
    args = parser.parse_args()
    for path in (args.out, args.digest_out):
        if 'target' not in path.parts:
            raise ValueError('source-derived outputs must remain under ignored target/')
    patch = args.install / 'Data/PatchData.package'
    paths = [patch, *sorted(args.install.rglob('*.package'))]
    packages = [(path, index(path)) for path in dict.fromkeys(paths)]

    def find(kind, instance):
        for path, entries in packages:
            for entry in entries:
                if entry['type'] == kind and entry['instance'] == instance:
                    return path, entry
        raise ValueError(f'missing exact resource {kind:08X}:{instance:016X}')

    rows, records = [], []
    for instance in args.instances:
        _, creature = find(CREATURE, instance)
        prop_path, prop = find(PROP, instance)
        prop_data = payload(prop_path, prop)
        _count, selected, _digest = selected_keys(prop_data)
        lod0 = [item for item in selected if item['property'] == '00F9EFBB']
        assert len(lod0) == 1 and lod0[0]['type'] == '0020' and lod0[0]['specifier'] == '0000'
        assert len(lod0[0]['keys']) == 1
        reference = lod0[0]['keys'][0]
        assert int(reference['type'], 16) == GMDL
        model_path, model = find(GMDL, int(reference['instance'], 16))
        model_data = payload(model_path, model)
        vertices, triangles, meshes = geometry_counts(model_data)
        raster_instance, settings = source_sampler(model_data, meshes)
        raster_path, raster = find(RASTER, raster_instance)
        raster_data = payload(raster_path, raster)
        header = parse_raster_header(raster_data)
        width, height, fmt = (header[name] for name in ('width', 'height', 'format'))
        if fmt in (FMT_DXT1, FMT_DXT5):
            rgba = decode_dxt(raster_data[28:], width, height, fmt)
        elif fmt == FMT_RAW:
            rgba = decode_raw_rgba(raster_data[28:], width, height)
        else:
            raise ValueError(f'unhandled measured raster format {fmt:08X}')
        rows.append('\t'.join(map(str, (key(creature), key(prop), key(model), key(raster),
            vertices, triangles, width, height, settings.hex().upper(), f'{fnv1a64(rgba):016X}'))))
        records.append({'creature': key(creature), 'prop': key(prop), 'model': key(model),
            'raster': key(raster), 'vertices': vertices, 'triangles': triangles,
            'width': width, 'height': height, 'settings': settings.hex(),
            'prop_sha256': hashlib.sha256(prop_data).hexdigest(),
            'model_sha256': hashlib.sha256(model_data).hexdigest(),
            'raster_sha256': hashlib.sha256(raster_data).hexdigest(),
            'rgba_sha256': hashlib.sha256(rgba).hexdigest()})
    normalized = '\n'.join(rows) + '\n'
    args.digest_out.parent.mkdir(parents=True, exist_ok=True)
    args.digest_out.write_text(normalized, encoding='utf-8')
    comparison = None
    if args.rust_digest:
        actual = args.rust_digest.read_text(encoding='utf-8')
        comparison = {'rows': len(rows), 'exact': actual == normalized}
        if actual != normalized:
            comparison['mismatches'] = [dict(ordinal=i, python=expected, rust=found)
                for i, (expected, found) in enumerate(zip(rows, actual.splitlines())) if expected != found]
    report = {'scope': 'bounded authored LOD0 saved geometry counts and base-mip RGBA; no retail shader/GPU claim',
        'counts': {'packages': len(packages), 'creatures': len(records)}, 'records': records,
        'normalized_digest_sha256': hashlib.sha256(normalized.encode()).hexdigest(), 'comparison': comparison}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({key: report[key] for key in ('scope', 'counts', 'normalized_digest_sha256', 'comparison')}))
    if comparison and not comparison['exact']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
