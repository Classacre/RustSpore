"""Check explicit authored creation links with the hidden original-program renderer.

Input is the independently measured direct-link inventory. This is a controlled
static art check, not retail scene/gameplay/animation acceptance. Original data
stays in the installation; captures and source-derived records belong in target/.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('inventory', type=Path)
    parser.add_argument('executable', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--limit', type=int)
    args = parser.parse_args()
    if args.limit is not None and args.limit <= 0:
        parser.error('--limit must be positive')
    inventory = json.loads(args.inventory.read_text())
    if inventory.get('difference_count') or inventory.get('framing_failures') or inventory.get('index_errors'):
        raise ValueError('direct-link inventory has unresolved input discrepancies')
    creations = {}
    for row in inventory['associations']:
        if row['lod0']:
            creations.setdefault(row['creature'].split(':')[-1], row)
    selected = sorted(creations)
    full_population = len(selected)
    if args.limit is not None:
        selected = selected[:args.limit]
    args.out.mkdir(parents=True, exist_ok=False)
    environment = dict(os.environ, WGPU_ADAPTER_NAME='RTX 3060')
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0
    records = []
    for identity in selected:
        image = args.out / f'{identity}.png'
        log = args.out / f'{identity}.log'
        start = time.monotonic()
        command = [str(args.executable.resolve()), '--source-creature', '--install',
                   str(args.install), '--creature', identity, '--shot', str(image)]
        timed_out = False
        try:
            result = subprocess.run(command, stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=environment,
                startupinfo=startup, creationflags=subprocess.CREATE_NO_WINDOW, timeout=60)
            output, code = result.stdout, result.returncode
        except subprocess.TimeoutExpired as failure:
            output, code, timed_out = failure.stdout or b'', None, True
        with log.open('xb') as stream:
            stream.write(output)
        text = output.decode('utf-8', errors='replace')
        final = re.search(r'SOURCE_3D_RESULT triangles=(\d+) non_background_pixels=(\d+).*presented=false', text)
        source = re.search(r'SOURCE_CREATURE_ART creation=([^ ]+).*model=([^ ]+)', text)
        adapter = 'description="NVIDIA GeForce RTX 3060 Laptop GPU"' in text
        success = code == 0 and final is not None and source is not None and adapter and image.is_file()
        if source and source[1].split(':')[-1] != identity:
            success = False
        record = dict(creation=identity, success=success, exit=code, timed_out=timed_out,
                      seconds=round(time.monotonic()-start, 3), log=str(log), image=str(image) if image.exists() else None,
                      image_sha256=digest(image) if image.exists() else None,
                      rtx_confirmed=adapter, source_model=source[2] if source else None,
                      runtime_vertex_compiles=text.count('SOURCE_3D_COMPILE stage=vs'))
        if final:
            record.update(triangles=int(final[1]), non_background_pixels=int(final[2]))
        if not success:
            record['failure_tail'] = text.splitlines()[-3:]
        records.append(record)
        passed = sum(row['success'] for row in records)
        if len(records) % 16 == 0 or len(records) == len(selected):
            print(f"checked={len(records)}/{len(selected)} passed={passed} failed={len(records)-passed} last={identity}", flush=True)
        (args.out/'progress.json').write_text(json.dumps(dict(status='running', checked=len(records),
            selected=len(selected), passed=passed, failed=len(records)-passed), indent=2))
    report = dict(scope='controlled static authored source art, generic_static, original programs/templates and source maps; no retail scene or animation claim',
        inventory=str(args.inventory), inventory_sha256=digest(args.inventory), executable=str(args.executable),
        executable_sha256=digest(args.executable), linked_unique_creations=full_population,
        selected_creations=len(selected), passed=sum(row['success'] for row in records),
        failed=sum(not row['success'] for row in records), hidden_processes=True,
        retail_fidelity=False, records=records)
    with (args.out/'report.json').open('x') as stream:
        json.dump(report, stream, indent=2)
    (args.out/'progress.json').write_text(json.dumps(dict(status='complete', checked=len(records),
        selected=len(selected), passed=report['passed'], failed=report['failed']), indent=2))
    print(json.dumps({key: report[key] for key in ['linked_unique_creations','selected_creations','passed','failed']}), flush=True)


if __name__ == '__main__':
    main()
