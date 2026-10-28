"""Preflight/single-attempt native root stage gate; never executes by default.
Execution requires actual linked independent peer closure. Explicit caller
zero-child topology only; no actor or generated nonzero-child admission.
"""
import argparse,json,struct,hashlib
from pathlib import Path
from types import SimpleNamespace
import anim_root_zero_child_stage_audit_v2 as audit
from cell_runtime_data_native_run import digest,pinned,saved_file_run,word
ROOT=Path(__file__).resolve().parent.parent
CARRIER_SHA='9279eceacf329d898adc610b53dd19cd7cb91bf35b9908980fe620a1cb63ee67'
OWNED_SHA='6c90439efff9a470ee228146b3e662e6a8b31a2de8f0f40884347a286e6edb00'
INPUT_SHA='83ecfab5238cbd6f8c312441724b81ed40b185b1b9061ead3dc675952d4b7f20'
POSE_SHA='08c0cb20188bd10fce97e876e98069a6de2014f90b53a80ac98569ce662143c8'
CONTROLS=[0x7f|p|r for p in(0,0x200,0x300) for r in(0,0x400,0x800,0xc00)]
PREP=ROOT/'target/animation/root-zero-child-stage-native-v3'
POSE=ROOT/'target/animation/root-pose-sparse-carrier-native-run-v3'
PAIR=ROOT/'target/animation/root-pose-conversion-inputs-v1'
def save(p,x):
 with p.open('x') as f:json.dump(x,f,indent=2);f.write('\n')
def resolve(p):return Path(p) if Path(p).is_absolute() else ROOT/p
def bits(x):return struct.unpack('<I',struct.pack('<f',x))[0]
def setword(b,o,v):struct.pack_into('<I',b,o,v)
def check(raw):
 assert len(raw)==1256
 assert word(raw,0) in CONTROLS
 assert word(raw,4)==0 and word(raw,8)==1 and word(raw,1252)==0
 assert word(raw,20)<=1 and word(raw,28)==0 and raw[32:48]==bytes(16)
 finite=[12,16,24]+list(range(48,88,4))+list(range(92,192,4))+list(range(1000,1028,4))
 assert all(word(raw,x)&0x7f800000!=0x7f800000 for x in finite)
 assert word(raw,12)&0x80000000==0

def peer(path,required):
 if path is None:
  assert not required,'Actual linked independent peer report required before execution'
  return dict(admitted=False,reason='actual_linked_peer_pending')
 report=json.loads(path.read_text())
 assert report['no_blocking_static_issue_found'] is True and report['native_executed'] is False
 for name,value in report['pins'].items():assert digest(resolve(name))==value,name
 assert CARRIER_SHA in report['pins'].values() and OWNED_SHA in report['pins'].values()
 return dict(admitted=True,sha256=digest(path),path=str(path.resolve()))

