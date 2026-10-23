"""Run a reviewed skip-flag Cell cache host using source-derived and raw rows.

Source-derived classes are explicitly assigned fixture modes, not a substitute
for the unresolved live class resolver. Original geometry inputs come from the
already matched native/Rust E83E60 corpus. Large request streams stay on disk.
"""
import argparse
from collections import Counter
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path
import random
import struct
import subprocess


def digest(path):
    value=hashlib.sha256()
    with path.open('rb')as stream:
        while block:=stream.read(1048576):value.update(block)
    return value.hexdigest()


def part(flag,kind,center=(0.,0.,0.),radius=0.):
    value=bytearray([0xa5]*72);value[0]=flag
    struct.pack_into('<I3f',value,4,kind,*center);struct.pack_into('<f',value,56,radius)
    return bytes(value)


def request(rows,control=0x27f,empty=(0.,0.,0.),tail=None):
    assert len(rows)<=64 and all(len(row)==72 for row in rows)
    tail=bytes([0xa5])*140 if tail is None else tail;assert len(tail)==140
    return struct.pack('<IH2x3f12x',len(rows),control,*empty)+b''.join(rows)+bytes((64-len(rows))*72)+tail+bytes(4)


def small(host,data):
    return subprocess.run([str(host)],input=data,capture_output=True,
        creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)


