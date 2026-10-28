"""Prepare small source-derived multi-component joins from accepted native curves.

No host execution. Original full ANIM bytes and extracted descriptors stay in
ignored target/. Root separately admits contribution and data2 reader stages.
"""
import argparse
from collections import defaultdict
import csv
import hashlib
import json
from pathlib import Path
import struct

import cell_runtime_curves_fixture as source
import cell_runtime_contributions_inputs as contribution

ROOT = Path(__file__).resolve().parent.parent


def word(raw,at): return struct.unpack_from('<I',raw,at)[0]


def prepare(args):
    args.out.resolve().relative_to((ROOT/'target').resolve())
    source_report=json.loads((args.census/'report.json').read_text())
    native_report=json.loads((args.native/'report.json').read_text())
    fixture_report=json.loads((args.fixture/'report.json').read_text())
    assert native_report['cases']==fixture_report['cases']==84544
    comparison=json.loads((args.native/'rust-comparison-v1.json').read_text())
    assert comparison['exact'] and comparison['native_pins']['unchanged']
    assert source.sha_file(args.native/'corpus-attempt01-output.bin')==comparison['raw_native_sha256']
    assert source.sha_file(args.fixture/'cases.tsv')==fixture_report['hashes']['cases.tsv']
    # Pin complete producer artifacts, then select a bounded explicit join.
    pins={str(p):source.sha_file(p) for p in [args.census/'report.json',args.census/'components.jsonl',
        args.fixture/'report.json',args.fixture/'cases.tsv',args.native/'report.json',
        args.native/'corpus-attempt01-output.bin',args.native/'rust-comparison-v1.json']}
    assert source.sha_file(args.census/'components.jsonl')==source_report['components_sha256']
    groups=defaultdict(list)
    for line in (args.census/'components.jsonl').read_text().splitlines():
        item=json.loads(line)
        groups[tuple(item[k]for k in ['package','low','type','scope','channel'])].append(item)
    # Two- and three-component groups at root/channel0 and distinct wrapper1.
    selected=[]
    for count,channel in [(2,'0'),(3,'0'),(2,'1'),(3,'1')]:
        selected.append(next((key,items)for key,items in groups.items()if len(items)==count and key[-1]==channel))
    rare=next(((key,items)for key,items in groups.items()if any(x['tag']=='98C942B3'for x in items)),None)
    if rare is not None and rare[0]not in {key for key,_ in selected}:selected.append(rare)
    lookup={}
    with (args.fixture/'cases.tsv').open(newline='')as table:
        for row in csv.DictReader(table,delimiter='\t'):
            if row['package']=='-':continue
            key=tuple(row[k]for k in ['package','low','type','scope','channel'])
            if key in {k for k,_ in selected}:
                lookup[(key,row['component'],row['frame_bits'])]=row
    args.out.mkdir(parents=True,exist_ok=False)
    (args.out/'clips').mkdir()
    (args.out/'prepared').mkdir()
    cases=[]
    packages={}
    with (args.native/'corpus-attempt01-output.bin').open('rb')as replies,(args.out/'contribution-input.bin').open('xb')as requests:
        for group_index,(key,items)in enumerate(selected):
            package,low,typ,scope,channel_text=key
            channel=int(channel_text)
            if package not in packages:packages[package]=source.package_index(args.install/package)
            index,index_meta=packages[package]
            payload=source.source_payload(args.install/package,index[tuple(int(x,16)for x in [low,typ,scope])])
            payload_sha=hashlib.sha256(payload).hexdigest()
            assert all(item['source_sha256']==payload_sha for item in items)
            clip_name=f'clips/{payload_sha}.anim'
            clip_path=args.out/clip_name
            if not clip_path.exists():clip_path.write_bytes(payload)
            start=word(payload,word(payload,328)+4*channel)
            count,at=word(payload,start+220),word(payload,start+224)
            descriptors=[list(struct.unpack_from('<8I',payload,at+32*i))for i in range(count)]
            assert {i for i,d in enumerate(descriptors)if d[0]&15==3}=={int(item['component'])for item in items}
            assert word(payload,start+172)&1 and channel<64
            for item in items:
                d=descriptors[int(item['component'])]
                assert d[1]==int(item['tag'],16)
                d[0],d[4]=int(item['prepared_flags'],16),int(item['prepared_slot'],16)
            prepared=b''.join(struct.pack('<8I',*d)for d in descriptors)
            prepared_name=f'prepared/{group_index:02}.bin'
            (args.out/prepared_name).write_bytes(prepared)
            samples=items[0]['samples']
            # Five frame families, choosing first measured midpoint only.
            chosen=[]
            for kind in ['before_first','exact_first','segment_midpoint','exact_last','after_last']:
                match=next((sample for sample in samples if sample['kind']==kind),None)
                if match is not None:chosen.append(match)
            for sample in chosen:
                frame=sample['frame_bits']
                slots=[(0xA5A5A5A5,0xA5A5A5A5)]*6
                refs=[]
                for item in sorted(items,key=lambda x:int(x['component'])):
                    ref=lookup[(key,item['component'],frame)]
                    assert ref['cw']=='027F' and ref['expected_policy']=='-'
                    native_case=int(ref['case'])
                    replies.seek(native_case*476)
                    reply=replies.read(476)
                    slot=int(item['prepared_slot'],16)
                    value,weight=struct.unpack_from('<2I',reply,200+(slot-3)*40)
                    slots[slot-3]=(value,weight)
                    refs.append(dict(case=native_case,component=int(item['component']),slot=slot,
                        value_bits=f'{value:08X}',weight_bits=f'{weight:08X}'))
                mask=sum(1<<int(item['component'])for item in items)
                # Literal root-only binding with source wrapper index; selection
                # and claim policy remain explicitly supplied protocol context.
                binding=[1,0,mask,0x00FF0000|(channel<<8)]
                initial=bytearray(b'\xA5'*164)
                initial[:4]=bytes(4)
                request=contribution.request([struct.pack('<8I',*d)for d in descriptors],slots,
                    mask=mask,cw=0x027F,initial=initial)
                requests.write(request)
                cases.append(dict(case=len(cases),clip=clip_name,prepared=prepared_name,channel=channel,
                    source=key,source_sha256=payload_sha,frame_bits=frame,frame_kind=sample['kind'],cw='027F',
                    binding=binding,mask=f'{mask:08X}',native_curve_references=refs,
                    prior_aggregate='A5 bytes with count reset to zero',prior_slots='A5 value/weight bits',
                    actor_route='ExplicitResolvedRoot(state0,handle1,wordFC1), plus separate typed unresolved probe'))
    (args.out/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
    with (args.out/'cases.tsv').open('x',newline='')as table:
        writer=csv.writer(table,delimiter='\t',lineterminator='\n')
        writer.writerow(['case','clip','prepared','channel','frame_bits','cw','binding0','binding1','binding2','binding3'])
        for case in cases:writer.writerow([case['case'],case['clip'],case['prepared'],case['channel'],case['frame_bits'],case['cw'],*[f'{x:08X}'for x in case['binding']]])
    report=dict(schema='source-root-data-multi-component-join-v1',cases=len(cases),groups=len(selected),
        component_counts=[len(items)for _,items in selected],source_pins=pins,
        package_indexes={k:v[1]for k,v in packages.items()},
        tool_sha256=source.sha_file(Path(__file__)),contribution_input_sha256=source.sha_file(args.out/'contribution-input.bin'),
        files={str(p.relative_to(args.out)):source.sha_file(p)for p in args.out.rglob('*')if p.is_file()},
        native_executed=False,rust_executed=False,
        scope='Full original source ANIM channels with all their kind3 descriptors sampled in order; accepted original curve replies joined into a multi-component original contribution request. Explicit root-only binding/selection masks and actor context, not live routing. Root must execute admitted contribution host and reader, then compare cohesive Rust adapter. Clock conversion remains a separate finite non-loop slice awaiting native comparison.')
    (args.out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k]for k in ['schema','cases','groups','component_counts','contribution_input_sha256']},indent=2))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--install',type=Path,default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--census',type=Path,default=ROOT/'target/source-cell/runtime-curves-source-census-v1')
    parser.add_argument('--fixture',type=Path,default=ROOT/'target/source-cell/runtime-curves-fixture-v1')
    parser.add_argument('--native',type=Path,default=ROOT/'target/source-cell/runtime-curves-native-v1')
    parser.add_argument('--contribution-output',type=Path)
    args=parser.parse_args()
    if args.contribution_output is not None:
        from cell_runtime_modifiers_native_run import request
        report=json.loads((args.out/'report.json').read_text())
        for name,expected in report['files'].items():assert source.sha_file(args.out/name)==expected
        assert args.contribution_output.stat().st_size==report['cases']*184
        with args.contribution_output.open('rb')as native,(args.out/'consumer-input.bin').open('xb')as output:
            for case in range(report['cases']):
                reply=native.read(184)
                assert word(reply,172)==1 and struct.unpack_from('<H',reply,176)[0]==0x027F
                output.write(request(reply[:164]))
        result=dict(schema='source-root-data-consumer-input-v1',cases=report['cases'],
            contribution_output_sha256=source.sha_file(args.contribution_output),
            consumer_input_sha256=source.sha_file(args.out/'consumer-input.bin'),native_executed=False)
        with (args.out/'consumer-input-report.json').open('x')as output:json.dump(result,output,indent=2)
        print(json.dumps(result,indent=2))
    else:
        prepare(args)


if __name__=='__main__':main()
