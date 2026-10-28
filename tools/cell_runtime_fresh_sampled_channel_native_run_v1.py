"""Frozen sampled-channel runner: default static preflight; --run one attempt."""
import argparse,json,struct,hashlib
from pathlib import Path
from types import SimpleNamespace
import cell_runtime_fresh_sampled_channel_inputs as fixture
import cell_runtime_fresh_sampled_channel_hybrid_audit_v2 as linked
from cell_runtime_data_native_run import pinned,saved_file_run
ROOT=fixture.ROOT
PLAN=ROOT/'target/source-cell/runtime-fresh-sampled-channel-runner-plan-v1/plan.json'
CARRIER='ff1beaccbfecb5aab0a7637cd8822392b6f6d0390b6da319a7fe2c57e1c23036'
OWNED='0de9d295750e04a018b363c503a201bff588d336bda6d99119990be2377640b2'
MAP='53dc2d95b61fb7232a468d238872322b40037b6cd7b3335d63b20d8644b3fee2'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,r):
 with p.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
def resolve(p):return Path(p)if Path(p).is_absolute()else ROOT/p
def inspect(raw,reply,cases):
 assert len(raw)%256==0 and len(reply)==len(raw)//256*5452 and len(cases)==len(raw)//256
 projection=bytearray();counts=dict(endpoint_fullslots=0,interpolation=0,authored_events=0,snapshots=0);details=[]
 sources=fixture.host.source_cases()
 for i,case in enumerate(cases):
  req=raw[i*256:(i+1)*256];res=reply[i*5452:(i+1)*5452];source=sources[case['source_case']];ch=source['channels'][0];s=source['raw'];n=ch['count'];d=ch['descriptors'];stride=fixture.word(s,d+12);offset=fixture.word(s,d+8)
  assert offset==0 and n>0;keys=[ch['keys']+k*stride for k in range(n)];timestamps=[fixture.word(s,k)for k in keys];events=[k for k,at in enumerate(keys)if s[at+10]]
  expected_ctor=bytearray(440);struct.pack_into('<I',expected_ctor,4,fixture.word(req,76));assert res[:440]==expected_ctor
  assert fixture.word(res,440)==(fixture.word(req,76)+len(source['channels']))&0xffffffff and fixture.word(res,444)==0
  for j in range(9):
   r=res[448+j*556:448+(j+1)*556];assert r[10:12]==bytes(2) and struct.unpack_from('<H',r,548)[0]==struct.unpack_from('<H',req,4)[0]
   assert struct.unpack_from('<H',r,550)[0]&0x3800==0 and fixture.word(r,552)&~0x3f==0x1f80
   assert fixture.word(r,60)==16 and [fixture.word(r,x)for x in[64,68,72]]==[1,1,0] and fixture.word(r,76)==fixture.word(req,32)
   frame=struct.unpack_from('<f',r,16)[0];upper=sum(t<=frame for t in timestamps);left=min(n-1,max(0,upper-1));right=min(n-1,upper)
   expected_events=[k for k in events if timestamps[k]<=frame and fixture.word(r,32)>0];count=fixture.word(r,528);assert count==fixture.word(r,56)==len(expected_events)<=2
   actual=[struct.unpack_from('<H',r,532+8*k)[0]for k in range(count)];assert actual==[0]*count
   assert [fixture.word(r,536+8*k)for k in range(count)]==expected_events
   # INFO is a whole source20B copy on either path. Endpoint replaces every
   # complete authored slot; interpolation's numeric result is left to Rust.
   assert r[88:108]==s[keys[left]:keys[left]+20]
   if left==right:
    counts['endpoint_fullslots']+=1
    for k in range(fixture.word(s,ch['at']+220)):
     descriptor=d+32*k;kind=fixture.word(s,descriptor)&15;slot=fixture.word(s,descriptor+16);start=[0,20,100,200,240,280,320,360,400][slot];size=[20,80,100,40][kind];at=keys[left]+fixture.word(s,descriptor+8);assert r[88+start:88+start+size]==s[at:at+size]
   else:counts['interpolation']+=1
   counts['authored_events']+=count;counts['snapshots']+=1;projection+=r[:532]
   for k in range(2):projection+=(r[532+8*k:534+8*k]+r[536+8*k:540+8*k])if k<count else bytes(6)
   details.append(dict(case=case['case'],snapshot=j,frame_bits=f'{fixture.word(r,16):08X}',left=left,right=right,event_keys=expected_events))
 assert len(projection)==len(cases)*4896
 return bytes(projection),counts,details
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--runner-review',type=Path);p.add_argument('--run',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());plan=json.loads(PLAN.read_text());files=[PLAN,Path(__file__)]
 for name,value in plan['pins'].items():q=resolve(name);assert sha(q)==value,name;files.append(q)
 assert sha(Path(__file__))==plan['runner_sha256'];host=a.host_dir;assert sha(host/'carrier-x86.exe')==CARRIER and sha(host/'host-x86.exe')==OWNED and sha(host/'host-x86.map')==MAP
 actual=linked.previous.audit(SimpleNamespace(carrier=host/'carrier-x86.exe',owned_exe=host/'host-x86.exe',owned_map=host/'host-x86.map',source_exe=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),source_crt=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll')));assert actual['static_pass'] and actual['carrier_sha256']==CARRIER
 files.extend(q for q in host.iterdir()if q.is_file());peer=json.loads(resolve(plan['linked_peer']).read_text());assert peer['static_pass'] and not peer['native_executed'] and peer['carrier_sha256']==CARRIER
 if a.runner_review:
  review=json.loads(a.runner_review.read_text());assert review['no_blocking_static_issue_found'] and review['native_executed']is False
  assert plan['runner_sha256']in review['pins'].values() and sha(PLAN)in review['pins'].values() and CARRIER in review['pins'].values() and OWNED in review['pins'].values() and MAP in review['pins'].values()
  for name,value in review['pins'].items():q=resolve(name);assert sha(q)==value,name;files.append(q)
  files.append(a.runner_review)
 if a.run:assert a.runner_review,'Independent exact-runner review is required'
 input_dir=resolve(plan['inputs']);report=json.loads((input_dir/'report.json').read_text());raw=(input_dir/'input.bin').read_bytes();cases=json.loads((input_dir/'cases.json').read_text());assert len(raw)==76800 and len(cases)==300
 for i,row in enumerate(cases):fixture.domain(raw[256*i:256*(i+1)],row['source_case'],struct.unpack('<d',bytes.fromhex(row['duration_bits'])[::-1])[0])
 for name,value in report['protocols'].items():assert sha(input_dir/(name+'.bin'))==value
 asm=(host/'host-x86.asm').read_text();assert 'movd xmm0,Poison\npshufd xmm0,xmm0,0'in asm and 'mov Poison,0\ncall RunOnce'in asm and 'mov Poison,0FFFFFFFFh\ncall RunOnce'in asm
 a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(native_executed=False,execution_requested=a.run,records=300,snapshots=2700,single_attempt=True,retries=0,actual_linked_audit=True,linked_peer=plan['linked_peer'],runner_peer=bool(a.runner_review),carrier_sha256=CARRIER))
 if not a.run:print('Preflight PASS; no native execution');return
 try:
  with pinned(list(dict.fromkeys(q.resolve()for q in files)),a.out):
   proto=a.out/'protocol';proto.mkdir();seen={};smoke_counts=dict(endpoint_fullslots=0,interpolation=0,authored_events=0,snapshots=0)
   def execute(label,data,code,expected=None):
    req=proto/(label+'-input.bin');req.write_bytes(data);folder=proto/(label+'-pins');folder.mkdir()
    with pinned([req],folder):status,out=saved_file_run(host/'carrier-x86.exe',req,proto,label)
    assert status['exit']==code and status['stderr_bytes']==0 and not status['timeout'],(label,status)
    got=out.read_bytes()
    if expected is not None:assert got==expected,label
    return got
   def positive(index,label):
    req=raw[index*256:(index+1)*256];got=execute(label,req,0);projection,counts,details=inspect(req,got,[cases[index]]);seen[index]=got
    for key,value in counts.items():smoke_counts[key]+=value
    save(proto/(label+'-inspection.json'),dict(counts=counts,details=details,projection_sha256=hashlib.sha256(projection).hexdigest()));return got
   # The first executed input is one actual complete source history.
   ref=positive(0,'first_genuine_history')
   for clip in range(5):
    for family in range(5):
     index=clip*60+family
     if index:positive(index,f'source{clip}_history{family}')
   assert all(smoke_counts[key]>0 for key in ['endpoint_fullslots','interpolation','authored_events']),smoke_counts
   execute('empty',b'',0,b'');execute('repeat_two',raw[:256]*2,0,ref*2)
   sequence=execute('valid_sequence',raw[:512],0);inspect(raw[:512],sequence,cases[:2])
   execute('partial_first',raw[:128],7,b'');execute('complete_then_partial',raw[:256]+raw[:128],7,ref)
   for name in report['protocols']:execute(name,(input_dir/(name+'.bin')).read_bytes(),7,b'')
   status,out=saved_file_run(host/'carrier-x86.exe',input_dir/'input.bin',a.out,'corpus');assert status['exit']==0 and status['stderr_bytes']==0 and not status['timeout'],status
   native=out.read_bytes();projection,counts,details=inspect(raw,native,cases)
   for index,got in seen.items():assert native[index*5452:(index+1)*5452]==got
   assert native[:10904]==sequence;pp=a.out/'native-projection4896.bin';pp.write_bytes(projection);save(a.out/'inspection.json',dict(counts=counts,details=details))
   save(a.out/'report.json',dict(native_executed=True,records=300,snapshots=2700,raw_bytes_per_record=5452,defined_projection_bytes_per_record=4896,protocol_rejections=len(report['protocols']),framing_checks=5,retries=0,smokes=25,smoke_counts=smoke_counts,counts=counts,carrier_sha256=CARRIER,raw_output_sha256=sha(out),native_projection_sha256=sha(pp),rust_accepted=False,opaque_event_padding_preserved_raw=True,unused_event_capacity_nonsemantic=True,all440payload_bytes_projected=True,distinct_XMM_poison_loaded=True,fullraw_RunOnce_determinism=True))
 except BaseException as e:save(a.out/'failure.json',dict(error=repr(e),retained=True,retries=0,single_attempt=True));raise
if __name__=='__main__':main()
