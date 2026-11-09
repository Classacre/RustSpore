/* Source-free x86 reservation and protection test. Copies no original bytes. */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef unsigned int U;
static void report(U base,U first,U second,U result) {
 static char out[50];static const char hex[]="0123456789ABCDEF";U v[4],i,j,k=0,w;
 v[0]=base;v[1]=first;v[2]=second;v[3]=result;
 for(i=0;i<4;i++){if(i)out[k++]=' ';for(j=0;j<8;j++)out[k++]=hex[(v[i]>>(28-4*j))&15];}
 out[k++]='\r';out[k++]='\n';if(!WriteFile(GetStdHandle(STD_OUTPUT_HANDLE),out,k,(DWORD *)&w,0)||w!=k)ExitProcess(2);
}
void __cdecl HostMain(void) {
 U old,ok=0; unsigned char *base=(unsigned char *)VirtualAlloc(0,0x8f0000,MEM_RESERVE,PAGE_NOACCESS);
 if(!base){report(0,0,0,GetLastError());ExitProcess(3);}
 if(VirtualAlloc(base,0x10000,MEM_COMMIT,PAGE_READWRITE)==base &&
    VirtualAlloc(base+0x8e0000,0x10000,MEM_COMMIT,PAGE_READWRITE)==base+0x8e0000 &&
    VirtualProtect(base,0x10000,PAGE_EXECUTE_READ,(DWORD *)&old) &&
    VirtualProtect(base+0x8e0000,0x10000,PAGE_EXECUTE_READ,(DWORD *)&old))ok=1;
 report((U)base,(U)(base+0x9c40),(U)(base+0x8e7170),ok);
 if(!VirtualFree(base,0,MEM_RELEASE))ExitProcess(4);
 ExitProcess(ok?0:5);
}
