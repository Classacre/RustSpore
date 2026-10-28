"""Read-only symbolic source SSE expression check: exact binary grouping, no reassociation."""
import sys,json,struct
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'tools'))
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder

def check(image):
 rows=list(decoder().disasm(image.read(0x9b69a0,0xc10),0x9b69a0));from capstone.x86 import X86_OP_REG,X86_OP_MEM,X86_OP_IMM
 reg={'edi':0,'ebp':0x500000,'esp':0x700000,'esi':0x600000,'ecx':0x610000,'ebx':0x620000};mem={};xmm={'xmm7':('const',0x80000000)};seen=[]
 fields={0x10c:'cx',0x110:'cy',0x114:'cz',0x118:'x',0x11c:'y',0x120:'z',0x124:'w',0x138:'hx',0x13c:'hy',0x140:'hz'}
 def node(op,a,b=None):
  if b is None:return(op,a)
  if op in ('mul','add') and repr(a)>repr(b):a,b=b,a
  return(op,a,b)
 def address(r,o):return o.mem.disp+(reg[r.reg_name(o.mem.base)] if o.mem.base else 0)+(reg[r.reg_name(o.mem.index)]*o.mem.scale if o.mem.index else 0)
 def read(r,o):
  if o.type==X86_OP_REG:
   n=r.reg_name(o.reg);return xmm[n] if n.startswith('xmm') else reg[n]
  if o.type==X86_OP_IMM:return o.imm
  a=address(r,o)
  if a in mem:return mem[a]
  if a-0x500000 in fields:return('input',fields[a-0x500000])
  if a in (0x14719e4,0x13eb8b0):return('const',struct.unpack('<I',image.read(a,4))[0])
  return('opaque',a)
 def write(r,o,v):
  if o.type==X86_OP_REG:
   n=r.reg_name(o.reg);(xmm if n.startswith('xmm') else reg)[n]=v
  else:mem[address(r,o)]=v
 for r in rows:
  op=r.mnemonic;args=r.operands
  if r.address>=0x9b7583:break
  if op in ('movss','movaps','mov'):write(r,args[0],read(r,args[1]))
  elif op in ('mulss','addss','subss'):write(r,args[0],node({'mulss':'mul','addss':'add','subss':'sub'}[op],read(r,args[0]),read(r,args[1])))
  elif op=='xorps':
   a,b=read(r,args[0]),read(r,args[1]);assert b==('const',0x80000000);write(r,args[0],node('neg',a))
  elif op=='imul':write(r,args[0],read(r,args[1])*read(r,args[2]))
  elif op=='add':write(r,args[0],read(r,args[0])+read(r,args[1]))
  elif op=='comiss':seen +=[read(r,args[0]),read(r,args[1])]
  else:assert op in ('ja','jbe'),(hex(r.address),op)
 I=lambda s:('input',s);mul=lambda a,b:node('mul',a,b);add=lambda a,b:node('add',a,b);sub=lambda a,b:node('sub',a,b);neg=lambda a:node('neg',a)
 x,y,z,w=map(I,['x','y','z','w']);nx,ny,nz=[neg(mul(a,a))for a in (x,y,z)]
 c=[[add(nz,ny),sub(mul(y,x),mul(z,w)),add(mul(z,x),mul(y,w))],[add(mul(y,x),mul(z,w)),add(nz,nx),sub(mul(z,y),mul(x,w))],[sub(mul(z,x),mul(y,w)),add(mul(z,y),mul(x,w)),add(ny,nx)]]
 h=list(map(I,['hx','hy','hz']));v=[h,[neg(h[0]),h[1],h[2]],[neg(h[0]),h[1],neg(h[2])],[h[0],h[1],neg(h[2])]];orders=[[[0,1,2],[1,0,2],[0,2,1]],[[1,2,0],[1,2,0],[2,1,0]],[[2,1,0],[2,1,0],[2,1,0]],[[0,1,2],[1,0,2],[0,1,2]]];expected=[]
 for pair,vec in enumerate(v):
  for axis in range(3):
   t=[mul(c[axis][j],vec[j])for j in range(3)];o=orders[pair][axis];rot=add(mul(add(add(t[o[0]],t[o[1]]),t[o[2]]),('const',0x40000000)),vec[axis]);center=I(['cx','cy','cz'][axis]);expected +=[sub(center,rot),add(rot,center)]
 missing=[i for i,e in enumerate(expected)if e not in seen];assert not missing,missing
 return dict(source_compares=len(seen)//2,paired_corner_axis_expressions=len(expected),all_model_binary_groupings_found=True,commutative_binary_operand_sort_only=True,no_associative_regrouping=True,source_iteration_instructions=len(rows))
if __name__=='__main__':print(check(SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))))
