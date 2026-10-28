"""Find exact native Cell initialization word in installed base effect directories.

Raw byte matches do not prove a record boundary, an effect lookup, or an active
resource. Payloads remain in installed packages; the report stores digests and
offsets only. No executable is run.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from dbpf_extract import payload
from gmdl_skin_probe import indexes

TYPE = 0xEA5118B0
WORD = 0x05C7A4D0
PACKAGES = ('PatchData.package', 'Spore_Game.package', 'Spore_Graphics.package',
            'Spore_Content.package')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    records = []
    for name in PACKAGES:
        path = args.install / 'Data' / name
        for entry in indexes(path):
            if entry['type'] != TYPE:
                continue
            data = payload(path, entry)
            if len(data) != entry['dec']:
                raise ValueError(f'{name}: decoded size mismatch')
            offsets = {}
            for label, endian in (('little', '<'), ('big', '>')):
                needle = struct.pack(endian + 'I', WORD)
                positions = []
                at = data.find(needle)
                while at >= 0:
                    positions.append(at)
                    at = data.find(needle, at + 1)
                offsets[label] = positions
            records.append(dict(package=name,
                                key=f"{entry['group']:08X}:{TYPE:08X}:{entry['instance']:016X}",
                                decoded_bytes=len(data), payload_sha256=hashlib.sha256(data).hexdigest(),
                                offsets=offsets))
    report = dict(schema='cell-world-effect-id-candidates-v1', word=f'{WORD:08X}',
                  records=records, status='Raw matches only; effect directory semantics unresolved.')
    args.out.parent.mkdir(parents=True, exist_ok=True)
    if args.out.exists():
        raise FileExistsError(args.out)
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    count = sum(len(v) for r in records for v in r['offsets'].values())
    print(f'{len(records)} effect directories, {count} byte matches')


if __name__ == '__main__':
    main()
