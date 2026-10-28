"""Original instruction replay of source rounding; controls/MIRP/DP3 source pinning."""
import hashlib,json,struct,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_hint_prep_probe import PrepMachine,STACK,signed
ROOT=Path(__file__).resolve().parent.parent
def sha(b):return hashlib.sha256(b).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));front=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text());entries=[0x8ac650,0x8ac790,0x8ab710,0x8ab740,0x8ab680,0x8ae590,0x8aebe0,0x8aeab0];ins={};functions=[]
 for entry in entries:
  f=next(f for f in front['functions'] if f['entry']==f'{entry:08X}');raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16)) for a,b in f['ranges']);assert sha(raw)==f['sha256'];(out/f'{entry:08X}.bin').write_bytes(raw);rows=[r for a,b in f['ranges'] for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))];ins.update({r.address:r for r in rows});(out/f'{entry:08X}.asm').write_text('\n'.join(f'{r.address:08X} {r.bytes.hex()} {r.mnemonic} {r.op_str}' for r in rows)+'\n');functions.append(dict(entry=f'{entry:08X}',sha256=sha(raw),ranges=f['ranges']))
 assert all(struct.unpack('<I',im.read(0x1432f30+op*4,4))[0]==0x8ae590 for op in range(0xe0,0x100))
 switches=[]
 for op,callback in [(0x18,0x8ab710),(0x19,0x8ab740),(0x3d,0x8ab680)]:
  ix=im.read(0x8ac7fc+op-0x18,1)[0];target=struct.unpack('<I',im.read(0x8ac7e0+ix*4,4))[0];r=ins[target];assert r.mnemonic=='mov' and r.operands[1].imm==callback;switches.append(dict(opcode=op,index=ix,target=f'{target:08X}',callback=f'{callback:08X}'))
 lines=[]
 for callback in [0x8ab710,0x8ab740,0x8ab680]:
  for value in range(-8192,8193):
   m=PrepMachine(ins);m.region(STACK-32,bytes(128));m.reg('esp',STACK);m.word(STACK,value=0xffffffff);m.word(STACK+4,value=0);m.word(STACK+8,value=value);m.word(STACK+12,value=0);m.run(callback,0xffffffff);assert m.reg('esp')==STACK+4;lines.append(f'{callback:08X}\t{value}\t{signed(m.reg("eax"))}')
 raw=('\n'.join(lines)+'\n').encode();(out/'round-original-instruction-reference.tsv').write_bytes(raw);report=dict(schema='font0-simple-controls-original-proof-v1',source_sha256=SOURCE_SHA256,functions=functions,round_switches=switches,round_reference_rows=len(lines),round_reference_sha256=sha(raw),round_compensation=0,round_distance_domain=[-8192,8192],MIRP='All32E0..FFslots share608B8AE590. Ordinaryzone/equalaxis only; singlewidth disabled graphics4=0; autoflip requiredgraphics50=1. Lowcolorbitsunused; callback receivesliteralcompensation0.',DELTAP3='8AEBF9 adds32inDX;8AEAF3 MOVSXWORDbase;ppem15targetsubtract with unsigned0..15test. Successor mirrors wrapped signedWORD base for all3delta groups.',SMD='8AC660 literalDWORDstore state6C=graphics20. No saturation/sign invention.',native_executed=False,rust_executed=False,shape_policy='No relaxation yet: zero-program source bypass may skip VM phantom initialization; point>64 API/raster bounds remain explicit pending separate proof.',missing_REA_queries=[]);(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(lines),sha(raw))
if __name__=='__main__':main(sys.argv[1])
