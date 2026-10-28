"""Generate a local-only fixed-address original scaling host; never build or run it."""
import argparse, hashlib, json, struct, sys
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256

ROOT = Path(__file__).resolve().parent.parent
SLICES = [(0xE6CF18, 0xE6CF4D), (0xE6D15E, 0xE6D166), (0x935D40, 0x935D76)]
CONSTANTS = {0x143F52C: 0x2F800000, 0x1471B2C: 0x3F000000}
SLICE_SHA256 = [
    'f580f51e71a90dbf675c7af6e46280e43ca14f35459f0f4db5034d08fd0e7681',
    '55ad246e1ef4090b4d8944350f7f6c71de20bb96613aeabe4fe75b50139f7af3',
    '35d81765a9410a57d5516506680395d783bffbc0d2ff6cc20e1b74aa1211afce',
]


HOST = r'''#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
#if !defined(_M_IX86)
#error Requires 32-bit x86 MSVC
#endif
static const unsigned char a[] = {SLICE_A};
static const unsigned char b[] = {SLICE_B};
static const unsigned char rng[] = {SLICE_RNG};
static const unsigned char c1[] = {CONST_1};
static const unsigned char c2[] = {CONST_2};
static uint32_t saved_esp, sum_bits, draw_bits, exit_address;
/* 32-bit FSAVE image is 108 bytes. */
__declspec(align(16)) static unsigned char x87_image[108];
static unsigned short cw;
static void reserve(uintptr_t base, DWORD protection) {
 void *p = VirtualAlloc((void*)base, 0x10000, MEM_RESERVE|MEM_COMMIT, protection);
 if (p != (void*)base) { fprintf(stderr,"address collision at %08lx error %lu\n",(unsigned long)base,GetLastError()); ExitProcess(10); }
}
static void run_case(void) {
 uint32_t displacement;
 __asm { mov eax, offset host_exit }
 __asm { mov exit_address, eax }
 displacement=exit_address - 0xe6d16bu;
 *(unsigned char*)0xe6d166=0xe9; memcpy((void*)0xe6d167,&displacement,4);
 if(!FlushInstructionCache(GetCurrentProcess(),(void*)0xe60000,0x10000)) ExitProcess(15);
 __asm {
  fnsave x87_image
  fwait
  fninit
  fldcw cw
  mov saved_esp, esp
  sub esp, 40h
  mov eax, sum_bits
  mov [esp+10h], eax
  mov eax, 0e6cf18h
  jmp eax
 host_exit:
  mov eax, [esp+18h]
  mov draw_bits, eax
  mov esp, saved_esp
  frstor x87_image
 }
}
int main(void) {
 uint32_t row[3], out[2];
 _setmode(_fileno(stdin),_O_BINARY); _setmode(_fileno(stdout),_O_BINARY);
 reserve(0x00930000, PAGE_EXECUTE_READWRITE);
 reserve(0x00e60000, PAGE_EXECUTE_READWRITE);
 reserve(0x01430000, PAGE_READWRITE);
 reserve(0x01470000, PAGE_READWRITE);
 reserve(0x01600000, PAGE_READWRITE);
 memcpy((void*)0xe6cf18,a,sizeof a); memcpy((void*)0xe6d15e,b,sizeof b);
 memcpy((void*)0x935d40,rng,sizeof rng); memcpy((void*)0x143f52c,c1,4); memcpy((void*)0x1471b2c,c2,4);
 if(!FlushInstructionCache(GetCurrentProcess(),(void*)0x930000,0x10000)) return 15;
 for (;;) {
  size_t n=fread(row,1,sizeof row,stdin);
  if(!n) return ferror(stdin)?11:0;
  if(n!=sizeof row) return 12;
  cw=(unsigned short)row[2];
  if(!row[0] || row[2]!=cw || (cw & ~0x0f00)!=0x7f || ((cw & 0x300)!=0x200 && (cw & 0x300)!=0x300)) return 13;
  /* Finite, nonnegative binary32 sum (including positive zero only). */
  if((row[1]&0x80000000u) || (row[1]&0x7f800000u)==0x7f800000u) return 16;
  *(uint32_t*)0x1602760=row[0]; sum_bits=row[1]; run_case();
  out[0]=*(uint32_t*)0x1602760; out[1]=draw_bits;
  if(fwrite(out,1,sizeof out,stdout)!=sizeof out) return 14;
 }
}'''

