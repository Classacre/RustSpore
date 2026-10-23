"""Independent initial source-part selector census; never emits a bone pose.

Raw payloads stay in the installation. Reports belong under ignored target/.
The independent geometry calculation uses transposed-matrix multiplication;
Rust uses the native matrix-to-quaternion input convention.
"""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import struct

from dbpf_extract import parse_index, payload
from anim_rig_mapping_probe import inspect_creature, resource_id
from part_capabilities_probe import inspect as inspect_dictionary

CREATURE = 0x0F43029A
CANDIDATE = '0F43029A:0001FFFF:40626200067C5390'


def prep(data, dictionary):
    rows, _, _, parts, _ = inspect_creature(data)
    if len(rows) > 255:
        raise ValueError('too many source parts')
    aggregates = {int(a['capability'], 16): [int(v, 16) for v in a['members']]
                  for a in dictionary['aggregates']}
    result = []
    for i, row in enumerate(rows):
        rotation = [list(struct.unpack_from('<3f', row, 48+j*12)) for j in range(3)]
        scale = struct.unpack_from('<f', row, 44)[0]
        low = struct.unpack_from('<3f', row, 20)
        high = struct.unpack_from('<3f', row, 32)
        translation = struct.unpack_from('<3f', row, 84)
        effects = struct.unpack_from('<3f', row, 96)
        values = [scale, *low, *high, *translation, *effects, *(v for r in rotation for v in r)]
        if not all(math.isfinite(v) for v in values):
            raise ValueError(f'part {i} nonfinite geometry')
        if any(abs(sum(rotation[a][k]*rotation[b][k] for k in range(3))-(1 if a == b else 0)) > 0.01
               for a in range(3) for b in range(3)):
            raise ValueError(f'part {i} exceeds supported orthonormal tolerance')
        determinant = sum(rotation[0][j] * (rotation[1][(j+1)%3]*rotation[2][(j+2)%3]
                                           -rotation[1][(j+2)%3]*rotation[2][(j+1)%3]) for j in range(3))
        if abs(determinant-1) > 0.02:
            raise ValueError(f'part {i} unsupported improper orientation')
        center = [(low[j]+high[j])*scale*0.5 for j in range(3)]
        position = [translation[j]+sum(rotation[k][j]*center[k] for k in range(3)) for j in range(3)]
        delta = [translation[j]-position[j] for j in range(3)]
        body = [sum(rotation[j][k]*delta[k] for k in range(3)) for j in range(3)]
        expanded = {}
        for cap in parts[i]['capabilities']:
            ident = int.from_bytes(bytes.fromhex(cap['id']), 'little')
            if ident:
                expanded[ident] = max(cap['number'], expanded.get(ident, 0))
        changed = True
        while changed:
            changed = False
            for ident, number in list(expanded.items()):
                for member in aggregates.get(ident, []):
                    if member and (member not in expanded or expanded[member] < number):
                        expanded[member] = number
                        changed = True
                        if len(expanded) > 32:
                            raise ValueError(f'part {i} capability overflow')
        result.append({'source_row': i, 'instance': parts[i]['instance'], 'position': position,
                       'half_extents': [v if (v := (high[j]-low[j])*scale*0.5) > 0 else 0.1 for j in range(3)],
                       'body_origin_local': body, 'effects_origin_local': [effects[j]*scale+body[j] for j in range(3)],
                       'expanded_capabilities': {f'{k:08X}': v for k, v in sorted(expanded.items())}})
    return result


