"""Read-only full-source custom-curve census and streamed fixture preparation.

Canonical keys are (low,type,scope). Only ignored target/ may receive output.
This tool never compiles or starts any original/native fixture executable.
"""
import argparse
from collections import Counter
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path
import random
import struct

from native_archive_index_probe import decode
from cell_schema_probe import SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
CONTEXTS = [0x7F | pc | rc for pc, rc in itertools.product([0, 0x200, 0x300], [0, 0x400, 0x800, 0xC00])]
FLAGS = [0x83, 0x93, 0xA3, 0xB3]
POLICIES = [(flags, slot) for flags in FLAGS for slot in range(3, 9)]
INPUT_BYTES = 9632
OUTPUT_BYTES = 476


def sha_file(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1 << 20), b''):
            result.update(block)
    return result.hexdigest()


def bits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def number(value):
    return struct.unpack('<f', struct.pack('<I', value))[0]


def f32(value):
    return number(bits(value))


def word(raw, at):
    return struct.unpack_from('<I', raw, at)[0]


def finite(value):
    return value & 0x7F800000 != 0x7F800000


def next_float(value, upward):
    raw = bits(value)
    if raw & 0x7FFFFFFF == 0:
        return number(1 if upward else 0x80000001)
    return number(raw+(1 if (value > 0) == upward else -1))


def unsigned_key(value, cw):
    """Exact FILD, then conditional FADD2^32 rounded by PC/RC.

    FILD itself is exact, even under PC24. Only the high unsigned half executes
    the FADD, whose positive result may round when PC24 is selected.
    """
    if value < 0x80000000 or cw & 0x300 != 0:
        return value
    shift = max(value.bit_length()-24, 0)
    if not shift:
        return value
    whole, remainder = divmod(value, 1 << shift)
    rounding = cw & 0xC00
    if rounding == 0:
        half = 1 << (shift-1)
        whole += int(remainder > half or remainder == half and whole & 1)
    elif rounding == 0x800:
        whole += int(bool(remainder))
    return whole << shift


def upper_index(frames, frame, cw):
    for index, key in enumerate(frames):
        if frame < unsigned_key(key, cw):
            return index
    return len(frames) if frames else -1


def neighbors(count, upper, wrap):
    if count == 0:
        return [0xFFFFFFFF]*4
    right = min(upper, count-1)
    left = right if upper == count else max(upper-1, 0)
    previous = left-1 if left else ((count-2) % count if wrap else left)
    following = right+1 if right < count-1 else ((right+2) % count if wrap else right)
    return [previous, left, right, following]


def sample_schedule(frames):
    counts = Counter()
    counts['duplicate_adjacent_spans'] = sum(a == b for a, b in zip(frames, frames[1:]))
    counts['nonmonotonic_adjacent_spans'] = sum(a > b for a, b in zip(frames, frames[1:]))
    if not frames:
        return [], [], counts
    before = f32(frames[0])
    if before >= frames[0]:
        before = next_float(before, False)
    after = f32(frames[-1])
    if after <= frames[-1]:
        after = next_float(after, True)
    proposed = [(before, 'before_first'), (after, 'after_last')]
    for key, label in [(frames[0], 'exact_first'), (frames[-1], 'exact_last')]:
        if f32(key) == key:
            proposed.append((f32(key), label))
        else:
            counts['unrepresentable_exact_endpoint_times'] += 1
    gaps = []
    for left, right in zip(frames, frames[1:]):
        if right <= left:
            continue
        counts['distinct_positive_spans'] += 1
        midpoint = f32((left+right)/2)
        kind = 'segment_midpoint'
        if not left < midpoint < right:
            counts['no_representable_strict_midpoint'] += 1
            midpoint = f32(left)
            kind = 'segment_left_fallback'
            if midpoint < left:
                midpoint = next_float(midpoint, True)
        if not left <= midpoint < right:
            counts['no_representable_frame_in_half_open_span'] += 1
            gaps.append([left, right, 'no_representable_binary32_in_half_open_span'])
        else:
            proposed.append((midpoint, kind))
    seen, samples = set(), []
    for frame, kind in proposed:
        raw = bits(frame)
        if raw in seen:
            counts['duplicate_proposed_frames_removed'] += 1
            continue
        seen.add(raw)
        samples.append(dict(frame_bits=f'{raw:08X}', kind=kind))
    return samples, gaps, counts


def package_index(path):
    with path.open('rb') as stream:
        header = stream.read(96)
        assert header[:4] == b'DBPF'
        count, size, at = (word(header, i) for i in [0x24, 0x2C, 0x40])
        stream.seek(at)
        index = stream.read(size)
    flags, entries = decode(index, count)
    mapping = {}
    for entry in entries:
        key = tuple(entry[:3])
        assert key not in mapping
        mapping[key] = entry
    return mapping, dict(header_sha256=hashlib.sha256(header).hexdigest(),
        index_sha256=hashlib.sha256(index).hexdigest(), flags=f'{flags:08X}', records=count)


def source_payload(path, entry):
    import refpack_bulk
    refpack_bulk.install()
    import refpack
    _, _, _, at, stored, decoded, compression, *_ = entry
    with path.open('rb') as stream:
        stream.seek(at)
        raw = stream.read(stored)
    assert len(raw) == stored
    if compression == 0xFFFF:
        raw, _, _ = refpack.decompress(raw, out_size=None, header=True)
    else:
        assert compression == 0
    assert len(raw) == decoded
    return raw


