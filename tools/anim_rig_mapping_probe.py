"""Measure joins between ANIM selectors, creature parts and optional RW4 IDs.

Reads package indexes and individual payloads, never entire archives. Existing
wide-index helper limitations are reported explicitly. No inferred correspondence
is used to deform geometry. Reports belong under ignored target/.
"""
import argparse
from collections import Counter, defaultdict
import csv
import json
import math
from pathlib import Path
import re
import struct

from dbpf_extract import parse_index, payload
from prop_full import entry_size, is_separator, KNOWN_SPECS, KNOWN_TYPES
from type_0f43_validate import parse as validate_creature, read_compressed

ANIM = 0xEE17C6AD
CREATURE = 0x0F43029A
PROP = 0x00B1B104
MODELS = {0x2F4E681B, 0x01C135DA, 0x00E6BCE5}
MODEL_SLOTS = [0x00F9EFBB, 0x00F9EFBC, 0x00F9EFBD, 0x00F9EFBE]
LOCOMOTION = re.compile(r'walk|run|locom|gait|sprint|idle', re.I)


def u32(data, at):
    return struct.unpack_from('<I', data, at)[0]


def fourcc(value):
    raw = struct.pack('<I', value).rstrip(b'\0')
    if 2 <= len(raw) <= 4 and all(32 <= c < 127 for c in raw):
        return raw.decode('ascii')
    return None


def fnv32(value, multiply_first=False):
    result = 0x811C9DC5
    for byte in value.lower().encode('ascii', 'replace'):
        if multiply_first:
            result = ((result * 0x1000193) ^ byte) & 0xFFFFFFFF
        else:
            result = ((result ^ byte) * 0x1000193) & 0xFFFFFFFF
    return result


def resource_id(entry):
    return f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"


def properties(data):
    """Use the repository's measured robust property walk, retaining fields."""
    at = 4
    records = []
    while at + 8 <= len(data):
        if is_separator(data, at):
            at += 4
            continue
        ident, typ, spec = struct.unpack_from('>IHH', data, at)
        header = 8
        size = entry_size(data, at + header, typ, spec)
        if size is None:
            typ, spec = struct.unpack_from('>HH', data, at)
            header = 4
            size = entry_size(data, at + header, typ, spec)
            if typ not in KNOWN_TYPES or spec not in KNOWN_SPECS or size is None:
                raise ValueError(f'property walk stopped at {at}')
            ident = None
        value = data[at+header:at+header+size]
        records.append((ident, typ, spec, value))
        at += header + size
    if is_separator(data, at):
        at += 4
    if at != len(data):
        raise ValueError(f'property tail {len(data)-at}')
    return records


def key_values(typ, spec, value):
    if typ != 0x20:
        return []
    if spec in (0x001C, 0x009C):
        count, stride = struct.unpack_from('>II', value)
        chunks = [value[8+i*stride:8+(i+1)*stride] for i in range(count)]
    else:
        chunks = [value]
    # Measured key layout: low instance word, type, high instance word.
    return [(a | (c << 32), b) for a, b, c in
            (struct.unpack_from('<3I', chunk) for chunk in chunks)]


