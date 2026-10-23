"""Read-only original/Python/Rust capability comparison; never execute a host."""
import argparse
import hashlib
import json
from pathlib import Path

from anim_body_capabilities_native_run import projection


def digest(value):
    return hashlib.sha256(value).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--input-dir', type=Path, required=True)
    p.add_argument('--native-run', type=Path, required=True)
    p.add_argument('--rust', type=Path, required=True)
    p.add_argument('--rust-source-digests', type=Path)
    p.add_argument('--report', type=Path, required=True)
    a = p.parse_args()
    manifest = json.loads((a.input_dir/'manifest.json').read_text())
    native_report = json.loads((a.native_run/'run.json').read_text())
    data, expected, rust = [(a.input_dir/'inputs.bin').read_bytes(),
                            (a.input_dir/'python-output.bin').read_bytes(), a.rust.read_bytes()]
    assert digest(data) == manifest['input_sha256'] == native_report['input_sha256']
    assert digest(expected) == manifest['python_output_sha256']
    native_output = (a.native_run/'output.bin').read_bytes()
    assert digest(native_output) == native_report['native_output_sha256']
    native, controls = projection(data, native_output)
    rows = manifest['rows']
    assert len(expected) == len(native) == len(rust) == rows*168
    index = json.loads((a.input_dir/'input-index.json').read_text())
    assert len(index) == rows
    problems = []
    for row in range(rows):
        begin, end = row*168, (row+1)*168
        original, python, actual = native[begin:end], expected[begin:end], rust[begin:end]
        if original != python or original != actual:
            problems.append(dict(row=row, input=index[row], native=original.hex(),
                                 python=python.hex(), rust=actual.hex()))
    source_comparison = None
    if a.rust_source_digests:
        python = (a.input_dir/'python-source-digests.txt').read_bytes()
        assert digest(python) == manifest['source_digests_sha256']
        fresh_rust = a.rust_source_digests.read_bytes()
        source_comparison = dict(source_resources=manifest['source_resources'],
            fresh_rust_resources=len(fresh_rust.splitlines()), exact=python == fresh_rust,
            expected_sha256=digest(python), rust_sha256=digest(fresh_rust))
        if not source_comparison['exact']:
            problems.append(dict(reason='fresh source intake digests differ'))
    report = dict(schema='native-body-capabilities-three-way-comparison-v1', rows=rows,
        source_rows=manifest['counts']['source_rows'], synthetic_rows=manifest['counts']['synthetic_rows'],
        exact=not problems, problems=problems, arithmetic_exclusions=0,
        input_sha256=digest(data), native_projection_sha256=digest(native),
        python_projection_sha256=digest(expected), rust_projection_sha256=digest(rust),
        fresh_source_intake_comparison=source_comparison, controls=controls,
        aggregate_table_sha256=manifest['canonical_aggregate_table_sha256'],
        all_three_source_aggregate_tables_identical=manifest['aggregate_tables_identical'],
        scope='Exact168-byte32-slot capability preparation including stale number bytes and both native masks. Native guard/state checks rechecked. Fresh Rust source adapter validation is separate and explicit; no finalized rig/hierarchy or runtime pose claim.')
    a.report.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    with a.report.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({key: value for key, value in report.items() if key not in {'problems', 'controls'}}, indent=2))
    if problems:
        raise AssertionError(problems[:2])


if __name__ == '__main__':
    main()
