"""Endpoint successor carrier build/audit and reviewed execution; no implicit compilation."""
import argparse,hashlib,json,struct
from pathlib import Path
from types import SimpleNamespace
import cell_runtime_fresh_sampled_channel_endpoint_prepare_v1 as prep
import cell_runtime_fresh_sampled_channel_native_probe as base
import cell_runtime_fresh_sampled_channel_native_probe_v3 as v3
import cell_runtime_fresh_sampled_channel_sparse_carrier as sparse
sparse.mechanical.probe=v3
import cell_runtime_fresh_sampled_channel_hybrid_audit as hybrid
import cell_runtime_fresh_sampled_channel_native_run_v1 as runner
from cell_runtime_data_native_run import pinned,saved_file_run
# Successor extends the source-data allowlist before generating or auditing.
import cell_runtime_fresh_sampled_channel_native_probe_v2 as v2
from anim_root_pose_sparse_carrier_audit_v2 import Image
EXTRA=(0x014477F8,4)
for module in (base.source,base,v2,v3):
 module.DATA=sorted(set(module.DATA+[EXTRA]))
ROOT=prep.ROOT;P=ROOT/'target/source-cell/runtime-fresh-sampled-channel-endpoint-prep-v2';H=P/'host';I=P/'inputs' 
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x):
 with p.open('x') as f:json.dump(x,f,indent=2);f.write('\n')
