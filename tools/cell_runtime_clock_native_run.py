"""Preflight a pinned clock packet; --run is reserved for root admission.

Only the reviewed owned host may execute. Every attempt and true failure stays
retained. This clock gate is separate from root-frame joins and live routing.
"""
import argparse
from collections import Counter
import csv
from fractions import Fraction
import json
import math
from pathlib import Path
import struct

import cell_runtime_clock_native_probe as probe
from cell_runtime_data_native_run import digest,pinned,reviewed_file_run
from cell_schema_probe import SourceImage,SOURCE_SHA256

ROOT=Path(__file__).resolve().parent.parent
HOST='9b78caa91da34bcdbc26007c3f14f58fc2e9e9c8ba3c14d98eab62d4fd1fb72b'
ASM='16b5ea19f5c942dfd608639ee1abe0430d28c8a11b482fb9e87f5935a721353f'
OWNED='52b09f57cfcb85f565077d2fa4e57f06e31b3dc8031fff65066f59be5bce83f5'
EXPECTED={
 'tools/cell_runtime_clock_native_probe.py':'393af5d597c3587652c34e123c4c274bf676e66b99bd4fe0833be767c877e74b',
 'tools/cell_runtime_clock_inputs.py':'8013d4e8d5282fdd63f30b28b648a14718abc805d5efd475aca235c8856f76b0',
 'tools/cell_runtime_data_native_run.py':'9673df5b44c40c7e92521ae15d0549d831203de33bdc523775bf048f93530856',
 'tools/cell_schema_probe.py':'05c9e3586e021b9de17668a8cce7ebc8c7ccbee9388dab25e9e4ca9cfc4493fe',
 'tools/cell_native_math_probe.py':'a09ab9fa0b698892f6a8b116037cb6dffb54712e691e5f90c52f8d74abf05535',
 'tools/native_crt_acos_probe.py':'593ba6b27d75faa4d8af6d42310021d8a4bd15c5abe5474c5f93b468d346834b',
 'tools/cell_runtime_curves_fixture.py':'dd7d0f795fb9d3855572686853b507db722f6bf1f3f73803493780144986fa8c',
 'target/source-cell/native-runtime-clock-host-v1/manifest.json':'f9b188900f7b8492e45a572d8bfc5cdfa9b5d0c546fb2579ade222851ce11f9f',
 'target/source-cell/native-runtime-clock-host-v1/host-x86.asm':ASM,
 'target/source-cell/runtime-clock-inputs-v1/report.json':'3e466800482c10f44d2273c1c7ae878bdd2a11dd7d88d21962e339d663435466',
 'target/source-cell/runtime-clock-inputs-v1/static-preparation-review.json':'655154ffbd4f48e39e0fec3e8cdab53617ec62965e80e219e5575fbf690be50b',
 'target/source-cell/native-runtime-clock-host-root-reproduction-v1/linked-audit.json':'9444e45bb97a28c4a788673155ca1fcfda771bd72484da4ab1e7a5c3eda7a839',
}
def save(path,value):
    with path.open('x')as output:json.dump(value,output,indent=2);output.write('\n')
def word(raw,at):return struct.unpack_from('<I',raw,at)[0]
def f32(bits):return struct.unpack('<f',struct.pack('<I',bits))[0]
def f64(bits):return struct.unpack('<d',struct.pack('<Q',bits))[0]

def request_closure(input_path,table_path):
    families,contexts,modes=Counter(),Counter(),Counter();count=0
    assert input_path.stat().st_size==1560*64
    with input_path.open('rb')as binary,table_path.open(newline='')as table:
        for meta in csv.DictReader(table,delimiter='\t'):
            raw=binary.read(64);assert len(raw)==64
            mode,cw,reserved,time,previous,duration,frames,loop=struct.unpack_from('<IHHIQQII',raw)
            assert mode in[0,1]and reserved==loop==0 and raw[36:]==bytes(28)
            assert cw&0xF0FF==0x7F and cw&0x300!=0x100
            assert math.isfinite(f32(time))and math.isfinite(f64(duration))and f64(duration)>0
            assert math.isfinite(f32(frames))and f32(frames)>=1
            assert not mode or math.isfinite(f64(previous))
            exact=Fraction.from_float(f32(time))+(Fraction.from_float(f64(previous))if mode else 0)
            bound=Fraction.from_float(f64(duration));assert 0<=exact<bound
            # Cohort deltas cannot cross duration after any declared rounding.
            # These are exact domain bounds, not a floating-point output oracle.
            if mode:assert(exact<=Fraction(1)+Fraction(1,2**52)and bound==2)or(exact==Fraction(1,2)and bound==1)
            assert int(meta['case'])==count and int(meta['cw'],16)==cw and int(meta['mode'])==mode
            assert int(meta['time_or_delta_bits'],16)==time and int(meta['previous_time_bits'],16)==previous
            assert int(meta['duration_bits'],16)==duration and int(meta['frames_bits'],16)==frames
            families[meta['family']]+=1;contexts[f'{cw:04X}']+=1;modes[str(mode)]+=1;count+=1
        assert count==1560 and binary.read(1)==b''
    return dict(cases=count,families=dict(families),contexts=dict(contexts),modes=dict(modes))