def main():
 p=argparse.ArgumentParser(); p.add_argument('exe',type=Path); p.add_argument('--out',required=True,type=Path); args=p.parse_args()
 out=args.out.resolve(); out.relative_to((ROOT/'target').resolve())
 image=SourceImage(args.exe)
 sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'))
 from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_REG_ESP
 decoder=Cs(CS_ARCH_X86,CS_MODE_32); decoder.detail=True
 blobs=[]; disassembly=[]; controls=[]; absolute=[]; stack_refs=[]; immediates=[]
 for (start,end), expected_hash in zip(SLICES,SLICE_SHA256):
  blob=image.read(start,end-start); ins=list(decoder.disasm(blob,start))
  if hashlib.sha256(blob).hexdigest()!=expected_hash: raise ValueError(f'slice hash mismatch at {start:08X}')
  if not ins or ins[0].address!=start or ins[-1].address+ins[-1].size!=end or sum(i.size for i in ins)!=len(blob):
   raise ValueError(f'incomplete decode at {start:08X}')
  for i in ins:
   if i.group(CS_GRP_CALL) or i.group(CS_GRP_JUMP):
    if len(i.operands)!=1 or i.operands[0].type!=X86_OP_IMM: raise ValueError('indirect control flow')
    controls.append((i.address,i.operands[0].imm & 0xffffffff))
   for op in i.operands:
    if op.type==X86_OP_MEM:
     if op.mem.base==X86_REG_ESP and not op.mem.index: stack_refs.append((i.address,op.mem.disp))
     if not op.mem.base and not op.mem.index: absolute.append((i.address,op.mem.disp & 0xffffffff))
    elif op.type==X86_OP_IMM: immediates.append((i.address,op.imm & 0xffffffff))
  blobs.append(blob); disassembly.append([f'{i.address:08X} {i.mnemonic} {i.op_str}'.strip() for i in ins])
 expected_controls=[(0xe6cf25,0x935d40),(0xe6cf34,0xe6d15e),(0xe6cf40,0xe6d15e),
                    (0xe6cf48,0xe6d160),(0x935d44,0x935d4a),(0x935d6e,0x935d74)]
 expected_stack=[(0xe6cf18,0x10),(0xe6cf21,0x18),(0xe6cf2a,0x18),
                 (0xe6d162,0x18),(0x935d52,0),(0x935d57,0)]
 if controls!=expected_controls or stack_refs!=expected_stack or absolute!=[(0x935d5a,0x143f52c),(0x935d60,0x1471b2c)]:
  raise ValueError('unexpected control flow or memory operands')
 if (0xe6cf1c,0x1602760) not in immediates: raise ValueError('RNG global address mismatch')
 for address,bits in CONSTANTS.items():
  if struct.unpack('<I',image.read(address,4))[0]!=bits: raise ValueError('constant mismatch')
 def array(data): return ','.join(f'0x{x:02x}' for x in data)
 c=HOST
 for key,blob in zip(['SLICE_A','SLICE_B','SLICE_RNG','CONST_1','CONST_2'],blobs+[image.read(x,4) for x in CONSTANTS]): c=c.replace(key,array(blob))
 out.mkdir(parents=True,exist_ok=False)
 (out/'host.c').write_text(c,encoding='utf-8')
 manifest=dict(source_sha256=SOURCE_SHA256,slices=[dict(start=hex(lo),end_exclusive=hex(hi),sha256=hashlib.sha256(b).hexdigest(),disassembly=d) for (lo,hi),b,d in zip(SLICES,blobs,disassembly)],host_sha256=hashlib.sha256((out/'host.c').read_bytes()).hexdigest(),executed=False)
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(manifest,indent=2))
if __name__=='__main__': main()
