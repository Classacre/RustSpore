"""Independent static sparse-carrier linked audit; never execute target code."""
import argparse,hashlib,json,re,struct
from pathlib import Path
from cell_schema_probe import SourceImage
from native_crt_acos_probe import PE,decoder
import anim_root_pose_native_probe_v6 as source
BASE=0x590000;OLD=0x29000000
IMPORTS={'GetStdHandle','ReadFile','WriteFile','ExitProcess','GetModuleHandleA','GetLastError'}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def u32(b,o):return struct.unpack_from('<I',b,o)[0]
def roundup(n,a):return(n+a-1)&~(a-1)
def syms(p):
 d={}
 for line in p.read_text().splitlines():
  m=re.match(r'\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]+\s+(\S+)\s+([0-9A-Fa-f]{8})\b',line)
  if m:d[m[1]]=int(m[2],16)
 return d
class Image:
 def __init__(self,path):
  self.path=path;self.b=path.read_bytes();b=self.b;self.nt=u32(b,60);self.op=self.nt+24;o=self.op
  assert b[:2]==b'MZ' and b[self.nt:self.nt+4]==b'PE\0\0'
  assert struct.unpack_from('<H',b,self.nt+4)[0]==0x14c
  assert struct.unpack_from('<H',b,o)[0]==0x10b
  assert struct.unpack_from('<H',b,self.nt+20)[0]==224
  self.base=u32(b,o+28);self.image=u32(b,o+56);self.headers=u32(b,o+60)
  assert u32(b,o+32)==4096 and u32(b,o+36)==512 and self.headers<=4096
  assert u32(b,o+92)==16
  n=struct.unpack_from('<H',b,self.nt+6)[0];self.sections=[]
  for i in range(n):
   h=o+224+40*i;vs,rva,rs,fo=struct.unpack_from('<4I',b,h+8);f=u32(b,h+36)
   assert rva%4096==0 and fo%512==0 and rs%512==0 and fo>=self.headers and fo+rs<=len(b)
   assert vs>0 and vs<=rs and u32(b,h+24)==u32(b,h+28)==0
   self.sections.append(dict(name=b[h:h+8],va=self.base+rva,vs=vs,raw=fo,rs=rs,flags=f))
  ordered=sorted(self.sections,key=lambda s:s['va']);assert self.sections==ordered
  assert all(a['va']+roundup(a['vs'],4096)<=z['va'] for a,z in zip(ordered,ordered[1:]))
  disk=sorted(self.sections,key=lambda s:s['raw']);assert all(a['raw']+a['rs']==z['raw'] for a,z in zip(disk,disk[1:]))
  assert disk[0]['raw']==self.headers and disk[-1]['raw']+disk[-1]['rs']==len(b)
  assert self.image==roundup(ordered[-1]['va']+ordered[-1]['vs']-self.base,4096)
 def sec(self,a,n):return next(s for s in self.sections if s['va']<=a and a+n<=s['va']+s['vs'])
 def read(self,a,n):
  s=self.sec(a,n);return self.b[s['raw']+a-s['va']:s['raw']+a-s['va']+n]
 def perm(self,a,n):return self.sec(a,n)['flags']&0xe0000000
 def dirs(self):return[struct.unpack_from('<II',self.b,self.op+96+8*i) for i in range(16)]
 def imports(self):
  rva,size=self.dirs()[1];assert rva and size;cursor=self.base+rva;out={};edits=[]
  def string(a):
   chars=bytearray()
   while True:
    c=self.read(a,1)[0];a+=1
    if not c:return chars.decode('ascii')
    chars.append(c);assert len(chars)<256
  for _ in range(32):
   values=struct.unpack('<5I',self.read(cursor,20))
   if not any(values):break
   oft,stamp,chain,name,iat=values;assert oft and iat and stamp==chain==0
   assert string(self.base+name).lower()=='kernel32.dll'
   edits += [cursor,cursor+12,cursor+16]
   for i in range(64):
    a=self.base+oft+i*4;b=self.base+iat+i*4
    x=u32(self.read(a,4),0);y=u32(self.read(b,4),0);assert x==y
    if not x:break
    assert x&0x80000000==0
    out[b]=string(self.base+x+2);edits += [a,b]
   else:raise AssertionError('unterminated import table')
   cursor+=20
  else:raise AssertionError('unterminated descriptors')
  return out,edits

