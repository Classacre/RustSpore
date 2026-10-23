"""Prepare full scalar-cache goldens; execute only a separately reviewed host.

Default preparation never starts the generated binary. Source counts come from
the verified actual-class composition; both source PROP/Globals copies remain
separate lanes. All streams, raw assets and outputs stay ignored under target.
"""
import argparse
from collections import Counter
import csv
import hashlib
import json
import math
from pathlib import Path
import random
import struct
import subprocess
import time

from anim_rig_mapping_probe import properties
from cell_capability_scalar_probe import IDS, INPUT_BYTES, OUTPUT_BYTES
from cell_schema_probe import SOURCE_SHA256

PROPERTY_SHA='c00576f276d3e3daa9f898985775ba13e173d66b492bc8e8fcf0b7b3b25a6462'
COMPOSITION_SHA='a2a9964d1a5532970ed4a68106e472ac53329e3afdb57c51b9e37f4d9849e549'
DYNAMIC_ID=0x5a38c4ba


def digest(path):
    h=hashlib.sha256()
    with path.open('rb')as stream:
        while value:=stream.read(1048576):h.update(value)
    return h.hexdigest()


def request(counts,weights,points,control=0x27f,tail=None,present=1,storage=None,dynamic_id=DYNAMIC_ID):
    assert len(counts)==13 and all(0<=value<=128 for value in counts)
    assert len(weights)==5 and len(points)==9
    storage=[0]*9 if storage is None else storage
    tail=bytearray(140)if tail is None else bytearray(tail);assert len(tail)==140
    struct.pack_into('<13I',tail,0,*counts)
    out=struct.pack('<HHI5fI',control,0,dynamic_id,*weights,present)+tail+bytes(20)
    for mode,rows in zip(storage,points):
        assert mode in (0,1)and len(rows)<=(1 if mode else 16)
        out+=struct.pack('<4I',mode,len(rows),0,0)+b''.join(struct.pack('<2f',*row)for row in rows)+bytes((16-len(rows))*8)
    assert len(out)==INPUT_BYTES
    return out


def source_curves(args):
    rows=[]
    for ordinal in range(2):
        path=args.curve_dir/f'source-{ordinal}.prop';raw=path.read_bytes()
        assert hashlib.sha256(raw).hexdigest()==PROPERTY_SHA
        name=f'curve-{ordinal}.prop';(args.out/name).write_bytes(raw)
        fields=properties(raw);assert len(fields)==18
        found={}
        for ident,kind,spec,value in fields:
            assert kind==0x30 and spec==0x9c
            count,stride=struct.unpack_from('>2I',value);assert stride==8 and len(value)==8+count*8
            found[ident]=[struct.unpack_from('<2f',value,8+i*8)for i in range(count)]
        assert len(found)==18 and sum(map(len,found.values()))==104
        points=[found[DYNAMIC_ID if ident is None else ident]for ident in IDS]
        assert all(len(value)<=16 for value in points)
        rows.append((name,points))
    return rows


def source_globals(args):
    inventory=json.loads((args.globals_dir/'inventory.json').read_text())
    data=(args.globals_dir/'corpus.bin').read_bytes();assert data[:8]==b'RSCELLG1'
    assert hashlib.sha256(data).hexdigest()==inventory['fixture_sha256']
    count=struct.unpack_from('<I',data,8)[0];assert count==len(inventory['records'])==6
    at=12;rows=[];excluded=[]
    for ordinal,record in enumerate(inventory['records']):
        size,supported,_semantic=struct.unpack_from('<IBQ',data,at);at+=13
        raw=data[at:at+size];at+=size;assert hashlib.sha256(raw).hexdigest()==record['sha256']
        if 'synthetic'in record['source']:continue
        name=f'globals-{ordinal}.bin';(args.out/name).write_bytes(raw)
        if not supported:
            assert size==264;excluded.append(dict(file=name,size=size,reason='unsupported original older264-byte Globals schema'));continue
        assert size==276
        weights=[struct.unpack_from('<f',raw,offset)[0]for offset in [0x74,0x78,0x7c,0x80,0x90]]
        assert all(math.isfinite(value)for value in weights)
        rows.append((name,weights))
    assert at==len(data)and len(rows)==2 and len(excluded)==1
    return rows,excluded


