"""Prepare review-only relocated E5BE20/1041E40 plus E283A0; never execute."""
import argparse,hashlib,json
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32,CS_GRP_CALL,CS_GRP_JUMP
from capstone.x86 import X86_OP_MEM,X86_REG_INVALID
from cell_schema_probe import SourceImage,SOURCE_SHA256

CONSTANTS=[0x1478a7c,0x13f0e3c,0x13fdcb0,0x13fdcac,0x13ec5ac,0x13ec478,0x14861d0,0x13ec42c,0x13ec4a8,0x1486e24,0x149a808]
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output parent outside target')
 img=SourceImage(a.exe); ranges=[(0xe5be20,0xe5bf68),(0x1041e40,0x1041e57),(0xe283a0,0xe28471)]
 arrays=[img.read(s,e-s) for s,e in ranges]; const=[int.from_bytes(img.read(x,4),'little') for x in CONSTANTS]
 if const!=[0x43480000,0x43c80000,0x44160000,0x44480000,0x447a0000,0x3e4ccccd,0x3f800000,0x3ecccccd,0x3f19999a,0x3f4ccccd,0x37800000]:raise ValueError('constant mismatch')
 d=Cs(CS_ARCH_X86,CS_MODE_32);d.detail=True;patch=[];calls=[]; unpatched=[]; listings=[]
 for n,(start,end) in enumerate(ranges):
  ins=list(d.disasm(arrays[n],start))
  if not ins or ins[-1].address+ins[-1].size!=end:raise ValueError('incomplete decode')
  for i in ins:
   if i.group(CS_GRP_CALL):
    if n!=0 or i.operands[0].imm!=0x1041e40 or i.imm_size!=4:raise ValueError('unexpected call')
    calls.append(i.address-start+i.imm_offset)
   if i.group(CS_GRP_JUMP) and not start<=i.operands[0].imm<end:raise ValueError('external branch')
   for op in i.operands:
    if op.type==X86_OP_MEM and op.mem.base==X86_REG_INVALID and op.mem.index==X86_REG_INVALID:
     addr=op.mem.disp & 0xffffffff
     if n==2 and addr==0x13f4f98: unpatched.append(i.address);continue
     if n==0 and addr==0x16b4be4: index=0
     elif addr in CONSTANTS:index=CONSTANTS.index(addr)+1
     else:raise ValueError(f'unexpected absolute {addr:08x}')
     if i.disp_size!=4:raise ValueError('unexpected displacement size')
     patch.append((n,i.address-start+i.disp_offset,index))
   listings.append({'address':f'{i.address:08X}','bytes':i.bytes.hex(),'text':f'{i.mnemonic} {i.op_str}'})
 if len(calls)!=5 or unpatched!=[0xe283c5,0xe28436,0xe2845f] or len(patch)!=21:raise ValueError(f'unexpected patch census {len(calls)} {len(patch)} {unpatched}')
 a.out.mkdir()
 def arr(name,values):return 'static const unsigned char '+name+'[]={'+','.join(f'0x{x:02x}' for x in values)+'};\n'
 header='#pragma once\n'+''.join(arr('code'+str(i),v) for i,v in enumerate(arrays))
 header+='static const unsigned int constant_words[]={'+','.join(f'0x{x:08x}' for x in const)+'};\n'
 header+='static const unsigned int fixups[][3]={'+','.join('{%d,%d,%d}'%x for x in patch)+'};\n'
 header+='static const unsigned int call_offsets[]={'+','.join(map(str,calls))+'};\n'
 (a.out/'source_bytes.h').write_text(header,encoding='ascii')
 sources=[Path(__file__).with_name('cell_ui_input_host_review_v1.c'),Path(__file__).resolve().parent.parent/'crates/spore-core/examples/cell_ui_input_compare_v1.rs']
 manifest={'scope':'review only; no execution','source_sha256':SOURCE_SHA256,'ranges':[f'{s:08X}..{e-1:08X}' for s,e in ranges],
           'slice_sha256':[sha(v) for v in arrays], 'constants':[{ 'address':f'{x:08X}','bits':f'{y:08X}'} for x,y in zip(CONSTANTS,const)],
           'fixups':patch,'call_offsets':calls,'unpatched_absolute_addresses':[f'{x:08X}' for x in unpatched],
           'source_hashes':{str(x):sha(x.read_bytes()) for x in sources},'header_sha256':sha(header.encode()),'instructions':listings}
 (a.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf8')
 print('manifest',sha((a.out/'manifest.json').read_bytes()),'slice hashes',*[sha(x) for x in arrays])
if __name__=='__main__':main()
