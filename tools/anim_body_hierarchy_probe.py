"""Fresh canonical constructor topology/capability input census; no game execution.

Uses the native flag-driven archive index and full128-byte CreatureData header.
This records the exact9CBFA0 initial permutation/parent/capability inputs; it
does not substitute source rows for finalized runtime blocks or run finalization.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct

from creature_header_probe import header, inspect
from dbpf_extract import payload
from native_archive_index_probe import decode

MASKS = {b'root': 1, b'limb': 2, b'foot': 4, b'spin': 8,
         b'psft': 16, b'mout': 32, b'grsp': 64, b'nstr': 128}
ROOT = Path(__file__).resolve().parent.parent


def package_entries(path):
    with path.open('rb') as stream:
        head = stream.read(96)
        assert head[:4] == b'DBPF'
        count, length, at = [struct.unpack_from('<I', head, offset)[0]
                             for offset in (0x24, 0x2C, 0x40)]
        stream.seek(at)
        block = stream.read(length)
    assert len(block) == length
    flags, rows = decode(block, count)
    return [dict(instance=low, type=typ, group=scope, offset=offset,
                 stored=stored, dec=decoded, compression=compression)
            for low, typ, scope, offset, stored, decoded, compression, committed, metadata in rows]


def constructor_inputs(data):
    inspect(data, Counter())
    version = struct.unpack_from('<I', data, 4)[0]
    counts = struct.unpack_from('<'+('3I' if version == 9 else '4I'), data, 8)
    start, decoded, encoded, chunks = header(data, 8+4*len(counts))
    count, cap_total, morph_total = counts[:3]
    assert 0 < count <= 255
    rows = [data[start+i*140:start+(i+1)*140] for i in range(count)]
    cap_start = start+count*140
    caps = [data[cap_start+i*4:cap_start+(i+1)*4] for i in range(cap_total)]
    numbers = data[cap_start+cap_total*4:cap_start+cap_total*5]
    source_parents = [struct.unpack_from('<H', row, 4)[0] for row in rows]
    source_symmetric = [struct.unpack_from('<H', row, 6)[0] for row in rows]
    nominated = source_parents[0] if source_parents[0] != 0xFFFF else 0
    assert nominated < count and source_parents[nominated] == 0xFFFF
    source_to_initial = [i+1 if i < nominated else 0 if i == nominated else i
                         for i in range(count)]
    initial_to_source = sorted(range(count), key=source_to_initial.__getitem__)
    assert sorted(source_to_initial) == list(range(count))
    blocks = []
    cap_cursor = morph_cursor = 0
    for source, row in enumerate(rows):
        parent = source_parents[source]
        symmetric = source_symmetric[source]
        assert parent == 0xFFFF or parent < count and parent != source
        assert symmetric == 0xFFFF or symmetric < count and symmetric != source
        cap_at, morph_at, cap_count, morph_count = struct.unpack_from('<HHBB', row, 12)
        assert cap_at == cap_cursor and cap_at+cap_count <= cap_total
        assert morph_at == morph_cursor and morph_at+morph_count <= morph_total
        cap_cursor += cap_count
        morph_cursor += morph_count
        presence = nonzero = 0
        stopped = False
        for ident, number in zip(caps[cap_at:cap_cursor], numbers[cap_at:cap_cursor]):
            if ident == bytes(4):
                stopped = True
                break
            presence |= MASKS.get(ident, 0)
            if number:
                nonzero |= MASKS.get(ident, 0)
        blocks.append(dict(source_palette_id=source, initial_ordinal=source_to_initial[source],
            parent=None if parent == 0xFFFF else source_to_initial[parent],
            # Native9CC.. copies this word directly, unlike the parent. Keep
            # that distinction even when a particular source corpus is safe.
            symmetric_word_written=0xFFFFFFFF if symmetric == 0xFFFF else symmetric,
            model_group=None if row[10] == 0 else row[10], source_class=row[11],
            capability_count=cap_count, morph_count=morph_count,
            raw_pre_expansion_presence_mask=presence, raw_pre_expansion_nonzero_mask=nonzero,
            raw_capability_zero_before_end=stopped))
    assert cap_cursor == cap_total and morph_cursor == morph_total
    blocks.sort(key=lambda block: block['initial_ordinal'])
    assert blocks[0]['parent'] is None
    children = [[] for _ in blocks]
    for i, block in enumerate(blocks):
        if block['parent'] is not None:
            children[block['parent']].append(i)
        seen, at = set(), i
        while at is not None:
            assert at not in seen
            seen.add(at)
            at = blocks[at]['parent']
    for block, descendants in zip(blocks, children):
        block['initial_child_count'] = len(descendants)
    feet = sum(bool(block['raw_pre_expansion_nonzero_mask'] & 4) for block in blocks)
    roots = [i for i, block in enumerate(blocks) if block['raw_pre_expansion_presence_mask'] & 1]
    return dict(source_rows=count, source_capabilities=cap_total, source_morphs=morph_total,
        root_nomination=nominated, source_to_initial=source_to_initial,
        initial_to_source=initial_to_source, blocks=blocks,
        raw_pre_expansion_nonzero_feet=feet,
        raw_pre_expansion_nonzero_spines=sum(bool(block['raw_pre_expansion_nonzero_mask'] & 8) for block in blocks),
        raw_pre_expansion_initial_root_has_spin=bool(blocks[0]['raw_pre_expansion_nonzero_mask'] & 8),
        raw_pre_expansion_first_root_capability=roots[0] if roots else None,
        raw_pre_expansion_optional_feet_gate=feet == 0 and nominated == 0,
        nonzero_nomination_with_symmetry=nominated != 0 and any(x != 0xFFFF for x in source_symmetric))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    stats, limits = Counter(), Counter()
    records, problems = [], []
    for package in sorted(args.install.rglob('*.package')):
        try:
            entries = package_entries(package)
            stats['package_indexes'] += 1
            for entry in entries:
                if entry['type'] != 0x0F43029A:
                    continue
                assert entry['compression'] in (0, 0xFFFF)
                assert (entry['compression'] == 0xFFFF) == (entry['dec'] > entry['stored'])
                data = payload(package, entry)
                assert len(data) == entry['dec']
                record = constructor_inputs(data)
                record.update(package=package.relative_to(args.install).as_posix(),
                    native_key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}",
                    source_sha256=hashlib.sha256(data).hexdigest())
                records.append(record)
                stats['resources'] += 1
                for field in ['source_rows', 'source_capabilities', 'source_morphs']:
                    stats[field] += record[field]
                for field in ['raw_pre_expansion_initial_root_has_spin', 'raw_pre_expansion_optional_feet_gate',
                              'nonzero_nomination_with_symmetry']:
                    stats[field] += record[field]
                stats['nonzero_root_nomination'] += record['root_nomination'] != 0
                stats['raw_pre_expansion_with_root_capability'] += record['raw_pre_expansion_first_root_capability'] is not None
                stats['raw_pre_expansion_without_optional_feet_gate'] += not record['raw_pre_expansion_optional_feet_gate']
                for block in record['blocks']:
                    limits['max_capabilities_per_row'] = max(limits['max_capabilities_per_row'], block['capability_count'])
                    limits['max_morphs_per_row'] = max(limits['max_morphs_per_row'], block['morph_count'])
                    stats['rows_with_zero_capability_id'] += block['raw_capability_zero_before_end']
                    stats['rows_over_32_capability_slots'] += block['capability_count'] > 32
                    stats['rows_over_8_morph_slots'] += block['morph_count'] > 8
        except Exception as error:
            problems.append(dict(package=str(package), error=repr(error)))
    report = dict(schema='canonical-body-constructor-topology-input-census-v1',
        statistics=dict(stats), limits=dict(limits), problems=problems, records=records,
        native_constructor='9CBFA0: source palette IDs, first-parent root nomination, initial runtime permutation, remapped parents, raw symmetry copy and raw capability inputs.',
        capability_caveat='All raw_pre_expansion fields are provisional input projections, not native finalizer masks/eligibility.9FC2F0 first expands aggregates through9FC210, deduplicates with maximum numbers, compacts holes from the tail, then9B2E20 rebuilds native masks. A selected dictionary context is required before deriving actual insertion/root policy inputs.',
        archive_keys='Canonical native type:scope:low, using native flag-driven index8DAFE0 semantics.',
        scope='Fresh source inputs and bounded initial permutation/parent semantics only. Raw pre-expansion masks must not establish finalizer eligibility or fake-foot insertion. No complete finalized hierarchy, root-policy selection, selector binding or evaluated runtime poses inferred.')
    with args.out.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({key: report[key] for key in ['schema', 'statistics', 'limits', 'problems', 'scope']}, indent=2))
    if problems or stats['resources'] != 1365 or stats['package_indexes'] != 82:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
