"""Prepare a fresh clock -> full curves -> contributions -> data2 join.

Every later stage consumes the preceding actual native replies. No inverse
time mapping, old midpoint substitution, native execution or compilation.
"""
import argparse
import csv
import json
from pathlib import Path
import struct
import cell_runtime_clock_inputs as clock
import cell_runtime_curves_inputs as curves
import cell_runtime_contributions_inputs as contributions
from cell_runtime_modifiers_native_run import request as consumer_request
from cell_runtime_curves_fixture import sha_file,number

ROOT=Path(__file__).resolve().parent.parent
def word(raw,at):return struct.unpack_from('<I',raw,at)[0]
def save(path,value):
    with path.open('x')as output:json.dump(value,output,indent=2);output.write('\n')
def verify_files(root,report):
    for name,expected in report['files'].items():assert sha_file(root/name)==expected,name
def verify_initial(args):
    report=json.loads((args.out/'clock-preparation.json').read_text());verify_files(args.out,report)
    for path,expected in report['source_pins'].items():assert sha_file(Path(path))==expected,path
    return report

def prepare_clock(args):
    assert not args.out.exists()
    base=json.loads((args.base/'report.json').read_text());verify_files(args.base,base)
    accepted=json.loads((args.clock_acceptance).read_text());assert accepted['exact']and accepted['cases']==1560
    assert accepted['projection_sha256']=='5bed86653abd77395f3e5c22d64c0bacb940d3c0ba3d1b21ecd7708dfa51d91d'
    source_cases=json.loads((args.base/'cases.json').read_text());groups={}
    for row in source_cases:groups.setdefault((row['clip'],row['prepared'],row['channel']),row)
    assert len(groups)==5
    args.out.mkdir(parents=True);(args.out/'clips').mkdir();(args.out/'prepared').mkdir()
    cases=[]
    with (args.out/'clock-input.bin').open('xb')as binary:
        for group,row in groups.items():
            clip,prepared,channel=group
            payload=(args.base/clip).read_bytes();descriptors=(args.base/prepared).read_bytes()
            frames=word(payload,288);assert 0x3F800000<=frames<0x7F800000
            for name,raw in [(clip,payload),(prepared,descriptors)]:
                destination=args.out/name
                if destination.exists():assert destination.read_bytes()==raw
                else:destination.write_bytes(raw)
            # Exact binary fractions; duration1 is explicit protocol context.
            for mode in [0,1]:
                for quarter,(time,prior,delta)in enumerate([(0.,0.,0.),(.25,.125,.125),(.5,.25,.25),(.75,.5,.25)]):
                    request=clock.request(mode=mode,cw=0x027F,time=clock.bits(time if mode==0 else delta),
                        previous=clock.double(prior),duration=clock.double(1.),frames=frames)
                    binary.write(request)
                    cases.append(dict(case=len(cases),clip=clip,prepared=prepared,channel=channel,
                        source_sha256=row['source_sha256'],source_identity=row['source'],binding=row['binding'],
                        clock_mode=mode,quarter=quarter,explicit_duration_bits='3FF0000000000000',resource_frames_bits=f'{frames:08X}',cw='027F'))
    assert len(cases)==40
    save(args.out/'cases.json',cases)
    with (args.out/'cases.tsv').open('x',newline='')as table:
        writer=csv.writer(table,delimiter='\t',lineterminator='\n');writer.writerow(['case','clip','prepared','channel','cw','binding0','binding1','binding2','binding3'])
        for row in cases:writer.writerow([row['case'],row['clip'],row['prepared'],row['channel'],row['cw'],*[f'{x:08X}'for x in row['binding']]])
    report=dict(schema='clock-root-data-native-join-clock-preparation-v1',cases=40,groups=5,clock_modes=[0,1],
        source_pins={str(p):sha_file(p)for p in [args.base/'report.json',args.clock_acceptance,Path(__file__),
            ROOT/'tools/cell_runtime_clock_inputs.py',ROOT/'tools/cell_runtime_curves_inputs.py',ROOT/'tools/cell_runtime_curves_fixture.py',
            ROOT/'tools/cell_runtime_contributions_inputs.py',ROOT/'tools/cell_runtime_modifiers_native_run.py']},
        files={str(p.relative_to(args.out)):sha_file(p)for p in args.out.rglob('*')if p.is_file()},
        native_executed=False,scope='Five original full channels, actual literal descriptors and explicit root-only binding/actor context. Absolute and delta clock at0/quarter/half/threequarter of explicit duration1. Frames are not predicted here; the next stage requires actual reviewed native clock replies. No live duration setter, active clip or Cell registry association claim.')
    save(args.out/'clock-preparation.json',report);print(json.dumps(dict(stage='clock_requests',cases=40,sha256=sha_file(args.out/'clock-input.bin'))))

