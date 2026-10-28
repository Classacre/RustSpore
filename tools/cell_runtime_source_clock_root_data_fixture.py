"""Immutable source metadata5 -> clock32 -> curves72 -> contributions32 -> consumer32.
No original execution; each dependent request uses actual preceding native bytes.
"""
import argparse,csv,json,math,struct
from fractions import Fraction
from pathlib import Path
import cell_runtime_clock_inputs as clock
import cell_runtime_curves_inputs as curves
import cell_runtime_contributions_inputs as contributions
from cell_runtime_modifiers_native_run import request as consumer_request
from cell_runtime_curves_fixture import sha_file,number
from cell_runtime_clip_metadata_inputs import request as metadata_request,bounded
ROOT=Path(__file__).resolve().parent.parent
MAPPING=ROOT/'target/source-cell/runtime-clip-association-trace-v1/source-metadata-mapping-v1/report.json'
def word(raw,at):return struct.unpack_from('<I',raw,at)[0]
def save(path,value):
 with path.open('x')as f:json.dump(value,f,indent=2);f.write('\n')
def verify_files(root,report):
 for name,expected in report['files'].items():assert sha_file(root/name)==expected,name
def verify_initial(args):
 report=json.loads((args.out/'clock-preparation.json').read_text());verify_files(args.out,report)
 for path,expected in report['source_pins'].items():assert sha_file(Path(path))==expected,path
 return report
def prepare_metadata(args):
 assert not args.out.exists();mapping=json.loads(MAPPING.read_text());assert len(mapping['clips'])==5
 
 for evidence in mapping['evidence']:assert sha_file(Path(evidence['path']))==evidence['sha256']
 proof={row['sha256']:row for row in mapping['clips']};base=json.loads((args.base/'report.json').read_text());verify_files(args.base,base)
 groups={}
 for row in json.loads((args.base/'cases.json').read_text()):groups.setdefault((row['clip'],row['prepared'],row['channel']),row)
 assert len(groups)==5;args.out.mkdir(parents=True);records=[]
 with(args.out/'metadata-input.bin').open('xb')as binary:
  for(clip,prepared,channel),row in groups.items():
   payload=(args.base/clip).read_bytes();descriptors=(args.base/prepared).read_bytes();sha=sha_file(args.base/clip);assert sha in proof
   ch_at=word(payload,word(payload,328)+4*channel);rows,key,dc,da=struct.unpack_from('<4I',payload,ch_at+212);assert 0<rows<=160 and dc<=8 and len(descriptors)==32*dc
   for i in range(dc):
    d=struct.unpack_from('<8I',descriptors,32*i);original=struct.unpack_from('<8I',payload,da+32*i);assert d[1:4]==original[1:4] and d[5:]==original[5:]
    if d[0]&15==3:assert not(d[0]&0x40) and 3<=d[4]<=8 and d[2]+40<=d[3] and key+(rows-1)*d[3]+d[2]+40<=len(payload)
   assert word(payload,8)==25 and word(payload,4)==len(payload);fl,iv,fr=struct.unpack_from('<III',payload,280);assert bounded(fr,iv)
   for name,raw in[(clip,payload),(prepared,descriptors)]:
    dest=args.out/name;dest.parent.mkdir(parents=True,exist_ok=True)
    with dest.open('xb')as f:f.write(raw)
   binary.write(metadata_request(flags=fl,frames=fr,interval=iv));records.append(dict(metadata_case=len(records),**row))
 save(args.out/'metadata-cases.json',records)
 with(args.out/'metadata-sources.tsv').open('x')as f:
  f.write('metadata_case\tclip\n')
  for row in records:f.write(str(row['metadata_case'])+'\t'+row['clip']+'\n')
