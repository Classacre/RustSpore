"""Preflight or separately admit the pinned root coordinate/accumulation host.

Default mode never executes. --run requires an independent peer closure report.
Uses hidden file-backed executions and never retries; each retained run is one attempt.
All true failures, partial outputs and immutable pins remain saved.
"""
import argparse
import json
from pathlib import Path
import struct

import anim_root_pose_native_probe_v6 as probe
import anim_root_pose_sparse_carrier_audit_v1 as carrier_audit
from types import SimpleNamespace
from cell_runtime_data_native_run import digest,pinned,saved_file_run as reviewed_file_run,word
from cell_schema_probe import SourceImage,SOURCE_SHA256

ROOT=Path(__file__).resolve().parent.parent
HOST_SHA='4a3c18ffa486543e853d1c1a15cc3ddccfa74d36da274c344da1b5d60ef7fb2c'
OWNED_SHA='c9175b6c6ac179a7c2e1cd73cb3f948d1816ee83eb349f63ec43756575da6c03'
ASM_SHA='7af99154c44f1fd955faa19c9702738b3b3340646cece058a4c1066d0dd26b8a'
ROT_SHA='8b53cdb96631791556116957ce02ee299cedde0092c1144c0084dc9b7eb8a98f'
NEW_PINS={'tools/anim_root_pose_sparse_carrier_v2.py': '0b7ff5a9e12fc70d1f558f465ef81565652f5999337f00c8cca00098bf359528', 'tools/anim_root_pose_sparse_carrier_audit_v1.py': '941049d7ae3dda35191af68105725f62809612a9bff6137b24ec5dba142d3e68', 'crates/spore-core/src/anim_root_pose.rs': 'fd434d07d85ed6ab86588b7ec647f11f524f2c6e855f0555d52835463b5eff21', 'crates/spore-bevy/examples/anim_root_pose_verify.rs.pending': 'bd5088b90c74371987af8a241b9f7efd63202948773da6a0c39103793969aa49'}
CONTROLS=[0x7f|p|r for p in (0,0x200,0x300) for r in (0,0x400,0x800,0xc00)]


def save(path,data):
    with path.open('x') as f:json.dump(data,f,indent=2);f.write('\n')
def bits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def resolve(name):return Path(name) if Path(name).is_absolute() else ROOT/name


def check_domain(raw):
    assert len(raw)==144
    w=struct.unpack('<36I',raw)
    assert w[0] in CONTROLS and w[1] in (0x31,0x71,0xb1) and w[2] in (0,0x10) and w[3] in (0,1)
    assert w[10]&~8==0 and w[15]&0xff000000==0 and w[15]&6==6
    assert w[4]&0x80000000==0 and w[4]&0x7fffffff!=0
    assert all(w[i]&0x7f800000!=0x7f800000 for i in range(4,36) if i not in (10,15))
    return w


def peer_admission(path,required):
    if path is None:
        assert not required,'Independent peer report required before --run'
        return dict(admitted=False,reason='peer_review_pending')
    peer=json.loads(path.read_text())
    for key in ('no_blocking_static_issue_found','original_spans_exact','unreachable_calls_closed',
                'partial_entry_frame_registers_closed','guarded_memory_bounds_closed'):
        assert peer[key] is True,key
    assert peer['host_executed'] is False
    for name,expected in peer['pins'].items():assert digest(resolve(name))==expected,name
    return dict(admitted=True,report=str(path.resolve()),sha256=digest(path))


