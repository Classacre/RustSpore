"""Pin original MDRP/SLOOP source and independent WORD loop boundary oracle."""
import hashlib,json,struct,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
ROOT=Path(__file__).resolve().parent.parent
def sha(b):return hashlib.sha256(b).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));front=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text());entries=[0x8ae410,0x8ac6e0,0x8ad030,0x8ad0a0,0x8ad310,0x8ad530];reports=[]
 for entry in entries:
  f=next(f for f in front['functions'] if f['entry']==f'{entry:08X}');raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16)) for a,b in f['ranges']);assert sha(raw)==f['sha256'];(out/f'{entry:08X}.bin').write_bytes(raw);rows=[r for a,b in f['ranges'] for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))];(out/f'{entry:08X}.asm').write_text('\n'.join(f'{r.address:08X} {r.bytes.hex()} {r.mnemonic} {r.op_str}' for r in rows)+'\n');reports.append(dict(entry=f'{entry:08X}',ranges=f['ranges'],sha256=sha(raw)))
 assert all(struct.unpack('<I',im.read(0x1432f30+op*4,4))[0]==0x8ae410 for op in range(0xc0,0xe0));assert struct.unpack('<I',im.read(0x1432f30+0x17*4,4))[0]==0x8ac6e0;assert struct.unpack('<I',im.read(0x8ac714+7*4,4))[0]==0x8ac70b
 assert im.read(0x8ac70b,6)==bytes.fromhex('496689483ec3');lines=[];counts={}
 for low in range(65536):
  for upper in (0,0x12340000,0xffff0000):
   value=upper|low;ecx=(value-1)&0xffffffff;word=ecx&65535;sf=(word>>15)&1;count=0 if sf else word+1;lines.append(f'{value:08X}\t{word:04X}\t{count}');counts[count]=counts.get(count,0)+1
 blob=('\n'.join(lines)+'\n').encode();(out/'sloop-original-word-reference.tsv').write_bytes(blob)
 report=dict(schema='font0-mdrp-sloop-original-source-proof-v1',source_sha256=SOURCE_SHA256,functions=reports,dispatch_family='All32 C0..DF table slots target8AE410;17targets8AC6E0 and jumpindex7 selects8AC70B.',MDRP='Admitted ordinary equal-axis non-twilight designword projection+sourceunits scale; singlewidth branch disabled by requiredgraphicsDWORD4=0; bit4round invokes callback with literalgray0,bit8minimumusesunrounded sign,bit10setsRP0,RP1oldRP0/RP2point always. Lowcolorbits0/1unused by handler.',SLOOP='DECfullECX thenstoreCXframe3E. SHP8AD030/IP8AD310/ALIGNRP8AD530 test signedWORD, execute0ifnegative elsecount+1, thenresetWORD0; lowword1..32768 yields1..32768,0and32769..65535 yield0; upperignored. API requires sufficientownedstack before iteration.',word_reference_rows=len(lines),word_reference_sha256=sha(blob),native_executed=False,rust_executed=False,missing_REA_queries=[])
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(lines),sha(blob))
if __name__=='__main__':main(sys.argv[1])