def prepare_clock(args):
 records=json.loads((args.out/'metadata-cases.json').read_text());native=args.metadata_output.read_bytes();assert len(native)==len(records)*24
 cases=[];rejections=[]
 with(args.out/'clock-input.bin').open('xb')as binary:
  for row in records:
   reply=native[24*row['metadata_case']:24*(row['metadata_case']+1)];cw,sw,mxcsr=struct.unpack_from('<HHI',reply,16);assert cw==0x027f and sw&0x3800==0 and mxcsr&~0x3f==0x1f80
   commit,duration,loop=struct.unpack_from('<IQI',reply);assert loop in[0,1]
   d=struct.unpack('<f',struct.pack('<I',commit))[0];wide=struct.unpack('<d',struct.pack('<Q',duration))[0];assert math.isfinite(d)and d>0 and float(d)==wide
   payload=(args.out/row['clip']).read_bytes();assert loop==((word(payload,280)>>3)&1);frames=word(payload,288)
   for mode in[0,1]:
    for quarter in range(4):
     if loop:rejections.append(dict(metadata_case=row['metadata_case'],clock_mode=mode,quarter=quarter,error='Composition.Clock.UnsupportedLoop'));continue
     time=number(clock.bits(d*quarter/4));delta=number(clock.bits(time/2));previous=delta
     request=clock.request(mode=mode,cw=0x027f,time=clock.bits(time if mode==0 else delta),previous=clock.double(previous if mode else 0.),duration=duration,frames=frames)
     exact=Fraction.from_float(number(word(request,8)))+(Fraction.from_float(previous)if mode else 0);assert exact==Fraction.from_float(time)and 0<=exact<Fraction.from_float(wide)
     binary.write(request);cases.append(dict(case=len(cases),metadata_case=row['metadata_case'],clip=row['clip'],prepared=row['prepared'],channel=row['channel'],source_sha256=row['source_sha256'],source_identity=row['source'],binding=row['binding'],clock_mode=mode,quarter=quarter,derived_duration_bits=f'{duration:016X}',resource_frames_bits=f'{frames:08X}',cw='027F'))
 assert len(cases)==32 and len(rejections)==8;save(args.out/'cases.json',cases);save(args.out/'loop-rejections.json',rejections)
 with(args.out/'cases.tsv').open('x',newline='')as f:
  w=csv.writer(f,delimiter='\t',lineterminator='\n');w.writerow(['case','clip','prepared','channel','cw','binding0','binding1','binding2','binding3','metadata_case'])
  for row in cases:w.writerow([row['case'],row['clip'],row['prepared'],row['channel'],row['cw'],*[f'{x:08X}'for x in row['binding']],row['metadata_case']])
 save(args.out/'clock-preparation.json',dict(schema='source-metadata-derived-clock-preparation-v1',cases=32,metadata_output=str(args.metadata_output.resolve()),metadata_output_sha256=sha_file(args.metadata_output),source_pins={str(MAPPING):sha_file(MAPPING)},files={n:sha_file(args.out/n)for n in['clock-input.bin','cases.json','cases.tsv','metadata-cases.json','metadata-input.bin','loop-rejections.json']},native_executed=False))
def prepare_curves(args):
    report=verify_initial(args);cases=json.loads((args.out/'cases.json').read_text())
    assert args.clock_output.stat().st_size==32*28
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
    assert len(records)==72
    save(args.out/'curve-cases.json',records)
    save(args.out/'curve-preparation.json',dict(schema='clock-root-data-fresh-curve-preparation-v1',clock_cases=32,curve_cases=72,
        clock_output=str(args.clock_output.resolve()),clock_output_sha256=sha_file(args.clock_output),
        clock_preparation_sha256=sha_file(args.out/'clock-preparation.json'),
        files={n:sha_file(args.out/n)for n in ['curve-input.bin','curve-cases.json']},native_executed=False,
        scope='Fresh full-channel requests at literal actual native clock frame bits. No old sampled reply or midpoint is substituted.'))
    print(json.dumps(dict(stage='fresh_curve_requests',clock_cases=32,curve_cases=72,sha256=sha_file(args.out/'curve-input.bin'))))

