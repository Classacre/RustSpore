"""Join fresh217 root ROT100 requests to accepted native source POS frames.

Preparation only. No native process, Cargo, compiler or original module runs.
Full-source neighbor selections are preserved by checked <=8-row local windows.
"""
import argparse
import bisect
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path
import struct

from cell_runtime_curves_fixture import package_index, source_payload
from anim_rotation_native_run import neighbors
from cell_schema_probe import SourceImage, SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
RETENTION = ROOT/'target/animation/root-whole-builder-followup-v1/first-root-static-retention-v3.json'
WHOLE = ROOT/'target/animation/root-whole-builder-followup-v1/all818-whole-channel-prepared-v2.json'
POS = ROOT/'target/animation/source-position-fixture-v1'
ROT_HOST = ROOT/'target/animation/rotation-native-v2'
ROT_EXE_SHA = '972bf058d3cf7711b2e9f0ea5ff63c03d5e4e3f1a7a6dafa7fa17653e9d27f18'
ROT_ASM_SHA = 'b914ed8a7ff6cd972b60cfc3733628489895ec9a2e538a79988547f27f8e84c6'
RETENTION_SHA = '9220c5df63d3f90249da7c00b88febeb5841c2324f5328b7eeb4deaf49767143'


def sha(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(1 << 20), b''):
            result.update(block)
    return result.hexdigest()


def word(raw, at):
    return struct.unpack_from('<I', raw, at)[0]


def finite(bits):
    return bits & 0x7f800000 != 0x7f800000


def make_window(raw, start, descriptor, frame_bits):
    count, keys = struct.unpack_from('<2I', raw, start+212)
    flags, tag, offset, stride, *_ = descriptor
    assert flags & 15 == 2 and flags & 16 and stride >= offset+100
    assert count > 0
    frames = [word(raw, keys+i*stride) for i in range(count)]
    assert frames == sorted(frames)
    if any(f > 0x1000000 for f in frames):
        raise ValueError('source_INFO_frame_above_existing_ROT_host_2pow24_limit')
    frame = struct.unpack('<f', struct.pack('<I',frame_bits))[0]
    if not finite(frame_bits) or abs(frame) > 0x1000000:
        raise ValueError('accepted_POS_frame_outside_existing_ROT_host_finite_2pow24_limit')
    header_flags, duration = word(raw,280), word(raw,288)
    duration_value = struct.unpack('<f',struct.pack('<I',duration))[0]
    if not (1 <= duration_value <= 0x1000000 and frames[-1] <= duration_value-1):
        raise ValueError('source_duration_outside_existing_ROT_host_domain')
    wrap = header_flags & 8
    upper = bisect.bisect_right(frames,frame)
    wanted = neighbors(count,upper,wrap)
    indices = set(wanted)
    if wrap:
        indices.update((0,min(1,count-1),max(0,count-2),count-1))
    indices = sorted(indices)
    assert len(indices) <= 8
    local_frames = [frames[i] for i in indices]
    local_upper = bisect.bisect_right(local_frames,frame)
    local = neighbors(len(indices),local_upper,wrap)
    assert tuple(indices[i] for i in local) == wanted
    normalized = bytearray(1920)
    for local_index,source_index in enumerate(indices):
        at = keys+source_index*stride
        info,rot = raw[at:at+20],raw[at+offset:at+offset+100]
        assert len(info) == 20 and len(rot) == 100
        # This is the existing native host's exact finite input guard.
        if not all(finite(word(rot,i)) for i in (0,4,8,12,16,84,88)):
            raise ValueError('source_ROT_row_nonfinite_in_existing_host_guarded_fields')
        normalized[local_index*120:(local_index+1)*120] = info+rot
    request = struct.pack('<8I',frame_bits,len(indices),duration,wrap,0x027f,1,1,0)+normalized
    assert len(request) == 1952
    return request, dict(full_count=count,full_upper=upper,full_neighbors=list(wanted),
        local_source_indices=indices,local_upper=local_upper,local_neighbors=list(local),
        source_descriptor_words=[f'{v:08X}' for v in descriptor],source_header_flags=f'{header_flags:08X}',
        source_row_window_sha256=hashlib.sha256(normalized).hexdigest())


