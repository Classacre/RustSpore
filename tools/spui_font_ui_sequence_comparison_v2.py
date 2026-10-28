"""Original raster replay for source persistent gamma sequence vs retained native."""
import hashlib,json,sys,textwrap
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import spui_font_visible_raster_scout as base

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(source,out):
 source=Path(source).resolve();out=Path(out).resolve();out.relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 text=(ROOT/'tools/spui_font_simple_raster_census_v1.py').read_text();setup=text.split(" text=(ROOT/",1)[1].split("native={}",1)[0];env=dict(ROOT=ROOT,base=base);exec(textwrap.dedent(' text=(ROOT/'+setup),env);execute=env['execute'];image,frontier,ins=env['image'],env['frontier'],env['ins'];rows=[]
 import spui_font_ui_sequence_reference_v2 as seq
 from spui_font_visible_hint_scout import PrepMachine
 from spui_font_scaling_probe import STACK,signed
 import struct
 for row in json.loads((source/'manifest.json').read_text())['rows']:
  hp=ROOT/row['hint'];assert sha(hp)==row['hint_sha256'];h=json.loads(hp.read_text());g=h['glyph'];n=g['real_points'];points=g['output_points'][:n]
  # Actual on-curve flags from the original zone's source-populated flag array.
  data=bytes.fromhex(g['data_hex']);flags=list(data[(n+4)*20:(n+4)*21])[:n]
  r=execute(image,frontier,ins,points,g['contour_ends'],flags,0,0xa5);assert r['blocker'] is None
  m=PrepMachine(ins);obj,xs,ys=0x10000000,0x10010000,0x10020000;allpoints=g['output_points'];m.region(obj,bytes(512));m.region(xs,struct.pack('<'+str(len(allpoints))+'i',*(p[0] for p in allpoints)));m.region(ys,struct.pack('<'+str(len(allpoints))+'i',*(p[1] for p in allpoints)));m.region(STACK-1024,bytes(2048));m.reg('esp',STACK);m.word(STACK+0x14,value=ys)
  for name,value in [('esi',obj),('edi',xs),('ebx',n),('ecx',ys),('edx',0)]:m.reg(name,value)
  mt=r['metrics'];m.word(obj+0xe4,value=mt['1C']);m.word(obj+0xe0,value=mt['18']);pc=0x8c0f7d
  for _ in range(30):
   if pc==0x8c0fa8:break
   pc=m.apply(ins[pc])
  assert pc==0x8c0fa8;left,top=signed(m.word(obj+0xe4)),signed(m.word(obj+0xe0));advance=(allpoints[n+1][0]-allpoints[n][0])<<10;matches=[]
  cohort='control' if row['context']=='fresh' else 'preceded'
  for p in sorted((ROOT/'target/source-ui/font-cvt14-order-counterfactual-v1').glob(cohort+'-*/attempt-*/parsed.json')):
   native=json.loads(p.read_text());ng=next(ng for ng in native['glyphs'] if ng['glyph_index']==row['gid']);ap=p.parent/'active'/f"glyph-{ng['ordinal']:02d}-{row['gid']:03d}.alpha";checks=dict(alpha=bytes.fromhex(r['alpha_hex'])==ap.read_bytes(),metrics=(mt['0C']-mt['08'],mt['14']-mt['10'],mt['20'],left,top)==(ng['width'],ng['height'],ng['pitch'],signed(ng['offset_e4_raw']),signed(ng['offset_e0_raw'])),advance=advance==ng['advance_d0_raw'],cvt=h['cvt']==ng['hint']['cvt_s32'],storage=h['storage']==ng['hint']['storage_s32']);assert all(checks.values()),(row,checks);matches.append(dict(parsed=str(p.relative_to(ROOT)),parsed_sha256=sha(p),alpha=str(ap.relative_to(ROOT)),alpha_sha256=sha(ap),checks=checks))
  assert len(matches)==4;rp=out/f"{row['context']}-gid{row['gid']}-raster.json";rp.write_text(json.dumps(r,indent=2)+'\n');rows.append(dict(**row,raster_sha256=sha(rp),width=mt['0C']-mt['08'],height=mt['14']-mt['10'],origin=[left,top],native=matches))
 report=dict(schema='font0-persistent-gamma-source-native-comparison-v2',source_sha256=seq.base.SOURCE_SHA256,tool_sha256=sha(__file__),sequence_tool_sha256=sha(seq.__file__),source_manifest_sha256=sha(source/'manifest.json'),new_native_execution=False,rust_execution=False,native_observations=12,rows=rows,scope='Two actualsourcecontexts [354] and[342,354],freshsourcePREP once percontext; original hint+raster+origin replay vs retained order-only native intervention. Full point/GSnativeobservation,all7contextdependentglyphs,livecacheorder andtextlayout remainopen.')
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS',[(r['context'],r['gid'],r['height'],r['before_cvt14'],r['after_cvt14']) for r in rows],sha(out/'manifest.json'))
if __name__=='__main__':main(sys.argv[1],sys.argv[2])
