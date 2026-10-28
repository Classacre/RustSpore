"""Join accepted source-pose accumulators to explicit zero-child stage contexts.
No native execution and no source-actor/body association is inferred.
"""
import argparse,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
POSE=ROOT/'target/animation/root-pose-sparse-carrier-native-run-v3'
FIXTURE=ROOT/'target/animation/root-pose-conversion-inputs-v1'
NATIVE_SHA='08c0cb20188bd10fce97e876e98069a6de2014f90b53a80ac98569ce662143c8'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def word(b,at):return struct.unpack_from('<I',b,at)[0]
def setword(b,at,x):struct.pack_into('<I',b,at,x)
def bits(x):return struct.unpack('<I',struct.pack('<f',x))[0]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--host-dir',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 native=POSE/'native-v1/corpus-output.bin';assert sha(native)==NATIVE_SHA
 report=json.loads((POSE/'native-v1/report.json').read_text());assert report['records']==22212 and report['observed']['nonfinite_projection_cases']==0
 comparison=json.loads((POSE/'rust-comparison-v1.json').read_text());assert comparison # root accepted full native/Rust equality; retained artifact pinned
 source_index=json.loads((FIXTURE/'input-index.json').read_text());assert len(source_index)==22212
 contexts=[dict(name='explicit_zero_dt_no_history',dt=0.,smoothing=0.,skip=0,initial_offset=[0.,0.,0.]),dict(name='explicit_positive_dt_quarter_history',dt=1./60.,smoothing=.25,skip=0,initial_offset=[.125,-.25,.5]),dict(name='explicit_skip_commit_threequarter_history',dt=.5,smoothing=.75,skip=1,initial_offset=[-.5,.25,-.125])]
 a.out.mkdir(parents=True,exist_ok=False);rows=0
 with native.open('rb') as replies,(FIXTURE/'inputs.bin').open('rb') as original,(a.out/'input.bin').open('xb') as output,(a.out/'index.jsonl').open('x') as table:
  for source_row,item in enumerate(source_index):
   reply=replies.read(176);previous=original.read(144);assert len(reply)==176 and len(previous)==144
   for context in contexts:
    raw=bytearray(1256);struct.pack_into('<8I',raw,0,word(previous,0),0,1,bits(context['dt']),bits(context['smoothing']),context['skip'],word(previous,20),0)
    root=memoryview(raw)[32:732];definition=memoryview(raw)[732:]
    # All 88 accumulator bytes are genuine accepted original output, never replaced.
    root[0x48:0xa0]=reply[28:116]
    for i in range(3):
     rest=word(previous,48+i*4);setword(definition,0x10c+i*4,rest)
     initial=bits(struct.unpack('<f',struct.pack('<I',rest))[0]+context['initial_offset'][i]);setword(root,0x10+i*4,initial);setword(root,0x3c+i*4,initial)
    for i in range(4):setword(definition,0x118+i*4,word(previous,64+i*4));setword(root,0x1c+i*4,word(previous,64+i*4))
    # Child count stays its declared supplied zero; never converted from a generated body.
    assert word(definition,0x208)==0 and bytes(root[0:16])==bytes(16)
    output.write(raw);table.write(json.dumps(dict(row=rows,accepted_pose_row=source_row,source_curve_row=item['curve_row'],key=item['key'],source_sha256=item['source_sha256'],frame_bits=item['frame_bits'],binding_pass=item['pass_index'] if 'pass_index' in item else item.get('pass'),pose_context=item['context'],stage_context=context['name'],pose_reply_sha256=hashlib.sha256(reply).hexdigest(),input_sha256=hashlib.sha256(raw).hexdigest(),actor_association=False,body_topology='explicit_supplied_zero_child_root_only'))+'\n');rows+=1
  assert not replies.read(1) and not original.read(1)
 (a.out/'stage-contexts.json').write_text(json.dumps(contexts,indent=2)+'\n')
 pins=[native,POSE/'native-v1/report.json',POSE/'rust-comparison-v1.json',FIXTURE/'inputs.bin',FIXTURE/'input-index.json',FIXTURE/'manifest.json',a.host_dir/'manifest.json',a.host_dir/'host-x86.asm',Path(__file__),ROOT/'crates/spore-core/src/anim_root_zero_child_stage.rs',ROOT/'crates/spore-core/examples/anim_root_zero_child_stage_verify.rs.pending']
 manifest=dict(schema='zero-child-stage-accepted-pose-inputs-v1',rows=rows,accepted_pose_rows=22212,source_curve_pairs=3702,source_clips=215,stage_contexts=3,original_accumulator_bytes_per_record=88,all_accepted_pose_rows_retained=True,actor_body_association=False,generated_nonzero_child_supported=False,actual_definition_208=0,pose_count=1,pins={str(p.resolve()):sha(p) for p in pins},files={p.name:sha(p) for p in a.out.iterdir() if p.is_file()},native_executed=False,scope='Accepted original curve->coordinate/accumulator projection plus three explicitly supplied finite root-stage contexts. Zerochild is an explicit caller topology, not a source/generated body census claim.')
 (a.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(dict(rows=rows,input_sha256=sha(a.out/'input.bin'))))
if __name__=='__main__':main()
