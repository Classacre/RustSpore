"""Inventory actual camera/editor-setup PROP records with exact named framing.

Labels and links are source observations. This does not assign camera meanings
from names or claim retail package priority, anchor transforms or capture time.
Source-derived records must remain under ignored target/.
"""
import argparse
import hashlib
import json
from pathlib import Path
from dbpf_extract import payload
from prop_numeric_probe import index, PROP
from source_camera_probe import properties


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        raise FileExistsError(args.out)
    rows, errors, indexes = [], [], 0
    for package in sorted(args.install.rglob('*.package')):
        entries = index(package)
        indexes += 1
        for entry in entries:
            if entry['type'] != PROP or entry['instance'] >> 32 not in (0x40410100, 0x40600100):
                continue
            raw = payload(package, entry)
            try:
                fields = properties(raw)
            except Exception as failure:
                errors.append(dict(package=str(package.relative_to(args.install)),
                    instance=f"{entry['instance']:016X}", error=str(failure)))
                continue
            rows.append(dict(package=str(package.relative_to(args.install)),
                instance=f"{entry['instance']:016X}", group=f"{entry['group']:08X}",
                source_sha256=hashlib.sha256(raw).hexdigest(),
                labels=[field.get('text') for field in fields if field['identifier']=='00B2CCCA'],
                properties=fields))
    report = dict(method='fresh exact named/count-bounded source PROP camera/setup census',
        indexes=indexes, records=rows, errors=errors)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open('x') as stream:
        json.dump(report, stream, indent=2, allow_nan=False)
    print(f'indexes={indexes} records={len(rows)} errors={len(errors)}')


if __name__ == '__main__':
    main()
