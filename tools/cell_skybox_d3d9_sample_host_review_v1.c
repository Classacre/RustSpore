/* REVIEW REQUIRED: controlled fixed-function per-mip point sampling; no window created/presented. */
#define main unused_upload_fixture_main
#include "cell_skybox_d3d9_upload_host_review_v1.c"
#undef main
__declspec(dllexport) DWORD NvOptimusEnablement=1;
typedef struct { float x,y,z,rhw,u,v; } Vertex;
static unsigned char pixels[349524];
#define CHECK(expr) do { hr=(expr); if(FAILED(hr)){fprintf(stderr,"failed line=%d hr=%08lX\n",__LINE__,(unsigned long)hr);goto finish;} } while(0)
int main(void) {
 IDirect3D9 *d=NULL; IDirect3DDevice9 *dev=NULL; IDirect3DTexture9 *tex=NULL; IDirect3DSurface9 *rt=NULL,*sys=NULL;
 D3DPRESENT_PARAMETERS pp; D3DADAPTER_IDENTIFIER9 id; D3DLOCKED_RECT lock; D3DVIEWPORT9 vp; HRESULT hr; BOOL debug=FALSE;
 unsigned i,j,found=0,adapter=0,off=0,outn=0,w; DWORD header[5]={0x31504d53,0,0xffffffff,0xffffffff,0}; int status=1;
 _setmode(_fileno(stdin),_O_BINARY);_setmode(_fileno(stdout),_O_BINARY);
 if(IsDebuggerPresent()||!CheckRemoteDebuggerPresent(GetCurrentProcess(),&debug)||debug)goto finish;
 if(fread(input,1,sizeof input,stdin)!=sizeof input||fgetc(stdin)!=EOF||memcmp(input+0x124,"DXT5",4)||memcmp(input+0x130,"\x00\x01\x00\x01",4)||input[0x135]!=9)goto finish;
 size_code=code(size_bytes,sizeof size_bytes,1);copy_code=code(copy_bytes,sizeof copy_bytes,0);if(!size_code||!copy_code)goto finish;
 d=Direct3DCreate9(D3D_SDK_VERSION);if(!d)goto finish;
 for(i=0;i<IDirect3D9_GetAdapterCount(d);i++)if(SUCCEEDED(IDirect3D9_GetAdapterIdentifier(d,i,0,&id))&&id.VendorId==0x10de&&id.DeviceId==0x2520&&strstr(id.Description,"RTX 3060")){adapter=i;found++;}
 if(found!=1)goto finish;header[1]=1;header[2]=adapter;header[3]=0x252010de;
 memset(&pp,0,sizeof pp);pp.Windowed=TRUE;pp.SwapEffect=D3DSWAPEFFECT_DISCARD;pp.hDeviceWindow=GetDesktopWindow();pp.BackBufferWidth=1;pp.BackBufferHeight=1;pp.BackBufferFormat=D3DFMT_UNKNOWN;pp.PresentationInterval=D3DPRESENT_INTERVAL_IMMEDIATE;
 CHECK(IDirect3D9_CreateDevice(d,adapter,D3DDEVTYPE_HAL,GetDesktopWindow(),D3DCREATE_HARDWARE_VERTEXPROCESSING|D3DCREATE_MULTITHREADED,&pp,&dev));
 CHECK(IDirect3DDevice9_CreateTexture(dev,256,256,9,0,D3DFMT_DXT5,D3DPOOL_MANAGED,&tex,NULL));
 for(i=0;i<9;i++){w=256u>>i;if(!w)w=1;if(source_size(input+0x124,i)!=lengths[i]||off+lengths[i]>87408)goto finish;CHECK(IDirect3DTexture9_LockRect(tex,i,&lock,NULL,0));if(!lock.pBits||lock.Pitch!=(INT)(((w+3)/4)*16)){IDirect3DTexture9_UnlockRect(tex,i);goto finish;}source_copy(lock.pBits,input+0x174+off,lengths[i]);CHECK(IDirect3DTexture9_UnlockRect(tex,i));off+=lengths[i];}
 header[1]=2;CHECK(IDirect3DDevice9_CreateRenderTarget(dev,256,256,D3DFMT_A8R8G8B8,D3DMULTISAMPLE_NONE,0,FALSE,&rt,NULL));CHECK(IDirect3DDevice9_CreateOffscreenPlainSurface(dev,256,256,D3DFMT_A8R8G8B8,D3DPOOL_SYSTEMMEM,&sys,NULL));CHECK(IDirect3DDevice9_SetRenderTarget(dev,0,rt));
 CHECK(IDirect3DDevice9_SetRenderState(dev,D3DRS_ZENABLE,FALSE));CHECK(IDirect3DDevice9_SetRenderState(dev,D3DRS_ALPHABLENDENABLE,FALSE));CHECK(IDirect3DDevice9_SetRenderState(dev,D3DRS_ALPHATESTENABLE,FALSE));CHECK(IDirect3DDevice9_SetRenderState(dev,D3DRS_CULLMODE,D3DCULL_NONE));CHECK(IDirect3DDevice9_SetRenderState(dev,D3DRS_SRGBWRITEENABLE,FALSE));CHECK(IDirect3DDevice9_SetRenderState(dev,D3DRS_DITHERENABLE,FALSE));
 CHECK(IDirect3DDevice9_SetTexture(dev,0,(IDirect3DBaseTexture9*)tex));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_MINFILTER,D3DTEXF_POINT));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_MAGFILTER,D3DTEXF_POINT));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_MIPFILTER,D3DTEXF_POINT));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_ADDRESSU,D3DTADDRESS_CLAMP));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_ADDRESSV,D3DTADDRESS_CLAMP));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_SRGBTEXTURE,0));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_MIPMAPLODBIAS,0));
 CHECK(IDirect3DDevice9_SetTextureStageState(dev,0,D3DTSS_COLOROP,D3DTOP_SELECTARG1));CHECK(IDirect3DDevice9_SetTextureStageState(dev,0,D3DTSS_COLORARG1,D3DTA_TEXTURE));CHECK(IDirect3DDevice9_SetTextureStageState(dev,0,D3DTSS_ALPHAOP,D3DTOP_SELECTARG1));CHECK(IDirect3DDevice9_SetTextureStageState(dev,0,D3DTSS_ALPHAARG1,D3DTA_TEXTURE));CHECK(IDirect3DDevice9_SetTextureStageState(dev,1,D3DTSS_COLOROP,D3DTOP_DISABLE));CHECK(IDirect3DDevice9_SetFVF(dev,D3DFVF_XYZRHW|D3DFVF_TEX1));
 for(i=0;i<9;i++) { Vertex v[4];w=256u>>i;if(!w)w=1;
  v[0]=(Vertex){-.5f,-.5f,0,1,0,0};v[1]=(Vertex){(float)w-.5f,-.5f,0,1,1,0};v[2]=(Vertex){-.5f,(float)w-.5f,0,1,0,1};v[3]=(Vertex){(float)w-.5f,(float)w-.5f,0,1,1,1};
  vp.X=vp.Y=0;vp.Width=vp.Height=w;vp.MinZ=0;vp.MaxZ=1;CHECK(IDirect3DDevice9_SetViewport(dev,&vp));CHECK(IDirect3DDevice9_SetSamplerState(dev,0,D3DSAMP_MAXMIPLEVEL,i));CHECK(IDirect3DDevice9_BeginScene(dev));hr=IDirect3DDevice9_DrawPrimitiveUP(dev,D3DPT_TRIANGLESTRIP,2,v,sizeof(Vertex));if(FAILED(hr)){IDirect3DDevice9_EndScene(dev);goto finish;}CHECK(IDirect3DDevice9_EndScene(dev));CHECK(IDirect3DDevice9_GetRenderTargetData(dev,rt,sys));CHECK(IDirect3DSurface9_LockRect(sys,&lock,NULL,D3DLOCK_READONLY));if(!lock.pBits||lock.Pitch<1024){IDirect3DSurface9_UnlockRect(sys);goto finish;}
  for(j=0;j<w;j++)memcpy(pixels+outn+j*w*4,(unsigned char*)lock.pBits+j*lock.Pitch,w*4);CHECK(IDirect3DSurface9_UnlockRect(sys));outn+=w*w*4;header[4]=i+1;
 }
 header[1]=3;status=0;
finish:
 if(sys)IDirect3DSurface9_Release(sys);if(rt)IDirect3DSurface9_Release(rt);if(tex)IDirect3DTexture9_Release(tex);if(dev)IDirect3DDevice9_Release(dev);if(d)IDirect3D9_Release(d);
 fwrite(header,1,sizeof header,stdout);fwrite(pixels,1,outn,stdout);fflush(stdout);fprintf(stderr,"stage=%lu levels=%lu bytes=%u status=%d\n",(unsigned long)header[1],(unsigned long)header[4],outn,status);return status;
}
