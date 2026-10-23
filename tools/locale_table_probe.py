"""Independent original locale byte-grammar corpus; no native execution.

Grammar follows exact source6B6CC0 and map assignment564880. Only well-formed
UTF8 text is admitted. Formatting/detokenization and malformed UTF8 conversion
remain separate. Original tables and digests stay ignored under target/.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct

from native_archive_index_probe import decode
from dbpf_extract import payload
from refpack_bulk import install as bulk_copy

TYPE, SCOPE = 0x02FAC0B6, 0x02FABF01


def tokenize(data):
    position = 0
    if data[:1] == b'\xef':
        if not data.startswith(b'\xef\xbb\xbf'):
            raise ValueError('invalid source BOM')
        position = 3
    pending, previous = 0xffffffff, 0xffffffff
    entries = []
    kinds = Counter()
    while position < len(data) and data[position] != 0:
        space = re.match(br'[\t-\r ]+', data[position:])
        if space:
            position += len(space[0])
            continue
        if data[position] == ord('#'):
            position += len(re.match(br'[^\x00\r\n]*', data[position:])[0])
            kinds['comments'] += 1
            continue
        if pending == 0xffffffff:
            if data[position] == ord('0'):
                match = re.match(br'0[xX]([0-9a-fA-F]{0,8})', data[position:])
                if not match:
                    raise ValueError(f'invalid hex prefix at{position}')
                pending = int(match[1] or b'0', 16)
                position += len(match[0])
                kinds['explicit_ids'] += 1
            else:
                pending = (previous + 1) & 0xffffffff
                if pending == 0xffffffff:
                    raise ValueError('native automatic-ID sentinel loop outside checked domain')
                kinds['automatic_ids'] += 1
            continue
        raw = re.match(br'[^\x00\r\n]*', data[position:])[0]
        characters = raw.decode('utf8')
        # Source93C8A2..93C8D1 retains only one16-bit unit per decoded value.
        # This is observable for four-byte UTF8; no installed text uses it.
        text = b''.join(struct.pack('<H', ord(character) & 0xffff) for character in characters)
        kinds['four_byte_characters'] += sum(ord(character) > 0xffff for character in characters)
        entries.append((pending, position, len(raw), text))
        previous, pending = pending, 0xffffffff
        position += len(raw)
    kinds['duplicate_stored_ids'] = len(entries) - len({entry[0] for entry in entries})
    return entries, position, kinds


def canonical(entries, stopped):
    data = bytearray(b'locale-table-grammar-v1\0')
    data += struct.pack('<2I', len(entries), stopped)
    for identifier, start, length, text in entries:
        data += struct.pack('<4I', identifier, start, length, len(text)//2) + text
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out/'sources').mkdir()
    bulk_copy()
    resources, failures, totals, languages = [], [], Counter(), Counter()
    paths = sorted(args.install.rglob('*.package'))
    lines = []
    for package in paths:
        with package.open('rb') as stream:
            header = stream.read(96)
            assert header[:4] == b'DBPF'
            count, length, offset = [struct.unpack_from('<I', header, at)[0] for at in (36,44,64)]
            stream.seek(offset)
            _, rows = decode(stream.read(length), count)
        for low, typ, scope, offset, stored, decoded, compression, *_ in rows:
            if typ != TYPE:
                continue
            relative = package.relative_to(args.install).as_posix()
            raw = payload(package, dict(offset=offset,stored=stored,dec=decoded))
            assert len(raw) == decoded and scope == SCOPE
            number = len(resources) + len(failures)
            raw_name = f'sources/{number:05}.locale'
            (args.out/raw_name).write_bytes(raw)
            item = dict(package=relative, instance=f'{low:08X}', scope=f'{scope:08X}',
                        raw_file=raw_name, source_sha256=hashlib.sha256(raw).hexdigest())
            try:
                entries, stopped, kinds = tokenize(raw)
                digest = canonical(entries, stopped)
                item.update(entries=len(entries), stopped_at=stopped, fnv1a64=f'{digest:016X}',
                            utf16_units=sum(len(row[3])//2 for row in entries), kinds=dict(kinds))
                resources.append(item)
                totals.update(kinds)
                totals.update(resources=1, entries=len(entries), utf16_units=item['utf16_units'],
                              source_bytes=len(raw), BOM=raw.startswith(b'\xef\xbb\xbf'),
                              first_NUL_before_EOF=stopped<len(raw))
                languages[relative.split('/')[2]] += 1
                lines.append(f'{relative}\t{low:08X}\t{scope:08X}\t{digest:016X}')
            except Exception as error:
                item['error'] = str(error)
                failures.append(item)
    canonical_lines = '\n'.join(sorted(lines))+'\n'
    (args.out/'python-digests.tsv').write_text(canonical_lines, newline='\n')
    report = dict(schema='original-locale-grammar-source-corpus-v1', package_indexes=len(paths),
                  totals=dict(totals), languages=dict(languages), failures=failures, resources=resources,
                  digests_sha256=hashlib.sha256(canonical_lines.encode()).hexdigest(),
                  source_facts='Exact dossiers826(6B6CC0),829(lookup6B6BB0),830(byte-reader6B5F50),832(map assignment564880),833(converter93C5E0). Native localized lookup824 uses low table ID/type02FAC0B6/scope02FABF01. Four-byte conversion truncates to one16-bit unit.',
                  scope='Independent byte grammar and checked well-formed UTF8 source-unit census. No native parser execution, malformed conversion, locale overlay selection, detokenization, shaping or pixels.')
    (args.out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({key:value for key,value in report.items() if key!='resources'},indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
