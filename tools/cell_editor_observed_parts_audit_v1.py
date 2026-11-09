"""Audit parts observed in resolved shipped Cell creation chains, not editor availability.

Inputs are the prior Cell chain inventory and independently computed class census.
Every referenced creature payload is read again, hashed and parsed from the install.
Output is restricted to a new directory under ignored target; no package is changed.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path

from cell_capability_cache_probe import source_rows
from dbpf_extract import payload
from gmdl_skin_probe import indexes


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('install', type=Path)
    p.add_argument('--inventory', type=Path, required=True)
    p.add_argument('--classes', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    for path in (a.inventory, a.classes, a.out):
        path.resolve().relative_to(target)
    if a.out.exists():
        raise FileExistsError(a.out)
    inv = json.loads(a.inventory.read_text(encoding='utf-8'))
    classes = json.loads(a.classes.read_text(encoding='utf-8'))
    if classes['failures']:
        raise ValueError('classification census contains failures')
    class_by_key = {(int(x['scope'], 16) << 32) | int(x['low'], 16): x for x in classes['parts']}
    providers = defaultdict(list)
    packages = sorted(a.install.rglob('*.package'))
    for package in packages:
        for entry in indexes(package):
            providers[(entry['type'], entry['instance'])].append((package, entry))
    records = []
    part_occurrences = defaultdict(list)
    failures = []
    for cell in inv['records']:
        for art in cell.get('model_art', []):
            group, typ, instance = (int(x, 16) for x in art['key'].split(':'))
            if typ != 0x0F43029A:
                continue
            rel = art['package'].replace('\\', '/')
            candidates = [(p, e) for p, e in providers[(typ, instance)]
                          if p.relative_to(a.install).as_posix() == rel and e['group'] == group]
            if len(candidates) != 1:
                failures.append({'cell': cell['cell_key'], 'rig': art['key'], 'error': 'missing_or_ambiguous_source_record'})
                continue
            package, entry = candidates[0]
            try:
                raw = payload(package, entry)
                if len(raw) != entry['dec']:
                    raise ValueError('decoded_size_mismatch')
                rows = source_rows(raw)
                row_ids = []
                for ordinal, row in enumerate(rows):
                    key = (int.from_bytes(row[132:136], 'little') << 32) | int.from_bytes(row[136:140], 'little')
                    row_ids.append(f'{key:016X}')
                    part_occurrences[key].append({'cell': cell['cell_key'], 'rig': art['key'], 'row': ordinal})
                records.append({'cell': cell['cell_key'], 'structure': cell['structure_key'],
                                'rig_package': rel, 'rig': art['key'], 'rig_sha256': hashlib.sha256(raw).hexdigest(),
                                'rows': len(rows), 'part_instances': row_ids})
            except Exception as exc:
                failures.append({'cell': cell['cell_key'], 'rig': art['key'], 'error': repr(exc)})
    result_parts = []
    for key, occurrences in sorted(part_occurrences.items()):
        cls = class_by_key.get(key)
        copies = []
        for package, entry in providers[(0x00B1B104, key)]:
            try:
                raw = payload(package, entry)
                if len(raw) != entry['dec']:
                    raise ValueError('decoded_size_mismatch')
                copies.append({'package': package.relative_to(a.install).as_posix(),
                    'legacy_archive_group_word': f"{entry['group']:08X}",
                    'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)})
            except Exception as exc:
                failures.append({'part': f'{key:016X}', 'error': repr(exc)})
        result_parts.append({'part_instance': f'{key:016X}', 'occurrences': occurrences,
            'prop_candidates': copies, 'agreed_native_class': cls['possible_classes'][0]
                if cls and cls['all_candidate_copies_agree'] and len(cls['possible_classes']) == 1 else None,
            'class_census_present': cls is not None})
    summary = {'packages': len(packages), 'inventory_cells': len(inv['records']),
        'resolved_cell_rig_chains': len(records), 'unique_rigs': len({(r['rig_package'], r['rig']) for r in records}),
        'row_occurrences': sum(r['rows'] for r in records), 'unique_parts': len(result_parts),
        'part_prop_copies': sum(len(x['prop_candidates']) for x in result_parts),
        'parts_without_prop': sum(not x['prop_candidates'] for x in result_parts),
        'parts_without_agreed_class': sum(x['agreed_native_class'] is None for x in result_parts),
        'agreed_classes': dict(sorted(Counter(str(x['agreed_native_class']) for x in result_parts).items())),
        'failures': len(failures)}
    report = {'scope': 'Observed part rows in previously resolved Cell creation rigs only. Not an editor catalog, availability, labels, unlock order, mesh or player-facing classification. No overlay winner inferred. Index group is a legacy archive word.',
        'inputs': {'inventory_sha256': hashlib.sha256(a.inventory.read_bytes()).hexdigest(),
                   'classes_sha256': hashlib.sha256(a.classes.read_bytes()).hexdigest()},
        'summary': summary, 'chains': records, 'parts': result_parts, 'failures': failures}
    a.out.mkdir(parents=True, exist_ok=False)
    (a.out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(summary, indent=2))
    if failures:
        raise SystemExit(1)

if __name__ == '__main__':
    main()
