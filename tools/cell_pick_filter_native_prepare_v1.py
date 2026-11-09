"""Generate, never execute, bounded E6B560 original-instruction host."""
import argparse,hashlib,json
from pathlib import Path
from cell_schema_probe import SourceImage,SOURCE_SHA256
from cell_pick_filter_native_audit_v1 import CODE
from cell_pick_tree_native_prepare_v1 import byte_array
HOST=r'''
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef unsigned int U;
typedef struct { U player,candidate,player_size,candidate_size,flag0,flag1,kind,progress; } Request;
typedef struct { U accepted,player_calls,cache_calls,temporary_calls,definition_calls,cleanup_calls; } Reply;
static struct { U pre[4]; Request value; U post[4]; } input;
static Request saved;static Reply reply;
static struct { U pre[4]; unsigned char bytes[0x5194]; U post[4]; } root;
static struct { U pre[4]; unsigned char bytes[0x35c]; U post[4]; } player,candidate;
static struct { U pre[4]; unsigned char bytes[0x1254]; U post[4]; } cache;
static struct { U pre[4]; unsigned char bytes[0xb8]; U post[4]; } definition;
static U progress[8];static unsigned char *code;static HANDLE hin,hout;
static U p_calls,c_calls,t_calls,d_calls,f_calls,expected_handle,expected_manager;
__declspec(naked) static void PlayerStub(void) { __asm {
 inc p_calls
 cmp ecx,expected_manager
 jne player_fail
 mov eax,dword ptr [esp+4]
 cmp eax,expected_handle
 jne player_fail
 mov eax,offset player
 add eax,16
 ret 4
player_fail:
 push 61
 call ExitProcess
} }
__declspec(naked) static void CacheStub(void) { __asm {
 inc c_calls
 cmp dword ptr [esp+4],0
 jne cache_fail
 cmp dword ptr [esp+8],0
 jne cache_fail
 cmp dword ptr [esp+12],0
 jne cache_fail
 mov eax,offset cache
 add eax,16
 ret
cache_fail:
 push 62
 call ExitProcess
} }
__declspec(naked) static void TempStub(void) { __asm {
 inc t_calls
 ret
} }
__declspec(naked) static void DefStub(void) { __asm {
 inc d_calls
 cmp dword ptr [esp+4],0x10203040
 jne def_fail
 mov eax,offset definition
 add eax,16
 ret
def_fail:
 push 63
 call ExitProcess
} }
__declspec(naked) static void FreeStub(void) { __asm {
 inc f_calls
 ret
} }
static void fail(U x){ExitProcess(x);}
static void write_u(unsigned char *p,U offset,U v){*(U *)(p+offset)=v;}
static int read_record(void){U n=0,c;unsigned char *p=(unsigned char *)&input.value;
 while(n<sizeof(Request)){if(!ReadFile(hin,p+n,sizeof(Request)-n,(DWORD *)&c,0)){if(!n&&GetLastError()==ERROR_BROKEN_PIPE)return 0;fail(31);}if(!c){if(!n)return 0;fail(32);}n+=c;}return 1;}
static void write_record(void){U n=0,c;unsigned char *p=(unsigned char *)&reply;
 while(n<sizeof(Reply)){if(!WriteFile(hout,p+n,sizeof(Reply)-n,(DWORD *)&c,0)||!c)fail(33);n+=c;}}
static void allocate_fixed(U address,U readonly_table){U i,old;unsigned char *p=(unsigned char *)VirtualAlloc((void *)address,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
 if((U)p!=address)fail(21);if(readonly_table){for(i=0;i<sizeof(source_table);i++)p[0x46c4+i]=source_table[i];
 if(!VirtualProtect(p,65536,PAGE_READONLY,(DWORD *)&old))fail(22);}}
static void stub(U offset,void *target){code[offset]=0xb8;*(U *)(code+offset+1)=(U)target;code[offset+5]=0xff;code[offset+6]=0xe0;}
static void init_code(void){U i,old;static const U pages[]={0,0x3c0000,0x690000,0x6a0000,0x6b0000,0x6d0000};
 code=(unsigned char *)VirtualAlloc(0,0x6e0000,MEM_RESERVE,PAGE_NOACCESS);if(!code)fail(23);
 for(i=0;i<6;i++)if(VirtualAlloc(code+pages[i],65536,MEM_COMMIT,PAGE_READWRITE)!=code+pages[i])fail(24);
 COPY_CODE
 stub(0x3c1d00,PlayerStub);stub(0x6b7a50,CacheStub);stub(0x2350,TempStub);stub(0x69caa0,DefStub);stub(0x6d2160,FreeStub);
 for(i=0;i<6;i++)if(!VirtualProtect(code+pages[i],65536,PAGE_EXECUTE_READ,(DWORD *)&old)||!FlushInstructionCache(GetCurrentProcess(),code+pages[i],65536))fail(25);
}
void __cdecl HostMain(void){U i;typedef U (__cdecl *Run)(void *);
 hin=GetStdHandle(STD_INPUT_HANDLE);hout=GetStdHandle(STD_OUTPUT_HANDLE);
 allocate_fixed(0x1480000,1);allocate_fixed(0x16b0000,0);init_code();
 while(read_record()){
  if(input.value.progress>1000||input.value.flag0>255||input.value.flag1>255)fail(34);
  for(i=0;i<4;i++)input.pre[i]=input.post[i]=root.pre[i]=root.post[i]=player.pre[i]=player.post[i]=candidate.pre[i]=candidate.post[i]=cache.pre[i]=cache.post[i]=definition.pre[i]=definition.post[i]=0xa5a5a5a5u;
  for(i=0;i<sizeof(root.bytes);i++)root.bytes[i]=0;
  for(i=0;i<sizeof(player.bytes);i++)player.bytes[i]=candidate.bytes[i]=0;
  for(i=0;i<sizeof(cache.bytes);i++)cache.bytes[i]=0;
  for(i=0;i<sizeof(definition.bytes);i++)definition.bytes[i]=0;
  for(i=0;i<8;i++)progress[i]=0;
  write_u(root.bytes,0x411c,input.value.player);write_u(root.bytes,0x5190,(U)progress);
  write_u(player.bytes,0x358,input.value.player_size);write_u(candidate.bytes,0,input.value.candidate);
  write_u(candidate.bytes,0x358,input.value.candidate_size);write_u(candidate.bytes,0x108,0x10203040);
  cache.bytes[0x1250]=(unsigned char)input.value.flag0;cache.bytes[0x1251]=(unsigned char)input.value.flag1;
  write_u(definition.bytes,0xb4,input.value.kind);progress[7]=input.value.progress;
  *(U *)0x16b4be4=(U)root.bytes;expected_handle=input.value.player;expected_manager=(U)(root.bytes+0x1c);
  p_calls=c_calls=t_calls=d_calls=f_calls=0;saved=input.value;
  reply.accepted=((Run)(code+0x6bb560))(candidate.bytes)&255u;
  reply.player_calls=p_calls;reply.cache_calls=c_calls;reply.temporary_calls=t_calls;reply.definition_calls=d_calls;reply.cleanup_calls=f_calls;
  for(i=0;i<4;i++)if(input.pre[i]!=0xa5a5a5a5u||input.post[i]!=0xa5a5a5a5u||root.pre[i]!=0xa5a5a5a5u||root.post[i]!=0xa5a5a5a5u||player.pre[i]!=0xa5a5a5a5u||player.post[i]!=0xa5a5a5a5u||candidate.pre[i]!=0xa5a5a5a5u||candidate.post[i]!=0xa5a5a5a5u||cache.pre[i]!=0xa5a5a5a5u||cache.post[i]!=0xa5a5a5a5u||definition.pre[i]!=0xa5a5a5a5u||definition.post[i]!=0xa5a5a5a5u)fail(35);
  for(i=0;i<sizeof(Request)/4;i++)if(((U *)&saved)[i]!=((U *)&input.value)[i])fail(36);
  write_record();
 }ExitProcess(0);
}
'''
def main():
 p=argparse.ArgumentParser();p.add_argument('exe',type=Path);p.add_argument('--audit',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 root=Path(__file__).resolve().parent.parent;out=a.out.resolve();out.relative_to((root/'target').resolve());out.mkdir(parents=True,exist_ok=False)
 img=SourceImage(a.exe);audit=json.loads(a.audit.read_text());assert hashlib.sha256(a.exe.read_bytes()).hexdigest()==SOURCE_SHA256==audit['source_sha256']
 source_table=img.read(0x14846c4,0x254);assert hashlib.sha256(source_table).hexdigest()==audit['table_sha256']
 decl=[byte_array('source_table',source_table)];copies=[]
 for i,(start,end) in enumerate(CODE):
  data=img.read(start,end-start);assert hashlib.sha256(data).hexdigest()==audit['ranges'][i]['sha256']
  decl.append(byte_array(f'fragment_{i}',data));copies.append(f'for(i=0;i<sizeof(fragment_{i});i++)code[0x{start-0x7b0000:x}+i]=fragment_{i}[i];')
 source=HOST.replace('typedef unsigned int U;',''.join(decl)+'\ntypedef unsigned int U;').replace('COPY_CODE','\n '.join(copies))
 (out/'host.c').write_text(source,encoding='ascii',newline='\n')
 report={'source_executed':False,'source_sha256':SOURCE_SHA256,'audit_sha256':hashlib.sha256(a.audit.read_bytes()).hexdigest(),
  'source_table_sha256':audit['table_sha256'],'host_c_sha256':hashlib.sha256(source.encode()).hexdigest(),
  'request_bytes':32,'reply_bytes':24,'mapping':'shared code delta plus owned provider stubs; pinned fixed source table and global',
  'scope':'supplied synthetic actors/provider outputs; no live resources or game process'}
 (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