def inspect_creature(data):
    if not validate_creature(data):
        raise ValueError('creature layout is not exact')
    version = u32(data, 4)
    counts = struct.unpack_from('<' + 'I'*(3 if version == 9 else 4), data, 8)
    start = read_compressed(data, 8+4*len(counts), 0x44)
    rows = [data[start+i*140:start+(i+1)*140] for i in range(counts[0])]
    cap_start = start + counts[0]*140
    raw_caps = [data[cap_start+4*i:cap_start+4*i+4] for i in range(counts[1])]
    caps = [cap.rstrip(b'\0').decode('ascii', 'replace') for cap in raw_caps]
    numbers = data[cap_start+4*counts[1]:cap_start+5*counts[1]]
    morph_start = cap_start + counts[1]*5
    morph_ids = [u32(data, morph_start+4*i) for i in range(counts[2])]
    morphs = [(morph_ids[i], u32(data, morph_start+4*(counts[2]+i)),
               u32(data, morph_start+4*(2*counts[2]+i))) for i in range(counts[2])]
    digest = 0xCBF29CE484222325
    def feed(raw):
        nonlocal digest
        for value in raw:
            digest = ((digest ^ value) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    def word(value):
        feed(struct.pack('<I', value))
    feed(b'creature-rig-v1\0')
    for value in (version, len(rows), len(caps), len(morphs)):
        word(value)
    parts = []
    cap_cursor = morph_cursor = 0
    for ordinal, row in enumerate(rows):
        fields = struct.unpack_from('<6H', row)
        cap_at, morph_at, cap_count, morph_count, reserved = struct.unpack_from('<HHBBH', row, 12)
        if cap_at != cap_cursor or cap_at+cap_count > len(caps):
            raise ValueError(f'part {ordinal} capability range {cap_at}+{cap_count} expected {cap_cursor}/{len(caps)}')
        if morph_at != morph_cursor or morph_at+morph_count > len(morphs):
            raise ValueError(f'part {ordinal} morph range {morph_at}+{morph_count} expected {morph_cursor}/{len(morphs)}')
        cap_cursor, morph_cursor = cap_at+cap_count, morph_at+morph_count
        for value in (*fields, cap_at, morph_at, cap_count, morph_count, reserved,
                      u32(row, 132), u32(row, 136)):
            word(value)
        for offset in [20, 24, 28, 32, 36, 40, *range(48, 84, 4), 84, 88, 92, 108, 112, 116, 120, 124]:
            word(u32(row, offset))
        word(cap_count)
        part_caps = []
        for i in range(cap_at, cap_cursor):
            word(int.from_bytes(raw_caps[i], 'little'))
            word(numbers[i])
            part_caps.append({'id': raw_caps[i].hex().upper(), 'tag': caps[i], 'number': numbers[i]})
        word(morph_count)
        part_morphs = []
        for ident, a, b in morphs[morph_at:morph_cursor]:
            for value in (ident, a, b):
                word(value)
            part_morphs.append({'id': f'{ident:08X}', 'value_bits': [f'{a:08X}', f'{b:08X}']})
        parts.append({'instance': f'{((u32(row, 132) << 32) | u32(row, 136)):016X}',
                      'fields_u16': list(fields), 'capability_start': cap_at, 'morph_start': morph_at,
                      'capability_count': cap_count, 'morph_count': morph_count, 'range_reserved': reserved,
                      'capabilities': part_caps, 'morphs': part_morphs,
                      'translation': list(struct.unpack_from('<3f', row, 84))})
    if cap_cursor != len(caps) or morph_cursor != len(morphs):
        raise ValueError('part ranges do not cover the flat arrays')
    return rows, caps, morph_ids, parts, f'{digest:016X}'


def refresh_creatures(report, indexes):
    """Refresh only the source creature records while reusing the ANIM survey."""
    creatures = []
    for package, entries in indexes:
        for entry in entries:
            if entry['type'] == CREATURE:
                _, caps, _, parts, digest = inspect_creature(payload(package, entry))
                creatures.append({'package': package.name, 'key': resource_id(entry),
                                  'capabilities': caps, 'parts': parts, 'rig_digest': digest})
    report['creatures'] = creatures


def write_creature_digests(report, path):
    if path:
        rows = sorted(f"{c['package']} {c['key']} {c['rig_digest']}" for c in report['creatures'])
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('\n'.join(rows)+'\n', encoding='utf-8')


def inspect_anim(data, package, entry, names, selector_values, selectors, clips, component_tags):
    count, table = u32(data, 324), u32(data, 328)
    source_path = data[12:272].split(b'\0')[0].decode('ascii', 'replace')
    basename = source_path.replace('\\', '/').split('/')[-1]
    candidate = bool(LOCOMOTION.search(basename))
    channels = []
    for index in range(count):
        at = u32(data, table + index*4)
        assert data[at:at+4] == b'CHAN'
        name = data[at+8:at+136].split(b'\0')[0].decode('ascii', 'replace')
        words = struct.unpack_from('<19I', data, at+136)
        keys, pointer, ncomp, comp_at = struct.unpack_from('<4I', data, at+212)
        components = [struct.unpack_from('<I4s6I', data, comp_at+32*i) for i in range(ncomp)]
        component_tags.update(c[1].hex().upper() for c in components)
        names[name] += 1
        selectors[name][words] += 1
        for i, value in enumerate(words):
            selector_values[i][value] += 1
        row = {'name': name, 'selectors': list(words), 'key_count': keys,
               'tags': [c[1].hex().upper() for c in components]}
        if candidate and keys:
            by_tag = {c[1]: c for c in components}
            info = by_tag[b'INFO']
            stride = info[3]
            row['frame_range'] = [u32(data, pointer+info[2]),
                                  u32(data, pointer+(keys-1)*stride+info[2])]
            row['finite'] = True
            row['dynamic_position'] = False
            row['dynamic_rotation'] = False
            for tag, width, label in [(b'POS\0', 3, 'position'), (b'ROT\0', 4, 'rotation')]:
                component = by_tag.get(tag)
                if component:
                    values = [struct.unpack_from('<'+'f'*width, data, pointer+i*stride+component[2]) for i in range(keys)]
                    row['finite'] &= all(math.isfinite(v) for value in values for v in value)
                    row['dynamic_'+label] = any(value != values[0] for value in values[1:])
                    if label == 'position' and row['finite']:
                        row['position_extent'] = [max(v[j] for v in values)-min(v[j] for v in values) for j in range(3)]
                        row['position_end_minus_start'] = [values[-1][j]-values[0][j] for j in range(3)]
        channels.append(row)
    if candidate:
        clips.append({'package': package.name, 'key': resource_id(entry), 'name': basename,
                      'duration_frames': struct.unpack_from('<f', data, 288)[0],
                      'frame_seconds': struct.unpack_from('<f', data, 284)[0], 'channels': channels})


def analyse(report, inventory_path, root):
    parts = report['part_props']
    primary_models = {model['instance'] for part in parts for prop in part['props']
                      for model in prop.get('models', []) if model['slot'] == '00F9EFBB'}
    summary = {'part_prop_instances': len(parts),
               'part_prop_records': sum(len(part['props']) for part in parts),
               'part_prop_parse_failures': sum('error' in prop for part in parts for prop in part['props']),
               'direct_lod0_model_instances': len(primary_models),
               'direct_model_types': Counter(model['type'] for part in parts for prop in part['props'] for model in prop.get('models', [])),
               'primary_capability_tags': [], 'unknown_primary_capability_tags': [],
               'finite_locomotion_candidates': sum(all(channel.get('finite', True) for channel in clip['channels']) for clip in report['locomotion_candidates'])}
    graph = Counter()
    ranges = Counter()
    roots = Counter()
    for creature in report['creatures']:
        rows = creature['parts']
        if all('capabilities' in row for row in rows):
            ranges['creatures'] += 1
            ranges['rows'] += len(rows)
            ranges['capabilities'] += sum(len(row['capabilities']) for row in rows)
            ranges['morphs'] += sum(len(row['morphs']) for row in rows)
        roots[str(sum(row['fields_u16'][2] == 65535 for row in rows))] += 1
        for i, row in enumerate(rows):
            fields = row['fields_u16']
            graph['rows'] += 1
            graph['index_is_ordinal'] += fields[0] == i
            graph['candidate_parent_valid'] += fields[2] == 65535 or fields[2] < len(rows)
            graph['candidate_parent_self'] += fields[2] == i
            seen = set()
            at = i
            while at != 65535 and at < len(rows) and at not in seen:
                seen.add(at)
                at = rows[at]['fields_u16'][2]
            graph['candidate_parent_chain_terminates'] += at == 65535
            mirror = fields[3]
            if mirror != 65535:
                graph['candidate_mirror_references'] += 1
                graph['candidate_mirror_valid'] += mirror < len(rows)
                graph['candidate_mirror_reciprocal'] += mirror < len(rows) and rows[mirror]['fields_u16'][3] == i
    summary['rig_graph'] = dict(graph)
    summary['validated_part_range_partitions'] = dict(ranges)
    summary['candidate_parent_roots_per_creature'] = dict(roots)
    content_package = root / 'Data/Spore_Content.package'
    if content_package.exists():
        entries, _ = parse_index(content_package)
        skins = {entry['instance'] & 0xFFFFFFFF for entry in entries if entry['type'] == 0x2F4E681C}
        ready = {part['instance']: any(model['present'] for prop in part['props'] for model in prop.get('models', [])
                                       if model['slot'] == '00F9EFBB') for part in parts}
        candidates = [creature for creature in report['creatures'] if creature['package'] == 'Spore_Content.package'
                      and {'foot', 'grsp', 'mout', 'eye'} <= set(creature['capabilities'])
                      and all(ready[part['instance']] for part in creature['parts'])
                      and int(creature['key'].split(':')[-1], 16) & 0xFFFFFFFF in skins]
        summary['complete_skinned_capable_creatures'] = [
            {'key': creature['key'], 'part_rows': len(creature['parts']), 'capability_counts': dict(Counter(creature['capabilities']))}
            for creature in sorted(candidates, key=lambda c: len(c['parts']))]
    for value in report['selector_values'][2].values():
        if value['fourcc']:
            row = {'tag': value['fourcc'], 'channels': value['count'],
                   'creature_records': report['creature_capabilities'].get(value['fourcc'], 0)}
            summary['primary_capability_tags'].append(row)
            if not row['creature_records']:
                summary['unknown_primary_capability_tags'].append(row)
    if inventory_path:
        with inventory_path.open(newline='', encoding='utf-8') as file:
            bones = list(csv.DictReader(file, delimiter='\t'))
        bone_ids = {int(bone['bone_id'], 16) for bone in bones}
        part_bones = [bone for bone in bones if bone['resource_key'].split(':')[-1] in primary_models]
        part_bone_ids = {int(bone['bone_id'], 16) for bone in part_bones}
        def matches(algorithm, allowed):
            return [{'name': name, 'hash': hashes[algorithm], 'channels': report['channel_names'][name]}
                    for name, hashes in report['hash_candidates'].items() if int(hashes[algorithm], 16) in allowed]
        summary['rw4'] = {'inventory_rows': len(bones), 'distinct_bone_ids': len(bone_ids),
                          'part_model_instances_with_skeleton': len({bone['resource_key'].split(':')[-1] for bone in part_bones}),
                          'part_inventory_rows': len(part_bones), 'part_distinct_bone_ids': len(part_bone_ids),
                          'selector_direct_matches': [sum(value['count'] for raw, value in field.items() if int(raw, 16) in bone_ids)
                                                      for field in report['selector_values']],
                          'name_fnv1_matches': matches('fnv1', bone_ids), 'name_fnv1a_matches': matches('fnv1a', bone_ids),
                          'part_name_fnv1_matches': matches('fnv1', part_bone_ids)}
        summary['rw4']['joint_series_hash_candidates'] = [
            {'name': f'joint{i}', 'hash': f'{fnv32(f"joint{i}", True):08X}'}
            for i in range(512) if fnv32(f'joint{i}', True) in part_bone_ids]
        if 'component_tags' in report:
            tag_ids = {int.from_bytes(bytes.fromhex(tag), 'little'): count for tag, count in report['component_tags'].items()}
            summary['rw4']['component_tag_bone_matches'] = {f'{tag:08X}': count for tag, count in tag_ids.items() if tag in bone_ids}
            summary['component_tag_creature_morph_matches'] = {f'{tag:08X}': count for tag, count in tag_ids.items()
                                                              if f'{tag:08X}' in report.get('creature_morph_ids', {})}
        # Count qualified target ambiguity without choosing any target. Collapse
        # repeated package versions; repeated placed parts remain distinct.
        by_model = defaultdict(set)
        for bone in part_bones:
            by_model[bone['resource_key'].split(':')[-1]].add((bone['skeleton_section'], bone['bone_index'], bone['bone_id']))
        model_by_part = {part['instance']: {model['instance'] for prop in part['props'] for model in prop.get('models', [])
                                           if model['slot'] == '00F9EFBB'} for part in parts}
        root_distribution = Counter()
        for creature in report['creatures']:
            seen = set()
            targets = 0
            for part in creature['parts']:
                key = (part['instance'], part['fields_u16'][0])
                if key in seen:
                    continue
                seen.add(key)
                targets += sum(bone[2] == '7BD05F71' for model in model_by_part[part['instance']] for bone in by_model[model])
            root_distribution[str(targets)] += 1
        summary['rw4']['candidate_root_targets_per_creature'] = root_distribution
    report['join_summary'] = summary
    return summary


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('root', nargs='?', type=Path, default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--bone-inventory', type=Path,
                        help='Optional TSV inventory emitted by rw4_animation_verify')
    parser.add_argument('--from-report', type=Path, help='Recompute joins from an existing survey without rereading assets')
    parser.add_argument('--refresh-creatures', action='store_true', help='With --from-report, reread only source creatures')
    parser.add_argument('--creature-digest-out', type=Path, help='Canonical per-part typed/range digests for Rust comparison')
    args = parser.parse_args()
    if args.from_report:
        report = json.loads(args.from_report.read_text(encoding='utf-8'))
        if args.refresh_creatures:
            indexes = []
            for package in sorted(args.root.rglob('*.package')):
                try:
                    entries, _ = parse_index(package)
                    indexes.append((package, entries))
                except ValueError:
                    pass  # Explicit skipped-index denominator remains in the source report.
            refresh_creatures(report, indexes)
        write_creature_digests(report, args.creature_digest_out)
        summary = analyse(report, args.bone_inventory, args.root)
        args.out.write_text(json.dumps(report, indent=2), encoding='utf-8')
        print(json.dumps(summary, indent=2))
        return
    indexes = []
    skipped = []
    resources = defaultdict(list)
    for package in sorted(args.root.rglob('*.package')):
        try:
            entries, _ = parse_index(package)
            indexes.append((package, entries))
            for entry in entries:
                if entry['type'] in {PROP, CREATURE} | MODELS:
                    resources[(entry['type'], entry['instance'])].append((package, entry))
        except Exception as exc:
            skipped.append({'package': str(package), 'error': repr(exc)})
    report = {'scope': {'wide_index_packages': len(indexes), 'skipped_packages': skipped},
              'counts': Counter(), 'failures': []}
    names = Counter()
    selectors = defaultdict(Counter)
    selector_values = [Counter() for _ in range(19)]
    clips = []
    capability_counts = Counter()
    morph_ids = Counter()
    component_tags = Counter()
    prop_rows = defaultdict(Counter)
    rig_fields = [Counter() for _ in range(6)]
    creatures = []
    for package, entries in indexes:
        local = Counter()
        for entry in entries:
            if entry['type'] not in (ANIM, CREATURE):
                continue
            report['counts']['anim' if entry['type'] == ANIM else 'creatures'] += 1
            try:
                data = payload(package, entry)
                if entry['type'] == ANIM:
                    inspect_anim(data, package, entry, names, selector_values, selectors, clips, component_tags)
                    local['anim'] += 1
                else:
                    rows, caps, morphs, parts, digest = inspect_creature(data)
                    morph_ids.update(f'{value:08X}' for value in morphs)
                    local['creatures'] += 1
                    report['counts']['rig_rows'] += len(rows)
                    capability_counts.update(caps)
                    for row in rows:
                        fields = struct.unpack_from('<6H', row)
                        for i, value in enumerate(fields):
                            rig_fields[i][value] += 1
                        instance = (u32(row, 132) << 32) | u32(row, 136)
                        prop_rows[instance][fields] += 1
                    creatures.append({'package': package.name, 'key': resource_id(entry),
                                      'capabilities': caps, 'parts': parts, 'rig_digest': digest})
            except Exception as exc:
                report['failures'].append({'package': package.name, 'key': resource_id(entry), 'error': repr(exc)})
        if local:
            print(package.name, dict(local), flush=True)
    report['counts']['channels'] = sum(names.values())
    report['channel_names'] = names
    report['selector_values'] = [{f'{value:08X}': {'count': count, 'fourcc': fourcc(value)}
                                  for value, count in sorted(counter.items())} for counter in selector_values]
    report['selectors_by_name'] = {name: [{'words': list(words), 'count': count} for words, count in variants.most_common()]
                                   for name, variants in selectors.items()}
    report['creature_capabilities'] = capability_counts
    report['creature_morph_ids'] = morph_ids
    report['component_tags'] = component_tags
    report['rig_u16_fields'] = rig_fields
    report['creatures'] = creatures
    report['locomotion_candidates'] = sorted(clips, key=lambda x: (x['name'].lower(), x['package'], x['key']))
    part_reports = []
    for instance, rows in sorted(prop_rows.items()):
        candidates = resources.get((PROP, instance), [])
        result = {'instance': f'{instance:016X}', 'rig_rows': sum(rows.values()),
                  'rig_field_variants': [{'fields': list(fields), 'count': count} for fields, count in rows.items()],
                  'props': []}
        for package, entry in candidates:
            try:
                records = properties(payload(package, entry))
                models = []
                matched_caps = []
                for ident, typ, spec, value in records:
                    keys = key_values(typ, spec, value)
                    if ident in MODEL_SLOTS:
                        for model, kind in keys:
                            models.append({'slot': f'{ident:08X}', 'type': f'{kind:08X}', 'instance': f'{model:016X}',
                                           'present': bool(resources.get((kind, model)))})
                    for cap in capability_counts:
                        raw = cap.encode('ascii', 'replace')
                        if raw and len(raw) >= 3 and raw in value:
                            matched_caps.append({'property': f'{ident:08X}' if ident is not None else None,
                                                 'type': typ, 'capability_bytes': cap})
                result['props'].append({'package': package.name, 'key': resource_id(entry),
                                        'models': models, 'capability_byte_occurrences': matched_caps,
                                        'property_ids': [f'{ident:08X}' for ident, _, _, _ in records if ident is not None]})
            except Exception as exc:
                result['props'].append({'package': package.name, 'key': resource_id(entry), 'error': repr(exc)})
        part_reports.append(result)
    report['part_props'] = part_reports
    report['hash_candidates'] = {name: {'fnv1a': f'{fnv32(name):08X}', 'fnv1': f'{fnv32(name, True):08X}'} for name in names}
    write_creature_digests(report, args.creature_digest_out)
    analyse(report, args.bone_inventory, args.root)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps({'wide_index_packages': len(indexes), 'skipped_package_count': len(skipped), 'counts': report['counts'],
                      'capabilities': capability_counts, 'channel_names': len(names),
                      'locomotion_candidates': len(clips), 'referenced_part_props': len(part_reports),
                      'joins': report['join_summary'],
                      'failures': report['failures'][:10]}, indent=2))
    if report['failures']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
