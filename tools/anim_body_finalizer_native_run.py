"""Execute only a separately reviewed finalizer host/cohort with streamed IO.

This file is preparation until all literal cohort pins are filled and the
parent independently reviews the source/linked host and fixture. No execution
is performed by imports, compilation, preparation or comparison utilities.
"""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
import struct
import time

import anim_body_finalizer_native_probe as probe
from anim_body_optional_native_run import sha,save_json,copy_new,reviewed_run,pinned_artifacts
from cell_schema_probe import SourceImage, SOURCE_SHA256

HOST='30fbd8b05c8a7992f663d0a956233d721050d45dc6f3a66815acbd258aee615b'
ASM='d894f8e774a4e8aaf53577644ad86cbe9a9fcee937eb1422d1e68cdd524f2724'
INPUT='c41b69cdeaca2767102815f3ffb191377d96188e596d749fdbf8167d3c682b33'
EXPECTED='e09a5b85cc07bfbe51cd97179b82f2b4630ac746f7af3190a4857d3e7b62367b'
ROWS=4590

def normalize(input_path,output_path,destination,reference_path=None):
    import hashlib
    count,remainder=divmod(input_path.stat().st_size,probe.INPUT_BYTES)
    assert not remainder and output_path.stat().st_size==count*probe.OUTPUT_BYTES
    controls,modes,sticky,x87=Counter(),Counter(),Counter(),Counter()
    digest=hashlib.sha256();problems=[];mismatches=0
    with input_path.open('rb') as inp,output_path.open('rb') as native,destination.open('xb') as output:
        reference=None if reference_path is None else reference_path.open('rb')
        try:
            for row in range(count):
                request=inp.read(probe.INPUT_BYTES);reply=native.read(probe.OUTPUT_BYTES)
                assert len(request)==probe.INPUT_BYTES and len(reply)==probe.OUTPUT_BYTES
                cw,sw,mxcsr=struct.unpack_from('<HHI',reply,probe.PROJECTION_BYTES)
                assert cw==struct.unpack_from('<H',request)[0] and not sw&0x3800 and mxcsr&~0x3F==0x1F80
                projected=reply[:probe.PROJECTION_BYTES]
                assert projected[:32]==request[:32]
                output.write(projected);digest.update(projected)
                controls[f'{cw:04X}']+=1;modes[str(struct.unpack_from('<I',request,4)[0])]+=1
                sticky[f'{mxcsr&0x3F:02X}']+=1;x87[f'{sw:04X}']+=1
                if reference is not None:
                    expected=reference.read(probe.PROJECTION_BYTES);assert len(expected)==probe.PROJECTION_BYTES
                    if projected!=expected:
                        mismatches+=1
                        if len(problems)<20:
                            first=next(i for i in range(probe.PROJECTION_BYTES) if projected[i]!=expected[i])
                            detail=dict(row=row,byte=first,native=projected[first],expected=expected[first])
                            if first>=32:
                                detail['block']=(first-32)//probe.RECORD_BYTES
                                offset=(first-32)%probe.RECORD_BYTES
                                detail['field']='capabilities' if offset<168 else f'{probe.FIELDS[(offset-168)//4]:03X}'
                            problems.append(detail)
            assert not inp.read(1) and not native.read(1)
            if reference is not None:assert not reference.read(1)
        finally:
            if reference is not None:reference.close()
    return dict(rows=count,projection_sha256=digest.hexdigest(),controls=dict(controls),modes=dict(modes),
        sse_sticky=dict(sticky),x87_status=dict(x87),problem_count=mismatches,problems=problems,exact=not mismatches)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host-dir',type=Path,required=True);parser.add_argument('--input-dir',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    args=parser.parse_args();root=Path(__file__).resolve().parent.parent
    for path in [args.host_dir,args.input_dir,args.out]:path.resolve().relative_to((root/'target').resolve())
    assert INPUT is not None and EXPECTED is not None and ROWS is not None,'Cohort pins incomplete; preparation only'
    manifest_path=args.host_dir/'manifest.json';prepared_path=args.input_dir/'report.json'
    manifest=json.loads(manifest_path.read_text());prepared=json.loads(prepared_path.read_text())
    assert manifest['schema']=='bounded-original-body-finalizer-fields-host-v1'
    assert manifest['source_sha256']==SOURCE_SHA256
    assert manifest['host_exe_sha256']==HOST and manifest['host_source_sha256']==ASM
    assert sha(args.host_dir/'host-x86.exe')==HOST and sha(args.host_dir/'host-x86.asm')==ASM
    assert prepared['rows']==ROWS and prepared['input_sha256']==INPUT and prepared['expected_projection_sha256']==EXPECTED
    assert sha(args.input_dir/'inputs.bin')==INPUT and sha(args.input_dir/'python-output.bin')==EXPECTED
    assert (args.input_dir/'inputs.bin').stat().st_size==ROWS*probe.INPUT_BYTES
    assert (args.input_dir/'python-output.bin').stat().st_size==ROWS*probe.PROJECTION_BYTES
    assert sha(args.input_dir/'source-digests.txt')==prepared['source_digest_sha256']
    image=SourceImage(args.source_exe)
    assert probe.audit_source(image)==manifest['code_allowlist']
    assert probe.host_source(image)[0]==(args.host_dir/'host-x86.asm').read_text()
    assert probe.audit_linked(args.host_dir/'host-x86.exe',args.host_dir/'host-x86.map',image)==manifest['linked_audit']
    args.out.mkdir(parents=True,exist_ok=False)
    copy_new(manifest_path,args.out/'reviewed-host-manifest.json');copy_new(prepared_path,args.out/'reviewed-input-report.json')
    tools=['anim_body_finalizer_native_probe.py','anim_body_finalizer_fixture.py','anim_body_finalizer_native_run.py',
        'anim_body_finalizer_compare.py','anim_body_optional_fixture.py','anim_body_optional_native_probe.py',
        'anim_body_optional_native_run.py','anim_body_anchors_probe.py','anim_body_fields_probe.py',
        'anim_body_hierarchy_probe.py','anim_body_root_fixture.py','anim_body_source_probe.py','anim_body_pose_probe.py',
        'anim_order_probe.py','cell_schema_probe.py','cell_native_math_probe.py','native_crt_acos_probe.py',
        'native_x87_probe.py','creature_header_probe.py','dbpf_extract.py','native_archive_index_probe.py']
    paths=[args.source_exe,*[path for path in args.host_dir.iterdir() if path.is_file()],
        *[path for path in args.input_dir.iterdir() if path.is_file()],
        args.out/'reviewed-host-manifest.json',args.out/'reviewed-input-report.json',
        *[root/'tools'/name for name in tools],root/'re/anim-body-finalizer-fields-native.md',
        root/'crates/spore-bevy/examples/support/optional_foot_protocol.rs']
    paths += [root/'crates/spore-core/src'/name for name in ['anim_body_finalizer_fields.rs','anim_body_optional_feet.rs',
        'anim_body_source_anchors.rs','anim_body_source_fields.rs','anim_body_source.rs','anim_body_capabilities.rs',
        'anim_body_topology.rs','anim_order.rs','native_sse.rs','native_x87.rs','creature_data.rs','part_capabilities.rs']]
    for name in ['anim_body_finalizer_fields_verify','anim_body_finalizer_source_verify']:
        paths += [path for suffix in ['.rs','.rs.pending'] if (path:=root/'crates/spore-bevy/examples'/(name+suffix)).is_file()]
    with pinned_artifacts(paths,args.out):
        assert sha(args.source_exe)==SOURCE_SHA256
        host=(args.host_dir/'host-x86.exe').resolve();started=time.perf_counter()
        cases=args.out/'cases';cases.mkdir()
        with (args.input_dir/'cases.tsv').open(newline='') as table:
            indices=list(csv.DictReader(table,delimiter='\t'))
        assert len(indices)==ROWS
        def record(row):
            with (args.input_dir/'inputs.bin').open('rb') as stream:
                stream.seek(row*probe.INPUT_BYTES);raw=stream.read(probe.INPUT_BYTES)
            assert len(raw)==probe.INPUT_BYTES;return raw
        def execute(label,raw,expected_exit=0,expected_reply=None):
            path=cases/f'{label}-request.bin'
            with path.open('xb') as stream:stream.write(raw)
            result=reviewed_run(host,path,cases,label,120)
            assert result['exit']==expected_exit and not result['stderr_bytes'],label
            if expected_reply is not None:assert result['output'].read_bytes()==expected_reply,label
            return path,result
        smoke_families=['source_reroot','source_local','source_relabel','synthetic_chain255_reroot',
            'synthetic_nonunit_local','synthetic_subnormal_reroot','synthetic_saturated_reroot',
            'synthetic_hole_stale_relabel_skip0','synthetic_duplicate_spine_relabel_skip0',
            'synthetic_branch_stop_relabel_skip0','synthetic_opaque_root_reroot','synthetic_opaque_root_local']
        for family in smoke_families:
            row=next(int(item['case']) for item in indices if item['family']==family)
            path,result=execute(f'smoke{row:05}',record(row))
            normalized=cases/f'smoke{row:05}-projection.bin';normalize(path,result['output'],normalized)
            with (args.input_dir/'python-output.bin').open('rb') as stream:
                stream.seek(row*probe.PROJECTION_BYTES);expected=stream.read(probe.PROJECTION_BYTES)
            assert normalized.read_bytes()==expected
        selected=int(next(item['case'] for item in indices if item['family']=='synthetic_chain3_reroot'))
        smoke=record(selected);_,reference=execute('protocol-reference',smoke);reply=reference['output'].read_bytes()
        tests=[('empty',b'',0,b''),('two_complete',smoke+smoke,0,reply+reply)]
        for length in [1,31,32,probe.INPUT_BYTES-1]:
            tests += [(f'partial{length}',smoke[:length],7,b''),
                (f'complete_then_partial{length}',smoke+smoke[:length],7,reply)]
        mutations=[('reservedPC',0,0x017F),('unmaskedCW',0,0x027E),('highCW',0,0x227F),('reservedWord',0,0x1027F),
            ('mode3',4,3),('count0',8,0),('count256',8,256),('selected256',12,256),('reserved28',28,1),
            ('unconsumedStart',16,1),('rootParent',32+probe.FIELD_INDEX[0x1FC],0),
            ('secondRoot',32+probe.RECORD_BYTES+probe.FIELD_INDEX[0x1FC],0xFFFFFFFF),
            ('cycle',32+probe.RECORD_BYTES+probe.FIELD_INDEX[0x1FC],1),
            ('childCountWrong',32+probe.FIELD_INDEX[0x208],0),('badOrdinal',32+probe.FIELD_INDEX[0x1F8],1),
            ('badSymmetry',32+probe.FIELD_INDEX[0x200],256),('maskMismatch',32,256),
            ('consumedAnchorInf',32+probe.FIELD_INDEX[0x344],0x7F800000),
            ('consumedOriginOutside',32+probe.FIELD_INDEX[0x144],0x71800001),('padding',probe.INPUT_BYTES-4,1)]
        for label,offset,value in mutations:
            raw=bytearray(smoke);struct.pack_into('<I',raw,offset,value);tests.append((label,bytes(raw),7,b''))
        for mode,family in [(1,'synthetic_chain3_local'),(2,'synthetic_chain3_relabel_skip0')]:
            source=record(int(next(item['case'] for item in indices if item['family']==family)))
            extra=[('quaternionOutside',32+probe.FIELD_INDEX[0x118],0x40000001)] if mode==1 else [
                ('rangeOutside',32+probe.FIELD_INDEX[0x204],256),('startOutside',16,256),('flag2',24,2)]
            for label,offset,value in extra:
                raw=bytearray(source);struct.pack_into('<I',raw,offset,value);tests.append((label,bytes(raw),7,b''))
        statuses=[]
        for label,raw,code,expected in tests:
            execute('protocol-'+label,raw,code,expected);statuses.append(dict(case=label,exit=code,exact=True))
        save_json(args.out/'protocol-checks.json',statuses)
        result=reviewed_run(host,args.input_dir/'inputs.bin',args.out,'corpus',300)
        assert result['exit']==0 and not result['stderr_bytes']
        normalized=normalize(args.input_dir/'inputs.bin',result['output'],args.out/'native-projection.bin',args.input_dir/'python-output.bin')
        report=dict(schema='bounded-original-body-finalizer-fields-run-v1',host_exe_sha256=HOST,
            host_source_sha256=ASM,input_sha256=INPUT,expected_sha256=EXPECTED,protocol_checks=len(statuses),
            elapsed_seconds=time.perf_counter()-started,**normalized,
            scope=manifest['scope'],original_bytes_executed=True)
        save_json(args.out/'run.json',report)
        assert normalized['rows']==ROWS and normalized['exact'] and normalized['projection_sha256']==EXPECTED
        print(json.dumps(report,indent=2))

if __name__=='__main__':main()
