"""Inventory local shader/paint/style resources; retain source only in ignored target/."""
import argparse
import json
import re
from pathlib import Path
from dbpf_extract import parse_index, payload
from shader_fragment_probe import Reader, vertex, pixel


def digest(data):
    value = 0xCBF29CE484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    records, digests = [], []
    for package in [args.install / 'Data/PatchData.package', args.install / 'Data/Spore_Graphics.package',
                    args.install / 'DataEP1/Spore_EP1_Data.package']:
        entries, _ = parse_index(package)
        for entry in entries:
            if entry['type'] not in [0x0469A3F7, 0x0248F226, 0x497767B9]:
                continue
            data = payload(package, entry)
            filename = f"{package.stem}-{entry['type']:08X}-{entry['instance']:016X}.bin"
            (args.out / filename).write_bytes(data)
            if entry['type'] == 0x0469A3F7 and data[:4] == b'\0\0\0\1':
                reader = Reader(data)
                assert reader.word() == 1
                vertices = [vertex(reader) for _ in range(255)]
                pixels = [pixel(reader) for _ in range(256)]
                assert reader.offset == len(data)
                uniforms = sum(len(record['uniforms']) for record in vertices + pixels)
                digests.append(f"{package.name}\t{entry['instance']:016X}\t{sum(bool(r['name']) for r in vertices)}\t{sum(bool(r['name']) for r in pixels)}\t{uniforms}\t{digest(data):016X}")
            records.append(dict(package=str(package), instance=f"{entry['instance']:016X}",
                                type=f"{entry['type']:08X}", bytes=len(data), file=filename,
                                prefix=data[:64].hex(' '),
                                ascii_bytes=sum(32 <= v < 127 or v in (9,10,13) for v in data),
                                string_samples=[m.group().decode('ascii')[:120]
                                                for m in list(re.finditer(rb'[\x20-\x7e]{6,}', data))[:5]]))
    (args.out / 'inventory.json').write_text(json.dumps(records, indent=2), encoding='utf-8')
    (args.out / 'python-fragment-digests.tsv').write_text('\n'.join(digests)+'\n', encoding='utf-8')
    for record in records:
        if record['type'] == '0469A3F7':
            print(record['file'], record['bytes'], record['prefix'])


if __name__ == '__main__':
    main()