def review(a):
 preparation=json.loads((PREP/'final-preparation-review.json').read_text())
 for name,value in preparation['pins'].items():assert digest(resolve(name))==value,name
 assert digest(a.host_dir/'stage-carrier-x86.exe')==CARRIER_SHA
 assert digest(a.host_dir/'host-x86.exe')==OWNED_SHA
 assert (a.host_dir/'host-x86.asm').read_bytes()==(PREP/'host-x86.asm').read_bytes()
 linked=json.loads((a.host_dir/'independent-linked-audit.json').read_text())
 actual=audit.audit(SimpleNamespace(carrier=a.host_dir/'stage-carrier-x86.exe',owned_exe=a.host_dir/'host-x86.exe',owned_map=a.host_dir/'host-x86.map',source_exe=a.source_exe))
 assert actual==linked and actual['static_pass'] and actual['source_bytes']==5901 and actual['source_pages']==8 and actual['virtual_gap_sections']==7
 manifest=json.loads((a.fixture/'manifest.json').read_text());assert manifest['rows']==66636 and manifest['accepted_pose_rows']==22212 and manifest['stage_contexts']==3 and not manifest['actor_body_association'] and not manifest['generated_nonzero_child_supported']
 for name,value in manifest['pins'].items():assert digest(resolve(name))==value,name
 for name,value in manifest['files'].items():assert digest(a.fixture/name)==value,name
 assert digest(a.fixture/'input.bin')==INPUT_SHA and digest(POSE/'native-v1/corpus-output.bin')==POSE_SHA
 contexts=json.loads((a.fixture/'stage-contexts.json').read_text());assert len(contexts)==3;ctx={x['name']:x for x in contexts}
 index=json.loads((PAIR/'input-index.json').read_text());assert len(index)==22212
 counts={};seen={};rows=0
 with (a.fixture/'input.bin').open('rb') as inputs,(a.fixture/'index.jsonl').open() as table,(POSE/'native-v1/corpus-output.bin').open('rb') as replies,(PAIR/'inputs.bin').open('rb') as pairs:
  for source_row,item in enumerate(index):
   reply=replies.read(176);old=pairs.read(144);assert len(reply)==176 and len(old)==144
   for context in contexts:
    raw=inputs.read(1256);check(raw);meta=json.loads(table.readline());assert meta['row']==rows and meta['accepted_pose_row']==source_row and meta['source_curve_row']==item['curve_row']
    assert meta['key']==item['key'] and meta['source_sha256']==item['source_sha256'] and meta['frame_bits']==item['frame_bits'] and meta['binding_pass']==item['pass_index'] and meta['pose_context']==item['context']
    assert meta['stage_context']==context['name'] and meta['actor_association'] is False and meta['body_topology']=='explicit_supplied_zero_child_root_only'
    assert meta['input_sha256']==hashlib.sha256(raw).hexdigest() and meta['pose_reply_sha256']==hashlib.sha256(reply).hexdigest()
    # Independent exact reconstruction of graph-free payload/context, not merely domain validation.
    expected=bytearray(1256);struct.pack_into('<8I',expected,0,word(old,0),0,1,bits(context['dt']),bits(context['smoothing']),context['skip'],word(old,20),0)
    expected[32+0x48:32+0xa0]=reply[28:116]
    for i in range(3):
     rest=word(old,48+i*4);setword(expected,732+0x10c+i*4,rest)
     initial=bits(struct.unpack('<f',struct.pack('<I',rest))[0]+context['initial_offset'][i]);setword(expected,32+0x10+i*4,initial);setword(expected,32+0x3c+i*4,initial)
    for i in range(4):setword(expected,732+0x118+i*4,word(old,64+i*4));setword(expected,32+0x1c+i*4,word(old,64+i*4))
    assert raw==expected and raw[32+0x48:32+0xa0]==reply[28:116]
    counts[context['name']]=counts.get(context['name'],0)+1;seen[source_row]=seen.get(source_row,0)+1;rows+=1
  assert inputs.read(1)==b'' and table.readline()=='' and replies.read(1)==b'' and pairs.read(1)==b''
 assert rows==66636 and len(seen)==22212 and set(seen.values())=={3} and set(counts.values())=={22212}
 protocols=json.loads((a.fixture/'protocol/report.json').read_text());assert len(protocols['cases'])==15
 for p in protocols['cases']:assert digest(a.fixture/'protocol'/(p['case']+'.bin'))==p['sha256']
 return manifest,dict(records=rows,source_pose_rows=22212,source_curve_pairs=3702,source_clips=215,contexts=counts,all_88B_native_accumulators_exact=True,whole_payload_graph_context_reconstruction_exact=True,actual_linked_audit_reproduced=True,peer=peer(a.peer_review,a.run),native_executed=False)

def inspect(requests,replies,projection=None):
 assert requests.stat().st_size%1256==0
 n=requests.stat().st_size//1256;assert replies.stat().st_size==n*736
 nonfinite=0;out=None if projection is None else projection.open('xb')
 try:
  with requests.open('rb') as inputs,replies.open('rb') as outputs:
   for row in range(n):
    raw=inputs.read(1256);reply=outputs.read(736);check(raw)
    assert struct.unpack_from('<H',reply,728)[0]==word(raw,0)
    assert struct.unpack_from('<H',reply,730)[0]&0x3800==0 and word(reply,732)&~0x3f==0x1f80
    assert reply[:4]==bytes(4) and reply[0x38]==1
    # Check every root byte outside the exact source write union (plus canonical pointer).
    mutable=set(range(0,4))|set(range(0x10,0x38))|{0x38}|set(range(0x3c,0x48))|set(range(0x58,0x7c))|set(range(0x190,0x1ac))|set(range(0x1b0,0x1e0))|set(range(0x200,0x218))
    assert all(reply[i]==raw[32+i] for i in range(700) if i not in mutable),(row,'root immutable bytes')
    assert reply[0x200:0x210]==reply[0x19c:0x1ac]
    for at in(0x1b0,0x1bc,0x1c8,0x1d4):assert reply[at:at+12]==reply[0x190:0x19c]
    assert reply[0x210:0x218]==bytes(8)
    if word(raw,20)==0:assert reply[0x10:0x2c]==reply[0x190:0x1ac]
    else:assert reply[0x10:0x38]==raw[32+0x10:32+0x38]
    if any(word(reply,i)&0x7f800000==0x7f800000 for i in range(0,728,4)):nonfinite+=1
    if out is not None:out.write(reply[:728])
 finally:
  if out is not None:out.close()
 return dict(records=n,nonfinite_projection_cases=nonfinite)

