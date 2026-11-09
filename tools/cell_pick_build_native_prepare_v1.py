"""Generate only: original-byte bounded tree-operation host and static manifest."""
import argparse,hashlib,json
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from cell_pick_build_native_audit_v1 import CODE
from cell_pick_tree_native_prepare_v1 import byte_array

HOST=r'''
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef unsigned int U;
int _fltused=0;
typedef struct Node { U xyz[3],r; struct Node *parent,*first; U second; } Node;
typedef struct { U count,issued; Node *free; Node *data; U stride,capacity; } Pool;
typedef struct { U unknown; Pool *pool; Node *root,*pending; } Owner;
typedef struct { U op,id,xyz[3],r,handle; } Step;
typedef struct { U cw,n; Step steps[8]; } Request;
typedef struct { U n; U rows[31][6]; U pending_count,pending[8],cw,mx; } Reply;
static struct { U pre[4]; Node a[64]; U post[4]; } arena;
static struct { U pre[4]; Request q; U post[4]; } input;
static Request saved;
static Pool pool; static Owner owner; static Node *leaves[8]; static Reply reply;
static U marks[64],cw,oldcw,mx=0x1f80,oldmx,postcw,postmx; static unsigned char *code;
static HANDLE hin,hout;
static void fail(U code) {ExitProcess(code);}
static int read_record(void) {
 U i=0,n;unsigned char *p=(unsigned char *)&input.q;
 while(i<sizeof(Request)) {if(!ReadFile(hin,p+i,sizeof(Request)-i,(DWORD *)&n,0)) {if(i==0&&GetLastError()==ERROR_BROKEN_PIPE)return 0;fail(31);}
  if(!n){if(!i)return 0;fail(32);}i+=n;}return 1;
}
static void output_record(void) {U i=0,n;unsigned char *p=(unsigned char *)&reply;
 while(i<sizeof(Reply)){if(!WriteFile(hout,p+i,sizeof(Reply)-i,(DWORD *)&n,0)||!n)fail(33);i+=n;}}
static int valid(U bits) {union{U u;float f;}x;x.u=bits;return (bits&0x7f800000u)!=0x7f800000u&&x.f>=-100.0f&&x.f<=100.0f;}
static void put(U a,U v) {*(U *)a=v;}
static void constants(U address) {
 U old;void *p=VirtualAlloc((void *)address,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
 if((U)p!=address)fail(21);
 if(address==0x1470000)put(0x1471b2c,0x3f000000);
 else {put(0x14861d0,0x3f800000);put(0x1486bd0,0x3f8ccccd);put(0x1486bd8,0x37800000);}
 if(!VirtualProtect(p,65536,PAGE_READONLY,(DWORD *)&old))fail(22);
}
static void init_code(void) {
 U i,old;code=(unsigned char *)VirtualAlloc(0,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
 if(!code)fail(23);
 /* Source sequences have the same relocation delta; no source absolute code calls. */
 COPY_CODE
 if(!VirtualProtect(code,65536,PAGE_EXECUTE_READ,(DWORD *)&old)||!FlushInstructionCache(GetCurrentProcess(),code,65536))fail(24);
}
static U idx(Node *p){U a=(U)p,b=(U)&arena.a[0];if(a<b||a>=b+sizeof(arena.a)||(a-b)%sizeof(Node))fail(41);return (a-b)/sizeof(Node);}
static void visit(Node *p,Node *parent) {
 U i,k;if(!p)fail(42);i=idx(p);if(marks[i]++)fail(43);if(p->parent!=parent)fail(44);
 if(reply.n>=31)fail(45);k=reply.n++;reply.rows[k][0]=p->first?1:0;
 reply.rows[k][1]=p->first?0:p->second;reply.rows[k][2]=p->xyz[0];reply.rows[k][3]=p->xyz[1];reply.rows[k][4]=p->xyz[2];reply.rows[k][5]=p->r;
 if(p->first){visit(p->first,p);visit((Node *)p->second,p);}
}
void __cdecl HostMain(void) {
 U i,j;U *a,*b;Node *node;union{U u;float f;} radius;
 typedef Node *(__cdecl *Create)(Owner *,float *,float,U);
 typedef void (__cdecl *Flush)(Owner *);
 typedef void (__cdecl *Update)(Owner *,Node *,float *,float);
 typedef void (__cdecl *Remove)(Owner *,Node *);
 hin=GetStdHandle(STD_INPUT_HANDLE);hout=GetStdHandle(STD_OUTPUT_HANDLE);
 constants(0x1470000);constants(0x1480000);init_code();
 while(read_record()) {
  if(input.q.n>8)fail(34);cw=input.q.cw;
  if((cw&~0xf00u)!=0x7f||(cw&0x300u)==0x100u)fail(35);
  for(i=0;i<4;i++)arena.pre[i]=arena.post[i]=input.pre[i]=input.post[i]=0xa5a5a5a5u;
  for(i=0;i<64;i++){arena.a[i].xyz[0]=arena.a[i].xyz[1]=arena.a[i].xyz[2]=arena.a[i].r=0;arena.a[i].parent=arena.a[i].first=0;arena.a[i].second=0;}
  for(i=0;i<8;i++)leaves[i]=0;
  pool.count=pool.issued=0;pool.free=0;pool.data=arena.a;pool.stride=28;pool.capacity=64;
  owner.unknown=0;owner.pool=&pool;owner.root=owner.pending=0;saved=input.q;
  __asm { fnstcw word ptr oldcw }
  __asm { stmxcsr oldmx }
  __asm { fldcw word ptr cw }
  __asm { ldmxcsr mx }
  for(i=0;i<input.q.n;i++) {
   Step *s=&input.q.steps[i]; if(s->op>3||s->id>=8)fail(36);
   if(s->op==0||s->op==2){for(j=0;j<3;j++)if(!valid(s->xyz[j]))fail(37);if(!valid(s->r)||(s->r&0x80000000u))fail(38);}
   radius.u=s->r;
   if(s->op==0){if(leaves[s->id])fail(39);node=((Create)(code+0x6960))(&owner,(float *)s->xyz,radius.f,s->handle);if(!node)fail(46);leaves[s->id]=node;}
   else if(s->op==1)((Flush)(code+0x6fe0))(&owner);
   else if(s->op==2){if(!leaves[s->id])fail(47);((Update)(code+0x70b0))(&owner,leaves[s->id],(float *)s->xyz,radius.f);}
   else {if(!leaves[s->id])fail(48);((Remove)(code+0x69f0))(&owner,leaves[s->id]);leaves[s->id]=0;}
  }
  __asm { fnstcw word ptr postcw }
  __asm { stmxcsr postmx }
  __asm { fldcw word ptr oldcw }
  __asm { ldmxcsr oldmx }
  for(i=0;i<64;i++)marks[i]=0;for(i=0;i<sizeof(Reply)/4;i++)((U *)&reply)[i]=0;
  if(owner.root)visit(owner.root,0);
  node=owner.pending;while(node){i=idx(node);if(marks[i]++||node->parent||reply.pending_count>=8)fail(49);
   reply.pending[reply.pending_count++]=node->second;node=node->first;}
  for(i=0;i<4;i++)if(arena.pre[i]!=0xa5a5a5a5u||arena.post[i]!=0xa5a5a5a5u||input.pre[i]!=0xa5a5a5a5u||input.post[i]!=0xa5a5a5a5u)fail(50);
  a=(U *)&saved;b=(U *)&input.q;for(i=0;i<sizeof(Request)/4;i++)if(a[i]!=b[i])fail(51);
  reply.cw=postcw&0xffffu;reply.mx=postmx;output_record();
 }ExitProcess(0);
}
'''

