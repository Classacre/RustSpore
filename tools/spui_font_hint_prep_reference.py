"""Emit complete PREP state/trace reference and compare retained fresh contexts.

Inputs are saved source-instruction report and native parser reports; this tool
executes no source code, native host, compiler, or game process.
"""
import argparse,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--reference',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--comparison',type=Path,required=True);p.add_argument('--native-root',type=Path,required=True);args=p.parse_args()
 for output in [args.out,args.comparison]:output.resolve().relative_to((ROOT/'target').resolve());assert not output.exists()
 r=json.loads(args.reference.read_text());assert r['blocker'] is None and r['final_stack_empty'] and r['cpu_nonvolatile_registers_restored'] and r['frame_cursor_restored']
 lines=[]
 for n,x in enumerate(r['trace']):lines.append(f"dispatch\t{n}\t{0 if x['stream']=='prep' else 1}\t{x['pc']}\t{x['opcode']:02X}\t{x['stack']}")
 for field in ['cvt','storage']:
  for n,x in enumerate(r[field]):lines.append(f'{field}\t{n}\t{x}')
 state=bytes.fromhex(r['state_hex']);frame=bytes.fromhex(r['frame_hex']);refs=struct.unpack_from('<3i',frame,0x30)
 lines+=['graphics\t'+state[0x58:0x8c].hex(),'twilight\t'+r['twilight_state_hex'],f"final\t40\t0\t1\t{refs[0]}\t{refs[1]}\t{refs[2]}"]
 args.out.write_bytes(('\n'.join(lines)+'\n').encode())
 cases=[]
 for path in sorted(args.native_root.glob('*/attempt-*/parsed.json')):
  d=json.loads(path.read_text());assert d['success']
  for context in d['contexts']:
   assert context['ppem']==[15,15];assert context['controls']==dict(x87_cw=0x37f,mxcsr=0x1f80,direction_flag=0);h=context['hint'];assert h['cvt_s32']==r['cvt'] and h['storage_s32']==r['storage']
   assert h['round_function']==f"{struct.unpack_from('<I',state,0x70)[0]:08X}" and h['cvt_function']==f"{struct.unpack_from('<I',state,0x98)[0]:08X}" and h['raw_flags_e0_e1']==int.from_bytes(state[0xc8:0xca],'little')
  cases.append(dict(path=str(path),sha256=sha(path),contexts=len(d['contexts'])))
 assert len(cases)==80 and sum(c['contexts'] for c in cases)==2352
 report=dict(schema='font0-prep-source-instruction-retained-native-comparison-v2',reference=str(args.reference),reference_sha256=sha(args.reference),canonical_sha256=sha(args.out),rows=len(lines),tool_sha256=sha(Path(__file__)),new_native_execution=False,cases=cases,contexts=sum(c['contexts'] for c in cases),matches_per_context=dict(cvt=173,storage=47,round_pointer=1,cvt_pointer=1,flags=1),scope='Fresh context only. Full graphics/frame/twilight state is source-instruction-only, not emitted by native host. No Rust or pixel acceptance.')
 args.comparison.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
if __name__=='__main__':main()
