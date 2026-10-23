"""Run offscreen original-program draws for every resolved primary Cell model.

Takes the existing independent Cell chain inventory. A passing process and a
nonempty PNG establish a controlled draw, not retail screenshot fidelity.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install', type=Path)
    ap.add_argument('inventory', type=Path)
    ap.add_argument('probe', type=Path)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    source = json.loads(args.inventory.read_text())
    models = {}
    rows = 0
    for record in source['records']:
        primary = next((m for m in record.get('model_keys', [])
                        if m.get('matches') and m['type'] == '00E6BCE5'), None)
        if primary is None:
            continue
        instance = primary['instance']
        assert re.fullmatch('[0-9A-Fa-f]{16}', instance), 'invalid inventory model ID'
        models.setdefault(instance.upper(), []).append(record['cell_key'])
        rows += 1
    assert models, 'no source models in inventory'
    args.out.mkdir(parents=True, exist_ok=False)
    records = []
    exe = args.probe.resolve()
    for instance, definitions in sorted(models.items()):
        output = args.out / f'{instance}.png'
        log = args.out / f'{instance}.log'
        started = time.monotonic()
        with log.open('w', encoding='utf-8') as stream:
            process = subprocess.run([str(exe), str(args.install), str(output), instance],
                stdout=stream, stderr=subprocess.STDOUT, timeout=60, check=False)
        text = log.read_text()
        success = process.returncode == 0 and output.is_file()
        success &= 'untouched_vertex_bytes=true' in text
        success &= 'presented=false' in text
        success &= 'NVIDIA GeForce RTX 3060 Laptop GPU' in text
        result = re.search(r'SOURCE_3D_RESULT triangles=(\d+) non_background_pixels=(\d+)', text)
        success &= result is not None
        records.append(dict(instance=instance, definitions=definitions, exit=process.returncode,
            success=success, seconds=round(time.monotonic()-started,3),
            triangles=int(result[1]) if result else None,
            non_background_pixels=int(result[2]) if result else None,
            image_sha256=hashlib.sha256(output.read_bytes()).hexdigest() if output.is_file() else None,
            log=str(log), image=str(output) if output.is_file() else None))
        print(instance, 'PASS' if success else 'FAIL', flush=True)
    report = dict(inventory=str(args.inventory), inventory_sha256=hashlib.sha256(args.inventory.read_bytes()).hexdigest(),
        executable=str(exe), executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
        source_definitions=source['cells'], resolved_definition_rows=rows, unique_primary_models=len(models),
        passed=sum(r['success'] for r in records), failed=sum(not r['success'] for r in records),
        context='controlled generic_static with source material data and identity-frame editor lighting',
        retail_fidelity=False, records=records)
    (args.out/'report.json').write_text(json.dumps(report,indent=2))
    assert report['failed'] == 0, 'one or more source GPU draw gates failed'


if __name__ == '__main__':
    main()