def prepare_contributions(args):
    verify_initial(args);stage=json.loads((args.out/'curve-preparation.json').read_text());verify_files(args.out,stage)
    assert sha_file(Path(stage['clock_output']))==stage['clock_output_sha256']
    records=json.loads((args.out/'curve-cases.json').read_text());cases=json.loads((args.out/'cases.json').read_text())
    assert args.curve_output.stat().st_size==len(records)*476
    grouped={i:[]for i in range(32)}
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
    save(args.out/'contribution-preparation.json',dict(schema='clock-root-data-contribution-preparation-v1',cases=32,
        fresh_curve_output=str(args.curve_output.resolve()),fresh_curve_output_sha256=sha_file(args.curve_output),
        curve_preparation_sha256=sha_file(args.out/'curve-preparation.json'),
        files={'contribution-input.bin':sha_file(args.out/'contribution-input.bin')},native_executed=False))
    print(json.dumps(dict(stage='contribution_requests',cases=32,sha256=sha_file(args.out/'contribution-input.bin'))))

def prepare_consumer(args):
    verify_initial(args);stage=json.loads((args.out/'contribution-preparation.json').read_text());verify_files(args.out,stage)
    assert sha_file(Path(stage['fresh_curve_output']))==stage['fresh_curve_output_sha256']
    assert args.contribution_output.stat().st_size==32*184
    with args.contribution_output.open('rb')as native,(args.out/'consumer-input.bin').open('xb')as requests:
        for case in range(32):
            reply=native.read(184);cw,sw,mxcsr=struct.unpack_from('<HHI',reply,176)
            assert cw==0x027F and sw&0x3800==0 and mxcsr&~0x3F==0x1F80 and word(reply,172)==1
            requests.write(consumer_request(reply[:164]))
    save(args.out/'consumer-preparation.json',dict(schema='clock-root-data-consumer-preparation-v1',cases=32,
        contribution_output=str(args.contribution_output.resolve()),contribution_output_sha256=sha_file(args.contribution_output),
        contribution_preparation_sha256=sha_file(args.out/'contribution-preparation.json'),
        files={'consumer-input.bin':sha_file(args.out/'consumer-input.bin')},native_executed=False))
    print(json.dumps(dict(stage='consumer_requests',cases=32,sha256=sha_file(args.out/'consumer-input.bin'))))

def clone(source,dest):
 assert not dest.exists()and not dest.resolve().is_relative_to(source.resolve());dest.mkdir(parents=True)
 for p in source.rglob('*'):
  assert not p.is_symlink()
  if p.is_file():
   d=dest/p.relative_to(source);d.parent.mkdir(parents=True,exist_ok=True)
   with d.open('xb')as f:f.write(p.read_bytes())
def strict_clock_domain(root):
 cases=json.loads((root/'cases.json').read_text());raw=(root/'clock-input.bin').read_bytes();assert len(cases)==32 and len(raw)==32*64
 for case in cases:
  rec=raw[64*case['case']:64*(case['case']+1)];mode,cw,zero,time,previous,duration,frames,loop=struct.unpack_from('<IHHIQQII',rec)
  assert mode in[0,1]and cw==0x027f and zero==loop==0 and rec[36:]==bytes(28)
  d=struct.unpack('<d',struct.pack('<Q',duration))[0];p=struct.unpack('<d',struct.pack('<Q',previous))[0]
  assert math.isfinite(d)and d>0 and math.isfinite(p)and math.isfinite(number(time))
  exact=Fraction.from_float(number(time))+(Fraction.from_float(p)if mode else 0)
  assert 0<=exact<Fraction.from_float(d)
  committed=number(clock.bits(float(exact)));assert math.isfinite(committed)and 0<=committed<d
  payload=(root/case['clip']).read_bytes();assert word(payload,288)==frames and word(payload,280)&8==0
 return dict(cases=32,finite_strict_first_duration_verified=True,source_frames_verified=True,clock_frame_projection_predicted=False)
