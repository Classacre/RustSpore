"""Fresh source-row/first-group constraints at explicit authored LOD0 keys.

This uses the independently verified direct PROP report to select model keys.
The native first-group clamp is tested in anim_palette.rs. This census does not
construct a finalized rig, infer palette ordinals or evaluate source animation.
"""
from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

from creature_header_probe import header, inspect as inspect_header
from dbpf_extract import payload
from gmdl_skin_probe import tail
from prop_numeric_probe import index


def key(entry):
    return f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"


def source_counts(data):
    inspect_header(data, Counter())
    version, count = struct.unpack_from('<2I', data, 4)
    at, _decoded, _encoded, _chunks = header(data, 20 if version == 9 else 24)
    classes = [data[at + ordinal * 140 + 11] for ordinal in range(count)]
    prefix = next((ordinal for ordinal, value in enumerate(classes) if value == 1), count)
    assert all(value == 1 for value in classes[prefix:]), 'class-one rows are not a contiguous tail'
    assert prefix == sum(value != 1 for value in classes)
    return count, prefix


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--links', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--digest-out', type=Path, required=True)
    args = parser.parse_args()
    for path in (args.out, args.digest_out):
        if 'target' not in path.parts:
            raise ValueError('source-derived outputs must remain under ignored target/')
    links_data = args.links.read_bytes()
    links = json.loads(links_data)
    assert links['rust_comparison']['exact_match'] and links['difference_count'] == 0
    packages = {}

    def resource(relative, wanted):
        path = args.install / relative
        if path not in packages:
            packages[path] = {key(entry): entry for entry in index(path)}
        entry = packages[path][wanted]
        return payload(path, entry)

    counts, records, digests, problems = Counter(), [], [], []
    for source in links['associations']:
        if not source['lod0']:
            counts['source_creatures_without_local_prop'] += 1
            continue
        counts['source_creatures_with_direct_lod0'] += 1
        source_relative = f"Data/{source['creature_package']}"
        # The retained source census currently names Data packages. Resolve an
        # exact basename uniquely for other layouts rather than guessing a path.
        if not (args.install / source_relative).is_file():
            matches = list(args.install.rglob(source['creature_package']))
            assert len(matches) == 1, 'source package basename is ambiguous'
            source_relative = matches[0].relative_to(args.install).as_posix()
        source_data = resource(source_relative, source['creature'])
        source_count, body_prefix = source_counts(source_data)
        assert source_count == source['source_rows'] and body_prefix == source['nonclass1_rows']
        models = {(available['package'], available['key'])
            for ref in source['lod0'] for available in ref['available']}
        assert models
        matched_all = True
        for package, model_key in sorted(models):
            model_data = resource(package, model_key)
            skin = tail(model_data)
            assert skin and skin['parts'], 'no source part for a first-group constraint'
            first = skin['parts'][0]
            assert first['model_type'] == 0xFFFFFFFF, 'external first group requires its actual skeleton'
            count = first['field'] & 255
            assert count < 128, 'native first-group count is a negative signed byte'
            bounded = min(source_count, count)
            matched = bounded == body_prefix
            matched_all &= matched
            counts['direct_lod0_models_checked'] += 1
            counts['matching_first_group_bounds'] += matched
            counts['serialized_rows'] += source_count
            counts['nonclass1_rows'] += body_prefix
            if not matched:
                problems.append({'creature': source['creature'], 'model': model_key,
                    'source_rows': source_count, 'prefix': body_prefix, 'first_group_count': count})
            record = {'creature': source['creature'], 'source_package': source_relative,
                'model': model_key, 'model_package': package, 'source_rows': source_count,
                'nonclass1_rows': body_prefix, 'source_parts': len(skin['parts']),
                'first_raw_count_word': first['field'], 'first_group_count': count,
                'bounded_serialized_count': bounded, 'matches': matched,
                'source_sha256': hashlib.sha256(source_data).hexdigest(),
                'model_sha256': hashlib.sha256(model_data).hexdigest()}
            records.append(record)
            digests.append('\t'.join(map(str, (source['creature'], package, model_key,
                source_count, body_prefix, first['field'], count, bounded))))
        counts['source_creatures_with_all_direct_bounds_matching'] += matched_all
    assert counts['source_creatures_with_direct_lod0'] == links['counts']['source_creatures_with_direct_lod0']
    normalized = '\n'.join(sorted(digests)) + '\n'
    report = {'scope': 'fresh source rows and direct authored first-group counts; serialized-count clamp constraint only',
        'direct_link_report_sha256': hashlib.sha256(links_data).hexdigest(), 'counts': counts,
        'normalized_digest_sha256': hashlib.sha256(normalized.encode()).hexdigest(),
        'problems': problems, 'records': records}
    args.digest_out.parent.mkdir(parents=True, exist_ok=True)
    args.digest_out.write_text(normalized, encoding='utf-8')
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({name: report[name] for name in ('scope', 'counts', 'normalized_digest_sha256', 'problems')}))
    if problems:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
