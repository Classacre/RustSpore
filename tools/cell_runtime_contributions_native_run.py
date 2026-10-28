"""Admit and run the reviewed, isolated original scalar contribution fixture.

Default mode only validates preparation and writes a preflight. --run performs
hidden file-backed executions with immutable pins and bounded pre-input mapping
retries. No retail process, actual clip selection or game controller is invoked.
"""
import argparse
import json
from pathlib import Path
import struct

import cell_runtime_contributions_inspect as inspection
import cell_runtime_contributions_native_probe as probe
from cell_runtime_data_native_run import digest, pinned, reviewed_file_run, word
from cell_schema_probe import SourceImage, SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
HOST = 'd1f9d69e69882985cc6263a370187796c0f67f582059d85dfde60409e8966f49'
ASM = '25ee82698cce887fe336fc9cbaec4476ff34224685336e5c8bda8c8f7130311c'


def save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def review(args):
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    linked = json.loads((args.host_dir/'linked-audit.json').read_text())
    original = args.prepared_host
    preparation = json.loads((original/'preparation-review.json').read_text())
    peer = json.loads((original/'animation-agent-independent-review-v1.json').read_text())
    assert manifest['source_sha256'] == SOURCE_SHA256
    assert manifest['source_bytes'] == 370
    assert digest(args.host_dir/'host-x86.asm') == manifest['host_source_sha256'] == ASM
    assert (args.host_dir/'host-x86.asm').read_bytes() == (original/'host-x86.asm').read_bytes()
    assert digest(args.host_dir/'host-x86.exe') == linked['host_exe_sha256'] == HOST
    assert digest(Path(probe.__file__)) == manifest['preparation_tool_sha256']
    for name, expected in preparation['pins'].items():
        assert digest(ROOT/name) == expected, name
    for name, expected in peer['pins'].items():
        assert digest(ROOT/name) == expected, name
    assert peer['status'] == 'STATIC_DOMAIN_CLEAR_NATIVE_ADMISSION_PENDING'
    assert peer['source_exact'] and not peer['findings'] and peer['source_bytes'] == 370
    image = SourceImage(args.source_exe)
    assert probe.audit_source(image) == manifest['code_allowlist']
    assert probe.host_source(image) == (args.host_dir/'host-x86.asm').read_text()
    actual = probe.linked_audit(args.host_dir/'host-x86.exe', args.host_dir/'host-x86.map', image)
    assert all(linked[key] == value for key, value in actual.items())
    check = argparse.Namespace(host_dir=original, input_dir=args.input_dir,
        source_rows=args.source_rows, source_exe=args.source_exe, native=None)
    closure = inspection.inspect(check)
    assert closure['cases'] == 52911 and closure['source_cases'] == 44127
    return manifest, closure


def inspect_native(input_path, output_path, projection=None):
    assert input_path.stat().st_size % probe.INPUT_BYTES == 0
    count = input_path.stat().st_size // probe.INPUT_BYTES
    assert output_path.stat().st_size == count*probe.OUTPUT_BYTES
    output = None if projection is None else projection.open('xb')
    try:
        with input_path.open('rb') as requests, output_path.open('rb') as replies:
            for index in range(count):
                raw, reply = requests.read(probe.INPUT_BYTES), replies.read(probe.OUTPUT_BYTES)
                active = inspection.active(word(raw, 0))
                mask = word(raw, 8)
                if active:
                    mask &= ~word(raw, 12) & ~word(raw, 16)
                assert word(reply, 164) == mask and word(reply, 172) == int(active), index
                assert struct.unpack_from('<H', reply, 176)[0] == struct.unpack_from('<H', raw, 4)[0], index
                assert struct.unpack_from('<H', reply, 178)[0] & 0x3800 == 0, index
                assert word(reply, 180) & ~0x3F == 0x1F80 and word(reply, 0) <= 8, index
                if not active:
                    assert reply[:164] == raw[32:196] and word(reply, 168) == 0xA5A5A5A5, index
                if output is not None:
                    output.write(reply[:probe.PROJECTION_BYTES])
    finally:
        if output is not None:
            output.close()
    return count


