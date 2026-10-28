"""Read-only source-instruction PREP path exploration. No native execution.

Decodes only digest-bound saved frontier ranges. Unimplemented instructions stop
positively; state admission is explicitly square pixels and a caller ppem.
"""
import hashlib,io,json,re,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path[:0]=[str(ROOT/'tools'),str(ROOT/'target/rea/capstone-runtime')]
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
from spui_font_scaling_probe import Machine,STACK,signed,coefficients
from spui_font_hint_program_probe import load_source,full_bootstrap
from fontTools.ttLib import TTFont
from spui_font_outline_probe import FACES
FRAME,STATE,PREP,FPGM,FUNCS,OPERANDS,STORAGE,CVT= [0x300000+i*0x10000 for i in range(8)]
class PrepMachine(Machine):
 def __init__(self,ins):
  super().__init__(ins);self.trace=[];self.call_entries=[]
 def condition(self,name):
  if name=='jg':return not self.flags['z'] and self.flags['s']==self.flags['o']
  return super().condition(name)
 def apply(self,ins):
  op,args=ins.mnemonic,ins.operands; nxt=ins.address+ins.size
  read=lambda n:self.operand(ins,args[n]);write=lambda n,v:self.operand(ins,args[n],v)
  if ins.address==0x8ABCD0 and self.word(FRAME+4)==0x3e:raise AssertionError('admitted scalar prefix ends before MIAP point handler')
  if ins.address==0x8ABCD0:
   cursor=self.word(FRAME)-1;stream='prep' if PREP<=cursor<PREP+457 else 'fpgm';base=PREP if stream=='prep' else FPGM
   self.trace.append(dict(stream=stream,pc=cursor-base,opcode=self.word(FRAME+4),handler=f'{read(0):08X}',stack=(self.word(FRAME+0x20)-OPERANDS)//4))
  if ins.address in [0x8AE833,0x8AE903]:self.call_entries.append(dict(start=self.word(self.reg('esp')+4)-FPGM,end=self.word(self.reg('esp')+8)-FPGM))
  if op in ['call','jmp']:
   self.steps+=1;target=read(0);assert target in self.instructions,f'undeclared transfer {target:08X}'
   if op=='call':self.reg('esp',self.reg('esp')-4);self.word(self.reg('esp'),value=nxt)
   return target
  if op=='nop':self.steps+=1;return nxt
  if op.startswith('set'):
   self.steps+=1;write(0,int(self.condition({'sete':'je','setne':'jne','setl':'jl','setle':'jle','setg':'jg','setge':'jge'}[op])));return nxt
  if op=='imul' and len(args)==3:
   self.steps+=1;v=signed(read(1))*signed(read(2));write(0,v);overflow=not -(1<<31)<=v<(1<<31);self.flags=dict(z=None,s=None,c=overflow,o=overflow);return nxt
  if op=='not':
   self.steps+=1;write(0,~read(0));return nxt
  if op=='cdq':
   self.steps+=1;self.reg('edx',0xffffffff if signed(self.reg('eax'))<0 else 0);return nxt
  if op=='idiv':
   self.steps+=1;a=signed((self.reg('edx')<<32)|self.reg('eax'),64);b=signed(read(0));assert b;v=abs(a)//abs(b);v=-v if (a<0)!=(b<0) else v;assert -(1<<31)<=v<(1<<31);self.reg('eax',v);self.reg('edx',a-v*b);return nxt
  if op.startswith('cmov'):
   self.steps+=1
   if self.condition({'cmovl':'jl','cmovg':'jg'}[op]):write(0,read(1))
   return nxt
  if op=='and':
   self.steps+=1;a=read(0)&read(1);write(0,a);self.flags=dict(z=a==0,s=bool(a&(1<<(args[0].size*8-1))),c=False,o=False);return nxt
  return super().apply(ins)
def run(out):
 image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
 frontier=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text());cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True
 ins,tables,proofs=load_source(image)
 for row in frontier['functions']:
  raw=b''.join(image.read(int(a,16),int(b,16)-int(a,16)) for a,b in row['ranges'])
  assert hashlib.sha256(raw).hexdigest()==row['sha256']
  for a,b in row['ranges']:
   decoded=list(cs.disasm(image.read(int(a,16),int(b,16)-int(a,16)),int(a,16)))
   assert sum(x.size for x in decoded)==int(b,16)-int(a,16)
   ins.update({x.address:x for x in decoded})
 text=(ROOT/'target/source-ui/font-glyph-native-v2/native-font-payload.h').read_text().split('font_payload[117628] = {',1)[1].split('};',1)[0]
 fontdata=bytes(int(x,16) for x in re.findall(r'0x([0-9a-fA-F]{2})',text));assert len(fontdata)==117628;assert hashlib.sha256(fontdata).hexdigest()==FACES['511B49CA']
 font=TTFont(io.BytesIO(fontdata));prep=font.getTableData('prep');fpgm=font.getTableData('fpgm')
 # Bootstrap is independently source-interpreted, not manually guessed.
 setup=full_bootstrap(*load_source(image)[:2],fpgm,font['maxp'].maxFunctionDefs,font['maxp'].maxStackElements)
 m=PrepMachine(ins);m.region(STACK-8192,bytes(16384));frame=bytearray(0x100);state=bytearray(0x100)
 struct.pack_into('<I',frame,0,0xFEEDFACE);struct.pack_into('<I',frame,0x20,OPERANDS);struct.pack_into('<I',frame,0x28,STATE);struct.pack_into('<I',frame,0x50,0x8ABCA0);struct.pack_into('<I',frame,0x54,0x8ABAC0);struct.pack_into('<I',frame,0x58,0x8ABB10);struct.pack_into('<I',frame,0x2c,STATE+0x24)
 for offset,value in [(4,STORAGE),(8,CVT),(0x8c,0x1432f30),(0x90,FUNCS),(0x9c,PREP),(0xa0,FPGM)]:struct.pack_into('<I',state,offset,value)
 struct.pack_into('<H',state,0xc,15);state[0xb6]=1;state[0xc8]=1
 co,_,_=coefficients(ins,15,font['head'].unitsPerEm)
 for offset,value in [(0xa8,co['multiplier']),(0xac,co['denominator']),(0xa4,co['fixed_multiplier']),(0xb0,co['rounding_addend']),(0xb4,co['shift']),(0x98,[0x8CC230,0x8CC250,0x8CC2A0][co['selector']])]:struct.pack_into('<I',state,offset,value)
 struct.pack_into('<I',state,0x24+0x18,0x8AB710)
 for off,val in [(0x24,0x44),(0x24+0x14,64),(0x24+0xc,0x307c0)]:struct.pack_into('<I',state,off,val)
 for off,val in [(0x24+0x2a,9),(0x24+0x2c,3),(0x24+0x2e,128)]:struct.pack_into('<H',state,off,val)
 state[0x24+0x32]=1
 defs=bytearray(font['maxp'].maxFunctionDefs*8)
 for f in setup['functions']:struct.pack_into('<IHH',defs,f['index']*8,f['offset'],f['length'],1)
 for at,raw in [(FRAME,frame),(STATE,state),(PREP,prep),(FPGM,fpgm),(FUNCS,defs),(OPERANDS,bytes(font['maxp'].maxStackElements*4)),(STORAGE,bytes(font['maxp'].maxStorage*4)),(CVT,bytes(len(font['cvt '].values)*4))]:m.region(at,raw)
 for base,virtual,stored,offset in image.sections:m.region(base,image.read(base,max(virtual,stored)),False)
 for index,value in enumerate(font['cvt '].values):
  m.reg('esp',STACK)
  for n,v in enumerate([0xffffffff,STATE,value]):m.word(STACK+n*4,value=v)
  m.run(m.word(STATE+0x98),0xffffffff,1000);m.word(CVT+index*4,value=m.reg('eax'))
 m.reg('esp',STACK)
 for n,v in enumerate([0xffffffff,FRAME,PREP,PREP+len(prep)]):m.word(STACK+n*4,value=v)
 pc=0x8ABCA0;blocker=None
 for _ in range(1000000):
  if pc==0xffffffff:break
  try:pc=m.apply(ins[pc])
  except (AssertionError,KeyError) as ex:blocker=dict(pc=f'{pc:08X}',instruction=f'{ins[pc].mnemonic} {ins[pc].op_str}' if pc in ins else None,error=str(ex));break
 report=dict(schema='source-instruction-prep-frontier-v1',source_sha256=SOURCE_SHA256,font_sha256=hashlib.sha256(fontdata).hexdigest(),prep_bytes=len(prep),prep_sha256=hashlib.sha256(prep).hexdigest(),admission='square pixel B6=1; ppem15; PREP C8=1; zero initial storage; CVT source coefficients and exact scale callback; defaults8CC0E9..8CC135; no native execution; incomplete point/zone state initialization',trace=m.trace,call_entries=m.call_entries,tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),frontier_sha256=hashlib.sha256((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_bytes()).hexdigest(),source_integer_base_sha256=hashlib.sha256((ROOT/'tools/spui_font_scaling_probe.py').read_bytes()).hexdigest(),fpgm_bootstrap_tool_sha256=hashlib.sha256((ROOT/'tools/spui_font_hint_program_probe.py').read_bytes()).hexdigest(),coefficients=co,body_definitions=[dict(index=f['index'],offset=f['offset'],length=f['length']) for f in setup['functions'] if any(e['start']==f['offset'] for e in m.call_entries)],storage=[signed(m.word(STORAGE+i*4)) for i in range(font['maxp'].maxStorage)],cvt=[signed(m.word(CVT+i*4)) for i in range(len(font['cvt '].values))],state_hex=m.bytes(STATE,256).hex(),frame_hex=m.bytes(FRAME,256).hex(),instruction_steps=m.steps,blocker=blocker,remaining_stack=[signed(m.word(p)) for p in range(OPERANDS,m.word(FRAME+0x20),4)])
 out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['trace','remaining_stack']}))
if __name__=='__main__':run(sys.argv[1])
