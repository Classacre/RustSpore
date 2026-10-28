"""Literal original instruction proof for isolated2E/05/0E/7D controls. No native."""
import hashlib,json,struct,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_vector_primitives_reference_v1 import machine,ZONE
from spui_font_hint_prep_probe import STACK,FRAME,OPERANDS,signed
ROOT=Path(__file__).resolve().parent.parent
ENTRIES=[0x8acb10,0x8abfe0,0x8ac3a0,0x8ac790,0x8ab870,0x8ab970,0x8ab9a0,0x8d0db0,0x8d0c70,0x8d0d50,0x8ab6b0]
def sha(b):return hashlib.sha256(b).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));front=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text());ins={};functions=[]
 for e in ENTRIES:
  f=next(x for x in front['functions']if int(x['entry'],16)==e);raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16))for a,b in f['ranges']);assert sha(raw)==f['sha256'];(out/f'{e:08X}.bin').write_bytes(raw)
  for a,b in f['ranges']:ins.update({r.address:r for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))})
  functions.append(dict(entry=f'{e:08X}',ranges=f['ranges'],sha256=sha(raw)))
 table=im.read(0x8ac7e0,130);(out/'round-table.bin').write_bytes(table);idx=table[0x1c+(0x7d-0x18)];assert struct.unpack_from('<I',table,idx*4)[0]==0x8ac7cf
 vectors=[[16384,0],[0,16384],[-16384,0],[0,-16384],[11585,11585],[-11585,11585],[0,0],[32767,-32768],[1023,1],[-1023,1]];rows=[]
 for op,e in [(5,0x8abfe0),(14,0x8ac3a0),(46,0x8acb10),(125,0x8ac790)]:
  for p in vectors:
   for f in vectors:
    for fast in ([False,True]if op==46 and f in vectors[:2]else[False]):
     m=machine(ins);m.region(0x3a1000,bytes(256));m.region(0x8ac7e0,table,False);m.word(STACK+4,value=FRAME);m.word(FRAME+4,value=op);m.word(FRAME+0x20,value=OPERANDS+4);m.word(OPERANDS,value=0);m.word(FRAME+0x28,value=0x3a1000)
     for off,v in zip([0x14,0x16,0x18,0x1a,0x1c,0x1e],[*p,*f,1234,-5678]):m.word(FRAME+off,2,value=v)
     m.word(FRAME+0x40,2,value=16384);m.word(FRAME+0x44,value=(0x8ab970 if f==vectors[0]else 0x8ab9a0)if fast else 0x8ab870);m.word(FRAME+0x48,value=0x8ab9d0);m.word(FRAME+0x4c,value=0x8aba00);m.word(0x3a1070,value=0x8ab710)
     for off,v in [(0x100,65),(0x200,-63)]:m.word(ZONE+off,value=v)
     m.run(e,0xffffffff,limit=10000);assert m.reg('esp')==STACK+4
     result=[signed(m.word(FRAME+a,2),16)for a in [0x14,0x16,0x18,0x1a,0x1c,0x1e]]+[m.word(FRAME+0x40,2)]+[m.word(FRAME+a)for a in [0x44,0x48,0x4c]]+[signed(m.word(ZONE+0x100)),signed(m.word(ZONE+0x200)),m.word(ZONE+0x500,1)]+[m.word(FRAME+a)for a in [0x30,0x34,0x38]]+[m.word(0x3a1070)]
     if op==5:assert result[:6]==[*p,16384,0,1234,-5678]and result[7]==0x8ab870 and result[8:10]==[0x8ab9d0,0x8aba00]
     if op==14:assert result[:6]==[*p,*p,1234,-5678]and result[6:10]==[16384,0x8ab870,0x8ab9d0,0x8aba00]
     if op==46:assert result[10:12]==[65,-63]and result[13:16]==[0,0,0]
     if op==125:assert result[-1]==0x8ab6b0
     rows.append('\t'.join(map(str,[op,*p,*f,int(fast),*result])))
 for d in [-2147483648,-8193,-8192,-65,-64,-63,-1,0,1,63,64,65,8192,8193,2147483647]:
  m=machine(ins)
  for i,v in enumerate([FRAME,d,0]):m.word(STACK+4+i*4,value=v)
  m.run(0x8ab6b0,0xffffffff);rows.append(f'round\t{d}\t{signed(m.reg("eax"))}')
 blob=('\n'.join(rows)+'\n').encode();(out/'expected.tsv').write_bytes(blob);report=dict(schema='font-general-positive-source-controls-reference-v1',source_sha256=SOURCE_SHA256,functions=functions,round_table_sha256=sha(table),rows=len(rows),canonical_sha256=sha(blob),tool_sha256=sha(Path(__file__).read_bytes()),native_executed=False,rust_executed=False,scope='Whole original handlers2E/05/0E/7D plus rounddown callback, owned vectors/initialfields; no syntheticexpectedfromRust. 2Ezero displacement still executes movement/touch;0Edot literal4000. Fullglyph candidate acceptance remains separate.');(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(rows),sha(blob))
if __name__=='__main__':main(sys.argv[1])