def execute(args, closure):
    folder = args.out/'native-v1'
    folder.mkdir(exist_ok=False)
    report = json.loads((args.input_dir/'report.json').read_text())
    prep = json.loads((args.prepared_host/'preparation-review.json').read_text())
    paths = [args.source_exe, Path(__file__), args.out/'preflight.json',
        *[p for p in args.host_dir.iterdir() if p.is_file()],
        *[p for p in args.prepared_host.iterdir() if p.is_file()],
        *[ROOT/name for name in prep['pins']],
        args.input_dir/'report.json', *[args.input_dir/name for name in report['hashes']],
        args.source_rows/'report.json', *[args.source_rows/name for name in report['source_rows_files']],
        ROOT/'tools/cell_runtime_data_native_run.py', ROOT/'tools/cell_schema_probe.py',
        ROOT/'tools/native_crt_acos_probe.py', ROOT/'crates/spore-core/src/cell_runtime_data.rs',
        ROOT/'crates/spore-core/src/native_sse.rs', ROOT/'crates/spore-core/src/native_x87.rs']
    paths = list(dict.fromkeys(path.resolve() for path in paths))
    host = (args.host_dir/'host-x86.exe').resolve()
    with pinned(paths, folder):
        protocol = folder/'protocol'
        protocol.mkdir()
        with (args.input_dir/'input.bin').open('rb') as stream:
            smoke = stream.read(probe.INPUT_BYTES)
        reference = protocol/'reference-input.bin'
        reference.write_bytes(smoke)
        status, output = reviewed_file_run(host, reference, protocol, 'reference')
        assert status['exit'] == 0 and status['stderr_bytes'] == 0
        assert inspect_native(reference, output) == 1
        expected = output.read_bytes()
        checks = [('empty', b'', 0, b'')]
        for size in [1, 31, probe.INPUT_BYTES-1]:
            checks += [(f'partial{size}', smoke[:size], 7, b''),
                (f'complete_then_partial{size}', smoke+smoke[:size], 7, expected)]
        for at, value, label in [(4, 0x17F, 'reserved_pc'), (4, 0x27E, 'unmasked_cw'),
                (4, 0x1027F, 'reserved_header_half'), (24, 1, 'reserved24'),
                (28, 1, 'reserved28'), (32, 9, 'aggregate_count9')]:
            raw = bytearray(smoke)
            struct.pack_into('<I', raw, at, value)
            checks.append((label, bytes(raw), 7, b''))
        base = bytearray(smoke)
        struct.pack_into('<I', base, 0, 0x3F800000)
        struct.pack_into('<I', base, 8, 1)
        struct.pack_into('<I', base, 20, 1)
        base[244:] = bytes(1024)
        struct.pack_into('<I', base, 244, 3)
        struct.pack_into('<I', base, 260, 3)
        for at, value, label in [(8, 0x1000000, 'mask_high'), (20, 33, 'count33'),
                (244, 0x43, 'angular'), (260, 2, 'slot2'), (260, 9, 'slot9'),
                (276, 1, 'descriptor_padding')]:
            raw = bytearray(base)
            struct.pack_into('<I', raw, at, value)
            checks.append((label, bytes(raw), 7, b''))
        results = []
        for label, raw, code, wanted in checks:
            path = protocol/f'{label}-input.bin'
            path.write_bytes(raw)
            status, output = reviewed_file_run(host, path, protocol, label)
            assert status['exit'] == code and status['stderr_bytes'] == 0, (label, status)
            assert output.read_bytes() == wanted, label
            results.append(dict(case=label, exact=True, exit=status['exit']))
        save(protocol/'report.json', dict(checks=len(results), cases=results))
        status, output = reviewed_file_run(host, args.input_dir/'input.bin', folder, 'corpus', timeout=120)
        assert status['exit'] == 0 and status['stderr_bytes'] == 0, status
        count = inspect_native(args.input_dir/'input.bin', output, folder/'native-projection.bin')
        assert count == closure['cases']
        result = dict(schema='original-cell-scalar-contributions-native-run-v1', exact=True,
            native_executed=True, cases=count, source_cases=closure['source_cases'],
            protocol_checks=len(results), host_exe_sha256=HOST, host_assembly_sha256=ASM,
            input_sha256=digest(args.input_dir/'input.bin'), output_path=str(output.resolve()),
            output_sha256=digest(output), projection_sha256=digest(folder/'native-projection.bin'),
            source_sha256=SOURCE_SHA256,
            scope='Unchanged bounded scalar caller spans and complete scalar writer only; all raw nonfinite outcomes retained. Structural native reply checks here; independent Rust numerical comparison is a separate gate.')
        save(folder/'report.json', result)
        print(json.dumps(result, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir', type=Path, required=True)
    parser.add_argument('--prepared-host', type=Path, required=True)
    parser.add_argument('--input-dir', type=Path, required=True)
    parser.add_argument('--source-rows', type=Path, default=ROOT/'target/source-cell/runtime-data-source-curves-v1')
    parser.add_argument('--source-exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve())
    _, closure = review(args)
    args.out.mkdir(parents=True, exist_ok=False)
    save(args.out/'preflight.json', dict(schema='reviewed-cell-contributions-preflight-v1',
        reviewed_host_sha256=HOST, source_assembly_sha256=ASM, runner_sha256=digest(Path(__file__)),
        closure=closure, native_execution_requested=args.run))
    if args.run:
        execute(args, closure)
    else:
        print(json.dumps(dict(preflight=True, cases=closure['cases'], native_executed=False)))


if __name__ == '__main__':
    main()
