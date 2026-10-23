"""Original byte-level PartCapabilities survey, independent of Rust decoding.

No dictionary asset is distributed. The report contains measured metadata and
belongs under ignored target/. Explicitly reports compact-index exclusions.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from dbpf_extract import parse_index, payload

TYPE = 0x7C19AA7A


def inspect(data):
    if data[:4] != b'pctp':
        raise ValueError('bad pctp magic')
    at = 4
    canonical = bytearray(b'pctp')

    def word():
        nonlocal at
        if at + 4 > len(data):
            raise ValueError(f'truncated word at {at}')
        value = struct.unpack_from('>I', data, at)[0]
        at += 4
        canonical.extend(struct.pack('<I', value))
        return value

    def bounded(count, minimum):
        if count > (len(data)-at)//minimum:
            raise ValueError(f'impossible count {count} at {at}')
        return range(count)

    version = word()
    if version not in (3, 4):
        raise ValueError(f'unsupported version {version}')
    priority_bits = word() if version >= 4 else None
    names = []
    for _ in bounded(word(), 8):
        size = word()
        if size > len(data)-at:
            raise ValueError('name overruns resource')
        name = data[at:at+size]
        at += size
        canonical.extend(name)
        names.append({'name': name.decode('ascii', 'replace'), 'id': f'{word():08X}'})
    remaps = [{'key': f'{word():08X}', 'capability': f'{word():08X}', 'index': word()}
              for _ in bounded(word(), 12)]
    aggregates = []
    for _ in bounded(word(), 8):
        capability = word()
        members = [f'{word():08X}' for _ in bounded(word(), 4)]
        aggregates.append({'capability': f'{capability:08X}', 'members': members})
    deforms = []
    for _ in bounded(word(), 8):
        capability = word()
        channels = []
        for _ in bounded(word(), 16):
            identifier = word()
            raw = [word(), word(), word()]
            channels.append({'id': f'{identifier:08X}', 'raw_parameters': [f'{v:08X}' for v in raw],
                             'first_two_as_floats': [struct.unpack('<f', struct.pack('<I', v))[0] for v in raw[:2]],
                             'flags_word': f'{raw[2]:08X}'})
        deforms.append({'capability': f'{capability:08X}', 'channels': channels})
    if at != len(data):
        raise ValueError(f'{len(data)-at} trailing bytes')
    digest = 0xCBF29CE484222325
    for byte in canonical:
        digest = ((digest ^ byte)*0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return {'version': version, 'priority_bits': priority_bits, 'names': names, 'remaps': remaps,
            'aggregates': aggregates, 'deforms': deforms, 'consumed': at,
            'canonical_fnv1a64': f'{digest:016X}', 'source_sha256': hashlib.sha256(data).hexdigest()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('root', nargs='?', type=Path, default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--digest-out', type=Path)
    args = parser.parse_args()
    records, failures, excluded, digests = [], [], [], []
    for package in sorted(args.root.rglob('*.package')):
        try:
            entries, _ = parse_index(package)
        except ValueError as exc:
            excluded.append({'package': str(package), 'error': str(exc)})
            continue
        for entry in entries:
            if entry['type'] != TYPE:
                continue
            key = f"{TYPE:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            try:
                record = inspect(payload(package, entry))
                record.update(package=package.name, key=key)
                records.append(record)
                digests.append(f"{package.name} {key} {record['canonical_fnv1a64']}")
            except Exception as exc:
                failures.append({'package': package.name, 'key': key, 'error': repr(exc)})
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps({'records': records, 'failures': failures, 'excluded_packages': excluded}, indent=2), encoding='utf-8')
    if args.digest_out:
        args.digest_out.write_text('\n'.join(sorted(digests))+'\n', encoding='utf-8')
    print(json.dumps({'resources': len(records), 'failures': failures, 'excluded_packages': len(excluded),
                      'counts': [{'package': r['package'], 'key': r['key'], 'version': r['version'],
                                  'names': len(r['names']), 'remaps': len(r['remaps']),
                                  'aggregates': len(r['aggregates']), 'deforms': len(r['deforms'])} for r in records]}, indent=2))
    if failures or not records:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
