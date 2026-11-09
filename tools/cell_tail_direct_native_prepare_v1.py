"""Review-only E6A2B0 finite non-relation-5 host preparation; never builds or runs it."""
import argparse, hashlib, json, struct
from pathlib import Path
import capstone
import pefile

ROOT = Path(__file__).resolve().parent.parent
PIN = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
START, END = 0xE6A2B0, 0xE6A442
HOST = r'''#include <windows.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <xmmintrin.h>
static const unsigned char source[] = {SOURCE};
typedef void (__cdecl *tail)(void*,void*,void*,uint32_t,void*,void*,int32_t,uint32_t);
int main(int argc, char **argv) {
 uint32_t a[21], out[14], before[14], point[3], normal[3], p0[3], n0[3];
 uint32_t actor_a[3], actor_b[3], aa[3], bb[3]; DWORD old; unsigned int mx;
 if (sizeof(void*) != 4 || argc != 22) return 10;
 for(int i=0;i<21;i++) { char *end; if(strlen(argv[i+1])!=8) return 11;
  for(int j=0;j<8;j++) { char c=argv[i+1][j]; if(!((c>='0'&&c<='9')||(c>='a'&&c<='f')||(c>='A'&&c<='F'))) return 11; }
  a[i]=(uint32_t)strtoul(argv[i+1],&end,16); if(*end) return 11; }
 if((a[5]&0x7f800000u)==0x7f800000u || (a[20]&0x7f800000u)==0x7f800000u) return 15;
 out[0]=0x9a9a9a9a; out[13]=0x6b6b6b6b; memcpy(out+1,a,48);
 memcpy(point,a+14,12); memcpy(normal,a+17,12); memcpy(p0,point,12); memcpy(n0,normal,12);
 actor_a[0]=0xa1a1a1a1; actor_a[1]=a[12]; actor_a[2]=0xa2a2a2a2;
 actor_b[0]=0xb1b1b1b1; actor_b[1]=a[13]; actor_b[2]=0xb2b2b2b2;
 memcpy(aa,actor_a,sizeof aa); memcpy(bb,actor_b,sizeof bb); memcpy(before,out,sizeof out);
 void *code=VirtualAlloc(NULL,0x1000,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE); if(!code) return 12;
 memcpy(code,source,sizeof source); if(!VirtualProtect(code,0x1000,PAGE_EXECUTE_READ,&old) ||
 !FlushInstructionCache(GetCurrentProcess(),code,sizeof source)) return 13;
 mx=_mm_getcsr(); _mm_setcsr(0x1f80);
 /* Fixed relation=0: no execution can enter external-call branch. */
 ((tail)code)(out+1,point,normal,a[20],actor_a+1,actor_b+1,0,0);
 _mm_setcsr(mx);
 if(out[0]!=before[0] || out[13]!=before[13] || memcmp(point,p0,12) || memcmp(normal,n0,12) ||
 memcmp(actor_a,aa,sizeof aa) || memcmp(actor_b,bb,sizeof bb)) return 14;
 for(int i=1;i<=12;i++) printf("%08x%c",out[i],i==12?'\n':' ');
 return 0;
}
'''

def sha(b): return hashlib.sha256(b).hexdigest()

def main():
    p=argparse.ArgumentParser(); p.add_argument('exe',type=Path); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
    out=a.out.resolve(); out.relative_to((ROOT/'target').resolve())
    data=a.exe.read_bytes()
    if sha(data)!=PIN: raise ValueError('source digest mismatch')
    pe=pefile.PE(data=data,fast_load=True)
    if pe.FILE_HEADER.Machine!=0x14c or pe.OPTIONAL_HEADER.ImageBase!=0x400000: raise ValueError('unexpected PE')
    code=pe.get_data(START-pe.OPTIONAL_HEADER.ImageBase,END-START)
    if len(code)!=END-START: raise ValueError('missing source bytes')
    # Independently inspect non-5 branch: first four instructions and both
    # reachable blocks; addresses and branch targets must be reviewed further.
    md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
    ins=list(md.disasm(code,START)); by={i.address:i for i in ins}
    ranges=((START,0xE6A2BC),(0xE6A333,0xE6A335),(0xE6A3C9,0xE6A442))
    lines=[f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in ins if any(lo<=i.address<hi for lo,hi in ranges)]
    if by[0xE6A2B6].mnemonic!='jne' or by[0xE6A2B6].op_str!='0xe6a3c9': raise ValueError('unexpected dispatch')
    if any(by[x].mnemonic in ('call','jmp') for x in by if 0xE6A3C9<=x<END): raise ValueError('unexpected call or jump')
    expected={0xE6A3E5:('jl','0xe6a3f7'),0xE6A3E7:('jne','0xe6a333'),0xE6A3F1:('jbe','0xe6a333')}
    for addr,(op,target) in expected.items():
        if (by[addr].mnemonic,by[addr].op_str)!=(op,target): raise ValueError('unexpected direct-branch CFG')
    if by[0xE6A333].mnemonic!='pop' or by[0xE6A334].mnemonic!='ret' or by[0xE6A440].mnemonic!='pop' or by[0xE6A441].mnemonic!='ret': raise ValueError('unexpected return')
    reachable=[i for i in ins if 0xE6A3C9<=i.address<END]
    if [(i.address,i.mnemonic) for i in reachable if i.mnemonic.startswith('j')] != [(a,v[0]) for a,v in expected.items()]: raise ValueError('unexpected conditional branch')
    out.mkdir(parents=True,exist_ok=False)
    (out/'source.bin').write_bytes(code)
    (out/'host.c').write_text(HOST.replace('SOURCE',','.join('0x%02x'%x for x in code)),encoding='ascii')
    (out/'reachable-listing.txt').write_text('\n'.join(lines)+'\n',encoding='ascii')
    files=['source.bin','host.c','reachable-listing.txt']
    manifest={'schema':'cell-tail-direct-preparation-v1','source_sha256':sha(data),'start':hex(START),'end_exclusive':hex(END),
      'file_sha256':{f:sha((out/f).read_bytes()) for f in files}, 'generator_sha256':sha(Path(__file__).read_bytes()),
      'model_sha256':sha((ROOT/'crates/spore-core/src/cell_feeding_tail_e6a2b0_v1.rs').read_bytes()),
      'built':False,'executed':False,'scope':'fixed relation 0 only; manual CFG, ABI, host and linked-binary audits required'}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='ascii')
    print(json.dumps(manifest,indent=2))
if __name__=='__main__': main()