def review(args):
    for name,value in NEW_PINS.items():assert digest(ROOT/name)==value,name
    manifest=json.loads((args.host_dir/'manifest.json').read_text())
    linked=json.loads((args.host_dir/'independent-linked-audit.json').read_text())
    assert digest(args.host_dir/'carrier-x86.exe')==HOST_SHA==linked['carrier_sha256']
    assert digest(args.host_dir/'host-x86.exe')==OWNED_SHA==linked['owned_sha256']
    actual=carrier_audit.audit(SimpleNamespace(carrier=args.host_dir/'carrier-x86.exe',owned_exe=args.host_dir/'host-x86.exe',owned_map=args.host_dir/'host-x86.map',source_exe=args.source_exe))
    assert actual==linked
    assert linked['static_pass'] and linked['native_executed'] is False
    assert manifest['tool_sha256']==digest(ROOT/'tools/anim_root_pose_sparse_carrier_v2.py')
    assert digest(args.host_dir/'host-x86.asm')==manifest['asm_sha256']
    assert digest(ROOT/'tools/anim_root_pose_conversion_native_run.py')=='123597d7d3aa1581cab01215176c5f64194c1d2db814a9109eb44958f0e63ee1'
    fixture=json.loads((args.fixture/'manifest.json').read_text())
    assert fixture['rows']==22212 and fixture['source_curve_pairs']==3702 and fixture['source_clips']==215
    assert fixture['contexts']==3 and fixture['source_binding_passes']==2 and fixture['positive_blend_policy']
    for name,expected in fixture['pins'].items():assert digest(resolve(name))==expected,name
    for name,expected in fixture['files'].items():assert digest(args.fixture/name)==expected,name
    curves=ROOT/'target/animation/root-pose-curve-inputs-v2'
    native=ROOT/'target/animation/root-pose-curve-native-v1'
    assert digest(native/'corpus-attempt01-output.bin')==ROT_SHA
    context={r['name']:r for r in json.loads((args.fixture/'reference-contexts.json').read_text())}
    index=json.loads((args.fixture/'input-index.json').read_text());assert len(index)==22212
    source_index=json.loads((curves/'input-index.json').read_text())
    whole={tuple(r['key']):r for r in json.loads((ROOT/'target/animation/root-whole-builder-followup-v1/all818-whole-channel-prepared-v2.json').read_text())}
    counts={}
    with (args.fixture/'inputs.bin').open('rb') as requests, (curves/'accepted-pos-replies.bin').open('rb') as pos, \
            (native/'corpus-attempt01-output.bin').open('rb') as rot:
        for row,item in enumerate(index):
            raw=requests.read(144);w=check_domain(raw)
            assert item['row']==row and digest_bytes(raw)==item['input_sha256']
            curve=item['curve_row'];source=source_index[curve]
            assert item['key']==source['key'] and item['frame_bits']==source['frame_bits']
            assert item['source_sha256']==source['source_sha256']
            pos.seek(curve*124);rot.seek(curve*240)
            p=struct.unpack_from('<4I',pos.read(124),20);q=struct.unpack_from('<5I',rot.read(240),100)
            assert w[27:31]==p and w[31:36]==q
            ch=whole[tuple(item['key'])]['channels'][0]
            assert w[1]==next(d[0] for d in ch['descriptors'] if d[0]&15==1 and d[1])
            assert w[2]==ch['raw_metadata_words_88_d0'][0]
            assert w[3]==(int(item['binding_words'][3],16)>>25)&1
            assert w[15]==int(item['binding_words'][2],16)&0xffffff
            c=context[item['context']]
            expected=[bits(c['blend']),bits(c['scale']),*[bits(v) for v in c['dimensions']],bits(c['size374']),
                c['rootmask'],bits(c['size434']),*[bits(v) for v in c['rest_position']],w[15],
                *[bits(v) for v in c['rest_rotation']],*[bits(v) for v in c['reference_position']],
                *[bits(v) for v in c['reference_rotation']]]
            assert w[4:27]==tuple(expected)
            counts[item['context']]=counts.get(item['context'],0)+1
        assert requests.read(1)==b''
    peer=peer_admission(args.peer_review,args.run)
    return fixture,index,dict(records=22212,source_curve_pairs=3702,source_clips=215,contexts=counts,
        paired_native_curve_values_exact=True,whole_binding_mask_preserved=True,positive_blend_only=True,peer=peer)


def digest_bytes(raw):
    import hashlib
    return hashlib.sha256(raw).hexdigest()


