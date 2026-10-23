"""Prepare native root-selection inputs and an independent integer-SSE oracle."""
import argparse
from collections import Counter
import hashlib
import itertools
import json
from pathlib import Path
import random
import struct
import sys

from anim_body_hierarchy_probe import package_entries, constructor_inputs
from anim_body_pose_probe import arithmetic
from anim_body_source_probe import SourceOracle
from anim_body_root_native_probe import INPUT_BYTES, PROJECTION_BYTES
from cell_schema_probe import SourceImage, SOURCE_SHA256
from creature_header_probe import header
from dbpf_extract import payload

CAPABILITIES_SHA = '582aca8c6e189c9d8385749d68798bb03efdd9bb8da0446f7ac3c01032c6b337'
CONTROLS = [0x7F | p | r for p, r in itertools.product([0, 0x200, 0x300], [0, 0x400, 0x800, 0xC00])]


def bits(x):
    return struct.unpack('<I', struct.pack('<f', x))[0]


def value(x):
    return struct.unpack('<f', struct.pack('<I', x))[0]


def squared(center, z_first):
    squares = [arithmetic('mulss', x, x) for x in center]
    total = squares[0]
    for i in ([2, 1] if z_first else [1, 2]):
        total = 0x7F800000 if total == 0x7F800000 or squares[i] == 0x7F800000 else arithmetic('addss', total, squares[i])
    return total


