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
R = r'''#[path="SOURCE_PATH"] mod arbitration;
use arbitration::{ContactOutput,submit_contact_candidate};
fn contact(a:&[u32])->ContactOutput {ContactOutput{actor_ids:[a[0],a[1]],selector:a[2] as i32,feature_indices:[a[3],a[4]],depth_bits:a[5],point_words:[a[6],a[7],a[8]],normal_words:[a[9],a[10],a[11]]}}
fn main(){let a:Vec<u32>=std::env::args().skip(1).map(|x|u32::from_str_radix(&x,16).unwrap()).collect(); if a.len()!=25 {std::process::exit(10)} let mut o=contact(&a[0..12]); let c=contact(&a[12..24]); let _=submit_contact_candidate(&mut o,c,a[24]); let x=[o.actor_ids[0],o.actor_ids[1],o.selector as u32,o.feature_indices[0],o.feature_indices[1],o.depth_bits,o.point_words[0],o.point_words[1],o.point_words[2],o.normal_words[0],o.normal_words[1],o.normal_words[2]]; println!("{}",x.iter().map(|n|format!("{:08x}",n)).collect::<Vec<_>>().join(" "));}
'''
def sha(x): return hashlib.sha256(x).hexdigest()
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
 (out/'adapter.rs').write_text(R.replace('SOURCE_PATH',rust.as_posix()),encoding='utf-8')
 manifest={'source_sha256':sha(data),'function_sha256':sha(code),'source_file_offset':matches[0],'start':hex(START),'size':SIZE,'host_sha256':sha((out/'host.c').read_bytes()),'adapter_sha256':sha((out/'adapter.rs').read_bytes()),'rust_source_sha256':sha(rust.read_bytes()),'schema':'cell-arbitration-prep-v2','generator_sha256':sha(Path(__file__).read_bytes()),'builder_sha256':sha((ROOT/'tools/cell_arbitration_native_build_v2.py').read_bytes()),'runner_sha256':sha((ROOT/'tools/cell_arbitration_native_compare_v2.py').read_bytes()),'built':False,'executed':False,'note':'Prebuild and linked binary audit required. The function bytes remain local under target.'}
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8'); print(json.dumps(manifest))
if __name__=='__main__': main()
