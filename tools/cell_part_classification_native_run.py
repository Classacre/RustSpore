"""Prepare source-chain cases, then optionally run a separately reviewed host.

Default preparation executes no generated code. --execute-existing runs only
the hash-pinned existing cohort after the host manifest has been reviewed.
"""
import argparse
from collections import Counter
import csv
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import random
import struct
import subprocess

from cell_capability_cache_probe import TAGS
from dbpf_extract import payload
from gmdl_skin_probe import indexes


def digest(path):
    value=hashlib.sha256()
    with path.open('rb')as stream:
        while block:=stream.read(1048576):value.update(block)
    return value.hexdigest()


def request(nodes,low=0,scope=0,control=0x27f):
    assert len(nodes)<=8
    result=struct.pack('<3IH18x',len(nodes),low,scope,control)
    for ids,parent in nodes:
        assert len(ids)<=32
        result+=struct.pack('<2I32I',len(ids),0xffffffff if parent is None else parent,
            *ids,*([0]*(32-len(ids))))
    return result+bytes((8-len(nodes))*136)


def run_small(host,data):
    return subprocess.run([str(host)],input=data,capture_output=True,
        creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)


def prepare(args,manifest):
    args.out.mkdir(parents=True,exist_ok=False)
    report=json.loads((args.census_dir/'report.json').read_text())
    assert not report['failures']and report['source_part_identities']==654
    assert all(row['all_candidate_copies_agree']for row in report['parts'])
    rows=[];by_key={};cache={};source_dir=args.out/'sources';source_dir.mkdir()
    with (args.out/'properties.tsv').open('w',newline='\n')as table:
        table.write('record\tlow\tscope\tpackage\traw_file\tsha256\tparent_key\tfield_count\n')
        for key,records in report['properties'].items():
            by_key[key]=[]
            for record in records:
                ordinal=len(rows);low=int(record['low'],16);scope=int(record['scope'],16)
                relative=Path(record['package']);assert not relative.is_absolute()and'..'not in relative.parts
                package=args.install/relative
                if package not in cache:
                    # The graph alternates package copies for each key. Retain
                    # only PROP index rows so every package index is read once.
                    cache[package]=[entry for entry in indexes(package)if entry['type']==0xb1b104]
                matches=[entry for entry in cache[package]if entry['type']==0xb1b104 and entry['instance']==int(key,16)]
                candidates=[payload(package,entry)for entry in matches]
                candidates=[raw for raw in candidates if hashlib.sha256(raw).hexdigest()==record['sha256']]
                assert len(candidates)==1,(key,relative,len(candidates))
                name=f'sources/{ordinal:04}.prop';(args.out/name).write_bytes(candidates[0])
                table.write(f'{ordinal}\t{low:08X}\t{scope:08X}\t{record["package"]}\t{name}\t{record["sha256"]}\t{record["parent_key"]or"-"}\t{len(record["ids"])}\n')
                rows.append(record);by_key[key].append(ordinal)
    assert len(rows)==1411 and len(by_key)==719
    @lru_cache(None)
    def chains(key):
        result=[]
        for ordinal in by_key[key]:
            parent=rows[ordinal]['parent_key']
            if parent is None:result.append((ordinal,))
            else:result.extend((ordinal,*suffix)for suffix in chains(parent))
        return tuple(result)
    count=0;labels=Counter();policies=Counter();source_copies=set();rng=random.Random(0xe827a0)
    with (args.out/'input.bin').open('wb')as stream,(args.out/'source-chains.tsv').open('w',newline='\n')as table:
        table.write('case\tlow\tscope\tcw\tnodes\t'+'\t'.join(f'record{i}'for i in range(8))+'\n')
        def emit(data,label):
            nonlocal count
            assert len(data)==1120;stream.write(data);labels[label]+=1
            policies[f'{struct.unpack_from("<H",data,12)[0]:04X}']+=1;count+=1
        for key in sorted(by_key):
            for chain in chains(key):
                assert len(chain)<=8
                nodes=[([int(value,16)for value in rows[ordinal]['ids']],index+1 if index+1<len(chain)else None)
                    for index,ordinal in enumerate(chain)]
                first=rows[chain[0]];source_copies.update(chain)
                table.write(f'{count}\t{first["low"]}\t{first["scope"]}\t027F\t{len(nodes)}\t'+
                    '\t'.join(str(value)for value in (*chain,*([-1]*(8-len(chain)))))+'\n')
                emit(request(nodes,int(first['low'],16),int(first['scope'],16)),'source_all_candidate_paths')
        source_cases=count;assert source_cases==23306 and len(source_copies)==1411
        for precision in [0,0x200,0x300]:
            for rounding in [0,0x400,0x800,0xc00]:
                control=0x7f|precision|rounding
                emit(request([],control=control),'synthetic_missing_list')
                for mask in range(2048):
                    ids=sorted(tag for index,(tag,_kind)in enumerate(TAGS)if mask&(1<<index))
                    emit(request([(ids,None)],control=control),'synthetic_all_masks_local')
                    distributed=[[]for _ in range(8)]
                    for index,tag in enumerate(ids):distributed[index%8].append(tag)
                    emit(request([(sorted(ids),index+1 if index<7 else None)for index,ids in enumerate(distributed)],control=control),
                        'synthetic_all_masks_parent_chain')
                for size in range(33):
                    ids=sorted([tag for tag,_ in TAGS]+[0x7ffffff0+i for i in range(21)])[:size]
                    emit(request([(ids,None)],control=control),'synthetic_all_field_counts')
                for _ in range(256):
                    size=rng.randrange(1,9);nodes=[]
                    for index in range(size):
                        ids=[rng.choice([rng.getrandbits(32),*([tag for tag,_ in TAGS])])for _ in range(rng.randrange(33))]
                        if rng.randrange(2):ids.sort()
                        parent=rng.choice([None,*range(index+1,size)])
                        nodes.append((ids,parent))
                    emit(request(nodes,rng.getrandbits(32),rng.getrandbits(32),control),'synthetic_unsorted_duplicate_random')
    result=dict(host_executed=False,host_exe_sha256=manifest['host_exe_sha256'],cases=count,
        source_chain_cases=source_cases,source_property_copies=1411,source_property_keys=719,source_part_identities=654,
        source_maximum_fields=31,source_maximum_chain_lists=7,source_context='027F',
        labels=dict(labels),policy_counts=dict(policies),input_bytes=(args.out/'input.bin').stat().st_size,
        hashes={name:digest(args.out/name)for name in ['input.bin','properties.tsv','source-chains.tsv']},
        scope='Allcandidateexplicit(low,scope)parentpaths; no packagewinnerchosen. Native fixturelistvaluesomittedbecauseordinarypredicateonlyreadsIDs.')
    (args.out/'prepare-report.json').write_text(json.dumps(result,indent=2))
    print(json.dumps(result,indent=2))