def prepare(args):
    args.out.resolve().relative_to((ROOT/'target').resolve())
    assert sha(RETENTION) == RETENTION_SHA
    retained = json.loads(RETENTION.read_text())
    whole = json.loads(WHOLE.read_text())
    expected_keys = {tuple(r['key']):r for r in retained['candidates']}
    assert len(expected_keys) == retained['static_first_root_retention_clips'] == 217
    assert sha(ROT_HOST/'host-x86.exe') == ROT_EXE_SHA
    assert sha(ROT_HOST/'host-x86.asm') == ROT_ASM_SHA
    SourceImage(args.install/'SporeBin/SporeApp.exe')
    inventory = json.loads((POS/'inventory.json').read_text())
    comparison = json.loads((POS/'rust-comparison.json').read_text())
    assert inventory['source_sha256'] == SOURCE_SHA256
    assert comparison['byte_identical'] and comparison['full_source_selection']
    assert sha(POS/'output.bin') == inventory['reply_report']['output_sha256']
    assert sha(POS/'expected.tsv') == comparison['expected_sha256'] == inventory['reply_report']['expected_sha256']
    whole_map = {tuple(r['key']):r for r in whole}
    # Legacy display labels only locate candidates. Canonical package lookup,
    # full payload hash and literal neighbor-frame validation establish identity.
    lookup = {(key[0].replace('\\','/').lower(),int(key[1],16)):key for key in expected_keys}
    assert len(lookup) == len(expected_keys)
    indices,resources = {},{}
    for key,entry in expected_keys.items():
        package,low,ty,scope = key
        if package not in indices:
            indices[package] = package_index(args.install/package)[0]
        raw = source_payload(args.install/package,indices[package][tuple(int(v,16) for v in (low,ty,scope))])
        assert hashlib.sha256(raw).hexdigest() == entry['sha256']
        count,table = struct.unpack_from('<2I',raw,324)
        assert count > 0
        start = word(raw,table)
        assert raw[start:start+4] == b'CHAN'
        n,at = struct.unpack_from('<2I',raw,start+220)
        descriptors = [struct.unpack_from('<8I',raw,at+i*32) for i in range(n)]
        rotations = [(i,d) for i,d in enumerate(descriptors) if d[0]&15 == 2 and d[1]]
        assert len(rotations) == 1
        ordinal,descriptor = rotations[0]
        assert descriptor[0] == whole_map[key]['channels'][0]['descriptors'][ordinal][0] == 0x32
        resources[key] = (raw,start,ordinal,descriptor)
    args.out.mkdir(parents=True,exist_ok=False)
    records,excluded,coverage = [],[],Counter()
    with (POS/'expected.tsv').open(newline='') as table, (POS/'output.bin').open('rb') as native_pos, \
            (args.out/'inputs.bin').open('xb') as requests, (args.out/'accepted-pos-replies.bin').open('xb') as pos_copies:
        for item in csv.DictReader(table,delimiter='\t'):
            if int(item['channel']) != 0:
                continue
            display = (item['package'].replace('\\','/').lower(),int(item['instance'],16)&0xffffffff)
            key = lookup.get(display)
            if key is None:
                continue
            assert item['type'] == key[2] and item['cw'] == '027F'
            raw,start,ordinal,descriptor = resources[key]
            source_case = int(item['native_case_ordinal'])
            native_pos.seek(source_case*124)
            reply = native_pos.read(124)
            assert len(reply) == 124
            expected_pos = tuple(int(item[name],16) for name in ('x_bits','y_bits','z_bits','weight_bits'))
            assert struct.unpack_from('<4I',reply,20) == expected_pos
            try:
                request,detail = make_window(raw,start,descriptor,int(item['frame_bits'],16))
            except ValueError as error:
                excluded.append(dict(key=list(key),source_pos_case=source_case,frame_bits=item['frame_bits'],
                                     reason='source_window_or_existing_host_domain',error=repr(error)))
                continue
            assert detail['full_neighbors'] == [int(item[k]) for k in ('previous','left','right','following')]
            requests.write(request)
            pos_copies.write(reply)
            coverage[key] += 1
            records.append(dict(row=len(records),key=list(key),source_sha256=expected_keys[key]['sha256'],
                channel=0,rotation_component=ordinal,frame_bits=item['frame_bits'],cw='027F',blend=1,
                enabled=1,source_pos_case=source_case,accepted_pos_bits=[f'{v:08X}' for v in expected_pos],**detail))
    missing = [dict(key=list(key),sha256=entry['sha256'],reason='no_admitted_matching_accepted_POS_frame')
               for key,entry in expected_keys.items() if not coverage[key]]
    (args.out/'input-index.json').write_text(json.dumps(records,indent=2)+'\n')
    (args.out/'excluded.json').write_text(json.dumps(excluded,indent=2)+'\n')
    (args.out/'missing-clips.json').write_text(json.dumps(missing,indent=2)+'\n')
    pins = [RETENTION,WHOLE,POS/'inventory.json',POS/'rust-comparison.json',POS/'expected.tsv',POS/'output.bin',
            ROT_HOST/'host-x86.exe',ROT_HOST/'host-x86.asm',ROT_HOST/'manifest.json',Path(__file__)]
    manifest = dict(schema='source-first-root-ROT100-paired-native-POS-inputs-v1',native_executed=False,
        source_sha256=SOURCE_SHA256,requested_clips=217,covered_clips=len(coverage),missing_clips=len(missing),
        rows=len(records),excluded_frames=len(excluded),input_bytes=1952,output_bytes=240,
        expected_native_output_bytes=len(records)*240,accepted_POS_reply_bytes=124,
        accepted_ROT_host_sha256=ROT_EXE_SHA,accepted_ROT_host_assembly_sha256=ROT_ASM_SHA,
        context='ExplicitCW027F/MXCSR1F80,enabled descriptor,blend1,sourceheaderwrapbit8. No clock/actor route implied.',
        descriptor_projection='Literal source ROT32 to immutable host ROT12: retained original sampler tests only lowkind nibble and bit10; bit20 belongs to downstream coordinate conversion. Source32 retained in index.',
        source_window='Every actualacceptedPOSframe is requested fromfreshoriginalROTrows; <=8sortedlocalrows preserveallfourfullsourceneighbors. No authoredendpointsubstitute or inferred quaternion output.',
        coverage=[dict(key=list(key),rows=coverage[key]) for key in expected_keys],
        pins={str(p.relative_to(ROOT)):sha(p) for p in pins},
        files={name:sha(args.out/name) for name in ('inputs.bin','accepted-pos-replies.bin','input-index.json','excluded.json','missing-clips.json')},
        next_gate='Root reviews/pins this cohort and executes alreadyacceptedROT100host separately. ROT/POS coordinateconversion andposeaccumulator/stagedcommits remain subsequent gates.')
    (args.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({k:manifest[k] for k in ('schema','requested_clips','covered_clips','missing_clips','rows','excluded_frames','native_executed')},indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--install',type=Path,default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--out',type=Path,required=True)
    prepare(parser.parse_args())