def prepare_curves(args):
    report=verify_initial(args);cases=json.loads((args.out/'cases.json').read_text())
    assert args.clock_output.stat().st_size==40*28
    assert not (args.out/'curve-input.bin').exists();records=[]
    with args.clock_output.open('rb')as native,(args.out/'curve-input.bin').open('xb')as requests:
        for case in cases:
            reply=native.read(28);cw,sw,mxcsr=struct.unpack_from('<HHI',reply,20)
            assert cw==0x027F and sw&0x3800==0 and mxcsr&~0x3F==0x1F80
            frame_bits=word(reply,16)
            payload=(args.out/case['clip']).read_bytes();prepared=(args.out/case['prepared']).read_bytes();channel=case['channel']
            channel_at=word(payload,word(payload,328)+4*channel)
            count,key_at,descriptor_count,descriptor_at=struct.unpack_from('<4I',payload,channel_at+212)
            assert count<=160 and len(prepared)==descriptor_count*32
            descriptors=[struct.unpack_from('<8I',prepared,32*i)for i in range(descriptor_count)]
            original=[struct.unpack_from('<8I',payload,descriptor_at+32*i)for i in range(descriptor_count)]
            info=next(d for d in original if d[0]&15==0 and d[1])
            for component,(descriptor,source_descriptor)in enumerate(zip(descriptors,original)):
                assert descriptor[1:4]==source_descriptor[1:4]and descriptor[5:]==source_descriptor[5:]
                if descriptor[0]&15!=3:continue
                rows=b''.join(payload[key_at+i*info[3]+info[2]:key_at+i*info[3]+info[2]+20]+
                    payload[key_at+i*source_descriptor[3]+source_descriptor[2]:key_at+i*source_descriptor[3]+source_descriptor[2]+40]for i in range(count))
                assert len(rows)==count*60
                request=curves.request(number(frame_bits),rows,number(word(payload,288)),bool(word(payload,280)&8),
                    0x027F,descriptor[0],descriptor[4])
                assert word(request,0)==frame_bits
                requests.write(request)
                records.append(dict(curve_case=len(records),clock_case=case['case'],component=component,
                    slot=descriptor[4],flags=f'{descriptor[0]:08X}',frame_bits=f'{frame_bits:08X}',full_rows=count,
                    normalized_row_sha256=__import__('hashlib').sha256(rows).hexdigest()))
    assert len(records)==96
    save(args.out/'curve-cases.json',records)
    save(args.out/'curve-preparation.json',dict(schema='clock-root-data-fresh-curve-preparation-v1',clock_cases=40,curve_cases=96,
        clock_output=str(args.clock_output.resolve()),clock_output_sha256=sha_file(args.clock_output),
        clock_preparation_sha256=sha_file(args.out/'clock-preparation.json'),
        files={n:sha_file(args.out/n)for n in ['curve-input.bin','curve-cases.json']},native_executed=False,
        scope='Fresh full-channel requests at literal actual native clock frame bits. No old sampled reply or midpoint is substituted.'))
    print(json.dumps(dict(stage='fresh_curve_requests',clock_cases=40,curve_cases=96,sha256=sha_file(args.out/'curve-input.bin'))))