def preflight(args):
    for name,expected in EXPECTED.items():assert digest(ROOT/name)==expected,name
    original=ROOT/'target/source-cell/native-runtime-clock-host-v1'
    manifest=json.loads((args.host_dir/'manifest.json').read_text())
    linked=json.loads((args.host_dir/'linked-audit.json').read_text())
    prepared=json.loads((args.input_dir/'report.json').read_text())
    review=json.loads((args.input_dir/'static-preparation-review.json').read_text())
    assert manifest==json.loads((original/'manifest.json').read_text())
    assert manifest['source_sha256']==SOURCE_SHA256 and manifest['host_source_sha256']==ASM
    assert digest(args.host_dir/'host-x86.exe')==HOST and digest(args.host_dir/'host-x86.asm')==ASM
    assert linked['host_exe_sha256']==HOST and linked['owned_code_sha256']==OWNED
    assert linked['source_bytes_exact']and linked['sections_and_guards_checked']
    for name,expected in review['pins'].items():assert digest(ROOT/name)==expected,name
    assert digest(args.host_dir/'manifest.json')==prepared['host_manifest_sha256']==review['host_manifest_sha256']
    assert review['source_input_domain_bounds_verified']and not review['arithmetic_oracle']
    for name,expected in prepared['hashes'].items():assert digest(args.input_dir/name)==expected,name
    assert digest(args.source_census/'report.json')==prepared['source_census_report_sha256']
    assert digest(args.source_census/'components.jsonl')==prepared['source_components_sha256']
    image=SourceImage(args.source_exe)
    assert probe.audit_source(image)==manifest['code_allowlist']
    assert probe.host_source(image)==(args.host_dir/'host-x86.asm').read_text()
    assert probe.linked_audit(args.host_dir/'host-x86.exe',args.host_dir/'host-x86.map',image)==linked
    pe=probe.PE(args.host_dir/'host-x86.exe');at=struct.unpack_from('<I',pe.data,0x3C)[0]
    entry=pe.base+struct.unpack_from('<I',pe.data,at+24+16)[0]
    assert entry==int(linked['symbols']['HostMain'],16)
    closure=request_closure(args.input_dir/'input.bin',args.input_dir/'cases.tsv')
    assert closure['families']==prepared['families']and closure['contexts']==prepared['contexts']
    assert prepared['cases']==1560 and len(prepared['protocols'])==16
    for item in prepared['protocols']:
        assert item['expected_exit']==7 and item['expected_output_bytes']==0
        assert(args.input_dir/item['path']).stat().st_size in[63,64]
    return prepared,review,closure,entry

def inspect(request_path,reply_path,projection_path=None):
    count,remainder=divmod(request_path.stat().st_size,64);assert not remainder
    assert reply_path.stat().st_size==count*28
    contexts,sticky=Counter(),Counter()
    projection=projection_path.open('xb')if projection_path is not None else None
    try:
        with request_path.open('rb')as requests,reply_path.open('rb')as replies:
            for case in range(count):
                req,reply=requests.read(64),replies.read(28)
                cw=struct.unpack_from('<H',req,4)[0];actual_cw,sw,mxcsr=struct.unpack_from('<HHI',reply,20)
                assert cw==actual_cw and sw&0x3800==0 and mxcsr&~0x3F==0x1F80
                time=word(reply,0);stored=struct.unpack_from('<Q',reply,4)[0];ratio,frame=word(reply,12),word(reply,16)
                assert math.isfinite(f32(time))and 0<=f32(time)<f64(struct.unpack_from('<Q',req,20)[0])
                assert stored==struct.unpack('<Q',struct.pack('<d',f32(time)))[0]
                assert math.isfinite(f32(ratio))and 0<=f32(ratio)<=1
                assert math.isfinite(f32(frame))and f32(frame)>=0
                if word(req,0)==0:assert time==word(req,8)
                if projection is not None:projection.write(reply[:20])
                contexts[f'{cw:04X}']+=1;sticky[f'{mxcsr&0x3F:02X}']+=1
    finally:
        if projection is not None:projection.close()
    return dict(cases=count,contexts=dict(contexts),mxcsr_sticky=dict(sticky),metadata_checked=True,projection_bytes_per_case=20)

