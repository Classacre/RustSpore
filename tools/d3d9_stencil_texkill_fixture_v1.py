"""Run a reviewed, pinned synthetic D3D9 fixture; retain hashes, no game data."""
import argparse
import hashlib
import json
import re
import subprocess
import threading
from pathlib import Path
from cell_hierarchy_76f930_compare_v1 import digest, expected_hash, inside_target

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'crates/spore-bevy/examples/d3d9_stencil_texkill_fixture_v1.rs'
HELPERS = [ROOT / 'crates/spore-bevy/src/native_d3d9.rs', ROOT / 'crates/spore-bevy/src/native_d3dx.rs']


def run_bounded(example, timeout, arguments=()):
    process = subprocess.Popen([str(example), *arguments], stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW)
    outputs = [bytearray(), bytearray()]
    errors = []
    def drain(stream, target, limit):
        try:
            while True:
                chunk = stream.read(8192)
                if not chunk: break
                if len(target) + len(chunk) > limit:
                    errors.append('output cap exceeded')
                    process.kill()
                    break
                target.extend(chunk)
        except (OSError, ValueError) as exc:
            errors.append(str(exc))
        finally:
            stream.close()
    threads = [threading.Thread(target=drain, args=(process.stdout, outputs[0], 65536)),
               threading.Thread(target=drain, args=(process.stderr, outputs[1], 65536))]
    for thread in threads: thread.start()
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.kill(); process.wait(); errors.append('timeout')
    finally:
        for thread in threads: thread.join()
    if errors or process.returncode or outputs[1]:
        stderr = bytes(outputs[1]).decode('utf-8', errors='replace')
        raise ValueError(f'fixture exit={process.returncode}; errors={errors}; stderr={stderr}')
    return bytes(outputs[0])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--example', type=inside_target, required=True)
    p.add_argument('--example-sha256', type=expected_hash, required=True)
    p.add_argument('--source-sha256', type=expected_hash, required=True)
    p.add_argument('--shader-sha256', type=expected_hash, required=True)
    p.add_argument('--d3d9-source-sha256', type=expected_hash, required=True)
    p.add_argument('--d3dx-source-sha256', type=expected_hash, required=True)
    p.add_argument('--compiler', type=Path, required=True)
    p.add_argument('--compiler-sha256', type=expected_hash, required=True)
    p.add_argument('--out', type=inside_target, required=True)
    p.add_argument('--timeout', type=float, default=30)
    a = p.parse_args()
    if not 0 < a.timeout <= 60 or not a.example.is_file() or a.example.suffix.lower() != '.exe':
        raise ValueError('invalid fixture or timeout')
    compiler = a.compiler.resolve(strict=True)
    pins = [(SOURCE, a.source_sha256), (a.example, a.example_sha256),
            (HELPERS[0], a.d3d9_source_sha256), (HELPERS[1], a.d3dx_source_sha256),
            (compiler, a.compiler_sha256)]
    if any(digest(path) != pin for path, pin in pins):
        raise ValueError('source or executable pin mismatch')
    a.out.mkdir(parents=True, exist_ok=False)
    report = {'success': False, 'scope': 'synthetic D3D9 fixture only; no source gameplay or shader claim',
              'source_sha256_before': a.source_sha256, 'example_sha256_before': a.example_sha256,
              'helper_sha256_before': [a.d3d9_source_sha256, a.d3dx_source_sha256],
              'compiler_sha256_before': a.compiler_sha256}
    try:
        def verify_output(stdout, mode):
            text = stdout.decode('utf-8')
            module = re.search(r'^COMPILER_PATH=(.+)$', text, re.M)
            if module is None or Path(module.group(1)).resolve(strict=True) != compiler:
                raise ValueError('loaded compiler path differs from pinned file')
            if mode == 'preflight':
                match = re.search(r'^PREFLIGHT_OK shader_hex=([0-9a-f]+) gpu=false$', text, re.M)
                count = None
            else:
                match = re.search(r'^FIXTURE_OK width=64 height=64 coverage_pixels=(\d+) shader_hex=([0-9a-f]+) semantics=synthetic_only$', text, re.M)
                count = int(match.group(1)) if match else None
            if match is None:
                raise ValueError('missing fixture result')
            encoded = match.group(1 if mode == 'preflight' else 2)
            if len(encoded) % 2 or len(encoded) > 32768:
                raise ValueError('invalid fixture shader encoding')
            shader = bytes.fromhex(encoded)
            if hashlib.sha256(shader).hexdigest() != a.shader_sha256:
                raise ValueError('compiled shader pin mismatch')
            return shader, count
        preflight = run_bounded(a.example, a.timeout, ('--preflight',))
        shader, _ = verify_output(preflight, 'preflight')
        if any(digest(path) != pin for path, pin in pins):
            raise ValueError('input changed before GPU run')
        stdout = run_bounded(a.example, a.timeout)
        adapter_lines = re.findall(r'^SOURCE_3D_ADAPTER index=(\d+) vendor=10DE description="NVIDIA GeForce RTX 3060 Laptop GPU"$', stdout.decode('utf-8'), re.M)
        if len(adapter_lines) != 1:
            raise ValueError('expected exactly one logged NVIDIA RTX 3060 adapter')
        shader_after, count = verify_output(stdout, 'fixture')
        if shader_after != shader:
            raise ValueError('preflight and GPU shader bytes differ')
        report.update(success=True, coverage_pixels=count, adapter={'index': int(adapter_lines[0]), 'vendor': '10DE', 'description': 'NVIDIA GeForce RTX 3060 Laptop GPU'}, preflight_stdout_sha256=hashlib.sha256(preflight).hexdigest(),
                      stdout_sha256=hashlib.sha256(stdout).hexdigest(),
                      compiled_shader_sha256=hashlib.sha256(shader).hexdigest(), compiled_shader_bytes=len(shader))
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        report['error'] = f'{type(exc).__name__}: {exc}'
    finally:
        try:
            report['source_sha256_after'] = digest(SOURCE)
            report['example_sha256_after'] = digest(a.example)
            report['helper_sha256_after'] = [digest(path) for path in HELPERS]
            report['compiler_sha256_after'] = digest(compiler)
            if any(digest(path) != pin for path, pin in pins):
                report['success'] = False
                report['error'] = 'input changed during run'
        except OSError as exc:
            report['success'] = False
            report['error'] = f'final hash failed: {exc}'
        (a.out / 'report.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    if not report['success']:
        raise ValueError('fixture failed; see report')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