def setup():
 original=base.source_cases(); raw=(I/'clips'/f'{prep.S}.anim').read_bytes();u=lambda a:struct.unpack_from('<I',raw,a)[0]; channels=[];table=u(328)
 for i in range(u(324)):
  at=u(table+4*i);channels.append(dict(at=at,keys=u(at+216),descriptors=u(at+224),count=u(at+212)))
 c=channels[0];mask=bytearray(444)
 for i in range(u(c['at']+220)):
  d=c['descriptors']+32*i; kind=u(d)&15;slot=u(d+16);start=4+[0,20,100,200,240,280,320,360,400][slot];n=[20,80,100,40][kind];mask[start:start+n]=bytes([1])*n
 candidate=dict(path=I/'clips'/f'{prep.S}.anim',raw=raw,channels=channels,table=table,flags=u(280),interval=u(284),frames=u(288),mask=bytes(mask));cases=[candidate]+original[1:];base.source_cases=lambda:cases
 return cases
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('mode',choices=['carrier','audit','run']);p.add_argument('--review',type=Path);p.add_argument('--prebuild-review',type=Path);p.add_argument('--out',type=Path);a=p.parse_args();cases=setup();report=json.loads((P/'preparation.json').read_text());
 for path,expected in report['pins'].items():assert sha(path)==expected,path
 images=base.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll')); assert (H/'host-x86.asm').read_text()==v3.host_source(images)
 exe=H/'host-x86.exe'; mapping=H/'host-x86.map'; carrier=H/'carrier-x86.exe'
 if a.mode=='carrier':
  assert a.prebuild_review
  peer=json.loads(a.prebuild_review.read_text()); assert peer['static_pass'] and peer['native_executed'] is False
  required_pre=[Path(__file__),P/'preparation.json',H/'host-x86.asm',I/'input.bin',I/'cases.json',I/'cases.tsv',I/'clips'/f'{prep.S}.anim',P/'endpoint_verify_v1.rs.pending']
  assert all(str(x.resolve()) in peer['pins'] and sha(x)==peer['pins'][str(x.resolve())] for x in required_pre)
  assert all(sha(path)==value for path,value in peer['pins'].items())
  assert not carrier.exists();proof=base.audit_source(images);owned=v3.linked_audit(exe,mapping,images); data,info=sparse.mechanical.build(exe,mapping,images,proof);carrier.write_bytes(data);save(H/'endpoint-carrier-build.json',dict(schema='endpoint-carrier-build-v1',carrier_sha256=sha(carrier),owned_audit=owned,mechanical=info,native_executed=False));print(sha(carrier));return
 if a.mode=='audit':
  hybrid.source=v3;result=hybrid.audit(SimpleNamespace(carrier=carrier,owned_exe=exe,owned_map=mapping,source_exe=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),source_crt=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll')));actual=Image(carrier); slots=[];sizes=[]
  for i in range(9):
   address=0x014477D8+32*i; value=struct.unpack('<I',images.read(address,4))[0]; assert actual.read(address,4)==images.read(address,4); slots.append(value)
  for i in range(4):
   address=0x015514CC+4*i; value=struct.unpack('<I',images.read(address,4))[0]; assert actual.read(address,4)==images.read(address,4); sizes.append(value)
  assert slots==[0,20,100,200,240,280,320,360,400] and sizes==[20,80,100,40]
  result['source_slot_offsets']=slots;result['source_kind_lengths']=sizes;result['carrier_tables_equal_source']=True
  save(H/'endpoint-linked-audit.json',result);print(sha(H/'endpoint-linked-audit.json'));return
 assert a.review and a.out;out=a.out.resolve();out.relative_to((ROOT/'target/source-cell').resolve()); peer=json.loads(a.review.read_text());assert peer['no_blocking_static_issue_found'] and peer['native_executed'] is False
 required=[Path(__file__),Path(prep.__file__),H/'host-x86.asm',exe,mapping,carrier,H/'endpoint-linked-audit.json',P/'preparation.json',I/'input.bin',I/'cases.json',I/'cases.tsv',I/'clips'/f'{prep.S}.anim',P/'endpoint_verify_v1.rs.pending']; pins=peer['pins'];assert all(str(x.resolve()) in pins and sha(x)==pins[str(x.resolve())] for x in required);assert all(sha(path)==value for path,value in pins.items())
 linked=json.loads((H/'endpoint-linked-audit.json').read_text());assert linked['static_pass'] and linked['carrier_sha256']==sha(carrier) and linked['carrier_tables_equal_source'] and linked['source_slot_offsets']==[0,20,100,200,240,280,320,360,400] and linked['source_kind_lengths']==[20,80,100,40]
 raw=(I/'input.bin').read_bytes();cases_json=json.loads((I/'cases.json').read_text());assert len(raw)==12*256 and len(cases_json)==12
 out.mkdir(parents=True,exist_ok=False)
 with pinned(list(dict.fromkeys(required+[a.review.resolve()]+[Path(path) for path in report['pins']]+[Path(path) for path in pins])),out):
  proto=out/'protocol';proto.mkdir()
  def run(label,data,exit_code,expected=None):
   request=proto/(label+'-input.bin');request.write_bytes(data)
   pin_dir=proto/(label+'-pins');pin_dir.mkdir()
   with pinned([request],pin_dir):status,path=saved_file_run(carrier,request,proto,label)
   assert status['exit']==exit_code and not status['timeout'] and status['stderr_bytes']==0
   b=path.read_bytes();assert expected is None or b==expected;return b
  first=run('first',raw[:256],0);run('empty',b'',0,b'');run('repeat',raw[:256]*2,0,first*2);run('partial',raw[:128],7,b'');run('complete_partial',raw[:256]+raw[:128],7,first)
  for name,offset,value in [('capacity',16,1),('lookup_mask',44,0),('sample_mask',52,0),('source_case',72,5),('rotation',80,0),('reserved',84,1),('channel_count',36,9)]:
   bad=bytearray(raw[:256]);struct.pack_into('<I',bad,offset,value);run(name,bad,7,b'')
  status,path=saved_file_run(carrier,I/'input.bin',out,'corpus');assert status['exit']==0 and not status['timeout'] and status['stderr_bytes']==0
  native=path.read_bytes();projection,counts,details=runner.inspect(raw,native,cases_json);assert native[:5452]==first and counts['snapshots']==108 and counts['endpoint_fullslots']==48 and counts['interpolation']==60 and counts['authored_events']==0
  for d in details:
   frame=struct.unpack('<f',bytes.fromhex(d['frame_bits'])[::-1])[0]
   if d['snapshot'] in (1,3,5,7):assert 197<frame<217 and d['left']==d['right']==20
  (out/'native-projection4896.bin').write_bytes(projection);save(out/'inspection.json',dict(counts=counts,details=details));save(out/'report.json',dict(native_executed=True,records=12,snapshots=108,counts=counts,endpoint_full_slots_checked=True,first_channel_only=True,raw_sha256=sha(path),projection_sha256=sha(out/'native-projection4896.bin'),framing_checks=5,protocol_rejections=7,protocol_invocations=12,retries=0,rust_accepted=False))
if __name__=='__main__':main()
