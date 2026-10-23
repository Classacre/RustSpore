"""Inspect source SPUI containers and match resource names to executable literals.

Reads the user's install without modifying it. Raw recovered data and source images
belong only in ignored target/, never in distributed assets. Name matches are hash
candidates, not proof of a runtime caller or layout semantics.
"""
import argparse
import json
import re
import struct
from collections import Counter, defaultdict
from pathlib import Path

from dbpf_extract import parse_index, payload

SPUI = 0x250FE9A2


def fnv(text):
    value = 0x811C9DC5
    for byte in text.lower().encode('ascii', 'ignore'):
        value = ((value * 0x01000193) ^ byte) & 0xFFFFFFFF
    return value


def fnv64(data):
    value = 0xCBF29CE484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def container(data):
    magic, version, ic, ac, hc, pc = struct.unpack_from('<I5H', data)
    assert magic == 0xE3FE3FB8 and version <= 3
    offset = 14
    images = [struct.unpack_from('<3I', data, offset + 12 * i) for i in range(ic)]
    offset += 12 * ic
    atlases = [struct.unpack_from('<3I', data, offset + 12 * i) for i in range(ac)]
    offset += 12 * ac
    for _ in range(hc):
        rle = version >= 3 and data[offset] != 0
        offset += int(version >= 3)
        if rle:
            _, _, words = struct.unpack_from('<3i', data, offset)
            assert words >= 0
            offset += 12 + words * 2
        else:
            offset += 12
    proxies = struct.unpack_from(f'<{pc}I', data, offset)
    offset += 4 * pc
    assert offset <= len(data)
    return dict(version=version, images=images, atlases=atlases, hit_mask_count=hc,
                proxies=proxies, body_offset=offset, body_bytes=len(data)-offset,
                body_prefix=data[offset:offset+80].hex())


def elements(data, offset):
    """Typed body walk. Unknown tags fail explicitly rather than guessing widths."""
    start = offset
    types = Counter()

    def read(fmt):
        nonlocal offset
        value = struct.unpack_from('<' + fmt, data, offset)
        offset += struct.calcsize('<' + fmt)
        return value[0] if len(value) == 1 else value

    def properties(count, depth=0):
        nonlocal offset
        assert depth < 32
        result = []
        for _ in range(count):
            at = offset
            ident, kind, size = read('IHH')
            types[f'{kind:04X}'] += 1
            if kind in (2, 3, 5, 7, 9, 11, 15, 16, 17, 19):
                widths = {2: 1, 3: 1, 5: 4, 7: 1, 9: 4, 11: 4, 15: 8,
                          16: 16, 17: 8, 19: 2}
                width = widths[kind] * size
                assert offset + width <= len(data)
                value = data[offset:offset+width].hex()
                offset += width
            elif kind == 18:
                value = []
                for _ in range(size):
                    length = read('H')
                    if length == 0xFFFF:
                        table, instance = read('II')
                        value.append(dict(table=f'{table:08X}', instance=f'{instance:08X}'))
                    else:
                        width = length * 2
                        assert offset + width <= len(data)
                        value.append(data[offset:offset+width].decode('utf-16-le'))
                        offset += width
            elif kind == 20:
                value = []
                struct_type = read('I')
                for _ in range(size):
                    prop_count = read('H')
                    value.append(dict(struct_type=f'{struct_type:08X}', properties=properties(prop_count, depth+1)))
            else:
                raise ValueError(f'unknown type {kind:04X} id {ident:08X} at {at:08X}')
            result.append(dict(id=f'{ident:08X}', type=f'{kind:04X}', count=size, value=value))
        return result

    result = []
    while True:
        at = offset
        marker, index = read('HH')
        if marker != 0x5FF5:
            raise ValueError(f'bad record marker {marker:04X} at {at:08X}')
        if index == 0xFFFF:
            break
        count = read('H')
        result.append(dict(index=index, root=bool(count & 0x8000), properties=properties(count & 0x7FFF)))
    assert read('I') == 0x1C01C047
    assert offset == len(data)
    return dict(bytes=offset-start, types=types, records=result)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    exe = (args.install / 'SporeBin/SporeApp.exe').read_bytes()
    names = defaultdict(set)
    for match in re.finditer(rb'[\x20-\x7e]{3,200}', exe):
        text = match.group().decode('ascii').strip()
        if not re.fullmatch(r'[A-Za-z0-9_./\\ -]{3,200}', text):
            continue
        for value in {text, text.rsplit('/', 1)[-1], text.rsplit('\\', 1)[-1]}:
            names[fnv(value)].add(value)
    records, errors, histogram, body_errors, body_types = [], [], Counter(), [], Counter()
    for folder in ('Data', 'DataEP1', 'bp1content'):
        for package in sorted((args.install / folder).glob('*.package')):
            try:
                entries, _ = parse_index(package)
            except Exception as error:
                errors.append(dict(package=str(package), error=str(error)))
                continue
            for entry in entries:
                if entry['type'] != SPUI:
                    continue
                data = payload(package, entry)
                record = dict(package=str(package), group=f"{entry['group']:08X}",
                              instance=f"{entry['instance']:016X}", bytes=len(data),
                              name_candidates=sorted(names[entry['instance'] & 0xFFFFFFFF]),
                              **container(data))
                histogram[str(record['version'])] += 1
                try:
                    body = elements(data, record['body_offset'])
                    body_types.update(body['types'])
                    record['body_records'] = len(body['records'])
                    record['body_properties'] = sum(body['types'].values())
                    record['body_digest'] = f"{fnv64(data[record['body_offset']:]):016X}"
                    (args.out / f"{package.stem}-{entry['instance']:016X}.elements.json").write_text(json.dumps(body, indent=2), encoding='utf-8')
                except Exception as error:
                    body_errors.append(dict(instance=record['instance'], package=str(package), error=str(error)))
                records.append(record)
                (args.out / f"{package.stem}-{entry['instance']:016X}.spui").write_bytes(data)
    result = dict(packages_scope='top-level Data, DataEP1, bp1content',
                  resources=len(records), versions=histogram, package_errors=errors,
                  body_errors=body_errors, body_types=body_types, records=records)
    (args.out / 'inventory.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    (args.out / 'python-digests.tsv').write_text(''.join(
        f"{Path(r['package']).name}\t{r['instance']}\t{r['body_records']}\t{r['body_properties']}\t{r['body_digest']}\n"
        for r in records if 'body_digest' in r), encoding='utf-8')
    print(f"SPUI resources={len(records)} named={sum(bool(r['name_candidates']) for r in records)} package_errors={len(errors)}")
    print(f"Body complete={len(records)-len(body_errors)}/{len(records)} types={dict(body_types)}")
    print('First body errors:', body_errors[:12])
    for record in records:
        if record['name_candidates']:
            print(record['instance'], record['name_candidates'], 'proxies', len(record['proxies']))


if __name__ == '__main__':
    main()
