"""Generate E55B20/E557A0 goldens in the reviewed bounded original-x86 host.

Requires a successful turn-mode protocol cross-check for the exact host SHA.
No source range is extended. Host/input/output bytes stay in ignored target/.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct
import subprocess


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def parameter(values, mode, control, hint):
    raw = bytearray(100)
    struct.pack_into('<IHH', raw, 0, mode, control, 0)
    # direction, position, current, target, speed, factor6, heading-test7
    struct.pack_into('<15f', raw, 8, *values[:15])
    struct.pack_into('<I', raw, 68, hint)
    struct.pack_into('<7f', raw, 72, *values[15:])
    return bytes(raw)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--host-dir', type=Path, required=True)
    ap.add_argument('--turn-check', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    args.host_dir.resolve().relative_to(target)
    args.out.resolve().relative_to(target)
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    host = (args.host_dir/'host-x86.exe').resolve()
    assert hashlib.sha256(host.read_bytes()).hexdigest() == manifest['host_exe_sha256']
    turn = json.loads(args.turn_check.read_text())
    assert turn['exact_bits'] and turn['host_returncode'] == 0 and turn['cases'] == 12924
    assert turn['host_exe_sha256'] == manifest['host_exe_sha256']
    assert [(f['start'],f['end_exclusive']) for f in manifest['code_allowlist']] == [
        ('00E51230','00E51290'), ('00E55650','00E55795'), ('00E557A0','00E55B12'),
        ('00E55B20','00E55E3D'), ('00E55E40','00E55FA2'), ('01041E40','01041E57')]
    cases = []
    for distance in [0., .25, .5, f32(1.-2.**-24), 1., f32(1.+2.**-23), 3., 4.]:
        for direction in [[1., 0., 0.], [0., 1., 0.], [-1., 0., 0.]]:
            for current in [[0., 0., 0.], [10., 0., 0.]]:
                for hint in [0, 1, 2, 255, 256, 0xffffffff]:
                    for gate in [0., 2.**-16, 2.**-15]:
                        values = [*direction, 0., 0., 0., *current, distance, 0., 0.,
                            2., .75, gate, .25, 0., 0., 0., 0., 0., 0.]
                        cases.append((values, hint))
    for speed in [0., -0.]:
        cases.append(([float('nan')]*12+[speed]+[float('nan')]*6+[-0., 1., -2.], 256))
    rng = random.Random(0xe55b20)
    for _ in range(1024):
        size = 10.**rng.uniform(-12, 3)
        direction = [f32(rng.uniform(-2., 2.)) for _ in range(3)]
        position = [f32(rng.uniform(-size, size)) for _ in range(3)]
        current = [f32(rng.uniform(-10., 10.)) for _ in range(3)]
        destination = [f32(rng.uniform(-size, size)) for _ in range(3)]
        values = [*direction, *position, *current, *destination,
            f32(rng.uniform(-10., 10.)), f32(rng.uniform(-2., 2.)),
            rng.choice([0., 2.**-16, 2.**-15, 1.]), f32(rng.uniform(-3., 3.)),
            *[f32(rng.uniform(-10., 10.)) for _ in range(3)], 0., 0., 0.]
        cases.append((values, rng.choice([0, 1, 255, 256, 0xffffffff])))
    for tiny in [2.**-126, 2.**-140, 2.**-149]:
        cases.append(([tiny, -tiny, tiny, 0., -0., tiny, -tiny, tiny, 0., tiny, -tiny, tiny,
            tiny, tiny, 1., tiny, tiny, -tiny, 0., 0., 0., 0.], 1))
    records, policies = [], {}
    for mode in [1, 2]:
        for pc in [0, 0x200, 0x300]:
            for rc in [0, 0x400, 0x800, 0xc00]:
                control = 0x7f | pc | rc
                for values, hint in cases:
                    assert len(values) == 22
                    records.append(parameter(values, mode, control, hint))
                policies[f'mode{mode}_CW{control:04X}'] = len(cases)
    args.out.mkdir(parents=True, exist_ok=False)
    supplied = b''.join(records)
    (args.out/'input.bin').write_bytes(supplied)
    result = subprocess.run([str(host)], input=supplied, capture_output=True,
        creationflags=subprocess.CREATE_NO_WINDOW, timeout=60)
    (args.out/'output.bin').write_bytes(result.stdout)
    (args.out/'stderr.bin').write_bytes(result.stderr)
    assert result.returncode == 0 and not result.stderr
    assert len(result.stdout) == 16*len(records)
    fixture_records = []
    for ordinal, record in enumerate(records):
        output = result.stdout[16*ordinal:16*(ordinal+1)]
        status = struct.unpack_from('<I', output, 12)[0]
        assert status in [0, 1]
        assert struct.unpack_from('<I', record)[0] == 2 or status == 0
        fixture_records.append(record+output)
    fixture = args.out/'corpus.bin'
    fixture.write_bytes(b'RSCELLP1'+struct.pack('<I', len(records))+b''.join(fixture_records))
    report = dict(source_sha256=manifest['source_sha256'], host_exe_sha256=manifest['host_exe_sha256'],
        source_ranges_unchanged=True, original_x86_instructions_executed=True,
        synthetic_cases=len(records), cases_per_mode_and_control_word=policies,
        host_returncode=result.returncode, output_guards_intact=True,
        scope='Only allowlisted original E55B20/E557A0 and helpers in reviewed no-CRT fixture host; explicit environment stub; no game initializer or retail process.',
        native_control_word_established=False, controlled_mxcsr='00001F80',
        fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest())
    (args.out/'inventory.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