def inspect(input_path,output_path,projection=None):
    count=input_path.stat().st_size//144
    assert input_path.stat().st_size==count*144 and output_path.stat().st_size==count*176
    projected=None if projection is None else projection.open('xb')
    nonfinite=0
    try:
        with input_path.open('rb') as requests,output_path.open('rb') as replies:
            for row in range(count):
                raw,reply=requests.read(144),replies.read(176)
                assert struct.unpack_from('<H',reply,156)[0]==word(raw,0)
                assert struct.unpack_from('<H',reply,158)[0]&0x3800==0
                assert word(reply,160)&~0x3f==0x1f80 and reply[164:]==bytes(12)
                assert word(reply,152) in (0,1)
                if any(word(reply,at)&0x7f800000==0x7f800000 for at in range(0,152,4)):nonfinite+=1
                if projected is not None:projected.write(reply[:156])
    finally:
        if projected is not None:projected.close()
    return dict(records=count,nonfinite_projection_cases=nonfinite)


def execute(args,fixture,closure):
    out=args.out/'native-v1';out.mkdir(exist_ok=False)
    peer=json.loads(args.peer_review.read_text())
    paths=[args.source_exe,args.carrier_peer,ROOT/'tools/anim_root_pose_sparse_carrier_v2.py',ROOT/'tools/anim_root_pose_sparse_carrier_audit_v1.py',ROOT/'crates/spore-core/src/anim_root_pose.rs',ROOT/'crates/spore-bevy/examples/anim_root_pose_verify.rs.pending',Path(__file__),Path(probe.__file__),Path(probe.v3.__file__),
        args.peer_review,args.out/'preflight.json',args.fixture/'manifest.json',
        *[resolve(p) for p in fixture['pins']],*[args.fixture/n for n in fixture['files']],
        *[resolve(p) for p in peer['pins']],*[resolve(p) for p in json.loads(args.carrier_peer.read_text())['pins']],*[p for p in args.host_dir.iterdir() if p.is_file()],
        ROOT/'tools/cell_runtime_data_native_run.py',ROOT/'tools/cell_schema_probe.py',
        ROOT/'tools/native_crt_acos_probe.py']
    paths=list(dict.fromkeys(path.resolve() for path in paths))
    host=(args.host_dir/'carrier-x86.exe').resolve()
    with pinned(paths,out):
        protocol=out/'protocol';protocol.mkdir()
        with (args.fixture/'inputs.bin').open('rb') as f:smoke=f.read(144)
        reference=protocol/'reference-input.bin';reference.write_bytes(smoke)
        status,output=reviewed_file_run(host,reference,protocol,'reference')
        assert status['exit']==0 and status['stderr_bytes']==0
        inspect(reference,output);expected=output.read_bytes()
        if args.reference_only:
            save(out/'reference-only-report.json',dict(native_executed=True,records=1,carrier_sha256=HOST_SHA,reference_only=True,output_sha256=digest(output),retries=0))
            return
        checks=[('empty',b'',0,b'')]
        for n in(1,32,143):
            checks.extend([(f'partial{n}',smoke[:n],71,b''),(f'complete_then_partial{n}',smoke+smoke[:n],71,expected)])
        mutations=[(0,0x17f,'reserved_precision'),(0,0x27e,'unmasked_CW'),(0,0x10027f,'CW_highbits'),
            (4,0x11,'unsupported_POS_flags'),(8,1,'relative_channel'),(8,0x20,'unsupported_channel'),
            (12,2,'mirror2'),(16,0,'zero_blend'),(16,0x80000000,'negative_zero_blend'),(16,0xbf800000,'negative_blend'),
            (16,0x7fc00000,'NaN_blend'),(20,0x7f800000,'infinite_scale'),(40,1,'unsupported_root_mask'),
            (60,0x1000006,'high_binding_mask'),(60,2,'missing_ROT_bit'),(60,4,'missing_POS_bit'),
            (108,0x7fc00000,'NaN_sample_POS'),(120,0x7f800000,'infinite_POS_weight'),
            (124,0x7fc00000,'NaN_sample_ROT'),(140,0x7f800000,'infinite_ROT_weight')]
        for at,value,name in mutations:
            raw=bytearray(smoke);struct.pack_into('<I',raw,at,value);checks.append((name,bytes(raw),71,b''))
        checks.append(('repeat_two',smoke+smoke,0,expected+expected))
        results=[]
        for name,raw,code,wanted in checks:
            path=protocol/f'{name}-input.bin';path.write_bytes(raw)
            status,output=reviewed_file_run(host,path,protocol,name)
            assert status['exit']==code and status['stderr_bytes']==0,(name,status)
            assert output.read_bytes()==wanted,name
            results.append(dict(case=name,exit=code,exact=True,input_sha256=digest(path)))
        for cw in CONTROLS:
            name=f'policy_{cw:04X}';raw=bytearray(smoke);struct.pack_into('<I',raw,0,cw)
            path=protocol/f'{name}-input.bin';path.write_bytes(raw)
            status,output=reviewed_file_run(host,path,protocol,name)
            assert status['exit']==0 and status['stderr_bytes']==0
            inspect(path,output);assert output.read_bytes()[:156]==expected[:156]
            results.append(dict(case=name,exit=0,exact_projection=True,input_sha256=digest(path)))
        for value,label in((0,'zero_weights'),(0xbf800000,'negative_weights')):
            raw=bytearray(smoke)
            for at in(120,140):struct.pack_into('<I',raw,at,value)
            path=protocol/f'{label}-input.bin';path.write_bytes(raw)
            status,output=reviewed_file_run(host,path,protocol,label)
            assert status['exit']==0 and status['stderr_bytes']==0
            inspect(path,output);reply=output.read_bytes()
            assert reply[:28]==expected[:28] and reply[44:116]==bytes(72) and word(reply,152)==0
            assert word(reply,28)==word(smoke,16)
            results.append(dict(case=label,exit=0,zero_pose_sums=True,input_sha256=digest(path)))
        save(protocol/'report.json',dict(checks=len(results),results=results))
        status,output=reviewed_file_run(host,args.fixture/'inputs.bin',out,'corpus',timeout=120)
        assert status['exit']==0 and status['stderr_bytes']==0,status
        observed=inspect(args.fixture/'inputs.bin',output,out/'native-projection.bin')
        assert observed['records']==22212
        result=dict(schema='source-root-coordinate-accumulation-sparse-native-run-v1',native_executed=True,
            records=22212,source_curve_pairs=3702,source_clips=215,protocol_checks=len(results),
            host_exe_sha256=HOST_SHA,host_assembly_sha256=ASM_SHA,input_sha256=digest(args.fixture/'inputs.bin'),
            output_sha256=digest(output),projection_sha256=digest(out/'native-projection.bin'),observed=observed,
            scope='Accepted sourcePOS/ROTsamplepairs to originalcoordinateconversion andweightedaccumulators;threeexplicitreferencecontexts,bothretainedsourcebindingpasses. Noactualactorbodyassociation,stagedcommit,childconstraints,clockorvisibleanimationclaim.')
        save(out/'report.json',result);print(json.dumps(result,indent=2))