def audit(args):
 new,old=Image(args.carrier),Image(args.owned_exe);s=syms(args.owned_map);image=SourceImage(args.source_exe);proof=source.v3.source_audit(image)
 assert new.base==BASE and old.base==OLD
 assert struct.unpack_from('<H',new.b,new.nt+22)[0]&1
 assert not struct.unpack_from('<H',new.b,new.op+70)[0]&0x40
 assert new.b[:new.nt]==old.b[:old.nt], 'DOS header/stub preserved exactly; explicit native-page header exception'
 assert u32(new.b,new.op+64)==0
 assert all((rva,size)==(0,0) for i,(rva,size) in enumerate(new.dirs()) if i not in (1,12))
 ni,ne=new.imports();oi,oe=old.imports();assert set(ni.values())==set(oi.values())==IMPORTS and len(ni)==len(oi)==6
 assert ni==oi,'absolute IAT addresses and symbols unchanged'
 assert ne==oe,'absolute import metadata locations unchanged'
 assert new.base+u32(new.b,new.op+16)==s['HostMain']
 delta=OLD-BASE
 for i in(1,12):
  a,n=old.dirs()[i];assert new.dirs()[i]==(a+delta,n)
 # High sections byte-identical except independently enumerated import RVA DWORDs.
 high=[z for z in new.sections if z['va']>=OLD]
 assert len(high)==len(old.sections)
 for a,b in zip(old.sections,high):
  assert (a['name'],a['va'],a['vs'],a['rs'],a['flags'])==(b['name'],b['va'],b['vs'],b['rs'],b['flags'])
  expected=bytearray(old.read(a['va'],a['vs']))
  for at in oe:
   if a['va']<=at< a['va']+a['vs']:
    j=at-a['va'];struct.pack_into('<I',expected,j,u32(expected,j)+delta)
  assert new.read(b['va'],b['vs'])==expected
  assert new.b[b['raw']+b['vs']:b['raw']+b['rs']]==old.b[a['raw']+a['vs']:a['raw']+a['rs']]
 assert all(not(s['OwnedStart']<=a<s['OwnedEnd']) for a in oe)
 assert new.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart'])==old.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart'])
 # Independently construct all approved native pages and every gap byte.
 code={a&~0xffff for a,b in source.CODE}|{source.v3.EXIT&~0xffff}|{int(a,16)&~0xffff for a in proof['unreachable_calls']}
 data={int(r['address'],16)&~0xffff for r in proof['data']}|{a&~0xffff for a in source.v3.GLOBALS};assert not code&data
 pages={p:bytearray(b'\xcc'*65536 if p in code else bytes(65536)) for p in code|data}
 def put(a,b):pages[a&~0xffff][a&0xffff:(a&0xffff)+len(b)]=b
 for a,b in source.CODE:put(a,image.read(a,b-a))
 for r in proof['data']:put(int(r['address'],16),image.read(int(r['address'],16),r['bytes']))
 exitbytes=b'\xb8'+struct.pack('<I',s['OriginalDone'])+b'\xff\xe0';put(source.v3.EXIT,exitbytes)
 low=[z for z in new.sections if z['va']<OLD];assert len(low)==len(pages)
 for z in low:
  p=z['va']&~0xffff;skip=4096 if p==BASE else 0
  assert z['va']==p+skip and z['vs']==65536-skip
  assert z['flags']==(0x60000020 if p in code else 0x40000040)
  assert new.read(z['va'],z['vs'])==pages[p][skip:]
 assert new.read(s['CarrierExitBytes'],7)==exitbytes
 # All guard regions, buffers and shadows retain declared bounds/permissions.
 ext=[]
 for name,n in source.BUFFERS:
  at=s[name+'Buffer'];assert at-s[name+'Before']==16 and s[name+'After']-at==n
  for lo,count in [(at-16,n+32)]+([] if name in('Converted','Reply') else[(s[name+'Shadow'],n)]):
   assert new.perm(lo,count)==0xc0000000
   assert all(lo+count<=x or y<=lo for x,y in ext);ext.append((lo,lo+count))
  assert new.read(at-16,16)==new.read(at+n,16)==b'\xa5'*16
 # Owned control closure and import thunk resolution, including all native entries.
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_REG,X86_REG_EAX
 start,end=s['OwnedStart'],s['OwnedEnd'];rows=list(decoder().disasm(new.read(start,end-start),start))
 assert sum(r.size for r in rows)==end-start;bounds={r.address for r in rows};calls=[];entries=[]
 for j,r in enumerate(rows):
  if not(r.group(CS_GRP_CALL) or r.group(CS_GRP_JUMP)):continue
  op=r.operands[0]
  if op.type==X86_OP_REG:
   assert op.reg==X86_REG_EAX and rows[j-1].mnemonic=='mov' and rows[j-1].operands[1].type==X86_OP_IMM
   v=rows[j-1].operands[1].imm;assert(r.mnemonic,v) in [('call',0x9b0ca0),('jmp',0x99d74f)];entries.append(v);continue
  assert op.type==X86_OP_IMM
  if op.imm in bounds:continue
  assert r.mnemonic=='call';thunk=new.read(op.imm,6);assert thunk[:2]==b'\xff\x25';name=ni[u32(thunk,2)];calls.append((r.address,name))
 assert sorted(entries)==[0x99d74f,0x9b0ca0]
 module=next(a for a,n in calls if n=='GetModuleHandleA');read=next(a for a,n in calls if n=='ReadFile');assert s['HostMain']<module<read
 j=next(i for i,r in enumerate(rows) if r.address==module)
 assert rows[j+1].mnemonic=='cmp' and rows[j+1].operands[1].imm==BASE
 assert rows[j+2].mnemonic in('jne','jnz') and rows[j+2].operands[0].imm==s['HostFailure']
 assert sum(1 for r in rows if module<r.address<read and r.mnemonic=='repe cmpsb')==len(source.CODE)+len(proof['data'])+1
 return dict(schema='independent-sparse-root-pose-carrier-linked-audit-v1',static_pass=True,native_executed=False,carrier_sha256=sha(args.carrier),owned_sha256=sha(args.owned_exe),map_sha256=sha(args.owned_map),source_byte_exact=True,owned_instruction_byte_exact=True,source_pages_and_slack_exact=True,header_DOS_exception_explicit=True,imports_OFT_IAT_closed=True,unknown_RVA_directories_rejected=True,owned_control_closed=True,guards_closed=True,primary_base=f'{BASE:08X}',virtual_image_size=new.image,source_pages=len(pages),owned_sections=len(high),imports=sorted(IMPORTS),entry=f'{s["HostMain"]:08X}',limits='Static linked closure only. Root execution and numerical/protocol acceptance separate; actual actor/stage integration unsupported.')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--carrier',type=Path,required=True);p.add_argument('--owned-exe',type=Path,required=True);p.add_argument('--owned-map',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();report=audit(a)
 with a.out.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
