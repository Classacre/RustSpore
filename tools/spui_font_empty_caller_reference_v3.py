"""Actual simple caller scale + empty8CC5C0 bypass; no fabricated VM frame."""
import copy,hashlib,json,struct,sys,textwrap
from pathlib import Path
import spui_font_visible_hint_scout as base
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 source=ROOT/'tools/spui_font_visible_hint_scout.py';body=source.read_text().split('def run(out,gid):\n',1)[1].split('if __name__==',1)[0];prefix=body.split(' assert gid in [14,42,64,93]\n',1)[0];env=dict(base.__dict__);exec(textwrap.dedent(prefix),env);template=env['m'];image=env['image'];ins=env['ins'];cs=env['cs'];font=env['font'];rows=[]
 for a,b in [(0x8c0552,0x8c05c8),(0x8bf350,0x8bf424),(0x8cc5c0,0x8cc7f3)]:
  raw=image.read(a,b-a);(out/f'{a:08X}.bin').write_bytes(raw);ins.update({i.address:i for i in cs.disasm(raw,a)})
 co,raw,steps=base.coefficients(ins,15,font['head'].unitsPerEm)
 for gid in [5,132,219,230]:
  name=font.getGlyphOrder()[gid];g=font['glyf'][name];coords,ends,flags=g.getCoordinates(font['glyf']);assert len(g.program.getBytecode())==0
  va,top,_=base.vertical(ins,font['hhea']);advance,bearing=font['hmtx'].metrics[name];phantoms,_=base.phantom(ins,g,advance,bearing,va,top);design=list(coords)+phantoms;n=len(coords);obj,packet,xs,ys,xword,yword,owner=0x10000000,0x10010000,0x10020000,0x10030000,0x10040000,0x10050000,0x10060000
  m=copy.copy(template);m.memory=[(a,bytearray(b)if w else b,w)for a,b,w in template.memory];m.r=template.r.copy();m.flags=template.flags.copy();m.region(obj,bytes(0x200));m.region(packet,bytes(0x100));m.region(owner,bytes(0x100));m.region(xs,bytes(len(design)*4));m.region(ys,bytes(len(design)*4));m.region(xword,struct.pack('<'+str(len(design))+'h',*(p[0]for p in design)));m.region(yword,struct.pack('<'+str(len(design))+'h',*(p[1]for p in design)))
  m.bytes(obj+0x140,20,raw);m.bytes(obj+0x154,20,raw);m.word(obj+0x184,value=owner);m.word(owner+0x44,value=base.STATE);m.word(packet+0x44,value=xword);m.word(packet+0x48,value=yword);m.word(packet+0x50,value=xs);m.word(packet+0x54,value=ys);m.word(packet+0x64,value=0);m.word(packet+0x68,value=0);m.word(packet+0x38,2,n)
  m.reg('esi',obj);m.reg('ebp',packet);m.reg('ebx',n);m.reg('edx',xs);m.reg('esp',base.STACK);m.word(base.STACK+0x18,value=xs);m.word(base.STACK+0x1c,value=ys);m.word(base.STACK+0x24,value=1);before=m.bytes(base.STATE,256);cvt=m.bytes(base.CVT,173*4);storage=m.bytes(base.STORAGE,47*4);m.trace=[];m.call_entries=[];m.run(0x8c0582,0x8c05c8,limit=10000)
  points=[[base.signed(m.word(xs+i*4)),base.signed(m.word(ys+i*4))]for i in range(n+4)];assert points==[[((v*15+16)>>5)for v in p]for p in design];assert m.bytes(base.STATE,256)==before and m.bytes(base.CVT,173*4)==cvt and m.bytes(base.STORAGE,47*4)==storage;assert not m.trace and not m.call_entries
  r=dict(schema='font0-actual-simple-empty-caller-source-v1',gid=gid,branch='Bypassed',real_points=n,contour_ends=list(ends),on_curve=[int(f&1!=0)for f in flags],design_points=design,output_points=points,graphics_unchanged=True,cvt_unchanged=True,storage_unchanged=True,vm_entered=False,phantom_gridlock_executed=False,caller_range=['008C0582','008C05C8'],admission='Actualsimplecaller8C0582scalesallreal+4phantoms via original8BF350; helper8CC5C0 bypasses. NoVMframe/resultinvented. Postcallertransform/raster/origin/cachejoinseparate.');p=out/f'gid{gid}.json';p.write_text(json.dumps(r,indent=2)+'\n');rows.append(dict(gid=gid,sha256=sha(p)))
 (out/'manifest.json').write_text(json.dumps(dict(rows=rows,tool_sha256=sha(__file__),source_sha256=base.SOURCE_SHA256,caller_scaled=True,postcaller_raster_join=False,new_native_executed=False,rust_executed=False),indent=2)+'\n');print(rows)
if __name__=='__main__':main(sys.argv[1])
