"""Prepare actual finalized-body/nonroot selector source gate inputs/reference.
No original code execution. Retains every1644sourcechannel and all3Z exclusions.
"""
import argparse,json,hashlib,struct,math,collections
from pathlib import Path
from anim_body_finalizer_native_probe import FIELDS,RECORD_BYTES,PROJECTION_BYTES
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder,PE
ROOT=Path(__file__).resolve().parent.parent
FINAL=ROOT/'target/animation/body-finalizer-fields-native-v3-run'
FINPUT=ROOT/'target/animation/body-finalizer-fields-inputs-v2'
WHOLE=ROOT/'target/animation/root-whole-builder-followup-v1'
BODY_SHA='e09a5b85cc07bfbe51cd97179b82f2b4630ac746f7af3190a4857d3e7b62367b'
INDEX={at:168+4*i for i,at in enumerate(FIELDS)}
QUERIES=[('20261022-anim-rig-focused-v1',43,0x9b2150),('20261022-anim-rig-focused-v1',44,0x9b2e60),('20261022-anim-rig-focused-v1',46,0x9b0090),('20261022-anim-rig-focused-v1',42,0x9b1b90),('20261027-source-runtime-v4',61,0x9b0130)]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def word(b,o):return struct.unpack_from('<I',b,o)[0]
def f(x):return struct.unpack('<f',struct.pack('<f',x))[0]
def number(b,o):return struct.unpack_from('<f',b,o)[0]
def mirror(flags):
 x=flags&0x2000c;x={4:8,8:4,0x20000:0x20004,0x20004:0x20000}.get(x,x);rank=flags&0xe000;rank={0x2000:0x4000,0x4000:0x2000}.get(rank,rank);return(flags&~0x2e00c)|x|rank
def select(blocks,s,mirrored):
 flags=mirror(s[0])if mirrored else s[0];x=flags&0x2000c;y=flags&0x40030;rank=flags&0xe000;hier=flags&0x801c00
 eligible=[]
 for i,b in enumerate(blocks):
  for slot in range(32):
   cap=word(b,8+slot*4)
   if cap==0:break
   if cap==s[1]:
    if b[136+slot]:eligible.append(i)
    break
 if not eligible:return[]
 center=lambda i,axis:number(blocks[i],INDEX[0x10c+axis*4])
 low=[min(center(i,a)for i in eligible)for a in range(2)];high=[max(center(i,a)for i in eligible)for a in range(2)]
 add=lambda a,b:f(a+b);sub=lambda a,b:f(a-b);mul=lambda a,b:f(a*b);floor=lambda a:math.floor(mul(a,100.));p525=f(.525)
 out=[]
 for i in eligible:
  b=blocks[i];px,py=center(i,0),center(i,1);tol=mul(add(add(number(b,INDEX[0x140]),number(b,INDEX[0x13c])),number(b,INDEX[0x138])),f(.15))
  rejectx={0:lambda:False,4:lambda:px< -tol,8:lambda:tol<px,12:lambda:abs(px)>tol,0x20000:lambda:floor(px)<floor(sub(high[0],mul(sub(high[0],low[0]),p525))),0x20004:lambda:floor(add(mul(sub(high[0],low[0]),p525),low[0]))<floor(px),0x20008:lambda:abs(floor(sub(px,add(mul(sub(high[0],low[0]),.5),low[0]))))>10}[x]()
  rejecty={0:lambda:False,0x10:lambda:py>0.,0x20:lambda:py<0.,0x30:lambda:abs(py)>tol,0x40000:lambda:floor(py)>floor(add(mul(sub(high[1],low[1]),p525),low[1])),0x40010:lambda:floor(py)<floor(sub(high[1],mul(sub(high[1],low[1]),p525))),0x40020:lambda:abs(floor(sub(py,add(mul(sub(high[1],low[1]),.5),low[1]))))>10}[y]()
  if rejectx or rejecty:continue
  if not out or not rank:out.append(i)
  else:
   old=out[0];replace={0x2000:lambda:center(old,0)<px,0x4000:lambda:px<center(old,0),0x6000:lambda:py<center(old,1),0x8000:lambda:center(old,1)<py}[rank]()
   if replace:out[-1]=i
  if hier==0x1000:
   parent=word(b,INDEX[0x1fc])
   if parent!=0xffffffff:
    out.pop()
    while word(blocks[parent],4)&8==0:parent=word(blocks[parent],INDEX[0x1fc]);assert parent!=0xffffffff
    if parent not in out:out.append(parent)
 return out

