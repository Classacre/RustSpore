"""Prepare an actual-class source composition; execute only after review.

The original classifier corpus already proves all candidate Parent paths.
Geometry and skip-flag cache hosts remain the reviewed, unchanged binaries.
Archive/active-product binding, the empty-origin globals and FP policy remain
explicit; this is not live actor or full scalar-cache execution.
"""
import argparse
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import time

from cell_capability_cache_probe import source_rows
from cell_part_geometry_run import request as geometry_request
from cell_capability_geometry_run import request as cache_request
from cell_schema_probe import SOURCE_SHA256

GEOMETRY_SHA='e17531352ba5a6e6e6929617cdc345c7289dbfd6d42b0b34da32b41339e8499e'
CACHE_SHA='7eaa8d9d868327601a55f37da9d34d1791b1d12836b8d98bd700623a4b8e463c'
GEOMETRY_ASM='d708381148c7fb7d833fc84bfb1e96f15d5977853f887a875074a57f06d8abca'
CACHE_ASM='44dbd9fddf470a2fbf7a9752c284f72e19dcbdc349c397afcef1e62c79334232'
CLASS_SHA='c47eeda476dd9a90e3e0c368c4c6917a2479ddd2ebb5603ef8a67567d7283854'


def digest(path):
    h=hashlib.sha256()
    with path.open('rb')as stream:
        while block:=stream.read(1048576):h.update(block)
    return h.hexdigest()


def classification(root,census):
    assert digest(root/'output.bin')==digest(root/'rust-output-v1.bin')==CLASS_SHA
    rows=list(csv.DictReader((root/'source-chains.tsv').open(newline=''),delimiter='\t'))
    assert len(rows)==23306
    classes={};evidence={}
    with (root/'output.bin').open('rb')as stream:
        for index,row in enumerate(rows):
            kind,manager,queries,visits=struct.unpack('<4I',stream.read(16))
            assert int(row['case'])==index and manager==1 and 1<=kind<=12
            pair=(int(row['low'],16),int(row['scope'],16))
            classes.setdefault(pair,set()).add(kind);evidence.setdefault(pair,index)
    assert len(classes)==719 and all(len(values)==1 for values in classes.values())
    classes={pair:next(iter(values))for pair,values in classes.items()}
    parts=json.loads((census/'report.json').read_text())['parts'];assert len(parts)==654
    wanted=set()
    for part in parts:
        pair=(int(part['low'],16),int(part['scope'],16));wanted.add(pair)
        assert part['all_candidate_copies_agree']and part['possible_classes']==[classes[pair]]
    return classes,evidence,wanted


