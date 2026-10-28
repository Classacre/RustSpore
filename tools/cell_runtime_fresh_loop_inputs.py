"""Prepare source-backed256B/fullfresh activation histories; never execute."""
import argparse,json,struct,math,hashlib
from fractions import Fraction
from pathlib import Path
from cell_runtime_curves_fixture import CONTEXTS
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def f32(bits):return struct.unpack('<f',struct.pack('<I',bits))[0]
def bits(x):return struct.unpack('<I',struct.pack('<f',x))[0]
def word(raw,at):return struct.unpack_from('<I',raw,at)[0]
def bracket(value):
 # Conservative adjacentbinary32 enclosure; no sourceframe prediction.
 if value==0:return Fraction(0),Fraction(0)
 nearest=bits(float(value));n=Fraction.from_float(f32(nearest));lo=nearest if n<=value else nearest-1;hi=nearest if n>=value else nearest+1
 return Fraction.from_float(f32(lo)),Fraction.from_float(f32(hi))
def domain(raw,duration):
 assert len(raw)==256;cw=struct.unpack_from('<H',raw,4)[0];assert cw in CONTEXTS and raw[6:8]==bytes(2)
 assert word(raw,8)==1 and word(raw,12)==0 and word(raw,16)<=16 and word(raw,40)==8 and raw[44:60]==bytes(16)and all(word(raw,a)==1 for a in[60,64,68])and raw[72:128]==bytes(56)
 assert 0<word(raw,36)<=16 and 0x3f800000<word(raw,32)<0x7f800000 and 0<word(raw,28)<0x7f800000
 d=Fraction.from_float(duration);lo=hi=Fraction(0)
 for step in range(8):
  at=128+16*step;mode,val,count,reserved=struct.unpack_from('<4I',raw,at);assert mode in[0,1]and count<=word(raw,16)and reserved==0 and math.isfinite(f32(val))
  v=Fraction.from_float(f32(val))
  if mode==0:lo=hi=v
  else:
   low,high=lo+v,hi+v;assert low>=0
   lo=bracket(low)[0];hi=bracket(high)[1]
  assert lo>=0 and hi/d<Fraction(1<<30) # amplemargin below counterCVT2^31
 return True

