"""Canonical glyph0 outline reference and retained native scalar-state check."""
import argparse,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--reference',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--comparison',type=Path,required=True);p.add_argument('--native-root',type=Path,required=True);a=p.parse_args()
 for o in [a.out,a.comparison]:o.resolve().relative_to((ROOT/'target').resolve());assert not o.exists()
 r=json.loads(a.reference.read_text());g=r['glyph0'];assert r['blocker'] is None and r['final_stack_empty'] and r['cpu_nonvolatile_registers_restored'] and r['frame_cursor_restored']
 lines=[]
 for n,x in enumerate(r['trace']):assert x['stream']=='glyph0';lines.append(f"dispatch\t{n}\t{x['pc']}\t{x['opcode']:02X}\t{x['stack']}")
 for i,(design,original,point,touched) in enumerate(zip(g['design_points'],g['original_scaled_points'],g['output_points'],g['touched'])):lines.append('point\t'+str(i)+'\t'+'\t'.join(map(str,[*design,*original,*point,touched])))
 for field in ['cvt','storage']:
  for i,v in enumerate(r[field]):lines.append(f'{field}\t{i}\t{v}')
 state=bytes.fromhex(r['state_hex']);frame=bytes.fromhex(r['frame_hex']);refs=struct.unpack_from('<3i',frame,0x30)
 lines+=['graphics\t'+state[0x58:0x8c].hex(),'final\t0\t0\t'+'\t'.join(map(str,refs))]
 a.out.write_bytes(('\n'.join(lines)+'\n').encode());native=[]
 for path in sorted(a.native_root.glob('*/attempt-*/parsed.json')):
  data=json.loads(path.read_text())
  for obs in data['glyphs']:
   if obs['glyph_index']!=0:continue
   assert obs['controls']==dict(x87_cw=0x37f,mxcsr=0x1f80,direction_flag=0);h=obs['hint'];assert h['cvt_s32']==r['cvt'] and h['storage_s32']==r['storage']
   assert h['round_function']=='008AB710' and h['cvt_function']=='008CC230' and h['raw_flags_e0_e1']==0
   native.append(dict(path=str(path),parsed_sha256=sha(path),context=obs['context'],width=obs['width'],height=obs['height'],left=obs['offset_e4_raw'],top=obs['offset_e0_raw'],active_sha256=obs['active_sha256']))
 assert len(native)==8
 report=dict(schema='source-glyph0-outline-reference-v1',reference=str(a.reference),reference_sha256=sha(a.reference),canonical_sha256=sha(a.out),rows=len(lines),tool_sha256=sha(Path(__file__)),native_scalar_matches=native,new_native_execution=False,native_hinted_points_observed=False,scope='Hinted points/graphics/frame are source-instruction-only. Retained native comparisons cover CVT/storage/callbacks/flags; native pixel/metric records are references only, not modeled/equal pixels.')
 a.comparison.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='native_scalar_matches'}))
if __name__=='__main__':main()
