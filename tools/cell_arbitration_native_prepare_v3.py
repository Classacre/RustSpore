"""Prepare local E59B40 oracle sources only. Does not compile or execute code."""
import argparse, hashlib, json, struct
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
PIN = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
START, SIZE = 0xe59b40, 0xcc
C = r'''#include <windows.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <xmmintrin.h>
static const unsigned char code[] = {SOURCE_BYTES};
static uint32_t root[0x4200/4], output[14], point[3], normal[3];
typedef void *(__cdecl *func)(void *,void *,void *,uint32_t,uint32_t,uint32_t,uint32_t,uint32_t,int32_t);
int main(int argc,char **argv) {
 uint32_t a[25], before[14], root_before[0x4200/4], point_before[3], normal_before[3];
 unsigned int mx, after; DWORD old; void *ret;
 if(sizeof(void*)!=4 || argc!=26) return 10;
 for(int i=0;i<25;i++) { char *e; if(strlen(argv[i+1])!=8) return 11; a[i]=(uint32_t)strtoul(argv[i+1],&e,16); if(*e) return 11; }
 void *p=VirtualAlloc((void*)0x016b0000,0x10000,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
 if(p!=(void*)0x016b0000) return 12;
 void *f=VirtualAlloc(NULL,0x1000,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE); if(!f) return 13;
 memcpy(f,code,sizeof code); if(!VirtualProtect(f,0x1000,PAGE_EXECUTE_READ,&old) || !FlushInstructionCache(GetCurrentProcess(),f,sizeof code)) return 14;
 memset(root,0xa5,sizeof root); root[0x411c/4]=a[24]; *(uint32_t*)0x016b4be4=(uint32_t)(uintptr_t)root;
 output[0]=0x9a9a9a9a; output[13]=0x6b6b6b6b; memcpy(output+1,a,48); memcpy(point,a+18,12); memcpy(normal,a+21,12);
 memcpy(before,output,sizeof output); memcpy(root_before,root,sizeof root); memcpy(point_before,point,sizeof point); memcpy(normal_before,normal,sizeof normal);
 mx=_mm_getcsr(); _mm_setcsr(0x1f80); /* exceptions masked, DAZ and FTZ off */
 ret=((func)f)(output+1,point,normal,a[17],a[12],a[13],a[15],a[16],(int32_t)a[14]);
 after=_mm_getcsr(); _mm_setcsr(mx);
 if(ret!=output+1 || output[0]!=before[0] || output[13]!=before[13] || memcmp(root,root_before,sizeof root) || memcmp(point,point_before,sizeof point) || memcmp(normal,normal_before,sizeof normal) || *(uint32_t*)0x016b4be4!=(uint32_t)(uintptr_t)root) return 15;
 for(int i=1;i<=12;i++) printf("%08x%c",output[i],i==12?'\n':' ');
 fprintf(stderr,"mxcsr_after=%08x\n",after); return 0;
}
'''

def sha(x): return hashlib.sha256(x).hexdigest()
def source_tree_sha():
 entries=[(str(p.relative_to(ROOT)).replace('\\','/'),sha(p.read_bytes())) for p in sorted((ROOT/'crates/spore-core/src').rglob('*.rs'))]
 return sha(json.dumps(entries,separators=(',',':')).encode())
def main():
 p=argparse.ArgumentParser(); p.add_argument('exe',type=Path); p.add_argument('--out',type=Path,required=True); a=p.parse_args()
 out=a.out.resolve(); out.relative_to((ROOT/'target').resolve()); data=a.exe.read_bytes()
 if sha(data)!=PIN: raise ValueError('incorrect source SHA-256')
 pe=struct.unpack_from('<I',data,0x3c)[0]
 if data[:2]!=b'MZ' or data[pe:pe+4]!=b'PE\0\0': raise ValueError('invalid PE')
 machine,nsects=struct.unpack_from('<HH',data,pe+4); optsize=struct.unpack_from('<H',data,pe+20)[0]; opt=pe+24
 if machine!=0x14c or struct.unpack_from('<H',data,opt)[0]!=0x10b: raise ValueError('expected x86 PE32')
 base=struct.unpack_from('<I',data,opt+28)[0]; rva=START-base
 sections=opt+optsize; matches=[]
 for i in range(nsects):
  pos=sections+40*i; vs,va,rawsize,raw=struct.unpack_from('<IIII',data,pos+8)
  if va<=rva and rva+SIZE<=va+rawsize and raw+rawsize<=len(data): matches.append(raw+rva-va)
 if len(matches)!=1: raise ValueError('function not uniquely file-backed')
 code=data[matches[0]:matches[0]+SIZE]; out.mkdir(parents=True,exist_ok=False)
 rust=ROOT/'crates/spore-core/src/cell_feeding_contact_arbitration_e59b40_v1.rs'
 (out/'host.c').write_text(C.replace('SOURCE_BYTES',','.join('0x%02x'%x for x in code)),encoding='utf-8')
 manifest={'source_sha256':sha(data),'function_sha256':sha(code),'source_file_offset':matches[0],'start':hex(START),'size':SIZE,'host_sha256':sha((out/'host.c').read_bytes()),'source_tree_sha256':source_tree_sha(),'cargo_builder_sha256':sha((ROOT/'tools/cell_arbitration_cargo_build_v3.py').read_bytes()),'example_source_sha256':sha((ROOT/'crates/spore-core/examples/cell_arbitration_oracle.rs').read_bytes()),'build_bat_sha256':sha((ROOT/'tools/build.bat').read_bytes()),'cargo_toml_sha256':sha((ROOT/'Cargo.toml').read_bytes()),'crate_toml_sha256':sha((ROOT/'crates/spore-core/Cargo.toml').read_bytes()),'cargo_lock_sha256':sha((ROOT/'Cargo.lock').read_bytes()),'rust_source_sha256':sha(rust.read_bytes()),'schema':'cell-arbitration-prep-v3','generator_sha256':sha(Path(__file__).read_bytes()),'builder_sha256':sha((ROOT/'tools/cell_arbitration_native_build_v3.py').read_bytes()),'runner_sha256':sha((ROOT/'tools/cell_arbitration_native_compare_v3.py').read_bytes()),'built':False,'executed':False,'note':'Prebuild and linked binary audit required. The function bytes remain local under target.'}
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8'); print(json.dumps(manifest))
if __name__=='__main__': main()
