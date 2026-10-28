"""Inventory installed PROP records containing Cell/world candidate strings.

This is a string-based candidate search, not proof of stage use or overlay order.
Only source package metadata, digests and successfully framed properties are
retained. It neither exports assets nor executes the game.
"""
import argparse
import hashlib
import json
from pathlib import Path

from dbpf_extract import payload
from prop_numeric_probe import PROP, index
from source_camera_probe import properties

TERMS = ('cell', 'tidepool', 'tide pool', 'microbe')
PACKAGES = ('PatchData.package', 'Spore_Game.package', 'Spore_Graphics.package',
            'Spore_Content.package')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    records, errors, counts = [], [], {}
    for name in PACKAGES:
        package = args.install / 'Data' / name
        entries = [row for row in index(package) if row['type'] == PROP]
        counts[name] = len(entries)
        for row in entries:
            key = f"{row['group']:08X}:{row['type']:08X}:{row['instance']:016X}"
            try:
                data = payload(package, row)
                if len(data) != row['dec']:
                    raise ValueError('decoded length mismatch')
                lower = data.lower()
                matched = [term for term in TERMS if term.encode() in lower or
                           term.encode('utf-16le') in lower]
                if not matched:
                    continue
                result = dict(package=name, key=key, payload_sha256=hashlib.sha256(data).hexdigest(),
                              bytes=len(data), matched_terms=matched)
                try:
                    result['properties'] = properties(data)
                except (ValueError, UnicodeError, RecursionError, IndexError) as error:
                    result['framing_error'] = str(error)
                records.append(result)
            except Exception as error:
                errors.append(dict(package=name, key=key, error=str(error)))
    report = dict(schema='cell-world-prop-candidates-v1', counts=counts, records=records,
                  errors=errors, scope='String candidates only; active use and precedence unverified.')
    args.out.parent.mkdir(parents=True, exist_ok=True)
    if args.out.exists():
        raise FileExistsError(args.out)
    args.out.write_text(json.dumps(report, indent=2, allow_nan=False) + '\n', encoding='utf-8')
    print(f'{sum(counts.values())} records scanned; {len(records)} candidates; {len(errors)} errors')


if __name__ == '__main__':
    main()