def request(serial,cw,capacity,generation,flags,interval,frames,channels,ops):
 raw=bytearray(256);struct.pack_into('<IHH',raw,0,serial,cw,0)
 for at,value in[(8,1),(12,0),(16,capacity),(20,generation),(24,flags),(28,interval),(32,frames),(36,channels),(40,8),(60,1),(64,1),(68,1)]:struct.pack_into('<I',raw,at,value)
 counts=[0,1,16,8,3,0,15,2]
 for i,(mode,val)in enumerate(ops):struct.pack_into('<4I',raw,128+16*i,mode,val,counts[i]if capacity else 0,0)
 return bytes(raw)

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--source-native',type=Path,default=ROOT/'target/source-cell/runtime-source-clock-root-data-native-run-v1');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());m=json.loads((a.host_dir/'manifest.json').read_text());assert sha(a.host_dir/'host-x86.asm')==m['host_source_sha256'];accepted=json.loads((a.source_native/'rust-comparison-v1.json').read_text());assert accepted['accepted'] and accepted['records']==32
 native=json.loads((a.source_native/'report.json').read_text());stage=Path(native['final_stage']);meta=Path(native['metadata_output']);replies=meta.read_bytes();sources=json.loads((stage/'metadata-cases.json').read_text());assert len(sources)==5 and len(replies)==120
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'clips').mkdir();cases=[];sourcepins={str(a.source_native/'rust-comparison-v1.json'):sha(a.source_native/'rust-comparison-v1.json'),str(a.source_native/'report.json'):sha(a.source_native/'report.json'),str(stage/'metadata-cases.json'):sha(stage/'metadata-cases.json'),str(meta):sha(meta)}
 with(a.out/'input.bin').open('xb')as binary:
  for clip_index,row in enumerate(sources):
   path=stage/row['clip'];payload=path.read_bytes();assert word(payload,8)==25 and word(payload,4)==len(payload);sourcepins[str(path)]=sha(path);dest=a.out/'clips'/path.name;dest.write_bytes(payload);flags,interval,frames=struct.unpack_from('<III',payload,280);channels=word(payload,324)
   r=replies[24*row['metadata_case']:24*(row['metadata_case']+1)];d=f32(word(r,0));assert struct.unpack_from('<d',r,4)[0]==float(d)and 0<d<100 and word(r,12)==((flags>>3)&1)
   dword=word(r,0);two=bits(d*2);families=[('absolute',[(0,v)for v in[0,bits(d/4),bits(d/2),dword-1,dword,bits(d*1.5),two,bits(d*7.75)]]),('delta',[(1,bits(v))for v in[d/4,d/4,d/2,d/2,d/2,d*5.75,-d/4,-0.]]),('backtrack',[(0,bits(d*v))for v in[7.75,1.5,7.75,0,8.25,8.25,2,9.5]]),('boundaries',[(0,v)for v in[dword-1,dword+1,dword,two-1,two+1,dword,bits(d*15.5),bits(d*16)]]),('large_integer',[(0,0x4a800000),(0,0x4b000001),(0,0x4b800001),(0,0x4e800000),(0,0x4f000000),(1,0xcf000000),(0,0x4effffff),(0,0)])]
   for cw in CONTEXTS:
    for family_index,(family,ops)in enumerate(families):
     serial=[0,1,0xfffffffe,0xffffffff,0x12345678][family_index];generation=[0,0xfffffffe,0xffffffff,17,0x12345678][family_index];cap=16 if(clip_index+family_index)%2 else 0;raw=request(serial,cw,cap,generation,flags,interval,frames,channels,ops);domain(raw,d);binary.write(raw);cases.append(dict(case=len(cases),family=family,clip=str(dest.relative_to(a.out)),source_sha256=sha(path),source_identity=row['source'],cw=f'{cw:04X}',duration_bits=f'{struct.unpack_from("<Q",r,4)[0]:016X}',resource_frames=f'{frames:08X}',channels=channels,serial_seed=f'{serial:08X}',generation_seed=f'{generation:08X}',event_capacity=cap))
 assert len(cases)==300;(a.out/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
 with(a.out/'cases.tsv').open('x')as f:
  f.write('case\tclip\tfamily\tcw\n')
  for row in cases:f.write(str(row['case'])+'\t'+row['clip']+'\t'+row['family']+'\t'+row['cw']+'\n')
 raw=(a.out/'input.bin').read_bytes();valid=raw[:256];protocols={};invalid={'cw_unmasked':(4,struct.pack('<H',0x027e)),'cw_reserved':(4,struct.pack('<H',0x017f)),'reserved_header':(6,struct.pack('<H',1)),'fresh_mode':(8,struct.pack('<I',0)),'initial_event_count':(12,struct.pack('<I',1)),'capacity':(16,struct.pack('<I',17)),'interval_zero':(28,bytes(4)),'frames_one':(32,struct.pack('<I',0x3f800000)),'channel_zero':(36,bytes(4)),'channel_over_bound':(36,struct.pack('<I',17)),'step_count':(40,struct.pack('<I',7)),'sampling_mask':(44,struct.pack('<I',1)),'crt_policy':(60,bytes(4)),'property_mode':(64,bytes(4)),'owner_boundary':(68,bytes(4)),'header_tail':(72,struct.pack('<I',1)),'step_mode':(128,struct.pack('<I',2)),'step_reserved':(140,struct.pack('<I',1)),'negative_absolute':(132,struct.pack('<I',0xbf800000)),'nonfinite_absolute':(132,struct.pack('<I',0x7fc12345)),'counter_indefinite_domain':(132,struct.pack('<I',0x7f7fffff))}
 for name,(at,value)in invalid.items():q=bytearray(valid);q[at:at+len(value)]=value;path=a.out/(name+'.bin');path.write_bytes(q);protocols[name]=sha(path)
 q=bytearray(valid);struct.pack_into('<III',q,128,1,0xbf800000,0);path=a.out/'negative_first_delta.bin';path.write_bytes(q);protocols['negative_first_delta']=sha(path)
 q=bytearray(valid);struct.pack_into('<I',q,136,17);path=a.out/'pending_count.bin';path.write_bytes(q);protocols['pending_count']=sha(path)
 paths=[Path(__file__),ROOT/'tools/cell_runtime_fresh_loop_native_probe.py',ROOT/'tools/cell_runtime_fresh_loop_sparse_carrier.py',ROOT/'tools/cell_runtime_fresh_loop_hybrid_carrier_audit_v1.py',ROOT/'crates/spore-core/src/cell_runtime_fresh_loop.rs',ROOT/'crates/spore-core/examples/cell_runtime_fresh_loop_verify.rs.pending',a.host_dir/'manifest.json',a.host_dir/'host-x86.asm',ROOT/'target/source-cell/runtime-fresh-aid-sparse-native-run-v1/rust-comparison-v1.json',ROOT/'target/source-cell/runtime-controller-loop-native-run-v1/rust-comparison-v1.json',ROOT/'target/source-cell/runtime-loop-clock-native-run-v1/rust-comparison-v1.json']
 pins={str(p):sha(p)for p in paths};pins.update(sourcepins);(a.out/'report.json').write_text(json.dumps(dict(schema='fresh-full-activation-loop-inputs-v1',records=300,clock_calls_per_record=9,total_clock_calls=2700,contexts=12,source_clips=5,source_channel_counts=sorted(set(row['channels']for row in cases)),protocols=protocols,input_sha256=sha(a.out/'input.bin'),cases_sha256=sha(a.out/'cases.json'),pins=pins,domain='ConservativeFractionenclosures acrossallsourceCW forprivateF32history; nonnegativefinitecommittedtime,quotientmargin<2^30,actualsource metadata/durations fromacceptednative5; no predictedframeoutputs/inverseclockmapping.',no_execution=True),indent=2)+'\n');print(300)
if __name__=='__main__':main()
