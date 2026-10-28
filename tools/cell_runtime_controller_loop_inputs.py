"""Prepare bounded183B fullsetter requests; no native execution or expected replies."""
import argparse,json,struct,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def request(primary=0,secondary=1,mode=1,topology=0,match=True,handle_zero=False,unread_bad_link=False,secondary_match=True):
 epoch=0x2345;sepoch=0x5432;handle=0 if handle_zero else(epoch<<8)|(primary+1);link=((sepoch<<8)|(secondary+1))if topology in[1,3]else 0
 if unread_bad_link:link=0xffff0000
 pa=int(topology in[0,3]);sa=int(topology in[1,3]);raw=struct.pack('<16I',handle,primary,epoch if match else epoch+1,secondary,sepoch if secondary_match else sepoch+1,link,mode,pa,sa,0xa3,0xb4,0xc5,0xd6,int(mode==1),0,0);assert len(raw)==64;return raw
def validate(raw):
 assert len(raw)==64;q=struct.unpack('<16I',raw);assert q[1]<16 and q[3]<16 and q[1]!=q[3]and q[7]in[0,1]and q[8]in[0,1]and all(v<=255 for v in q[9:13])and q[13]==int(q[6]==1)and q[14:]==(0,0)
 if q[0]:
  assert q[0]&255==q[1]+1
  if q[0]>>8==q[2]and q[5]:assert q[5]&255==q[3]+1
 return True
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());m=json.loads((a.host_dir/'manifest.json').read_text());assert sha(a.host_dir/'host-x86.asm')==m['host_source_sha256'];a.out.mkdir(parents=True,exist_ok=False);cases=[]
 with(a.out/'input.bin').open('xb')as f:
  def emit(raw,family):validate(raw);f.write(raw);cases.append(dict(case=len(cases),family=family,words=[f'{v:08X}'for v in struct.unpack('<16I',raw)]))
  for i in range(16):
   for mode in[0,1,2,0xffffffff]:
    for topology in range(4):
     for match in[False,True]:emit(request(i,(i+1)%16,mode,topology,match),'direct_linked_epoch')
  for mode in[0,1,2,0xffffffff]:
   for topology in range(4):emit(request(mode=mode,topology=topology,handle_zero=True),'null_handle')
   emit(request(mode=mode,match=False,unread_bad_link=True),'unread_invalid_link')
   emit(request(mode=mode,topology=1,secondary_match=False),'secondary_epoch_mismatch')
 assert len(cases)==536;(a.out/'cases.json').write_text(json.dumps(cases,indent=2)+'\n');protocols={};valid=request()
 errors={'primary_index':(4,16),'secondary_index':(12,16),'aliased_slots':(12,0),'primary_handle_low_zero':(0,0x123400),'primary_handle_wrong_index':(0,0x234503),'primary_aid_bool':(28,2),'secondary_aid_bool':(32,2),'initial_loop_byte':(36,256),'policy_byte':(52,0),'reserved':(56,1)}
 for name,(at,value)in errors.items():r=bytearray(valid);struct.pack_into('<I',r,at,value);path=a.out/(name+'.bin');path.write_bytes(r);protocols[name]=sha(path)
 r=bytearray(request(topology=1));struct.pack_into('<I',r,20,0x5432ff);path=a.out/'accessed_link_index.bin';path.write_bytes(r);protocols['accessed_link_index']=sha(path)
 paths=[Path(__file__),ROOT/'tools/cell_runtime_controller_loop_native_probe.py',ROOT/'tools/cell_runtime_clip_metadata_native_probe.py',ROOT/'tools/cell_runtime_clock_native_probe.py',ROOT/'crates/spore-core/src/cell_runtime_controller_loop.rs',ROOT/'crates/spore-core/examples/cell_runtime_controller_loop_verify.rs.pending',a.host_dir/'manifest.json',a.host_dir/'host-x86.asm']
 paths +=[ROOT/f'target/rea/20261027-source-runtime-v4/{n}-analyze_function.json'for n in[53,54,55]]
 (a.out/'report.json').write_text(json.dumps(dict(schema='source-cell-controller-loop-inputs-v1',cases=536,protocols=protocols,input_sha256=sha(a.out/'input.bin'),cases_sha256=sha(a.out/'cases.json'),pins={str(p):sha(p)for p in paths},scope='Full183Bsetter/resolver; two disjointreachedslots/aids,propertysourceword1vsother,primary/secondaryepochmatch,handle0,unreadinvalidlink. Noactualcontrollerconstruction/counter/eventclock admission.',no_execution=True),indent=2)+'\n');print(536)
if __name__=='__main__':main()