def execute(args,manifest,host):
    prepared=json.loads((args.out/'prepare-report.json').read_text())
    assert prepared['host_exe_sha256']==manifest['host_exe_sha256']
    assert all(digest(args.out/name)==value for name,value in prepared['hashes'].items())
    assert not(args.out/'output.bin').exists()
    smoke=request([([0x11b79a74],1),([0x100f0f5a],None)],low=0x12345678,scope=0xabcdef01)
    result=run_small(host,smoke)
    (args.out/'smoke-input.bin').write_bytes(smoke);(args.out/'smoke-output.bin').write_bytes(result.stdout)
    smoke_report=dict(exit=result.returncode,bytes=len(result.stdout),stderr=result.stderr.decode(errors='replace'))
    (args.out/'smoke-report.json').write_text(json.dumps(smoke_report,indent=2))
    assert result.returncode==0 and not result.stderr and struct.unpack('<4I',result.stdout)==(12,1,1,2),smoke_report
    malformed=[('empty',b'',0,0),('partial1',smoke[:1],7,0),('partial1119',smoke[:1119],7,0),
        ('complete_then_partial',smoke+b'x',7,16)]
    for name,offset,value,fmt in [('nodes9',0,9,'I'),('fieldcount33',32,33,'I'),
        ('selfparent',36,0,'I'),('out_of_bounds_parent',36,2,'I'),
        ('reserved_precision',12,0x17f,'H'),('unmasked_control',12,0x270,'H')]:
        raw=bytearray(smoke);struct.pack_into('<'+fmt,raw,offset,value);malformed.append((name,bytes(raw),7,0))
    checks=[]
    for name,data,status,size in malformed:
        result=run_small(host,data);item=dict(case=name,exit=result.returncode,reply_bytes=len(result.stdout),expected_exit=status,expected_reply_bytes=size)
        checks.append(item);assert result.returncode==status and not result.stderr and len(result.stdout)==size,item
    (args.out/'protocol-checks.json').write_text(json.dumps(checks,indent=2))
    with (args.out/'input.bin').open('rb')as inp,(args.out/'output.bin').open('wb')as out,(args.out/'stderr.bin').open('wb')as err:
        result=subprocess.run([str(host)],stdin=inp,stdout=out,stderr=err,
            creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)
    run=dict(exit=result.returncode,output_bytes=(args.out/'output.bin').stat().st_size,
        stderr_bytes=(args.out/'stderr.bin').stat().st_size)
    (args.out/'run.json').write_text(json.dumps(run,indent=2))
    assert result.returncode==0 and run['output_bytes']==prepared['cases']*16 and run['stderr_bytes']==0,run
    classes=Counter()
    with (args.out/'output.bin').open('rb')as stream:
        for _ in range(prepared['cases']):
            kind,manager,queries,visits=struct.unpack('<4I',stream.read(16))
            assert 1<=kind<=12 and manager==1 and 0<=queries<=11 and queries<=visits<=88
            classes[str(kind)]+=1
    report=dict(prepared,host_executed=True,native_exit=0,protocol_checks=len(checks),
        checked_manager_tagorder_parent_count_stack_FP_and_buffer_guards=True,
        observed_class_counts=dict(classes),output_sha256=digest(args.out/'output.bin'),
        source_exe_sha256=manifest['source_sha256'])
    (args.out/'report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))


def main():
    ap=argparse.ArgumentParser();ap.add_argument('install',type=Path)
    ap.add_argument('--census-dir',type=Path,required=True);ap.add_argument('--host-dir',type=Path,required=True)
    ap.add_argument('--reviewed-host-sha256',required=True);ap.add_argument('--out',type=Path,required=True)
    ap.add_argument('--execute-existing',action='store_true');args=ap.parse_args()
    target=(Path(__file__).resolve().parent.parent/'target').resolve()
    for path in [args.census_dir,args.host_dir,args.out]:path.resolve().relative_to(target)
    manifest=json.loads((args.host_dir/'manifest.json').read_text());host=(args.host_dir/'host-x86.exe').resolve()
    assert digest(host)==manifest['host_exe_sha256']==args.reviewed_host_sha256
    assert manifest['protocol']['input_bytes']==1120 and manifest['protocol']['output_bytes']==16
    if args.execute_existing:execute(args,manifest,host)
    else:prepare(args,manifest)


if __name__=='__main__':main()