def source_parts(root):
    review=json.loads((root/'rust-comparison.json').read_text())
    expected='a5533b75c432cb3b9938ef0f68c7ea49dd839d04d5793ddcaf9d145817fbe0fb'
    assert review['exact_bits']and digest(root/'output.bin')==digest(root/'rust-output-v1.bin')==expected
    resources=[];current=None;ordinal=0
    with (root/'source-map.tsv').open(newline='')as mapping,(root/'output.bin').open('rb')as values:
        for row in csv.DictReader(mapping,delimiter='\t'):
            assert int(row['native_case_ordinal'])==ordinal
            key=tuple(row[name]for name in ['package','type','group','instance'])
            if current is None or current['key']!=key:
                current=dict(key=key,ordinary=[],class2=[]);resources.append(current)
            record=values.read(160);assert len(record)==160
            assert struct.unpack_from('<4I',record)==(1,1,1,1)
            assert record[16:88]==record[88:160]
            which=ordinal%3;row_index=int(row['row'])
            if which in (0,1):
                field='ordinary'if which==0 else'class2'
                assert row_index==len(current[field]);current[field].append(record[16:88])
            ordinal+=1
    assert ordinal==136992 and len(resources)==1365
    assert sum(len(resource['ordinary'])for resource in resources)==45664
    for resource in resources:assert len(resource['ordinary'])==len(resource['class2'])<=64
    return resources


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--geometry-dir',type=Path,required=True)
    ap.add_argument('--host-dir',type=Path,required=True);ap.add_argument('--reviewed-host-sha256',required=True)
    ap.add_argument('--out',type=Path,required=True);args=ap.parse_args()
    target=(Path(__file__).resolve().parent.parent/'target').resolve()
    for path in [args.geometry_dir,args.host_dir,args.out]:path.resolve().relative_to(target)
    manifest=json.loads((args.host_dir/'manifest.json').read_text());host=(args.host_dir/'host-x86.exe').resolve()
    assert digest(host)==manifest['host_exe_sha256']==args.reviewed_host_sha256
    assert manifest['protocol']['input_bytes']==4784 and manifest['protocol']['output_bytes']==144
    assert manifest['post_call_guards']and manifest['indirect_calls']==0
    args.out.mkdir(parents=True,exist_ok=False)
    smoke=request([part(1,12,radius=10.)]);result=small(host,smoke)
    (args.out/'smoke-input.bin').write_bytes(smoke);(args.out/'smoke-output.bin').write_bytes(result.stdout)
    smoke_report=dict(exit=result.returncode,bytes=len(result.stdout),stderr=result.stderr.decode(errors='replace'))
    (args.out/'smoke-report.json').write_text(json.dumps(smoke_report,indent=2))
    assert result.returncode==0 and len(result.stdout)==144 and not result.stderr,smoke_report
    assert struct.unpack_from('<I',result.stdout)[0]==1
    assert struct.unpack_from('<I',result.stdout,4+12*4)[0]==2
    assert struct.unpack_from('<3f',result.stdout,56)==(0.,5.,0.)
    assert struct.unpack_from('<f',result.stdout,68)[0]==15.
    assert result.stdout[72:116]==b'\xa5'*44
    malformed=[('empty',b'',0,0),('partial1',smoke[:1],7,0),
        ('partial4783',smoke[:4783],7,0),('complete_then_partial',smoke+b'x',7,144)]
    for name,offset,value,fmt in [('count65',0,65,'I'),('class13',36,13,'I'),
        ('negative_class',36,0xffffffff,'I'),('reserved_precision',4,0x17f,'H'),('unmasked_control',4,0x270,'H')]:
        invalid=bytearray(smoke);struct.pack_into('<'+fmt,invalid,offset,value)
        malformed.append((name,bytes(invalid),7,0))
    checks=[]
    for name,data,status,size in malformed:
        result=small(host,data);item=dict(case=name,exit=result.returncode,bytes=len(result.stdout),expected_exit=status,expected_bytes=size)
        checks.append(item);assert result.returncode==status and len(result.stdout)==size and not result.stderr,item
    (args.out/'protocol-checks.json').write_text(json.dumps(checks,indent=2))
    resources=source_parts(args.geometry_dir)
    count=0;labels=Counter();policies=Counter();rng=random.Random(0xe51ec0)
    with (args.out/'input.bin').open('wb')as stream,(args.out/'source-map.tsv').open('w',newline='\n')as mapping:
        mapping.write('package\ttype\tgroup\tinstance\texplicit_class_mode\trow_count\tcw\tnative_case_ordinal\n')
        def emit(rows,label,control=0x27f,empty=(0.,0.,0.)):
            nonlocal count
            stream.write(request(rows,control,empty,rng.randbytes(140)))
            labels[label]+=1;policies[f'{control:04X}']+=1;count+=1
        for resource in resources:
            for mode in range(1,15):
                rows=[]
                for index,(ordinary,class2)in enumerate(zip(resource['ordinary'],resource['class2'])):
                    kind=mode if mode<=12 else 1+(index%12 if mode==13 else(11-index)%12)
                    row=bytearray(class2 if kind==2 else ordinary);struct.pack_into('<I',row,4,kind);rows.append(bytes(row))
                mapping.write('\t'.join(resource['key'])+f'\t{mode}\t{len(rows)}\t027F\t{count}\n')
                emit(rows,'source_derived_explicit_class_modes')
        source_cases=count
        for precision in [0,0x200,0x300]:
            for rounding in [0,0x400,0x800,0xc00]:
                control=0x7f|precision|rounding
                for size in range(65):
                    rows=[part([0,1,2,255][i%4],i%13,(float(i%7)-3.,float(i%11)-5.,float(i%5)-2.),float(i%3))for i in range(size)]
                    emit(rows,'synthetic_all_counts',control,(1.,-2.,3.))
                for kind in range(13):
                    for flag in [0,1,2,255]:
                        for y in [-10_000.,-1.,-0.,0.,1.,1_000.,1000.00006103515625,10_000.,10000.0009765625]:
                            emit([part(flag,kind,(-0.,y,0.),1.)],'synthetic_class_flag_threshold',control)
                choices=[part(1,5,(0.,1.,0.)),part(1,5,(0.,-1.,0.)),
                    part(0,5,(1.,1000.,0.)),part(255,5,(-1.,1000.,0.))]
                for mask in range(16):
                    picked=[row for index,row in enumerate(choices)if mask&(1<<index)]
                    for rows in itertools.permutations(picked):emit(rows,'synthetic_class5_order',control)
                for propulsion in [0,1,2,3,4,8,16,32]:
                    for last in [0,2,4,7,5]:
                        rows=[part(1,10,radius=1.)]*propulsion+[part(0,6,radius=1.)]*4+[part(1,7,radius=1.)]*4
                        rows+=[part(1,last,(0.,-1.,0.),1.)]
                        emit(rows,'synthetic_integer_stat_branches',control)
                for value in [0,0x80000000,1,0x80000001,0x7fffff,0x807fffff,0x3f800001,0x4b800001]:
                    x=struct.unpack('<f',struct.pack('<I',value))[0]
                    emit([part(0,1,(x,x,x),x)],'synthetic_radius_boundaries',control)
                for _ in range(128):
                    size=rng.choice([0,1,2,3,4,5,7,8,16,63,64]);rows=[]
                    for index in range(size):
                        center=tuple(math.ldexp(rng.uniform(-2.,2.),rng.randrange(-120,101))for _ in range(3))
                        radius=math.ldexp(rng.uniform(-2.,2.),rng.randrange(-120,101))
                        # Keep flags outside the extent path to avoid SSE-bound
                        # overflow while stressing retained x87 radius arithmetic.
                        rows.append(part(rng.choice([0,2,255]),rng.randrange(13),center,radius))
                    emit(rows,'synthetic_retained_radius_random',control)
    print(f'CAPABILITY_GEOMETRY_NATIVE cases={count} source_cases={source_cases} input_bytes={count*4784}',flush=True)
    with (args.out/'input.bin').open('rb')as inp,(args.out/'output.bin').open('wb')as out,(args.out/'stderr.bin').open('wb')as err:
        result=subprocess.run([str(host)],stdin=inp,stdout=out,stderr=err,
            creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)
    run=dict(exit=result.returncode,input_bytes=(args.out/'input.bin').stat().st_size,
        output_bytes=(args.out/'output.bin').stat().st_size,stderr_bytes=(args.out/'stderr.bin').stat().st_size)
    (args.out/'run.json').write_text(json.dumps(run,indent=2))
    assert result.returncode==0 and run['output_bytes']==count*144 and run['stderr_bytes']==0,run
    with (args.out/'output.bin').open('rb')as stream:
        for ordinal in range(count):
            raw=stream.read(144);assert struct.unpack_from('<I',raw)[0]==1
            assert all(math.isfinite(v)for v in struct.unpack_from('<4f',raw,56)),ordinal

    # Nonfinite consumed values are retained in a separate stream, with explicit
    # checked-API rejection expectations rather than finite agreement exclusions.
    nonfinite=[];base=bytearray(part(0,1))
    for name,offset,value in [('nan_center_x',8,0x7fc01234),('negative_infinity_center_y',12,0xff800000),('infinite_radius',56,0x7f800000)]:
        row=bytearray(base);struct.pack_into('<I',row,offset,value)
        nonfinite.append((name,request([row]),'NonFiniteInput'))
    nonfinite.append(('nan_empty_center',request([],empty=(float('nan'),0.,0.)),'NonFiniteInput'))
    nonfinite.append(('finite_extent_overflow',request([part(1,1,(0.,3.4e38,0.),3.4e38)]),'NonFiniteIntermediate'))
    data=b''.join(record for _,record,_ in nonfinite);result=small(host,data)
    (args.out/'nonfinite-input.bin').write_bytes(data);(args.out/'nonfinite-output.bin').write_bytes(result.stdout)
    assert result.returncode==0 and len(result.stdout)==len(nonfinite)*144 and not result.stderr
    (args.out/'nonfinite-policy.json').write_text(json.dumps([dict(case=name,expected_checked_error=error)for name,_,error in nonfinite],indent=2))
    hashes={name:digest(args.out/name)for name in ['input.bin','output.bin','source-map.tsv','nonfinite-input.bin','nonfinite-output.bin']}
    report=dict(host_exe_sha256=manifest['host_exe_sha256'],source_exe_sha256=manifest['source_sha256'],
        cases=count,source_derived_cases=source_cases,source_creature_copies=len(resources),
        source_geometry_rows=45664,source_class_modes=14,source_context='027F',labels=dict(labels),
        policy_counts=dict(policies),protocol_checks=len(checks),native_exit=run['exit'],
        count_class_buffers_untouched_prefix_and_post_call_stack_FP_guards_checked=True,
        finite_case_exclusions=0,nonfinite_consumed_cases_separate=len(nonfinite),hashes=hashes,
        mxcsr_control='1F80',scope='OriginalE51EC0skipflagprefix only. Source geometry real; classes explicitfixturemodes, emptycenterexplicit. No original classresolver, latercurve/maskpath orliveactorbinding.')
    (args.out/'report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))


if __name__=='__main__':main()