def reference(blocks, policy):
    votes, candidates, highest, authored = [0]*len(blocks), [], 0, None
    for i, b in enumerate(blocks):
        if b['presence'] & 1:
            authored = i
            break
        if not b['nonzero'] & 4:
            continue
        ancestor = b['parent']
        while ancestor is not None and not blocks[ancestor]['nonzero'] & 8:
            ancestor = blocks[ancestor]['parent']
        assert ancestor is not None
        votes[ancestor] += 1
        if votes[ancestor] > highest:
            candidates, highest = [], votes[ancestor]
        if votes[ancestor] == highest:
            candidates.append(ancestor)
    selected, branch = None, 'initial'
    if policy & 1:
        best = value(0x7F7FFFFF)
        unrolled_end = len(blocks)//4*4
        for i, b in enumerate(blocks):
            if b['presence'] & 8:
                distance = value(squared(b['center'], i >= unrolled_end or i % 4 == 1))
                if distance < best:
                    best, selected = distance, i
        if selected is not None:
            branch = 'nearest'
    if selected is None and authored is not None:
        selected, branch = authored, 'authored'
    if selected is None and candidates:
        best, branch = value(0xFF7FFFFF), 'feet'
        for i in candidates:
            y = value(blocks[i]['center'][1])
            if y > best:
                best, selected = y, i
    elif selected is None:
        selected = 0
        if policy & 2:
            spines = [i for i, b in enumerate(blocks) if b['nonzero'] & 8]
            if len(spines) > 2:
                children = [0]*len(blocks)
                for b in blocks:
                    if b['parent'] is not None and b['nonzero'] & 8:
                        children[b['parent']] += 1
                terminals = [i for i in spines if children[i] == 0]
                if terminals:
                    at = terminals[0]
                    for _ in range(len(spines)//2):
                        at = blocks[at]['parent']
                        if at is None:
                            break
                    if at is not None:
                        selected, branch = at, 'midpoint'
    projection = struct.pack('<4I', 0xFFFFFFFF if selected is None else selected,
        not candidates, len(candidates), highest)
    projection += struct.pack('<255I', *(votes+[0]*(255-len(votes))))
    projection += struct.pack('<255I', *(candidates+[0]*(255-len(candidates))))
    assert len(projection) == PROJECTION_BYTES
    admission = 'NoSelectedRoot' if selected is None else 'SelectedRootMissingNonzeroSpine' if not blocks[selected]['nonzero'] & 8 else 'Ok'
    return projection, dict(branch=branch, admission=admission, root=selected)


def request(blocks, policy, cw):
    data = bytearray(struct.pack('<HHIII', cw, 0, len(blocks), policy, 0))
    for b in blocks:
        data.extend(struct.pack('<8I', 0xFFFFFFFF if b['parent'] is None else b['parent'],
            b['presence'], b['nonzero'], *b['center'], b.get('palette', 0),
            0xFFFFFFFF if b.get('symmetric') is None else b['symmetric']))
    data.extend(bytes((255-len(blocks))*32))
    assert len(data) == INPUT_BYTES
    return bytes(data)


def source_digest(raw_rows, initial, optional, blocks):
    data = bytearray(b'native-body-topology-input-v1\0')
    data.extend(struct.pack('<3I', len(blocks), initial['root_nomination'], optional))
    for name in ['source_to_initial', 'initial_to_source']:
        data.extend(struct.pack('<'+'I'*len(blocks), *initial[name]))
    data.extend(raw_rows)
    data.extend(request(blocks, 0, 0x27F)[16:16+len(blocks)*32])
    digest = 0xCBF29CE484222325
    for byte in data:
        digest = ((digest ^ byte)*0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return digest


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('install', type=Path)
    p.add_argument('--source-exe', type=Path, required=True)
    p.add_argument('--capability-inputs', type=Path, required=True)
    p.add_argument('--capability-run', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    a.out.mkdir(parents=True, exist_ok=False)
    cap_run = json.loads((a.capability_run/'comparison-v1.json').read_text())
    assert cap_run['exact']
    caps = (a.capability_run/'native-projection.bin').read_bytes()
    assert hashlib.sha256(caps).hexdigest() == CAPABILITIES_SHA
    cap_resources = json.loads((a.capability_inputs/'source-resources.json').read_text())
    by_key = {(r['package'], r['key']): r for r in cap_resources}
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    oracle = SourceOracle(SourceImage(a.source_exe))
    counts, branches, admissions = Counter(), Counter(), Counter()
    source_records, source_digests, index, rows = [], [], [], 0
    with (a.out/'inputs.bin').open('xb') as inp, (a.out/'python-output.bin').open('xb') as out:
        def emit(blocks, policy, cw, family, source_record=None):
            nonlocal rows
            output, result = reference(blocks, policy)
            inp.write(request(blocks, policy, cw)); out.write(output)
            index.append(dict(row=rows, family=family, blocks=len(blocks), policy=policy, cw=f'{cw:04X}',
                source_record=source_record, **result))
            counts[family] += 1; branches[result['branch']] += 1; admissions[result['admission']] += 1
            rows += 1
        for package in sorted(a.install.rglob('*.package')):
            entries = package_entries(package)
            counts['package_indexes'] += 1
            for entry in entries:
                if entry['type'] != 0x0F43029A:
                    continue
                raw = payload(package, entry)
                key = f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}"
                relative = package.relative_to(a.install).as_posix()
                retained = by_key[(relative, key)]
                sha = hashlib.sha256(raw).hexdigest()
                assert sha == retained['source_sha256']
                initial = constructor_inputs(raw)
                count = initial['source_rows']
                version = struct.unpack_from('<I', raw, 4)[0]
                start = header(raw, 20 if version == 9 else 24)[0]
                centers = [oracle.evaluate(raw[start+i*140:start+(i+1)*140], 53, 0)[0][:3] for i in range(count)]
                blocks = []
                for b in initial['blocks']:
                    source = b['source_palette_id']
                    presence, nonzero = struct.unpack_from('<2I', caps, (retained['first_row']+source)*168)
                    blocks.append(dict(parent=b['parent'], symmetric=None if b['symmetric_word_written']==0xFFFFFFFF else b['symmetric_word_written'],
                        palette=source, center=centers[source], presence=presence, nonzero=nonzero))
                feet = sum(bool(b['nonzero'] & 4) for b in blocks)
                optional = not feet and not initial['root_nomination']
                disposition = 'OptionalFootConstructionRequired' if optional else 'InitialRootMissingNonzeroSpine' if not blocks[0]['nonzero'] & 8 else 'AdmittedWithoutOptionalConstruction'
                input_digest = source_digest(raw[start:start+count*140], initial, optional, blocks)
                source_digests.append(f'{relative} {key} {input_digest:016X}')
                ordinal = len(source_records)
                source_records.append(dict(package=relative, key=key, payload_sha256=sha, rows=count,
                    root_nomination=initial['root_nomination'], optional_foot_driver=optional,
                    disposition=disposition, source_input_fnv1a64=f'{input_digest:016X}',
                    first_native_row=rows if disposition=='AdmittedWithoutOptionalConstruction' else None))
                counts['source_resources'] += 1; counts['source_rows'] += count; counts[disposition] += 1
                if disposition == 'AdmittedWithoutOptionalConstruction':
                    for policy in range(4):
                        emit(blocks, policy, 0x27F, 'source_constructor_no_optional_driver', ordinal)
                if counts['source_resources'] % 128 == 0:
                    print(f"source_resources={counts['source_resources']} source_rows={counts['source_rows']} prepared_cases={rows}", flush=True)
        rng = random.Random(0x9C3150)
        families = []
        for n in [1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 254, 255]:
            for kind in ['chain', 'star', 'random']:
                blocks = []
                for i in range(n):
                    parent = None if i == 0 else i-1 if kind == 'chain' else 0 if kind == 'star' else rng.randrange(i)
                    nonzero = 8 if i==0 or rng.randrange(3)==0 else 4 if rng.randrange(3)==0 else 0
                    presence = nonzero | (8 if rng.randrange(5)==0 else 0)
                    if i and rng.randrange(13)==0:
                        presence |= 1
                    center = [bits(rng.uniform(-8., 8.)) for _ in range(3)]
                    blocks.append(dict(parent=parent, presence=presence, nonzero=nonzero,
                        center=center, palette=rng.getrandbits(32), symmetric=None))
                families.append((f'synthetic_{kind}_{n}', blocks))
        def b(parent, mask, center=(0, 0, 0), nonzero=None):
            return dict(parent=parent, presence=mask, nonzero=mask if nonzero is None else nonzero,
                        center=list(center), palette=0, symmetric=None)
        families += [('synthetic_unset_minimum_height', [b(None, 8), b(0, 8, [0, 0xFF7FFFFF, 0]), b(1, 4)]),
            ('synthetic_presence_only_nearest', [b(None, 8, [bits(8.),0,0]), b(0,8,nonzero=0)]),
            ('synthetic_overflow_distances', [b(None, 8, [0x7F7FFFFF]*3), b(0,8,[0xFF7FFFFF]*3)]),
            ('synthetic_signedzero_subnormal_distances', [b(None,8,[0x80000000,1,0]), b(0,8,[0,0x80000001,0])]),
            ('synthetic_candidate_vote_reset', [b(None,8),b(0,8,[0,bits(2.),0]),b(0,8,[0,bits(2.),0]),b(1,4),b(2,4),b(1,4),b(2,4)])]
        families += [('synthetic_allspine_chain255', [b(None,8)]+[b(i-1,8) for i in range(1,255)]),
                     ('synthetic_allspine_star255', [b(None,8)]+[b(0,8) for _ in range(1,255)]),
                     ('synthetic_midpoint_nonspine_parent', [b(None,8),b(0,0),b(1,0),b(2,8),b(0,8),b(4,8),b(5,8)])]
        # Find a literal rounding witness whose equal coordinate vectors select
        # different roots solely because lane1 adds z^2 before y^2.
        for _ in range(100000):
            center = [bits(2.**rng.uniform(-8.,8.)) for _ in range(3)]
            if value(squared(center, True)) < value(squared(center, False)):
                witness = center
                break
        else:
            raise AssertionError('no SSE lane-order witness')
        families += [('synthetic_unrolled_lane_rounding_witness', [b(None,8,witness),b(0,8,witness),b(0,0),b(0,0)]),
                     ('synthetic_tail_lane_rounding_witness', [b(None,8,witness),b(0,8,witness),b(0,0)])]
        for family, blocks in families:
            for policy, control in itertools.product(range(4), CONTROLS):
                emit(blocks, policy, control, family)
    assert counts['package_indexes']==82 and counts['source_resources']==1365 and counts['source_rows']==46418
    assert counts['AdmittedWithoutOptionalConstruction']==1244
    assert len(source_records)==len(cap_resources)
    (a.out/'source-resources.json').write_text(json.dumps(source_records,indent=2)+'\n')
    (a.out/'source-input-digests.txt').write_text('\n'.join(sorted(source_digests))+'\n',newline='\n')
    (a.out/'input-index.json').write_text(json.dumps(index,indent=2)+'\n')
    report = dict(schema='native-body-root-selection-cohort-v1',host_executed=False,rows=rows,
        statistics=dict(counts),branches=dict(branches),admissions=dict(admissions),source_geometry_context='027F/MXCSR1F80',
        source_geometry_instruction_regions=oracle.evidence,source_sha256=SOURCE_SHA256,
        capability_native_projection_sha256=CAPABILITIES_SHA,
        rounding_witness_center_bits=[f'{x:08X}' for x in witness],
        hashes={name:hashlib.sha256((a.out/name).read_bytes()).hexdigest() for name in ['inputs.bin','python-output.bin','source-resources.json','source-input-digests.txt','input-index.json']},
        scope='Fresh canonical82indexes/1365creationresources/46418rows. Actual root-selection inputs use the1244source resources with admitted initial root and no optional-foot driver, explicit geometry027F and allfourglobal policies.106required optional-driver and15other nonspin-root contexts remain explicit separate gates. No full rig, anchor/localquaternion/capabilityrelabel/selector/runtimepose claim.')
    (a.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__ == '__main__':
    main()
