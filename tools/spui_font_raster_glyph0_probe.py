"""Read-only original scan-converter instruction probe, no native execution.

Uses source glyph0 hinted points and bounded owned buffers. Provider allocator
and memset are explicit model boundaries, not native allocation acceptance.
"""
import hashlib,json,struct,sys
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path[:0]=[str(ROOT/'tools'),str(ROOT/'target/rea/capstone-runtime')]
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from cell_schema_probe import SourceImage,SOURCE_SHA256
from spui_font_hint_glyph0_probe import PrepMachine
from spui_font_scaling_probe import STACK,signed
class RasterMachine(PrepMachine):
 def __init__(self,ins):super().__init__(ins);self.allocations=[];self.next_allocation=0x40000000;self.providers=[];self.visited=set();self.entry_counts=Counter();self.source_entries=set();self.poison=0xa5
 def allocate(self,size):
  assert 0<size<1000000
  at=self.next_allocation;self.next_allocation+=(size+4095)&~4095
  self.region(at,bytes([self.poison])*size);self.allocations.append(dict(address=f'{at:08X}',bytes=size));return at
 def condition(self,name):
  if name=='jg':assert all(self.flags[x] is not None for x in ['z','s','o'])
  return super().condition(name)
 def apply(self,ins):
  self.visited.add(ins.address)
  if ins.address in self.source_entries:self.entry_counts[f'{ins.address:08X}']+=1
  if ins.mnemonic=='call':
   target=self.operand(ins,ins.operands[0]);sp=self.reg('esp');next_pc=ins.address+ins.size
   if target in [0x8D0BB0,0x8D0940]:
    size=self.word(sp+4);at=self.allocate(size);self.reg('eax',at);self.providers.append(dict(call=f'{ins.address:08X}',target=f'{target:08X}',size=size));self.steps+=1;return next_pc
   if target==0x11E052E:
    dst,value,count=[self.word(sp+i*4) for i in range(3)];assert count<1000000;self.bytes(dst,count,bytes([value&255])*count);self.reg('eax',dst);self.providers.append(dict(call=f'{ins.address:08X}',target='memset',bytes=count));self.steps+=1;return next_pc
  if ins.mnemonic in ['rep stosd','rep stosb','rep stosw']:
   width=4 if ins.mnemonic.endswith('d') else 2 if ins.mnemonic.endswith('w') else 1;count=self.reg('ecx');assert count<1000000
   self.bytes(self.reg('edi'),count*width,(self.reg('eax')&((1<<(width*8))-1)).to_bytes(width,'little')*count);self.reg('edi',self.reg('edi')+count*width);self.reg('ecx',0);self.steps+=1;return ins.address+ins.size
  if ins.mnemonic=='adc':
   a=self.operand(ins,ins.operands[0]);b=self.operand(ins,ins.operands[1]);bits=ins.operands[0].size*8;assert self.flags['c'] is not None;mask=(1<<bits)-1;sign=1<<(bits-1);a&=mask;b&=mask;total=a+b+int(self.flags['c']);value=total&mask;self.operand(ins,ins.operands[0],value);self.flags=dict(z=value==0,s=bool(value&sign),c=total>mask,o=bool(~(a^b)&(a^value)&sign));self.steps+=1;return ins.address+ins.size
  if ins.mnemonic=='sbb':
   a=self.operand(ins,ins.operands[0]);b=self.operand(ins,ins.operands[1]);bits=ins.operands[0].size*8;assert self.flags['c'] is not None;mask=(1<<bits)-1;sign=1<<(bits-1);a&=mask;b&=mask;total=a-b-int(self.flags['c']);value=total&mask;self.operand(ins,ins.operands[0],value);self.flags=dict(z=value==0,s=bool(value&sign),c=total<0,o=bool((a^b)&(a^value)&sign));self.steps+=1;return ins.address+ins.size
  return super().apply(ins)
