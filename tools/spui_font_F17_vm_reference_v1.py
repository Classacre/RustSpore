"""Whole original F17/dynamic44..47 source VM cases; no native or Rust expected."""
import copy,hashlib,json,sys,textwrap
from pathlib import Path
import spui_font_visible_hint_scout as base
ROOT=Path(__file__).resolve().parent.parent

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 template_path=ROOT/'tools/spui_font_visible_hint_scout.py';body=template_path.read_text().split('def run(out,gid):\n',1)[1].split('if __name__==',1)[0];prefix,tail=body.split(' assert gid in [14,42,64,93]\n',1);tail=tail.replace(' assert g.numberOfContours==1 and len(coords)==4 and all(f&1 for f in flags)\n','').split(' out=Path(out);',1)[0]
 class StrictMachine(base.PrepMachine):
  def condition(self,name):
   if name=='jg':assert all(self.flags[k]is not None for k in ['z','s','o'])
   return super().condition(name)
 env=dict(base.__dict__);env['PrepMachine']=StrictMachine;exec(textwrap.dedent(prefix),env);template=env['m'];instructions=env['ins']
 init=StrictMachine(instructions);init.region(base.FRAME,bytes(256));init.reg('ebp',base.FRAME+0x3c);init.reg('eax',0x38002c);init.run(0x8abd1f,0x8abd81);words={off:init.word(base.FRAME+off,2)for off in [0x14,0x16,0x18,0x1a,0x1c,0x1e,0x40]}
 old='for off,value in [(0x14,0x4000),(0x18,0x4000),(0x1a,0x4000),(0x40,0x4000)]:';assert old in tail;tail=tail.replace(old,'for off,value in INITIAL_WORDS.items():').replace('glyph_code=g.program.getBytecode()','glyph_code=OWNED_PROGRAM');env['INITIAL_WORDS']=words;code=compile(textwrap.dedent(tail),str(template_path)+'#F17ownedsourceVM','exec')
 values=[-257,-193,-129,-65,-1,0,1,31,32,63,64,65,95,96,127,128,129,159,160,191,192,193,223,224,255,256,257,319,320,383,384,511,512,1024];inputs=[];lines=[];rows=[];ids=set();case=0
 for axis in [0,1]:
  for roundop in [0x18,0x19,0x3d,0x7d,0x7a]:
   for x in values:
    program=bytes([axis,roundop,0xb0,41,0xb8,(x>>8)&255,x&255,0xb0,17,0x2b,0x44]);m=copy.copy(template);m.memory=[(a,bytearray(b)if w else b,w)for a,b,w in template.memory];m.r=template.r.copy();m.flags=template.flags.copy();m.memory=[(a,b,w)for a,b,w in m.memory if a not in [0x38002c,0x3b0000,0x3c0000,0x3d0000]];m.cvt_writes=[];m.delta_inputs=[];env.update(m=m,gid=0,OWNED_PROGRAM=program);exec(code,env);r=env['report'];assert r['blocker']is None and r['final_stack_empty'];defs={d['offset']:d['index']for d in r['body_definitions']};calls=[defs[c['start']]for c in r['call_entries']];ids.update(calls);frame=bytes.fromhex(r['frame_hex']);state=bytes.fromhex(r['state_hex']);import struct
    vectors=struct.unpack_from('<6h',frame,0x14);dot=struct.unpack_from('<H',frame,0x40)[0];callbacks=struct.unpack_from('<3I',frame,0x44);refs=struct.unpack_from('<3i',frame,0x30)
    lines +=[f'owned{case}\tresult\t{r["cvt"][41]}',f'owned{case}\tframe\t'+'\t'.join(map(str,[*vectors,dot]))+'\t'+'\t'.join(f'{v:08X}'for v in callbacks),f'owned{case}\tgraphics\t'+state[0x58:0x8c].hex(),f'owned{case}\tfinal\t0\t{len(calls)}\t'+'\t'.join(map(str,refs))];inputs.append(f'{case}\t{axis}\t{roundop}\t{x}');rows.append(dict(case=case,axis=axis,round_opcode=f'{roundop:02X}',input=x,calls=calls,dispatch_count=len(r['trace']),cvt41=r['cvt'][41]));case+=1
 assert ids=={17,44,45,46,47},ids
 for name,rows_data in [('input.tsv',inputs),('expected.tsv',lines)]: (out/name).write_bytes(('\n'.join(rows_data)+'\n').encode())
 pins={str(p.relative_to(ROOT)):sha(p)for p in [Path(__file__),template_path,ROOT/'tools/spui_font_general_vector_hint_reference_v2.py',ROOT/'target/source-ui/font-fixture-frontier-v7.json',ROOT/'target/source-ui/font-fpgm-bootstrap-static-v1.json']}
 report=dict(schema='font-F17-dynamic44through47-whole-source-VM-reference-v1',cases=case,rows=len(lines),input_sha256=sha(out/'input.tsv'),canonical_sha256=sha(out/'expected.tsv'),called_ids=sorted(ids),case_calls=rows,pins=pins,native_executed=False,rust_executed=False,scope='OriginalFPGM17/44..47bytes execute via originalwholeVM andfull freshoriginalPREP/actualfont0. Ownedaxis+round+scalarprogram stores result viaoriginal44 intoCVT41; no invented functionoutput. All five roundprofiles; actualselectedcallchain explicit. FullF20/generalvector/glyphgate remains separate.');(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(case,len(lines),report['canonical_sha256'])
if __name__=='__main__':main(sys.argv[1])
