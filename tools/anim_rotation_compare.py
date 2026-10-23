"""Compare already-produced ROT100 native/Rust projections; never execute."""
import argparse
from collections import Counter
from fractions import Fraction
import json
from pathlib import Path
import struct

import anim_quaternion_probe as quat
import anim_rotation_native_probe as native
from anim_rotation_native_run import inspect
from native_x87_probe import operand, quantize, commit


def expected_fraction(request, neighbors):
    previous, left, right, following = neighbors
    if left == right:
        return 0
    frame, count, duration, flags, cw, *rest = struct.unpack_from('<fIf5I', request)
    frame_bits = struct.unpack_from('<I', request)[0]
    lf = struct.unpack_from('<I', request, 32+left*120)[0]
    rf = struct.unpack_from('<I', request, 32+right*120)[0]
    precision, mode = {0: 24, 0x200: 53, 0x300: 64}[cw & 0x300], (cw >> 10) & 3
    difference = operand(frame_bits)-lf
    negative_zero = difference == 0 and (mode == 1 or frame_bits >> 31 and lf == 0)
    numerator = quantize(difference, precision, mode, negative_zero)[0]
    result = quantize(numerator/Fraction(rf-lf), precision, mode, negative_zero)[0]
    return commit(result, mode, negative_zero)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-dir', type=Path, required=True)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--rust', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--allow-large-finite-exclusions', action='store_true',
        help='Explicitly reproduce the earlier bounded-API comparison; default requires zero arithmetic exclusions.')
    args = parser.parse_args()
    data = (args.input_dir/'inputs.bin').read_bytes()
    meta = json.loads((args.input_dir/'input-index.json').read_text())
    manifest = json.loads((args.input_dir/'manifest.json').read_text())
    assert quat.digest(data) == manifest['input_sha256']
    original, rust = args.native.read_bytes(), args.rust.read_bytes()
    summary = inspect(data, original)
    assert len(meta) == len(data)//native.INPUT_BYTES
    assert len(rust) == len(meta)*48
    counts, excluded, problems = Counter(), [], []
    projected, rust_projected = bytearray(), bytearray()
    for n, item in enumerate(meta):
        request = data[n*native.INPUT_BYTES:(n+1)*native.INPUT_BYTES]
        response = original[n*native.OUTPUT_BYTES:(n+1)*native.OUTPUT_BYTES]
        words = struct.unpack_from('<12I', rust, n*48)
        frame, count, duration, flags, cw, blend, enabled, reserved = struct.unpack_from('<fIf5I', request)
        neighbors = struct.unpack_from('<4I', response, 208)
        expected_present = count != 0 and (neighbors[1] == neighbors[2] or enabled)
        if words[0] == 2:
            # This cohort deliberately contains a clearly named large-finite
            # family that crosses the accepted primitive's finite-dot domain.
            # Do not let arbitrary typed errors silently pass as exclusions.
            if (not args.allow_large_finite_exclusions or item['family'] != 'large-finite'
                    or words[11] != 4 or not expected_present):
                problems.append(dict(row=n, reason='unanticipated arithmetic exclusion', input=item,
                                     rust_words=[f'{v:08X}' for v in words]))
            values = struct.unpack_from('<5I', response, 100)
            excluded.append(dict(row=n, source_window=item['source'], family=item['family'],
                error_code=words[11], native_result_finite=all(v & 0x7F800000 != 0x7F800000 for v in values),
                native_values=[f'{v:08X}' for v in values]))
            counts['explicit_arithmetic_exclusions'] += 1
            continue
        rust_projected += rust[n*48:(n+1)*48]
        if words[0] != int(expected_present):
            problems.append(dict(row=n, reason='presence differs', input=item, rust_kind=words[0]))
        if words[6:10] != neighbors:
            problems.append(dict(row=n, reason='native neighbors differ', native=neighbors, rust=words[6:10]))
        if expected_present:
            values = struct.unpack_from('<5I', response, 100)
            fraction = expected_fraction(request, neighbors)
            expected = (1, *values, *neighbors, fraction, 0)
            if words != expected:
                problems.append(dict(row=n, reason='sample bits differ', input=item,
                    expected=[f'{v:08X}' for v in expected], actual=[f'{v:08X}' for v in words]))
            counts['sampled_source_windows' if item['source'] else 'sampled_synthetic'] += 1
            counts['copied' if neighbors[1] == neighbors[2] else 'interpolated'] += 1
            projected += struct.pack('<12I', *expected)
        else:
            expected = (0, 0, 0, 0, 0, 0, *neighbors, 0, 0)
            if words != expected:
                problems.append(dict(row=n, reason='absent projection differs', input=item,
                    expected=[f'{v:08X}' for v in expected], actual=[f'{v:08X}' for v in words]))
            counts['absent'] += 1
            projected += struct.pack('<12I', *expected)
    report = dict(schema='native-rot100-curve-projection-comparison-v1', rows=len(meta),
        compared_rows=len(meta)-len(excluded), explicit_arithmetic_exclusions=len(excluded),
        exact_for_compared_rows=not problems, problems=problems, counts=dict(counts), excluded_rows=excluded,
        input_sha256=quat.digest(data), native_output_sha256=quat.digest(original),
        rust_output_sha256=quat.digest(rust), compared_native_projection_sha256=quat.digest(projected),
        compared_rust_projection_sha256=quat.digest(rust_projected),
        arithmetic_exclusion_policy='explicit earlier large-finite bounded-API exclusions' if args.allow_large_finite_exclusions else 'none permitted',
        native_structural_checks=summary,
        scope='Whole source INFO+ROT100 branch projection under explicit bounded policies. Raw100-byte endpoint copies and source index/INFO/neighbor metadata checked independently. Fractions checked by exact rational PC/RC oracle. Arithmetic exclusions are forbidden by default; the earlier bounded-API mode requires an explicit flag and retains every excluded native result. No full source/runtime animation claim.')
    args.report.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k: v for k, v in report.items() if k not in {'problems', 'excluded_rows', 'native_structural_checks'}}, indent=2))
    if problems:
        raise AssertionError(problems[:5])


if __name__ == '__main__':
    main()
