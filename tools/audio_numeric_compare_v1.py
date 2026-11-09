#!/usr/bin/env python3
"""Compare independent Rust/Python numeric censuses with a new target receipt.

Caller admits that all named files, interpreter, and path components remain
trusted and stable throughout; endpoint hashes cannot establish continuity.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / 'target'
TIMEOUT_SECONDS = 1800
FIELDS = ('events', 'numeric_single_fields', 'numeric_supported_array_fields',
          'numeric_supported_array_values', 'numeric_raw_unsupported_fields',
          'numeric_raw_unsupported_bytes', 'numeric_stream_sha256')

# The local imports are compiled from these explicit source paths, without
# consulting import caches or local bytecode. -I -S -B excludes environment,
# site startup and .pyc writes; Python's own stdlib remains an external input.
SOURCE_BOOTSTRAP = r'''
import pathlib, sys, types
index_path, refpack_path, census_path, patch, audio = sys.argv[1:]
for name, path in (("native_archive_index_probe", index_path), ("refpack", refpack_path)):
    module = types.ModuleType(name)
    module.__file__ = path
    module.__package__ = ""
    sys.modules[name] = module
    source = pathlib.Path(path).read_text(encoding="utf-8-sig")
    exec(compile(source, path, "exec"), module.__dict__)
sys.argv = [census_path, "--patchdata", patch, "--audio2", audio]
source = pathlib.Path(census_path).read_text(encoding="utf-8-sig")
exec(compile(source, census_path, "exec"), {"__name__": "__main__", "__file__": census_path, "__package__": None})
'''


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def execute(argv):
    flags = subprocess.CREATE_NO_WINDOW if sys.platform == 'win32' else 0
    try:
        p = subprocess.run(argv, cwd=str(ROOT), stdin=subprocess.DEVNULL,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                           shell=False, timeout=TIMEOUT_SECONDS, check=False,
                           creationflags=flags)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise RuntimeError(f'verifier launch or timeout failed: {error}') from error
    if p.returncode:
        raise RuntimeError(f'verifier exit {p.returncode}; stderr bytes={len(p.stderr)}: {p.stderr[:1000].decode("utf-8", "replace")}')
    try:
        return json.loads(p.stdout), {'exit_code': p.returncode, 'stdout_bytes': len(p.stdout),
                                      'stdout_sha256': hashlib.sha256(p.stdout).hexdigest(),
                                      'stderr_bytes': len(p.stderr), 'stderr_sha256': hashlib.sha256(p.stderr).hexdigest()}
    except (ValueError, UnicodeError) as error:
        raise ValueError(f'verifier did not return JSON: {error}') from error


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--patchdata', type=Path, required=True)
    ap.add_argument('--audio2', type=Path, required=True)
    ap.add_argument('--rust-exe', type=Path, required=True)
    ap.add_argument('--receipt', type=Path, required=True)
    ap.add_argument('--admit-stable-trusted-paths', action='store_true', required=True)
    args = ap.parse_args()
    target = TARGET.resolve(strict=True)
    parent = args.receipt.parent.resolve(strict=True)
    if not parent.is_dir() or not parent.is_relative_to(target):
        raise ValueError('receipt parent must already exist under workspace target')
    name = args.receipt.name
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_-]{0,79}\.json', name, re.IGNORECASE):
        raise ValueError('receipt filename must be a simple ASCII stem followed by .json')
    if name.split('.')[0].upper() in {'CON', 'PRN', 'AUX', 'NUL', *(f'COM{i}' for i in range(1, 10)), *(f'LPT{i}' for i in range(1, 10))}:
        raise ValueError('receipt filename must not use a reserved Windows device name')
    receipt = parent / args.receipt.name
    paths = {
        'patchdata': args.patchdata.resolve(strict=True),
        'audio2': args.audio2.resolve(strict=True),
        'rust_exe': args.rust_exe.resolve(strict=True),
        'rust_verifier_source': ROOT / 'crates/spore-bevy/examples/audio_numeric_corpus_verify_v1.rs',
        'audio_event_source': ROOT / 'crates/spore-core/src/audio_event.rs',
        'native_index_source': ROOT / 'crates/dbpf/src/native_index.rs',
        'refpack_source': ROOT / 'crates/dbpf/src/refpack.rs',
        'python_census': ROOT / 'tools/audio_event_numeric_census_v1.py',
        'python_native_index': ROOT / 'tools/native_archive_index_probe.py',
        'python_refpack': ROOT / 'tools/refpack.py',
        'compare_script': Path(__file__).resolve(strict=True),
        'python_interpreter': Path(sys.executable).resolve(strict=True),
    }
    paths = {key: value.resolve(strict=True) for key, value in paths.items()}
    if paths['patchdata'] == paths['audio2']:
        raise ValueError('package paths must be distinct')
    before = {key: digest(path) for key, path in paths.items()}
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    rust, rust_process = execute([str(paths['rust_exe']), str(paths['patchdata']), str(paths['audio2'])])
    python, python_process = execute([str(paths['python_interpreter']), '-I', '-S', '-B', '-c', SOURCE_BOOTSTRAP,
                                      str(paths['python_native_index']), str(paths['python_refpack']),
                                      str(paths['python_census']), str(paths['patchdata']), str(paths['audio2'])])
    after = {key: digest(path) for key, path in paths.items()}
    if before != after:
        raise ValueError('input or implementation endpoint hash changed during inspection')
    if rust.get('schema') != 'audio-numeric-corpus-verify-v1' or python.get('schema') != 'audio-event-numeric-census-v1':
        raise ValueError('unexpected verifier JSON schema')
    rust_packages = rust.get('packages')
    if not isinstance(rust_packages, list) or len(rust_packages) != 2:
        raise ValueError('unexpected Rust package report')
    rust_roles = {p['role']: p for p in rust_packages}
    if set(rust_roles) != {'PatchData', 'Spore_Audio2'}:
        raise ValueError('unexpected or duplicate Rust roles')
    python_roles = python.get('packages')
    if not isinstance(python_roles, dict) or set(python_roles) != {'patchdata', 'audio2'}:
        raise ValueError('unexpected Python roles')
    comparisons = []
    for role, rust_role in (('patchdata', 'PatchData'), ('audio2', 'Spore_Audio2')):
        r, p = rust_roles[rust_role], python_roles[role]
        if p.get('role') != role:
            raise ValueError(f'{role}: Python role mismatch')
        for side, report in (('rust', r), ('python', p)):
            if Path(report['path']).resolve(strict=True) != paths[role]:
                raise ValueError(f'{role}: {side} package path mismatch')
            if report['sha256_before'] != before[role] or report['sha256_after'] != before[role]:
                raise ValueError(f'{role}: {side} endpoint hash mismatch')
        if p.get('events_exact') != p.get('events') or p.get('events_inexact') != 0:
            raise ValueError(f'{role}: Python reported inexact event framing')
        for field in FIELDS:
            if field not in r or field not in p or r[field] != p[field]:
                raise ValueError(f'{role}: mismatch in {field}: Rust={r.get(field)!r} Python={p.get(field)!r}')
        comparisons.append({'role': role, **{field: r[field] for field in FIELDS}})
    record = {
        'schema': 'audio-numeric-compare-v1', 'started_utc': started,
        'finished_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'inputs': {key: {'path': str(paths[key]), 'sha256_before': before[key], 'sha256_after': after[key]} for key in paths},
        'timeout_seconds_per_verifier': TIMEOUT_SECONDS,
        'processes': {'rust': rust_process, 'python': python_process}, 'comparisons': comparisons,
        'scope': 'Agreement on exact inspection framing counts and the specified numeric stream, not native loader/provider selection, live runtime or playback.',
        'provenance': 'Rust executable and source endpoint hashes are recorded; compiler, build flags, build provenance and source-to-binary correspondence are not established. Python local helpers and census execute from explicitly compiled source under -I -S -B; the Python standard library and broader build environment are not pinned.',
        'stability': 'Caller admitted trusted stable files and path components throughout. Equal endpoint hashes do not establish continuous stability or rule out transient replacement.',
        'write': 'Exclusive create, flush and sync; a failed write or sync can leave a partial receipt, with no automatic deletion.'}
    data = (json.dumps(record, indent=2) + '\n').encode('utf-8')
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, 'O_BINARY', 0)
    fd = os.open(receipt, flags, 0o600)
    try:
        with os.fdopen(fd, 'wb') as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
    except BaseException as error:
        raise RuntimeError(f'receipt write/sync failed; partial file may remain at {receipt}: {error}') from error
    print(f'matching audio numeric census receipt {receipt} sha256={hashlib.sha256(data).hexdigest()}')


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        print(f'error: {error}', file=sys.stderr)
        sys.exit(1)
