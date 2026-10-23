"""Compare fresh canonical Rust source fields with pinned instruction-oracle digests."""
import argparse
import hashlib
import json
from pathlib import Path


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--reference-dir',type=Path,required=True)
    p.add_argument('--rust-digest',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    a=p.parse_args()
    reference_raw=(a.reference_dir/'report.json').read_bytes();reference=json.loads(reference_raw)
    expected_raw=(a.reference_dir/'source-digests.txt').read_bytes()
    expected=expected_raw.decode().splitlines();actual=a.rust_digest.read_text().splitlines()
    assert hashlib.sha256(expected_raw).hexdigest()==reference['source_digest_sha256']
    assert not reference['failures'] and len(expected)==4095
    exact=sorted(expected)==sorted(actual)
    report=dict(schema='canonical-source-body-fields-rust-comparison-v1',exact=exact,
        source_resources=1365,source_rows=46418,contexts=3,evaluated_rows=139254,
        expected_records=len(expected),rust_records=len(actual),arithmetic_exclusions=0,
        expected_sha256=reference['source_digest_sha256'],
        rust_sha256=hashlib.sha256(('\n'.join(sorted(actual))+'\n').encode()).hexdigest(),
        reference_report_sha256=hashlib.sha256(reference_raw).hexdigest(),source_sha256=reference['source_sha256'],
        head_regions=reference['head_regions'],tail_region=reference['tail_region'],
        scope='Exact bounded source-instruction oracle versus fresh canonical Rust reads for rest/extents/origin fields under three explicit x87 contexts and scopedMXCSR1F80. No original-byte host execution, optional-foot construction, full hierarchy or runtime animation claim.')
    with a.out.open('x') as out:json.dump(report,out,indent=2);out.write('\n')
    print(json.dumps(report,indent=2));assert exact


if __name__=='__main__':main()
