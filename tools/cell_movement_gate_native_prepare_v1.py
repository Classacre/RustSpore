"""Prepare, but do not build or execute, the original-byte E56960 host."""
import argparse, hashlib, json, struct
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256
ROOT = Path(__file__).resolve().parent.parent
START, END = 0xe56960, 0xe569cf
CODE_HASH = '1bb2c41693ab1d0ba73d6f37256d38620d852b0e7f1ea9ddfaf0681a15895b31'
CONSTANTS = {0x13eb96c: 0x40400000, 0x1485ff4: 0x3fc00000}
HOST = r'''#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <io.h>
#include <fcntl.h>
#if !defined(_M_IX86)
#error Requires MSVC x86
#endif
static const unsigned char code[] = {ORIGINAL_BYTES};
static const unsigned char c3[] = {CONST_THREE};
static const unsigned char c15[] = {CONST_TIMER};
__declspec(align(16)) static unsigned char actor[0x388], before[0x388], x87save[108];
static uint32_t row[10], mxsave, mx=0x1f80, mxafter, savedesp, returnedesp, alresult, eflags;
static unsigned short cw, cwafter, swafter;
static void reserve(uintptr_t p, DWORD prot) {
 if(VirtualAlloc((void*)p,0x10000,MEM_RESERVE|MEM_COMMIT,prot)!=(void*)p) ExitProcess(10);
}
static void call_original(void) {
 __asm {
  fnsave x87save
  fwait
  stmxcsr mxsave
  fninit
  fldcw cw
  ldmxcsr mx
  cld
  mov savedesp, esp
  push row[28]
  lea eax, actor
  add eax, 10h
  push eax
  mov eax, 0e56960h
  call eax
  movzx eax, al
  mov alresult, eax
  add esp, 8
  mov returnedesp, esp
  pushfd
  pop eflags
  fnstcw cwafter
  fnstsw swafter
  stmxcsr mxafter
  mov esp, savedesp
  frstor x87save
  ldmxcsr mxsave
 }
}
int main(void) {
 uint32_t out[2]; size_t n, i;
 _setmode(_fileno(stdin),_O_BINARY); _setmode(_fileno(stdout),_O_BINARY);
 reserve(0x00e50000,PAGE_EXECUTE_READWRITE); reserve(0x013e0000,PAGE_READWRITE); reserve(0x01480000,PAGE_READWRITE);
 memcpy((void*)0xe56960,code,sizeof code); memcpy((void*)0x13eb96c,c3,4); memcpy((void*)0x1485ff4,c15,4);
 if(!FlushInstructionCache(GetCurrentProcess(),(void*)0xe50000,0x10000)) return 11;
 for(;;) {
  n=fread(row,1,sizeof row,stdin); if(!n) return ferror(stdin)?12:0; if(n!=sizeof row) return 13;
  cw=(unsigned short)row[8];
  if(row[8]!=cw || (cw & ~0x0f00)!=0x7f || ((cw & 0x300)!=0x200 && (cw & 0x300)!=0x300) || row[9]!=0x1f80) return 14;
  for(i=0;i<8;i++) if((row[i]&0x7f800000u)==0x7f800000u) return 15;
  memset(actor,0xa5,sizeof actor); memcpy(actor+0x18,row,12); memcpy(actor+0x5c,row+3,12); memcpy(actor+0x378,row+6,4); memcpy(before,actor,sizeof actor);
  call_original();
  if(savedesp!=returnedesp || (eflags&0x400) || cwafter!=cw || (swafter&0x3800) || (mxafter&~0x3f)!=0x1f80 || alresult>1) return 16;
  for(i=0;i<sizeof actor;i++) if((i<0x378 || i>=0x37c) && actor[i]!=before[i]) return 17;
  out[0]=alresult; memcpy(&out[1],actor+0x378,4);
  if(fwrite(out,1,sizeof out,stdout)!=sizeof out) return 18;
 }
}'''
def main():
 p=argparse.ArgumentParser(); p.add_argument('exe',type=Path); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
 out=a.out.resolve(); out.relative_to((ROOT/'target').resolve())
 image=SourceImage(a.exe); code=image.read(START,END-START)
 if hashlib.sha256(code).hexdigest()!=CODE_HASH: raise ValueError('routine digest mismatch')
 const=[]
 for address,bits in CONSTANTS.items():
  b=image.read(address,4)
  if struct.unpack('<I',b)[0]!=bits: raise ValueError('constant mismatch')
  const.append(b)
 def arr(b): return ','.join('0x%02x'%x for x in b)
 host=HOST.replace('ORIGINAL_BYTES',arr(code)).replace('CONST_THREE',arr(const[0])).replace('CONST_TIMER',arr(const[1]))
 out.mkdir(parents=True,exist_ok=False)
 (out/'host.c').write_text(host,encoding='utf-8')
 (out/'original.bin').write_bytes(code)
 manifest=dict(source_sha256=SOURCE_SHA256,code_start=hex(START),code_end_exclusive=hex(END),code_sha256=CODE_HASH,constants={hex(k):hex(v) for k,v in CONSTANTS.items()},host_sha256=hashlib.sha256((out/'host.c').read_bytes()).hexdigest(),built=False,executed=False)
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(manifest,indent=2))
if __name__=='__main__': main()
