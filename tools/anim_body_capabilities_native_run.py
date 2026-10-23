"""Execute only a separately reviewed pinned capability host through hidden pipes."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import time

import anim_body_capabilities_native_probe as probe
from anim_body_capabilities_fixture import request
from anim_rotation_native_run import saved_run
from cell_schema_probe import SourceImage, SOURCE_SHA256

HOST = '187db8dea46679a0cb25dd0596828a604eb77bd23d66e8e620e7d23d5f39f253'
ASM = '40ce06078b813d8e6f4e0cd02556e089f6865d5cab5cf54b374bfe7e58f322aa'
INPUT = '47d1bd4aee83d7c11f4bb9a7b5b042dd04071e1b38bedc5ae0f73e2f17d7edb6'
EXPECTED = '582aca8c6e189c9d8385749d68798bb03efdd9bb8da0446f7ac3c01032c6b337'
ROWS = 58802


def digest(value):
    return hashlib.sha256(value).hexdigest()


def projection(data, output):
    assert len(data) % probe.INPUT_BYTES == 0
    assert len(output) == len(data)//probe.INPUT_BYTES*probe.OUTPUT_BYTES
    result = bytearray()
    controls = {}
    for n, at in enumerate(range(0, len(data), probe.INPUT_BYTES)):
        response = output[n*probe.OUTPUT_BYTES:(n+1)*probe.OUTPUT_BYTES]
        mxcsr, cw, sw = struct.unpack_from('<IHH', response, 168)
        assert mxcsr == 0x1F80 and cw == struct.unpack_from('<H', data, at)[0] and sw == 0
        assert response[176:] == bytes(16)
        controls[f'{cw:04X}'] = controls.get(f'{cw:04X}', 0)+1
        result.extend(response[:168])
    return bytes(result), controls


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir', type=Path, required=True)
    p.add_argument('--input-dir', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    fixture, inputs, out = a.host_dir.resolve(), a.input_dir.resolve(), a.out.resolve()
    for path in [fixture, inputs, out]:
        path.relative_to(target)
    manifest_bytes = (fixture/'manifest.json').read_bytes()
    manifest = json.loads(manifest_bytes)
    input_manifest_bytes = (inputs/'manifest.json').read_bytes()
    input_manifest = json.loads(input_manifest_bytes)
    assert manifest['schema'] == 'bounded-original-body-capabilities-host-v1'
    assert manifest['host_exe_sha256'] == HOST and manifest['host_source_sha256'] == ASM
    assert input_manifest['input_sha256'] == INPUT and input_manifest['rows'] == ROWS
    assert input_manifest['python_output_sha256'] == EXPECTED
    for name, sha in [('host-x86.exe', HOST), ('host-x86.asm', ASM)]:
        assert digest((fixture/name).read_bytes()) == sha
    image = SourceImage(Path(manifest['source_path']))
    assert probe.audit_source(image) == manifest['code_allowlist']
    assert probe.linked_audit(fixture/'host-x86.exe', fixture/'host-x86.map') == manifest['linked_audit']
    assert probe.host_source(image) == (fixture/'host-x86.asm').read_text()
    data = (inputs/'inputs.bin').read_bytes()
    expected = (inputs/'python-output.bin').read_bytes()
    assert digest(data) == INPUT and len(data) == ROWS*probe.INPUT_BYTES
    assert digest(expected) == EXPECTED and len(expected) == ROWS*168
    index = json.loads((inputs/'input-index.json').read_text())
    assert len(index) == ROWS
    out.mkdir(parents=True, exist_ok=False)
    (out/'reviewed-host-manifest.json').write_bytes(manifest_bytes)
    (out/'reviewed-input-manifest.json').write_bytes(input_manifest_bytes)
    started = time.perf_counter()
    host = fixture/'host-x86.exe'
    for family in ['source', 'duplicate-max-tail-order', 'zero-member-stops-before-compaction',
                   'complete-expanded32', 'recursive-chain', 'unsigned-tree-keys']:
        n = next(i for i, entry in enumerate(index) if entry['family'] == family)
        record = data[n*probe.INPUT_BYTES:(n+1)*probe.INPUT_BYTES]
        result = saved_run(host, record, out, f'smoke-{family}')
        assert result.returncode == 0 and not result.stderr
        actual, _ = projection(record, result.stdout)
        assert actual == expected[n*168:(n+1)*168]
    empty = request([0]*32, [0]*32, {}, 0x027F)
    first = saved_run(host, empty, out, 'protocol-complete-reference')
    assert first.returncode == 0 and not first.stderr
    projection(empty, first.stdout)
    negatives = [('empty', b'', 0, b'')]
    for size in [1, 15, 16, probe.INPUT_BYTES-1]:
        negatives += [(f'partial{size}', empty[:size], 7, b''),
                      (f'complete_then_partial{size}', empty+empty[:size], 7, first.stdout)]
    for label, offset, value in [('reservedPC', 0, 0x017F), ('unmaskedCW', 0, 0x027E),
        ('reservedCWWord', 0, 0x0001027F), ('aggregateCount9', 4, 9),
        ('reserved8', 8, 1), ('reserved12', 12, 1), ('memberCount33', 180, 33)]:
        value_bytes = bytearray(empty)
        struct.pack_into('<I', value_bytes, offset, value)
        negatives.append((label, bytes(value_bytes), 7, b''))
    negatives += [('full_initial_without_zero', request([10]*32, [1]*32, {}, 0x027F), 7, b''),
        ('recursive_capacity', request([10]+[0]*31, [1]*32, {10: [10]}, 0x027F), 7, b''),
        ('direct_capacity', request(list(range(100, 131))+[0], [1]*32, {100: [1, 2]}, 0x027F), 7, b'')]
    repeated = bytearray(request([0]*32, [0]*32, {10: [], 20: []}, 0x027F))
    struct.pack_into('<I', repeated, 176+136, 10)
    negatives.append(('duplicate_key', bytes(repeated), 7, b''))
    struct.pack_into('<I', repeated, 176+136, 1)
    negatives.append(('unsorted_key', bytes(repeated), 7, b''))
    checks = out/'protocol'
    checks.mkdir()
    protocol = []
    for name, record, code, reply in negatives:
        result = saved_run(host, record, checks, name)
        assert result.returncode == code and result.stdout == reply and not result.stderr, name
        protocol.append(dict(case=name, exit=code, exact=True))
    (out/'protocol-checks.json').write_text(json.dumps(protocol, indent=2)+'\n')
    result = saved_run(host, data, out, 'corpus')
    assert result.returncode == 0 and not result.stderr
    actual, controls = projection(data, result.stdout)
    (out/'output.bin').write_bytes(result.stdout)
    (out/'native-projection.bin').write_bytes(actual)
    problems = [dict(row=n, expected=expected[n*168:(n+1)*168].hex(), actual=actual[n*168:(n+1)*168].hex())
                for n in range(ROWS) if actual[n*168:(n+1)*168] != expected[n*168:(n+1)*168]]
    for name, sha in [('host-x86.exe', HOST), ('host-x86.asm', ASM)]:
        assert digest((fixture/name).read_bytes()) == sha
    SourceImage(Path(manifest['source_path']))
    assert (fixture/'manifest.json').read_bytes() == manifest_bytes
    assert (inputs/'manifest.json').read_bytes() == input_manifest_bytes
    assert digest((inputs/'inputs.bin').read_bytes()) == INPUT
    assert digest((inputs/'python-output.bin').read_bytes()) == EXPECTED
    report = dict(schema='reviewed-native-body-capability-run-v1', rows=ROWS,
        source_rows=46418, source_resources=1365, synthetic_rows=12384, protocol_checks=len(protocol),
        input_sha256=INPUT, native_output_sha256=digest(result.stdout),
        native_projection_sha256=digest(actual), python_projection_sha256=EXPECTED,
        exact_python_match=not problems, problems=problems, controls=controls,
        elapsed_seconds=time.perf_counter()-started, host_exe_sha256=HOST, source_sha256=SOURCE_SHA256,
        original_sources_and_fixture_unchanged=True, hidden_binary_pipes=True,
        original_modules_loaded=False, retail_process_started=False, gui_used=False,
        scope='Exact four-function native32-slot preparation with supplied bounded aggregate tree, including every shipped source row. No source hierarchy/finalizer/animation pose or retail dictionary/FP policy inference.')
    (out/'run.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'problems'}, indent=2))
    if problems:
        raise AssertionError(problems[:2])


if __name__ == '__main__':
    main()
