"""Prepare review-only 763BC0 hook gate bytes; never executes source."""
import argparse,hashlib,json
from pathlib import Path
from capstone import Cs,CS_ARCH_X86,CS_MODE_32,CS_GRP_CALL,CS_GRP_JUMP
from capstone.x86 import X86_OP_MEM,X86_REG_INVALID
from cell_schema_probe import SourceImage,SOURCE_SHA256
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('out',type=Path);a=p.parse_args()
 root=(Path(__file__).resolve().parent.parent/'target').resolve(strict=True)
 if not a.out.parent.resolve(strict=True).is_relative_to(root):raise ValueError('output parent outside target')
 image=SourceImage(a.exe);start,end=0x763bc0,0x763c18;raw=image.read(start,end-start);d=Cs(CS_ARCH_X86,CS_MODE_32);d.detail=True
 ins=list(d.disasm(raw,start));glob=[];calls=[]
 if not ins or ins[-1].address+ins[-1].size!=end:raise ValueError('incomplete decode')
 for i in ins:
  if i.group(CS_GRP_JUMP) and not start<=i.operands[0].imm<end:raise ValueError('external branch')
  if i.group(CS_GRP_CALL):
   if i.operands[0].imm!=0x11f04a0 or i.imm_size!=4:raise ValueError('unexpected call')
   calls.append(i.address-start+i.imm_offset)
  for op in i.operands:
   if op.type==X86_OP_MEM and op.mem.base==X86_REG_INVALID and op.mem.index==X86_REG_INVALID:
    if op.mem.disp!=0x15fe918 or i.disp_size!=4:raise ValueError('unexpected absolute memory')
    glob.append(i.address-start+i.disp_offset)
 if len(glob)!=1 or len(calls)!=1:raise ValueError('unexpected relocation census')
 a.out.mkdir();header='#pragma once\n#define GLOBAL_DISPLACEMENT '+str(glob[0])+'\n#define CALL_DISPLACEMENT '+str(calls[0])+'\nstatic const unsigned char source_code[]={'+','.join(f'0x{x:02x}' for x in raw)+'};\n'
 (a.out/'source_bytes.h').write_bytes(header.encode('ascii'));host=Path(__file__).with_name('cell_skybox_hook_gate_host_review_v1.c')
 manifest={'scope':'review only; no source execution','source_sha256':SOURCE_SHA256,'range':'00763BC0..00763C17','source_slice_sha256':sha(raw),'global_displacement':glob[0],'call_displacement':calls[0],
  'header_sha256':sha(header.encode()),'host_source_sha256':sha(host.read_bytes()),'admissions':'synthetic stable memory, patched global and explicit pure recording callback; no real provider, hook or device',
  'instructions':[{'address':f'{i.address:08X}','bytes':i.bytes.hex(),'text':f'{i.mnemonic} {i.op_str}'} for i in ins]}
 (a.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print('slice',sha(raw),'manifest',sha((a.out/'manifest.json').read_bytes()))
if __name__=='__main__':main()
