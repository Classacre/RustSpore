"""Reproduce complete original-instruction visible-character hint/raster fixtures.

No native host/compiler/GPU is executed. Native pixel/state observations are
retained references; full source graphics/point structures are source-only.
"""
import hashlib,json,struct,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from spui_font_visible_raster_scout import load_source,execute_source
CASES=[(64,95),(93,124),(14,45),(42,73)]
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def emit_hint(key,report):
 g=report['glyph'];assert report['blocker'] is None and report['final_stack_empty'] and report['cpu_nonvolatile_registers_restored'] and report['frame_cursor_restored'];assert len(g['output_points'])==8
 lines=[f"{key}\tshape\t4\t1\t{g['program_bytes']}"]
 for n,t in enumerate(report['trace']):lines.append(f"{key}\tdispatch\t{n}\t{0 if t['stream']=='glyph0' else 1}\t{t['pc']}\t{t['opcode']:02X}\t{t['stack']}")
 for i,(design,original,point,touched) in enumerate(zip(g['design_points'],g['original_scaled_points'],g['output_points'],g['touched'])):lines.append(f'{key}\tpoint\t{i}\t'+'\t'.join(map(str,[*design,*original,*point,touched])))
 for field in ['cvt','storage']:
  for i,value in enumerate(report[field]):lines.append(f'{key}\t{field}\t{i}\t{value}')
 frame=bytes.fromhex(report['frame_hex']);refs=struct.unpack_from('<3i',frame,0x30);state=bytes.fromhex(report['state_hex'])
 lines+= [f'{key}\tgraphics\t'+state[0x58:0x8c].hex(),f"{key}\tfinal\t0\t0\t{refs[0]}\t{refs[1]}\t{refs[2]}\t{len(report['call_entries'])}",f"{key}\tadvance\t{(g['output_points'][5][0]-g['output_points'][4][0])<<10}"]
 return lines
def emit_raster(key,r):
 assert r['blocker'] is None;o=r['metrics'];lines=[f"{key}\tmetrics\t{o['0C']-o['08']}\t{o['14']-o['10']}\t{o['20']}\t{o['1C']}\t{o['18']}"]
 for kind in ['horizontal','vertical']:
  for row in r['crossings'][kind]:lines.append(f"{key}\t{kind}\t{row['row']}\t"+(','.join(map(str,row['events'])) or '-'))
 for i,value in enumerate(bytes.fromhex(r['alpha_hex'])):lines.append(f'{key}\talpha\t{i}\t{value}')
 return lines
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);image,frontier,ins=load_source();lines=[];cases=[];native_unique=set()
 for gid,code in CASES:
  key=f'gid{gid}';hint_path=out/f'{key}-hint.json'
  with (out/f'{key}-hint.log').open('wb') as log:subprocess.run([sys.executable,str(ROOT/'tools/spui_font_visible_hint_scout.py'),str(hint_path),str(gid)],stdout=log,stderr=subprocess.STDOUT,check=True,cwd=ROOT)
  hint=json.loads(hint_path.read_text());assert hint['glyph']['glyph_index']==gid;lines.append(f'{key}\tmapping\t{code:04X}\t{gid}');lines+=emit_hint(key,hint);points=hint['glyph']['output_points'][:4];rasters=[]
  for dropout,poison in [(0,0xa5),(1,0)]:
   r=execute_source(image,frontier,ins,points,dropout,poison);assert r['blocker'] is None;rpath=out/f'{key}-raster-d{dropout}.json';rpath.write_text(json.dumps(r,indent=2)+'\n');lines+=emit_raster(f'{key}:d{dropout}',r);native=[]
   for path in sorted((ROOT/'target/source-ui/font-glyph-corpus-v1/results').glob('*/attempt-*/parsed.json')):
    data=json.loads(path.read_text())
    if data['mode']!=0:continue
    for g in data['glyphs']:
     if g['glyph_index']!=gid:continue
     pixels=path.parent/'active'/f"glyph-{g['ordinal']:02d}-{gid:03d}.alpha";assert bytes.fromhex(r['alpha_hex'])==pixels.read_bytes();o=r['metrics'];assert (o['0C']-o['08'],o['14']-o['10'],o['20'],o['1C'],o['18'])==(g['width'],g['height'],g['pitch'],g['offset_e4_raw'],g['offset_e0_raw'])
     assert hint['cvt']==g['hint']['cvt_s32'] and hint['storage']==g['hint']['storage_s32'];assert g['advance_d0_raw']==(hint['glyph']['output_points'][5][0]-hint['glyph']['output_points'][4][0])<<10
     native_unique.add(str(pixels));native.append(dict(path=str(pixels.relative_to(ROOT)),sha256=sha(pixels),parsed_sha256=sha(path)))
   assert len(native)==4;rasters.append(dict(dropout=dropout,poison=poison,path=rpath.name,sha256=sha(rpath),native=native))
  cases.append(dict(glyph=gid,codepoint=f'{code:04X}',hint_path=hint_path.name,hint_sha256=sha(hint_path),raster=rasters));print(key,flush=True)
 expected=out/'expected.tsv';expected.write_bytes(('\n'.join(lines)+'\n').encode());assert len(native_unique)==16
 report=dict(schema='four-visible-original-font0-fixtures-v1',tool_sha256=sha(__file__),hint_tool_sha256=sha(ROOT/'tools/spui_font_visible_hint_scout.py'),raster_tool_sha256=sha(ROOT/'tools/spui_font_visible_raster_scout.py'),rows=len(lines),canonical_sha256=sha(expected),cases=cases,native_fresh_observations=16,new_native_execution=False,rust_execution=False,scope='Four actualfont0characters _|-I at fresh15ppem. Exact retained nativepixels/metrics/advance/CVT/storage. Full hintedpoints/GS/frame are source-instruction-only. No otherglyph/font/text/UI acceptance.')
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
if __name__=='__main__':main(sys.argv[1])