def select(blocks, words):
    flags, capability, alternate, reserved = words
    side, extremum = flags & 0x2000C, flags & 0xE000
    if (flags & ~(0x2000C | 0xE000) or alternate or reserved or extremum == 0xE000
            or side not in (0, 4, 8, 12, 0x20000, 0x20004, 0x20008)):
        return {'unsupported_selector': f'{flags:08X}'}
    matching = [b for b in blocks if b['expanded_capabilities'].get(f'{capability:08X}', 0)]
    low = min((b['position'][0] for b in matching), default=math.inf)
    high = max((b['position'][0] for b in matching), default=-math.inf)
    selected = []
    for b in matching:
        x = b['position'][0]
        threshold = sum(b['half_extents'])*0.15
        excluded = {
            0: lambda: False, 4: lambda: x < -threshold, 8: lambda: x > threshold,
            12: lambda: abs(x) > threshold,
            0x20000: lambda: math.floor(x*100) < math.floor((high-(high-low)*0.525)*100),
            0x20004: lambda: math.floor(x*100) > math.floor(((high-low)*0.525+low)*100),
            0x20008: lambda: abs(math.floor((x-((high-low)*0.5+low))*100)) > 10,
        }[side]()
        if not excluded:
            selected.append(b)
    if extremum and selected:
        axis = {0x2000: 0, 0x4000: 0, 0x6000: 1, 0x8000: 1, 0xA000: 2, 0xC000: 2}[extremum]
        chooser = max if extremum in (0x2000, 0x8000, 0xA000) else min
        selected = [chooser(selected, key=lambda b: b['position'][axis])]
    return {'source_rows': [b['source_row'] for b in selected], 'unsupported_preparation': 'NativeFinalizationRequired'}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('root', nargs='?', type=Path, default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--survey', required=True, type=Path)
    parser.add_argument('--rust-blocks', type=Path, help='Compare initial geometry TSV emitted by anim_binding_verify')
    parser.add_argument('--selection-digest', type=Path)
    parser.add_argument('--rust-selection-digest', type=Path)
    parser.add_argument('--geometry-tolerance', type=float, default=0.01,
                        help='Absolute matrix-vs-quaternion tolerance; does not assert native-runtime equality')
    args = parser.parse_args()
    indexes, skipped = [], []
    for path in sorted(args.root.rglob('*.package')):
        try:
            entries, _ = parse_index(path)
            indexes.append((path, entries))
        except ValueError as error:
            skipped.append({'package': str(path), 'error': str(error)})
    source = [(p, e) for p, entries in indexes for e in entries if e['type'] == 0x7C19AA7A
              and e['instance'] == 0x000082D0142D3338 and p.name == 'PatchData.package']
    if len(source) != 1:
        raise ValueError('explicit PatchData v4 dictionary source is not unique')
    dictionary = inspect_dictionary(payload(*source[0]))
    counts, failures, blocks_by_key = Counter(), [], {}
    for path, entries in indexes:
        for entry in entries:
            if entry['type'] != CREATURE:
                continue
            counts['creatures'] += 1
            try:
                blocks = prep(payload(path, entry), dictionary)
                counts['prepared_creatures'] += 1
                counts['prepared_source_parts'] += len(blocks)
                blocks_by_key[f'{path.name} {resource_id(entry)}'] = blocks
            except ValueError as error:
                failures.append({'package': path.name, 'key': resource_id(entry), 'unsupported': str(error)})
    survey = json.loads(args.survey.read_text())
    walk = [c for c in survey['locomotion_candidates'] if c['package'] == 'PatchData.package'
            and c['key'] == 'EE17C6AD:0001FFFF:00000000DAC89265']
    if len(walk) != 1:
        raise ValueError('explicit patched BP1_cycle_walk source is not unique')
    candidate = blocks_by_key['Spore_Content.package '+CANDIDATE]
    selections = [{'channel': c['name'], 'selector': [f'{v:08X}' for v in c['selectors'][1:5]],
                   **select(candidate, c['selectors'][1:5])} for c in walk[0]['channels']]
    digest_lines = []
    for key, blocks in sorted(blocks_by_key.items()):
        words = [len(walk[0]['channels'])]
        for channel in walk[0]['channels']:
            selector = channel['selectors'][1:5]
            words.extend(selector)
            result = select(blocks, selector)
            if 'unsupported_selector' in result:
                words.append(1)
            else:
                words.extend([0, len(result['source_rows']), *result['source_rows']])
        value = 0xCBF29CE484222325
        for byte in b'initial-source-selectors-v1\0'+struct.pack('<'+'I'*len(words), *words):
            value = ((value ^ byte)*0x100000001B3) & 0xFFFFFFFFFFFFFFFF
        digest_lines.append(f'{key} {value:016X}')
    if args.selection_digest:
        args.selection_digest.write_text('\n'.join(digest_lines)+'\n')
    digest_comparison = None
    if args.rust_selection_digest:
        rust = args.rust_selection_digest.read_text().splitlines()
        digest_comparison = {'python_rows': len(digest_lines), 'rust_rows': len(rust),
                             'equal': digest_lines == rust,
                             'only_python': sorted(set(digest_lines)-set(rust))[:10],
                             'only_rust': sorted(set(rust)-set(digest_lines))[:10]}
    comparison = None
    if args.rust_blocks:
        maxima = Counter()
        seen = set()
        for line in args.rust_blocks.read_text().splitlines():
            package, key, ordinal, *raw = line.split('\t')
            i = int(ordinal)
            expected = blocks_by_key[package+' '+key][i]
            if (package+' '+key, i) in seen or len(raw) != 12:
                raise ValueError('duplicate or incomplete Rust block row')
            seen.add((package+' '+key, i))
            values = list(map(float, raw))
            if not all(math.isfinite(value) for value in values):
                raise ValueError('nonfinite Rust block observation')
            for field, start in [('position', 0), ('half_extents', 3), ('body_origin_local', 6), ('effects_origin_local', 9)]:
                maxima[field] = max(maxima[field], *(abs(a-b) for a, b in zip(expected[field], values[start:start+3])))
        comparison = {'rows': len(seen), 'expected_rows': counts['prepared_source_parts'],
                      'max_absolute_difference': dict(maxima), 'tolerance': args.geometry_tolerance,
                      'passed': len(seen) == counts['prepared_source_parts'] and
                                all(value <= args.geometry_tolerance for value in maxima.values())}
    report = {'scope': {'wide_indexes': len(indexes), 'skipped_indexes': skipped}, 'counts': counts,
              'unsupported': failures, 'dictionary_key': resource_id(source[0][1]),
              'candidate_key': CANDIDATE, 'candidate_blocks': candidate,
              'BP1_cycle_walk_source_candidates': selections, 'rust_comparison': comparison,
              'rust_selection_digest_comparison': digest_comparison,
              'unsupported_preparation': 'NativeFinalizationRequired'}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2))
    print(json.dumps({'counts': counts, 'unsupported_count': len(failures), 'selections': selections,
                      'rust_comparison': comparison, 'rust_selection_digest_comparison': digest_comparison}, indent=2))
    if comparison is not None and not comparison['passed']:
        raise SystemExit(1)
    if digest_comparison is not None and not digest_comparison['equal']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
