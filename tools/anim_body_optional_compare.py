"""Stream all native/Python/Rust optional-foot fields and fresh source digests."""
import argparse
import hashlib
import json
from pathlib import Path

from anim_body_optional_native_probe import FIELDS, PROJECTION_BYTES, RECORD_BYTES


def sha(path):
    result=hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(1<<20),b''):result.update(block)
    return result.hexdigest()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--input-dir',type=Path,required=True);p.add_argument('--native-run',type=Path,required=True)
    p.add_argument('--rust-output',type=Path,required=True);p.add_argument('--rust-source-digest',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    a.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    prepared=json.loads((a.input_dir/'report.json').read_text());run=json.loads((a.native_run/'run.json').read_text())
    assert not prepared['source_exclusions'] and run['exact'] and not run['problems']
    expected_path=a.input_dir/'python-output.bin';native_path=a.native_run/'native-projection.bin'
    assert sha(expected_path)==prepared['expected_projection_sha256'] and sha(native_path)==run['projection_sha256']
    rows=prepared['rows'];assert run['rows']==rows
    assert all(path.stat().st_size==rows*PROJECTION_BYTES for path in [expected_path,native_path,a.rust_output])
    problems=[];problem_count=0
    with expected_path.open('rb') as expected,native_path.open('rb') as native,a.rust_output.open('rb') as rust:
        for row in range(rows):
            e,n,r=[stream.read(PROJECTION_BYTES) for stream in [expected,native,rust]]
            if not(e==n==r):
                problem_count+=1
                if len(problems)<20:
                    first=next(i for i in range(PROJECTION_BYTES) if not(e[i]==n[i]==r[i]))
                    detail=dict(row=row,byte=first,python=e[first],native=n[first],rust=r[first])
                    if first>=16:
                        detail['block']=(first-16)//RECORD_BYTES;at=(first-16)%RECORD_BYTES
                        detail['field']='capabilities' if at<168 else f'{FIELDS[(at-168)//4]:03X}'
                    problems.append(detail)
    source_path=a.input_dir/'source-digests.txt';assert sha(source_path)==prepared['source_digest_sha256']
    expected_lines=source_path.read_text().splitlines();rust_lines=a.rust_source_digest.read_text().splitlines()
    assert len(expected_lines)==8190 and len(expected_lines)==len(set(expected_lines))
    source_exact=sorted(expected_lines)==sorted(rust_lines) and len(rust_lines)==len(set(rust_lines))
    normalized='\n'.join(sorted(rust_lines))+'\n'
    report=dict(schema='native-optional-foot-three-way-comparison-v2',exact=not problem_count and source_exact,
        rows=rows,source_native_contexts=prepared['statistics']['source_call_gate'],
        synthetic_contexts=rows-prepared['statistics']['source_call_gate'],record_bytes=RECORD_BYTES,
        projection_bytes=PROJECTION_BYTES,problem_count=problem_count,problems=problems,arithmetic_exclusions=0,
        native_projection_sha256=sha(native_path),python_projection_sha256=sha(expected_path),rust_projection_sha256=sha(a.rust_output),
        protocol_checks=run['protocol_checks'],host_exe_sha256=run['host_exe_sha256'],source_executable_sha256=prepared['source_sha256'],
        fresh_source_intake=dict(resources=1365,source_rows=46418,contexts=3,initial_flags=2,expected_records=8190,
            rust_records=len(rust_lines),exact=source_exact,expected_sha256=prepared['source_digest_sha256'],
            rust_sha256=hashlib.sha256(normalized.encode()).hexdigest()),output_counts=prepared['output_counts'],
        source_input_scope=prepared['scope'],
        native_execution_scope=dict(original_bytes_executed=True,
            description=run['scope'],
            execution='Ten reviewed original spans in an owned hidden bounded host; no original game process, modules, GUI or network.',
            unclaimed='Complete constructor padding/debug names, allocator behavior, caller DF/nonvolatile restoration, full finalization and animation binding.'))
    with a.out.open('x') as stream:json.dump(report,stream,indent=2);stream.write('\n')
    print(json.dumps(report,indent=2));assert report['exact']


if __name__=='__main__':main()
