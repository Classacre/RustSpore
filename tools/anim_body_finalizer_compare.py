"""Stream three-way finalizer projections and all fresh source-intake digests."""
import argparse
import hashlib
import json
from pathlib import Path
from anim_body_finalizer_native_probe import FIELDS,RECORD_BYTES,PROJECTION_BYTES
from anim_body_optional_native_run import sha,save_json

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-dir',type=Path,required=True);parser.add_argument('--native-run',type=Path,required=True)
    parser.add_argument('--rust-output',type=Path,required=True);parser.add_argument('--rust-source-digest',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True);args=parser.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    prepared=json.loads((args.input_dir/'report.json').read_text());run=json.loads((args.native_run/'run.json').read_text())
    assert run['exact'] and not run['problem_count'] and run['rows']==prepared['rows']
    paths=[args.input_dir/'python-output.bin',args.native_run/'native-projection.bin',args.rust_output]
    assert sha(paths[0])==prepared['expected_projection_sha256'] and sha(paths[1])==run['projection_sha256']
    assert all(path.stat().st_size==prepared['rows']*PROJECTION_BYTES for path in paths)
    problems=[];mismatches=0
    with paths[0].open('rb') as python,paths[1].open('rb') as native,paths[2].open('rb') as rust:
        for row in range(prepared['rows']):
            values=[stream.read(PROJECTION_BYTES) for stream in [python,native,rust]]
            if not values[0]==values[1]==values[2]:
                mismatches+=1
                if len(problems)<20:
                    first=next(i for i in range(PROJECTION_BYTES) if not values[0][i]==values[1][i]==values[2][i])
                    detail=dict(row=row,byte=first,python=values[0][first],native=values[1][first],rust=values[2][first])
                    if first>=32:
                        detail['block']=(first-32)//RECORD_BYTES;offset=(first-32)%RECORD_BYTES
                        detail['field']='capabilities' if offset<168 else f'{FIELDS[(offset-168)//4]:03X}'
                    problems.append(detail)
    source=args.input_dir/'source-digests.txt';assert sha(source)==prepared['source_digest_sha256']
    expected_lines=source.read_text().splitlines();rust_lines=args.rust_source_digest.read_text().splitlines()
    assert len(expected_lines)==4095 and len(expected_lines)==len(set(expected_lines))
    source_exact=sorted(expected_lines)==sorted(rust_lines) and len(rust_lines)==len(set(rust_lines))
    normalized='\n'.join(sorted(rust_lines))+'\n'
    report=dict(schema='native-body-finalizer-fields-three-way-comparison-v1',exact=not mismatches and source_exact,
        rows=prepared['rows'],record_bytes=RECORD_BYTES,projection_bytes=PROJECTION_BYTES,
        problem_count=mismatches,problems=problems,python_projection_sha256=sha(paths[0]),
        native_projection_sha256=sha(paths[1]),rust_projection_sha256=sha(paths[2]),protocol_checks=run['protocol_checks'],
        fresh_source_intake=dict(resources=1365,source_rows=46418,contexts=3,records=4095,
            expected_admissions=prepared['source_admission_contexts'],exact=source_exact,
            expected_sha256=prepared['source_digest_sha256'],rust_sha256=hashlib.sha256(normalized.encode()).hexdigest()),
        source_input_scope=prepared['scope'],native_execution_scope=run['scope'],original_bytes_executed=True)
    save_json(args.out,report);print(json.dumps(report,indent=2));assert report['exact']

if __name__=='__main__':main()
