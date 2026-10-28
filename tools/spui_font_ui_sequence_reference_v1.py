"""Fresh versus persistent font0 source gamma sequence. No native execution."""
import copy,hashlib,json,sys,textwrap
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import spui_font_visible_hint_scout as base

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 out=Path(out).resolve();out.relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 template_path=ROOT/'tools/spui_font_visible_hint_scout.py';body=template_path.read_text().split('def run(out,gid):\n',1)[1].split("if __name__==",1)[0];prefix,tail=body.split(' assert gid in [14,42,64,93]\n',1)
 tail=tail.replace(' assert g.numberOfContours==1 and len(coords)==4 and all(f&1 for f in flags)\n','').split(' out=Path(out);',1)[0]
 class StrictMachine(base.PrepMachine):
  def condition(self,name):
   if name=='jg':assert all(self.flags[k] is not None for k in ['z','s','o'])
   return super().condition(name)
  def apply(self,ins):
   result=super().apply(ins)
   if ins.address==0x8abcd0:
    cursor=self.word(base.FRAME)-1
    if 0x3c0000<=cursor<0x3d0000:self.trace[-1].update(stream='glyph0',pc=cursor-0x3c0000)
   return result
 env=dict(base.__dict__);env['PrepMachine']=StrictMachine;exec(textwrap.dedent(prefix),env);template=env['m'];code=compile(textwrap.dedent(tail),str(template_path)+'#sequence','exec');rows=[]
 for key,gids in [('fresh',[354]),('preceded',[342,354])]:
  m=copy.copy(template);m.memory=[(a,bytearray(b) if w else b,w) for a,b,w in template.memory];m.r=template.r.copy();m.flags=template.flags.copy()
  for gid in gids:
   m.memory=[(a,b,w) for a,b,w in m.memory if a not in [0x38002c,0x3b0000,0x3c0000,0x3d0000]]
   m.cvt_writes=[];m.delta_inputs=[];before=[base.signed(m.word(base.CVT+i*4)) for i in range(173)];before_steps=m.steps;env.update(m=m,gid=gid);exec(code,env);r=env['report'];assert r['blocker'] is None;r['schema']='font0-persistent-gamma-source-sequence-v1';r['admission']='Freshsquare15ppem; full sourcePREP once percontext; CVT/storage/twilight persist; glyph-frame/ordinaryGS reset by source VM; corrected full-owned-program trace address window; no native execution';r['before_cvt']=before;r['glyph']['source_steps']=m.steps-before_steps;p=out/f'{key}-gid{gid}-hint.json';p.write_text(json.dumps(r,indent=2)+'\n');rows.append(dict(context=key,gid=gid,before_cvt14=before[14],after_cvt14=r['cvt'][14],hint=str(p.relative_to(ROOT)),hint_sha256=sha(p)))
 report=dict(schema='font0-source-persistent-gamma-sequence-v1',tool_sha256=sha(__file__),template_sha256=sha(template_path),source_sha256=base.SOURCE_SHA256,new_native_execution=False,rust_execution=False,rows=rows);(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(rows)
if __name__=='__main__':main(sys.argv[1])
