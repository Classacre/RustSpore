"""Preflight or run the separately reviewed constructor-metadata fixture.

Single hidden attempts; complete failures and before/after hashes are retained.
The constructor default loop is distinct from a queued clip's later override.
"""
import argparse
import json
from pathlib import Path
import struct

import cell_runtime_clip_metadata_native_probe as probe
import cell_runtime_clip_metadata_inputs as fixture
from cell_runtime_data_native_run import digest, pinned, saved_file_run, word

ROOT = Path(__file__).resolve().parent.parent
HOST = '0145de54218b405a0cab19fe17d894895dfe4d80455b8ec0e7b8dfebca249227'
ASM = 'f2896df3b316d1f78608debc88051786e5c6b93af0bd2928e20ed80e48f5b70e'
CONTROLS = [0x7f | p | r for p in (0, 0x200, 0x300) for r in (0, 0x400, 0x800, 0xc00)]


def save(path, data):
    with path.open('x') as out:
        json.dump(data, out, indent=2)
        out.write('\n')


def preflight(args):
    image = probe.SourceImage(args.source_exe)
    host = args.host_dir
    manifest = json.loads((host/'manifest.json').read_text())
    audit = json.loads((host/'linked-audit.json').read_text())
    assert digest(host/'host-x86.exe') == HOST and digest(host/'host-x86.asm') == ASM
    assert probe.audit_source(image) == manifest['code_allowlist']
    assert probe.host_source(image) == (host/'host-x86.asm').read_text()
    probe.base.BUFFERS = probe.BUFFERS
    actual = probe.base.linked_audit(host/'host-x86.exe', host/'host-x86.map', image)
    assert audit['pe_entrypoint_hostmain'] is True
    assert {k:v for k,v in audit.items() if k != 'pe_entrypoint_hostmain'} == actual
    pe = probe.base.PE(host/'host-x86.exe')
    nt = struct.unpack_from('<I', pe.data, 60)[0]
    assert pe.base+struct.unpack_from('<I', pe.data, nt+40)[0] == int(audit['symbols']['HostMain'], 16)
    paths = [Path(__file__), args.source_exe, *[p for p in host.iterdir() if p.is_file()]]
    prepared = json.loads((args.input_dir/'report.json').read_text())
    assert prepared['cases'] == 2304 and len(prepared['protocols']) == 11
    for name, expected in prepared['pins'].items():
        p = ROOT/name
        assert digest(p) == expected, name
        paths.append(p)
    for name, expected in [('input.bin', prepared['input_sha256']), ('cases.json', prepared['cases_sha256'])]:
        assert digest(args.input_dir/name) == expected
        paths.append(args.input_dir/name)
    for name, expected in prepared['protocols'].items():
        p = args.input_dir/(name+'.bin')
        assert digest(p) == expected
        paths.append(p)
    raw = (args.input_dir/'input.bin').read_bytes()
    assert len(raw) == 2304*32
    for offset in range(0, len(raw), 32):
        row = raw[offset:offset+32]
        assert struct.unpack_from('<H', row)[0] in CONTROLS and row[2:4] == bytes(2)
        assert row[16:] == bytes(16) and fixture.bounded(word(row, 8), word(row, 12))
    if args.peer_review:
        peer = json.loads(args.peer_review.read_text())
        assert peer['no_blocking_static_issue_found'] is True and peer['native_executed'] is False
        assert HOST in peer['pins'].values()
        for name, expected in peer['pins'].items():
            p = ROOT/name
            assert digest(p) == expected
            paths.append(p)
        paths.append(args.peer_review)
    else:
        assert not args.run, 'Independent actual-linked peer required before execution'
    paths += [args.input_dir/'report.json', Path(probe.__file__), Path(fixture.__file__),
        ROOT/'tools/cell_runtime_data_native_run.py', ROOT/'tools/cell_runtime_clock_native_probe.py',
        ROOT/'tools/cell_schema_probe.py', ROOT/'tools/native_crt_acos_probe.py']
    return prepared, list(dict.fromkeys(p.resolve() for p in paths))


