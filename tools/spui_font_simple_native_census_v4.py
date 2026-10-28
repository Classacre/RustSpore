"""Close source driver bitmap-origin policy over retained simple-glyph census.
No native execution. Replays original8C0F7D..0FA8 after documented EDX=0.
"""
import hashlib,json,struct,sys
from pathlib import Path
from collections import Counter
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from spui_font_visible_raster_scout import load_source
from spui_font_visible_hint_scout import PrepMachine
from spui_font_scaling_probe import STACK,signed

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 image,frontier,ins=load_source();start,stop=0x8c0f7d,0x8c0fa8;raw=image.read(start,stop-start)
 hints=ROOT/'target/source-ui/font-simple-census-v2';rasters=ROOT/'target/source-ui/font-simple-raster-census-v1';native={}
 for p in sorted((ROOT/'target/source-ui/font-glyph-corpus-v1/results').glob('*/attempt-*/parsed.json')):
  j=json.loads(p.read_text())
  if j['mode']!=0:continue
  for g in j['glyphs']:native.setdefault(g['glyph_index'],[]).append((p,g))
 rows=[]
 for row in json.loads((hints/'manifest.json').read_text())['rows']:
  if row['status']!='hint_complete':continue
  gid=row['gid'];hp=hints/row['report'];rp=rasters/f'gid{gid}-raster.json';h=json.loads(hp.read_text());r=json.loads(rp.read_text());old=json.loads((ROOT/'target/source-ui/font-simple-census-v1'/row['report']).read_text());assert h['glyph']['output_points']==old['glyph']['output_points'] and h['cvt']==old['cvt'] and h['storage']==old['storage']
  points=h['glyph']['output_points'];count=h['glyph']['real_points'];m=PrepMachine(ins);obj,xs,ys=0x10000000,0x10010000,0x10020000
  m.region(obj,bytes(0x200));m.region(xs,struct.pack('<'+str(len(points))+'i',*(p[0] for p in points)));m.region(ys,struct.pack('<'+str(len(points))+'i',*(p[1] for p in points)));m.region(STACK-1024,bytes(2048));m.reg('esp',STACK);m.word(STACK+0x14,value=ys)
  for name,v in [('esi',obj),('edi',xs),('ebx',count),('ecx',ys),('edx',0)]:m.reg(name,v)
  m.word(obj+0xe4,value=r['metrics']['1C']);m.word(obj+0xe0,value=r['metrics']['18']);pc=start;trace=[]
  while pc!=stop:
   assert len(trace)<30;trace.append(f'{pc:08X}');pc=m.apply(ins[pc])
  left,top=signed(m.word(obj+0xe4)),signed(m.word(obj+0xe0));advance=(points[count+1][0]-points[count][0])<<10;observations=[]
  for p,g in native.get(gid,[]):
   ap=p.parent/'active'/f"glyph-{g['ordinal']:02d}-{gid:03d}.alpha";mt=r['metrics'];checks=dict(alpha=bytes.fromhex(r['alpha_hex'])==ap.read_bytes(),metrics=(mt['0C']-mt['08'],mt['14']-mt['10'],mt['20'],left,top)==(g['width'],g['height'],g['pitch'],signed(g['offset_e4_raw']),signed(g['offset_e0_raw'])),advance=advance==g['advance_d0_raw'],cvt=h['cvt']==g['hint']['cvt_s32'],storage=h['storage']==g['hint']['storage_s32'])
   observations.append(dict(parsed=str(p.relative_to(ROOT)),parsed_sha256=sha(p),alpha=str(ap.relative_to(ROOT)),alpha_sha256=sha(ap),checks=checks))
  assert len(observations)==4
  rows.append(dict(gid=gid,name=row['name'],status='pixel_metrics_state_exact' if all(all(o['checks'].values()) for o in observations) else 'difference',hint_sha256=sha(hp),raster_sha256=sha(rp),points=count,contours=row['contours'],source_steps=h['glyph']['source_steps'],dispatches=row['dispatches'],source_post_metrics_trace=trace,raw_origin=[r['metrics']['1C'],r['metrics']['18']],current_phantom_origin=points[count],final_origin=[left,top],advance16dot16=advance,observations=observations))
 manifest=dict(schema='font0-simple-source-native-census-v4',tool_sha256=sha(__file__),source_sha256=h['source_sha256'],font_sha256=h['font_sha256'],hint_manifest_sha256=sha(hints/'manifest.json'),raster_manifest_sha256=sha(rasters/'manifest.json'),metric_source_range=[f'{start:08X}',f'{stop:08X}'],metric_source_sha256=hashlib.sha256(raw).hexdigest(),metric_entry_contract='Source8C0F60 EDX=0; ESI glyph result, EDI current X array, ECX current Y array, EBX real point count; stack14 current Y array; raw bitmap E4/E0 copied8C0EAE/B1,8C0EA5/A8; source aligns origins after subtracting first current phantom',new_native_execution=False,rust_execution=False,whole_font_glyphs=578,simple_glyphs=389,composite_excluded=183,empty_excluded=6,native_observations=sum(len(r['observations']) for r in rows),counts=dict(Counter(r['status'] for r in rows)),scope='Retained original font0 fresh15x15ppem native mode0; grayscale3 curve2 dropout0 owned bounded raster allocations; no native intermediate point/full graphics observation; composites,other scales,sequence/layout remain excluded',rows=rows)
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(manifest['counts']);print('observations',manifest['native_observations'])
if __name__=='__main__':main(sys.argv[1])
