"""Original empty-program helper bypass proof, preserving historical census."""
import hashlib,json,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_hint_prep_probe import PrepMachine,STACK
ROOT=Path(__file__).resolve().parent.parent
def sha(b):return hashlib.sha256(b).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));f=next(x for x in json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text())['functions'] if x['entry']=='008CC5C0');raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16))for a,b in f['ranges']);assert sha(raw)==f['sha256'];(out/'source.bin').write_bytes(raw);rs=[r for a,b in f['ranges'] for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))];ins={r.address:r for r in rs};(out/'source.asm').write_text('\n'.join(f'{r.address:08X} {r.bytes.hex()} {r.mnemonic} {r.op_str}'for r in rs)+'\n');cases=[]
 for ptr,length in [(0,0),(0x400000,0),(0x400000,-1),(0,1)]:
  m=PrepMachine(ins);m.region(STACK-256,bytes(512));ctx=0x300000;glyph=0x310000;m.region(ctx,b'\xa5'*1024);m.region(glyph,b'\xa5'*256);m.word(glyph+0x64,value=ptr);m.word(glyph+0x68,value=length);before=[m.bytes(ctx,1024),m.bytes(glyph,256)];m.reg('esp',STACK);m.word(STACK,value=0xffffffff);m.word(STACK+4,value=ctx);m.word(STACK+8,value=glyph)
  sentinels=dict(ebx=0x12345678,ebp=0x87654321,ecx=0x34567890,esi=0x456789ab,edi=0x56789abc)
  for k,v in sentinels.items():m.reg(k,v)
  m.run(0x8cc5c0,0xffffffff);assert m.reg('esp')==STACK+4 and [m.bytes(ctx,1024),m.bytes(glyph,256)]==before and all(m.reg(k)==v for k,v in sentinels.items());cases.append(dict(pointer=f'{ptr:08X}',length=length,steps=m.steps,all_context_and_glyph_bytes_unchanged=True,nonvolatile_and_saved_ecx_restored=True))
 r=dict(schema='font0-empty-program-helper-bypass-proof-v1',source_sha256=SOURCE_SHA256,source_entry=f['entry'],ranges=f['ranges'],source_sha256_range=sha(raw),cases=cases,actual_empty_simple_glyphs=[5,132,219,230],historical_limit='Frozen scout manually performs8CC743phantomgridlock andVMentry for allsimpleglyphs; fouremptyprogramfullhintphantom/GSrows are not actual8CC5C0emptybranch outputs. Historicalnativealpha/metrics/advance/CVT/storage observations remain unchanged andseparate. Newemptyfull-driverfixture must not fitoldphantomcanonical.',native_executed=False,rust_executed=False,scope='Exact immediate8CC5C0helper return only. Caller/fullglyphpostmetrics/preexistingGS/cache/CVTlifetimenot inferred. No existingreference oracceptedmodule changed.');(out/'manifest.json').write_text(json.dumps(r,indent=2)+'\n');print(sha((out/'manifest.json').read_bytes()))
if __name__=='__main__':main(sys.argv[1])