def execute(a,manifest,closure):
 out=a.out/'native-v1';out.mkdir(exist_ok=False);actualpeer=json.loads(a.peer_review.read_text())
 paths=[a.source_exe,Path(__file__),a.peer_review,a.out/'preflight.json',PREP/'final-preparation-review.json',a.fixture/'manifest.json',a.fixture/'protocol/report.json',ROOT/'tools/cell_runtime_data_native_run.py',ROOT/'tools/cell_schema_probe.py',ROOT/'tools/native_crt_acos_probe.py']
 paths += [resolve(p) for p in manifest['pins']]+[resolve(p) for p in actualpeer['pins']]+[resolve(p) for p in json.loads((PREP/'final-preparation-review.json').read_text())['pins']]
 paths += [p for p in a.host_dir.iterdir() if p.is_file()]+[p for p in a.fixture.rglob('*') if p.is_file()]
 paths=list(dict.fromkeys(p.resolve() for p in paths));host=(a.host_dir/'stage-carrier-x86.exe').resolve()
 with pinned(paths,out):
  protocol=out/'protocol';protocol.mkdir();first=(a.fixture/'input.bin').open('rb').read(1256);reference=protocol/'reference-input.bin';reference.write_bytes(first)
  status,output=saved_file_run(host,reference,protocol,'reference')
  assert status['exit']==0 and status['stderr_bytes']==0,status
  observed=inspect(reference,output);expected=output.read_bytes()
  if a.reference_only:
   save(out/'reference-only-report.json',dict(native_executed=True,records=1,reference_only=True,retries=0,host_sha256=CARRIER_SHA,output_sha256=digest(output),observed=observed));return
  cases=[('empty',b'',0,b'')]
  for n in(1,32,1255):cases += [(f'partial{n}',first[:n],71,b''),(f'complete_then_partial{n}',first+first[:n],71,expected)]
  for case in json.loads((a.fixture/'protocol/report.json').read_text())['cases']:
   cases.append((case['case'],(a.fixture/'protocol'/(case['case']+'.bin')).read_bytes(),71,b''))
  cases.append(('repeat_two',first+first,0,expected+expected));results=[]
  for name,raw,code,wanted in cases:
   request=protocol/(name+'-input.bin');request.write_bytes(raw);status,reply=saved_file_run(host,request,protocol,name)
   assert status['exit']==code and status['stderr_bytes']==0,(name,status)
   assert reply.read_bytes()==wanted,name
   results.append(dict(case=name,expected_exit=code,exact_reply=True,input_sha256=digest(request),output_sha256=digest(reply)))
  for cw in CONTROLS:
   name=f'policy_{cw:04X}';raw=bytearray(first);setword(raw,0,cw);request=protocol/(name+'-input.bin');request.write_bytes(raw);status,reply=saved_file_run(host,request,protocol,name)
   assert status['exit']==0 and status['stderr_bytes']==0,(name,status)
   inspect(request,reply)
   # New x87 norm may differ by PC/RC. Preserve each original policy reply;
   # root Rust verifier must model it, never assume one reference fits all CWs.
   results.append(dict(case=name,expected_exit=0,metadata_and_root_invariants=True,input_sha256=digest(request),output_sha256=digest(reply),rust_policy_comparison_pending=True))
  save(protocol/'report.json',dict(checks=len(results),single_attempt=True,results=results,policy_projection_equality_assumed=False))
  status,output=saved_file_run(host,a.fixture/'input.bin',out,'corpus',timeout=180)
  assert status['exit']==0 and status['stderr_bytes']==0,status
  observed=inspect(a.fixture/'input.bin',output,out/'native-projection728.bin');assert observed['records']==66636
  save(out/'report.json',dict(schema='zero-child-full-entry-stage-native-run-v1',native_executed=True,retries=0,records=66636,source_pose_rows=22212,source_curve_pairs=3702,source_clips=215,explicit_stage_contexts=3,protocol_checks=len(results),host_sha256=CARRIER_SHA,input_sha256=INPUT_SHA,output_sha256=digest(output),projection_sha256=digest(out/'native-projection728.bin'),observed=observed,scope='Actual accepted88Bsourceposeaccumulators through original rootstage/zerochildhelper/finalcommit under three explicitly supplied contexts; no generatednonzerochildren/actor/whole9F4730/visibleanimation claim.'))

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--peer-review',type=Path);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--run',action='store_true');p.add_argument('--reference-only',action='store_true');a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 manifest,closure=review(a);a.out.mkdir(parents=True,exist_ok=False);save(a.out/'preflight.json',dict(schema='zero-child-stage-preflight-v1',runner_sha256=digest(__file__),closure=closure,native_executed=False,execution_requested=a.run))
 if a.run:execute(a,manifest,closure)
 else:print(json.dumps(dict(preflight=True,records=closure['records'],peer=closure['peer'],native_executed=False)))
if __name__=='__main__':main()