def execute(args,prepared,review):
    native=args.out/'native-v1';native.mkdir(exist_ok=False);protocol=native/'protocol';protocol.mkdir()
    raw=(args.input_dir/'input.bin').read_bytes();smoke=raw[:64]
    records=[raw[i:i+64]for i in range(0,len(raw),64)]
    other=next(r for r in records if word(r,0)==1 and word(r,8)==0x3E800000 and word(r,28)==0x43050000)
    for label,record in [('reference_absolute',smoke),('reference_delta',other)]: (protocol/f'{label}-input.bin').write_bytes(record)
    empty=protocol/'empty-input.bin';empty.write_bytes(b'')
    sequence=protocol/'valid-sequence-input.bin';sequence.write_bytes(smoke+other+smoke+other)
    partial=protocol/'valid-then-partial-input.bin';partial.write_bytes(smoke+b'\x00')
    paths=[args.source_exe,Path(__file__),args.out/'preflight.json',args.input_dir/'report.json',args.input_dir/'static-preparation-review.json',
        args.source_census/'report.json',args.source_census/'components.jsonl',
        *[ROOT/name for name in EXPECTED],*[ROOT/name for name in review['pins']],
        *[args.input_dir/name for name in prepared['hashes']],
        *[p for p in args.host_dir.iterdir()if p.is_file()],
        *[p for p in protocol.iterdir()if p.is_file()]]
    paths=list(dict.fromkeys(p.resolve()for p in paths))
    with pinned(paths,native):
        host=(args.host_dir/'host-x86.exe').resolve();references=[]
        for label in ['reference_absolute','reference_delta']:
            request=protocol/f'{label}-input.bin';status,reply=reviewed_file_run(host,request,protocol,label)
            assert status['exit']==0 and status['stderr_bytes']==0 and not status['timeout'] and status['exception']is None,status
            inspect(request,reply);references.append(reply)
        # Generated references are now pinned through every dependent protocol.
        reference_pins=native/'reference-pins';reference_pins.mkdir()
        with pinned(references,reference_pins):
            expected=[p.read_bytes()for p in references]
            protocols=[('empty_eof',empty,0,b''),('valid_sequence',sequence,0,expected[0]+expected[1]+expected[0]+expected[1]),
                ('valid_then_partial',partial,7,expected[0])]
            protocols +=[(item['name'],args.input_dir/item['path'],7,b'')for item in prepared['protocols']]
            results=[]
            for label,request,code,expected_reply in protocols:
                status,reply=reviewed_file_run(host,request,protocol,label)
                assert status['exit']==code and status['stderr_bytes']==0 and not status['timeout']and status['exception']is None,status
                assert reply.read_bytes()==expected_reply,(label,status)
                if code==0:inspect(request,reply)
                results.append(dict(name=label,exit=code,output_bytes=len(expected_reply)))
            save(protocol/'report.json',dict(protocol_checks=len(results),reference_cases=2,checks=results))
        status,reply=reviewed_file_run(host,args.input_dir/'input.bin',native,'corpus')
        assert status['exit']==0 and status['stderr_bytes']==0 and not status['timeout']and status['exception']is None,status
        projection=native/'native-projection20.bin';summary=inspect(args.input_dir/'input.bin',reply,projection)
        assert summary['cases']==1560
        result=dict(schema='original-finite-first-duration-clock-run-v1',**summary,protocol_checks=19,reference_cases=2,
            host_exe_sha256=HOST,host_source_sha256=ASM,raw_native_path=str(reply.resolve()),raw_native_sha256=digest(reply),
            native_projection_sha256=digest(projection),native_executed=True,rust_gate_accepted=False,
            scope='Original first-duration/nonloop time and delta arithmetic only; exact partial entry/exit contracts. No looping, events, crossings, active clip routing or root-data clock variant admission. Root Rust comparison remains separate.')
        save(native/'report.json',result);print(json.dumps(result,indent=2))

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir',type=Path,default=ROOT/'target/source-cell/native-runtime-clock-host-root-reproduction-v1')
    p.add_argument('--input-dir',type=Path,default=ROOT/'target/source-cell/runtime-clock-inputs-v1')
    p.add_argument('--source-census',type=Path,default=ROOT/'target/source-cell/runtime-curves-source-census-v1')
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--out',type=Path,required=True);p.add_argument('--run',action='store_true');args=p.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve());prepared,review,closure,entry=preflight(args)
    args.out.mkdir(parents=True,exist_ok=False)
    save(args.out/'preflight.json',dict(schema='clock-native-run-preflight-v1',**closure,host_exe_sha256=HOST,host_source_sha256=ASM,
        owned_code_sha256=OWNED,entrypoint=f'{entry:08X}',entrypoint_matches_HostMain=True,source_spans_and_linked_audit_reproduced=True,
        runner_sha256=digest(Path(__file__)),native_execution_requested=args.run))
    if not args.run:print(json.dumps(dict(preflight=True,cases=1560,native_executed=False)));return
    try:execute(args,prepared,review)
    except BaseException as error:
        save(args.out/'failure.json',dict(error=repr(error),true_failure_retained=True));raise
if __name__=='__main__':main()
