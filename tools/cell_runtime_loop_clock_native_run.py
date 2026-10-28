"""Review and optionally execute the original finite loop-arithmetic fixture.

Default is read-only preflight. Runs are hidden, file-backed, single attempts.
The explicit arithmetic snapshot does not admit counters, events or a live clip.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import struct

import cell_runtime_loop_clock_native_probe as probe
from cell_runtime_data_native_run import digest, pinned, saved_file_run, word

ROOT = Path(__file__).resolve().parent.parent
HOST = 'da0c28fdf706fb496b5401a9a6f1982386bfafb39ca9a39d325d456fbc28eb89'
ASM = '7ba9932ba65ab9e0a782afd4cc485115287656520a0ececff656256415feed4d'
OWNED = 'fddb92ed67bb21717c5070b4758cfaa0eebc8303ccc538c92e2c47998ce0e77a'
PEERS = {
    'target/source-cell/runtime-loop-clock-independent-animation-peer-v1.json':
        '619eb13e2232d6fbadca92d70a9b3fde56db6ec234696c9d1c155dd83b484f12',
    'target/source-cell/runtime-loop-clock-independent-animation-linked-peer-v1.json':
        '13094f505977d9dc15babb99112acf2896592e4afcf0abed6fcbc116ec8b9e92',
}
CONTROLS = [0x7f | p | r for p in (0, 0x200, 0x300) for r in (0, 0x400, 0x800, 0xc00)]


def save(path, value):
    with path.open('x') as out:
        json.dump(value, out, indent=2)
        out.write('\n')


def wide(raw, offset):
    return struct.unpack_from('<Q', raw, offset)[0]


def domain(raw):
    assert len(raw) == 64 and word(raw, 0) == 0
    assert struct.unpack_from('<H', raw, 4)[0] in CONTROLS
    assert raw[6:8] == bytes(2) and wide(raw, 12) == 0
    t, d, f = word(raw, 8), wide(raw, 20), word(raw, 28)
    assert t & 0x7fffffff < 0x7f800000
    assert t & 0x80000000 == 0 or t == 0x80000000
    assert 0 < d < 0x7ff0000000000000
    assert 0x3f800000 <= f < 0x7f800000
    assert [word(raw, at) for at in (32, 36, 40)] == [1, 1, 1]
    assert raw[44:] == bytes(20)


def preflight(args):
    paths = [Path(__file__), args.source_exe, args.source_crt]
    for name, expected in PEERS.items():
        path = ROOT / name
        assert digest(path) == expected
        peer = json.loads(path.read_text())
        assert peer['no_blocking_static_issue_found'] and peer['native_executed'] is False
        paths.append(path)
        for filename, wanted in peer['pins'].items():
            p = ROOT / filename
            assert digest(p) == wanted, filename
            paths.append(p)
    images = probe.Images(args.source_exe, args.source_crt)
    manifest = json.loads((args.host_dir / 'manifest.json').read_text())
    linked = json.loads((args.host_dir / 'linked-audit.json').read_text())
    assert probe.audit_source(images) == manifest['code_allowlist']
    assert probe.import_export_proof(images) == manifest['import_export']
    assert probe.host_source(images) == (args.host_dir / 'host-x86.asm').read_text()
    assert digest(args.host_dir / 'host-x86.asm') == ASM
    assert digest(args.host_dir / 'host-x86.exe') == HOST
    assert probe.linked_audit(args.host_dir / 'host-x86.exe', args.host_dir / 'host-x86.map', images) == linked
    assert linked['owned_code_sha256'] == OWNED and linked['source_bytes_exact']
    report = json.loads((args.input_dir / 'report.json').read_text())
    review = json.loads((args.input_dir / 'static-preparation-review.json').read_text())
    assert report['cases'] == 2256 and len(report['protocols']) == 12
    assert digest(args.host_dir / 'manifest.json') == report['host_manifest_sha256'] == review['host_manifest_sha256']
    for filename, expected in review['pins'].items():
        path = ROOT / filename
        assert digest(path) == expected
        paths.append(path)
    for filename, expected in report['hashes'].items():
        path = args.input_dir / filename
        assert digest(path) == expected
        paths.append(path)
    contexts = Counter()
    with (args.input_dir / 'input.bin').open('rb') as inp:
        while raw := inp.read(64):
            domain(raw)
            contexts[f'{struct.unpack_from("<H", raw, 4)[0]:04X}'] += 1
    assert dict(contexts) == report['contexts'] and sum(contexts.values()) == 2256
    paths += [args.host_dir / p for p in ('manifest.json', 'linked-audit.json', 'host-x86.asm', 'host-x86.exe', 'host-x86.map')]
    paths += [args.input_dir / p for p in ('report.json', 'static-preparation-review.json')]
    paths += [ROOT / 'tools' / p for p in ('cell_runtime_loop_clock_native_probe.py', 'cell_runtime_data_native_run.py',
        'cell_schema_probe.py', 'native_crt_acos_probe.py', 'cell_runtime_clock_native_probe.py',
        'cell_runtime_clock_native_probe_v2.py', 'cell_runtime_loop_clock_inputs.py')]
    return report, list(dict.fromkeys(p.resolve() for p in paths)), dict(records=2256, contexts=dict(contexts),
        game_bytes=157, crt_bytes=217, actual_linked_and_independent_peers_pass=True)


def inspect(request, reply, projection=None):
    raw, result = request.read_bytes(), reply.read_bytes()
    assert len(raw) % 64 == 0 and len(result) == len(raw)
    modeled = bytearray()
    for offset in range(0, len(raw), 64):
        q, r = raw[offset:offset+64], result[offset:offset+64]
        assert r[56:58] == q[4:6]
        assert struct.unpack_from('<H', r, 58)[0] & 0x3800 == 0
        assert word(r, 60) & ~0x3f == 0x1f80
        assert r[18:20] == r[46:48] == bytes(2)
        assert word(r, 20) == word(r, 24)
        modeled += r[:56]
    if projection is not None:
        with projection.open('xb') as out:
            out.write(modeled)
    return dict(records=len(raw)//64, metadata_checked=True, modeled_bytes=56)


def execute(args, prepared, paths):
    native = args.out / 'native-v1'
    native.mkdir()
    protocol = native / 'protocol'
    protocol.mkdir()
    host = (args.host_dir / 'host-x86.exe').resolve()
    paths.append(args.out / 'preflight.json')
    with pinned(paths, native):
        smoke = (args.input_dir / 'input.bin').read_bytes()[:64]
        reference = protocol / 'reference-input.bin'
        reference.write_bytes(smoke)
        status, output = saved_file_run(host, reference, protocol, 'reference')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0, status
        inspect(reference, output)
        expected = output.read_bytes()
        if args.reference_only:
            save(native / 'reference-only-report.json', dict(native_executed=True, reference_only=True,
                host_exe_sha256=HOST, output_sha256=digest(output), retries=0))
            return
        results = []
        checks = [('empty', b'', 0, b''), ('repeat_two', smoke+smoke, 0, expected+expected),
            ('complete_then_partial', smoke+smoke[:32], 7, expected)]
        checks += [(item['name'], (args.input_dir/item['path']).read_bytes(), 7, b'')
            for item in prepared['protocols']]
        for label, raw, code, wanted in checks:
            request = protocol / f'{label}-input.bin'
            request.write_bytes(raw)
            status, output = saved_file_run(host, request, protocol, label)
            assert status['exit'] == code and status['stderr_bytes'] == 0, (label, status)
            assert output.read_bytes() == wanted, label
            results.append(dict(case=label, exit=code, exact=True))
        save(protocol / 'report.json', dict(protocol_checks=len(results), checks=results))
        status, output = saved_file_run(host, args.input_dir/'input.bin', native, 'corpus')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0, status
        summary = inspect(args.input_dir/'input.bin', output, native/'native-projection56.bin')
        assert summary['records'] == 2256
        report = dict(schema='source-loop-clock-arithmetic-native-run-v1', **summary, native_executed=True,
            host_exe_sha256=HOST, host_assembly_sha256=ASM, protocol_checks=len(results), retries=0,
            input_sha256=digest(args.input_dir/'input.bin'), output_sha256=digest(output),
            projection_sha256=digest(native/'native-projection56.bin'), rust_accepted=False,
            scope='Original finite loop arithmetic and original finite SSE2 CRT modf only. Explicit arithmetic snapshot; no actual loop/counter/generation/event admission, live duration setter or actor association.')
        save(native/'report.json', report)
        print(json.dumps(report, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, default=ROOT/'target/source-cell/native-runtime-loop-clock-host-root-reproduction-v1')
    parser.add_argument('--input-dir', type=Path, default=ROOT/'target/source-cell/runtime-loop-clock-inputs-v1')
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--source-crt', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'))
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    parser.add_argument('--reference-only', action='store_true')
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    prepared, paths, summary = preflight(args)
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out/'preflight.json', dict(**summary, native_executed=False,
        native_execution_requested=args.run, runner_sha256=digest(Path(__file__))))
    if args.run:
        try:
            execute(args, prepared, paths)
        except BaseException as error:
            save(args.out/'failure.json', dict(error=repr(error), retained=True, retries=0))
            raise
    else:
        print(json.dumps(dict(preflight=True, **summary)))


if __name__ == '__main__':
    main()
