/* REVIEW REQUIRED. Fixed candidate, no created window, no Present. */
#define COBJMACROS
#include <windows.h>
#include <d3d9.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
#include "source_bytes.h"
static unsigned char *size_code,*copy_code;
__declspec(naked) static uint32_t __cdecl source_size(void *raster,uint32_t mip) {
 __asm {
  mov ecx,[esp+4]
  push dword ptr [esp+8]
  call dword ptr [size_code]
  ret
 }
}
__declspec(naked) static void __cdecl source_copy(void *dst,const void *src,uint32_t count) {
 __asm {
  push esi
  push edi
  mov edi,[esp+12]
  mov esi,[esp+16]
  mov eax,[esp+20]
  cld
  call dword ptr [copy_code]
  pop edi
  pop esi
  ret
 }
}
typedef struct { uint32_t size,desc_hr,width,height,format,lock_hr,pitch,unlock_hr,read_hr,read_pitch,read_unlock_hr,match; } Level;
typedef struct { uint32_t magic,stage,vendor,device,adapter,create_hr; Level level[9]; } Result;
static const uint32_t lengths[9]={65536,16384,4096,1024,256,64,16,16,16};
static unsigned char input[87780],readback[87408];
static unsigned char *code(const unsigned char *src,size_t n,int trap) {
 DWORD old; unsigned char *p=(unsigned char*)VirtualAlloc(NULL,n+3,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE);
 if(!p)return NULL; memcpy(p,src,n); if(trap){p[n]=0xcc;p[n+1]=0x0f;p[n+2]=0x0b;}else p[n]=0xc3;
 if(!VirtualProtect(p,n+3,PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),p,n+3))return NULL; return p;
}
int main(void) {
 Result r; IDirect3D9 *d=NULL; IDirect3DDevice9 *dev=NULL; IDirect3DTexture9 *tex=NULL;
 D3DPRESENT_PARAMETERS pp; D3DADAPTER_IDENTIFIER9 id; D3DSURFACE_DESC desc; D3DLOCKED_RECT lock;
 unsigned i,found=0,adapter=0,offset=0,readn=0; HRESULT hr; int status=1; BOOL debug=FALSE;
 memset(&r,0xff,sizeof r); r.magic=0x31583944; r.stage=0; _setmode(_fileno(stdin),_O_BINARY); _setmode(_fileno(stdout),_O_BINARY);
 if(IsDebuggerPresent()||!CheckRemoteDebuggerPresent(GetCurrentProcess(),&debug)||debug)goto done;
 if(fread(input,1,sizeof input,stdin)!=sizeof input||fgetc(stdin)!=EOF)goto done;
 /* Admit only the pinned candidate's fixed Raster shape and resource span. */
 if(memcmp(input+0x124,"DXT5",4)||memcmp(input+0x130,"\x00\x01\x00\x01",4)||input[0x135]!=9)goto done;
 size_code=code(size_bytes,sizeof size_bytes,1); copy_code=code(copy_bytes,sizeof copy_bytes,0); if(!size_code||!copy_code)goto done;
 r.stage=1; d=Direct3DCreate9(D3D_SDK_VERSION); if(!d)goto done;
 for(i=0;i<IDirect3D9_GetAdapterCount(d);i++) if(SUCCEEDED(IDirect3D9_GetAdapterIdentifier(d,i,0,&id))&&id.VendorId==0x10de&&strstr(id.Description,"RTX 3060")){found++;adapter=i;r.vendor=id.VendorId;r.device=id.DeviceId;}
 if(found!=1)goto done; r.adapter=adapter; memset(&pp,0,sizeof pp); pp.Windowed=TRUE;pp.SwapEffect=D3DSWAPEFFECT_DISCARD;pp.hDeviceWindow=GetDesktopWindow();pp.BackBufferWidth=1;pp.BackBufferHeight=1;pp.BackBufferFormat=D3DFMT_UNKNOWN;pp.PresentationInterval=D3DPRESENT_INTERVAL_IMMEDIATE;
 hr=IDirect3D9_CreateDevice(d,adapter,D3DDEVTYPE_HAL,GetDesktopWindow(),D3DCREATE_HARDWARE_VERTEXPROCESSING|D3DCREATE_MULTITHREADED,&pp,&dev); if(FAILED(hr))goto done;
 r.stage=2; hr=IDirect3DDevice9_CreateTexture(dev,256,256,9,0,D3DFMT_DXT5,D3DPOOL_MANAGED,&tex,NULL);r.create_hr=(uint32_t)hr;if(FAILED(hr)||!tex)goto done;
 for(i=0;i<9;i++) { Level *l=&r.level[i];uint32_t w=256u>>i,h=256u>>i,pitch;
  if(!w)w=1;if(!h)h=1;pitch=((w+3)/4)*16;l->size=source_size(input+0x124,i);if(l->size!=lengths[i]||offset+l->size>sizeof readback)goto done;
  hr=IDirect3DTexture9_GetLevelDesc(tex,i,&desc);l->desc_hr=(uint32_t)hr;if(FAILED(hr))goto done;l->width=desc.Width;l->height=desc.Height;l->format=desc.Format;if(desc.Width!=w||desc.Height!=h||desc.Format!=D3DFMT_DXT5)goto done;
  hr=IDirect3DTexture9_LockRect(tex,i,&lock,NULL,0);l->lock_hr=(uint32_t)hr;if(FAILED(hr))goto done;l->pitch=(uint32_t)lock.Pitch;if(!lock.pBits||lock.Pitch!=(INT)pitch){IDirect3DTexture9_UnlockRect(tex,i);goto done;}
  source_copy(lock.pBits,input+0x174+offset,l->size);hr=IDirect3DTexture9_UnlockRect(tex,i);l->unlock_hr=(uint32_t)hr;if(FAILED(hr))goto done;
  hr=IDirect3DTexture9_LockRect(tex,i,&lock,NULL,D3DLOCK_READONLY);l->read_hr=(uint32_t)hr;if(FAILED(hr))goto done;l->read_pitch=(uint32_t)lock.Pitch;if(!lock.pBits||lock.Pitch!=(INT)pitch){IDirect3DTexture9_UnlockRect(tex,i);goto done;}
  memcpy(readback+offset,lock.pBits,l->size);readn+=l->size;l->match=!memcmp(readback+offset,input+0x174+offset,l->size);hr=IDirect3DTexture9_UnlockRect(tex,i);l->read_unlock_hr=(uint32_t)hr;if(FAILED(hr)||!l->match)goto done;offset+=l->size;
 }
 r.stage=3;status=0;
done:
 if(tex)IDirect3DTexture9_Release(tex);if(dev)IDirect3DDevice9_Release(dev);if(d)IDirect3D9_Release(d);
 fwrite(&r,1,sizeof r,stdout);fwrite(readback,1,readn,stdout);fflush(stdout);fprintf(stderr,"stage=%u readback_bytes=%u status=%d\n",r.stage,readn,status);return status;
}
