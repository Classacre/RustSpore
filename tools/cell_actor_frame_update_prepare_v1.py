"""Build-only E6DB40 caller with guarded synthetic actor and two stubs."""
from pathlib import Path
import pefile,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-actor-frame-update-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S),fast_load=True);a=0xe6db40;b=p.get_data(a-p.OPTIONAL_HEADER.ImageBase,138);assert h(b)=='b7b8c3f42193e437e02249d601a2db13632f7e78f23253ccefc271e7c3b042a4';(O/'source.bin').write_bytes(b)
cases=[(0xffffffff,1.5,2.5,.5),(0,1.5,2.5,.5),(9,-2.,3.,1.),(0x80000000,0.,-0.,0.)];corpus=b''.join(struct.pack('<I3f',*c) for c in cases);(O/'cases.bin').write_bytes(corpus)
s='''.686p
.xmm
.model flat
option casemap:none
EXTERN _GetStdHandle@4:PROC
EXTERN _WriteFile@20:PROC
EXTERN _ExitProcess@4:PROC
.data
CW WORD 027Fh
MX DWORD 01F80h
GuardA DWORD 0A5A5A5A5h
Actor BYTE 872 DUP(0)
AfterA DWORD 0A5A5A5A5h
GuardC DWORD 0A5A5A5A5h
Cell BYTE 16648 DUP(0)
AfterC DWORD 0A5A5A5A5h
Manager DWORD 012345678h
CellGlobal DWORD OFFSET Cell
Center DWORD 3 DUP(0)
Scalar DWORD 0
Capture DWORD 11 DUP(0)
Results DWORD 44 DUP(0)
SavedESP DWORD 0
Written DWORD 0
FailureCode DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
 mov dword ptr [Cell+4104h],OFFSET Manager
'''
for k,(slot,x,y,scalar) in enumerate(cases):
 vals=struct.unpack('<III',struct.pack('<3f',x,y,scalar))
 s+=f''' mov edi,OFFSET Capture
 xor eax,eax
 mov ecx,11
 rep stosd
 mov dword ptr [Actor+364h],0{slot:08X}h
 mov dword ptr [Center],0{vals[0]:08X}h
 mov dword ptr [Center+4],0{vals[1]:08X}h
 mov dword ptr [Center+8],0
 mov dword ptr [Scalar],0{vals[2]:08X}h
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 mov eax,OFFSET Actor
 call CallerSource
 cmp esp,SavedESP
 jne Failure
 cmp GuardA,0A5A5A5A5h
 jne Failure
 cmp AfterA,0A5A5A5A5h
 jne Failure
 cmp GuardC,0A5A5A5A5h
 jne Failure
 cmp AfterC,0A5A5A5A5h
 jne Failure
 mov eax,dword ptr [Actor+364h]
 mov [Capture+32],eax
 fnstcw word ptr [Capture+36]
 fnstsw word ptr [Capture+38]
 stmxcsr [Capture+40]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{k*44}
 mov ecx,11
 rep movsd
'''
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 176
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,176
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 176
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 44
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 4
 push OFFSET FailureCode
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
GeometryStub PROC
 cmp esi,OFFSET Actor
 jne StubFailure
 mov eax,SavedESP
 sub eax,512
 cmp edi,eax
 jb StubFailure
 mov ecx,SavedESP
 sub ecx,12
 cmp edi,ecx
 ja StubFailure
 mov edx,[esp+4]
 cmp edx,eax
 jb StubFailure
 cmp edx,ecx
 ja StubFailure
 inc dword ptr [Capture]
 mov eax,[Center]
 mov [edi],eax
 mov eax,[Center+4]
 mov [edi+4],eax
 mov eax,[Center+8]
 mov [edi+8],eax
 mov eax,[Scalar]
 mov [edx],eax
 ret
GeometryStub ENDP
UpdateStub PROC
 cmp dword ptr [esp+4],OFFSET Manager
 jne StubFailure
 mov eax,SavedESP
 sub eax,512
 mov ecx,SavedESP
 sub ecx,8
 mov edx,[esp+12]
 cmp edx,eax
 jb StubFailure
 cmp edx,ecx
 ja StubFailure
 mov edx,[esp+16]
 cmp edx,eax
 jb StubFailure
 cmp edx,ecx
 ja StubFailure
 inc dword ptr [Capture+4]
 mov dword ptr [Capture+8],1
 mov eax,[esp+8]
 mov [Capture+12],eax
 mov edx,[esp+12]
 mov eax,[edx]
 mov [Capture+16],eax
 mov eax,[edx+4]
 mov [Capture+20],eax
 mov edx,[esp+16]
 mov eax,[edx]
 mov [Capture+24],eax
 mov eax,[edx+4]
 mov [Capture+28],eax
 ret
UpdateStub ENDP
StubFailure PROC
 mov FailureCode,0BAD00001h
 push 7
 call _ExitProcess@4
StubFailure ENDP
CallerSource PROC
'''
# direct geometry call, absolute global pointer, direct update call.
patch=[(0xe6db59-a,5,'call','GeometryStub'),(0xe6db88-a+2,4,'abs','CellGlobal'),(0xe6dbbc-a,5,'call','UpdateStub')];pos=0
def db(x):return ''.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+20])+'\n' for i in range(0,len(x),20))
for off,n,kind,target in patch:
 if kind=='call':assert b[off]==0xe8;s+=db(b[pos:off+1])+f' DD {target} - ($ + 4)\n'
 else:s+=db(b[pos:off])+f' DD OFFSET {target}\n'
 pos=off+n
s+=db(b[pos:])+'''CallerSource ENDP
END HostMain
''';(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'source.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S];m={'host_executed':False,'range':'E6DB40..E6DBC9','patches':'E6D7F0 call -> geometry stub, global address -> synthetic Cell pointer, BBBD10 call -> update recording stub','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'synthetic actor/manager identity; stubs admit geometry, freshness and update effects; no actual source actor or cache, E6E085 caller or gameplay'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
