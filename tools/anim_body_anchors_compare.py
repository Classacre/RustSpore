"""Compare retained source-instruction anchor digests with fresh Rust reads."""
import argparse
import hashlib
import itertools
import json
from pathlib import Path


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference-dir',type=Path,required=True)
    parser.add_argument('--rust-digest',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();root=Path(__file__).resolve().parent.parent
    args.out.resolve().relative_to((root/'target').resolve())
    report_path=args.reference_dir/'report.json';report_bytes=report_path.read_bytes()
    reference=json.loads(report_bytes)
    assert reference['schema']=='canonical-source-body-anchor-instruction-oracle-v1'
    assert not reference['failures'] and reference['records']==4095
    assert reference['statistics']==dict(package_indexes=82,evaluated_rows=139254,resources=1365,source_rows=46418)
    expected=(args.reference_dir/'source-digests.txt').read_bytes()
    assert hashlib.sha256(expected).hexdigest()==reference['normalized_digest_sha256']
    rust=args.rust_digest.read_text().splitlines()
    assert len(rust)==len(set(rust)), 'duplicate Rust digest rows'
    rust='\n'.join(sorted(rust))+'\n'
    problems=[dict(line=i+1,expected=a,rust=b) for i,(a,b) in
        enumerate(itertools.zip_longest(expected.decode().splitlines(),rust.splitlines())) if a!=b]
    result=dict(schema='canonical-source-body-anchor-rust-comparison-v1',exact=not problems,
        source_resources=1365,source_rows=46418,contexts=3,evaluated_rows=139254,
        expected_records=4095,rust_records=len(rust.splitlines()),arithmetic_exclusions=0,
        expected_sha256=hashlib.sha256(expected).hexdigest(),rust_sha256=hashlib.sha256(rust.encode()).hexdigest(),
        reference_report_sha256=hashlib.sha256(report_bytes).hexdigest(),source_sha256=reference['source_sha256'],
        anchor_regions=reference['anchor_regions'],problem_count=len(problems),problems=problems[:20],
        scope='Fresh canonical Rust source intake versus independent original-instruction anchor oracle. Explicit x87 contexts andSSE1F80; no original-byte host or optional append/finalized hierarchy/runtime animation claim.')
    with args.out.open('x') as stream:json.dump(result,stream,indent=2);stream.write('\n')
    print(json.dumps(result,indent=2))
    if problems:raise SystemExit(1)


if __name__=='__main__':main()