def inspect(request, output, projection=None):
    raw, reply = request.read_bytes(), output.read_bytes()
    assert len(raw)%32 == 0 and len(reply) == len(raw)//32*24
    result = bytearray()
    for i in range(len(raw)//32):
        q, r = raw[i*32:i*32+32], reply[i*24:i*24+24]
        assert r[16:18] == q[:2] and struct.unpack_from('<H', r, 18)[0]&0x3800 == 0
        assert word(r, 20)&~0x3f == 0x1f80
        assert word(r, 12) == (word(q, 4)>>3)&1
        assert 0 < word(r, 0) < 0x7f800000
        result += r[:16]
    if projection:
        with projection.open('xb') as out:
            out.write(result)


def execute(args, prepared, paths):
    native = args.out/'native-v1'
    native.mkdir()
    protocol = native/'protocol'
    protocol.mkdir()
    paths.append(args.out/'preflight.json')
    host = (args.host_dir/'host-x86.exe').resolve()
    with pinned(paths, native):
        smoke = (args.input_dir/'input.bin').read_bytes()[:32]
        reference = protocol/'reference-input.bin'
        reference.write_bytes(smoke)
        status, output = saved_file_run(host, reference, protocol, 'reference')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0, status
        inspect(reference, output)
        expected = output.read_bytes()
        if args.reference_only:
            save(native/'reference-only-report.json', dict(native_executed=True, retries=0, output_sha256=digest(output)))
            return
        checks = [('empty', b'', 0, b''), ('repeat_two', smoke+smoke, 0, expected+expected),
            ('complete_then_partial', smoke+smoke[:16], 7, expected)]
        checks += [(name, (args.input_dir/(name+'.bin')).read_bytes(), 7, b'') for name in prepared['protocols']]
        for name, raw, code, wanted in checks:
            request = protocol/(name+'-input.bin')
            request.write_bytes(raw)
            status, reply = saved_file_run(host, request, protocol, name)
            assert status['exit'] == code and status['stderr_bytes'] == 0, (name, status)
            assert reply.read_bytes() == wanted
        save(protocol/'report.json', dict(checks=len(checks), all_exact=True))
        status, output = saved_file_run(host, args.input_dir/'input.bin', native, 'corpus')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0, status
        inspect(args.input_dir/'input.bin', output, native/'native-projection16.bin')
        save(native/'report.json', dict(schema='original-clip-constructor-metadata-native-v1', native_executed=True,
            records=2304, protocols=len(checks), retries=0, host_sha256=HOST, asm_sha256=ASM,
            input_sha256=digest(args.input_dir/'input.bin'), raw_output_sha256=digest(output),
            projection_sha256=digest(native/'native-projection16.bin'), rust_accepted=False,
            scope='Selected nonnull resource metadata arithmetic and constructor-default loop only; synthetic flag variants. No full constructor, queued override, live selection, binding, clock or event admission.'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, default=ROOT/'target/source-cell/native-runtime-clip-metadata-host-root-reproduction-v1')
    parser.add_argument('--input-dir', type=Path, default=ROOT/'target/source-cell/runtime-clip-metadata-inputs-v1')
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--peer-review', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    parser.add_argument('--reference-only', action='store_true')
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    prepared, paths = preflight(args)
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out/'preflight.json', dict(records=2304, source_bytes=64, linked_audit_reproduced=True,
        independent_peer=bool(args.peer_review), native_executed=False, execution_requested=args.run,
        runner_sha256=digest(Path(__file__))))
    if args.run:
        try:
            execute(args, prepared, paths)
        except BaseException as err:
            save(args.out/'failure.json', dict(error=repr(err), retained=True, retries=0))
            raise
    else:
        print('Constructor metadata preflight passed; no native execution')


if __name__ == '__main__':
    main()
