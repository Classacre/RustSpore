"""Original state/stack handlers and bounded contour-start instruction fixtures."""
import hashlib,io,json,re,struct,sys,textwrap
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_hint_prep_probe import PrepMachine,STACK,FRAME,STATE,OPERANDS,signed
from spui_font_rectilinear_fixtures_v3 import emit_raster
ROOT=Path(__file__).resolve().parent.parent
def sha(b):return hashlib.sha256(b).hexdigest()
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));front=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text());entries=[0x8aea70,0x8addd0,0x8adf40,0x8aca00,0x8aca30,0x8acae0,0x8ae120,0x8adf70,0x8aba30,0x8aba40,0x8c8000];ins={};functions=[]
 for e in entries:
  f=next(f for f in front['functions'] if f['entry']==f'{e:08X}');raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16)) for a,b in f['ranges']);assert sha(raw)==f['sha256'];(out/f'{e:08X}.bin').write_bytes(raw);rs=[r for a,b in f['ranges'] for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))];ins.update({r.address:r for r in rs});(out/f'{e:08X}.asm').write_text('\n'.join(f'{r.address:08X} {r.bytes.hex()} {r.mnemonic} {r.op_str}' for r in rs)+'\n');functions.append(dict(entry=f'{e:08X}',ranges=f['ranges'],sha256=sha(raw)))
 switches=[]
 for op,tbase,ibase,sub,want in [(0x53,0x8ae0a4,0x8ae0e0,0x50,0x8adfdb),(0x64,0x8ae1a8,0x8ae1c4,0x56,0x8ae183)]:
  ix=im.read(ibase+op-sub,1)[0];target=struct.unpack('<I',im.read(tbase+4*ix,4))[0];assert target==want;switches.append(dict(opcode=op,index=ix,target=f'{target:08X}'))
 lines=[];cases=[]
 for op,entry,stacks in [(0x20,0x8aca00,[[0],[-1],[0x7fffffff]]),(0x23,0x8aca30,[[1,2],[-1,0x7fffffff]]),(0x8a,0x8acae0,[[1,2,3],[-1,0,0x7fffffff]]),(0x64,0x8ae120,[[x] for x in [-2147483648,-65,-1,0,1,65,2147483647]]),(0x53,0x8adf70,[[a,b] for a in [-2147483648,-1,0,1,2147483647] for b in [-2147483648,-1,0,1,2147483647]]),(0x5e,0x8aea70,[[x] for x in [0,1,65535,65536,-1,0x12345678]]),(0x4d,0x8adf40,[[]])]:
  for values in stacks:
   m=PrepMachine(ins);m.region(STACK-64,bytes(256));m.region(FRAME,bytes(256));m.region(STATE,bytes(256));m.region(OPERANDS,bytes(256));m.region(0x8ae000,im.read(0x8ae000,0x300),False);m.reg('esp',STACK);m.word(STACK,value=0xffffffff);m.word(STACK+4,value=FRAME);m.word(FRAME+4,value=op);m.word(FRAME+0x28,value=STATE);m.word(FRAME+0x20,value=OPERANDS+4*len(values))
   for i,v in enumerate(values):m.word(OPERANDS+4*i,value=v)
   m.run(entry,0xffffffff);after=[signed(m.word(at)) for at in range(OPERANDS,m.word(FRAME+0x20),4)];cases.append(dict(opcode=op,input=values,output=after,delta_base=m.word(STATE+0x82,2),auto_flip=m.word(STATE+0x8a,1)));lines.append(f'{op:02X}\t'+','.join(map(str,values))+'\t'+','.join(map(str,after))+f'\t{m.word(STATE+0x82,2)}\t{m.word(STATE+0x8a,1)}')
 raw=('\n'.join(lines)+'\n').encode();(out/'handler-original-reference.tsv').write_bytes(raw)
 header=(ROOT/'target/source-ui/font-glyph-native-v2/native-font-payload.h').read_text().split('font_payload[117628] = {',1)[1].split('};',1)[0];fontdata=bytes(int(x,16) for x in re.findall(r'0x([0-9a-fA-F]{2})',header));from fontTools.ttLib import TTFont
 font=TTFont(io.BytesIO(fontdata));f74=font.getTableData('fpgm')[1332:1345];assert f74.hex()=='208a2349648a2353583c1b2159'
 import spui_font_visible_raster_scout as base
 text=(ROOT/'tools/spui_font_simple_raster_census_v1.py').read_text();setup=text.split(' text=(ROOT/',1)[1].split('native={}',1)[0];env=dict(ROOT=ROOT,base=base);exec(textwrap.dedent(' text=(ROOT/'+setup),env);execute=env['execute'];image,frontier,rin=env['image'],env['frontier'],env['ins']
 shapes=[('half_x',[[32,0],[160,0],[160,128],[32,128]],[1,1,1,1]),('half_y',[[0,32],[128,32],[128,160],[0,160]],[1,1,1,1]),('first_half_last_off',[[32,0],[160,0],[160,128],[32,128]],[1,1,1,0]),('previous_off',[[32,0],[160,0],[160,128],[32,128],[32,64]],[1,1,1,0,1]),('previous_equal',[[32,0],[160,0],[160,128],[32,128],[32,64]],[1,1,1,1,1]),('first_off_last_on',[[64,192],[0,128],[0,0],[128,0],[128,128]],[0,1,1,1,1]),('both_off',[[0,128],[64,192],[128,128],[128,0],[0,0]],[0,1,0,1,0]),('both_off_half',[[32,160],[96,224],[160,160],[160,32],[32,32]],[0,1,0,1,0])];geometry=[];expected=[]
 for key,pts,flags in shapes:
  ends=[len(pts)-1];geometry.append(key+'\t'+';'.join(','.join(map(str,p)) for p in pts)+'\t'+','.join(map(str,flags))+'\t'+','.join(map(str,ends)))
  for d in (0,1):
   r=execute(image,frontier,rin,pts,ends,flags,d,0xa5);assert r['blocker'] is None,(key,r['blocker']);(out/f'{key}-d{d}.json').write_text(json.dumps(r,indent=2)+'\n');expected+=emit_raster(f'{key}:d{d}',r)
 (out/'geometry.tsv').write_text('\n'.join(geometry)+'\n');blob=('\n'.join(expected)+'\n').encode();(out/'geometry-expected.tsv').write_bytes(blob);report=dict(schema='font0-simple-state-original-proof-v1',source_sha256=SOURCE_SHA256,functions=functions,handler_rows=len(lines),handler_reference_sha256=sha(raw),switches=switches,function74=dict(offset=1332,length=13,bytes=f74.hex(),font_sha256=sha(fontdata)),opcode49='DEC49->48;TESTDL1zero branches8ADE46:currentfirst-minus-second projected byframe48. Design branchbelongs48,not49.',contour_geometry_cases=len(shapes),contour_raster_cases=16,geometry_expected_rows=len(expected),geometry_expected_sha256=sha(blob),native_executed=False,rust_executed=False,missing_REA_queries=[]);(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(lines),len(expected),sha(blob))
if __name__=='__main__':main(sys.argv[1])
