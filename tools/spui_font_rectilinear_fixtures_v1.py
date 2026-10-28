"""Reproduce complete original-instruction visible-character hint/raster fixtures.

No native host/compiler/GPU is executed. Native pixel/state observations are
retained references; full source graphics/point structures are source-only.
"""
import hashlib,json,struct,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from spui_font_visible_raster_scout import load_source,execute_source
CASES=[(38,69),(39,70),(41,72),(45,76),(53,84)]
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def emit_hint(key,report):
 g=report['glyph'];assert report['blocker'] is None and report['final_stack_empty'] and report['cpu_nonvolatile_registers_restored'] and report['frame_cursor_restored'];assert len(g['output_points'])==g['real_points']+4
 lines=[f"{key}\tshape\t{g['real_points']}\t{len(g['contour_ends'])}\t{g['program_bytes']}"]
 for n,t in enumerate(report['trace']):lines.append(f"{key}\tdispatch\t{n}\t{0 if t['stream']=='glyph0' else 1}\t{t['pc']}\t{t['opcode']:02X}\t{t['stack']}")
 for i,(design,original,point,touched) in enumerate(zip(g['design_points'],g['original_scaled_points'],g['output_points'],g['touched'])):lines.append(f'{key}\tpoint\t{i}\t'+'\t'.join(map(str,[*design,*original,*point,touched])))
 for field in ['cvt','storage']:
  for i,value in enumerate(report[field]):lines.append(f'{key}\t{field}\t{i}\t{value}')
 frame=bytes.fromhex(report['frame_hex']);refs=struct.unpack_from('<3i',frame,0x30);state=bytes.fromhex(report['state_hex'])
 lines+= [f'{key}\tgraphics\t'+state[0x58:0x8c].hex(),f"{key}\tfinal\t0\t0\t{refs[0]}\t{refs[1]}\t{refs[2]}\t{len(report['call_entries'])}",f"{key}\tadvance\t{(g['output_points'][g['real_points']+1][0]-g['output_points'][g['real_points']][0])<<10}"]
 return lines
def emit_raster(key,r):
 assert r['blocker'] is None;o=r['metrics'];lines=[f"{key}\tmetrics\t{o['0C']-o['08']}\t{o['14']-o['10']}\t{o['20']}\t{o['1C']}\t{o['18']}"]
 for kind in ['horizontal','vertical']:
  for row in r['crossings'][kind]:lines.append(f"{key}\t{kind}\t{row['row']}\t"+(','.join(map(str,row['events'])) or '-'))
 for i,value in enumerate(bytes.fromhex(r['alpha_hex'])):lines.append(f'{key}\talpha\t{i}\t{value}')
 return lines
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 census=ROOT/'target/source-ui/font-simple-native-census-v5/manifest.json';source=json.loads(census.read_text());lines=[];cases=[]
 for gid,code in CASES:
  row=next(r for r in source['rows'] if r['gid']==gid);assert row['status']=='pixel_metrics_state_exact'
  hp=ROOT/f'target/source-ui/font-simple-census-v3/gid{gid}-hint.json';assert sha(hp)==row['hint_sha256'];hint=json.loads(hp.read_text());key=f'gid{gid}'
  assert all(all(o['checks'].values()) for o in row['observations']);lines.append(f'{key}\tmapping\t{code:04X}\t{gid}');lines+=emit_hint(key,hint)
  rasters=[]
  for dropout,folder,pin in [(0,'font-simple-raster-census-v1','raster_sha256'),(1,'font-simple-raster-census-v2','raster_dropout1_sha256')]:
   rp=ROOT/f'target/source-ui/{folder}/gid{gid}-raster.json';assert sha(rp)==row[pin];r=json.loads(rp.read_text());assert [r['metrics']['1C'],r['metrics']['18']]==row['final_origin'];lines+=emit_raster(f'{key}:d{dropout}',r);rasters.append(dict(dropout=dropout,path=str(rp.relative_to(ROOT)),sha256=sha(rp)))
  cases.append(dict(gid=gid,codepoint=f'{code:04X}',hint=str(hp.relative_to(ROOT)),hint_sha256=sha(hp),raster=rasters,native=row['observations']))
 expected=out/'expected.tsv';expected.write_bytes(('\n'.join(lines)+'\n').encode())
 report=dict(schema='rectilinear-font0-source-fixtures-v1',tool_sha256=sha(__file__),census_sha256=sha(census),rows=len(lines),canonical_sha256=sha(expected),cases=cases,new_native_execution=False,rust_execution=False,scope='Five actual E/F/H/L/T fresh15ppem; canonical projections of frozen source instruction census,20 retainednativefresh observations. Regenerate upstream source census using versioned tools before rerunning this emitter for fresh instruction replay; no generalizedRustacceptance.')
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(report['rows'],report['canonical_sha256'])
if __name__=='__main__':main(sys.argv[1])