def census(args):
    args.out.mkdir(parents=True, exist_ok=False)
    rows_report = json.loads((args.source_rows/'report.json').read_text())
    for name, expected in rows_report['files'].items():
        assert sha_file(args.source_rows/name) == expected
    stats, kinds, packages, row_counts = Counter(), Counter(), {}, Counter()
    nonfinite, resources = Counter(), set()
    current_package, current_index, current_key, raw = None, None, None, None
    with (args.source_rows/'components.tsv').open(newline='') as components, \
            (args.source_rows/'source-INFO20-custom40-rows.bin').open('rb') as source_rows, \
            (args.out/'components.jsonl').open('x', newline='\n') as output:
        for component_ordinal, item in enumerate(csv.DictReader(components, delimiter='\t')):
            relative = item['package']
            if relative != current_package:
                current_package, current_key = relative, None
                current_index, metadata = package_index(args.install/relative)
                packages[relative] = metadata
            key = tuple(int(item[k], 16) for k in ['low', 'type', 'scope'])
            if key != current_key:
                raw = source_payload(args.install/relative, current_index[key])
                current_key = key
                assert hashlib.sha256(raw).hexdigest() == item['source_sha256']
            assert raw[:4] == b'ANIM' and word(raw, 4) == len(raw)
            duration, header_flags = word(raw, 288), word(raw, 280)
            assert finite(duration)
            channels, table = struct.unpack_from('<2I', raw, 324)
            channel, component = int(item['channel']), int(item['component'])
            assert channel < channels
            start = word(raw, table+4*channel)
            assert raw[start:start+4] == b'CHAN'
            count, keys, descriptors, descriptor_at = struct.unpack_from('<4I', raw, start+212)
            assert component < descriptors and count == int(item['rows']) and count <= 160
            descriptor = struct.unpack_from('<8I', raw, descriptor_at+32*component)
            info = next(struct.unpack_from('<8I', raw, descriptor_at+32*i) for i in range(descriptors)
                        if word(raw, descriptor_at+32*i) & 15 == 0 and word(raw, descriptor_at+32*i+4))
            flags, slot = int(item['prepared_flags'], 16), int(item['prepared_slot'], 16)
            assert flags in [0x93, 0xB3] and 3 <= slot <= 8
            assert descriptor[0] & 15 == 3 and descriptor[1] == int(item['tag'], 16)
            assert descriptor[2] == int(item['custom_offset']) and descriptor[3] == info[3] == int(item['source_stride'])
            source_rows.seek(int(item['rows_byte_offset']))
            block = source_rows.read(count*60)
            assert len(block) == count*60
            for index in range(count):
                at = keys+index*descriptor[3]
                expected = raw[at+info[2]:at+info[2]+20]+raw[at+descriptor[2]:at+descriptor[2]+40]
                assert block[index*60:(index+1)*60] == expected
                for label, offset in [('value', 0), ('weight', 4), ('value_prev_slope', 8),
                                      ('value_next_slope', 12), ('weight_prev_slope', 24), ('weight_next_slope', 28)]:
                    nonfinite[label] += int(not finite(word(block, index*60+20+offset)))
            frames = [word(block, 60*i) for i in range(count)]
            samples, gaps, counts = sample_schedule(frames)
            stats.update(counts)
            stats['components'] += 1
            stats['source_rows'] += count
            stats['source_cases'] += len(samples)
            stats['components_above16rows'] += count > 16
            stats['source_rows_in_components_above16rows'] += count if count > 16 else 0
            row_counts[str(count)] += 1
            resources.add((relative, key))
            for sample in samples:
                kinds[sample['kind']] += 1
            metadata = dict(component_ordinal=component_ordinal, **item,
                source_duration_bits=f'{duration:08X}', source_header_flags=f'{header_flags:08X}',
                source_flags=f'{descriptor[0]:08X}', source_slot=f'{descriptor[4]:08X}',
                frames=frames, samples=samples, uncovered_spans=gaps)
            output.write(json.dumps(metadata, separators=(',', ':'))+'\n')
    assert stats['components'] == 2086 and stats['source_rows'] == 44127 and len(resources) == 818
    assert stats['nonmonotonic_adjacent_spans'] == 0
    report = dict(schema='full-source-cell-runtime-curves-census-v1', native_executed=False,
        source_sha256=SOURCE_SHA256, statistics=dict(stats), source_resources=len(resources),
        packages=packages, row_count_histogram=dict(row_counts), sample_kinds=dict(kinds),
        nonfinite_source_fields=dict(nonfinite), explicit_source_context='CW027F/MXCSR1F80',
        full_original_channel_rows=True, source_row_capacity=160,
        source_input_bytes=stats['source_cases']*INPUT_BYTES, source_output_bytes=stats['source_cases']*OUTPUT_BYTES,
        source_rows_report_sha256=sha_file(args.source_rows/'report.json'), source_rows_files=rows_report['files'],
        census_tool_sha256=sha_file(Path(__file__)), components_sha256=sha_file(args.out/'components.jsonl'),
        scope='All2086 source data1..4 descriptors/818 ANIM resources retain every original INFO20/custom40 row in source order. Frame schedule includes before/after/exact endpoints and a representable sample for every positive adjacent distinct-time span. The native selector will see full original row counts and no supplied upper index. Active playback, pose selector binding and blend are separate.')
    (args.out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--install', type=Path, default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--source-rows', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    for path in [args.source_rows, args.out]:
        path.resolve().relative_to((ROOT/'target').resolve())
    census(args)


if __name__ == '__main__':
    main()