def prepare(args):
    args.out.mkdir(parents=True,exist_ok=False);(args.out/'sources').mkdir()
    classes,evidence,wanted=classification(args.classification_dir,args.classification_census)
    resources=json.loads((args.geometry_dir/'sources.json').read_text());assert len(resources)==1365
    assert digest(args.geometry_dir/'output.bin')=='a5533b75c432cb3b9938ef0f68c7ea49dd839d04d5793ddcaf9d145817fbe0fb'
    total=0;all_rows=0;used=set();class_counts=Counter();clamped=0;omitted=0
    with (args.out/'geometry-input.bin').open('wb')as inputs, (args.out/'geometry-expected.bin').open('wb')as expected, \
         (args.geometry_dir/'output.bin').open('rb')as prior, (args.out/'creatures.tsv').open('w',newline='\n')as table, \
         (args.out/'parts.tsv').open('w',newline='\n')as parts:
        table.write('case\tpackage\tlow\ttype\tscope\traw_file\tsha256\tsource_rows\tnative_rows\tgeometry_start\n')
        parts.write('geometry_case\tcreature_case\trow\tpart_low\tpart_scope\tclass\tclassification_case\n')
        for ordinal,resource in enumerate(resources):
            raw=(args.geometry_dir/resource['raw_file']).read_bytes();assert hashlib.sha256(raw).hexdigest()==resource['sha256']
            rows=source_rows(raw);assert len(rows)==resource['source_rows']
            all_rows+=len(rows)
            typ,_legacy,combined=(int(value,16)for value in resource['key'].split(':'))
            assert typ==0x0f43029a
            low,scope=combined&0xffffffff,combined>>32
            relative=f'sources/{ordinal:04}.creature';(args.out/relative).write_bytes(raw)
            table.write(f'{ordinal}\t{resource["package"]}\t{low:08X}\t{typ:08X}\t{scope:08X}\t{relative}\t{resource["sha256"]}\t{len(rows)}\t{min(len(rows),64)}\t{total}\n')
            clamped+=len(rows)>64;omitted+=max(0,len(rows)-64)
            for row_index,row in enumerate(rows[:64]):
                part_scope,part_low=struct.unpack_from('<2I',row,132);pair=(part_low,part_scope)
                kind=classes[pair];used.add(pair);class_counts[str(kind)]+=1
                inputs.write(geometry_request(row,kind=kind,low=low,typ=typ,high=scope))
                # Prior independent native geometry supplies the matching radius
                # branch; only the literal copied class word changes for others.
                prior_case=total*3+(1 if kind==2 else 0);prior.seek(prior_case*160)
                value=bytearray(prior.read(160));assert len(value)==160
                struct.pack_into('<I',value,20,kind);struct.pack_into('<I',value,92,kind)
                expected.write(value)
                parts.write(f'{total}\t{ordinal}\t{row_index}\t{part_low:08X}\t{part_scope:08X}\t{kind}\t{evidence[pair]}\n')
                total+=1
    assert total==45664 and all_rows==46418 and used==wanted and clamped==73 and omitted==754
    report=dict(schema='actual-class-source-cache-composition-v1',host_executed=False,
        source_creatures=1365,source_rows=46418,native_rows=45664,source_part_identities=654,
        clamped_creatures=clamped,omitted_rows=omitted,source_row_classes=dict(class_counts),
        classification_native_and_Rust_sha256=CLASS_SHA,explicit_CW='027F',
        explicit_empty_origin_words=[f'{x:08X}'for x in args.origin_words],
        origin_scope='Explicit environment input; not a measured live actor global.',
        geometry_host_sha256=GEOMETRY_SHA,cache_host_sha256=CACHE_SHA,
        hashes={name:digest(args.out/name)for name in ['geometry-input.bin','geometry-expected.bin','creatures.tsv','parts.tsv']},
        expected_geometry_cases=45664,expected_cache_cases=1365,
        scope='All installed source copies, consensus class across every candidate parent path; ordinary geometry and skip-flag cache only, no archive winner/active-product binding/scalar cache/actor.')
    (args.out/'prepare-report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


def execute_host(host,input_path,output_path,error_path,status_path,cases,width):
    assert not output_path.exists()
    started=time.perf_counter();failure=None;completed=None
    with input_path.open('rb')as inp,output_path.open('wb')as out,error_path.open('wb')as err:
        try:
            completed=subprocess.run([str(host.resolve())],stdin=inp,stdout=out,stderr=err,
                creationflags=subprocess.CREATE_NO_WINDOW,timeout=60)
        except Exception as exc:failure=exc
    result=dict(exit=None if completed is None else completed.returncode,
        input_bytes=input_path.stat().st_size,output_bytes=output_path.stat().st_size,
        stderr_bytes=error_path.stat().st_size,expected_cases=cases,expected_output_bytes=cases*width,
        elapsed_seconds=time.perf_counter()-started,error=None if failure is None else repr(failure))
    status_path.write_text(json.dumps(result,indent=2)+'\n')
    if failure is not None:raise failure
    assert result['exit']==0 and result['output_bytes']==cases*width and result['stderr_bytes']==0,result
    return result


def execution_pins(args):
    paths=[args.source_exe,
        *[root/name for root in [args.geometry_host,args.cache_host]for name in ['host-x86.exe','host-x86.asm','manifest.json']],
        *[args.out/name for name in ['prepare-report.json','geometry-input.bin','geometry-expected.bin','creatures.tsv','parts.tsv']],
        *[args.classification_dir/name for name in ['output.bin','rust-output-v1.bin','source-chains.tsv']],
        args.classification_census/'report.json',args.geometry_dir/'sources.json',args.geometry_dir/'output.bin']
    for row in csv.DictReader((args.out/'creatures.tsv').open(newline=''),delimiter='\t'):
        path=args.out/row['raw_file'];assert digest(path)==row['sha256'];paths.append(path)
    return {str(path.resolve()):digest(path)for path in paths}


def execute_body(args,report):
    assert all(digest(args.out/name)==value for name,value in report['hashes'].items())
    assert [f'{x:08X}'for x in args.origin_words]==report['explicit_empty_origin_words']
    origin=[struct.unpack('<f',struct.pack('<I',int(value,16)))[0]for value in report['explicit_empty_origin_words']]
    geometry=execute_host(args.geometry_host/'host-x86.exe',args.out/'geometry-input.bin',args.out/'geometry-output.bin',args.out/'geometry-stderr.bin',args.out/'geometry-status.json',45664,160)
    assert digest(args.out/'geometry-output.bin')==digest(args.out/'geometry-expected.bin')
    rows=list(csv.DictReader((args.out/'creatures.tsv').open(newline=''),delimiter='\t'))
    with (args.out/'geometry-output.bin').open('rb')as parts,(args.out/'cache-input.bin').open('wb')as out:
        for ordinal,row in enumerate(rows):
            assert int(row['case'])==ordinal and parts.tell()==int(row['geometry_start'])*160
            native=[]
            for _ in range(int(row['native_rows'])):
                record=parts.read(160);assert struct.unpack_from('<4I',record)==(1,1,1,1)
                assert record[16:88]==record[88:160];native.append(record[16:88])
            tail=bytes((index*37+ordinal*13)&255 for index in range(140))
            out.write(cache_request(native,empty=origin,tail=tail))
        assert parts.read(1)==b''
    cache_input_sha=digest(args.out/'cache-input.bin')
    (args.out/'cache-input-pin.json').write_text(json.dumps(dict(sha256=cache_input_sha),indent=2)+'\n')
    try:
        cache=execute_host(args.cache_host/'host-x86.exe',args.out/'cache-input.bin',args.out/'cache-output.bin',args.out/'cache-stderr.bin',args.out/'cache-status.json',1365,144)
    finally:
        after=digest(args.out/'cache-input.bin')
        (args.out/'cache-input-immutability.json').write_text(json.dumps(dict(before_sha256=cache_input_sha,after_sha256=after,unchanged=after==cache_input_sha),indent=2)+'\n')
        assert after==cache_input_sha
    assert digest(args.geometry_host/'host-x86.exe')==GEOMETRY_SHA
    assert digest(args.cache_host/'host-x86.exe')==CACHE_SHA
    report.update(host_executed=True,hidden_binary_pipes=True,retail_process=False,gui=False,
        geometry_run=geometry,cache_run=cache,geometry_exact_prior_branch_bytes=True,
        accepted_hashes={name:digest(args.out/name)for name in ['geometry-output.bin','cache-input.bin','cache-output.bin']})
    (args.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


def execute(args):
    report=json.loads((args.out/'prepare-report.json').read_text())
    pins=execution_pins(args)
    (args.out/'execution-pins-v1.json').write_text(json.dumps(pins,indent=2)+'\n')
    try:
        execute_body(args,report)
    finally:
        changed=[]
        for name,expected in pins.items():
            path=Path(name)
            if not path.exists()or digest(path)!=expected:changed.append(name)
        result=dict(all_original_source_hosts_manifests_inputs_and_source_copies_unchanged=not changed,
            pinned_files=len(pins),changed_files=changed)
        (args.out/'immutability-v1.json').write_text(json.dumps(result,indent=2)+'\n')
        assert not changed,result


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    for name in ['classification-dir','classification-census','geometry-dir','geometry-host','cache-host','out']:
        ap.add_argument('--'+name,type=Path,required=True)
    ap.add_argument('--source-exe',type=Path,required=True)
    ap.add_argument('--origin-words',required=True,help='Three explicit comma-separated hexadecimal float words.')
    ap.add_argument('--execute-existing',action='store_true');args=ap.parse_args()
    args.origin_words=[int(value,16)for value in args.origin_words.split(',')];assert len(args.origin_words)==3
    target=(Path(__file__).resolve().parent.parent/'target').resolve()
    for name in ['classification_dir','classification_census','geometry_dir','geometry_host','cache_host','out']:
        getattr(args,name).resolve().relative_to(target)
    assert digest(args.geometry_host/'host-x86.exe')==GEOMETRY_SHA
    assert digest(args.cache_host/'host-x86.exe')==CACHE_SHA
    assert digest(args.geometry_host/'host-x86.asm')==GEOMETRY_ASM
    assert digest(args.cache_host/'host-x86.asm')==CACHE_ASM
    assert digest(args.source_exe)==SOURCE_SHA256
    for root,exe_sha,asm_sha in [(args.geometry_host,GEOMETRY_SHA,GEOMETRY_ASM),(args.cache_host,CACHE_SHA,CACHE_ASM)]:
        manifest=json.loads((root/'manifest.json').read_text())
        assert manifest['source_sha256']==SOURCE_SHA256
        assert manifest['host_exe_sha256']==exe_sha and manifest['host_source_sha256']==asm_sha
    if args.execute_existing:execute(args)
    else:prepare(args)


if __name__=='__main__':main()