def verify_stage(root):
 r=json.loads((root/'stage.json').read_text());verify_files(root,r)
 for path,expected in r['pins'].items():assert sha_file(Path(path))==expected,path
 assert r['tool_sha256']==sha_file(Path(__file__))
 if r['stage']!='metadata':strict_clock_domain(root)
 for path,expected in r['actual_native_output_pins'].items():assert sha_file(Path(path))==expected,path
 return r
def finish(args,stage,parent):
 if stage!='metadata':strict_clock_domain(args.out)
 paths=[Path(__file__),MAPPING,ROOT/'tools/cell_runtime_clock_root_data_fixture.py',ROOT/'tools/cell_runtime_clip_metadata_inputs.py',ROOT/'crates/spore-core/src/cell_runtime_clip_metadata.rs',ROOT/'crates/spore-core/src/cell_runtime_source_clock_root_data.rs',ROOT/'crates/spore-core/examples/cell_runtime_source_clock_root_data_verify.rs.pending']
 paths +=[Path(e['path'])for e in json.loads(MAPPING.read_text())['evidence']]
 paths +=[args.base/'report.json',args.base/'cases.json']
 paths +=[ROOT/('crates/spore-core/src/'+name+'.rs')for name in['cell_runtime_clock','cell_runtime_clock_root_data','cell_runtime_root_data','cell_runtime_curves','cell_runtime_contributions','cell_runtime_data','cell_runtime_modifiers','native_sse','native_x87','anim']]
 refs={}if parent is None else dict(parent['actual_native_output_pins'])
 for option in['metadata_output','clock_output','curve_output','contribution_output']:
  path=getattr(args,option)
  if path is not None:refs[str(path.resolve())]=sha_file(path)
 save(args.out/'stage.json',dict(schema='immutable-source-clock-root-data-stage-v1',stage=stage,parent=None if parent is None else str(args.input_dir.resolve()),parent_manifest_sha256=None if parent is None else sha_file(args.input_dir/'stage.json'),tool_sha256=sha_file(Path(__file__)),pins={str(p):sha_file(p)for p in paths},actual_native_output_pins=refs,files={str(p.relative_to(args.out)):sha_file(p)for p in args.out.rglob('*')if p.is_file()},metadata_cases=5,clock_cases=32,curve_cases=72,constructor_loop_rejections=8,native_executed=False,scope='Only measuredv25 constructor-default metadata. Explicit source selection/binding/actorcontext,positiveblend1; noqueuedforcedloop/lifecycle/eventscheduling claim.'))
 print(json.dumps(dict(stage=stage,metadata_cases=5,clock_cases=32,native_executed=False)))
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--input-dir',type=Path);p.add_argument('--base',type=Path,default=ROOT/'target/source-cell/runtime-root-data-join-v3')
 phase=p.add_mutually_exclusive_group()
 for n in['metadata-output','clock-output','curve-output','contribution-output']:phase.add_argument('--'+n,type=Path)
 args=p.parse_args();args.out.resolve().relative_to((ROOT/'target').resolve());parent=None
 if args.input_dir is None:
  assert all(getattr(args,n)is None for n in['metadata_output','clock_output','curve_output','contribution_output']);prepare_metadata(args);stage='metadata'
 else:
  parent=verify_stage(args.input_dir);clone(args.input_dir,args.out);(args.out/'stage.json').rename(args.out/('ancestor-'+parent['stage']+'-stage.json'))
  if args.metadata_output:assert parent['stage']=='metadata';prepare_clock(args);stage='clock'
  elif args.clock_output:assert parent['stage']=='clock';prepare_curves(args);stage='curve'
  elif args.curve_output:assert parent['stage']=='curve';prepare_contributions(args);stage='contribution'
  elif args.contribution_output:assert parent['stage']=='contribution';prepare_consumer(args);stage='consumer'
  else:raise AssertionError('dependent phase required')
 finish(args,stage,parent)
if __name__=='__main__':main()