def source_proof(out,image):
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_MEM
 records=[];constants={}
 for folder,n,address in QUERIES:
  p=ROOT/f'target/rea/{folder}/{n}-analyze_function.json';e=json.loads(json.loads(p.read_text())['content'][0]['text']);v=e['raw_result']['value'];assert int(v['procedure']['address'],16)==address;spans=[]
  for span in v['procedure']['body']['ranges']:
   a,z=int(span['start'],16),int(span['end'],16)+1;raw=image.read(a,z-a);rows=list(decoder().disasm(raw,a));assert sum(i.size for i in rows)==len(raw);name=f'source-{a:08X}-{z:08X}.bin';(out/name).write_bytes(raw)
   for r in rows:
    for op in r.operands:
     if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:constants[op.mem.disp]=max(constants.get(op.mem.disp,0),op.size)
   spans.append(dict(start=f'{a:08X}',end_exclusive=f'{z:08X}',bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),blob=name))
  assert sum(x['bytes']for x in spans)==v['procedure']['body']['total_bytes'];records.append(dict(entry=f'{address:08X}',query=str(p),query_sha256=sha(p),evidence_id=e['evidence_id'],ranges=spans,callees=[x['address']for x in v['callees']]))
 raw=image.read(0x11e0534,6);instruction=list(decoder().disasm(raw,0x11e0534));assert len(instruction)==1 and instruction[0].mnemonic=='jmp' and instruction[0].size==6 and instruction[0].operands[0].mem.disp==0x13cc47c;(out/'source-011E0534-011E053A.bin').write_bytes(raw)
 for a,n in constants.items():(out/f'data-{a:08X}.bin').write_bytes(image.read(a,n))
 return dict(source_sha256=SOURCE_SHA256,functions=records,source_bytes=sum(sum(s['bytes']for s in r['ranges'])for r in records)+6,thunk=dict(start='011E0534',bytes=6,sha256=hashlib.sha256(raw).hexdigest(),original_IAT='013CC47C',import_name='MSVCR90.dll!memcpy',provider='Explicit owned bounded value-free memcpy ABI; no proprietary selector/numeric stub'),data=[dict(address=f'{a:08X}',bytes=n,sha256=hashlib.sha256(image.read(a,n)).hexdigest())for a,n in sorted(constants.items())],unreachable_calls={'009B03C0':'Hierarchy policy not0/1000','009B2EF0':'Hierarchy policy not0/1000'},native_executed=False)

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());a.out.mkdir(parents=True,exist_ok=False)
 decoder();image=SourceImage(a.source_exe);source=source_proof(a.out,image);(a.out/'source-closure.json').write_text(json.dumps(source,indent=2)+'\n')
 retained=json.loads((WHOLE/'first-root-static-retention-v3.json').read_text());keys={tuple(x['key'])for x in retained['candidates']};clips=json.loads((WHOLE/'all818-whole-channel-prepared-v2.json').read_text());aliases=[]
 for clip in clips:
  if tuple(clip['key'])not in keys:continue
  for ch in clip['channels']:
   if ch['enabled']and ch['primary'][0]&0x100003==0:aliases.append(dict(key=clip['key'],source_sha256=clip['sha256'],channel=ch['channel'],primary=ch['primary'],secondary=ch['secondary'],metadataAC=ch['metadataAC'],ownerAE=ch['ownerAE'],groupAF=ch['groupAF'],descriptors=ch['descriptors'],admitted=not bool(ch['primary'][0]&0x800c0),exclusion=None if not ch['primary'][0]&0x800c0 else'OriginalDefinitionZBounds360_36CRequired'))
 assert len(aliases)==1644 and all(x['primary'][0]&0x610300==0 and x['primary'][2:]==[0,0] for x in aliases)
 selectors=sorted({tuple(x['primary'])for x in aliases if x['admitted']});assert len(selectors)==59 and sum(x['admitted']for x in aliases)==1641
 (a.out/'selector-aliases.json').write_text(json.dumps(aliases,indent=2)+'\n');(a.out/'selector-table.json').write_text(json.dumps(selectors,indent=2)+'\n')
 assert sha(FINAL/'native-projection.bin')==BODY_SHA and json.loads((FINAL/'comparison-v1.json').read_text())['exact']
 resources=json.loads((ROOT/'target/animation/body-capabilities-inputs-v1/source-resources.json').read_text());bykey={r['key']:r for r in resources};assert len(bykey)==1365
 table=[r.split('\t')for r in(FINPUT/'cases.tsv').read_text().splitlines()[1:]];bodies=[];queries=0;blocks_total=0;matches=0
 with(FINAL/'native-projection.bin').open('rb')as native,(a.out/'input.bin').open('xb')as stream,(a.out/'expected1032.bin').open('xb')as expected:
  for row in table:
   if row[1]!='source_relabel':continue
   case=int(row[0]);native.seek(case*PROJECTION_BYTES);raw=native.read(PROJECTION_BYTES);n=word(raw,8);blocks=[raw[32+i*324:32+(i+1)*324]for i in range(n)];body_id=len(bodies);r=bykey[row[-1]]
   assert 1<=n<=255 and word(blocks[0],4)&8 and word(blocks[0],INDEX[0x1fc])==0xffffffff
   for i,b in enumerate(blocks):
    assert word(b,INDEX[0x1f8])==i
    for at in[0x10c,0x110,0x114,0x138,0x13c,0x140]:assert math.isfinite(number(b,INDEX[at]))and abs(number(b,INDEX[at]))<=1048576.
    for at in[0x138,0x13c,0x140]:assert number(b,INDEX[at])>0
    at=i;seen=set()
    while word(blocks[at],INDEX[0x1fc])!=0xffffffff:
     assert at not in seen;seen.add(at);at=word(blocks[at],INDEX[0x1fc]);assert at<n
    assert at==0
   qcount=len(selectors)*2;stream.write(struct.pack('<8I',0x314c4553,n,qcount,0x027f,body_id,0,0,0));stream.write(b''.join(blocks))
   for si,s in enumerate(selectors):
    for mirrored in(0,1):
     qi=si*2+mirrored;stream.write(struct.pack('<8I',*s,mirrored,255,0,qi));selected=select(blocks,s,mirrored);assert len(selected)<=255 and all(0<=i<n for i in selected);slots=selected+[0xffffffff]*(255-len(selected));expected.write(struct.pack('<258I',body_id,qi,len(selected),*slots));queries+=1;matches+=bool(selected)
   bodies.append(dict(body=body_id,native_case=case,package=r['package'],key=r['key'],source_sha256=r['source_sha256'],source_rows=r['source_rows'],finalized_blocks=n,generated_blocks=sum(word(b,INDEX[0x21c])==0xffffffff for b in blocks),native_projection_sha256=hashlib.sha256(raw).hexdigest(),body_records_sha256=hashlib.sha256(b''.join(blocks)).hexdigest(),source_palette_ids=[word(b,INDEX[0x21c])for b in blocks],actual_actor_association=False));blocks_total+=n
 assert len(bodies)==1322 and blocks_total==46256 and queries==155996
 (a.out/'body-index.json').write_text(json.dumps(bodies,indent=2)+'\n')
 pins=[FINPUT/'report.json',FINPUT/'source-digests.txt',FINPUT/'cases.tsv',FINAL/'native-projection.bin',FINAL/'comparison-v1.json',ROOT/'target/animation/body-capabilities-inputs-v1/source-resources.json',ROOT/'target/animation/part-capabilities.json',WHOLE/'first-root-static-retention-v3.json',WHOLE/'all818-whole-channel-prepared-v2.json',ROOT/'crates/spore-core/src/anim_nonroot_selector.rs',Path(__file__)]
 report=dict(schema='actual-finalized-body-nonroot-selector-inputs-v1',native_executed=False,original_actor_association=False,bodies=1322,finalized_blocks=46256,source_aliases=1644,admitted_source_aliases=1641,retained_Z_exclusions=3,unique_selector_words=59,mirrors=2,queries=queries,nonempty_python_queries=matches,record_bytes=324,group_header_bytes=32,query_bytes=32,projection_bytes=1032,output_bytes=1040,fields=FIELDS,all_genuine_parent_and_capability_and_palette_fields=True,zero_child_substitution=False,pins={str(x.resolve()):sha(x)for x in pins},files={x.name:sha(x)for x in a.out.iterdir()if x.is_file()},scope='Cross product of actual accepted finalized bodies and source selector policies, not inferred clip-to-actor associations. Full9B2150 mode0/noZ plus actual ancestor1000; all1644source channels retained,3Z exclusions explicit. Binding constructor/AE ownership/pruning/live admission separate.')
 (a.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(bodies=len(bodies),queries=queries,input_sha256=sha(a.out/'input.bin'),expected_sha256=sha(a.out/'expected1032.bin'))))
if __name__=='__main__':main()
