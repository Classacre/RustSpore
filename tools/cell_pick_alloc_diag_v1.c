/* Source-free x86 address-space diagnostic. Maps no original code. */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef unsigned int U;
static HANDLE output;
static void line(const char *tag,U a,U b,U c,U d) {
 static char text[100]; static const char hex[]="0123456789ABCDEF";
 U i=0,j,k,n; const char *s=tag; while(*s)text[i++]=*s++;
 U values[4]; values[0]=a; values[1]=b; values[2]=c; values[3]=d;
 for(j=0;j<4;j++){text[i++]=' ';for(k=0;k<8;k++)text[i++]=hex[(values[j]>>(28-4*k))&15];}
 text[i++]='\r';text[i++]='\n'; if(!WriteFile(output,text,i,(DWORD *)&n,0)||n!=i)ExitProcess(5);
}
void __cdecl HostMain(void) {
 MEMORY_BASIC_INFORMATION m; U addresses[2]={0x005A0000u,0x00E80000u};U i,error,freed;void *p;
 output=GetStdHandle(STD_OUTPUT_HANDLE); if(output==INVALID_HANDLE_VALUE)ExitProcess(2);
 for(i=0;i<2;i++) {
  if(!VirtualQuery((void *)addresses[i],&m,sizeof(m)))ExitProcess(3);
  line("query requested base allocationbase state type",addresses[i],(U)m.BaseAddress,(U)m.AllocationBase,m.State);
  line("region size protect allocationprotect type",(U)m.RegionSize,m.Protect,m.AllocationProtect,m.Type);
  SetLastError(0);p=VirtualAlloc((void *)addresses[i],65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE);
  error=GetLastError();freed=p ? VirtualFree(p,0,MEM_RELEASE) : 0;
  line("allocation requested returned error free",addresses[i],(U)p,error,freed);
 }
 ExitProcess(0);
}
