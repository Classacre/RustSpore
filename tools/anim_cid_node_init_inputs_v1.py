"""Materialize actual accepted body-field projections for original CID node loop."""
import argparse,hashlib,json,struct
from pathlib import Path
from anim_body_finalizer_native_probe import FIELDS
from anim_cid_node_init_native_prepare_v1 import ROOT,BLOCK,NODE,MASK,WRITES
COHORT=ROOT/'target/animation/nonroot-selector-Z-inputs-v1-complete'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def word(b,o):return struct.unpack_from('<I',b,o)[0]
def put(b,o,v):struct.pack_into('<I',b,o,v)
def projection(blocks):
 rows=[]
 for i,b in enumerate(blocks):
  r=bytearray(b'\xa5'*NODE);parent=word(b,0x1fc);child=word(b,0x204);count=word(b,0x208)
  assert not i or parent<len(blocks)
  assert not count or child<len(blocks)
  for o in list(range(0x58,0xa4,4))+[0x48,0x4c,0x50]+list(range(0x144,0x164,4)):put(r,o,0)
  put(r,0,i);put(r,4,0);put(r,8,parent if i else 0xffffffff);put(r,12,child if count else 0xffffffff)
  for d,s in [(0x10,0x10c),(0x14,0x110),(0x18,0x114),(0x3c,0x10c),(0x40,0x110),(0x44,0x114),(0x1c,0x118),(0x20,0x11c),(0x24,0x120),(0x28,0x124)]:
   bits=word(b,s);assert bits&0x7f800000!=0x7f800000;put(r,d,bits)
  for o in [0x38,0x54,0x55,0x2a0]:r[o]=0
  put(r,0x154,0x3f800000);rows.append(r)
 return b''.join(rows)
def prepare(out):
 out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 idx=json.loads((COHORT/'body-index.json').read_text());m=json.loads((COHORT/'manifest.json').read_text());assert len(idx)==1322 and m['body_upstream_exclusions']==43
 index=[];total=children=0
 with(COHORT/'input.bin').open('rb')as src,(out/'input.bin').open('xb')as inp,(out/'expected.bin').open('xb')as expected:
  for bi,body in enumerate(idx):
   src.seek(body['offset']);old=src.read(32);n=word(old,4);assert 1<=n<=255 and n==body['finalized_blocks'];records=src.read(n*324);assert hashlib.sha256(records).hexdigest()==body['body_records_sha256'];blocks=[]
   for i in range(n):
    rec=records[i*324:(i+1)*324];b=bytearray(b'\xa5'*BLOCK);b[0x150:0x1f8]=rec[:168]
    for j,at in enumerate(FIELDS):put(b,at,word(rec,168+j*4))
    assert word(b,0x1f8)==i
    blocks.append(bytes(b));children+=word(b,0x208)!=0
   header=struct.pack('<8I',0x31444f4e,n,0x27f,0x1f80,bi,0x3f800000,0,0);offset=inp.tell();inp.write(header+b''.join(blocks));gold=header+projection(blocks);expected.write(gold)
   index.append(dict(body=bi,key=body['key'],package=body['package'],source_sha256=body['source_sha256'],count=n,input_offset=offset,input_bytes=32+n*BLOCK,projection_bytes=len(gold),reply_bytes=len(gold)+8,nonzero_child_nodes=sum(word(b,0x208)!=0 for b in blocks),block_projection_sha256=hashlib.sha256(b''.join(blocks)).hexdigest(),projection_sha256=hashlib.sha256(gold).hexdigest(),actual_actor_association=False));total+=n
 assert total==46256
 (out/'index.json').write_text(json.dumps(index,indent=2)+'\n')
 first=(out/'input.bin').read_bytes()[:index[0]['input_bytes']];protocols={}
 for name,at,value in [('magic',0,0),('count_zero',4,0),('count_over',4,256),('cw',8,0x37f),('csr',12,0xdf81),('scale',20,0),('reserved',24,1),('nonfinite_center',32+0x10c,0x7f800000),('nonfinite_quaternion',32+0x118,0x7fc00000),('child_index',32+0x204,255)]:
  raw=bytearray(first);put(raw,at,value)
  if name=='child_index':put(raw,32+0x208,1)
  path=out/(name+'.bin');path.write_bytes(raw);protocols[name]=sha(path)
 raw=bytearray(first);put(raw,32+BLOCK+0x1fc,255);(out/'parent_index.bin').write_bytes(raw);protocols['parent_index']=sha(out/'parent_index.bin')
 pins={str(p):sha(p)for p in [COHORT/'input.bin',COHORT/'body-index.json',COHORT/'manifest.json',Path(__file__),ROOT/'tools/anim_body_finalizer_native_probe.py']}
 report=dict(schema='CID-node-init-actual1322-body-inputs-v1',bodies=1322,nodes=total,nonzero_child_nodes=children,upstream_body_exclusions=43,source_projection_fields=FIELDS,unknown_definition_bytes='A5;notclaimedoriginalfullrecordbytes',actual_factory_scale_bits='3F800000',cw='027F',csr='1F80',all700_node_bytes_projected=True,canonical_pointer_policy='Definitionordinal,CID0,parent/childordinalorFFFFFFFF',reference_arithmetic='Finitef32MULSSbypositive1andFLD/FSTPexact; signedzeroandsubnormalsretained',native_executed=False,full_factory=False,input_sha256=sha(out/'input.bin'),expected_sha256=sha(out/'expected.bin'),protocols=protocols,pins=pins)
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');return report
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);print(json.dumps(prepare(p.parse_args().out)))
