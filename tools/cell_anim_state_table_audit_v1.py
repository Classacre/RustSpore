"""Extract the source-evidenced Cell request table to a new ignored target directory.

Reads a PE file without modifying it. Checks source hash, PE mapping, section
file bounds and table extent. The output contains exact bytes as hex and an
index; it is an audit artifact, not game data to distribute.
"""
import argparse
import hashlib
import json
import struct
from collections import defaultdict
from pathlib import Path

SOURCE_SHA256 = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
VA = 0x01484A20
RECORD_SIZE = 36
COUNT = 131
SIZE = RECORD_SIZE * COUNT


def source_offset(data):
    def u16(offset):
        return struct.unpack_from('<H', data, offset)[0]
    def u32(offset):
        return struct.unpack_from('<I', data, offset)[0]
    if data[:2] != b'MZ':
        raise ValueError('missing DOS signature')
    pe = u32(0x3c)
    if data[pe:pe + 4] != b'PE\0\0' or u16(pe + 24) != 0x10b:
        raise ValueError('expected PE32 image')
    image_base = u32(pe + 24 + 28)
    rva = VA - image_base
    sections = pe + 24 + u16(pe + 20)
    mappings = []
    for i in range(u16(pe + 6)):
        entry = sections + 40 * i
        vaddr, raw_size, raw_ptr = struct.unpack_from('<III', data, entry + 12)
        if vaddr <= rva and rva + SIZE <= vaddr + raw_size:
            offset = raw_ptr + rva - vaddr
            if offset + SIZE <= len(data):
                mappings.append(offset)
    if len(mappings) != 1:
        raise ValueError('table must have exactly one file-backed section mapping')
    return mappings[0], image_base


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('executable', type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    args.out.resolve().relative_to(target)
    if args.out.exists():
        raise FileExistsError(args.out)
    data = args.executable.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != SOURCE_SHA256:
        raise ValueError('source executable SHA-256 mismatch')
    offset, image_base = source_offset(data)
    table = data[offset:offset + SIZE]
    records = []
    by_state = defaultdict(list)
    by_key = defaultdict(list)
    for i in range(COUNT):
        raw = table[i * RECORD_SIZE:(i + 1) * RECORD_SIZE]
        words = struct.unpack('<9I', raw)
        state, key = (f'{x:08X}' for x in words[:2])
        by_state[state].append(i)
        by_key[key].append(i)
        records.append({'ordinal': i, 'va': f'{VA + i * RECORD_SIZE:08X}',
                        'file_offset': offset + i * RECORD_SIZE, 'bytes_hex': raw.hex(),
                        'words_le_hex': [f'{x:08X}' for x in words],
                        'state_word': state, 'request_key': key})
    report = {'source_sha256': digest, 'image_base': f'{image_base:08X}',
              'start_va': f'{VA:08X}', 'end_va_exclusive': f'{VA + SIZE:08X}',
              'initial_loop_pointer': '01484A2C', 'terminal_loop_pointer': '01485C98',
              'file_offset': offset, 'byte_count': SIZE, 'record_size': RECORD_SIZE,
              'record_count': COUNT, 'table_sha256': hashlib.sha256(table).hexdigest(),
              'scope': 'Static table projection only. State is compared to argument 3; key is returned on selection. Actor-dependent predicates, fallback and selection are not modeled.',
              'by_state_word': dict(sorted(by_state.items())), 'by_request_key': dict(sorted(by_key.items())),
              'records': records}
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / 'table.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: report[k] for k in ('source_sha256', 'start_va', 'end_va_exclusive', 'record_count', 'table_sha256')}))


if __name__ == '__main__':
    main()
