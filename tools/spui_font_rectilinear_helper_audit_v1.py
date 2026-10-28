"""Source helper call-domain evidence for E/F/H/L/T; no native execution."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
path=ROOT/'tools/spui_font_simple_census_v3.py'
s=path.read_text().replace('for gid,name in enumerate(font.getGlyphOrder()):','for gid,name in enumerate(font.getGlyphOrder()):\n  if gid not in [38,39,41,45,53]:continue')
s=s.replace('  def condition(self,name):','  def apply(self,ins):\n   if ins.address in [0x8d0db0,0x8d0c70]:\n    sp=self.reg("esp");helper_calls.append(dict(gid=current_gid[0],entry=f"{ins.address:08X}",a=base.signed(self.word(sp+4)),b=base.signed(self.word(sp+8))))\n   return super().apply(ins)\n  def condition(self,name):')
s=s.replace('env.update(m=m,gid=gid)','env.update(m=m,gid=gid);current_gid[0]=gid')
s=s.replace("if __name__=='__main__':main(sys.argv[1])",'')
helper_calls=[];current_gid=[None];env=dict(__name__='audit',__file__=str(path),helper_calls=helper_calls,current_gid=current_gid)
exec(compile(s,str(path)+'#helper-audit','exec'),env);env['main'](sys.argv[1]);out=Path(sys.argv[1]);report=dict(schema='rectilinear-source-helper-domain-v1',calls=helper_calls,new_native_execution=False)
(out/'helper-calls.json').write_text(json.dumps(report,indent=2)+'\n');print(helper_calls)
