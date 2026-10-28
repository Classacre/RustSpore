"""Original0F handler VM binding/alias oracle over owned ordinary-zone arrays."""
import hashlib,itertools,json,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_vector_primitives_reference_v1 import machine,ZONE
from spui_font_hint_prep_probe import STACK,FRAME,OPERANDS,signed
ROOT=Path(__file__).resolve().parent.parent

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));front=ROOT/'target/source-ui/font-fixture-frontier-v7.json';fr=json.loads(front.read_text());ins={};functions=[]
 for e in [0x8ac3c0,0x8d0db0,0x8d0c70]:
  f=next(x for x in fr['functions']if int(x['entry'],16)==e);raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16))for a,b in f['ranges']);assert hashlib.sha256(raw).hexdigest()==f['sha256'];functions.append(f)
  for a,b in f['ranges']:ins.update({r.address:r for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))})
 geometry=[[[0,0],[64,0],[13,-31],[13,72]],[[-13,21],[72,-31],[18,97],[-91,-57]],[[0,0],[0,64],[-63,17],[65,17]],[[1,-3],[-66,64],[10,20],[11,84]],[[0,0],[0,0],[64,64],[-64,-64]]]
 bindings=list(itertools.permutations(range(4)))+[(0,0,1,2),(0,1,2,2),(0,1,0,1),(3,3,3,3)];inputs=[];rows=[];n=0;alias=0
 for geo in geometry:
  initial=geo+[[0,0]for _ in range(4)]
  for target in range(5):
   for binding in bindings:
    args=[target,*binding];m=machine(ins);m.word(STACK+4,value=FRAME);m.word(FRAME+0x20,value=OPERANDS+20)
    for j,v in enumerate(args):m.word(OPERANDS+4*j,value=v)
    for j,(x,y)in enumerate(initial):m.word(ZONE+0x100+j*4,value=x);m.word(ZONE+0x200+j*4,value=y)
    m.run(0x8ac3c0,0xffffffff,limit=10000);assert m.reg('esp')==STACK+4 and m.word(FRAME+0x20)==OPERANDS
    for j,point in enumerate(initial):
     got=[signed(m.word(ZONE+off+j*4))for off in [0x100,0x200]];touch=m.word(ZONE+0x500+j,1)
     if j!=target:assert got==point and touch==0
     rows.append(f'owned{n}\tpoint\t{j}\t{got[0]}\t{got[1]}\t{touch}')
    rows.append(f'owned{n}\tfinal\t0\t0\t0\t0');inputs.append('\t'.join(map(str,[n,*sum(initial,[]),*args])));alias+=target in binding;n+=1
 for name,data in [('input.tsv',inputs),('expected.tsv',rows)]: (out/name).write_text('\n'.join(data)+'\n')
 pins={str(p.relative_to(ROOT)):sha(p)for p in [Path(__file__),front,ROOT/'tools/spui_font_vector_primitives_reference_v1.py',ROOT/'tools/spui_font_hint_prep_probe.py',ROOT/'crates/spore-core/src/spui_font_intersection.rs',ROOT/'target/source-ui/font-intersection-rust-run-v1/acceptance-v1.json']}
 report=dict(schema='font-general-intersection-VM-alias-source-reference-v1',cases=n,rows=len(rows),target_aliases_line_endpoint=alias,input_sha256=sha(out/'input.tsv'),canonical_sha256=sha(out/'expected.tsv'),source_sha256=SOURCE_SHA256,functions=functions,pins=pins,native_executed=False,rust_executed=False,scope='Owned4real+4zero phantom current26.6points; stackbottomtotop target,b0,b1,a0,a1. Exact originalwhole0Fhandler, otherpoints/touches unchanged, all5operandsconsumed, refs unchanged. Alias cases require line snapshots before targetwrite. Previousupper-touch-bit preservation is separately accepted2172helpergate; VM freshtouch begins0. No glyph/native/UIadmission.');(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(n,len(rows),alias,report['canonical_sha256'])
if __name__=='__main__':main(sys.argv[1])
