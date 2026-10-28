"""Pinned retained original-instruction/native targets for all389 simple glyphs; no execution."""
import collections,hashlib,json,struct,sys
from pathlib import Path
from spui_font_curved_fixtures_v3 import emit_hint
from spui_font_rectilinear_fixtures_v3 import emit_raster
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
OPS={0x40,0x41,*range(0xb0,0xc0),0,1,0x10,0x11,0x12,0x21,0x43,0x54,0x58,0x1b,0x59,0x2b,0x2f,0x3f,0xd6,0xcd,0xc6,0xd4,0xed,0xfd,0xe4,0xf4,0xe6,0x32,0x33,0x39,0x3c,0x5f,0x5d,0x71,0x30,0x31}
OPS.update(range(0xc0,0x100));OPS.update([0x17,0x18,0x19,0x1a,0x3d,0x72])
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);cpath=ROOT/'target/source-ui/font-simple-native-census-v5/manifest.json';c=json.loads(cpath.read_text());rows=[];lines=[];counts=collections.Counter();pins={str(cpath.relative_to(ROOT)):sha(cpath)}
 for row in c['rows']:
  gid=row['gid'];key=f'gid{gid}';hp=ROOT/f'target/source-ui/font-simple-census-v3/{key}-hint.json';assert sha(hp)==row['hint_sha256'];h=json.loads(hp.read_text());g=h['glyph'];n=g['real_points'];bad=next((t for t in h['trace'] if t['opcode'] not in OPS),None);reason=(f"opcode_{bad['opcode']:02X}" if bad else 'shape' if not 4<=n<=64 or len(g['contour_ends'])>8 or not 0<g['program_bytes']<=512 else None)
  data=bytes.fromhex(g['data_hex']);flags=list(data[(n+4)*20:(n+4)*21])[:n];starts=[0]+[e+1 for e in g['contour_ends'][:-1]]
  if reason is None:
   if any(not(flags[i]&1) or any(v&63==32 for v in g['output_points'][i]) for i in starts):reason='raster_contour_start'
   elif any(abs(v)>8192 for p in g['output_points'][:n] for v in p):reason='raster_coordinate_bound'
  counts[reason or 'potential']+=1;pins[str(hp.relative_to(ROOT))]=sha(hp)
  try:hl=emit_hint(key,h)
  except AssertionError:hl=[];reason=reason or 'source_non_axis_final'
  if hl:
   numeric=[line for line in hl if '\tdispatch\t' not in line]
   at=next(i for i,line in enumerate(numeric) if '\tgraphics\t' in line)+1;numeric.insert(at,f'{key}\tloop_minus_one\t{struct.unpack_from("<H",bytes.fromhex(h["frame_hex"]),0x3e)[0]}');lines+=numeric
   for dropout,folder,field in [(0,'font-simple-raster-census-v1','raster_sha256'),(1,'font-simple-raster-census-v2','raster_dropout1_sha256')]:
    rp=ROOT/f'target/source-ui/{folder}/{key}-raster.json';assert sha(rp)==row[field];r=json.loads(rp.read_text());lines+=emit_raster(f'{key}:d{dropout}',r);raw=[r['metrics']['1C'],r['metrics']['18']];phantom=g['output_points'][n];final=[(raw[i]-phantom[i])&~63 for i in range(2)];assert final==row['final_origin'];lines.append(f'{key}:d{dropout}\tfinal_origin\t{final[0]}\t{final[1]}');pins[str(rp.relative_to(ROOT))]=sha(rp)
  for obs in row['observations']:
   assert all(obs['checks'].values()) and sha(ROOT/obs['alpha'])==obs['alpha_sha256'] and sha(ROOT/obs['parsed'])==obs['parsed_sha256']
  rows.append(dict(gid=gid,name=row['name'],first_static_exclusion=reason,first_missing_opcode=bad,real_points=n,program_bytes=g['program_bytes'],native_observations=len(row['observations']),expected_numeric_rows=bool(hl)))
 expected=out/'expected.tsv';expected.write_bytes(('\n'.join(lines)+'\n').encode());paths=[Path(__file__),ROOT/'crates/spore-bevy/examples/spui_font_simple_controls_batch_verify.rs.pending',ROOT/'crates/spore-core/src/spui_font_hint_simple_curved.rs',ROOT/'crates/spore-core/src/spui_font_hint_simple_loop.rs',ROOT/'crates/spore-core/src/spui_font_hint_simple_controls.rs',ROOT/'crates/spore-core/src/spui_font_raster_simple_curved.rs',ROOT/'crates/spore-core/src/spui_font_hint_rectilinear.rs',ROOT/'crates/spore-core/src/spui_font_raster_rectilinear.rs'];pins.update({str(p.relative_to(ROOT)):sha(p) for p in paths})
 report=dict(schema='font0-simple-controls-batch-source-fixtures-v1',simple_glyphs=len(rows),retained_native_observations=sum(r['native_observations'] for r in rows),composite_exclusions=183,empty_exclusions=6,static_frontier=dict(counts),rows=rows,canonical_rows=len(lines),canonical_sha256=sha(expected),pins=pins,native_executed=False,rust_executed=False,limits='Static necessary-condition frontier only; Rust typed first-stop determines actual support. Expected values are frozen original instruction/native references,not Rust. Historical glyphPC>100 stream/relativePC labels are unreliable; dispatch rows excluded,whole numeric points/graphics/CVT/storage/bitmap retained. Fresh15ppem font0 only; no sequence/cache/renderer/text closure. Final source origins subtract current first phantom thenANDFFFFFFC0; raw alpha unchanged.')
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(rows),sum(r['native_observations'] for r in rows),dict(counts),len(lines),sha(expected))
if __name__=='__main__':main(sys.argv[1])
