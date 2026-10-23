"""Cross-check the reviewed bounded x86 host against existing turn goldens.

This runs only the separately generated no-CRT fixture host. The original game
is never launched. Input/output, identities and comparison report remain target/.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--host-dir', type=Path, required=True)
    ap.add_argument('--turn-reference', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    workspace_target = (Path(__file__).resolve().parent.parent/'target').resolve()
    args.host_dir.resolve().relative_to(workspace_target)
    args.out.resolve().relative_to(workspace_target)
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    executable = (args.host_dir/'host-x86.exe').resolve()
    assert hashlib.sha256(executable.read_bytes()).hexdigest() == manifest['host_exe_sha256']
    reference = args.turn_reference.read_bytes()
    assert reference[:8] == b'RSCELLT1'
    count = struct.unpack_from('<I', reference, 8)[0]
    assert len(reference) == 12+78*count and 0 < count < 1_000_000
    input_records, expected = [], []
    for ordinal in range(count):
        record = reference[12+78*ordinal:12+78*(ordinal+1)]
        parameters = bytearray(100)
        struct.pack_into('<IHH', parameters, 0, 0, struct.unpack_from('<H', record, 64)[0], 0)
        parameters[20:32] = record[:12]
        parameters[32:44] = record[12:24]
        parameters[44:56] = record[24:36]
        parameters[56:60] = record[36:40]
        parameters[76:88] = record[40:52]
        parameters[88:100] = record[52:64]
        input_records.append(bytes(parameters))
        expected.append(record[66:78]+bytes(4))
    args.out.mkdir(parents=True, exist_ok=False)
    supplied = b''.join(input_records)
    (args.out/'turn-input.bin').write_bytes(supplied)
    result = subprocess.run([str(executable)], input=supplied, capture_output=True,
        creationflags=subprocess.CREATE_NO_WINDOW, timeout=60)
    (args.out/'turn-output.bin').write_bytes(result.stdout)
    (args.out/'host-stderr.bin').write_bytes(result.stderr)
    expected = b''.join(expected)
    difference = next((at for at, (actual, target) in enumerate(zip(result.stdout, expected))
        if actual != target), None)
    exact = result.returncode == 0 and result.stdout == expected and not result.stderr
    report = dict(host_exe_sha256=manifest['host_exe_sha256'],
        source_sha256=manifest['source_sha256'],
        reference_sha256=hashlib.sha256(reference).hexdigest(),
        cases=count, expected_output_bytes=len(expected), actual_output_bytes=len(result.stdout),
        host_returncode=result.returncode, exact_bits=exact,
        first_differing_case=None if difference is None else difference//16,
        execution_scope='Allowlisted original x86 math in no-CRT host, explicit field stub; no retail process.',
        native_control_word_established=False)
    (args.out/'report.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))
    assert exact, 'bounded x86 host differs from existing turn goldens'


if __name__ == '__main__':
    main()
