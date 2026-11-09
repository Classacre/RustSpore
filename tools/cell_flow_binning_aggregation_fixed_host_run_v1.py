"""Run one reviewed E4DC40 to E4DD50 combined host fixture; never executes a game or accepts data.

Only endpoint hashes are verified: the caller must keep every input and the
output directory trusted and stable throughout, including during OS opens.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys

TARGET = Path(__file__).resolve().parent.parent / 'target'
EXPECTED = {
    'source_pe': '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a',
    'host': '247aaaae7a5e080949152390c061dbb2a001d257f603947d086dcec64296e0ae',
    'assembly': '265a495ebc23b3f11d40e162265119a6a8511a6fd245ce33c56c67c87fff9815',
    'manifest': '3d887fff71ad480cbe4fdb7b6a6b44b7534f1003bf7ef44a9fb9cda4e02ff99a',
    'positions': '8c73602f4cd0f53391108ea0e6f0534b986186006b9e575eb7284464be9a16cd',
    'vectors': 'be0ba040ce8e731c21bd156e7ea370dc532e6c625297572e01c2f86739a09f16',
    'scalars': '0e6f5c53d8dd3c7b939a650fbf26bbf10c9a9b900740d74c18868ef48ac1705d',
}
TIMEOUT_SECONDS = 30
OUTPUT_BYTES = 84032


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def check_metadata(data):
    if len(data) != OUTPUT_BYTES:
        raise ValueError(f'expected {OUTPUT_BYTES} stdout bytes, got {len(data)}')
    return {
        'binning': check_one_metadata(data, 84000, 'binning'),
        'aggregation': check_one_metadata(data, 84016, 'aggregation'),
    }


def check_one_metadata(data, offset, label):
    cw, sw, mxcsr, eflags, reserved = struct.unpack_from('<HHIII', data, offset)
    if cw != 0x027f:
        raise ValueError(f'{label}: unexpected x87 control word {cw:04X}')
    if sw & 0x3840:
        raise ValueError(f'{label}: x87 stack TOP is nonzero or stack-fault flag is set: {sw:04X}')
    if mxcsr & ~0x3f != 0x1f80:
        raise ValueError(f'{label}: unexpected MXCSR control bits: {mxcsr:08X}')
    if eflags & 0x400:
        raise ValueError(f'{label}: direction flag is set: {eflags:08X}')
    if reserved != 0:
        raise ValueError(f'{label}: nonzero reserved metadata word')
    return {'x87_cw': f'{cw:04X}', 'x87_sw': f'{sw:04X}',
            'x87_top': (sw >> 11) & 7, 'x87_exception_flags': f'{sw & 0x3f:02X}',
            'mxcsr': f'{mxcsr:08X}', 'mxcsr_exception_flags': f'{mxcsr & 0x3f:02X}',
            'eflags': f'{eflags:08X}', 'direction_flag': 0, 'reserved': 0}


def exclusive_write(path, data):
    # O_EXCL rejects an existing file or link; the caller admits the directory
    # and its parents remain stable during the create and write.
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, 'O_BINARY', 0)
    fd = os.open(path, flags, 0o600)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
    except BaseException as error:
        raise RuntimeError(f'write/sync failed; partial output may remain at {path}: {error}') from error


def run():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-exe', required=True, type=Path)
    parser.add_argument('--host-dir', required=True, type=Path)
    parser.add_argument('--output-dir', required=True, type=Path)
    parser.add_argument('--stem', required=True, help='new simple filename stem for .bin and .json')
    parser.add_argument('--admit-stable-trusted-paths', action='store_true', required=True,
                        help='admit input paths/files and output directory remain trusted and stable throughout')
    args = parser.parse_args()
    if sys.platform != 'win32':
        raise ValueError('the reviewed 32-bit host requires Windows')
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,79}', args.stem) or args.stem in ('.', '..'):
        raise ValueError('stem must be a simple ASCII filename, 1 to 80 characters')
    root = TARGET.resolve(strict=True)
    host_dir = args.host_dir.resolve(strict=True)
    outdir = args.output_dir.resolve(strict=True)
    if not host_dir.is_dir() or not outdir.is_dir() or not host_dir.is_relative_to(root) or not outdir.is_relative_to(root):
        raise ValueError('host and output directories must already exist under workspace target')
    paths = {'source_pe': args.source_exe.resolve(strict=True),
             'host': host_dir / 'host-x86.exe', 'assembly': host_dir / 'host-x86.asm',
             'manifest': host_dir / 'manifest.json', 'positions': host_dir / 'positions.bin',
             'vectors': host_dir / 'vectors.bin', 'scalars': host_dir / 'scalars.bin'}
    # Resolve every existing artifact individually and reject redirected files
    # outside the admitted host directory, subject to caller path stability.
    for key in ('host', 'assembly', 'manifest', 'positions', 'vectors', 'scalars'):
        paths[key] = paths[key].resolve(strict=True)
        if paths[key].parent != host_dir:
            raise ValueError(f'{key} must resolve directly within host directory')
    binary = outdir / (args.stem + '.bin')
    receipt = outdir / (args.stem + '.json')
    if binary == receipt:
        raise ValueError('output paths must differ')
    runner_path = Path(__file__).resolve(strict=True)
    runner_before = digest(runner_path)
    before = {key: digest(path) for key, path in paths.items()}
    if before != EXPECTED:
        raise ValueError('input hash mismatch: ' + ', '.join(k for k in EXPECTED if before[k] != EXPECTED[k]))
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    process = None
    failure = None
    try:
        process = subprocess.run([str(paths['host'])], cwd=str(host_dir), stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=TIMEOUT_SECONDS,
                                 check=False, shell=False, creationflags=subprocess.CREATE_NO_WINDOW)
    except (OSError, subprocess.TimeoutExpired) as error:
        failure = f'host launch or timeout failed: {error}'
    after = {key: digest(path) for key, path in paths.items()}
    runner_after = digest(runner_path)
    if after != EXPECTED or runner_after != runner_before:
        raise ValueError('input or runner endpoint hash changed during run')
    if failure is not None:
        raise RuntimeError(failure)
    if process.returncode != 0:
        raise RuntimeError(f'host exit {process.returncode}; stderr bytes={len(process.stderr)}')
    metadata = check_metadata(process.stdout)
    output_hash = hashlib.sha256(process.stdout).hexdigest()
    record = {
        'schema': 'cell-flow-binning-aggregation-fixed-host-run-v1',
        'started_utc': started,
        'finished_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'inputs': {key: {'path': str(paths[key]), 'sha256_before': before[key], 'sha256_after': after[key]} for key in EXPECTED},
        'runner': {'path': str(runner_path), 'sha256_before': runner_before, 'sha256_after': runner_after},
        'host': {'exit_code': process.returncode, 'timeout_seconds': TIMEOUT_SECONDS,
                 'stdout_bytes': len(process.stdout), 'stderr_bytes': len(process.stderr),
                 'stderr_sha256': hashlib.sha256(process.stderr).hexdigest(), 'fp_and_flags': metadata},
        'output': {'path': str(binary), 'bytes': len(process.stdout), 'sha256': output_hash},
        'scope': 'One fixed synthetic binning-to-aggregation fixture in the reviewed original-instruction host; E4E730 and E4E4C0 are skipped. FP state is explicitly reset before each call and sticky flags are not carried between them. This is not a game process, live actor, field, or gameplay observation. Unused scratch records retain A5 fixture poison and are not meaningful native data.',
        'path_admission': 'Caller admitted trusted stable files, path components and output directory throughout. Equal endpoint hashes do not prove continuous stability or rule out transient replacement.',
        'receipt_write': 'Output and receipt use exclusive creation, flush and sync. A failed create/write/sync may leave a partial file; no automatic deletion.',
    }
    exclusive_write(binary, process.stdout)
    if digest(binary) != output_hash:
        raise RuntimeError(f'output readback hash mismatch; file remains at {binary}')
    try:
        exclusive_write(receipt, (json.dumps(record, indent=2) + '\n').encode('utf-8'))
    except Exception as error:
        raise RuntimeError(f'binary remains at {binary}; receipt creation failed: {error}') from error
    print(f'fixed combined fixture output {binary} sha256={output_hash}; receipt {receipt}')


if __name__ == '__main__':
    try:
        run()
    except Exception as error:
        print(f'error: {error}', file=sys.stderr)
        sys.exit(1)
