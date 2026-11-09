/* REVIEW REQUIRED. Enumerates D3D9 display modes without device/window/presentation. */
#define COBJMACROS
#include <windows.h>
#include <d3d9.h>
#include <stdio.h>
__declspec(dllexport) DWORD NvOptimusEnablement=1;
int main(void) {
 IDirect3D9 *d=Direct3DCreate9(D3D_SDK_VERSION);D3DADAPTER_IDENTIFIER9 id;D3DDISPLAYMODE desktop,m;UINT i,j,adapter=0,found=0,count;HRESULT hr;const D3DFORMAT formats[]={D3DFMT_A2R10G10B10,D3DFMT_X8R8G8B8,D3DFMT_X1R5G5B5,D3DFMT_R5G6B5};
 if(!d){fprintf(stderr,"Direct3DCreate9 failed\n");return 1;}
 for(i=0;i<IDirect3D9_GetAdapterCount(d);i++)if(SUCCEEDED(IDirect3D9_GetAdapterIdentifier(d,i,0,&id))&&id.VendorId==0x10de&&id.DeviceId==0x2520){adapter=i;found++;}
 if(found!=1){fprintf(stderr,"RTX adapter count=%u\n",found);IDirect3D9_Release(d);return 2;}
 hr=IDirect3D9_GetAdapterDisplayMode(d,adapter,&desktop);if(FAILED(hr)){fprintf(stderr,"displaymode hr=%08lX\n",(unsigned long)hr);IDirect3D9_Release(d);return 3;}
 printf("adapter=%u vendor=10DE device=2520 desktop=%u,%u,%u,%u\n",adapter,desktop.Width,desktop.Height,desktop.RefreshRate,desktop.Format);
 for(i=0;i<4;i++){
  count=IDirect3D9_GetAdapterModeCount(d,adapter,formats[i]);printf("format=%u count=%u\n",formats[i],count);if(count>65536){fprintf(stderr,"mode bound exceeded\n");IDirect3D9_Release(d);return 4;}
  for(j=0;j<count;j++){hr=IDirect3D9_EnumAdapterModes(d,adapter,formats[i],j,&m);if(FAILED(hr)){fprintf(stderr,"enum format=%u index=%u hr=%08lX\n",formats[i],j,(unsigned long)hr);IDirect3D9_Release(d);return 5;}printf("mode=%u,%u,%u,%u,%u,%u\n",formats[i],j,m.Width,m.Height,m.RefreshRate,m.Format);}
 }
 IDirect3D9_Release(d);return 0;
}
