"""Compose retained actual font0 alpha with source DXT3 byte-alpha primitive.
Owned placement at0,0 in default512 page; no game atlas/GPU execution claim.
"""
import hashlib,json,sys
from pathlib import Path
from collections import Counter
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from spui_glyph_atlas_probe import native_clear,native_alpha8

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False)
 cp=ROOT/'target/source-ui/font-simple-native-census-v5/manifest.json';census=json.loads(cp.read_text());rows=[];alphas=Counter();nibbles=Counter();side=512;pitch=(side//4)*16
 for case in census['rows']:
  assert case['status']=='pixel_metrics_state_exact';obs=case['observations'][0];p=ROOT/obs['alpha'];assert sha(p)==obs['alpha_sha256'];raw=p.read_bytes();alphas.update(raw)
  parsed=json.loads((ROOT/obs['parsed']).read_text());g=next(g for g in parsed['glyphs'] if g['glyph_index']==case['gid']);assert g['width']<=side and g['height']<=side and g['pitch']==g['width']
  dst=bytearray((side//4)*pitch);native_clear(dst,side,pitch);native_alpha8(dst,pitch,0,0,g['width'],g['height'],raw,g['pitch'])
  for y in range(g['height']):
   for x in range(g['width']):
    pos=(y//4)*pitch+(x//4)*16+(y&3)*2;word=int.from_bytes(dst[pos:pos+2],'little');nibble=(word>>((x&3)*4))&15;assert nibble==raw[y*g['pitch']+x]>>4;nibbles[nibble]+=1
  assert all(dst[at+8:at+16]==bytes.fromhex('ffffffff00000000') for at in range(0,len(dst),16))
  rows.append(dict(gid=case['gid'],name=case['name'],source_alpha=str(p.relative_to(ROOT)),source_alpha_sha256=sha(p),width=g['width'],height=g['height'],source_pitch=g['pitch'],owned_placement=[0,0],full_cleared_page_sha256=hashlib.sha256(dst).hexdigest(),written_alpha_nibbles_exact=True,white_rgb_blocks_preserved=True))
 report=dict(schema='font0-simple-alpha-to-source-dxt3-v1',new_native_execution=False,gpu_execution=False,rust_execution=False,actual_runtime_atlas=False,source_census_sha256=sha(cp),atlas_source_model_sha256=sha(ROOT/'tools/spui_glyph_atlas_probe.py'),atlas_accepted_comparison_sha256=sha(ROOT/'target/source-ui/glyph-atlas-comparison-v1.json'),tool_sha256=sha(__file__),page_side=side,page_pitch=pitch,page_format=3,source_bits=8,source_alpha_min=min(alphas),source_alpha_max=max(alphas),stored_alpha_nibble_min=min(nibbles),stored_alpha_nibble_max=max(nibbles),alpha_histogram=dict(sorted(alphas.items())),nibble_histogram=dict(sorted(nibbles.items())),rows=rows,scope='389 actual retainedfreshfont0 simple-glyph alpha inputs composed with accepted original source DXT3 clear/write loop model at owned0,0 placement. No normalization. This is byte composition evidence, not actual live placement/cacheorder/shader sampling/GPU/screen acceptance.')
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(rows),report['source_alpha_max'],report['stored_alpha_nibble_max'],sha(out/'manifest.json'))
if __name__=='__main__':main(sys.argv[1])
