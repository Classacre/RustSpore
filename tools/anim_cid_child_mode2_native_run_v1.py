"""Mode-2 native runner; default static preflight, independent peers required to run."""
import argparse,hashlib,json,struct
from pathlib import Path
import anim_cid_child_mode2_native_prepare_v1 as prep
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
from cell_schema_probe import SourceImage
from cell_runtime_data_native_run import pinned,saved_file_run
ROOT=prep.ROOT
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,v):
 with p.open('x')as f:json.dump(v,f,indent=2);f.write('\n')
def verify_peer(p,required):
 v=json.loads(p.read_text());assert v['no_blocking_static_issue_found'] and not v['native_executed']
 for item in required:assert sha(item) in v['pins'].values(),str(item)
 for name,value in v['pins'].items():assert sha(Path(name))==value,name
 return v
def static(host,carrier):
 image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));fx=prep.fixture_bytes();audit=prep.proof(image);asm=host/'host-x86.asm';exe=host/'host-x86.exe';mapping=host/'host-x86.map'
 assert asm.read_text()==prep.host(image,fx);m=json.loads((host/'manifest.json').read_text());assert m['asm_sha256']==sha(asm) and m['tool_sha256']==sha(prep.__file__)
 rebuilt,_=prep.build(exe,mapping,image);assert rebuilt==carrier.read_bytes();old=Image(exe);new=Image(carrier);symbols=syms(mapping);assert old.base==0x29000000 and new.base==prep.BASE
 ni,_=new.imports();oi,_=old.imports();assert ni==oi and set(ni.values())=={'GetStdHandle','ReadFile','WriteFile','ExitProcess','GetModuleHandleA','GetLastError'}
 assert all(not(s['flags']&0x20000000 and s['flags']&0x80000000)for s in new.sections)
 for address,data,flags in prep.pages(image,audit,symbols['OriginalDone']):
  skip=4096 if address==prep.BASE else 0;assert new.read(address+skip,len(data)-skip)==data[skip:] and new.perm(address+skip,len(data)-skip)==(flags&0xe0000000)
 assert new.read(symbols['Fixtures'],len(fx))==fx and new.perm(symbols['Fixtures'],len(fx))==0x40000000
 for name,size in prep.SIZES.items():
  at=symbols[name+'Buffer'];assert symbols[name+'Before']==at-16 and symbols[name+'After']==at+size and new.perm(at-16,size+32)==0xc0000000 and new.perm(symbols[name+'Shadow'],size)==0xc0000000
 first,last=symbols['OwnedStart'],symbols['OwnedEnd'];assert new.read(first,last-first)==old.read(first,last-first)
 return dict(static_pass=True,actual_linked=True,native_executed=False,carrier_sha256=sha(carrier),owned_sha256=sha(exe),map_sha256=sha(mapping),source_pages_exact=True,imports_closed=True,no_WX=True,limits='Reproducibility and basic PE checks; independent control flow, ABI, byte guards, stack, runner review required.')
def inspect(raw,n):
 assert len(raw)==n*60
 for i in range(n):
  at=i*60;assert struct.unpack_from('<3I',raw,at)==(1,1,1);assert raw[at+40:at+48]==bytes.fromhex('0400ffff0400ffff');assert struct.unpack_from('<I',raw,at+48)[0]==0xbf800000
  cw,sw,csr=struct.unpack_from('<HHI',raw,at+52);assert cw==0x27f and sw&0x3800==0 and csr&~0x3f==0x1f80
 return raw
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--carrier',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--linked-peer',type=Path,required=True);p.add_argument('--plan',type=Path);p.add_argument('--runner-peer',type=Path);p.add_argument('--run',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 checks=static(a.host_dir,a.carrier);owned=a.host_dir/'host-x86.exe';mapping=a.host_dir/'host-x86.map';files=[Path(__file__),Path(prep.__file__),ROOT/'tools/anim_root_pose_sparse_carrier_v3.py',ROOT/'tools/anim_root_pose_sparse_carrier_audit_v2.py',ROOT/'tools/cell_runtime_data_native_run.py',a.host_dir/'host-x86.asm',a.host_dir/'manifest.json',owned,mapping,a.carrier,prep.FIX/'fixtures.json',prep.FIX/'manifest.json',prep.AUDIT,prep.CONTRACT,a.linked_peer];verify_peer(a.linked_peer,[owned,mapping,a.carrier,Path(prep.__file__)])
 if a.plan:
  plan=json.loads(a.plan.read_text());assert plan['runner_sha256']==sha(__file__)
  for name,value in plan['pins'].items():assert sha(Path(name))==value,name
  files.append(a.plan)
 if a.runner_peer:
  assert a.plan;verify_peer(a.runner_peer,[Path(__file__),a.plan,owned,mapping,a.carrier]);files.append(a.runner_peer)
 if a.run:assert a.plan and a.runner_peer,'Frozen plan and independent linked and exact-runner reviews required'
 a.out.mkdir(parents=True,exist_ok=False)
 if not a.plan:save(a.out/'plan.json',dict(schema='cid-child-mode2-runner-plan-v1',runner_sha256=sha(__file__),native_executed=False,pins={str(f.resolve()):sha(f)for f in dict.fromkeys(files)}))
 save(a.out/'preflight.json',dict(**checks,execution_requested=a.run,synthetic_stage=True,fixture_count=64,independent_linked_peer=str(a.linked_peer)))
 if not a.run:print('Static preflight PASS; native not executed');return
 try:
  with pinned(list(dict.fromkeys(f.resolve() for f in files)),a.out):
   protocol=a.out/'protocol';protocol.mkdir()
   def execute(label,data,code,expected=None):
    request=protocol/(label+'-input.bin');request.write_bytes(data);pins=protocol/(label+'-pins');pins.mkdir()
    with pinned([request],pins):status,out=saved_file_run(a.carrier,request,protocol,label)
    assert status['exit']==code and not status['timeout'] and status['stderr_bytes']==0,(label,status)
    got=out.read_bytes()
    if expected is not None:assert got==expected,label
    return got
   one=execute('single',struct.pack('<I',0),0);inspect(one,1);execute('empty',b'',0,b'');execute('repeat',bytes(8),0,one+one);execute('partial',bytes(3),71,b'');execute('out_of_range',struct.pack('<I',64),71,b'');execute('complete_then_partial',bytes(7),71,one)
   data=b''.join(struct.pack('<I',i)for i in range(64));raw=execute('corpus',data,0);inspect(raw,64);assert raw[:60]==one
   output=a.out/'native-output.bin';output.write_bytes(raw);save(a.out/'report.json',dict(native_executed=True,fixtures=64,synthetic_stage=True,native_sha256=sha(output),rust_comparison=False,actual_actor_association=False,scope='mode-2 branch only; no live animation or full traversal'))
 except BaseException as e:save(a.out/'failure.json',dict(error=repr(e),retained=True,retries=0));raise
if __name__=='__main__':main()
