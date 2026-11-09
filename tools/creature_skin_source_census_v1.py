"""Run explicit, conditional edge inspections on a pinned source snapshot set.

Each file and its caller-supplied pin are passed to the Rust inspection example.
This does not establish that the supplied serialized bytes equal later mutable
native rows, nor does it validate palette identity, geometry or gameplay.
"""
import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import subprocess


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source_dir', type=Path)
    parser.add_argument('pins', type=Path)
    parser.add_argument('exe', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--workers', type=int, default=4)
    parser.add_argument('--include-remap', action='store_true')
    args = parser.parse_args()
    if not 1 <= args.workers <= 8:
        raise ValueError('workers must be in 1..8')
    if not hasattr(subprocess, 'CREATE_NO_WINDOW'):
        raise RuntimeError('Windows hidden-process execution is required')
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    output_parent = args.output.parent.resolve(strict=True)
    if not output_parent.is_relative_to(target):
        raise ValueError('output must be beneath the workspace target directory')
    if args.output.exists() or args.output.is_symlink():
        raise FileExistsError(args.output)
    source_dir = args.source_dir.resolve(strict=True)
    executable = args.exe.resolve(strict=True)
    pin_bytes = args.pins.read_bytes()
    pin_hash = hashlib.sha256(pin_bytes).hexdigest()
    pins = json.loads(pin_bytes)
    files = sorted(source_dir.glob('*.creature'))
    if not files or len(files) > 10_000 or any(p.is_symlink() or not p.is_file() for p in files):
        raise ValueError('source set must contain 1..10000 regular .creature files')
    selected = []
    for path in files:
        path = path.resolve(strict=True)
        pin = pins.get(str(path))
        if not isinstance(pin, str) or len(pin) != 64 or any(c not in '0123456789abcdef' for c in pin):
            raise ValueError(f'missing or malformed source pin for {path}')
        selected.append((path, pin))
    exe_hash = digest(executable)

    def inspect(case):
        path, pin = case
        if digest(path) != pin:
            raise ValueError(f'source hash mismatch before inspection: {path}')
        command = [str(executable), str(path), pin, '--assume-frozen-post-mutation']
        if args.include_remap:
            command.append('--include-remap')
        result = subprocess.run(
            command,
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            timeout=30, check=False, shell=False, creationflags=subprocess.CREATE_NO_WINDOW,
        )
        if digest(path) != pin:
            raise ValueError(f'source hash mismatch after inspection: {path}')
        entry = {'file': path.name, 'sha256': pin}
        if result.returncode:
            if result.stdout:
                raise ValueError(f'failed inspection unexpectedly wrote stdout: {path}')
            entry['error'] = result.stderr.decode('utf-8', errors='replace').strip()
        else:
            report = json.loads(result.stdout)
            if (report.get('schema') != 'creature-skin-source-edge-inspection-v1'
                    or report.get('source_sha256') != pin):
                raise ValueError(f'inspection identity mismatch: {path}')
            edges = report['edges']
            members = report['endpoint_members']
            if (len(members) != report['endpoint_count']
                    or sum(map(len, members)) != 2 * len(edges)):
                raise ValueError(f'inspection extent mismatch: {path}')
            entry.update(rows=report['row_count'], endpoints=len(members), edges=len(edges),
                         membership_entries=sum(map(len, members)))
            if args.include_remap:
                endpoint_map = report['endpoint_to_row']
                edge_map = report['edge_to_row']
                if len(endpoint_map) != len(members) or len(edge_map) != len(edges):
                    raise ValueError(f'remap extent mismatch: {path}')
                entry.update(unmapped_endpoints=endpoint_map.count(0xffffffff),
                             nonidentity_edge_entries=sum(i != row for i, row in enumerate(edge_map)))
        return entry

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        records = list(pool.map(inspect, selected))
    if digest(executable) != exe_hash:
        raise ValueError('inspection executable hash changed')
    if digest(args.pins) != pin_hash:
        raise ValueError('source pin manifest hash changed')
    failures = Counter(r['error'] for r in records if 'error' in r)
    result = {
        'schema': 'creature-skin-source-edge-census-v1',
        'source_dir': str(source_dir), 'pins_file': str(args.pins.resolve()),
        'pins_sha256': pin_hash, 'inspector': str(executable),
        'inspector_sha256': exe_hash, 'include_remap': args.include_remap, 'files': len(records),
        'accepted': len(records) - sum(failures.values()),
        'rejected_by_error': dict(sorted(failures.items())),
        'admissions': 'Every supplied payload is assumed to be a frozen post-mutation row snapshot; earlier native preparation and vector operations are assumed successful.',
        'limits': 'This is a bounded model corpus inspection, not native execution or evidence of active resource selection, palette identity, generated geometry or gameplay. Hashes before/after do not prove continuous file stability.',
        'records': records,
    }
    payload = (json.dumps(result, indent=2) + '\n').encode('utf-8')
    with args.output.open('xb') as f:
        f.write(payload)
        f.flush()
    print(f"files={result['files']} accepted={result['accepted']} rejected={sum(failures.values())}")
    for error, count in failures.most_common(12):
        print(f'{count}: {error}')
    if len(failures) > 12:
        print(f'{len(failures) - 12} additional error categories are in the report')
    print(f'output_sha256={hashlib.sha256(payload).hexdigest()}')


if __name__ == '__main__':
    main()
