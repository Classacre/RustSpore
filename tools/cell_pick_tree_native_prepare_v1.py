"""Prepare, but never build or execute, a bounded original-byte pick host.

Input/output are synthetic supplied trees, not retail process state. Generated
source and original bytes stay in a fresh ignored target directory. Review the
manifest, source, compiler/linker output and runner before native execution.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from cell_schema_probe import SOURCE_SHA256, SourceImage
from native_crt_acos_probe import decoder
from capstone import CS_GRP_CALL, CS_GRP_JUMP
from capstone.x86 import X86_OP_IMM, X86_OP_MEM

CODE = [(0x5A9C40, 0x5A9D3B), (0xE87170, 0xE87261)]


def audit(image):
    result = []
    for start, end in CODE:
        raw = image.read(start, end-start)
        rows = list(decoder().disasm(raw, start))
        assert sum(x.size for x in rows) == len(raw)
        boundaries = {x.address for x in rows}
        transfers, absolute, memory = [], [], []
        for ins in rows:
            if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
                op = ins.operands[0]
                assert op.type == X86_OP_IMM, (hex(ins.address), ins.op_str)
                if ins.group(CS_GRP_CALL):
                    assert (ins.address, op.imm) in {(0xE871EA, 0x5A9C40), (0xE8724C, 0xE87170)}
                else:
                    assert op.imm in boundaries, (hex(ins.address), hex(op.imm))
                transfers.append([f'{ins.address:08X}', ins.mnemonic, f'{op.imm:08X}'])
            for op in ins.operands:
                if op.type == X86_OP_MEM:
                    if not op.mem.base:
                        absolute.append([f'{ins.address:08X}', ins.op_str])
                    else:
                        memory.append([f'{ins.address:08X}', ins.op_str])
        assert not absolute, absolute
        result.append(dict(start=f'{start:08X}', end_exclusive=f'{end:08X}',
            sha256=hashlib.sha256(raw).hexdigest(), size=len(raw), transfers=transfers,
            absolute_memory=absolute, register_relative_memory=memory,
            disassembly=[f'{x.address:08X}: {x.mnemonic} {x.op_str}' for x in rows]))
    return result


def byte_array(name, data):
    return f'static const unsigned char {name}[] = {{\n' + '\n'.join(
        '  '+','.join(f'0x{x:02x}' for x in data[i:i+16])+','
        for i in range(0,len(data),16)) + '\n};\n'


C_HOST = r'''
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef unsigned int U;
int _fltused = 0; /* MSVC floating-point marker; no CRT dependency. */
typedef struct { U x,y,z,r,kind,a,b; } DiskNode;
typedef struct { U cw,n,capacity,root,origin[3],direction[3],limit; DiskNode nodes[31]; } Request;
typedef struct Node { U center[3],r,parent; struct Node *first; U second_or_handle; } Node;
typedef struct { U count,handles[31],cw,mxcsr; } Reply;
static struct { U before[4]; Request value; U after[4]; } in;
static struct { U before[4]; Node value[31]; U after[4]; } tree;
static struct { U before[4]; U value[31]; U after[4]; } out;
static Request saved;
static Node saved_tree[31];
static Reply reply;
static U oldcw, oldmx, cw, mx=0x1f80, capturedcw, capturedmx;
static HANDLE input,output;
static unsigned char *mapped;
static int read_record(void) {
 U got=0,n=0; unsigned char *p=(unsigned char *)&in.value;
 while(n<sizeof(Request)) {
  if(!ReadFile(input,p+n,(DWORD)(sizeof(Request)-n),(DWORD *)&got,0)) {
   if(n==0 && GetLastError()==ERROR_BROKEN_PIPE) return 0;
   ExitProcess(11);
  }
  if(!got) { if(!n) return 0; ExitProcess(12); } n+=got;
 } return 1;
}
static void write_record(void) {
 U n=0,got; unsigned char *p=(unsigned char *)&reply;
 while(n<sizeof(reply)) { if(!WriteFile(output,p+n,(DWORD)(sizeof(reply)-n),(DWORD *)&got,0)||!got) ExitProcess(13); n+=got; }
}
static int bounded(U bits) { union {U u; float f;} v; v.u=bits;
 return (bits&0x7f800000u)!=0x7f800000u && v.f>=-1000.0f && v.f<=1000.0f; }
static void fail(void) { ExitProcess(14); }
static void map(const unsigned char *src,U len,U offset) {
 U i,old; unsigned char *base=mapped+(offset&~0xffffu);
 unsigned char *p=(unsigned char *)VirtualAlloc(base,65536,MEM_COMMIT,PAGE_READWRITE);
 if(p!=base) ExitProcess(21);
 for(i=0;i<len;i++) mapped[offset+i]=src[i];
 if(!VirtualProtect(p,65536,PAGE_EXECUTE_READ,(DWORD *)&old)) ExitProcess(22);
 if(!FlushInstructionCache(GetCurrentProcess(),p,65536)) ExitProcess(23);
}
void __cdecl HostMain(void) {
 U i,j,seen[31],count; U *q; float *o,*d;
 typedef void (__cdecl *Walk)(Node *,float *,float *,U *,U *,U,float);
 union { U u; float f; } limit;
 input=GetStdHandle(STD_INPUT_HANDLE); output=GetStdHandle(STD_OUTPUT_HANDLE);
 if(input==INVALID_HANDLE_VALUE||output==INVALID_HANDLE_VALUE) fail();
 mapped=(unsigned char *)VirtualAlloc(0,0x8f0000,MEM_RESERVE,PAGE_NOACCESS);
 if(!mapped) ExitProcess(20);
 map(code_005a9c40,sizeof(code_005a9c40),0x9c40);
 map(code_00e87170,sizeof(code_00e87170),0x8e7170);
 while(read_record()) {
  if(in.value.n>31 || in.value.capacity>31 || (in.value.n && in.value.root>=in.value.n)) fail();
  cw=in.value.cw;
  if((cw & ~0x0f00u)!=0x7fu || (cw&0x300u)==0x100u) fail();
  for(i=0;i<3;i++) if(!bounded(in.value.origin[i]) || !bounded(in.value.direction[i])) fail();
  if(!bounded(in.value.limit) && in.value.limit!=0x7f7fffffu) fail();
  for(i=0;i<31;i++) seen[i]=0;
  for(i=0;i<in.value.n;i++) {
   DiskNode *s=&in.value.nodes[i]; Node *t=&tree.value[i];
   if(!bounded(s->x)||!bounded(s->y)||!bounded(s->z)||!bounded(s->r)||s->kind>1) fail();
   t->center[0]=s->x; t->center[1]=s->y; t->center[2]=s->z; t->r=s->r; t->parent=0;
   if(!s->kind) { t->first=0; t->second_or_handle=s->a; }
   else { if(s->a<=i||s->b<=i||s->a>=in.value.n||s->b>=in.value.n||s->a==s->b) fail();
    if(++seen[s->a]>1||++seen[s->b]>1) fail();
    t->first=&tree.value[s->a]; t->second_or_handle=(U)&tree.value[s->b]; }
  }
  if(in.value.n && seen[in.value.root]) fail();
  for(i=0;i<4;i++) in.before[i]=in.after[i]=tree.before[i]=tree.after[i]=out.before[i]=out.after[i]=0xa5a5a5a5u;
  saved=in.value; for(i=0;i<31;i++) { saved_tree[i]=tree.value[i]; out.value[i]=0xccccccccu; }
  count=0; o=(float *)in.value.origin; d=(float *)in.value.direction; limit.u=in.value.limit;
  __asm {
   fnstcw word ptr oldcw
   stmxcsr oldmx
   fldcw word ptr cw
   ldmxcsr mx
  }
  if(in.value.n) ((Walk)(mapped+0x8e7170))(&tree.value[in.value.root],o,d,out.value,&count,in.value.capacity,limit.f);
  __asm {
   fnstcw word ptr capturedcw
   stmxcsr capturedmx
   fldcw word ptr oldcw
   ldmxcsr oldmx
  }
  if(count>in.value.capacity || count>31) fail();
  for(i=0;i<4;i++) if(in.before[i]!=0xa5a5a5a5u||in.after[i]!=0xa5a5a5a5u||tree.before[i]!=0xa5a5a5a5u||tree.after[i]!=0xa5a5a5a5u||out.before[i]!=0xa5a5a5a5u||out.after[i]!=0xa5a5a5a5u) fail();
  q=(U *)&saved; for(j=0;j<sizeof(Request)/4;j++) if(q[j]!=((U *)&in.value)[j]) fail();
  q=(U *)saved_tree; for(j=0;j<sizeof(saved_tree)/4;j++) if(q[j]!=((U *)tree.value)[j]) fail();
  for(i=count;i<31;i++) if(out.value[i]!=0xccccccccu) fail();
  reply.count=count; for(i=0;i<31;i++) reply.handles[i]=out.value[i];
  reply.cw=capturedcw&0xffffu; reply.mxcsr=capturedmx; write_record();
 } ExitProcess(0);
}
'''


def main():
    ap=argparse.ArgumentParser(); ap.add_argument('exe',type=Path); ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args(); root=Path(__file__).resolve().parent.parent
    out=args.out.resolve(); out.relative_to((root/'target').resolve()); out.mkdir(parents=True,exist_ok=False)
    image=SourceImage(args.exe)
    source_hash=hashlib.sha256(args.exe.read_bytes()).hexdigest(); assert source_hash==SOURCE_SHA256
    ranges=audit(image)
    body=''.join(byte_array(f'code_{start:08x}',image.read(start,end-start)) for start,end in CODE)
    # Put declarations after the Windows includes so the C compiler sees them.
    source=C_HOST.replace('typedef unsigned int U;',body+'\ntypedef unsigned int U;')
    (out/'host.c').write_text(source,encoding='ascii',newline='\n')
    manifest=dict(source_sha256=source_hash, source_ranges=ranges, input_bytes=912,
        output_bytes=136, max_nodes=31, source_executed=False,
        mapping=dict(reservation_bytes=0x8f0000,source_image_anchor='005A0000',
                     offsets=['00009C40','008E7170'],same_delta=True,
                     committed_regions=['00000000+00010000','008E0000+00010000']),
        host_c_sha256=hashlib.sha256(source.encode('ascii')).hexdigest(),
        limitations=['synthetic supplied finite tree only','not a game process or live actor',
                     'host and executable require independent review before execution'])
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf8')
    print(json.dumps({'out':str(out),'source_executed':False,'host_c_sha256':manifest['host_c_sha256']}))

if __name__=='__main__': main()
