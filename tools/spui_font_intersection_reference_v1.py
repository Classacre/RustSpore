"""Bounded whole original0F instruction oracle; source bytes stay in target."""
import hashlib,itertools,json,random,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_vector_primitives_reference_v1 import machine,ZONE
from spui_font_hint_prep_probe import STACK,FRAME,OPERANDS,signed
ROOT=Path(__file__).resolve().parent.parent
ENTRIES=[0x8ac3c0,0x8d0db0,0x8d0c70]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
 front=ROOT/'target/source-ui/font-fixture-frontier-v7.json';fs=json.loads(front.read_text());ins={};functions=[]
 for entry in ENTRIES:
  f=next(f for f in fs['functions']if f['entry']==f'{entry:08X}')
  raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16))for a,b in f['ranges']);assert hashlib.sha256(raw).hexdigest()==f['sha256']
  (out/f'{entry:08X}.bin').write_bytes(raw)
  decoded=[r for a,b in f['ranges']for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))]
  assert sum(r.size for r in decoded)==len(raw)
  ins.update({r.address:r for r in decoded});functions.append(f)
  (out/f'{entry:08X}.asm').write_text('\n'.join(f'{r.address:08X} {r.mnemonic} {r.op_str}'for r in decoded)+'\n')
 # Explicit orientations, degeneracy, parallel and signed odd-coordinate cases.
 lines=[([0,0],[0,0]),([0,0],[64,0]),([1,-3],[-66,-3]),([0,0],[0,64]),([-3,1],[-3,-66]),([0,0],[64,64]),([1,-3],[-66,64]),([-8192,8192],[8192,-8192]),([10,20],[11,84]),([-43,-17],[82,-16])]
 cases=[(*a,*b)for a in lines for b in lines]
 points=[[-13,21],[72,-31],[18,97],[-91,-57]]
 cases.extend(tuple(p)for p in itertools.permutations(points))
 rng=random.Random(0x8ac3c0)
 cases.extend(tuple([rng.randint(-8192,8192),rng.randint(-8192,8192)]for _ in range(4))for _ in range(2048))
 rows=[];coverage=set();snapshots=[]
 for i,(a0,a1,b0,b1)in enumerate(cases):
  m=machine(ins);visited=[];apply=m.apply
  def observe(r):visited.append(r.address);return apply(r)
  m.apply=observe;m.word(STACK+4,value=FRAME);m.word(FRAME+0x20,value=OPERANDS+20)
  # Stack order is target,b0,b1,a0,a1; distinct zone fields alias here by contract.
  for j,v in enumerate([0,1,2,3,4]):m.word(OPERANDS+4*j,value=v)
  initial=[[777,-555],b0,b1,a0,a1]
  for j,(x,y)in enumerate(initial):m.word(ZONE+0x100+4*j,value=x);m.word(ZONE+0x200+4*j,value=y)
  touch=[0,1,2,0x40][i%4];m.word(ZONE+0x500,1,value=touch)
  m.run(0x8ac3c0,0xffffffff,limit=10000)
  assert m.reg('esp')==STACK+4 and m.word(FRAME+0x20)==OPERANDS
  for j,point in enumerate(initial[1:],1):assert [signed(m.word(ZONE+off+4*j))for off in(0x100,0x200)]==point
  result=[signed(m.word(ZONE+off))for off in(0x100,0x200)]+[m.word(ZONE+0x500,1)]
  args=[*a0,*a1,*b0,*b1,touch];rows.append('I0F\t'+str(i)+'\t'+','.join(map(str,args))+'\t'+','.join(map(str,result)))
  coverage.update(visited);snapshots.append(dict(case=i,instructions=len(visited)))
 blob=('\n'.join(rows)+'\n').encode();(out/'expected.tsv').write_bytes(blob)
 branch_starts=[0x8ac46f,0x8ac485,0x8ac49e,0x8ac4b4,0x8ac4d7,0x8ac513,0x8ac571,0x8ac5ad,0x8ac5ed,0x8ac60b]
 reached=[f'{a:08X}'for a in branch_starts if a in coverage]
 pins={str(p.relative_to(ROOT)):sha(p)for p in [Path(__file__),ROOT/'tools/spui_font_vector_primitives_reference_v1.py',ROOT/'tools/spui_font_hint_prep_probe.py',front,ROOT/'crates/spore-core/src/spui_font_intersection.rs']}
 report=dict(schema='font0-source-whole0F-instruction-reference-v1',source_sha256=SOURCE_SHA256,functions=functions,rows=len(rows),canonical_sha256=hashlib.sha256(blob).hexdigest(),branch_blocks_reached=reached,unreached_branch_blocks=[f'{a:08X}'for a in branch_starts if a not in coverage],pins=pins,native_executed=False,rust_executed=False,limits='Wholeoriginal641B0F+198Bfixedhelpers interpreted; bounded owned current26.6 line coordinates,sharedvalidzones,actual fiveoperand order/stackconsumption/touchOR and otherpoints preserved. No VM/glyph/otherzone/native admission. Zero-numerator alternatebranch is unreachable in owned nonoverflow magnitude domain.')
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(rows=len(rows),canonical_sha256=report['canonical_sha256'],reached=reached,unreached=report['unreached_branch_blocks'])))
if __name__=='__main__':main(sys.argv[1])
