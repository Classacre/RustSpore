"""Versioned read-only font0 census by original saved x86 instruction replay.
Frozen four-character source probe is a pinned template; no native execution.
"""
import copy,contextlib,hashlib,io,json,sys,textwrap
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import spui_font_visible_hint_scout as base

def main(out):
 out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 path=ROOT/'tools/spui_font_visible_hint_scout.py';text=path.read_text()
 body=text.split('def run(out,gid):\n',1)[1].split("if __name__==",1)[0]
 prefix,tail=body.split(' assert gid in [14,42,64,93]\n',1)
 env=dict(base.__dict__)
 exec(textwrap.dedent(prefix),env)
 template=env['m'];font=env['font'];rows=[]
 tail=tail.replace(' assert g.numberOfContours==1 and len(coords)==4 and all(f&1 for f in flags)\n','')
 tail=tail.replace(' assert blocker is None and pc==0xffffffff and m.word(FRAME+0x20)==OPERANDS and m.word(FRAME)==0xFEEDFACE\n','')
 tail=tail.replace(" assert m.reg('esp')==STACK+4 and m.trace\n",'')
 tail=tail.split(' out=Path(out);',1)[0]
 code=compile(textwrap.dedent(tail),str(path)+'#census-adaptation','exec')
 for gid,name in enumerate(font.getGlyphOrder()):
  g=font['glyf'][name]
  if g.numberOfContours<=0:
   rows.append(dict(gid=gid,name=name,status='composite' if g.numberOfContours<0 else 'empty'));continue
  m=copy.copy(template);m.memory=[(a,bytearray(b) if w else b,w) for a,b,w in template.memory];m.r=template.r.copy();m.flags=template.flags.copy();m.trace=[];m.call_entries=[];m.cvt_writes=[];m.delta_inputs=[]
  env.update(m=m,gid=gid)
  try:
   exec(code,env);report=env['report'];blocker=report['blocker'];status='hint_complete' if blocker is None else 'hint_first_stop'
   dest=out/f'gid{gid}-hint.json';dest.write_text(json.dumps(report,indent=2)+'\n')
   rows.append(dict(gid=gid,name=name,status=status,points=report['glyph']['real_points'],contours=g.numberOfContours,blocker=blocker,report=dest.name,dispatches=len(report['trace'])))
  except Exception as ex:
   rows.append(dict(gid=gid,name=name,status='model_setup_or_report_stop',error=type(ex).__name__+': '+str(ex)))
  if gid%25==0:print(gid,flush=True)
 manifest=dict(schema='font0-simple-source-census-v1',new_native_execution=False,rust_execution=False,template_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),source_sha256=base.SOURCE_SHA256,scope='Fresh font0 square15ppem PREP snapshot cloned for each actual simple glyph; original x86 model; first unsupported instruction is not implementation acceptance',rows=rows)
 from collections import Counter
 manifest['counts']=dict(Counter(r['status'] for r in rows));(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(manifest['counts'])
if __name__=='__main__':main(sys.argv[1])
