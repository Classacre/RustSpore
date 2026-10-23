"""Fresh installed ANIM ROT100 framing and source-input census.

Reads all measured package index shapes, never executes native code, and keeps
all proprietary row bytes/fixtures under a new ignored target/ directory.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import struct

from dbpf_extract import payload
from prop_numeric_probe import index as package_index

ROOT = Path(__file__).resolve().parent.parent
TYPE = 0xEE17C6AD


def census(install, output):
    output.resolve().relative_to((ROOT/'target').resolve())
    output.mkdir(parents=True, exist_ok=False)
    counts, descriptors, controls, weight_controls = Counter(), Counter(), Counter(), Counter()
    packages, records, problems, digests = {}, [], [], []
    quaternion_maximum = weight_maximum = 0.0
    source_samples = []
    for package in sorted(install.rglob('*.package')):
        entries = package_index(package)
        counts['package_indexes'] += 1
        found = 0
        for entry in entries:
            if entry['type'] != TYPE:
                continue
            found += 1
            counts['resources'] += 1
            key = f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            try:
                data = payload(package, entry)
                word = lambda at: struct.unpack_from('<I', data, at)[0]
                assert data[:4] == b'ANIM' and word(4) == len(data)
                channel_count, table = word(324), word(328)
                assert table+4*channel_count <= len(data)
                feed = hashlib.sha256()
                feed.update(struct.pack('<3I', word(280), word(288), channel_count))
                resource_rotations = resource_channels = 0
                for channel in range(channel_count):
                    at = word(table+4*channel)
                    assert data[at:at+4] == b'CHAN'
                    count, rows, component_count, table_at = struct.unpack_from('<4I', data, at+212)
                    assert table_at+32*component_count <= len(data)
                    counts['channels'] += 1
                    rotational = []
                    info = None
                    stride = None
                    for component in range(component_count):
                        descriptor_at = table_at+32*component
                        raw = data[descriptor_at:descriptor_at+32]
                        flags, tag, offset, step, slot, *reserved = struct.unpack('<I4s6I', raw)
                        assert stride is None or stride == step
                        stride = step
                        if tag == b'INFO':
                            assert info is None
                            info = offset
                        if tag == b'ROT\0':
                            descriptors[f'{flags:08X}/{slot:08X}/{offset}/{step}'] += 1
                            assert offset+100 <= step
                            rotational.append((offset, raw, flags))
                    if not rotational:
                        continue
                    assert info is not None and stride and rows+count*stride <= len(data)
                    counts['rotation_channels'] += 1
                    counts['empty_rotation_channels'] += count == 0
                    counts['rotation_descriptors'] += len(rotational)
                    resource_channels += 1
                    feed.update(struct.pack('<4I', channel, count, stride, len(rotational)))
                    frames = [word(rows+i*stride+info) for i in range(count)]
                    assert all(a <= b for a, b in zip(frames, frames[1:]))
                    for offset, raw_descriptor, flags in rotational:
                        feed.update(raw_descriptor)
                        counts['enabled_rotation_channels'] += bool(flags & 0x10)
                        for row, frame in enumerate(frames):
                            raw = data[rows+row*stride+offset:rows+row*stride+offset+100]
                            assert len(raw) == 100
                            counts['rotation_rows'] += 1
                            resource_rotations += 1
                            feed.update(struct.pack('<I', frame))
                            feed.update(raw)
                            quat = struct.unpack_from('<4f', raw)
                            weight, previous_slope, next_slope = [struct.unpack_from('<f', raw, x)[0]
                                                                  for x in [16, 84, 88]]
                            controls[f'{raw[28]:02X}/{raw[29]:02X}'] += 1
                            weight_controls[f'{raw[92]:02X}/{raw[93]:02X}'] += 1
                            counts['finite_quaternions'] += all(math.isfinite(v) for v in quat)
                            counts['zero_quaternions'] += all(v == 0 for v in quat)
                            counts['finite_weights'] += math.isfinite(weight)
                            counts['weights_outside_unit_interval'] += math.isfinite(weight) and not 0 <= weight <= 1
                            counts['finite_weight_slopes'] += math.isfinite(previous_slope) and math.isfinite(next_slope)
                            counts['quaternion_norm_outside_001'] += abs(sum(v*v for v in quat)-1) >= .01
                            if all(math.isfinite(v) for v in quat):
                                quaternion_maximum = max(quaternion_maximum, *(abs(v) for v in quat))
                            if math.isfinite(weight):
                                weight_maximum = max(weight_maximum, abs(weight))
                        # Retain small deterministic real source channels for
                        # host review. Later native source tests can select
                        # windows of four rows without fabricating controls.
                        if count and len(source_samples) < 64:
                            source_samples.append(dict(package=str(package), key=key,
                                channel=channel, count=count, stride=stride, offset=offset,
                                flags=f'{flags:08X}', descriptor=raw_descriptor.hex(),
                                duration_bits=f'{word(288):08X}', header_flags=f'{word(280):08X}',
                                frames=frames[:16], row_bytes=[data[rows+i*stride+offset:rows+i*stride+offset+100].hex()
                                                            for i in range(min(count, 16))]))
                digests.append(f'{package.name} {key} {feed.hexdigest()}')
                records.append(dict(package=package.name, key=key, rotation_channels=resource_channels,
                                    rotation_rows=resource_rotations, sha256=feed.hexdigest()))
            except Exception as error:
                problems.append(dict(package=str(package), key=key, error=repr(error)))
        packages[str(package.relative_to(install))] = found
        if found:
            print(json.dumps(dict(package=package.name, resources=found,
                                  total_rows=counts['rotation_rows'], problems=len(problems))), flush=True)
    digest_bytes = ('\n'.join(sorted(digests))+'\n').encode()
    (output/'source-digests.txt').write_bytes(digest_bytes)
    report = dict(schema='fresh-anim-rotation100-census-v1', install=str(install.resolve()),
        counts=dict(counts), package_resources=packages, descriptors=dict(descriptors),
        quaternion_mode_bytes=dict(controls), weight_mode_bytes=dict(weight_controls),
        maximum_absolute_quaternion_component=quaternion_maximum, maximum_absolute_weight=weight_maximum,
        source_digest_lines=len(digests), source_digest_sha256=hashlib.sha256(digest_bytes).hexdigest(),
        records=records, problems=problems,
        scope='Fresh source framing/input facts only. No source quaternion normalization, filtering, native curve output or retail playback claim.')
    (output/'census.json').write_text(json.dumps(report, indent=2)+'\n')
    (output/'source-samples.json').write_text(json.dumps(source_samples, indent=2)+'\n')
    print(json.dumps({k: v for k, v in report.items() if k not in {'records', 'package_resources', 'descriptors'}}, indent=2))
    if problems:
        raise AssertionError(problems[:5])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    census(args.install, args.out)


if __name__ == '__main__':
    main()
