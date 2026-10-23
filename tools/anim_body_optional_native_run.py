"""Run only a separately reviewed pinned optional-foot host, using file-backed IO."""
import argparse
from collections import Counter
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import time

import anim_body_optional_native_probe as probe
from cell_schema_probe import SourceImage

HOST='b1fbd363b6e81d9542d398a5ff7307f6c1062b75a794df3e5dfb00a01b0aa84f'
ASM='0a13cfc1ca547c212d8a9f3fca0fc066afa85746d79f5a9f2a4b5eae5b58c4a9'
INPUT='5fda00ed148a574473f98ea048008d721e34913901c5b6de4a472734597c4965'
EXPECTED='3675514158d42c004abedc05d2d1fa6837de9746ae8439a3f24ccb86a2d872bd'
ROWS=1212


def sha(path):
    digest=hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(1<<20),b''):digest.update(block)
    return digest.hexdigest()
def save_json(path,value):
    with path.open('x') as stream:json.dump(value,stream,indent=2);stream.write('\n')
def copy_new(source,destination):
    with source.open('rb') as inp,destination.open('xb') as out:shutil.copyfileobj(inp,out,1<<20)


def saved_run(host,request_path,folder,label,timeout_seconds=60):
    """Save input before launch; output files persist even on timeout/exception."""
    inp=folder/f'{label}-input.bin';out=folder/f'{label}-output.bin';err=folder/f'{label}-stderr.bin'
    copy_new(request_path,inp)
    started=time.perf_counter();process=None;failure=None;cleanup=None;timed_out=False
    try:
        with inp.open('rb') as source,out.open('xb') as stdout,err.open('xb') as stderr:
            try:
                process=subprocess.Popen([str(host)],stdin=source,stdout=stdout,stderr=stderr,
                    creationflags=subprocess.CREATE_NO_WINDOW)
                process.wait(timeout=timeout_seconds)
            except BaseException as exc:
                failure=exc;timed_out=isinstance(exc,subprocess.TimeoutExpired)
                if process is not None:
                    try:
                        if process.poll() is None:process.kill()
                        process.wait(timeout=5)
                    except BaseException as cleanup_exc:cleanup=repr(cleanup_exc)
            finally:
                stdout.flush();stderr.flush()
    except BaseException as exc:
        if failure is None:failure=exc
    finally:
        status=dict(exit=None if process is None else process.returncode,timeout=timed_out,
            exception=None if failure is None else repr(failure),cleanup_exception=cleanup,
            input_bytes=inp.stat().st_size,output_bytes=out.stat().st_size if out.exists() else 0,
            stderr_bytes=err.stat().st_size if err.exists() else 0,elapsed_seconds=time.perf_counter()-started,
            hidden_file_backed_io=True)
        save_json(folder/f'{label}-status.json',status)
    if failure is not None:raise failure
    return dict(status,output=out,stderr=err)


def reviewed_run(host,request_path,folder,label,timeout_seconds=60):
    attempts=[]
    try:
        for attempt in range(1,9):
            prefix=f'{label}-attempt{attempt:02}'
            try:result=saved_run(host,request_path,folder,prefix,timeout_seconds)
            except BaseException as exc:
                attempts.append(dict(attempt=attempt,label=prefix,retried=False,exception=repr(exc)));raise
            allocation_only=result['exit']==10 and not result['output_bytes'] and not result['stderr_bytes']
            attempts.append(dict(attempt=attempt,label=prefix,exit=result['exit'],output_bytes=result['output_bytes'],
                stderr_bytes=result['stderr_bytes'],allocation_only_exit10=allocation_only,retried=allocation_only and attempt<8))
            if not allocation_only:return result
        return result
    finally:save_json(folder/f'{label}-attempts.json',dict(maximum_attempts=8,attempts=attempts,
        retry_policy='Only pre-ReadRecord allocation exit10 with empty stdout/stderr. Never retry native exit7, output-bearing failure, timeout or exception.'))


