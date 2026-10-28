"""Prepare original endpoint clip and fresh-sampler host, without compilation/execution."""
import hashlib,json,struct,shutil
from pathlib import Path
import cell_runtime_curves_fixture as census
import cell_runtime_fresh_sampled_channel_native_probe as base
import cell_runtime_fresh_sampled_channel_native_probe_v3 as v3
import cell_runtime_fresh_sampled_channel_inputs as fixture
ROOT=base.ROOT; OUT=ROOT/'target/source-cell/runtime-fresh-sampled-channel-endpoint-prep-v1'
S='8df1d42720ccf8090a578190646a33b76a4b8eaca431a65adff3c0552f093845'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def word(b,a):return struct.unpack_from('<I',b,a)[0]
def f32(x):return struct.unpack('<f',struct.pack('<f',x))[0]
def main():
 assert not OUT.exists(); package=Path('D:/Games/SPORE Collection/Data/PatchData.package');idx,_=census.package_index(package);raw=census.source_payload(package,idx[(0xD83A2CD1,0xEE17C6AD,0)])
 assert hashlib.sha256(raw).hexdigest()==S and raw[:4]==b'ANIM' and word(raw,4)==len(raw) and word(raw,8)==25 and len(raw)%4==0 and len(raw)<=base.RESOURCE_BYTES
 n=word(raw,324); table=word(raw,328);assert 0<n<=16 and table+4*n<=len(raw)
 channels=[]
 for i in range(n):
  at=word(raw,table+4*i); assert at+228<=len(raw) and raw[at:at+4]==b'CHAN';count,keys,num,descriptors=struct.unpack_from('<4I',raw,at+212);assert 0<count<=46 and descriptors+32*num<=len(raw)
  for j in range(num):
   flags,tag,offset,stride,slot=struct.unpack_from('<5I',raw,descriptors+32*j);assert stride>0 and keys+(count-1)*stride+offset<=len(raw)
  channels.append(dict(at=at,keys=keys,descriptors=descriptors,count=count))
 ch=channels[0]; assert word(raw,ch['at']+136+9*4)&1
 stride=word(raw,ch['descriptors']+12);times=[word(raw,ch['keys']+i*stride) for i in range(ch['count'])];assert times==sorted(times) and times[-1]==197 and ch['count']==21
 events=[i for i in range(ch['count']) if raw[ch['keys']+i*stride+10]];assert not events
 mask=bytearray(444);kinds=[];slots=[];ranges=[]
 for i in range(word(raw,ch['at']+220)):
  d=ch['descriptors']+32*i;kind=word(raw,d)&15;slot=word(raw,d+16);offset=word(raw,d+8); size=[20,80,100,40][kind];assert kind<4 and slot<9 and offset+size<=stride and ch['keys']+(ch['count']-1)*stride+offset+size<=len(raw);assert word(raw,d+12)==stride
  start=4+[0,20,100,200,240,280,320,360,400][slot];mask[start:start+size]=bytes([1])*size;kinds.append(kind);slots.append(slot);ranges.append([start,start+size])
 assert kinds==[0,1,2,3,3] and slots==[0,1,2,4,5] and raw[ch['descriptors']+4:ch['descriptors']+8]==b'INFO'
 interval=struct.unpack_from('<f',raw,284)[0];frames=struct.unpack_from('<f',raw,288)[0];assert frames==218.0 and interval>0
 duration=f32(f32(frames-1.0)*interval); target=f32(duration*(207.0/217.0)); assert 0<target<duration
 OUT.mkdir(parents=True); inp=OUT/'inputs';inp.mkdir();(inp/'clips').mkdir();p=inp/'clips'/f'{S}.anim';p.write_bytes(raw)
 candidate=dict(path=p,raw=raw,channels=channels,table=table,flags=word(raw,280),interval=word(raw,284),frames=word(raw,288),mask=bytes(mask));original=base.source_cases();cases=[candidate]+original[1:];base.source_cases=lambda:cases
 images=base.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));base.audit_source(images);asm=v3.host_source(images);assert asm.count(v3.SHADOW)==3 and max(map(len,asm.splitlines()))<512
 host=OUT/'host';host.mkdir();(host/'host-x86.asm').write_text(asm)
 old=ROOT/'target/source-cell/runtime-fresh-sampled-channel-inputs-v1';oldraw=(old/'input.bin').read_bytes();oldcases=json.loads((old/'cases.json').read_text());binary=bytearray();rows=[]
 for i in range(12):
  source=oldcases[i*5];b=bytearray(oldraw[i*5*256:(i*5+1)*256]);
  for pos,val in [(24,candidate['flags']),(28,candidate['interval']),(32,candidate['frames']),(36,n),(72,0)]:struct.pack_into('<I',b,pos,val)
  for j,t in enumerate([target,0.0,target,f32(duration*0.5),target,0.0,target,f32(duration*0.25)]):struct.pack_into('<4I',b,128+j*16,0,struct.unpack('<I',struct.pack('<f',t))[0],0,0)
  fixture.domain(b,0,duration);binary+=b;rows.append(dict(case=i,clip='clips/'+p.name,family='endpoint-interpolation-alternating',cw=source['cw'],source_case=0,duration_bits=f'{struct.unpack("<Q",struct.pack("<d",duration))[0]:016X}',source_sha256=S))
 (inp/'input.bin').write_bytes(binary);(inp/'cases.json').write_text(json.dumps(rows,indent=2)+'\n');(inp/'cases.tsv').write_text('case\tclip\tfamily\tcw\n'+''.join(f"{r['case']}\t{r['clip']}\t{r['family']}\t{r['cw']}\n" for r in rows))
 src=ROOT/'crates/spore-core/examples/cell_runtime_fresh_sampled_channel_full_verify_v2.rs'; ver=src.read_text();ver=ver.replace('300*256','12*256').replace('300*5452','12*5452').replace('assert_eq!(cases,300)','assert_eq!(cases,12)');(OUT/'endpoint_verify_v1.rs.pending').write_text(ver)
 pins=[Path(__file__),Path(base.__file__),Path(v3.__file__),Path(fixture.__file__),src,ROOT/'crates/spore-core/src/cell_runtime_fresh_sampled_channel_v2.rs',ROOT/'target/source-cell/runtime-curves-source-census-v1/components.jsonl',ROOT/'target/source-cell/runtime-fresh-sampled-channel-corpus-run-v3-independent-static-review.json',p,inp/'input.bin',inp/'cases.json',inp/'cases.tsv',host/'host-x86.asm',OUT/'endpoint_verify_v1.rs.pending'];pins.extend(c['path'] for c in original[1:])
 report=dict(schema='fresh-sampled-channel-endpoint-source-prep-v1',compiled=False,native_executed=False,source_candidate_sha256=S,source_identity='Data/PatchData.package D83A2CD1/EE17C6AD/00000000',original_size=len(raw),channel_count=n,channel_key_counts=[c['count'] for c in channels],first_channel_times=times,first_channel_event_keys=events,first_channel_descriptor_kinds=kinds,first_channel_slots=slots,wrapper_relative_write_ranges=ranges,source_interval=interval,source_duration_predicted=duration,absolute_test_time=target,expected_frame=207,expected_branch='last-key endpoint; prediction only, require observed native frame >197 and <217',records=12,snapshots=108,contexts=12,unchanged_guard_capacity=16,event_guard_max=2,remaining_masks_zero=True,host_sha256=sha(host/'host-x86.asm'),rust_verifier_sha256=sha(OUT/'endpoint_verify_v1.rs.pending'),pins={str(q):sha(q) for q in pins},limitations=['Duration derived from source SSE arithmetic and candidate metadata; confirm native metadata/projection.','Generated host uncompiled, carrier unaudited, and runner not prepared; no native execution or endpoint acceptance.','Endpoint and interpolation alternate; events absent in this clip. No full binding, actor, lifetime, dispatch or scheduling acceptance.','A new linked audit must bind the substituted source cases and review the generated host before any native execution.'])
 (OUT/'preparation.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ['channel_key_counts','source_duration_predicted','absolute_test_time','host_sha256']}))
if __name__=='__main__':main()
