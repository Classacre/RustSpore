"""Run a separately reviewed, frozen font fixture with regular-file binary IO.

Preparation is the default. Original replies stay under target/. No desktop,
original game process, DLL loading or changes to the reviewed host are used.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

from native_font_glyph_reply import parse_reply, parse_request, request_bytes

ROOT = Path(__file__).resolve().parent.parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--reviewed-host-sha256', required=True)
    parser.add_argument('--glyphs', default='0')
    parser.add_argument('--poison', type=lambda value: int(value, 0), choices=[0, 165], default=0)
    parser.add_argument('--layout', type=int, choices=[0, 1], default=0)
    parser.add_argument('--sequence', action='store_true')
    parser.add_argument('--root-review', type=Path)
    parser.add_argument('--peer-review', type=Path)
    parser.add_argument('--execute-reviewed', action='store_true')
    args = parser.parse_args()
    for path in [args.host_dir, args.out]:
        path.resolve().relative_to((ROOT / 'target').resolve())
    manifest_path = args.host_dir / 'manifest.json'
    manifest = json.loads(manifest_path.read_text())
    lock = json.loads((args.host_dir / 'bundle-lock.json').read_text())
    host = (args.host_dir / 'host-x86.exe').resolve()
    assert digest(host) == manifest['host_exe_sha256'] == args.reviewed_host_sha256
    pinned = [args.host_dir / row['path'] for row in lock['files']]
    for row, path in zip(lock['files'], pinned):
        assert path.stat().st_size == row['bytes'] and digest(path) == row['sha256']
    pinned.append(args.host_dir / 'bundle-lock.json')
    for path_key, sha_key in [('source_path', 'source_sha256'),
                              ('crt_path', 'crt_sha256'), ('font_path', 'font_sha256')]:
        path = Path(manifest[path_key])
        assert digest(path) == manifest[sha_key]
        pinned.append(path)
    for row in manifest['analysis_reports'].values():
        path = ROOT / row['path']
        assert digest(path) == row['sha256']
        pinned.append(path)
    if args.execute_reviewed:
        assert args.root_review is not None and args.peer_review is not None
        for path in [args.root_review, args.peer_review]:
            path.resolve().relative_to((ROOT / 'target').resolve())
            assert path.is_file()
            pinned.append(path)
    glyphs = [int(value, 0) for value in args.glyphs.split(',')]
    request = request_bytes(glyphs, poison=args.poison, layout=args.layout, sequence=args.sequence)
    expected = parse_request(request)
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / 'request.bin').write_bytes(request)
    pinned += [args.out / 'request.bin', Path(__file__), ROOT / 'tools/native_font_glyph_reply.py']
    pins = {str(path.resolve()): digest(path) for path in pinned}
    save(args.out / 'pins-v1.json', pins)
    save(args.out / 'preparation.json', dict(host_sha256=args.reviewed_host_sha256,
        request=expected, executed=False, regular_file_stdin=True, hidden=True))
    if not args.execute_reviewed:
        print(json.dumps(dict(prepared=True, executed=False, request=expected)))
        return
    started, completed, failure = time.perf_counter(), None, None
    try:
        with (args.out / 'request.bin').open('rb') as inp, \
             (args.out / 'reply.bin').open('xb') as out, \
             (args.out / 'stderr.bin').open('xb') as err:
            try:
                completed = subprocess.run([str(host)], stdin=inp, stdout=out, stderr=err,
                    creationflags=subprocess.CREATE_NO_WINDOW, timeout=30)
            except Exception as error:
                failure = error
        status = dict(exit=None if completed is None else completed.returncode,
            elapsed_seconds=time.perf_counter() - started,
            reply_bytes=(args.out / 'reply.bin').stat().st_size,
            stderr_bytes=(args.out / 'stderr.bin').stat().st_size,
            exception=None if failure is None else repr(failure), regular_file_stdin=True,
            hidden_process=True, original_module_loaded=False, GUI=False)
        save(args.out / 'status.json', status)
        if failure is not None:
            raise failure
        exports = []
        try:
            parsed = parse_reply((args.out / 'reply.bin').read_bytes(), expected,
                                 active_sink=lambda item, pixels: exports.append((item, pixels)))
        except Exception as error:
            save(args.out / 'parse-failure.json', dict(error=repr(error)))
            raise
        save(args.out / 'parsed.json', parsed)
        active_dir = args.out / 'active'
        active_dir.mkdir()
        for item, pixels in exports:
            name = f"glyph-{item['ordinal']:02d}-{item['glyph_index']:03d}.alpha"
            (active_dir / name).write_bytes(pixels)
        print(json.dumps(dict(status=status, success=parsed['success'],
                              native_error=parsed.get('native_error'),
                              glyphs=[{key: item[key] for key in ['glyph_index', 'width', 'height',
                                       'pitch', 'active_sha256', 'semantic_sha256']}
                                      for item in parsed['glyphs']])))
        if completed.returncode or status['stderr_bytes'] or not parsed['success']:
            raise SystemExit(2)
    finally:
        changed = [path for path, sha in pins.items()
                   if not Path(path).is_file() or digest(Path(path)) != sha]
        save(args.out / 'immutability-v1.json', dict(pinned_files=len(pins),
            unchanged=not changed, changed_files=changed))
        assert not changed, changed


if __name__ == '__main__':
    main()