@contextmanager
def pinned_artifacts(paths,folder):
    pins={str(p.resolve()):sha(p) for p in paths};save_json(folder/'execution-pins-v1.json',pins);failure=None
    try:yield
    except BaseException as exc:failure=repr(exc);raise
    finally:
        changed=[];errors=[]
        for name,digest in pins.items():
            try:
                if sha(Path(name))!=digest:changed.append(name)
            except BaseException as exc:changed.append(name);errors.append(dict(path=name,error=repr(exc)))
        save_json(folder/'immutability-v1.json',dict(pinned_files=len(pins),unchanged=not changed,
            changed_files=changed,read_errors=errors,operation_exception=failure))
        if changed:raise AssertionError('Pinned artifacts changed: '+repr(changed))


def normalize(request_path,output_path,destination,reference_path=None):
    controls,sticky,x87=Counter(),Counter(),Counter();digest=hashlib.sha256();problems=[];rows=0
    with request_path.open('rb') as inp,output_path.open('rb') as native,destination.open('xb') as out:
        reference=None if reference_path is None else reference_path.open('rb')
        try:
            while True:
                request=inp.read(probe.INPUT_BYTES)
                if not request:break
                assert len(request)==probe.INPUT_BYTES
                result=native.read(probe.OUTPUT_BYTES);assert len(result)==probe.OUTPUT_BYTES
                mxcsr,cw,sw=struct.unpack_from('<IHH',result,probe.PROJECTION_BYTES)
                assert cw==struct.unpack_from('<H',request)[0] and not sw&0x3800 and mxcsr&~0x3F==0x1F80
                controls[f'{cw:04X}']+=1;sticky[f'{mxcsr&0x3F:02X}']+=1;x87[f'{sw:04X}']+=1
                projected=result[:probe.PROJECTION_BYTES];out.write(projected);digest.update(projected)
                if reference is not None:
                    expected=reference.read(probe.PROJECTION_BYTES);assert len(expected)==probe.PROJECTION_BYTES
                    if projected!=expected:
                        first=next(i for i,(a,b) in enumerate(zip(projected,expected)) if a!=b)
                        if len(problems)<20:problems.append(dict(row=rows,first_byte=first,native=projected[first],expected=expected[first]))
                rows+=1
            assert not native.read(1)
            if reference is not None:assert not reference.read(1)
        finally:
            if reference is not None:reference.close()
    return dict(rows=rows,projection_sha256=digest.hexdigest(),controls=dict(controls),sse_sticky=dict(sticky),
        x87_status=dict(x87),problems=problems,exact=not problems)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--input-dir',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True);a=p.parse_args();root=Path(__file__).resolve().parent.parent
    for path in [a.host_dir,a.input_dir,a.out]:path.resolve().relative_to((root/'target').resolve())
    assert INPUT is not None and EXPECTED is not None and ROWS is not None,'cohort preparation/review pin incomplete'
    manifest_path=a.host_dir/'manifest.json';prepared_path=a.input_dir/'report.json'
    manifest=json.loads(manifest_path.read_text());prepared=json.loads(prepared_path.read_text())
    assert manifest['schema']=='bounded-original-optional-foot-construction-host-v1'
    assert manifest['host_exe_sha256']==HOST and manifest['host_source_sha256']==ASM
    assert sha(a.host_dir/'host-x86.exe')==HOST and sha(a.host_dir/'host-x86.asm')==ASM
    assert prepared['rows']==ROWS and prepared['input_sha256']==INPUT and prepared['expected_projection_sha256']==EXPECTED
    assert sha(a.input_dir/'inputs.bin')==INPUT and sha(a.input_dir/'python-output.bin')==EXPECTED
    assert (a.input_dir/'inputs.bin').stat().st_size==ROWS*probe.INPUT_BYTES
    assert (a.input_dir/'python-output.bin').stat().st_size==ROWS*probe.PROJECTION_BYTES
    assert sha(a.input_dir/'source-digests.txt')==prepared['source_digest_sha256']
    image=SourceImage(Path(manifest['source_path']))
    assert probe.audit_source(image)==manifest['code_allowlist']
    assert probe.host_source(image)[0]==(a.host_dir/'host-x86.asm').read_text()
    assert probe.linked_audit(a.host_dir/'host-x86.exe',a.host_dir/'host-x86.map')==manifest['linked_audit']
    a.out.mkdir(parents=True,exist_ok=False)
    copy_new(manifest_path,a.out/'reviewed-host-manifest.json');copy_new(prepared_path,a.out/'reviewed-input-report.json')
    tool_names=['anim_body_optional_native_run.py','anim_body_optional_native_probe.py','anim_body_optional_fixture.py',
        'anim_body_optional_compare.py','anim_body_anchors_probe.py','anim_body_fields_probe.py','anim_body_hierarchy_probe.py',
        'anim_body_source_probe.py','anim_body_pose_probe.py','cell_schema_probe.py','native_x87_probe.py',
        'native_archive_index_probe.py','creature_header_probe.py','dbpf_extract.py','native_crt_acos_probe.py','cell_native_math_probe.py']
    paths=[Path(manifest['source_path']),*[v for v in a.host_dir.iterdir() if v.is_file()],
        *[v for v in a.input_dir.iterdir() if v.is_file()],a.out/'reviewed-host-manifest.json',a.out/'reviewed-input-report.json',
        *[root/'tools'/v for v in tool_names],root/'re/anim-optional-feet-native.md',
        root/'crates/spore-bevy/examples/support/optional_foot_protocol.rs']
    paths += [root/'crates/spore-core/src'/name for name in ['anim_body_optional_feet.rs','anim_body_source_anchors.rs',
        'anim_body_source_fields.rs','anim_body_source.rs','anim_body_capabilities.rs','native_sse.rs','native_x87.rs',
        'creature_data.rs','part_capabilities.rs','anim_body_topology.rs']]
    for name in ['anim_body_optional_verify','anim_body_optional_source_verify']:
        paths += [v for suffix in ['.rs','.rs.pending'] if (v:=root/'crates/spore-bevy/examples'/(name+suffix)).is_file()]
    with pinned_artifacts(paths,a.out):
        assert sha(Path(manifest['source_path']))==manifest['source_sha256']
        started=time.perf_counter();host=(a.host_dir/'host-x86.exe').resolve()
        cases=a.out/'cases';cases.mkdir()
        def input_record(row):
            with (a.input_dir/'inputs.bin').open('rb') as stream:stream.seek(row*probe.INPUT_BYTES);return stream.read(probe.INPUT_BYTES)
        def run_bytes(label,data,expected_exit=0,expected_output=None):
            path=cases/f'{label}-request.bin'
            with path.open('xb') as stream:stream.write(data)
            result=reviewed_run(host,path,cases,label)
            assert result['exit']==expected_exit and not result['stderr_bytes'],label
            if expected_output is not None:assert result['output'].read_bytes()==expected_output,label
            return path,result
        smoke_rows=[next(i['row'] for i in prepared['index'] if i['family']=='source_call_gate' and i['appended']==0),
            next(i['row'] for i in prepared['index'] if i['family']=='source_call_gate' and i['appended']>0)]
        for family in ['synthetic_mirror1_threshold3DCCCCCD','synthetic_empty255','synthetic_no_spines','synthetic_presence_only',
            'synthetic_threshold_unordered','synthetic_height_overflow','synthetic_source_opaque_nonfinite','synthetic_signed_subnormal','synthetic_nonunit_rotations']:
            smoke_rows.append(next(i['row'] for i in prepared['index'] if i['family']==family))
        for family in ['synthetic_exact_capacity255','synthetic_repeated_threshold_stop','synthetic_repeated_threshold_equal','synthetic_presence_foot_zero_number']:
            smoke_rows.append(next(i['row'] for i in prepared['index'] if i['family']==family))
        for row in smoke_rows:
            path,result=run_bytes(f'smoke{row:04}',input_record(row))
            normalized=normalize(path,result['output'],cases/f'smoke{row:04}-projection.bin')
            with (a.input_dir/'python-output.bin').open('rb') as stream:
                stream.seek(row*probe.PROJECTION_BYTES);expected=stream.read(probe.PROJECTION_BYTES)
            assert (cases/f'smoke{row:04}-projection.bin').read_bytes()==expected
        row=next(i['row'] for i in prepared['index'] if i['family']=='synthetic_mirror0_threshold3DCCCCCD')
        smoke=input_record(row);_,valid=run_bytes('protocol-reference',smoke);one=valid['output'].read_bytes()
        negatives=[('empty',b'',0,b''),('two_complete',smoke+smoke,0,one+one)]
        for length in [1,15,16,probe.INPUT_BYTES-1]:
            negatives += [(f'partial{length}',smoke[:length],7,b''),(f'complete_then_partial{length}',smoke+smoke[:length],7,one)]
        mutations=[('reservedPC',0,0x17F),('unmaskedCW',0,0x27E),('reservedCW',0,0x227F),
            ('reservedHeader',0,0x1027F),('count0',4,0),('count256',4,256),('thresholdInf',8,0x7F800000),
            ('thresholdNaN',8,0x7FC12345),('flag2',12,2),('mask256',16,256),('nonzeroFoot',20,4),
            ('ordinalWrong',16+probe.FIELD_INDEX[0x1F8],9),('paletteWrong',16+probe.FIELD_INDEX[0x21C],9),
            ('rootParent',16+probe.FIELD_INDEX[0x1FC],0),('symmetry256',16+probe.FIELD_INDEX[0x200],256),
            ('centerInf',16+probe.FIELD_INDEX[0x10C],0x7F800000),('quaternionOutside',16+probe.FIELD_INDEX[0x118],0x40000001),
            ('originOutside',16+probe.FIELD_INDEX[0x144],0x71800001),('extent0',16+probe.FIELD_INDEX[0x138],0),
            ('extentNegative',16+probe.FIELD_INDEX[0x138],0xBF800000),('childCountWrong',16+probe.FIELD_INDEX[0x208],0),
            ('secondRoot',16+probe.RECORD_BYTES+probe.FIELD_INDEX[0x1FC],0xFFFFFFFF),
            ('selfCycle',16+probe.RECORD_BYTES+probe.FIELD_INDEX[0x1FC],1),
            ('candidateMouthGrasp',16+probe.RECORD_BYTES,0x22),('nonzeroPadding',probe.INPUT_BYTES-4,1)]
        for label,at,value in mutations:
            changed=bytearray(smoke);struct.pack_into('<I',changed,at,value);negatives.append((label,bytes(changed),7,b''))
        capacity_row=next(i['row'] for i in prepared['index'] if i['family']=='synthetic_empty255')
        changed=bytearray(input_record(capacity_row));last=16+254*probe.RECORD_BYTES
        struct.pack_into('<2I',changed,last,2,2);changed[last+8:last+12]=b'limb'
        negatives.append(('potentialCapacityExceeded',bytes(changed),7,b''))
        checks=[]
        for label,data,exit,reply in negatives:
            run_bytes('protocol-'+label,data,exit,reply);checks.append(dict(case=label,exit=exit,exact=True))
        save_json(a.out/'protocol-checks.json',checks)
        result=reviewed_run(host,a.input_dir/'inputs.bin',a.out,'corpus',180)
        assert result['exit']==0 and not result['stderr_bytes']
        normalized=normalize(a.input_dir/'inputs.bin',result['output'],a.out/'native-projection.bin',a.input_dir/'python-output.bin')
        run=dict(schema='bounded-original-optional-foot-run-v1',host_exe_sha256=HOST,host_source_sha256=ASM,
            input_sha256=INPUT,expected_sha256=EXPECTED,protocol_checks=len(checks),elapsed_seconds=time.perf_counter()-started,
            **normalized,scope=manifest['limits'])
        save_json(a.out/'run.json',run);assert normalized['rows']==ROWS and normalized['exact'] and normalized['projection_sha256']==EXPECTED
        print(json.dumps(run,indent=2))


if __name__=='__main__':main()
