"""Generate a reviewable x86 host and Rust adapter; never build or execute them."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
H = r'''#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define START 0x00E7CE91u
#define END 0x00E7CF19u
static const unsigned char original[] = { ORIGINAL_BYTES };
static volatile uint32_t route = 0xFFFFFFFFu;
static uintptr_t saved_esp, resume_address;
static uint32_t amount_word, source_word;
static unsigned char state[0x100], state_before[0x100];
static unsigned char root[0x5200], root_before[0x5200];

static int parse(const char *s, uint32_t *v) {
    char *end; unsigned long x;
    if (strlen(s) != 8) return 0;
    for (int i=0;i<8;i++) if (!((s[i]>='0'&&s[i]<='9')||(s[i]>='a'&&s[i]<='f'))) return 0;
    x=strtoul(s,&end,16); if (*end) return 0; *v=(uint32_t)x; return 1;
}
static void store32(void *p, uint32_t x) { memcpy(p,&x,4); }
static uint32_t load32(const void *p) { uint32_t x; memcpy(&x,p,4); return x; }
static int map_at(uintptr_t at, DWORD protection) {
    return VirtualAlloc((void*)at,0x10000,MEM_RESERVE|MEM_COMMIT,protection)==(void*)at;
}
static void exit_stub(unsigned char *p, uint32_t value, uintptr_t dest) {
    p[0]=0xC7; p[1]=0x05; store32(p+2,(uint32_t)(uintptr_t)&route);
    store32(p+6,value); p[10]=0xE9; store32(p+11,(uint32_t)(dest-(uintptr_t)(p+15)));
}
int main(int argc, char **argv) {
    uint32_t a[6]; DWORD old; unsigned char *code=(unsigned char*)0x00E70000u;
    unsigned char *cap=(unsigned char*)0x01480000u, *global=(unsigned char*)0x016B0000u;
    if (sizeof(void*)!=4 || argc!=7) return 10;
    for (int i=0;i<6;i++) if (!parse(argv[i+1],&a[i])) return 11;
    if (a[5]>255) return 12;
    if (!map_at((uintptr_t)code,PAGE_READWRITE) || !map_at((uintptr_t)cap,PAGE_READWRITE) ||
        !map_at((uintptr_t)global,PAGE_READWRITE)) return 13;
    memset(code,0xCC,0x10000); memcpy((void*)START,original,sizeof original);
    store32((void*)0x014846DCu,1000); store32((void*)0x016B4BE4u,(uint32_t)(uintptr_t)root);
    memset(root,0xA5,sizeof root); memset(state,0xA5,sizeof state);
    store32(root+0x5190,(uint32_t)(uintptr_t)state);
    store32(state+0x1C,a[0]); store32(state+0x20,a[1]);
    store32(state+0x24,a[2]); store32(state+0x28,a[3]);
    memcpy(root_before,root,sizeof root); memcpy(state_before,state,sizeof state);
    amount_word=a[4]; source_word=a[5];
    __asm {
        mov eax, offset resume_point
        mov resume_address, eax
    }
    exit_stub((unsigned char*)0x00E7CF19u,1,resume_address);
    exit_stub((unsigned char*)0x00E7D090u,0,resume_address);
    if (memcmp((void*)START,original,sizeof original) || load32((void*)0x014846DCu)!=1000) return 14;
    if (!VirtualProtect(code,0x10000,PAGE_EXECUTE_READ,&old) || old!=PAGE_READWRITE ||
        !VirtualProtect(cap,0x10000,PAGE_READONLY,&old) || old!=PAGE_READWRITE ||
        !FlushInstructionCache(GetCurrentProcess(),code,0x10000)) return 15;
    __asm {
        push ebx
        push esi
        push edi
        mov saved_esp, esp
        sub esp, 24h
        mov eax, amount_word
        mov [esp+1ch], eax
        mov eax, source_word
        mov [esp+20h], eax
        mov eax, 00E7CE91h
        jmp eax
    resume_point:
        mov esp, saved_esp
        pop edi
        pop esi
        pop ebx
    }
    if (route>1 || memcmp(root,root_before,sizeof root) ||
        load32((void*)0x016B4BE4u)!=(uint32_t)(uintptr_t)root ||
        memcmp((void*)START,original,sizeof original) || load32((void*)0x014846DCu)!=1000) return 16;
    for (int i=0;i<(int)sizeof state;i++)
        if ((i<0x1C || i>=0x2C) && state[i]!=state_before[i]) return 17;
    printf("%08x %08x %08x %08x %u\n", load32(state+0x1c),load32(state+0x20),
           load32(state+0x24),load32(state+0x28),(unsigned)route);
    return 0;
}
'''
R = r'''#[path = "RUST_SOURCE"] mod cell_progression_v1;
use cell_progression_v1::{account_progress_v1, ProgressStateV1};
fn main() {
 let a: Vec<String> = std::env::args().skip(1).collect();
 if a.len()!=6 { std::process::exit(10) }
 let mut n=[0u32;6];
 for (i,s) in a.iter().enumerate() {
  if s.len()!=8 || !s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) { std::process::exit(11) }
  n[i]=u32::from_str_radix(s,16).unwrap();
 }
 if n[5]>255 { std::process::exit(12) }
 let x=account_progress_v1(ProgressStateV1{progress_1c:n[0] as i32,counter_20:n[1] as i32,counter_24:n[2] as i32,counter_28:n[3] as i32},n[4] as i32,n[5] as u8);
 println!("{:08x} {:08x} {:08x} {:08x} {}",x.state.progress_1c as u32,x.state.counter_20 as u32,x.state.counter_24 as u32,x.state.counter_28 as u32,x.changed_route as u8);
}
'''


def sha(data): return hashlib.sha256(data).hexdigest()


def main():
 p=argparse.ArgumentParser(); p.add_argument('--prep', type=Path, required=True); p.add_argument('--out', type=Path, required=True); a=p.parse_args()
 a.out.resolve().relative_to((ROOT/'target').resolve())
 m=json.loads((a.prep/'audit.json').read_text()); code=(a.prep/'source-slice.bin').read_bytes()
 if sha(code)!='9111637fe540d31fdb7b06db4d527d15b575e219c39bee60c50ef86de636d07e' or sha(code)!=m['source_slice_sha256'] or m['source_slice_start']!='0xe7ce91' or len(code)!=0x88 or (a.prep/'source-cap.bin').read_bytes()!=bytes.fromhex('e8030000'): raise ValueError('source evidence mismatch')
 rust=ROOT/'crates/spore-core/src/cell_progression_v1.rs'
 c=H.replace('ORIGINAL_BYTES',','.join('0x%02x'%x for x in code)); r=R.replace('RUST_SOURCE',rust.as_posix())
 a.out.mkdir(parents=True,exist_ok=False)
 (a.out/'host.c').write_bytes(c.encode('utf-8')); (a.out/'adapter.rs').write_bytes(r.encode('utf-8'))
 m=dict(schema='cell-progress-host-preparation-v2', native_executed=False, built=False,
        source_slice_sha256=sha(code), host_sha256=sha((a.out/'host.c').read_bytes()), adapter_sha256=sha((a.out/'adapter.rs').read_bytes()), rust_source_sha256=sha(rust.read_bytes()),
        prep_audit_sha256=sha((a.prep/'audit.json').read_bytes()), inputs_sha256=sha((a.prep/'inputs.json').read_bytes()), runner_sha256=sha((ROOT/'tools/cell_progress_native_run_v2.py').read_bytes()),
        limitations='Post-E79750 snapshot only. Stops before growth and unchanged return; isolated memory, no concurrent mutation. No live cap observation. Guards cover root and state outside the four fields and the global pointer; synthetic stack and remaining global mapping are not guarded.',
        review_required=['independent prebuild host/source/stack/input audit','independent linked host and adapter audit before execution'])
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(m))
if __name__=='__main__': main()
