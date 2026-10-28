"""Read-only general contour census adaptation of frozen source raster oracle."""
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import spui_font_visible_raster_scout as base

def main(source,out):
 source=Path(source);out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 text=(ROOT/'tools/spui_font_visible_raster_scout.py').read_text();body=text.split('def execute_source(',1)[1].split('\ndef main(',1)[0]
 body='def execute_source('+body
 body=body.replace('points,dropout,poison):','points,ends,flags,dropout,poison):')
 body=body.replace("struct.pack('<h',0)","struct.pack('<'+str(len(ends))+'h',*([0]+[e+1 for e in ends[:-1]]))").replace("struct.pack('<h',3)","struct.pack('<'+str(len(ends))+'h',*ends)")
 body=body.replace("struct.pack('<4i'","struct.pack('<'+str(len(points))+'i'").replace('bytes([1]*4)','bytes(flags)').replace(' assert len(points)==4',' assert len(points)==len(flags) and ends[-1]+1==len(points)').replace('args=[MEMORY,1,','args=[MEMORY,len(ends),')
 env=dict(base.__dict__);exec(body,env);execute=env['execute_source'];image,frontier,ins=base.load_source();native={}
 for p in sorted((ROOT/'target/source-ui/font-glyph-corpus-v1/results').glob('*/attempt-*/parsed.json')):
  n=json.loads(p.read_text())
  if n['mode']!=0:continue
  for g in n['glyphs']:native.setdefault(g['glyph_index'],[]).append((p,g))
 rows=[]
 for row in json.loads((source/'manifest.json').read_text())['rows']:
  if row['status']!='hint_complete':continue
  gid=row['gid'];hint=json.loads((source/row['report']).read_text());g=hint['glyph'];count=g['real_points']
  import spui_font_visible_hint_scout as hb,io,re
  from fontTools.ttLib import TTFont
  if 'font' not in locals():
   h=(ROOT/'target/source-ui/font-glyph-native-v2/native-font-payload.h').read_text().split('font_payload[117628] = {',1)[1].split('};',1)[0];font=TTFont(io.BytesIO(bytes(int(x,16) for x in re.findall(r'0x([0-9a-fA-F]{2})',h))))
  _,ends,flags=font['glyf'][g['name']].getCoordinates(font['glyf'])
  try:
   report=execute(image,frontier,ins,g['output_points'][:count],ends,list(flags),1,0xa5)
   matches=[]
   if report['blocker'] is None:
    for p,ng in native.get(gid,[]):
     alpha=p.parent/'active'/f"glyph-{ng['ordinal']:02d}-{gid:03d}.alpha";mt=report['metrics'];exact=bytes.fromhex(report['alpha_hex'])==alpha.read_bytes() and (mt['0C']-mt['08'],mt['14']-mt['10'],mt['20'],mt['1C'],mt['18'])==(ng['width'],ng['height'],ng['pitch'],ng['offset_e4_raw'],ng['offset_e0_raw']);matches.append(dict(parsed=str(p.relative_to(ROOT)),alpha_metrics_exact=exact,cvt_storage_exact=hint['cvt']==ng['hint']['cvt_s32'] and hint['storage']==ng['hint']['storage_s32']))
   dest=out/f'gid{gid}-raster.json';dest.write_text(json.dumps(report,indent=2)+'\n');rows.append(dict(gid=gid,name=g['name'],status='raster_first_stop' if report['blocker'] else 'pixel_exact' if matches and all(m['alpha_metrics_exact'] and m['cvt_storage_exact'] for m in matches) else 'raster_complete_difference',blocker=report['blocker'],report=dest.name,native=matches))
  except Exception as ex:rows.append(dict(gid=gid,name=g['name'],status='raster_model_stop',error=type(ex).__name__+': '+str(ex)))
  if gid%25==0:print(gid,flush=True)
 from collections import Counter
 manifest=dict(schema='font0-simple-raster-census-v2',new_native_execution=False,tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),hint_manifest_sha256=hashlib.sha256((source/'manifest.json').read_bytes()).hexdigest(),rows=rows,counts=dict(Counter(r['status'] for r in rows)));(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(manifest['counts'])
if __name__=='__main__':main(sys.argv[1],sys.argv[2])
