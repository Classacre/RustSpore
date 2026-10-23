"""Fresh canonical optional-foot call/candidate census; no native execution.

Matches fresh source payloads to the already accepted native capability
projection. Reports the literal9C8560 structural admission separately from
the9CBFA0 call gate; it does not assume geometric progress or append state.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import sys

from anim_body_hierarchy_probe import constructor_inputs, package_entries
from cell_schema_probe import SourceImage, SOURCE_SHA256
from dbpf_extract import payload

CAPABILITIES_SHA = '582aca8c6e189c9d8385749d68798bb03efdd9bb8da0446f7ac3c01032c6b337'
REGIONS = [
    (0x9C84B0, 0x9C855D, 'spine_minimum'),
    (0x9C8560, 0x9C8657, 'leaf_limb_minimum'),
    (0x9CBE70, 0x9CBF97, 'optional_foot_driver'),
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--source-exe', type=Path, required=True)
    parser.add_argument('--capability-inputs', type=Path, required=True)
    parser.add_argument('--capability-run', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    args.out.resolve().relative_to((root/'target').resolve())
    assert not args.out.exists()
    comparison_path = args.capability_run/'comparison-v1.json'
    comparison = json.loads(comparison_path.read_text())
    assert comparison['exact']
    projection_path = args.capability_run/'native-projection.bin'
    projection = projection_path.read_bytes()
    assert hashlib.sha256(projection).hexdigest() == CAPABILITIES_SHA
    resources_path = args.capability_inputs/'source-resources.json'
    retained = json.loads(resources_path.read_text())
    by_key = {(r['package'], r['key']): r for r in retained}
    assert len(by_key) == 1365
    image = SourceImage(args.source_exe)
    sys.path.insert(0, str(root/'target/rea/capstone-runtime'))
    from native_crt_acos_probe import decoder
    regions = []
    for start, end, name in REGIONS:
        raw = image.read(start, end-start)
        instructions = list(decoder().disasm(raw, start))
        assert sum(i.size for i in instructions) == len(raw)
        regions.append(dict(name=name, start=f'{start:08X}', end_exclusive=f'{end:08X}',
            bytes=len(raw), instructions=len(instructions), sha256=hashlib.sha256(raw).hexdigest()))
    counts, records = Counter(), []
    for package in sorted(args.install.rglob('*.package')):
        counts['package_indexes'] += 1
        for entry in package_entries(package):
            if entry['type'] != 0x0F43029A:
                continue
            raw = payload(package, entry)
            key = f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}"
            relative = package.relative_to(args.install).as_posix()
            known = by_key[(relative, key)]
            source_sha = hashlib.sha256(raw).hexdigest()
            assert source_sha == known['source_sha256']
            initial = constructor_inputs(raw)
            blocks = initial['blocks']
            for block in blocks:
                source = block['source_palette_id']
                presence, nonzero = struct.unpack_from('<2I', projection, (known['first_row']+source)*168)
                block.update(presence_mask=presence, nonzero_mask=nonzero)
            child_counts = [0]*len(blocks)
            for block in blocks:
                if block['parent'] is not None:
                    child_counts[block['parent']] += 1
            assert child_counts == [b['initial_child_count'] for b in blocks]
            candidates = [b['initial_ordinal'] for b, children in zip(blocks, child_counts)
                if children == 0 and b['presence_mask'] & 2 and not b['presence_mask'] & 4]
            feet = sum(bool(b['nonzero_mask'] & 4) for b in blocks)
            called = feet == 0 and initial['root_nomination'] == 0
            initial_spin = bool(blocks[0]['nonzero_mask'] & 8)
            record = dict(package=relative, key=key, source_sha256=source_sha, rows=len(blocks),
                root_nomination=initial['root_nomination'], nonzero_feet=feet,
                initial_root_has_nonzero_spin=initial_spin, optional_driver_called=called,
                leaf_limb_candidates=candidates)
            if called:
                record['candidate_blocks'] = [dict(initial_ordinal=i,
                    source_palette_id=blocks[i]['source_palette_id'], parent=blocks[i]['parent'],
                    symmetric_word_written=blocks[i]['symmetric_word_written'],
                    presence_mask=blocks[i]['presence_mask'], nonzero_mask=blocks[i]['nonzero_mask'],
                    child_count=child_counts[i]) for i in candidates]
                counts['driver_called_resources'] += 1
                counts['driver_called_source_rows'] += len(blocks)
                counts['called_leaf_limb_candidates'] += len(candidates)
                counts['called_candidates_with_mouth_or_grasp'] += sum(bool(blocks[i]['presence_mask'] & 0x60) for i in candidates)
                disposition = 'called_with_candidates' if candidates else 'called_without_candidates'
                counts[disposition+'_resources'] += 1
                counts[disposition+('_root_spin' if initial_spin else '_root_missing_spin')] += 1
            counts['resources'] += 1
            counts['source_rows'] += len(blocks)
            records.append(record)
    assert counts['package_indexes'] == 82 and counts['resources'] == 1365 and counts['source_rows'] == 46418
    report = dict(schema='canonical-optional-foot-structural-census-v1', statistics=dict(counts),
        source_sha256=SOURCE_SHA256, capability_projection_sha256=CAPABILITIES_SHA,
        capability_comparison_sha256=sha(comparison_path), capability_resources_sha256=sha(resources_path),
        source_regions=regions, records=records,
        native_admission='9C85A8 requires childcount208==0;9C85BB requires presence150&2;9C85C4 requires !(presence150&4).Canonical parent loop9CCA02 increments each remapped parent childcount once after9CC975 zeroes all208 fields.',
        native_empty_branch='9CBEA6 initializes selected ordinalFFFFFFFF;9C8560 has no ordinal store when no block passes its structural admission;9CBEC7 then branches directly to9CBF91 return, before point/append.',
        limits='Structural source correlation only. Nonempty candidates do not establish insertion; geometry/default local quaternion, threshold, point/append/progress and complete finalization remain separate. No accepted topology API gate changed and no original bytes executed.')
    with args.out.open('x') as stream:
        json.dump(report, stream, indent=2);stream.write('\n')
    print(json.dumps({k:report[k] for k in ['schema','statistics','source_regions','limits']}, indent=2))


if __name__ == '__main__':
    main()
