#!/usr/bin/env python3
"""Compare synthetic diagnostic buffers with scalar expectations from decoded Jawhead.

The hash-bound local census supplies layout metadata. Source triangle and influence
bytes are reread and checked; no claim is made about a live draw or retail pose.
All inputs and outputs remain under target/source-cell.
"""
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

ROOT = Path('target/source-cell')
SOURCE = ROOT / 'jawhead-decoded-v1.gmdl'
CENSUS = ROOT / 'jawhead-skin-census-v1.json'
OUTPUT = ROOT / 'gmdl-position-diagnostic-source-v1'
REPORT = OUTPUT / 'comparison.json'
ABS_TOL = 2e-5
REL_TOL = 2e-6


def compare():
    data = SOURCE.read_bytes()
    census = json.loads(CENSUS.read_text(encoding='utf-8-sig'))
    digest = hashlib.sha256(data).hexdigest()
    if len(data) != census['payload_size'] or digest != census['payload_sha256']:
        raise ValueError('source size or SHA-256 differs from census')
    meshes, descriptors = census['meshes'], census['palette_descriptors']
    if len(meshes) != 2 or len(descriptors) != 2:
        raise ValueError('expected two Jawhead meshes and descriptors')
    counts = []
    max_error = {'identity': 0.0, 'translation': 0.0}
    for ordinal, (mesh, desc) in enumerate(zip(meshes, descriptors)):
        if mesh['mesh'] != ordinal:
            raise ValueError('mesh order mismatch')
        first, count = struct.unpack_from('<II', data, desc['offset'])
        if (first, count) != (desc['first_word'], desc['count']):
            raise ValueError('descriptor source mismatch')
        tri = mesh['triangle_source']
        bits, n, at = tri['index_bits'], tri['index_count'], tri['data_offset']
        if bits not in (16, 32):
            raise ValueError('unsupported index width')
        vertices = sorted(set(struct.unpack_from('<' + ('H' if bits == 16 else 'I') * n, data, at)))
        if vertices != tri['referenced_vertices'] or len(vertices) != tri['unique_referenced_vertices']:
            raise ValueError('triangle references differ from census')
        elements = mesh['declaration_elements']
        def element(usage, typ):
            found = [e for e in elements if e['usage'] == usage and e['usage_index'] == 0]
            if len(found) != 1 or found[0]['stream'] != 0 or found[0]['type'] != typ or found[0]['method'] != 0:
                raise ValueError('unsupported declaration')
            return found[0]['offset']
        pos, indices, weights = element(0, 2), element(2, 5), element(1, 8)
        stride, start, end = mesh['stride'], mesh['vertex_data_offset'], mesh['vertex_data_end_exclusive']
        if end != start + mesh['vertex_count'] * stride or end > len(data):
            raise ValueError('invalid buffer extent')
        source = data[start:end]
        mask = bytearray(len(source))
        for vertex in vertices:
            if not 0 <= vertex < mesh['vertex_count']:
                raise ValueError('triangle vertex out of bounds')
            base = vertex * stride
            if min(pos, indices, weights) < 0 or max(pos + 12, indices + 4, weights + 4) > stride:
                raise ValueError('declaration outside stride')
            mask[base + pos:base + pos + 12] = b'\x01' * 12
            ix, wt = list(source[base + indices:base + indices + 4]), list(source[base + weights:base + weights + 4])
            if [ix, wt] != mesh['per_vertex_raw_indices_and_weights'][vertex] or sum(wt) != 255:
                raise ValueError('influence bytes or weight sum mismatch')
            for index, weight in zip(ix, wt):
                if weight and (index % 3 or index // 3 >= count):
                    raise ValueError('active palette index out of bounds')
        for mode in ('identity', 'translation'):
            actual = (OUTPUT / f'{mode}-mesh-{ordinal}.bin').read_bytes()
            if len(actual) != len(source):
                raise ValueError(f'{mode} mesh {ordinal}: buffer size mismatch')
            if any(a != b for a, b, is_position in zip(actual, source, mask) if not is_position):
                raise ValueError(f'{mode} mesh {ordinal}: non-position or unreferenced byte changed')
            for vertex in vertices:
                base = vertex * stride
                p = struct.unpack_from('<3f', source, base + pos)
                got = struct.unpack_from('<3f', actual, base + pos)
                ix = source[base + indices:base + indices + 4]
                wt = source[base + weights:base + weights + 4]
                if not all(math.isfinite(x) for x in (*p, *got)):
                    raise ValueError('non-finite position')
                for axis in range(3):
                    scale = (0.125, -0.0625, 0.03125)[axis]
                    expected = sum((p[axis] + (first + i // 3 + 1) * scale if mode == 'translation' else p[axis]) * w / 255.0 for i, w in zip(ix, wt) if w)
                    error = abs(got[axis] - expected)
                    max_error[mode] = max(max_error[mode], error)
                    if not math.isclose(got[axis], expected, abs_tol=ABS_TOL, rel_tol=REL_TOL):
                        raise ValueError(f'{mode} mesh {ordinal} vertex {vertex} axis {axis}: {got[axis]} != {expected}')
        counts.append(len(vertices))
    return {'status': 'pass', 'source_sha256': digest, 'referenced_vertices_per_mesh': counts,
            'position_components_compared': sum(counts) * 3 * 2, 'max_absolute_error': max_error,
            'absolute_tolerance': ABS_TOL, 'relative_tolerance': REL_TOL,
            'output_directory': str(OUTPUT), 'scope': 'synthetic palettes, conditional direct draw; no live draw, retail animation, or gameplay claim'}


def main():
    try:
        report = compare()
    except (OSError, ValueError, KeyError, TypeError, IndexError, struct.error) as exc:
        report = {'status': 'fail', 'error': str(exc)}
    OUTPUT.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    sys.exit(main())
