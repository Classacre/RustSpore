"""Fresh original-instruction D/P/U hint/raster targets plus owned translations."""
import hashlib,json,sys,subprocess,textwrap
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from spui_font_rectilinear_fixtures_v3 import emit_hint,emit_raster
import spui_font_visible_raster_scout as base
CASES=[(37,68),(49,80),(54,85)]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 out=Path(output).resolve();out.relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 hints=out/'fresh-hint';subprocess.run([sys.executable,str(ROOT/'tools/spui_font_curved_hint_reference_v1.py'),str(hints)],check=True,cwd=ROOT,stdout=subprocess.DEVNULL)
 text=(ROOT/'tools/spui_font_simple_raster_census_v1.py').read_text();setup=text.split(" text=(ROOT/",1)[1].split("native={}",1)[0];env=dict(ROOT=ROOT,base=base);exec(textwrap.dedent(' text=(ROOT/'+setup),env);execute=env['execute'];image,frontier,ins=env['image'],env['frontier'],env['ins']
 census=json.loads((ROOT/'target/source-ui/font-simple-native-census-v5/manifest.json').read_text());lines=[];cases=[]
 for gid,code in CASES:
  key=f'gid{gid}';hp=hints/f'{key}-hint.json';h=json.loads(hp.read_text());g=h['glyph'];n=g['real_points'];baseline=json.loads((ROOT/f'target/source-ui/font-simple-census-v3/{key}-hint.json').read_text());assert g['output_points']==baseline['glyph']['output_points'] and h['cvt']==baseline['cvt'] and h['storage']==baseline['storage']
  data=bytes.fromhex(g['data_hex']);flags=list(data[(n+4)*20:(n+4)*21])[:n];lines.append(f'{key}\tmapping\t{code:04X}\t{gid}');lines+=emit_hint(key,h);rasters=[];native=next(r['observations'] for r in census['rows'] if r['gid']==gid)
  for shift in [False,True]:
   points=[[x+(64 if shift else 0),y+(-128 if shift else 0)] for x,y in g['output_points'][:n]]
   for dropout,poison in [(0,0xa5),(1,0)]:
    r=execute(image,frontier,ins,points,g['contour_ends'],flags,dropout,poison);assert r['blocker'] is None;rk=key+(':shift' if shift else '')+f':d{dropout}';rp=out/f'{key}-{"shift-" if shift else ""}raster-d{dropout}.json';rp.write_text(json.dumps(r,indent=2)+'\n');lines+=emit_raster(rk,r)
    if not shift:
     for obs in native:
      ap=ROOT/obs['alpha'];assert sha(ap)==obs['alpha_sha256'] and bytes.fromhex(r['alpha_hex'])==ap.read_bytes()
     raw=json.loads((ROOT/native[0]['parsed']).read_text());ng=next(v for v in raw['glyphs'] if v['glyph_index']==gid);m=r['metrics'];assert(m['0C']-m['08'],m['14']-m['10'],m['20'],m['1C'],m['18'])==(ng['width'],ng['height'],ng['pitch'],ng['offset_e4_raw'],ng['offset_e0_raw'])
    rasters.append(dict(key=rk,path=rp.name,sha256=sha(rp),owned_translation=[64,-128] if shift else [0,0],dropout=dropout,poison=poison,native_observations=0 if shift else 4))
  cases.append(dict(gid=gid,codepoint=f'{code:04X}',hint=str(hp.relative_to(out)),hint_sha256=sha(hp),raster=rasters,native=native))
 expected=out/'expected.tsv';expected.write_bytes(('\n'.join(lines)+'\n').encode());report=dict(schema='font0-simple-curved-source-fixtures-v2',source_sha256=base.SOURCE_SHA256,tool_sha256=sha(__file__),hint_tool_sha256=sha(ROOT/'tools/spui_font_curved_hint_reference_v1.py'),frozen_raster_template_sha256=sha(ROOT/'tools/spui_font_visible_raster_scout.py'),rows=len(lines),canonical_sha256=sha(expected),cases=cases,actual_glyphs=3,owned_translations=3,raster_cases=12,distinct_retained_native_observations=12,new_native_execution=False,rust_execution=False,scope='Fresh font0 actual D/P/U15ppem source hint and original quadratic/line/gray replay; actual3glyphs match12retainednativefresh observations. Three derived translations are ownedgeometry/source-only,notfont/ppem claims. Full native points/GS,actualUIcache/lifetime,broaderRusttext remainopen.');(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(report['rows'],report['canonical_sha256'])
if __name__=='__main__':main(sys.argv[1])
