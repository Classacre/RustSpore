"""Build-only BBAD00 fresh whole-consumer host, source CRT activation required."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-fresh-consumer-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=S.parent/'msvcr90.dll';M=S.parent/'Microsoft.VC90.CRT.manifest';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
for p,pin in [(S,'17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'),(D,'ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185'),(M,'9e0220511d4ebdb014cc17ecb8319d57e3b0fea09681a80d8084aa8647196582')]:assert h(p.read_bytes())==pin
p=pefile.PE(str(S));spans=[(0xbbad00,0xbbae19,'Consumer'),(0xbba7c0,0xbba9bf,'SortSource'),(0x11e04f0,0x11e051b,'StackProbe'),(0xbba700,0xbba71d,'Comparator'),(0xbba9c0,0xbbaa1e,'PairFilter')];frags=[p.get_data(a-p.OPTIONAL_HEADER.ImageBase,z-a) for a,z,_ in spans]
for i,f in enumerate(frags):(O/f'fragment-{i}.bin').write_bytes(f)
def bits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
actors=[(0x80010000,0,0,0,4,20),(0x20001,0,1,1,9,21),(0x30002,0,2,2,8,22),(0x40003,0,5,3,7,23)];filtered=[actors[0][:1]+(7,)+actors[0][2:],actors[1][:1]+(7,)+actors[1][2:],(actors[2][0],0,2,30,8,31),actors[3]]
cases=[(6,actors),(2,actors),(6,filtered),(0,actors)];corpus=b''.join(struct.pack('<I',cap)+b''.join(struct.pack('<6I',a,t,bits(x),bits(y),bits(X),bits(Y)) for a,t,x,y,X,Y in rows) for cap,rows in cases);(O/'cases.bin').write_bytes(corpus)
def db(x):return '\n'.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+24]) for i in range(0,len(x),24))
patches={0:[(0xbbad05,'StackProbe'),(0xbbad31,'SortSource'),(0xbbad91,'PairFilter')],1:[(0xbba7c5,'StackProbe')]};procs=[]
for i,(a,z,name) in enumerate(spans):
 f=frags[i];parts=[];last=0
 if i==1:
  assert f[0x1c]==0x68 and f[0x25:0x27]==b'\xff\x15'
  parts=[db(f[:6]),' DD StackProbe - ($ + 4)',db(f[10:0x1d]),' DD OFFSET Comparator',db(f[0x21:0x27]),' DD OFFSET QsortPtr',db(f[0x2b:])]
 else:
  for addr,target in patches.get(i,[]):
   off=addr-a;assert f[off]==0xe8;parts.extend([db(f[last:off+1]),' DD '+target+' - ($ + 4)']);last=off+5
  parts.append(db(f[last:]))
 procs.append(name+' PROC\n'+'\n'.join(parts)+'\n'+name+' ENDP')
blocks=[]
for ci,(cap,rows) in enumerate(cases):
 setup=[]
 for i,(actor,tag,x,y,X,Y) in enumerate(rows):
  setup += [f' mov dword ptr [Slots+{i*16}],0{actor:08X}h',f' mov dword ptr [Slots+{i*16+4}],0{tag:08X}h',f' mov dword ptr [Slots+{i*16+8}],0{((0x8000+i*2)|((0x8000+i*2)<<16)):08X}h',f' mov dword ptr [Slots+{i*16+12}],0{((0x8001+i*2)|((0x8001+i*2)<<16)):08X}h']
  for axis,lo,hi in [('X',bits(x),bits(X)),('Y',bits(y),bits(Y))]:
   off=i*16;setup += [f' mov dword ptr [New{axis}+{off}],0{(i<<16):08X}h',f' mov dword ptr [New{axis}+{off+4}],0{lo:08X}h',f' mov dword ptr [New{axis}+{off+8}],0{((i<<16)|1):08X}h',f' mov dword ptr [New{axis}+{off+12}],0{hi:08X}h']
 blocks.append(f''' mov edi,OFFSET Manager
 xor eax,eax
 mov ecx,14
 rep stosd
 mov dword ptr [Manager],{cap}
 mov dword ptr [Manager+0Ch],OFFSET PairBuffer
 mov dword ptr [Manager+14h],OFFSET Slots
 mov dword ptr [Manager+24h],OFFSET OutX
 mov dword ptr [Manager+28h],OFFSET OutY
 mov dword ptr [Manager+2Ch],8
 mov dword ptr [Manager+30h],OFFSET NewX
 mov dword ptr [Manager+34h],OFFSET NewY
 mov edi,OFFSET PairBuffer
 mov eax,0CCCCCCCCh
 mov ecx,12
 rep stosd
{chr(10).join(setup)}
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push OFFSET Manager
 call Consumer
 add esp,4
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
 cmp GuardPairs,0A5A5A5A5h
 jne Failure
 cmp AfterPairs,0A5A5A5A5h
 jne Failure
 mov eax,[Manager+8]
 mov [Capture],eax
 mov eax,[Manager+20h]
 mov [Capture+4],eax
 mov eax,[Manager+2Ch]
 mov [Capture+8],eax
 cld
 mov esi,OFFSET Slots
 mov edi,OFFSET Capture+12
 mov ecx,16
 rep movsd
 mov esi,OFFSET OutX
 mov edi,OFFSET Capture+76
 mov ecx,16
 rep movsd
 mov esi,OFFSET OutY
 mov edi,OFFSET Capture+140
 mov ecx,16
 rep movsd
 mov esi,OFFSET PairBuffer
 mov edi,OFFSET Capture+204
 mov ecx,12
 rep movsd
 fnstcw word ptr [Capture+252]
 fnstsw word ptr [Capture+254]
 stmxcsr [Capture+256]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{ci*260}
 mov ecx,65
 rep movsd''')
def dw(path):return ','.join('0%04Xh'%ord(c) for c in str(path))+',0'
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
EXTERN _CreateActCtxW@4:PROC
EXTERN _ActivateActCtx@8:PROC
.data
CW WORD 027Fh
MX DWORD 01F80h
DllPath WORD {dw(D)}
QsortName BYTE "qsort",0
SourceExe WORD {dw(S)}
AssemblyDir WORD {dw(S.parent)}
ActivationContext DWORD 32,0Ch,OFFSET SourceExe,0,OFFSET AssemblyDir,1,0,0
ActivationHandle DWORD 0
ActivationCookie DWORD 0
Module DWORD 0
ModuleFromProc DWORD 0
QsortPtr DWORD 0
ModulePath WORD 1024 DUP(0)
GuardM DWORD 0A5A5A5A5h
Manager DWORD 14 DUP(0)
AfterManager DWORD 0A5A5A5A5h
GuardS DWORD 0A5A5A5A5h
Slots DWORD 16 DUP(0)
AfterSlots DWORD 0A5A5A5A5h
GuardNX DWORD 0A5A5A5A5h
NewX DWORD 16 DUP(0)
AfterNX DWORD 0A5A5A5A5h
GuardNY DWORD 0A5A5A5A5h
NewY DWORD 16 DUP(0)
AfterNY DWORD 0A5A5A5A5h
GuardOX DWORD 0A5A5A5A5h
OutX DWORD 16 DUP(0)
AfterOX DWORD 0A5A5A5A5h
GuardOY DWORD 0A5A5A5A5h
OutY DWORD 16 DUP(0)
AfterOY DWORD 0A5A5A5A5h
GuardPairs DWORD 0A5A5A5A5h
PairBuffer DWORD 12 DUP(0)
AfterPairs DWORD 0A5A5A5A5h
Capture DWORD 65 DUP(0)
Results DWORD 260 DUP(0)
SavedESP DWORD 0
Written DWORD 0
FailureCode DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
 push 3
 call _SetErrorMode@4
 push OFFSET ActivationContext
 call _CreateActCtxW@4
 cmp eax,0FFFFFFFFh
 je LoaderFailure
 mov ActivationHandle,eax
 push OFFSET ActivationCookie
 push eax
 call _ActivateActCtx@8
 test eax,eax
 jz LoaderFailure
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
 push 1040
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,1040
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
 push 1040
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 260
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
{chr(10).join(procs)}
END HostMain
''';(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),*[O/f'fragment-{i}.bin' for i in range(5)],O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S,D,M,R/'target/cell-fresh-sort-activated-result-v1/receipt.json',R/'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs',R/'crates/spore-core/src/cell_sweep_endpoint_compare_v1.rs',R/'crates/spore-core/src/cell_sweep_pair_filter_v1.rs',R/'crates/spore-core/src/cell_source_sweep_pairs_v1.rs',R/'crates/spore-core/examples/cell_fresh_sweep_consumer_oracle_v1.rs',R/'target/debug/examples/cell_fresh_sweep_consumer_oracle_v1.exe'];m={'host_executed':False,'source_sha256':h(S.read_bytes()),'spans':[[hex(a),hex(z-1),n] for a,z,n in spans],'pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Fresh old_count=0, four slots, unique finite endpoints, guarded disjoint arrays, capacity 0/2/6; direct pinned CRT qsort with source activation and private desktop. No live or persistent history.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
