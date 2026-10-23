"""Run a reviewed bounded curve host against source properties and finite cases.

Flat requests/replies and source PROP payloads remain ignored target evidence.
Every real C951EB63 copy is independently found through all package indexes.
This script must be run only after review of the specified host manifest.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import random
import struct
import subprocess

from anim_rig_mapping_probe import properties
from dbpf_extract import payload
from gmdl_skin_probe import indexes

SOURCE_PROPERTY_SHA256='c00576f276d3e3daa9f898985775ba13e173d66b492bc8e8fcf0b7b3b25a6462'


def bits(value):return struct.unpack('<I',struct.pack('<f',value))[0]
def f32(value):return struct.unpack('<f',struct.pack('<f',value))[0]


def adjacent(value,upward):
    word=bits(value)
    if word&0x7fffffff==0:word=1 if upward else 0x80000001
    else:word+=1 if (value>0)==upward else -1
    return struct.unpack('<f',struct.pack('<I',word))[0]


def request(ident,query,points,control,mode=1):
    assert len(points)<=16
    result=struct.pack('<3IHH4I',ident,bits(query),len(points),control,0,mode,0,0,0)
    result+=b''.join(struct.pack('<2f',*point)for point in points)
    return result+bytes((16-len(points))*8)


def run_host(host,data):
    return subprocess.run([str(host)],input=data,capture_output=True,
        creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)


def source_curves(root,out):
    sources=[];curves=[];packages=0
    for package in sorted(root.rglob('*.package')):
        packages+=1
        for entry in indexes(package):
            if entry['instance']!=0xc951eb63:continue
            assert entry['type']==0x00b1b104 and entry['group']==0x1ffff,entry
            data=payload(package,entry)
            digest=hashlib.sha256(data).hexdigest()
            assert digest==SOURCE_PROPERTY_SHA256
            ordinal=len(sources);name=f'source-{ordinal}.prop'
            (out/name).write_bytes(data)
            fields=properties(data);assert len(fields)==18
            total=0
            for ident,kind,spec,raw in fields:
                assert kind==0x30 and spec==0x9c
                count,stride=struct.unpack_from('>2I',raw)
                assert stride==8 and len(raw)==8+count*8
                points=[struct.unpack_from('<2f',raw,8+index*8)for index in range(count)]
                assert all(math.isfinite(v)for p in points for v in p)
                assert all(a[0]<b[0]for a,b in zip(points,points[1:]))
                curves.append((ordinal,ident,points));total+=count
            assert total==104
            sources.append(dict(package=package.relative_to(root).as_posix(),
                key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}",
                raw_file=name,sha256=digest,properties=18,points=104))
    assert packages==82 and len(sources)==2,(packages,len(sources))
    return sources,curves


def queries(points,seed):
    proposed=[-1.,0.,1.,1000.,adjacent(points[0][0],False),adjacent(points[-1][0],True)]
    for x,_ in points:proposed.extend([adjacent(x,False),x,adjacent(x,True)])
    for (left,_),(right,_)in zip(points,points[1:]):
        proposed.extend(f32(left+(right-left)*fraction)for fraction in [.25,.5,.75])
    proposed.extend(float(value)for value in range(131))
    rng=random.Random(seed)
    proposed.extend(f32(rng.uniform(points[0][0]-4.,points[-1][0]+4.))for _ in range(64))
    unique={bits(value):value for value in proposed}
    return list(unique.values())


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('install',type=Path)
    ap.add_argument('--host-dir',type=Path,required=True)
    ap.add_argument('--reviewed-host-sha256',required=True)
    ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args()
    target=(Path(__file__).resolve().parent.parent/'target').resolve()
    args.out.resolve().relative_to(target);args.host_dir.resolve().relative_to(target)
    manifest=json.loads((args.host_dir/'manifest.json').read_text())
    host=(args.host_dir/'host-x86.exe').resolve()
    digest=hashlib.sha256(host.read_bytes()).hexdigest()
    assert digest==manifest['host_exe_sha256']==args.reviewed_host_sha256
    assert manifest['protocol']['input_bytes']==160 and manifest['protocol']['output_bytes']==16
    assert manifest['replacement_provider']['immutable_objects_and_vtables']
    args.out.mkdir(parents=True,exist_ok=False)
    sources,curves=source_curves(args.install,args.out)

    smoke=request(0x12345678,1.,[[0.,-0.],[1.,1.]],0x27f)
    result=run_host(host,smoke)
    (args.out/'smoke-input.bin').write_bytes(smoke)
    (args.out/'smoke-output.bin').write_bytes(result.stdout)
    assert result.returncode==0 and not result.stderr and len(result.stdout)==16,(result.returncode,len(result.stdout),result.stderr)
    assert struct.unpack('<4I',result.stdout)==(0x3f7fff00,1,1,1)
    malformed=[('empty',b'',0,0),('partial1',smoke[:1],7,0),
        ('partial159',smoke[:159],7,0),('complete_then_partial',smoke+b'x',7,16)]
    for name,offset,value,width in [('count17',8,17,'I'),('inline_count2',16,2,'I'),
        ('missing_nonzero_count',16,0,'I'),('invalid_mode',16,3,'I'),
        ('reserved_precision',12,0x17f,'H'),('unmasked_control',12,0x270,'H'),
        ('nonfinite_query',4,0x7fc00000,'I')]:
        invalid=bytearray(smoke);struct.pack_into('<'+width,invalid,offset,value)
        malformed.append((name,bytes(invalid),7,0))
    negatives=[]
    for name,data,status,size in malformed:
        result=run_host(host,data)
        assert result.returncode==status and len(result.stdout)==size and not result.stderr,(name,result.returncode,len(result.stdout))
        negatives.append(dict(case=name,exit=status,reply_bytes=size,exact=True))
    (args.out/'protocol-checks.json').write_text(json.dumps(negatives,indent=2))

    # Each original property copy has every own row retained. The Rust verifier
    # decodes these PROP files itself, independently of Python's point walk.
    records=[];metadata=[];policy_counts=Counter();source_cases=0
    for precision in [0,0x200,0x300]:
        for rounding in [0,0x400,0x800,0xc00]:
            control=0x7f|precision|rounding
            before=len(records)
            for resource,ident,points in curves:
                for query in queries(points,ident):
                    ordinal=len(records)
                    records.append(request(ident,query,points,control))
                    metadata.append((ordinal,resource,ident,bits(query),control))
                    source_cases+=1
            for mode in [0,1,2]:
                for query in [0.,-0.,float('nan')]:
                    records.append(request(0x89abcdef,query,[],control,mode))
            for mode in [1,2]:
                for query in [-1.,0.,1.,2.]:
                    records.append(request(0x89abcdef,query,[[0.,-0.]],control,mode))
            rng=random.Random(0xe837f0)
            families=[([[1.,2.],[1.,4.],[2.,6.]],[0.,1.,1.5,2.,3.]),
                ([[0.,2.],[4.,8.],[2.,12.]],[-1.,0.,1.,3.,4.,5.]),
                ([[0.,-0.],[1.,0.]],[0.,.25,.5,1.]),
                ([[0.,1.e-38],[1.e-38,-1.e-38]],[0.,f32(5.e-39),f32(1.e-38)]),
                ([[16777216.,-123.],[16777220.,37.]],[16777214.,16777216.,16777218.,16777220.,16777222.])]
            for _ in range(256):
                count=rng.choice([2,4,8,16]);xs=[f32(rng.uniform(-100.,100.))for _ in range(count)]
                if rng.randrange(3):xs.sort()
                points=[[x,f32(rng.uniform(-10000.,10000.))]for x in xs]
                samples=[f32(rng.uniform(-110.,110.))for _ in range(4)]
                families.append((points,samples))
            for points,samples in families:
                for query in samples:records.append(request(0x89abcdef,query,points,control))
            policy_counts[f'{control:04X}']=len(records)-before
    data=b''.join(records)
    (args.out/'input.bin').write_bytes(data)
    result=run_host(host,data)
    (args.out/'output.bin').write_bytes(result.stdout)
    (args.out/'stderr.bin').write_bytes(result.stderr)
    assert result.returncode==0 and not result.stderr
    assert len(result.stdout)==16*len(records)
    finite=0;input_policy=Counter()
    for ordinal,record in enumerate(records):
        output=result.stdout[16*ordinal:16*(ordinal+1)]
        values=struct.unpack('<4I',output)
        mode=struct.unpack_from('<I',record,16)[0]
        assert values[1:]==(1,int(mode!=0),int(mode!=0)),ordinal
        finite+=math.isfinite(struct.unpack_from('<f',output)[0])
        count=struct.unpack_from('<I',record,8)[0]
        query_finite=math.isfinite(struct.unpack_from('<f',record,4)[0])
        input_policy['default_cases']+=count==0
        input_policy['nonfinite_unconsumed_query_cases']+=count==0 and not query_finite
        input_policy['nonfinite_consumed_query_cases']+=count>0 and not query_finite
        input_policy['nonfinite_active_point_cases']+=any(not math.isfinite(value)
            for index in range(count)for value in struct.unpack_from('<2f',record,32+8*index))
    with (args.out/'source-cases.tsv').open('w',encoding='utf-8',newline='')as log:
        log.write('native_case_ordinal\tproperty_file\tproperty_id\tquery_bits\tcw\texpected_bits\n')
        for ordinal,resource,ident,query,control in metadata:
            expected=struct.unpack_from('<I',result.stdout,16*ordinal)[0]
            log.write(f'{ordinal}\t{sources[resource]["raw_file"]}\t{ident:08X}\t{query:08X}\t{control:04X}\t{expected:08X}\n')
    report=dict(source_sha256=manifest['source_sha256'],host_exe_sha256=digest,
        original_x86_instructions_executed=True,retail_process_or_initializer_run=False,
        property_manager_stubbed=True,native_control_word_established=False,explicit_MXCSR='1F80',
        source_resources=sources,source_curve_copies=len(curves),unique_source_curves=18,
        unique_source_points=104,source_cases=source_cases,synthetic_cases=len(records)-source_cases,
        cases=len(records),cases_per_explicit_CW=dict(policy_counts),finite_outputs=finite,
        nonfinite_outputs=len(records)-finite,protocol_checks=len(negatives),
        rejected_invalid_protocol_checks=sum(item['exit']!=0 for item in negatives),
        accepted_empty_protocol_checks=sum(item['case']=='empty' and item['exit']==0 for item in negatives),
        host_exit=result.returncode,
        nonfinite_input_policy=dict(input_policy),
        input_output_guards_and_provider_counts_intact=True,
        input_sha256=hashlib.sha256(data).hexdigest(),output_sha256=hashlib.sha256(result.stdout).hexdigest(),
        source_tsv_sha256=hashlib.sha256((args.out/'source-cases.tsv').read_bytes()).hexdigest(),
        scope='Native property-curve math under12explicit contexts, actual18Vector2 arrays/104points from both original copies, checked mock source provider. No live capability cache, actor integration or full game.')
    (args.out/'inventory.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2))


if __name__=='__main__':main()
