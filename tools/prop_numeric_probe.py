"""Independent named/count-bounded PROP framing and primitive numeric corpus.

Unlike the historical prop_v5 size-only proof, primitive singles are four bytes
and each property's identifier is retained. Anonymous recovery is deliberately
unsupported; any such file is reported instead of used to certify numeric data.
No external implementation or asset dictionary is used. Fixtures contain local
game data and must remain under ignored target/.
"""
import argparse
from collections import Counter
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import struct
import sys

from dbpf_extract import payload

PROP = 0x00B1B104
ARRAYS = {0x001C, 0x009C}
PRIMITIVES = {0x09, 0x0A, 0x0D}
COMPOUNDS = {0x30: 8, 0x31: 12, 0x32: 12, 0x33: 16, 0x34: 16, 0x38: 64, 0x39: 24}
sys.setrecursionlimit(100000)


def index(path):
    """The three measured index shapes; retain compact-layout identities."""
    with path.open('rb') as handle:
        header = handle.read(96)
        if header[:4] != b'DBPF':
            raise ValueError('not DBPF')
        count = struct.unpack_from('<I', header, 36)[0]
        size = struct.unpack_from('<I', header, 44)[0]
        offset = struct.unpack_from('<I', header, 64)[0]
        handle.seek(offset)
        data = handle.read(size)
    shape = next(((prefix, stride) for prefix, stride, trailer in
                  [(4, 28, 4), (12, 24, 0), (16, 20, 0)]
                  if prefix + count * stride + trailer == size), None)
    if shape is None:
        raise ValueError(f'unsupported index: {count} records, {size} bytes')
    prefix, stride = shape
    entries = []
    for row in range(count):
        words = struct.unpack_from('<' + 'I' * (stride // 4), data, prefix + row * stride)
        if stride == 28:
            group, typ, high, low, offset, stored, decoded = words
            instance = (high << 32) | low
        elif stride == 24:
            typ, instance, offset, stored, decoded, group = words
        else:
            instance, offset, stored, decoded, group = words
            typ = 0  # no explicit type in this measured compact layout
        entries.append(dict(group=group, type=typ, instance=instance, offset=offset,
                            stored=stored & 0x7FFFFFFF, dec=decoded))
    return entries


def fields(data, selected_types=None):
    """Return selected numeric bits after the complete declared named list fits.

    The default remains the original primitive-only comparison. The optional
    color comparison reads little-endian components and retains HDR/NaN bits.
    """
    selected_types = PRIMITIVES if selected_types is None else selected_types
    if len(data) < 4:
        raise ValueError('missing item count')
    count = struct.unpack_from('>I', data)[0]
    if count > 10000:
        raise ValueError('item count exceeds limit')
    decisions = {}

    def sep(at):
        return data[at:at+4] == b'\1\0\0\0' or data[at+2:at+4] in (b'\x12\0', b'\xa0\0')

    def ends(start, typ, spec):
        if spec in ARRAYS:
            if start + 8 > len(data): return []
            number, stride = struct.unpack_from('>II', data, start)
            if number > 200000 or stride > 16384: return []
            at = start + 8
            if typ in (0x12, 0x13):
                for _ in range(number):
                    if at + 4 > len(data): return []
                    size = struct.unpack_from('>I', data, at)[0]
                    at += 4 + size * (1 if typ == 0x12 else 2)
                    if at > len(data): return []
            else:
                if typ in PRIMITIVES and stride < 4: return []
                at += number * stride
            return [at] if at <= len(data) else []
        if typ in PRIMITIVES: sizes = [4]
        elif typ == 0x01: sizes = [1]
        elif typ == 0x20: sizes = [16]
        elif typ in COMPOUNDS:
            sizes = [COMPOUNDS[typ]]
            if typ not in (0x33, 0x34): sizes.append(COMPOUNDS[typ] + 4)
        elif typ in (0x12, 0x13):
            if start + 4 > len(data): return []
            size = struct.unpack_from('>I', data, start)[0]
            sizes = [4 + size * (1 if typ == 0x12 else 2)]
        elif typ == 0x00: sizes = [spec]
        else: return []
        return [start + size for size in sizes if start + size <= len(data)]

    @lru_cache(None)
    def solve(at, remaining):
        while at + 4 <= len(data) and sep(at): at += 4
        if remaining == 0: return at == len(data)
        if at + 8 > len(data): return False
        ident, typ, spec = struct.unpack_from('>IHH', data, at)
        for end in ends(at + 8, typ, spec):
            if solve(end, remaining - 1):
                decisions[(at, remaining)] = (end, ident, typ, spec)
                return True
        return False

    if not solve(4, count):
        raise ValueError('declared named property list does not consume exactly')
    result = []
    at, remaining = 4, count
    while remaining:
        while at + 4 <= len(data) and sep(at): at += 4
        end, ident, typ, spec = decisions[(at, remaining)]
        if typ in selected_types:
            start = at + 8
            if spec in ARRAYS:
                number, stride = struct.unpack_from('>II', data, start)
                start += 8
            else: number, stride = 1, COMPOUNDS.get(typ, 4)
            components = {0x32: 3, 0x34: 4}.get(typ, 1)
            if stride < 4 * components:
                raise ValueError('numeric component stride too short')
            endian = '<I' if typ in (0x32, 0x34) else '>I'
            words = [struct.unpack_from(endian, data, start + row * stride + component * 4)[0]
                     for row in range(number) for component in range(components)]
            result.append((ident, typ, spec, words))
        at, remaining = end, remaining - 1
    return result


def digest(items):
    value = 0xCBF29CE484222325
    for ident, typ, spec, words in items:
        canonical = struct.pack('<IHHI', ident, typ, spec, len(words))
        canonical += struct.pack('<' + 'I' * len(words), *words)
        for byte in canonical:
            value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--fixtures', type=Path)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    records, errors, exclusions, counts = [], [], [], Counter()
    fixture = None
    if args.fixtures:
        args.fixtures.parent.mkdir(parents=True, exist_ok=True)
        fixture = args.fixtures.open('wb')
        fixture.write(b'RSPROP01' + struct.pack('<I', 0))
    for package in sorted(args.install.rglob('*.package')):
        try: entries = index(package)
        except Exception as error:
            exclusions.append(dict(package=str(package), error=str(error))); continue
        counts['packages_indexed'] += 1
        for entry in entries:
            if entry['type'] != PROP: continue
            counts['prop_resources'] += 1
            identity = f"{package.relative_to(args.install)} {entry['group']:08X}:{entry['instance']:016X}"
            try:
                data = payload(package, entry)
                parsed = fields(data)
            except Exception as error:
                errors.append(dict(resource=identity, error=str(error))); continue
            counts['exact_named_lists'] += 1
            for _, typ, spec, words in parsed:
                counts[f'type_{typ:04X}_values'] += len(words)
                counts['primitive_values'] += len(words)
                counts['primitive_array_fields' if spec in ARRAYS else 'primitive_single_fields'] += 1
            value = digest(parsed)
            records.append(dict(resource=identity, primitive_fields=len(parsed), digest=f'{value:016X}'))
            if fixture: fixture.write(struct.pack('<IQ', len(data), value) + data)
            if entry['instance'] == 0x4041010002AD26C9:
                print(identity, 'camera primitives', [(f'{ident:08X}', [f'{word:08X}' for word in words])
                      for ident, _, _, words in parsed])
    if fixture:
        fixture.seek(8); fixture.write(struct.pack('<I', len(records))); fixture.close()
    report = dict(method='independent named-only declared-count framing, four-byte BE primitive values',
                  counts=counts, package_exclusions=exclusions, framing_failures=errors, records=records)
    (args.out / 'inventory.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps(dict(counts=counts, package_exclusions=len(exclusions), framing_failures=len(errors))))
    if args.fixtures:
        print('fixture_sha256', hashlib.sha256(args.fixtures.read_bytes()).hexdigest())


if __name__ == '__main__': main()