def carrier_peer_admission(path,required):
    if path is None:
        assert not required,'Actual independent carrier peer report required'
        return dict(admitted=False,reason='carrier_peer_pending')
    report=json.loads(path.read_text())
    assert report['no_blocking_static_issue_found'] is True
    assert report['native_executed'] is False
    for name,value in report['pins'].items():assert digest(resolve(name))==value,name
    assert HOST_SHA in report['pins'].values()
    return dict(admitted=True,report=str(path.resolve()),sha256=digest(path))

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--fixture',type=Path,required=True)
    p.add_argument('--peer-review',type=Path);p.add_argument('--carrier-peer',type=Path);p.add_argument('--reference-only',action='store_true');p.add_argument('--out',type=Path,required=True)
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--run',action='store_true');a=p.parse_args()
    a.out.resolve().relative_to((ROOT/'target').resolve())
    fixture,index,closure=review(a)
    closure['carrier_peer']=carrier_peer_admission(a.carrier_peer,a.run)
    a.out.mkdir(parents=True,exist_ok=False)
    save(a.out/'preflight.json',dict(schema='root-coordinate-accumulation-preflight-v1',native_executed=False,
        native_execution_requested=a.run,runner_sha256=digest(__file__),closure=closure))
    if a.run:execute(a,fixture,closure)
    else:print(json.dumps(dict(preflight=True,records=len(index),peer=closure['peer'],native_executed=False)))


if __name__=='__main__':main()
