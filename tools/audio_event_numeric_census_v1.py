#!/usr/bin/env python3
"""Independent canonical-index census of audio-event numeric inspection words.

The serialized digest walks 02B9F662 records in native index order. Exact
inspection framing is required for every selected payload. A matching digest
is not proof of native loader selection, complete stream reads, provider priority,
or gameplay. Caller must keep the selected package paths and files stable
throughout; endpoint hashes do not establish continuous stability.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys

from native_archive_index_probe import decode
from refpack import decompress

EVENT_TYPE = 0x02B9F662
NUMERIC = {0x0009, 0x000A, 0x000D}
ARRAY = {0x009C, 0x001C}
STRINGS = {0x0012, 0x0013}
MAX_RESOURCE = 64 * 1024 * 1024
MAX_INDEX = 64 * 1024 * 1024


def file_sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def separator(data, offset):
    if offset + 4 > len(data):
        return False
    return (data[offset:offset+4] == b'\x01\x00\x00\x00'
            or data[offset+2:offset+4] in (b'\x12\x00', b'\xa0\x00'))


def numeric_fields(data):
    """Inspection walker, returning exactness and tuples (hash,type,flags,marker,count,bytes).

    Numeric words are returned as little-endian serialized raw u32 bit patterns;
    unsupported typed-array strides preserve the original framed bytes. The
    caller must reject any inexact payload and discard its partial fields.
    """
    if len(data) < 4:
        return False, []
    p, n, fields = 4, len(data), []
    while p < n:
        if separator(data, p):
            p += 4
            continue
        if p + 8 > n:
            return False, fields
        name, kind, flags = struct.unpack_from('>IHH', data, p)
        if flags in ARRAY:
            if p + 16 > n:
                return False, fields
            count, stride = struct.unpack_from('>II', data, p + 8)
            if count > 1_000_000 or stride > 1_000_000:
                return False, fields
            at = p + 16
            if kind in STRINGS:
                width = 1 if kind == 0x12 else 2
                for _ in range(count):
                    if at + 4 > n:
                        return False, fields
                    length, = struct.unpack_from('>I', data, at)
                    if length > 100_000:
                        return False, fields
                    at += 4 + length * width
                    if at > n:
                        return False, fields
                p = at
                continue
            size = count * stride
            end = at + size
            if end > n:
                return False, fields
            if kind in NUMERIC:
                raw = data[at:end]
                if stride == 4:
                    words = bytearray()
                    for offset in range(0, len(raw), 4):
                        words.extend(struct.pack('<I', int.from_bytes(raw[offset:offset+4], 'big')))
                    fields.append((name, kind, flags, 1, count, bytes(words)))
                else:
                    if size > 0xffffffff:
                        raise ValueError('unsupported numeric payload exceeds u32 digest length')
                    fields.append((name, kind, flags, 2, size, raw))
            p = end
        elif kind in STRINGS:
            if p + 12 > n:
                return False, fields
            length, = struct.unpack_from('>I', data, p + 8)
            width = 1 if kind == 0x12 else 2
            if length > 100_000 or p + 12 + length * width > n:
                return False, fields
            p += 12 + length * width
        elif kind == 1:
            if p + 9 > n:
                return False, fields
            p += 9
        else:
            if p + 12 > n:
                return False, fields
            if kind in NUMERIC:
                word = int.from_bytes(data[p+8:p+12], 'big')
                fields.append((name, kind, flags, 0, 1, struct.pack('<I', word)))
            p += 12
    return True, fields


def native_index(f, file_size):
    f.seek(0)
    header = f.read(96)
    if len(header) != 96 or header[:4] != b'DBPF':
        raise ValueError('invalid DBPF header')
    count, size, offset = (struct.unpack_from('<I', header, x)[0] for x in (0x24, 0x2c, 0x40))
    if size > MAX_INDEX or offset + size > file_size:
        raise ValueError('index outside file or exceeds 64 MiB bound')
    f.seek(offset)
    block = f.read(size)
    if len(block) != size:
        raise ValueError('truncated index')
    flags, rows = decode(block, count)
    return flags, rows


def read_payload(f, row, file_size):
    _, _, _, offset, stored, decoded, compression, _, _ = row
    if stored > MAX_RESOURCE or decoded > MAX_RESOURCE or offset + stored > file_size:
        raise ValueError('selected payload outside file or exceeds 64 MiB bound')
    f.seek(offset)
    raw = f.read(stored)
    if len(raw) != stored:
        raise ValueError('truncated selected payload')
    if compression == 0xffff:
        raw, _, info = decompress(raw, out_size=decoded, header=True)
        if info['declared_decompressed'] != decoded:
            raise ValueError('RefPack header and native index decoded sizes disagree')
    elif compression != 0:
        raise ValueError(f'unsupported compression {compression:04X}')
    if len(raw) != decoded:
        raise ValueError('decoded size mismatch')
    return raw


def census(role, path):
    before = file_sha(path)
    digest = hashlib.sha256()
    stats = {'events': 0, 'events_exact': 0, 'events_inexact': 0,
             'numeric_single_fields': 0, 'numeric_supported_array_fields': 0,
             'numeric_raw_unsupported_fields': 0, 'numeric_supported_array_values': 0,
             'numeric_raw_unsupported_bytes': 0}
    with path.open('rb') as f:
        f.seek(0, 2)
        flags, rows = native_index(f, f.tell())
        file_size = f.seek(0, 2)
        for row in rows:
            instance, type_id, scope = row[:3]
            if type_id != EVENT_TYPE:
                continue
            data = read_payload(f, row, file_size)
            exact, fields = numeric_fields(data)
            stats['events'] += 1
            if not exact:
                stats['events_inexact'] += 1
                raise ValueError(f'inexact inspection walk at native key {instance:08X}:{type_id:08X}:{scope:08X}')
            stats['events_exact'] += 1
            digest.update(struct.pack('<III', instance, type_id, scope))
            for name, kind, field_flags, marker, count, body in fields:
                digest.update(struct.pack('<IHHBI', name, kind, field_flags, marker, count))
                digest.update(body)
                if marker == 0:
                    stats['numeric_single_fields'] += 1
                elif marker == 1:
                    stats['numeric_supported_array_fields'] += 1
                    stats['numeric_supported_array_values'] += count
                else:
                    stats['numeric_raw_unsupported_fields'] += 1
                    stats['numeric_raw_unsupported_bytes'] += count
    after = file_sha(path)
    if after != before:
        raise ValueError(f'package endpoint hash changed: {path}')
    return {'role': role, 'path': str(path), 'sha256_before': before, 'sha256_after': after,
            'index_flags': f'{flags:08X}', **stats, 'numeric_stream_sha256': digest.hexdigest()}


def main():
    if not __debug__:
        raise RuntimeError('canonical index decoder requires assertions enabled')
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--patchdata', required=True, type=Path)
    ap.add_argument('--audio2', required=True, type=Path)
    args = ap.parse_args()
    paths = {'patchdata': args.patchdata.resolve(strict=True), 'audio2': args.audio2.resolve(strict=True)}
    if paths['patchdata'] == paths['audio2']:
        raise ValueError('choose two distinct package paths')
    report = {'schema': 'audio-event-numeric-census-v1',
              'order': 'native package index order; each selected key then numeric fields in inspection order',
              'digest_encoding': 'key instance/type/scope LE u32; field hash LE u32, type/flags LE u16, marker u8, count LE u32, numeric bits LE u32 or raw unsupported bytes',
              'scope': 'Exact Python inspection framing only; provider, native factory/loader selection and complete stream reads are not proven. Other numeric strides use count*stride inspection framing, not established native typed-stream consumption.',
              'stability': 'Caller must keep package paths and files stable throughout. Matching endpoint hashes do not prove continuous stability or prevent transient replacement.',
              'packages': {role: census(role, path) for role, path in paths.items()}}
    print(json.dumps(report, separators=(',', ':')))


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        print(f'error: {error}', file=sys.stderr)
        sys.exit(1)