def prepare(args,manifest):
    args.out.mkdir(parents=True,exist_ok=False)
    curves=source_curves(args);globals_rows,excluded=source_globals(args)
    assert digest(args.composition_dir/'cache-output.bin')==COMPOSITION_SHA
    creatures=list(csv.DictReader((args.composition_dir/'creatures.tsv').open(newline=''),delimiter='\t'))
    assert len(creatures)==1365
    contexts=Counter();labels=Counter();count=0;rng=random.Random(0xe524a4)
    with (args.out/'input.bin').open('wb')as stream,(args.out/'source-cases.tsv').open('w',newline='\n')as metadata:
        metadata.write('case\tcreature_case\tglobals_file\tcurve_file\tcw\n')
        def emit(data,label):
            nonlocal count
            assert len(data)==INPUT_BYTES;stream.write(data);labels[label]+=1
            contexts[f'{struct.unpack_from("<H",data)[0]:04X}']+=1;count+=1
        with (args.composition_dir/'cache-output.bin').open('rb')as native:
            for creature in range(1365):
                reply=native.read(144);assert struct.unpack_from('<I',reply)[0]==1
                tail=reply[4:];counts=struct.unpack_from('<13I',tail)
                for globals_name,weights in globals_rows:
                    for curve_name,points in curves:
                        metadata.write(f'{count}\t{creature}\t{globals_name}\t{curve_name}\t027F\n')
                        emit(request(counts,weights,points,tail=tail),'source_actual_class_counts_and_source_globals_curves')
            assert native.read(1)==b''
        source_cases=count;assert source_cases==5460
        base_points=curves[0][1];base_weights=globals_rows[0][1]
        integer_values=[-2147483648.,-1.5,-.5,-0.,0.,.5,1.5,2.5,3.5,2147483520.]
        weight_values=[-8.,-1.,-0.,0.,.125,.5,1.,8.]
        for precision in [0,0x200,0x300]:
            for rounding in [0,0x400,0x800,0xc00]:
                cw=0x7f|precision|rounding
                def synthetic(counts,weights=base_weights,points=base_points,label='synthetic',present=1,storage=None):
                    emit(request(counts,weights,points,cw,tail=rng.randbytes(140),present=present,storage=storage),label)
                for kind in range(13):
                    for value in range(129):
                        counts=[0]*13;counts[kind]=value
                        synthetic(counts,label='synthetic_each_class_all0to128_counts')
                for propulsion in range(8):
                    for diet in range(8):
                        for electric in [0,1,2,3,4,127,128]:
                            counts=[0]*13
                            for bit,kind in enumerate([8,9,10]):counts[kind]=(propulsion>>bit)&1
                            for bit,kind in enumerate([2,3,4]):counts[kind]=(diet>>bit)&1
                            counts[6]=electric;counts[7]=electric&1;counts[5]=electric;counts[11]=electric
                            synthetic(counts,label='synthetic_propulsion_diet_electric_branch_product')
                for _ in range(128):
                    counts=[rng.randrange(129)for _ in range(13)]
                    weights=[rng.uniform(-4.,4.)for _ in range(5)]
                    synthetic(counts,weights,label='synthetic_mixed_finite_counts_weights')
                for value in weight_values:
                    for mask in range(8):
                        counts=[0]*13
                        for bit,kind in enumerate([8,9,10]):counts[kind]=128 if mask&(1<<bit)else 0
                        synthetic(counts,[value]*5,label='synthetic_weight_sign_zero_mask_branches')
                for value in integer_values:
                    changed=list(base_points);changed[7]=[[0.,value]]
                    synthetic([0]*13,points=changed,label='synthetic_CVTSS2SI_nearest_ties_and_limits')
                for present in [0,1]:
                    for mode in [0,1]:
                        for size in [0,1]:
                            points=[[[0.,-0.]]*size for _ in range(9)]
                            counts=[0]*13;counts[8]=1
                            synthetic(counts,points=points,label='synthetic_empty_inline_array_missinglist',present=present,storage=[mode]*9)
    assert count==33480
    # Separate checked finite-API policy, retaining all original outcomes.
    nonfinite=[];base=[0]*13;base[10]=1
    for index,value in [(0,float('nan')),(1,float('inf'))]:
        weights=list(base_weights);weights[index]=value
        nonfinite.append(('NonFiniteInput',request(base,weights,base_points)))
    weights=list(base_weights);weights[0]=struct.unpack('<f',bytes.fromhex('ffff7f7f'))[0]
    counts=list(base);counts[8]=128
    nonfinite.append(('NonFiniteIntermediate',request(counts,weights,base_points)))
    points=list(base_points);points[0]=[[float('nan'),1.]]
    nonfinite.append(('CurveNonFiniteInput',request([0]*13,base_weights,points)))
    for value,error in [(float('nan'),'CurveNonFiniteInput'),(2147483648.,'IntegerConversionOutsideFiniteDomain')]:
        points=list(base_points);points[7]=[[0.,value]]
        nonfinite.append((error,request([0]*13,base_weights,points)))
    maximum=struct.unpack('<f',bytes.fromhex('ffff7f7f'))[0]
    points=list(base_points);points[0]=[[0.,-maximum],[2.,maximum]];counts=[0]*13;counts[11]=1
    nonfinite.append(('CurveNonFiniteIntermediate',request(counts,base_weights,points)))
    weights=list(base_weights);weights[4]=float('nan')
    nonfinite.append(('NonFiniteInput',request([0]*13,weights,base_points)))
    (args.out/'nonfinite-input.bin').write_bytes(b''.join(data for _,data in nonfinite))
    (args.out/'nonfinite-policy.tsv').write_text('case\texpected_checked_error\n'+''.join(f'{i}\t{error}\n'for i,(error,_)in enumerate(nonfinite)))
    report=dict(schema='full-original-scalar-cache-cohort-v1',host_executed=False,cases=count,source_cases=source_cases,
        synthetic_cases=count-source_cases,source_creatures=1365,source_globals_copies=2,source_curve_PROP_copies=2,
        relevant_curves_per_copy=9,source_context='027F',explicit_policy_counts=dict(contexts),labels=dict(labels),
        excluded_source_globals=excluded,nonfinite_policy_cases=len(nonfinite),finite_agreement_exclusions=0,
        explicit_dynamic_curve_id='5A38C4BA',MXCSR_control='1F80',
        host_exe_sha256=manifest['host_exe_sha256'],host_source_sha256=manifest['host_source_sha256'],
        source_composition_output_sha256=COMPOSITION_SHA,
        hashes={name:digest(args.out/name)for name in ['input.bin','source-cases.tsv','nonfinite-input.bin','nonfinite-policy.tsv',*[n for n,_ in curves],*[n for n,_ in globals_rows],*[r['file']for r in excluded]]},
        scope='Explicit resolved counts, source Globals/curve copies and mutable ID. No archive/active-product/actor or measured retail entry FP-state binding.')
    (args.out/'prepare-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


def run_small(host,data,out,name):
    start=time.perf_counter();completed=None;error=None
    try:completed=subprocess.run([str(host)],input=data,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)
    except Exception as exc:error=exc
    (out/f'{name}-input.bin').write_bytes(data)
    result=dict(exit=None if completed is None else completed.returncode,elapsed_seconds=time.perf_counter()-start,error=None if error is None else repr(error))
    if completed is not None:
        (out/f'{name}-output.bin').write_bytes(completed.stdout);(out/f'{name}-stderr.bin').write_bytes(completed.stderr)
        result.update(reply_bytes=len(completed.stdout),stderr_bytes=len(completed.stderr))
    (out/f'{name}-status.json').write_text(json.dumps(result,indent=2)+'\n')
    if error:raise error
    return completed


def inspect(input_path,output_path):
    count=input_path.stat().st_size//INPUT_BYTES;assert input_path.stat().st_size==count*INPUT_BYTES
    assert output_path.stat().st_size==count*OUTPUT_BYTES
    nonfinite=0;policies=Counter()
    with input_path.open('rb')as inp,output_path.open('rb')as out:
        for ordinal in range(count):
            request=inp.read(INPUT_BYTES);reply=out.read(OUTPUT_BYTES)
            cw=struct.unpack_from('<H',request)[0];policies[f'{cw:04X}']+=1
            actual_cw,sw=struct.unpack_from('<2H',reply,212)
            assert actual_cw==cw and sw&0x3800==0 and struct.unpack_from('<I',reply,208)[0]&~0x3f==0x1f80
            assert reply[232:240]==bytes(8)
            counts=struct.unpack_from('<13I',request,32);curve_calls=6+int(any(counts[c]for c in [8,9,10]))
            globals_calls=1 if curve_calls==7 else 2
            assert struct.unpack_from('<3I',reply,140)==(curve_calls,globals_calls,globals_calls)
            properties=curve_calls if struct.unpack_from('<I',request,28)[0]else 0
            assert struct.unpack_from('<4I',reply,216)==(globals_calls,curve_calls,properties,properties)
            assert reply[152+8*curve_calls:208]==b'\xa5'*(56-8*curve_calls)
            for offset in [72,84,88,92,96,100,108]:
                value=struct.unpack_from('<I',reply,offset)[0]
                if value&0x7f800000==0x7f800000:nonfinite+=1
    return dict(cases=count,nonfinite_scalar_output_words=nonfinite,contexts=dict(policies))


def execute(args,manifest):
    prepared=json.loads((args.out/'prepare-report.json').read_text())
    assert prepared['host_exe_sha256']==manifest['host_exe_sha256']
    assert all(digest(args.out/name)==value for name,value in prepared['hashes'].items())
    pinned_paths=[args.source_exe,*[args.host_dir/name for name in ['host-x86.asm','host-x86.exe','manifest.json']],
        args.out/'prepare-report.json',*[args.out/name for name in prepared['hashes']],
        args.composition_dir/'cache-output.bin',args.globals_dir/'corpus.bin',args.globals_dir/'inventory.json']
    pins={str(path.resolve()):digest(path)for path in pinned_paths}
    (args.out/'execution-pins-v1.json').write_text(json.dumps(pins,indent=2)+'\n')
    host=(args.host_dir/'host-x86.exe').resolve();checks=args.out/'protocol';checks.mkdir(exist_ok=False)
    try:
        with (args.out/'input.bin').open('rb')as stream:smoke=stream.read(INPUT_BYTES)
        first=run_small(host,smoke,checks,'smoke')
        assert first.returncode==0 and len(first.stdout)==OUTPUT_BYTES and not first.stderr
        alternate=bytearray(smoke);struct.pack_into('<I',alternate,4,0x89abcdef)
        alternate_result=run_small(host,bytes(alternate),checks,'explicit_alternate_id')
        assert alternate_result.returncode==0 and len(alternate_result.stdout)==OUTPUT_BYTES and not alternate_result.stderr
        malformed=[('empty',b'',0,b''),('partial1',smoke[:1],7,b''),('partial1487',smoke[:-1],7,b''),
            ('complete_then_partial',smoke+b'x',7,first.stdout),('ID_change',smoke+alternate,8,first.stdout)]
        for name,offset,value,fmt in [('count129',32,129,'I'),('points17',196,17,'I'),('storage2',192,2,'I'),
            ('reserved_PC',0,0x17f,'H'),('unmasked_CW',0,0x27e,'H'),('present2',28,2,'I'),
            ('reserved_header',2,1,'H'),('reserved_tail',172,1,'I'),('reserved_property',200,1,'I'),
            ('dynamicID_collision',4,IDS[0],'I')]:
            changed=bytearray(smoke);struct.pack_into('<'+fmt,changed,offset,value);malformed.append((name,bytes(changed),7,b''))
        changed=bytearray(smoke);struct.pack_into('<2I',changed,192,1,2);malformed.append(('inline_count2',bytes(changed),7,b''))
        protocol=[]
        for name,data,status,expected in malformed:
            completed=run_small(host,data,checks,name)
            assert completed.returncode==status and completed.stdout==expected and not completed.stderr,name
            protocol.append(dict(case=name,exit=status,reply_bytes=len(expected),exact=True))
        (args.out/'protocol-checks.json').write_text(json.dumps(protocol,indent=2)+'\n')
        runs={}
        for prefix in ['', 'nonfinite-']:
            input_path=args.out/f'{prefix}input.bin';output_path=args.out/f'{prefix}output.bin';error_path=args.out/f'{prefix}stderr.bin'
            assert not output_path.exists();completed=None;failure=None;started=time.perf_counter()
            with input_path.open('rb')as inp,output_path.open('wb')as out,error_path.open('wb')as err:
                try:completed=subprocess.run([str(host)],stdin=inp,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)
                except Exception as exc:failure=exc
            result=dict(exit=None if completed is None else completed.returncode,input_bytes=input_path.stat().st_size,
                output_bytes=output_path.stat().st_size,stderr_bytes=error_path.stat().st_size,
                elapsed_seconds=time.perf_counter()-started,error=None if failure is None else repr(failure))
            (args.out/f'{prefix}status.json').write_text(json.dumps(result,indent=2)+'\n')
            if failure:raise failure
            assert completed.returncode==0 and result['stderr_bytes']==0,result
            result.update(inspect(input_path,output_path));result['sha256']=digest(output_path);runs[prefix or'finite']=result
        assert runs['finite']['cases']==prepared['cases']and runs['finite']['nonfinite_scalar_output_words']==0
        assert runs['nonfinite-']['cases']==prepared['nonfinite_policy_cases']
        report=dict(prepared,host_executed=True,protocol_checks=len(protocol),runs=runs,
            hidden_binary_pipes=True,original_game_process=False,GUI=False)
        (args.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:
        changed=[name for name,sha in pins.items()if not Path(name).exists()or digest(Path(name))!=sha]
        (args.out/'immutability-v1.json').write_text(json.dumps(dict(pinned_files=len(pins),unchanged=not changed,changed_files=changed),indent=2)+'\n')
        assert not changed,changed


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    for name in ['host-dir','curve-dir','globals-dir','composition-dir','out','source-exe']:
        ap.add_argument('--'+name,type=Path,required=True)
    ap.add_argument('--reviewed-host-sha256',required=True);ap.add_argument('--execute-existing',action='store_true');args=ap.parse_args()
    target=(Path(__file__).resolve().parent.parent/'target').resolve()
    for name in ['host_dir','curve_dir','globals_dir','composition_dir','out']:getattr(args,name).resolve().relative_to(target)
    manifest=json.loads((args.host_dir/'manifest.json').read_text())
    assert digest(args.host_dir/'host-x86.exe')==manifest['host_exe_sha256']==args.reviewed_host_sha256
    assert digest(args.host_dir/'host-x86.asm')==manifest['host_source_sha256']
    assert manifest['source_sha256']==digest(args.source_exe)==SOURCE_SHA256
    assert manifest['protocol']['input_bytes']==INPUT_BYTES and manifest['protocol']['output_bytes']==OUTPUT_BYTES
    if args.execute_existing:execute(args,manifest)
    else:prepare(args,manifest)


if __name__=='__main__':main()
