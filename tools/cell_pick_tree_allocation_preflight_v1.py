"""Source-free VirtualAlloc alignment preflight; executes no game bytes."""
import ctypes,json
from ctypes import wintypes
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.VirtualAlloc.argtypes=[ctypes.c_void_p,ctypes.c_size_t,wintypes.DWORD,wintypes.DWORD];k.VirtualAlloc.restype=ctypes.c_void_p
k.VirtualFree.argtypes=[ctypes.c_void_p,ctypes.c_size_t,wintypes.DWORD];k.VirtualFree.restype=wintypes.BOOL
results=[]
for requested in [0x5a9000,0xe87000]:
 ctypes.set_last_error(0);p=k.VirtualAlloc(requested,4096,0x3000,4); error=ctypes.get_last_error()
 freed=bool(k.VirtualFree(p,0,0x8000)) if p else None
 results.append({'requested':f'{requested:08X}','returned':f'{p:08X}' if p else None,
                 'winerror':error,'freed':freed,'equal':p==requested})
print(json.dumps({'mode':'source-free-allocation-only','results':results},indent=2))