def main(path,dropout,poison):
 image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));frontier=json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text());cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True;ins={}
 for row in frontier['functions']:
  raw=b''.join(image.read(int(a,16),int(b,16)-int(a,16)) for a,b in row['ranges']);assert hashlib.sha256(raw).hexdigest()==row['sha256']
  for a,b in row['ranges']:
   decoded=list(cs.disasm(image.read(int(a,16),int(b,16)-int(a,16)),int(a,16)));assert sum(x.size for x in decoded)==int(b,16)-int(a,16);ins.update({x.address:x for x in decoded})
 m=RasterMachine(ins);m.poison=poison;m.source_entries={int(row['entry'],16) for row in frontier['functions']};m.region(STACK-65536,bytes(131072));m.reg('esp',STACK)
 for base,virtual,stored,offset in image.sections:m.region(base,image.read(base,max(virtual,stored)),False)
 points=json.loads((ROOT/'target/source-ui/font-hint-glyph0-reference-v2.json').read_text())['glyph0']['output_points'][:8]
 MEMORY,STARTS,ENDS,XS,YS,FLAGS=0x10000000,0x10010000,0x10020000,0x10030000,0x10040000,0x10050000
 for at,raw in [(MEMORY,bytes(512)),(STARTS,struct.pack('<2h',0,4)),(ENDS,struct.pack('<2h',3,7)),(XS,struct.pack('<8i',*(p[0] for p in points))),(YS,struct.pack('<8i',*(p[1] for p in points))),(FLAGS,bytes([1]*8))]:m.region(at,raw)
 args=[MEMORY,2,STARTS,ENDS,XS,YS,FLAGS,3,2,dropout,dropout,0,0,0,0]
 for i,value in enumerate([0xffffffff]+args):m.word(STACK+i*4,value=value)
 sentinels={name:m.reg(name) for name in ['ebx','esi','edi','ebp']}
 pc=0x8C8310;phase='constructor';blocker=None;obj=None;crossings=None
 for _ in range(2000000):
  if pc==0xffffffff:
   if phase=='constructor':
    obj=m.reg('eax');assert m.reg('esp')==STACK+4 and all(m.reg(k)==v for k,v in sentinels.items());crossings={}
    for kind,base,low,high in [('horizontal',0xd4,0xdc,0xe0),('vertical',0x164,0x16c,0x170)]:
     rows=[]
     for row in range(signed(m.word(obj+low)),signed(m.word(obj+high))+1):
      node=m.word(m.word(obj+base)+row*4);values=[]
      while node:
       assert len(values)<512;values.append(signed(m.word(node)));node=m.word(node+4)
      rows.append(dict(row=row,events=values))
     crossings[kind]=rows
    phase='bitmap';m.reg('esp',STACK)
    for i,value in enumerate([0xffffffff,obj,0,0,0,0,0,0,0]):m.word(STACK+i*4,value=value)
    pc=0x8C86E0
   else:break
  try:pc=m.apply(ins[pc])
  except (AssertionError,KeyError) as ex:blocker=dict(phase=phase,pc=f'{pc:08X}',instruction=f'{ins[pc].mnemonic} {ins[pc].op_str}' if pc in ins else None,error=str(ex));break
 report=dict(schema='source-raster-glyph0-frontier-v1',source_sha256=SOURCE_SHA256,dropout=dropout,allocator_poison=poison,constructor_args=args,source_entries=dict(m.entry_counts),crossings=crossings,tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),glyph_reference_sha256=hashlib.sha256((ROOT/'target/source-ui/font-hint-glyph0-reference-v2.json').read_bytes()).hexdigest(),source_steps=m.steps,providers=m.providers,allocations=m.allocations,object_address=None if obj is None else f'{obj:08X}',blocker=blocker,visited_source=[f'{v:08X}' for v in sorted(m.visited)],new_native_execution=False,scope='Original source hinted glyph0 points, grayscale3,quadratic2; explicit dropout flags input; original driver flag selection not yet closed')
 if obj is not None:
  report['object_hex']=m.bytes(obj,0x2238).hex();report['metrics']={f'{at:02X}':signed(m.word(obj+at)) for at in range(0,0x54,4)}
  if blocker is None:
   assert pc==0xffffffff and m.reg('esp')==STACK+4 and all(m.reg(k)==v for k,v in sentinels.items())
   bitmap=m.word(obj+0x24);pitch=m.word(obj+0x20);height=m.word(obj+0x14)-m.word(obj+0x10);raw=m.bytes(bitmap,pitch*height);report['alpha_hex']=raw.hex();report['alpha_sha256']=hashlib.sha256(raw).hexdigest()
 out=Path(path);out.resolve().relative_to((ROOT/'target').resolve());assert not out.exists();out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['object_hex','visited_source']}))
if __name__=='__main__':main(sys.argv[1],int(sys.argv[2]),int(sys.argv[3],0) if len(sys.argv)>3 else 0xa5)
