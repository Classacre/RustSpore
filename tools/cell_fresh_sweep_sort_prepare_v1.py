"""Build only: original BBA7C0 fresh merge with installed pinned MSVCR90 qsort."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-fresh-sort-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=Path(r'D:\Games\SPORE Collection\SporeBin\msvcr90.dll');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';assert h(D.read_bytes())=='ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185'
p=pefile.PE(str(S));get=lambda a,n:p.get_data(a-p.OPTIONAL_HEADER.ImageBase,n);f=get(0xbba7c0,511);probe=get(0x11e04f0,43);compare=get(0xbba700,29);assert h(f)=='46084f94dc1d8fb34cedf4f8f79b8912300bbf25e7ef8c033ed8b0f6e178a908';assert h(probe)=='f8675b4976f7b8efc0bd917aebab734d11ca90222d20bcd2a166ce9914ed2b6e';assert h(compare)=='b05619e6f5ded95238efc063c573fcbaf63b743a1d815931a032cfb25b2f8ee9'
for n,b in [('sort.bin',f),('probe.bin',probe),('compare.bin',compare)]:(O/n).write_bytes(b)
b=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
cases=[[(0x10000,0,b(0),b(0),b(3),b(3)),(0x20001,4,b(1),b(1),b(2),b(2))],[(0x10000,1,b(0),b(-3),b(1),b(-1)),(0x20001,2,b(2),b(2),b(4),b(6))],[(0x80010000,0xffffffff,b(-2),b(2),b(-1),b(6)),(0x20001,0,b(0),b(-3),b(1),b(-1))]];corpus=b''.join(struct.pack('<6I',*r) for case in cases for r in case);(O/'cases.bin').write_bytes(corpus)
def db(x):return '\n'.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+24]) for i in range(0,len(x),24))
assert f[5]==0xe8 and f[0x1c]==0x68 and f[0x25:0x27]==b'\xff\x15'
parts=[db(f[:6]),' DD StackProbe - ($ + 4)',db(f[10:0x1d]),' DD OFFSET Comparator',db(f[0x21:0x27]),' DD OFFSET QsortPtr',db(f[0x2b:])];frag='\n'.join(parts)
blocks=[]
for ci,case in enumerate(cases):
 setup=[]
 for i,(actor,tag,x,y,X,Y) in enumerate(case):
  setup += [f' mov dword ptr [Slots+{i*16}],0{actor:08X}h',f' mov dword ptr [Slots+{i*16+4}],0{tag:08X}h',f' mov dword ptr [Slots+{i*16+8}],0{((0x8000+i*2)|((0x8000+i*2)<<16)):08X}h',f' mov dword ptr [Slots+{i*16+12}],0{((0x8001+i*2)|((0x8001+i*2)<<16)):08X}h']
  for axis,lo,hi in [('X',x,X),('Y',y,Y)]:
   off=i*16;setup += [f' mov dword ptr [New{axis}+{off}],0{(i<<16):08X}h',f' mov dword ptr [New{axis}+{off+4}],0{lo:08X}h',f' mov dword ptr [New{axis}+{off+8}],0{((i<<16)|1):08X}h',f' mov dword ptr [New{axis}+{off+12}],0{hi:08X}h']
 calls=[]
 for ai,axis in enumerate(['X','Y']):calls.append(f''' push {ai}
 push 4
 push OFFSET New{axis}
 push 0
 push OFFSET Out{axis}
 push OFFSET Manager
 call SortSource
 add esp,24
 cmp esp,SavedESP
 jne Failure
 mov [Capture+{96+ai*4}],eax''')
 blocks.append(f''' mov edi,OFFSET Manager
 xor eax,eax
 mov ecx,14
 rep stosd
 mov dword ptr [Manager+14h],OFFSET Slots
{chr(10).join(setup)}
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
{chr(10).join(calls)}
 cmp esp,SavedESP
 jne Failure
 cmp GuardM,0A5A5A5A5h
 jne Failure
 cmp AfterManager,0A5A5A5A5h
 jne Failure
 cmp GuardS,0A5A5A5A5h
 jne Failure
 cmp AfterSlots,0A5A5A5A5h
 jne Failure
 cmp GuardNX,0A5A5A5A5h
 jne Failure
 cmp AfterNX,0A5A5A5A5h
 jne Failure
 cmp GuardNY,0A5A5A5A5h
 jne Failure
 cmp AfterNY,0A5A5A5A5h
 jne Failure
 cmp GuardOX,0A5A5A5A5h
 jne Failure
 cmp AfterOX,0A5A5A5A5h
 jne Failure
 cmp GuardOY,0A5A5A5A5h
 jne Failure
 cmp AfterOY,0A5A5A5A5h
 jne Failure
 cld
 mov esi,OFFSET Slots
 mov edi,OFFSET Capture
 mov ecx,8
 rep movsd
 mov esi,OFFSET OutX
 mov edi,OFFSET Capture+32
 mov ecx,8
 rep movsd
 mov esi,OFFSET OutY
 mov edi,OFFSET Capture+64
 mov ecx,8
 rep movsd
 fnstcw word ptr [Capture+104]
 fnstsw word ptr [Capture+106]
 stmxcsr [Capture+108]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{ci*112}
 mov ecx,28
 rep movsd''')
pathdw=','.join('0%04Xh'%ord(ch) for ch in str(D)) + ',0'
asm=f'''.686p
.xmm
.model flat
option casemap:none
EXTERN _GetStdHandle@4:PROC
EXTERN _WriteFile@20:PROC
EXTERN _ExitProcess@4:PROC
EXTERN _LoadLibraryExW@12:PROC
EXTERN _GetProcAddress@8:PROC
EXTERN _GetModuleHandleExW@12:PROC
EXTERN _GetModuleFileNameW@12:PROC
EXTERN _lstrcmpiW@8:PROC
EXTERN _SetErrorMode@4:PROC
EXTERN _GetLastError@0:PROC
.data
CW WORD 027Fh
MX DWORD 01F80h
DllPath WORD {pathdw}
QsortName BYTE "qsort",0
Module DWORD 0
ModuleFromProc DWORD 0
QsortPtr DWORD 0
ModulePath WORD 1024 DUP(0)
GuardM DWORD 0A5A5A5A5h
Manager DWORD 14 DUP(0)
AfterManager DWORD 0A5A5A5A5h
GuardS DWORD 0A5A5A5A5h
Slots DWORD 8 DUP(0)
AfterSlots DWORD 0A5A5A5A5h
GuardNX DWORD 0A5A5A5A5h
NewX DWORD 8 DUP(0)
AfterNX DWORD 0A5A5A5A5h
GuardNY DWORD 0A5A5A5A5h
NewY DWORD 8 DUP(0)
AfterNY DWORD 0A5A5A5A5h
GuardOX DWORD 0A5A5A5A5h
OutX DWORD 8 DUP(0)
AfterOX DWORD 0A5A5A5A5h
GuardOY DWORD 0A5A5A5A5h
OutY DWORD 8 DUP(0)
AfterOY DWORD 0A5A5A5A5h
Capture DWORD 28 DUP(0)
Results DWORD 84 DUP(0)
SavedESP DWORD 0
Written DWORD 0
FailureCode DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
 push 3
 call _SetErrorMode@4
 push 8
 push 0
 push OFFSET DllPath
 call _LoadLibraryExW@12
 test eax,eax
 jz LoaderFailure
 mov Module,eax
 push OFFSET QsortName
 push eax
 call _GetProcAddress@8
 test eax,eax
 jz LoaderFailure
 mov QsortPtr,eax
 push OFFSET ModuleFromProc
 push eax
 push 6
 call _GetModuleHandleExW@12
 test eax,eax
 jz LoaderFailure
 mov eax,ModuleFromProc
 cmp eax,Module
 jne LoaderFailure
 push 1024
 push OFFSET ModulePath
 push Module
 call _GetModuleFileNameW@12
 test eax,eax
 jz LoaderFailure
 cmp eax,1024
 jae LoaderFailure
 push OFFSET DllPath
 push OFFSET ModulePath
 call _lstrcmpiW@8
 test eax,eax
 jnz LoaderFailure
{chr(10).join(blocks)}
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 336
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,336
 jne Failure
 push 0
 call _ExitProcess@4
LoaderFailure:
 call _GetLastError@0
 mov FailureCode,eax
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 336
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 112
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
SortSource PROC
{frag}
SortSource ENDP
StackProbe PROC
{db(probe)}
StackProbe ENDP
Comparator PROC
{db(compare)}
Comparator ENDP
END HostMain
''';(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'sort.bin',O/'probe.bin',O/'compare.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',D,sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs',R/'crates/spore-core/src/cell_sweep_endpoint_compare_v1.rs',R/'crates/spore-core/examples/cell_fresh_sweep_sort_oracle_v1.rs',R/'target/debug/examples/cell_fresh_sweep_sort_oracle_v1.exe'];m={'host_executed':False,'source_sha256':h(S.read_bytes()),'ranges':['BBA7C0..BBA9BE','11E04F0..11E051A','BBA700..BBA71C'],'patch_offsets':['0x6 probe rel32','0x1d comparator ptr','0x27 qsort IAT ptr'],'pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'old_count=0, two synthetic slots, unique finite normal/zero, no tombstones, guarded disjoint buffers. Exact installed CRT qsort path/owner checked, OS loader and module memory integrity admitted. No mutation/lifetime or live actor claims.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
