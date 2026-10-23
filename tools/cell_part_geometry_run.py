"""Run a reviewed E83E60 fixture host with real rows and explicit classes.

The full source creature inventory is reloaded and hashed. Supplying class1,
class2 and the literal radius exception exercises geometry, not classification.
Every host binary, proprietary payload and request/reply remains under target.
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

from cell_capability_cache_probe import source_rows
from dbpf_extract import payload
from gmdl_skin_probe import indexes


def digest(path):
    value=hashlib.sha256()
    with path.open('rb')as stream:
        while block:=stream.read(1024*1024):value.update(block)
    return value.hexdigest()


def request(row,count=1,kind=1,low=0,typ=0x0f43029a,high=0,control=0x27f,
            origin=(0,0,0),matrix=(0x3f800000,0,0,0,0x3f800000,0,0,0,0x3f800000)):
    assert len(row)==140
    return struct.pack('<5IH10x',count,kind,low,typ,high,control)+row+struct.pack('<12I',*origin,*matrix)+bytes(4)


def run_small(host,data):
    return subprocess.run([str(host)],input=data,capture_output=True,
        creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)


def sample_row():
    row=bytearray(140)
    struct.pack_into('<H',row,6,0xffff)
    struct.pack_into('<7f',row,20,-1.,-2.,-3.,1.,2.,3.,1.)
    struct.pack_into('<9f',row,48,1.,0.,0.,0.,1.,0.,0.,0.,1.)
    struct.pack_into('<3f',row,84,4.,-5.,123.)
    struct.pack_into('<3I',row,96,0x7fc01234,0x80000000,0xff800000)
    return bytes(row)


def main():
    ap=argparse.ArgumentParser();ap.add_argument('install',type=Path)
    ap.add_argument('--census-dir',type=Path,required=True)
    ap.add_argument('--host-dir',type=Path,required=True)
    ap.add_argument('--reviewed-host-sha256',required=True)
    ap.add_argument('--out',type=Path,required=True);args=ap.parse_args()
    target=(Path(__file__).resolve().parent.parent/'target').resolve()
    for path in [args.census_dir,args.host_dir,args.out]:path.resolve().relative_to(target)
    manifest=json.loads((args.host_dir/'manifest.json').read_text())
    host=(args.host_dir/'host-x86.exe').resolve()
    assert digest(host)==manifest['host_exe_sha256']==args.reviewed_host_sha256
    assert manifest['protocol']['input_bytes']==224 and manifest['protocol']['output_bytes']==160
    assert manifest['replacement_provider']['outbound_relative_transfers_in_fixed_stubs']==0
    args.out.mkdir(parents=True,exist_ok=False)
    smoke=request(sample_row());result=run_small(host,smoke)
    (args.out/'smoke-input.bin').write_bytes(smoke);(args.out/'smoke-output.bin').write_bytes(result.stdout)
    smoke_report=dict(exit=result.returncode,reply_bytes=len(result.stdout),stderr=result.stderr.decode(errors='replace'))
    (args.out/'smoke-report.json').write_text(json.dumps(smoke_report,indent=2))
    assert result.returncode==0 and not result.stderr and len(result.stdout)==160,smoke_report
    assert struct.unpack_from('<4I',result.stdout)==(1,1,1,1)
    expected=bytearray([0xa5]*72);expected[0]=1
    struct.pack_into('<I3f',expected,4,1,10.,-12.5,0.)
    expected[20:56]=sample_row()[48:84];struct.pack_into('<f',expected,56,7.5)
    expected[60:72]=sample_row()[96:108]
    assert result.stdout[16:88]==result.stdout[88:160]==expected
    malformed=[('empty',b'',0,0),('partial1',smoke[:1],7,0),
        ('partial223',smoke[:223],7,0),('complete_then_partial',smoke+b'x',7,160)]
    for name,offset,value,fmt in [('count257',0,257,'I'),('class0',4,0,'I'),
        ('class13',4,13,'I'),('reserved_precision',20,0x17f,'H'),('unmasked_control',20,0x270,'H')]:
        invalid=bytearray(smoke);struct.pack_into('<'+fmt,invalid,offset,value)
        malformed.append((name,bytes(invalid),7,0))
    checks=[]
    for name,data,status,size in malformed:
        result=run_small(host,data)
        checks.append(dict(case=name,exit=result.returncode,reply_bytes=len(result.stdout),expected_exit=status,expected_reply_bytes=size))
        assert result.returncode==status and len(result.stdout)==size and not result.stderr,checks[-1]
    (args.out/'protocol-checks.json').write_text(json.dumps(checks,indent=2))

    census=json.loads((args.census_dir/'inventory.json').read_text())
    assert not census['failures']and census['statistics']['creature_resources']==1365
    count=0;source_cases=0;source_resources=[];policies=Counter();labels=Counter()
    cache={};source_dir=args.out/'sources';source_dir.mkdir()
    with (args.out/'input.bin').open('wb')as stream,(args.out/'source-map.tsv').open('w',newline='\n')as mapping:
        mapping.write('package\ttype\tgroup\tinstance\trow\tclass\tcreation_low\tcw\tnative_case_ordinal\n')
        def emit(data,label):
            nonlocal count
            assert len(data)==224
            stream.write(data);policies[f'{struct.unpack_from("<H",data,20)[0]:04X}']+=1
            labels[label]+=1;count+=1
        for ordinal,record in enumerate(census['creatures']):
            relative=Path(record['package']);assert not relative.is_absolute()and'..'not in relative.parts
            package=args.install/relative
            if package not in cache:
                cache.clear();cache[package]={(entry['type'],entry['group'],entry['instance']):entry for entry in indexes(package)}
            typ,group,instance=(int(part,16)for part in record['key'].split(':'))
            raw=payload(package,cache[package][typ,group,instance])
            assert hashlib.sha256(raw).hexdigest()==record['sha256']
            rows=source_rows(raw);assert len(rows)==record['source_rows']
            raw_name=f'{ordinal:04}.creature';(source_dir/raw_name).write_bytes(raw)
            source_resources.append(dict(record,raw_file='sources/'+raw_name))
            for row_index,row in enumerate(rows[:64]):
                for kind,low,label in [(1,instance&0xffffffff,'source_class1'),
                    (2,instance&0xffffffff,'source_class2'),(2,0xd9007204,'source_class2_exception')]:
                    mapping.write(f'{record["package"]}\t{typ:08X}\t{group:08X}\t{instance:016X}\t{row_index}\t{kind}\t{low:08X}\t027F\t{count}\n')
                    emit(request(row,kind=kind,low=low,typ=typ,high=instance>>32),label);source_cases+=1
            if (ordinal+1)%250==0:print(f'GEOMETRY_SOURCE resources={ordinal+1} cases={count}',flush=True)
        assert source_cases==45664*3
        row=sample_row();rng=random.Random(0xe83e60)
        for precision in [0,0x200,0x300]:
            for rounding in [0,0x400,0x800,0xc00]:
                control=0x7f|precision|rounding
                for kind in range(1,13):
                    for size in [0,1,2,63,64,65,128,256]:
                        for flag in [0,1,0xfffe,0xffff]:
                            changed=bytearray(row);struct.pack_into('<H',changed,6,flag)
                            emit(request(changed,count=size,kind=kind,control=control),'synthetic_class_count_flag')
                for low in [0,0xd9007204,0xffffffff]:
                    for size in [0,1,64,65,256]:
                        emit(request(row,count=size,kind=2,low=low,control=control),'synthetic_radius_exception')
                # Every copied/unused word may carry arbitrary bits. Arithmetic
                # remains finite and must not reject an opaque NaN payload.
                for offset in [56,68,80,92,96,100,104]:
                    for value in [0x80000000,0x7fc01234,0xff800000,1]:
                        changed=bytearray(row);struct.pack_into('<I',changed,offset,value)
                        emit(request(changed,control=control),'synthetic_opaque_word')
                for value in [0,0x80000000,1,0x80000001,0x7fffff,0x807fffff]:
                    changed=bytearray(row)
                    for offset in range(20,44,4):struct.pack_into('<I',changed,offset,value)
                    emit(request(changed,control=control),'synthetic_signed_zero_subnormal')
                for iteration in range(256):
                    changed=bytearray(rng.randbytes(140))
                    struct.pack_into('<H',changed,6,rng.choice([0,1,0xffff,0xfffe]))
                    for offset in range(20,44,4):struct.pack_into('<f',changed,offset,rng.uniform(-2.e5,2.e5))
                    struct.pack_into('<f',changed,44,rng.uniform(-100.,100.))
                    for offset in [48,52,60,64,72,76]:struct.pack_into('<f',changed,offset,rng.uniform(-10.,10.))
                    for offset in [84,88]:struct.pack_into('<f',changed,offset,rng.uniform(-1.e4,1.e4))
                    emit(request(changed,count=rng.choice([1,2,64,65,256]),kind=rng.randrange(1,13),
                        low=rng.choice([0,0xd9007204]),control=control),'synthetic_random_finite')
                for size in [0,1,65,256]:
                    for origin in [(0,0,0),(0x80000000,0x7fc01234,0xff800000)]:
                        emit(request(bytes([0xff])*140,count=size,kind=0,typ=0x476a98c7,
                            control=control,origin=origin,matrix=(0x7fc09876,)*9),'synthetic_special_copy')
    (args.out/'sources.json').write_text(json.dumps(source_resources,indent=2))
    print(f'GEOMETRY_NATIVE cases={count} source_cases={source_cases} input_bytes={count*224}',flush=True)
    with (args.out/'input.bin').open('rb')as inp,(args.out/'output.bin').open('wb')as out,(args.out/'stderr.bin').open('wb')as err:
        result=subprocess.run([str(host)],stdin=inp,stdout=out,stderr=err,
            creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)
    run=dict(exit=result.returncode,input_bytes=(args.out/'input.bin').stat().st_size,
        output_bytes=(args.out/'output.bin').stat().st_size,stderr_bytes=(args.out/'stderr.bin').stat().st_size)
    (args.out/'run.json').write_text(json.dumps(run,indent=2))
    assert result.returncode==0 and run['output_bytes']==count*160 and run['stderr_bytes']==0,run
    validation=Counter()
    with (args.out/'input.bin').open('rb')as inp,(args.out/'output.bin').open('rb')as out:
        for ordinal in range(count):
            record=inp.read(224);reply=out.read(160)
            size,kind,low,typ,high=struct.unpack_from('<5I',record)
            special=typ==0x476a98c7;expected_count=1 if special else min(size,64)
            assert struct.unpack_from('<4I',reply)==(expected_count,int(not special),0 if special else expected_count,int(not special)),ordinal
            if expected_count:
                assert reply[16:88]==reply[88:160],ordinal
                assert reply[17:20]==b'\xa5'*3
                center=struct.unpack_from('<3f',reply,24);radius=struct.unpack_from('<f',reply,72)[0]
                if not special:assert all(math.isfinite(v)for v in (*center,radius)),ordinal
                validation['special_nonfinite_copied_center_cases']+=special and any(not math.isfinite(v)for v in center)
                validation['opaque_nonfinite_output_word_cases']+=any(not math.isfinite(struct.unpack_from('<f',reply,16+offset)[0])for offset in [*range(20,56,4),60,64,68])
            else:assert reply[16:]==b'\xa5'*144,ordinal
    hashes={name:digest(args.out/name)for name in ['input.bin','output.bin','source-map.tsv','sources.json']}
    report=dict(host_exe_sha256=manifest['host_exe_sha256'],source_exe_sha256=manifest['source_sha256'],
        cases=count,source_cases=source_cases,source_resources=len(source_resources),source_row_copies=45664,
        explicit_classes=[1,2],source_context='027F',policy_counts=dict(policies),labels=dict(labels),
        protocol_checks=len(checks),native_exit=result.returncode,guards_provider_counts_padding_and_unwritten_suffix_checked=True,
        nonfinite_consumed_arithmetic_inputs=0,nonfinite_arithmetic_outputs=0,finite_case_exclusions=0,
        validation=dict(validation),hashes=hashes,mxcsr_control='1F80',
        scope='All first64source Creature rowcopies with suppliedclass1/class2/exception; original geometry and clamp only. Classification, inheritance, packageoverlay, livecache and actorintegration remain separate.')
    (args.out/'report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))


if __name__=='__main__':main()
