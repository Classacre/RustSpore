"""Extract a bounded, caller-selected Cell RW4 hierarchy fixture; never runs native code.

The A=clip, B=model assignment is a hypothesis, not a runtime association.
Outputs are newly created under target/ and contain only normalized integers.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

from dbpf_extract import payload
from gmdl_skin_probe import indexes
from rw4_animation_probe import sections

ROOT = Path(__file__).resolve().parent.parent
PACKAGE_SHA = 'e01bb86d8ddf02c56d52394b4e1db7181c1e1ebe1878bf6fb72c4ee523fcedd1'
TYPE = 0x2f4e681b
SPEC = {
    # Legacy wide-index fields: apparent group is the preceding metadata word;
    # apparent 64-bit instance is native scope concatenated with instance-low.
    'A_clip': (0x0001ffff, 0x4060600060bdd177, 17, 40,
               'c20051c08236501764cb052d36c74592ff005b7fe26bcd0976a59d9a4775e3b2'),
    'B_model': (0x00010000, 0x00000000dae8ce17, 10, 0,
                'bbd99a741d41a3606a310f242e14e5f85dc7b49af454972ff02124b16e318592'),
}

def digest(b):
    return hashlib.sha256(b).hexdigest()

def file_digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read_words(data, offset, count, fmt):
    if offset < 0 or count < 0 or offset + count * 4 > len(data):
        raise ValueError('array exceeds payload')
    return list(struct.unpack_from('<' + fmt * count, data, offset))

def extract(package, entries, name, spec):
    group, instance, binding, expected_skeleton, expected_sha = spec
    found = [e for e in entries if e['type'] == TYPE and e['group'] == group and e['instance'] == instance]
    if len(found) != 1:
        raise ValueError(f'{name}: exact resource must be unique')
    e = found[0]
    if max(e['stored'], e['dec']) > 32 * 1024 * 1024:
        raise ValueError(f'{name}: resource too large')
    data = payload(package, e)
    if len(data) != e['dec'] or digest(data) != expected_sha:
        raise ValueError(f'{name}: payload size or hash mismatch')
    ss = {i: (t, at, size) for i, t, at, size, _ in sections(data)}
    if len(ss) > 4096 or binding not in ss:
        raise ValueError(f'{name}: invalid section table')
    typ, off, size = ss[binding]
    if typ != 0x7000c or size < 20 or off + 20 > len(data):
        raise ValueError(f'{name}: bad binding')
    words = read_words(data, off, 5, 'I')
    if words[3] != expected_skeleton or expected_skeleton not in ss:
        raise ValueError(f'{name}: skeleton reference mismatch')
    typ, off, size = ss[expected_skeleton]
    if typ != 0x70002 or size < 24 or off + 24 > len(data):
        raise ValueError(f'{name}: bad skeleton')
    flags, parents, ids, n, skeleton_id, skeleton_flags = read_words(data, off, 6, 'I')
    if n != 9 or skeleton_id != 0x379cc22e:
        raise ValueError(f'{name}: unexpected skeleton')
    parent_values = read_words(data, parents, n, 'i')
    id_values = read_words(data, ids, n, 'I')
    read_words(data, flags, n, 'I')
    if len(set(id_values)) != n or any(v < -1 or v >= i for i, v in enumerate(parent_values)):
        raise ValueError(f'{name}: invalid IDs or hierarchy')
    return parent_values, id_values, dict(dbpf_type=f'{TYPE:08X}', legacy_group_metadata_field=f'{group:08X}',
        legacy_scoped_instance=f'{instance:016X}',
        native_key_instance_type_scope=f'{instance & 0xffffffff:08X}:{TYPE:08X}:{instance >> 32:08X}',
        payload_sha256=digest(data), binding_section=binding,
        skeleton_section=expected_skeleton, skeleton_id=f'{skeleton_id:08X}', skeleton_flags=skeleton_flags,
        parent_sha256=digest(struct.pack('<' + 'i'*n, *parent_values)),
        ids_sha256=digest(struct.pack('<' + 'I'*n, *id_values)), count=n)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('package', type=Path)
    parser.add_argument('--out', required=True, type=Path, help='new directory under target')
    args = parser.parse_args()
    target = (ROOT / 'target').resolve(strict=True)
    out = args.out.resolve()
    if out == target or not out.is_relative_to(target):
        raise ValueError('output must be a new directory under target')
    before = file_digest(args.package)
    if before != PACKAGE_SHA:
        raise ValueError('package hash mismatch')
    entries = indexes(args.package)
    a, ai, am = extract(args.package, entries, 'A_clip', SPEC['A_clip'])
    b, bi, bm = extract(args.package, entries, 'B_model', SPEC['B_model'])
    if a != b or ai != bi:
        raise ValueError('candidate skeleton identity or hierarchy mismatch')
    mapping = [next((j for j in range(len(bi)-1, -1, -1) if bi[j] == x), -1) for x in ai]
    if any(x < 0 for x in mapping):
        raise ValueError('candidate ID mismatch')
    if file_digest(args.package) != before:
        raise ValueError('package changed during inspection')
    arrays = struct.pack('<' + 'i'*(len(a)+len(b)+len(mapping)), *(a+b+mapping))
    cases = {f'side-{side}.h930': b'H930' + struct.pack('<III', len(a), len(b), side) + arrays for side in (0, 1)}
    manifest = dict(schema='cell-hierarchy-source-case-v1', assignment='caller-selected A=clip, B=model; runtime association unverified',
        package_sha256=before, A_clip=am, B_model=bm, map_sha256=digest(struct.pack('<'+'i'*len(mapping), *mapping)),
        arrays_sha256=digest(arrays), files={name:digest(data) for name,data in cases.items()},
        limitations='source selection, animation retargeting and pose are unverified; output contains normalized integers only')
    out.mkdir(parents=True, exist_ok=False)
    for name, data in cases.items():
        with (out / name).open('xb') as f:
            f.write(data)
    with (out / 'manifest.json').open('x', encoding='utf-8', newline='\n') as f:
        json.dump(manifest, f, indent=2)
        f.write('\n')
    print(json.dumps(manifest))

if __name__ == '__main__':
    main()
