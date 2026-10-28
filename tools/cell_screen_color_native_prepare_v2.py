"""Prepare relocated, unmodified original-byte 4584E0 host; do not execute."""
import argparse, hashlib, json, re, struct
from pathlib import Path
from cell_schema_probe import SourceImage, SOURCE_SHA256
ROOT = Path(__file__).resolve().parent.parent
START, END = 0x4584e0, 0x4585eb
CODE_HASH = 'bcd0b6a5a2d65fcceb4402b691a7304eb3511a703b844309a01138bcf6bc9be1'
EVIDENCE_HASH = '9fd5d3cfd3a59a43c697020606170773498c5d39477ea782e897952ad984ccab'
EVIDENCE = ROOT/'target/rea/20261028-source-runtime-v6/818-analyze_function.json'
RELOCATED = 0x04580000
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
static const unsigned char constant[] = {CONSTANT_BYTES};
static unsigned char data[48], before[48];
static uint32_t mxsave, mx=0x1f80, mxafter, savedesp, returnedesp, result, eflags;
static void reserve(uintptr_t p, DWORD prot) {
 if (VirtualAlloc((void*)p,0x10000,MEM_RESERVE|MEM_COMMIT,prot)!=(void*)p) ExitProcess(10);
}
static void call_original(void) {
 __asm {
  stmxcsr mxsave
  ldmxcsr mx
  cld
  mov savedesp, esp
  lea eax, data
  add eax, 16
  push eax
  mov eax, 04580000h
  call eax
  mov result, eax
  add esp, 4
  mov returnedesp, esp
  pushfd
  pop eflags
  stmxcsr mxafter
  mov esp, savedesp
  ldmxcsr mxsave
 }
}
int main(void) {
 size_t n;
 _setmode(_fileno(stdin),_O_BINARY); _setmode(_fileno(stdout),_O_BINARY);
 reserve(0x04580000,PAGE_EXECUTE_READWRITE); reserve(0x013e0000,PAGE_READWRITE);
 memcpy((void*)0x04580000,code,sizeof code); memcpy((void*)0x13ec540,constant,4);
 if (!FlushInstructionCache(GetCurrentProcess(),(void*)0x04580000,0x10000)) return 11;
 for (;;) {
  memset(data,0xa5,sizeof data);
  n=fread(data+16,1,16,stdin); if (!n) return ferror(stdin)?12:0; if (n!=16) return 13;
  memcpy(before,data,sizeof data);
  call_original();
  if (savedesp!=returnedesp || (eflags&0x400) || (mxafter&~0x3f)!=0x1f80) return 14;
  if (memcmp(data,before,sizeof data)) return 15;
  if (fwrite(&result,1,4,stdout)!=4) return 16;
 }
}'''
def main():
 p=argparse.ArgumentParser(); p.add_argument('exe',type=Path); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
 out=a.out.resolve(); out.relative_to((ROOT/'target').resolve())
 evidence=EVIDENCE.read_bytes()
 if hashlib.sha256(evidence).hexdigest()!=EVIDENCE_HASH: raise ValueError('static evidence digest mismatch')
 record=json.loads(evidence)['structuredContent']
 if record['evidence_id']!='ev_66a94fe55bf6a800d5c1e340c05c507ce72895e113785723ebc8cccd85123b30' or record['subject']['digest']['sha256']!=SOURCE_SHA256: raise ValueError('static evidence identity mismatch')
 result=record['normalized_result']; body=result['procedure']['body']; asm=result['assembly']
 if result['procedure']['address']!='0x4584e0' or body['ranges']!=[{'start':'0x4584e0','end':'0x4585ea'}] or body['total_bytes']!=267 or len(asm)!=57: raise ValueError('unexpected decoded span')
 if asm[-1]!='0x4585ea: RET': raise ValueError('unexpected final instruction')
 if any(re.search(r': (?:J\w+|CALL\w*|LOOP\w*|RET\w*|INT\w*|SYS\w*)\b',ins) for ins in asm[:-1]): raise ValueError('unexpected control flow in original code')
 if set(re.findall(r'\[0x[0-9a-fA-F]+\]', '\n'.join(asm)))!={'[0x013ec540]'}: raise ValueError('unexpected absolute data reference')
 image=SourceImage(a.exe); code=image.read(START,END-START); const=image.read(0x13ec540,4)
 if hashlib.sha256(code).hexdigest()!=CODE_HASH: raise ValueError('routine digest mismatch')
 if struct.unpack('<I',const)[0]!=0x437f0000: raise ValueError('constant mismatch')
 arr=lambda b: ','.join('0x%02x'%x for x in b)
 host=HOST.replace('ORIGINAL_BYTES',arr(code)).replace('CONSTANT_BYTES',arr(const))
 out.mkdir(parents=True,exist_ok=False)
 (out/'host.c').write_text(host,encoding='utf-8'); (out/'original.bin').write_bytes(code)
 manifest=dict(source_sha256=SOURCE_SHA256,code_start=hex(START),code_end_exclusive=hex(END),code_sha256=CODE_HASH,relocated_code_address=hex(RELOCATED),static_evidence_sha256=EVIDENCE_HASH,static_evidence_id=record['evidence_id'],static_control_flow='56 non-control instructions followed by RET; only absolute data address 0x013ec540',constant_address='0x13ec540',constant_bits='0x437f0000',host_sha256=hashlib.sha256((out/'host.c').read_bytes()).hexdigest(),built=False,executed=False)
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8'); print(json.dumps(manifest,indent=2))
if __name__=='__main__': main()