def prepare_contributions(args):
    verify_initial(args);stage=json.loads((args.out/'curve-preparation.json').read_text());verify_files(args.out,stage)
    assert sha_file(Path(stage['clock_output']))==stage['clock_output_sha256']
    records=json.loads((args.out/'curve-cases.json').read_text());cases=json.loads((args.out/'cases.json').read_text())
    assert args.curve_output.stat().st_size==len(records)*476
    grouped={i:[]for i in range(40)}
    with args.curve_output.open('rb')as native:
        for record in records:
            reply=native.read(476);cw,sw,mxcsr=struct.unpack_from('<HHI',reply,468)
            assert cw==0x027F and sw&0x3800==0 and mxcsr&~0x3F==0x1F80
            value,weight=struct.unpack_from('<2I',reply,200+(record['slot']-3)*40)
            grouped[record['clock_case']].append((record['component'],record['slot'],value,weight))
    with (args.out/'contribution-input.bin').open('xb')as requests:
        for case in cases:
            raw=(args.out/case['prepared']).read_bytes();descriptors=[raw[i:i+32]for i in range(0,len(raw),32)]
            slots=[(0xA5A5A5A5,0xA5A5A5A5)]*6
            for component,slot,value,weight in sorted(grouped[case['case']]):slots[slot-3]=(value,weight)
            initial=bytearray(b'\xA5'*164);initial[:4]=bytes(4)
            requests.write(contributions.request(descriptors,slots,mask=case['binding'][2]&0xffffff,cw=0x027F,initial=initial))
    save(args.out/'contribution-preparation.json',dict(schema='clock-root-data-contribution-preparation-v1',cases=40,
        fresh_curve_output=str(args.curve_output.resolve()),fresh_curve_output_sha256=sha_file(args.curve_output),
        curve_preparation_sha256=sha_file(args.out/'curve-preparation.json'),
        files={'contribution-input.bin':sha_file(args.out/'contribution-input.bin')},native_executed=False))
    print(json.dumps(dict(stage='contribution_requests',cases=40,sha256=sha_file(args.out/'contribution-input.bin'))))

def prepare_consumer(args):
    verify_initial(args);stage=json.loads((args.out/'contribution-preparation.json').read_text());verify_files(args.out,stage)
    assert sha_file(Path(stage['fresh_curve_output']))==stage['fresh_curve_output_sha256']
    assert args.contribution_output.stat().st_size==40*184
    with args.contribution_output.open('rb')as native,(args.out/'consumer-input.bin').open('xb')as requests:
        for case in range(40):
            reply=native.read(184);cw,sw,mxcsr=struct.unpack_from('<HHI',reply,176)
            assert cw==0x027F and sw&0x3800==0 and mxcsr&~0x3F==0x1F80 and word(reply,172)==1
            requests.write(consumer_request(reply[:164]))
    save(args.out/'consumer-preparation.json',dict(schema='clock-root-data-consumer-preparation-v1',cases=40,
        contribution_output=str(args.contribution_output.resolve()),contribution_output_sha256=sha_file(args.contribution_output),
        contribution_preparation_sha256=sha_file(args.out/'contribution-preparation.json'),
        files={'consumer-input.bin':sha_file(args.out/'consumer-input.bin')},native_executed=False))
    print(json.dumps(dict(stage='consumer_requests',cases=40,sha256=sha_file(args.out/'consumer-input.bin'))))

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--base',type=Path,default=ROOT/'target/source-cell/runtime-root-data-join-v3')
    p.add_argument('--clock-acceptance',type=Path,default=ROOT/'target/source-cell/runtime-clock-native-run-v2/rust-comparison-v1.json')
    phases=p.add_mutually_exclusive_group();phases.add_argument('--clock-output',type=Path);phases.add_argument('--curve-output',type=Path);phases.add_argument('--contribution-output',type=Path)
    args=p.parse_args();args.out.resolve().relative_to((ROOT/'target').resolve())
    if args.clock_output:prepare_curves(args)
    elif args.curve_output:prepare_contributions(args)
    elif args.contribution_output:prepare_consumer(args)
    else:prepare_clock(args)
if __name__=='__main__':main()
