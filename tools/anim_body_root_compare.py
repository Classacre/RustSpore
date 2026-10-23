"""Compare native/Python/Rust root projections and fresh canonical source intake."""
import argparse
import hashlib
import json
from pathlib import Path

from anim_body_root_native_probe import PROJECTION_BYTES


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--input-dir', type=Path, required=True)
    p.add_argument('--native-run', type=Path, required=True)
    p.add_argument('--rust-output', type=Path, required=True)
    p.add_argument('--rust-source-digest', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    manifest = json.loads((a.input_dir/'manifest.json').read_text())
    run = json.loads((a.native_run/'run.json').read_text())
    expected = (a.input_dir/'python-output.bin').read_bytes()
    native = (a.native_run/'native-projection.bin').read_bytes()
    rust = a.rust_output.read_bytes()
    assert sha(expected)==manifest['hashes']['python-output.bin']
    assert sha(native)==run['projection_sha256']
    rows=manifest['rows']
    assert len(expected)==len(native)==len(rust)==rows*PROJECTION_BYTES
    problems = [i for i in range(rows) if not (expected[i*PROJECTION_BYTES:(i+1)*PROJECTION_BYTES]
        ==native[i*PROJECTION_BYTES:(i+1)*PROJECTION_BYTES]
        ==rust[i*PROJECTION_BYTES:(i+1)*PROJECTION_BYTES])]
    expected_source = (a.input_dir/'source-input-digests.txt').read_text().splitlines()
    rust_source = a.rust_source_digest.read_text().splitlines()
    assert len(expected_source)==1365
    source_exact = sorted(expected_source)==sorted(rust_source)
    report = dict(schema='native-body-root-selection-three-way-comparison-v1',rows=rows,
        source_contexts=manifest['statistics']['source_constructor_no_optional_driver'],
        synthetic_contexts=rows-manifest['statistics']['source_constructor_no_optional_driver'],
        exact=not problems and source_exact,problems=problems,arithmetic_exclusions=0,
        native_projection_sha256=sha(native),python_projection_sha256=sha(expected),rust_projection_sha256=sha(rust),
        native_protocol_checks=run['protocol_checks'],
        fresh_source_intake=dict(resources=1365,rows=46418,rust_resources=len(rust_source),exact=source_exact,
            source_geometry_context='027F/MXCSR1F80',expected_sha256=sha(('\n'.join(sorted(expected_source))+'\n').encode()),
            rust_sha256=sha(('\n'.join(sorted(rust_source))+'\n').encode())),
        source_dispositions={k:manifest['statistics'][k] for k in ['AdmittedWithoutOptionalConstruction',
            'OptionalFootConstructionRequired','InitialRootMissingNonzeroSpine']},
        source_executable_sha256=manifest['source_sha256'],host_exe_sha256=run['host_exe_sha256'],
        scope=manifest['scope'])
    with a.out.open('x') as out: json.dump(report,out,indent=2);out.write('\n')
    print(json.dumps(report,indent=2))
    assert report['exact']


if __name__=='__main__':
    main()
