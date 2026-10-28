"""Prepare and compare the static Cell table/AnimList/ANIM join.

prepare scans supplied installed packages read-only and writes inputs, expected
TSV and provenance JSON beneath target/. compare checks standalone Rust stdout.
Index coverage follows gmdl_skin_probe.indexes; type-zero entries cannot prove
absence. Numeric instance matches are candidates only, including zero keys and
filename paths; they do not prove a keyed load, precedence, or playback.
prepare requires a new output directory and creates every file exclusively.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gmdl_skin_probe import indexes
from dbpf_extract import payload
from animlist_v7_corpus_compare import independent_parse as parse7
from animlist_v10_corpus_compare import independent_parse as parse10
from cell_anim_state_table_compare_v1 import read_audit, target_path

# Must match the explicit ceilings in the standalone Rust comparison driver.
MAX_ROWS = 131
MAX_RECORD_LINKS = 100_000
MAX_ENTRY_LINKS = 1_000_000
MAX_ANIM_EDGES = 1_000_000


def bounded_add(current, added, limit, kind):
    if added < 0 or current > limit - added:
        raise ValueError(f'{kind} output limit exceeded ({limit})')
    return current + added


def safe_field(value):
    value = str(value)
    if '\t' in value or '\n' in value or '\r' in value:
        raise ValueError('identity contains a TSV separator')
    return value


def prepare(root, audit, out):
    raw, table = read_audit(audit)
    bounded_add(0, len(table), MAX_ROWS, 'row')
    if not root.is_dir():
        raise ValueError('package root missing')
    packages = sorted(root.rglob('*.package'))
    if not packages:
        raise ValueError('no packages')
    lists, anims, errors = [], [], []
    for package in packages:
        try:
            entries = indexes(package)
        except Exception as exc:
            errors.append({'package': str(package), 'error': str(exc)})
            continue
        for ordinal, entry in enumerate(entries):
            typ = entry['type']
            if typ not in (0x4aeb6bc6, 0xee17c6ad):
                continue
            identity = {'package': safe_field(package), 'index_ordinal': ordinal, **entry}
            if typ == 0xee17c6ad:
                anims.append(identity)
            else:
                try:
                    data = payload(package, entry)
                    if data[:4] != b'tsla' or len(data) < 8:
                        raise ValueError('invalid header')
                    version = int.from_bytes(data[4:8], 'big')
                    if version not in (7, 10):
                        raise ValueError(f'unsupported version {version}')
                    parsed = (parse7 if version == 7 else parse10)(data)
                    lists.append((identity, version, data, parsed['records']))
                except Exception as exc:
                    errors.append({'identity': identity, 'error': str(exc)})
    if errors or not lists:
        raise ValueError(f'index/resource errors or no lists: {errors}')
    # Create a new leaf, then exclusively create every file; this rejects stale
    # files, symlinks and reparse points at each destination. target_path has
    # already resolved the parent and checked that it remains beneath target/.
    out.parent.mkdir(parents=True, exist_ok=True)
    out.mkdir(exist_ok=False)
    def write_bytes(name, data):
        with (out / name).open('xb') as stream:
            stream.write(data)
    def write_text(name, data):
        with (out / name).open('x', encoding='utf-8', newline='') as stream:
            stream.write(data)
    write_bytes('table.bin', raw)
    manifest = []
    for i, (identity, version, data, _) in enumerate(lists):
        filename = f'list-{i}.bin'
        write_bytes(filename, data)
        manifest.append('\t'.join(map(str, [version, identity['package'], identity['index_ordinal'], identity['type'], identity['group'], identity['instance'], filename])))
    write_text('manifest.tsv', '\n'.join(manifest) + '\n')
    write_text('anims.tsv', ''.join('\t'.join(map(str, [a['package'], a['index_ordinal'], a['type'], a['group'], a['instance']])) + '\n' for a in anims))
    by_instance = {}
    for i, identity in enumerate(anims):
        by_instance.setdefault(identity['instance'], []).append(i)
    expected = []
    matched_rows = record_links = entry_links = resolved_entries = anim_edges = 0
    for row_index, row in enumerate(table):
        expected.append(f'R\t{row_index}')
        found = False
        for list_index, (_, version, _, records) in enumerate(lists):
            for record_index, record in enumerate(records):
                if record['key'] != row[2]:
                    continue
                found = True
                record_links = bounded_add(record_links, 1, MAX_RECORD_LINKS, 'record link')
                expected.append(f'M\t{row_index}\t{list_index}\t{record_index}')
                if version == 10:
                    for vi, variant in enumerate(record['variants']):
                        for ei, entry in enumerate(variant['entries']):
                            instance = entry['words'][2]
                            matches = by_instance.get(instance, [])
                            entry_links = bounded_add(entry_links, 1, MAX_ENTRY_LINKS, 'entry link')
                            anim_edges = bounded_add(anim_edges, len(matches), MAX_ANIM_EDGES, 'ANIM edge')
                            resolved_entries += bool(matches)
                            expected.append(f'E\t{row_index}\t{list_index}\t{record_index}\t{vi}\t{ei}\t{instance}\t' + ','.join(map(str, matches)))
        matched_rows += found
    write_text('expected.tsv', '\n'.join(expected) + '\n')
    report = {'packages': len(packages), 'table_sha256': hashlib.sha256(raw).hexdigest(), 'rows': len(table), 'matched_rows': matched_rows, 'record_links': record_links, 'v10_entry_links': entry_links, 'v10_entries_with_index_matches': resolved_entries, 'anim_edges': anim_edges, 'anims': len(anims), 'lists': [dict(identity=identity, version=version, sha256=hashlib.sha256(data).hexdigest(), records=len(records)) for identity, version, data, records in lists], 'errors': errors}
    write_text('provenance.json', json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('mode', choices=['prepare', 'compare'])
    p.add_argument('--dir', type=Path, required=True)
    p.add_argument('--package-root', type=Path)
    p.add_argument('--audit', type=Path)
    p.add_argument('--rust-output', type=Path)
    a = p.parse_args()
    if a.mode == 'prepare' and os.path.lexists(a.dir):
        p.error('prepare output directory must not already exist')
    out = target_path(a.dir)
    if a.mode == 'prepare':
        if not a.package_root or not a.audit:
            p.error('prepare requires --package-root and --audit')
        prepare(a.package_root, a.audit, out)
    else:
        if not a.rust_output:
            p.error('compare requires --rust-output')
        expected = (out / 'expected.tsv').read_text(encoding='utf-8').splitlines()
        actual = target_path(a.rust_output).read_text(encoding='utf-8-sig').splitlines()
        if expected != actual:
            for i, (e, v) in enumerate(zip(expected, actual)):
                if e != v:
                    raise ValueError(f'mismatch at line {i + 1}: expected {e!r}, got {v!r}')
            raise ValueError(f'line count mismatch: {len(expected)} vs {len(actual)}')
        print(f'PASS: {len(expected)} ordered source-join lines')


if __name__ == '__main__':
    main()
