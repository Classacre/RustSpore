"""Native-schema goldens for current Cell globals and explicit older-layout rejection."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from dbpf_extract import payload
from gmdl_skin_probe import indexes


def semantic_digest(data, fields):
    value = 0xcbf29ce484222325
    for row in sorted(fields, key=lambda r: r['offset']):
        assert row['native_type'] in [1, 2, 3, 5, 7]
        at = row['offset']
        assert at+4 <= len(data)
        for byte in struct.pack('<III', at, row['native_type'], 4) + data[at:at+4]:
            value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return value


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('--metadata', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    fields = [r for r in json.loads(args.metadata.read_text()) if r['struct'] == 'globals']
    assert len(fields) == 68
    args.out.mkdir(parents=True, exist_ok=False)
    records, inventory = [], []
    packages = 0

    def add(data, source):
        accepted = len(data) == 276
        expected = semantic_digest(data, fields) if accepted else 0
        records.append(struct.pack('<IBQ', len(data), accepted, expected) + data)
        inventory.append(dict(source=source, size=len(data), supported=accepted,
            sha256=hashlib.sha256(data).hexdigest()))

    for package in sorted(args.install.rglob('*.package')):
        packages += 1
        for entry in indexes(package):
            if entry['type'] == 0x2a3ce5b7:
                add(payload(package, entry), dict(package=str(package.relative_to(args.install)),
                    key=f"{entry['group']:08X}:{entry['type']:08X}:{entry['instance']:016X}"))
    source_count = len(records)
    source_supported = sum(r['supported'] for r in inventory)
    # Distinct words expose wrong accessor offsets even when a shipped config
    # repeats values. These remain explicitly synthetic, not extra game files.
    for variant in range(3):
        words = [(0x3f000000 + i*0x1001 + variant*0x10101) & 0xffffffff for i in range(69)]
        if variant == 1:
            words = [v ^ 0x80000000 for v in words]
        if variant == 2:
            words[0x44//4] = 0x80000000
            words[0x48//4] = 0x7fc12345
        add(struct.pack('<69I', *words), dict(synthetic=variant))
    fixture = args.out/'corpus.bin'
    fixture.write_bytes(b'RSCELLG1' + struct.pack('<I', len(records)) + b''.join(records))
    report = dict(packages=packages, source_resources=source_count, source_supported=source_supported,
        source_unsupported=source_count-source_supported, synthetic_records=len(records)-source_count,
        fields_per_supported_record=len(fields), source_typed_fields=source_supported*len(fields),
        metadata_sha256=hashlib.sha256(args.metadata.read_bytes()).hexdigest(),
        fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(), records=inventory)
    (args.out/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps({k: v for k, v in report.items() if k != 'records'}, indent=2))


if __name__ == '__main__':
    main()
