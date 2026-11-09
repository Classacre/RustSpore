/* Review-only adapter inventory: no windows, device creation or rendering. */
#define COBJMACROS
#include <windows.h>
#include <d3d9.h>
#include <stdio.h>
int main(void) {
 IDirect3D9 *d; HMODULE module; char path[MAX_PATH]; DWORD session=0; UINT count,i; HRESULT hr; D3DADAPTER_IDENTIFIER9 id;
 printf("pointer_bits=%u\n",(unsigned)(8*sizeof(void*)));
 if(ProcessIdToSessionId(GetCurrentProcessId(),&session))printf("session_id=%lu\n",(unsigned long)session);else printf("session_error=%lu\n",(unsigned long)GetLastError());
 d=Direct3DCreate9(D3D_SDK_VERSION);module=GetModuleHandleA("d3d9.dll");
 if(module&&GetModuleFileNameA(module,path,MAX_PATH))printf("d3d9_module=%s\n",path);else printf("d3d9_module_error=%lu\n",(unsigned long)GetLastError());
 printf("direct3d_create9=%s\n",d?"success":"null");if(!d)return 2;
 count=IDirect3D9_GetAdapterCount(d);printf("adapter_count=%u\n",count);
 for(i=0;i<count;i++) {
  ZeroMemory(&id,sizeof id);hr=IDirect3D9_GetAdapterIdentifier(d,i,0,&id);
  printf("adapter=%u hr=%08lX",i,(unsigned long)hr);
  if(SUCCEEDED(hr))printf(" vendor=%04lX device=%04lX description=\"%s\" driver=\"%s\" driver_version=%08lX:%08lX",(unsigned long)id.VendorId,(unsigned long)id.DeviceId,id.Description,id.Driver,(unsigned long)id.DriverVersion.HighPart,(unsigned long)id.DriverVersion.LowPart);
  printf("\n");
 }
 IDirect3D9_Release(d);return 0;
}
