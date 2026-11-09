"""Compare an existing reply file with the pinned synthetic protocol; no process launch.

Comparison is meaningful only after an independently reviewed native host and
runner have produced a provenance-bound reply. This utility authenticates the
input against the generated corpus, not the origin of the reply.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
import anim_instance_handoff_protocol_v1 as protocol

INPUT_SHA = '9333b85c9b0e3d299d0819896e127def44287ab1133d2a49efe38f3921a7b9d7'
EXPECTED_SHA = 'e39a51c62262edc712442c8df63a0ddceb291457603dac65d01f3be5a8fab9e2'

def sha(data):
    return hashlib.sha256(data).hexdigest()

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--input', type=Path, required=True)
    ap.add_argument('--reply', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    args.out.resolve().relative_to((ROOT / 'target/animation').resolve())
    requests = args.input.read_bytes()
    replies = args.reply.read_bytes()
    assert sha(requests) == INPUT_SHA and len(requests) == 324 * protocol.REQUEST.size
    assert len(replies) == 324 * protocol.REPLY.size
    computed = b''.join(protocol.REPLY.pack(*protocol.expected(row))
                        for row in protocol.REQUEST.iter_unpack(requests))
    assert sha(computed) == EXPECTED_SHA
    mismatches = []
    for index, (want, got) in enumerate(zip(protocol.REPLY.iter_unpack(computed), protocol.REPLY.iter_unpack(replies))):
        if want != got:
            mismatches.append({'case': index, 'expected': want, 'actual': got})
    report = {'schema': 'instance-handoff-comparison-v1', 'input_sha256': sha(requests),
              'expected_sha256': sha(computed), 'reply_sha256': sha(replies),
              'cases': 324, 'mismatch_count': len(mismatches), 'mismatches': mismatches,
              'native_origin_authenticated': False,
              'limits': 'Byte comparison only; reply provenance and independent host review are separate.'}
    with args.out.open('x', encoding='utf-8') as f:
        json.dump(report, f, indent=2)
        f.write('\n')
    print(json.dumps({'cases': 324, 'mismatch_count': len(mismatches), 'native_origin_authenticated': False}))
    if mismatches:
        raise SystemExit(1)

if __name__ == '__main__':
    main()