def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('--audit',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 root=Path(__file__).resolve().parent.parent;out=a.out.resolve();out.relative_to((root/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 audit=json.loads(a.audit.read_text());img=SourceImage(a.exe);assert hashlib.sha256(a.exe.read_bytes()).hexdigest()==SOURCE_SHA256==audit['source_sha256']
 declarations=[];copies=[]
 for i,(start,end) in enumerate(CODE):
  raw=img.read(start,end-start); assert hashlib.sha256(raw).hexdigest()==audit['ranges'][i]['sha256']
  declarations.append(byte_array(f'fragment_{i}',raw));copies.append(f'for(i=0;i<sizeof(fragment_{i});i++)code[0x{start-0xe80000:x}+i]=fragment_{i}[i];')
 source=HOST.replace('typedef unsigned int U;',''.join(declarations)+'\ntypedef unsigned int U;').replace('COPY_CODE','\n '.join(copies))
 (out/'host.c').write_text(source,encoding='ascii',newline='\n')
 manifest={'source_executed':False,'source_sha256':SOURCE_SHA256,'audit_sha256':hashlib.sha256(a.audit.read_bytes()).hexdigest(),
           'host_c_sha256':hashlib.sha256(source.encode('ascii')).hexdigest(),'request_bytes':232,'reply_bytes':792,
           'code_region':'one relocated 64 KiB allocation; shared delta from E80000',
           'constant_regions':['01470000','01480000'],'max_steps':8,'pool_nodes':64,
           'limits':'synthetic checked valid finite inputs only; no game process'}
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf8');print(json.dumps(manifest))
if __name__=='__main__':main()
