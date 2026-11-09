"""Prepare bounded reference-handoff cases and expected callback observations.

This is a synthetic comparison protocol, not execution of original instructions.
Each object domain owns separate IDs 1 and 2; zero is null. Refcount words
start at 41 and 73 in each resource/binding domain. Callback providers only
record observations and return; they do not implement source callees.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parent.parent
REQUEST = struct.Struct('<6I')
REPLY = struct.Struct('<20I')

def sha(data):
    return hashlib.sha256(data).hexdigest()

def expected(row):
    old_resource, resource, old_owner, owner, old_binding, binding = row
    assert old_resource in range(3) and resource in (1, 2)
    assert old_owner in range(3) and owner in range(3)
    assert old_binding in range(3) and binding in (1, 2)
    resource_counts = [41, 73]
    binding_counts = [41, 73]
    events = []
    if resource != old_resource:
        resource_counts[resource - 1] += 1
        if old_resource:
            events.append((1, old_resource, resource))  # 99CD90, stored resource
    if owner != old_owner:
        if owner:
            events.append((2, owner, old_owner))  # A16FC0, before owner store
        if old_owner:
            events.append((3, old_owner, owner))  # 9C4B50, after owner store
    if binding != old_binding:
        binding_counts[binding - 1] += 1
        if old_binding:
            events.append((4, old_binding, binding))  # 99EDE0, after binding store
    assert len(events) <= 4
    flat = [resource, owner, binding, *resource_counts, *binding_counts, len(events)]
    for event in events:
        flat.extend(event)
    flat.extend([0] * (20 - len(flat)))
    return tuple(flat)

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    args.out.resolve().relative_to((ROOT / 'target/animation').resolve())
    cases = list(itertools.product(range(3), (1, 2), range(3), range(3), range(3), (1, 2)))
    assert len(cases) == 324 and len(set(cases)) == 324
    inputs = b''.join(REQUEST.pack(*row) for row in cases)
    reference = b''.join(REPLY.pack(*expected(row)) for row in cases)
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / 'input.bin').write_bytes(inputs)
    (args.out / 'expected.bin').write_bytes(reference)
    manifest = {
        'schema': 'instance-handoff-protocol-v1', 'cases': len(cases),
        'input_row_bytes': REQUEST.size, 'reply_row_bytes': REPLY.size,
        'input_sha256': sha(inputs), 'expected_sha256': sha(reference),
        'generator_sha256': sha(Path(__file__).read_bytes()),
        'native_executed': False,
        'request_fields': ['old_resource', 'resource', 'old_owner', 'owner', 'old_binding', 'binding'],
        'reply_fields': ['resource', 'owner', 'binding', 'resource1_count', 'resource2_count',
                         'binding1_count', 'binding2_count', 'event_count', 'four kind/id/observed_slot triples'],
        'provider_kinds': {'1': '99CD90', '2': 'A16FC0', '3': '9C4B50', '4': '99EDE0'},
        'domain': 'disjoint object domains; non-null resource and supplied binding; stable slots; non-mutating providers',
        'limits': 'Expected rows are an owned model; they are not native observations or a review of the host.'}
    (args.out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'cases': len(cases), 'input_sha256': sha(inputs), 'expected_sha256': sha(reference)}))

if __name__ == '__main__':
    main()
