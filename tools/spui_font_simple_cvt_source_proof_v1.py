"""Source WCVTP axis-factor proof; no native execution or guessed CVT scaling."""
import hashlib,json,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_hint_prep_probe import PrepMachine,STACK,FRAME,STATE,OPERANDS
ROOT=Path(__file__).resolve().parent.parent;CVT=0x370000
def sha(b):return hashlib.sha256(b).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));fs=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text())['functions'];ins={};reports=[]
 for entry in [0x8adc60,0x8aba50]:
  f=next(f for f in fs if f['entry']==f'{entry:08X}');raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16))for a,b in f['ranges']);assert sha(raw)==f['sha256'];(out/f'{entry:08X}.bin').write_bytes(raw);rows=[r for a,b in f['ranges']for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))];ins.update({r.address:r for r in rows});(out/f'{entry:08X}.asm').write_text('\n'.join(f'{r.address:08X} {r.bytes.hex()} {r.mnemonic} {r.op_str}'for r in rows)+'\n');reports.append(dict(entry=f'{entry:08X}',ranges=f['ranges'],sha256=sha(raw)))
 lines=[]
 for vector in [(16384,0),(0,16384),(-16384,0),(0,-16384)]:
  for at in (0,14,172):
   for value in [-2147483648,-65536,-256,-1,0,1,63,64,65536,2147483647]:
    m=PrepMachine(ins);m.region(STACK-256,bytes(512));m.region(FRAME,bytes(256));m.region(STATE,bytes(256));m.region(OPERANDS,bytes(32));m.region(CVT,b'\xa5'*(173*4));m.reg('esp',STACK);m.word(STACK,value=0xffffffff);m.word(STACK+4,value=FRAME);m.word(FRAME+0x28,value=STATE);m.word(FRAME+0x20,value=OPERANDS+8);m.word(FRAME+0x14,2,value=vector[0]);m.word(FRAME+0x16,2,value=vector[1]);m.word(STATE+8,value=CVT);m.word(STATE+0xb8,value=65536);m.word(STATE+0xbc,value=65536);m.word(OPERANDS,value=at);m.word(OPERANDS+4,value=value);m.run(0x8adc60,0xffffffff);assert m.word(FRAME+0x20)==OPERANDS and m.reg('esp')==STACK+4 and m.word(CVT+at*4)==value&0xffffffff;expected=bytearray(b'\xa5'*(173*4));expected[at*4:at*4+4]=(value&0xffffffff).to_bytes(4,'little');assert m.bytes(CVT,173*4)==expected;lines.append(f'{vector[0]}\t{vector[1]}\t{at}\t{value}\t{m.word(CVT+at*4):08X}')
 blob=('\n'.join(lines)+'\n').encode();(out/'expected.tsv').write_bytes(blob);r=dict(schema='font0-ordinary-axis-WCVTP-source-proof-v1',source_sha256=SOURCE_SHA256,functions=reports,rows=len(lines),canonical_sha256=sha(blob),required_source_B8_BC=[65536,65536],native_executed=False,rust_executed=False,scope='Originalwhole44handler including8ABA50 for4axisdirections,indexboundsandDWORDvalues. Factor10000directstore branch only; no anisotropic/generalvector source-scale substitution. Calleroriginalfont0square15 provenance stillrequired.');(out/'manifest.json').write_text(json.dumps(r,indent=2)+'\n');print(len(lines),sha(blob))
if __name__=='__main__':main(sys.argv[1])
