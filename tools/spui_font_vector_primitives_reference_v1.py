"""Original integer vector primitive/handler instruction oracle; never native."""
import hashlib,json,random,struct,sys
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder
from spui_font_hint_prep_probe import PrepMachine,STACK,FRAME,OPERANDS,signed
ROOT=Path(__file__).resolve().parent.parent;ZONE=0x390000
ENTRIES=[0x8ac050,0x8ac100,0x8ac3c0,0x8d0eb0,0x8d0db0,0x8d0c70,0x8d0d50,0x8abe80,0x8ab870,0x8ab9d0,0x8aba00]
def sha(b):return hashlib.sha256(b).hexdigest()
def machine(ins):
 m=PrepMachine(ins);m.region(STACK-8192,bytes(16384));m.region(FRAME,bytes(256));m.region(OPERANDS,bytes(256));m.region(ZONE,bytes(4096));m.reg('esp',STACK);m.word(STACK,value=0xffffffff)
 for off,ptr in [(0,0x100),(4,0x200),(8,0x300),(12,0x400),(0x28,0x500)]:m.word(ZONE+off,value=ZONE+ptr)
 for off in (8,12,16):m.word(FRAME+off,value=ZONE)
 return m
def call(ins,entry,args):
 m=machine(ins)
 for i,v in enumerate(args):m.word(STACK+4+4*i,value=v)
 m.run(entry,0xffffffff,limit=10000);assert m.reg('esp')==STACK+4
 return signed(m.reg('eax'))
def dot(ins,p,f):
 m=machine(ins);m.reg('eax',FRAME)
 for off,v in zip((0x14,0x16,0x18,0x1a),[*p,*f]):m.word(FRAME+off,2,value=v)
 m.run(0x8abe80,0xffffffff);return m.word(FRAME+0x40,2)
def main(output):
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(exist_ok=False);im=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));front=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text());ins={};functions=[]
 for e in ENTRIES:
  f=next(f for f in front['functions'] if f['entry']==f'{e:08X}');raw=b''.join(im.read(int(a,16),int(b,16)-int(a,16)) for a,b in f['ranges']);assert sha(raw)==f['sha256'];(out/f'{e:08X}.bin').write_bytes(raw);rs=[r for a,b in f['ranges'] for r in decoder().disasm(im.read(int(a,16),int(b,16)-int(a,16)),int(a,16))];ins.update({r.address:r for r in rs});(out/f'{e:08X}.asm').write_text('\n'.join(f'{r.address:08X} {r.bytes.hex()} {r.mnemonic} {r.op_str}' for r in rs)+'\n');functions.append(dict(entry=f'{e:08X}',ranges=f['ranges'],sha256=sha(raw)))
 rng=random.Random(0x8d0eb0);rows=[]
 def emit(kind,args,result):rows.append(f'{kind}\t{len(rows)}\t'+','.join(map(str,args))+'\t'+','.join(map(str,result)))
 vals=[0,1,-1,65535,-65535,65536,-65536,16777216,-16777216,500003,-500003]
 pairs=[(a,b)for a in vals for b in vals]+[(rng.randint(-16777216,16777216),rng.randint(-16777216,16777216))for _ in range(128)]
 for a,b in pairs:
  if b:emit('D16',[a,b],[call(ins,0x8d0db0,[a,b])])
  emit('M16',[a,b],[call(ins,0x8d0c70,[a,b])]);emit('LEN',[a,b],[call(ins,0x8d0eb0,[a,b])])
 for a in vals:
  for b in [-32768,-16384,-11585,-1,0,1,11585,16384,32767]:emit('M14',[a,b],[call(ins,0x8d0d50,[a,b])])
 vectors=[[16384,0],[0,16384],[-16384,0],[0,-16384],[11585,11585],[-11585,11585],[0,0],[32767,-32768]]
 for p in vectors:
  for f in vectors:emit('DOT',[*p,*f],[dot(ins,p,f)])
 for _ in range(128):
  d=[rng.randint(-8192,8192),rng.randint(-8192,8192)];v=rng.choice(vectors);m=machine(ins)
  for off,val in zip((0x14,0x16),v):m.word(FRAME+off,2,value=val)
  for i,val in enumerate([FRAME,*d]):m.word(STACK+4+4*i,value=val)
  m.run(0x8ab9d0,0xffffffff);emit('PROJ',[*d,*v],[signed(m.reg('eax'))])
 for f in vectors:
  for p in vectors:
   dp=dot(ins,p,f)
   for delta in [-8192,-65,-1,0,1,65,8192]:
    m=machine(ins)
    for off,val in zip((0x18,0x1a),f):m.word(FRAME+off,2,value=val)
    m.word(FRAME+0x40,2,value=dp)
    for i,val in enumerate([FRAME,ZONE,0,delta]):m.word(STACK+4+4*i,value=val)
    m.run(0x8ab870,0xffffffff,limit=10000);emit('MOVE',[delta,*f,dp],[signed(m.word(ZONE+0x100)),signed(m.word(ZONE+0x200)),m.word(ZONE+0x500,1)])
 for op in (7,0x87):
  for dx,dy in [[0,0],[64,0],[0,64],[64,64],[-64,64],[16384,-12345],[-8192,-16384]]+[[rng.randint(-4096,4096),rng.randint(-4096,4096)]for _ in range(48)]:
   for f in vectors[:6]:
    odx,ody=dx+1 if dx<16384 else dx-1,dy-1 if dy> -16384 else dy+1;m=machine(ins);m.word(STACK+4,value=FRAME);m.word(FRAME+4,value=op);m.word(FRAME+0x20,value=OPERANDS+8);m.word(OPERANDS,value=0);m.word(OPERANDS+4,value=1)
    for off,val in zip((0x14,0x16,0x18,0x1a,0x1c,0x1e),(16384,0,*f,16384,0)):m.word(FRAME+off,2,value=val)
    for off,val in [(0x100,dx),(0x200,dy),(0x300,odx),(0x400,ody)]:m.word(ZONE+off,value=val)
    m.run(0x8ac050 if op==7 else 0x8ac100,0xffffffff,limit=10000)
    values=[signed(m.word(FRAME+off,2),16)for off in (0x14,0x16,0x1c,0x1e)]+[m.word(FRAME+0x40,2)]+[m.word(FRAME+off)for off in (0x44,0x48,0x4c)]
    emit('V07'if op==7 else'V87',[dx,dy,odx,ody,*f],values)
 blob=('\n'.join(rows)+'\n').encode();(out/'expected.tsv').write_bytes(blob);r=dict(schema='font0-vector-original-instruction-reference-v1',source_sha256=SOURCE_SHA256,functions=functions,rows=len(rows),canonical_sha256=sha(blob),native_executed=False,rust_executed=False,scope='Integerhelpers+whole07/87handler arithmetic only; notVMopcodeadmission/intersection/wholeglyphgate. Inputsownedbounded; no mathsqrt/FP/normalizationlibrary substitute.',missing_REA_queries=[]);(out/'manifest.json').write_text(json.dumps(r,indent=2)+'\n');print(len(rows),sha(blob))
if __name__=='__main__':main(sys.argv[1])
