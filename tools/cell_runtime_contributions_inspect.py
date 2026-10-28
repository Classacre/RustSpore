"""Read-only frozen request/output closure; never starts a native host."""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
import struct

import cell_runtime_contributions_inputs as inputs
import cell_runtime_contributions_native_probe as probe
from cell_runtime_curves_fixture import CONTEXTS, sha_file
from cell_schema_probe import SourceImage


def word(raw, at):
    return struct.unpack_from('<I', raw, at)[0]


def active(blend):
    magnitude = blend & 0x7FFFFFFF
    return magnitude > 0x7F800000 or (not blend & 0x80000000 and magnitude != 0)


def inspect(args):
    prepared = json.loads((args.input_dir/'report.json').read_text())
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    assert sha_file(args.host_dir/'manifest.json') == prepared['host_manifest_sha256']
    assert sha_file(args.host_dir/'host-x86.asm') == manifest['host_source_sha256']
    assert sha_file(Path(probe.__file__)) == manifest['preparation_tool_sha256']
    assert sha_file(Path(inputs.__file__)) == prepared['tool_sha256']
    image = SourceImage(args.source_exe)
    assert probe.audit_source(image) == manifest['code_allowlist']
    assert probe.host_source(image) == (args.host_dir/'host-x86.asm').read_text()
    for name, expected in prepared['hashes'].items():
        assert sha_file(args.input_dir/name) == expected
    assert sha_file(args.source_rows/'report.json') == prepared['source_rows_report_sha256']
    for name, expected in prepared['source_rows_files'].items():
        assert sha_file(args.source_rows/name) == expected
    count = prepared['cases']
    assert (args.input_dir/'input.bin').stat().st_size == count*1268
    if args.native is not None:
        assert args.native.stat().st_size == count*184
        native_stream = args.native.open('rb')
    else:
        native_stream = None
    reached = Counter()
    families = Counter()
    try:
        with (args.input_dir/'input.bin').open('rb') as binary, (args.input_dir/'cases.tsv').open(newline='') as table:
            metadata = list(csv.DictReader(table, delimiter='\t'))
            assert len(metadata) == count
            for case, info in enumerate(metadata):
                raw = binary.read(1268)
                assert int(info['case']) == case
                cw = struct.unpack_from('<H',raw,4)[0]
                assert cw in CONTEXTS and int(info['cw'],16) == cw
                assert not word(raw,24) and not word(raw,28) and not struct.unpack_from('<H',raw,6)[0]
                assert word(raw,32) <= 8
                enabled = active(word(raw,0))
                remaining = word(raw,8)
                n = word(raw,20)
                last_reached = False
                if enabled:
                    assert remaining < 1 << 24 and n <= 32 and raw[244+32*n:] == bytes((32-n)*32)
                    remaining &= ~word(raw,12) & ~word(raw,16)
                    for index in range(n):
                        at = 244+32*index
                        if remaining & (1 << index) and word(raw,at) & 15 == 3:
                            assert word(raw,at)&0x40 == 0 and 3 <= word(raw,at+16) <= 8
                            last_reached = True
                            reached['scalar_descriptors'] += 1
                families[info['family']] += 1
                reached['active' if enabled else 'inactive'] += 1
                if native_stream is not None:
                    reply = native_stream.read(184)
                    assert word(reply,164) == remaining and word(reply,172) == int(enabled)
                    assert struct.unpack_from('<H',reply,176)[0] == cw
                    assert struct.unpack_from('<H',reply,178)[0]&0x3800 == 0
                    assert word(reply,180)&~0x3F == 0x1F80 and word(reply,0)<=8
                    if not last_reached:
                        assert reply[:164] == raw[32:196] and word(reply,168) == 0xA5A5A5A5
        assert dict(families) == prepared['families']
    finally:
        if native_stream is not None:
            native_stream.close()
    return dict(schema='cell-runtime-contributions-read-only-closure-v1', cases=count,
        source_cases=prepared['source_cases'], source_components=prepared['source_components'],
        source_input_hashes_verified=True, original_spans_and_owned_assembly_reproduced=True,
        reached=dict(reached), native_reply_inspected=args.native is not None,
        native_sha256=None if args.native is None else sha_file(args.native),
        scope='Static protocol/input closure and optional independent structural reply inspection; Rust equality and native admission remain separate.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir',type=Path,required=True)
    parser.add_argument('--input-dir',type=Path,required=True)
    parser.add_argument('--source-rows',type=Path,default=inputs.ROOT/'target/source-cell/runtime-data-source-curves-v1')
    parser.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--native',type=Path)
    parser.add_argument('--report',type=Path)
    args = parser.parse_args()
    report = inspect(args)
    if args.report is not None:
        args.report.resolve().relative_to((inputs.ROOT/'target').resolve())
        with args.report.open('x') as output:
            json.dump(report,output,indent=2)
            output.write('\n')
    print(json.dumps(report,indent=2))


if __name__ == '__main__':
    main()
