"""Reviewed small source hosts: default static preflight; --run attempts each protocol/corpus once."""
import argparse,importlib,json,struct
from pathlib import Path
from cell_runtime_data_native_run import digest,pinned,saved_file_run
ROOT=Path(__file__).resolve().parent.parent
CONFIG={'controller-loop':(536,36,28,'978d3c657688d5114abb6dbd83a4f892e666ea55e9ce7a458edc9c7ec0ede940'),'fresh-aid':(288,64,56,'aa5c95a8e25dc8a1a5ac19e690281ed5daebe29b038ad3106ef025b2026a1b6c')}
def save(p,x):
 with p.open('x') as f:json.dump(x,f,indent=2);f.write('\n')
def inspect(raw,reply,kind):
 count,size,proj,host=CONFIG[kind];assert len(raw)%64==0 and len(reply)==len(raw)//64*size
 out=bytearray()
 for i in range(len(raw)//64):
  q=raw[i*64:(i+1)*64];r=reply[i*size:(i+1)*size];cw,sw,mx=struct.unpack_from('<HHI',r,proj)
  assert cw==(0x27f if kind=='controller-loop' else struct.unpack_from('<H',q,4)[0]) and sw&0x3800==0 and mx==0x1f80
  out+=r[:proj]
 return out

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('kind',choices=CONFIG);ap.add_argument('--out',type=Path,required=True);ap.add_argument('--run',action='store_true');a=ap.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 count,size,proj,expected=CONFIG[a.kind];host=ROOT/('target/source-cell/native-runtime-'+a.kind+'-host-root-reproduction-v1');inputs=ROOT/('target/source-cell/runtime-'+a.kind+'-inputs-v1');peerpath=ROOT/('target/source-cell/runtime-'+a.kind+'-independent-font-review-v1.json');peer=json.loads(peerpath.read_text());prepared=json.loads((inputs/'report.json').read_text());probe=importlib.import_module('cell_runtime_'+a.kind.replace('-','_')+'_native_probe')
 assert peer['no_blocking_static_issue_found'] is True and peer['native_executed'] is False and expected in peer['pins'].values()
 paths=[Path(__file__),peerpath,ROOT/'tools/cell_runtime_data_native_run.py']
 for name,h in peer['pins'].items():
  p=ROOT/name;assert digest(p)==h,name;paths.append(p)
 image=probe.SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));assert probe.linked_audit(host/'host-x86.exe',host/'host-x86.map',image)==json.loads((host/'linked-audit.json').read_text());assert probe.host_source(image)==(host/'host-x86.asm').read_text()
 assert digest(host/'host-x86.exe')==expected and prepared['cases']==count and digest(inputs/'input.bin')==prepared['input_sha256'];raw=(inputs/'input.bin').read_bytes();assert len(raw)==count*64
 for name,h in prepared['protocols'].items():
  p=inputs/(name+'.bin');assert digest(p)==h;paths.append(p)
 a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='small-source-host-preflight-v1',kind=a.kind,cases=count,linked_audit_reproduced=True,peer_admitted=True,native_executed=False,execution_requested=a.run))
 if not a.run:print('Preflight PASS; native not executed');return
 try:
  with pinned(list(dict.fromkeys(p.resolve() for p in paths)),a.out):
   exe=host/'host-x86.exe';protocol=a.out/'protocol';protocol.mkdir();ref=protocol/'reference-input.bin';ref.write_bytes(raw[:64]);status,output=saved_file_run(exe,ref,protocol,'reference');assert status['exit']==0 and status['stderr_bytes']==0,status;inspect(ref.read_bytes(),output.read_bytes(),a.kind);wanted=output.read_bytes()
   checks=[('empty',b'',0,b''),('repeat_two',raw[:64]*2,0,wanted*2),('complete_then_partial',raw[:64]+raw[:32],7,wanted)]+[(n,(inputs/(n+'.bin')).read_bytes(),7,b'') for n in prepared['protocols']]
   for name,q,code,w in checks:
    request=protocol/(name+'-input.bin');request.write_bytes(q);status,output=saved_file_run(exe,request,protocol,name);assert status['exit']==code and status['stderr_bytes']==0,(name,status);assert output.read_bytes()==w,name
   status,output=saved_file_run(exe,inputs/'input.bin',a.out,'corpus');assert status['exit']==0 and status['stderr_bytes']==0,status;projection=inspect(raw,output.read_bytes(),a.kind);pp=a.out/('native-projection'+str(proj)+'.bin');pp.write_bytes(projection)
   save(a.out/'report.json',dict(schema='small-source-host-native-v1',kind=a.kind,native_executed=True,cases=count,protocols=len(checks),retries=0,host_sha256=expected,raw_output_sha256=digest(output),projection_sha256=digest(pp),rust_accepted=False,numeric_expected_from_rust=False))
 except BaseException as e:
  save(a.out/'failure.json',dict(error=repr(e),retained=True,retries=0));raise
if __name__=='__main__':main()
