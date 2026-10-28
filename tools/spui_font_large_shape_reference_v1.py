"""Source-correct original frame initialization + nonempty simple glyph replay."""
import copy,hashlib,json,sys,textwrap
from pathlib import Path
import spui_font_visible_hint_scout as base
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 template_path=ROOT/'tools/spui_font_visible_hint_scout.py';body=template_path.read_text().split('def run(out,gid):\n',1)[1].split('if __name__==',1)[0];prefix,tail=body.split(' assert gid in [14,42,64,93]\n',1);tail=tail.replace(' assert g.numberOfContours==1 and len(coords)==4 and all(f&1 for f in flags)\n','').split(' out=Path(out);',1)[0]
 class StrictMachine(base.PrepMachine):
  vector_args=[]
  def condition(self,name):
   if name=='jg':assert all(self.flags[k]is not None for k in ['z','s','o'])
   return super().condition(name)
  def apply(self,ins):
   if ins.address in [0x8d0c70,0x8d0db0,0x8d0d50,0x8d0eb0]:
    esp=self.reg('esp');args=[base.signed(self.word(esp+4+i*4))for i in range(2)];self.vector_args.append([ins.address,*args])
   result=super().apply(ins)
   if ins.address==0x8abcd0:
    cursor=self.word(base.FRAME)-1
    if 0x3c0000<=cursor<0x3d0000:self.trace[-1].update(stream='glyph0',pc=cursor-0x3c0000)
   return result
 env=dict(base.__dict__);env['PrepMachine']=StrictMachine;exec(textwrap.dedent(prefix),env);template=env['m'];instructions=env['ins']
 # Run literal original frame-construction prefix at its actual source addresses.
 init=StrictMachine(instructions);init.vector_args=[];init.region(base.FRAME,bytes(256));init.reg('ebp',base.FRAME+0x3c);init.reg('eax',0x38002c);init.run(0x8abd1f,0x8abd81)
 words={off:init.word(base.FRAME+off,2)for off in [0x14,0x16,0x18,0x1a,0x1c,0x1e,0x40]};callbacks={off:init.word(base.FRAME+off)for off in [0x44,0x48,0x4c]};assert [words[i]for i in [0x14,0x16,0x18,0x1a,0x1c,0x1e,0x40]]==[16384,0,16384,0,16384,0,16384];assert list(callbacks.values())==[0x8ab970,0x8aba30,0x8aba30]
 old='for off,value in [(0x14,0x4000),(0x18,0x4000),(0x1a,0x4000),(0x40,0x4000)]:';assert old in tail;tail=tail.replace(old,'for off,value in INITIAL_WORDS.items():');env['INITIAL_WORDS']=words;code=compile(textwrap.dedent(tail),str(template_path)+'#correct-original-frame','exec');census=json.loads((ROOT/'target/source-ui/font-simple-native-census-v5/manifest.json').read_text());rows=[];different=[]
 for row in census['rows']:
  gid=row['gid'];oldpath=ROOT/f'target/source-ui/font-simple-census-v3/gid{gid}-hint.json';baseline=json.loads(oldpath.read_text())
  if baseline['glyph']['program_bytes']==0 or baseline['glyph']['real_points']<=64:continue
  m=copy.copy(template);m.memory=[(a,bytearray(b)if w else b,w)for a,b,w in template.memory];m.r=template.r.copy();m.flags=template.flags.copy();m.memory=[(a,b,w)for a,b,w in m.memory if a not in [0x38002c,0x3b0000,0x3c0000,0x3d0000]];m.cvt_writes=[];m.delta_inputs=[];m.vector_args=[];env.update(m=m,gid=gid);exec(code,env);r=env['report'];assert r['blocker']is None
  changes=[]
  for field,a,b in [('points',r['glyph']['output_points'],baseline['glyph']['output_points']),('touched',r['glyph']['touched'],baseline['glyph']['touched']),('CVT',r['cvt'],baseline['cvt']),('storage',r['storage'],baseline['storage']),('graphics',r['state_hex'],baseline['state_hex'])]:
   if json.dumps(a,sort_keys=True)!=json.dumps(b,sort_keys=True):changes.append(field)
  if changes:different.append(dict(gid=gid,fields=changes))
  r['vector_call_inputs']=m.vector_args
  r['schema']='font0-correct-frame-source-hint-v1';r['initial_frame_words']={f'{k:02X}':v for k,v in words.items()};r['initial_frame_callbacks']={f'{k:02X}':f'{v:08X}'for k,v in callbacks.items()};r['admission']='Nonempty actualsimplefont0square15; original8ABD1F..81framewordsreplayed literally; sourcefullprogramtracewindow; freshCVT/storage/twilight per glyph. Emptyprogramdriverbypass excluded.';p=out/f'gid{gid}-hint.json';p.write_text(json.dumps(r,indent=2)+'\n');rows.append(dict(gid=gid,path=p.name,sha256=sha(p),old_hint_sha256=sha(oldpath),numeric_changes=changes))
 report=dict(schema='font0-correct-frame-nonempty-simple-reference-v2',tool_sha256=sha(__file__),template_sha256=sha(template_path),source_sha256=base.SOURCE_SHA256,original_init_source=['008ABD1F','008ABD81'],initial_words=words,initial_callbacks=callbacks,nonempty_simple_glyphs=len(rows),empty_program_exclusions=[5,132,219,230],numeric_changes=different,rows=rows,native_executed=False,rust_executed=False,limits='Newframe/controlreference; old389census preserved. OldinitialmanualfreedomY4000anddualX0 corrected fromliteraloriginalprefix. Nativepixels/metrics/CVT/storagecomparisonseparate; fullframevectors nowcanonical candidates.');(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(rows),len(different))
if __name__=='__main__':main(sys.argv[1])
