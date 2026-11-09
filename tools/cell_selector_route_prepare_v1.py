"""Build-only E73E10 route dispatch with guarded synthetic actor stubs."""
from pathlib import Path
import pefile,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-selector-route-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S),fast_load=True);a=0xe73e10;b=p.get_data(a-p.OPTIONAL_HEADER.ImageBase,118);assert h(b)=='2cc56aca7c975b20691022b59f968afe50e1dd03997cc1ae97e4f45051455dde';(O/'source.bin').write_bytes(b)
cases=[((i>>3)&1,(i>>2)&1,(i>>1)&1,i&1) for i in range(16)]+[(0x80000000,0xffffffff,0xffffffff,0x80000000)];(O/'cases.bin').write_bytes(b''.join(struct.pack('<4I',*r) for r in cases))
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
ActorA BYTE 588 DUP(0)
AfterA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
ActorB BYTE 588 DUP(0)
AfterB DWORD 0A5A5A5A5h
Capture DWORD 16 DUP(0)
Results DWORD 272 DUP(0)
CallIndex DWORD 0
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
'''
for k,row in enumerate(cases):
 s+=' mov edi,OFFSET Capture\n xor eax,eax\n mov ecx,16\n rep stosd\n mov CallIndex,0\n'
 for actor,j in [('ActorA',0),('ActorB',2)]:
  s+=f' mov dword ptr [{actor}+248h],0{row[j]:08X}h\n mov dword ptr [{actor}+0FCh],0{row[j+1]:08X}h\n'
 s+=''' fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push 022222222h
 push OFFSET ActorB
 push OFFSET ActorA
 push 011111111h
 call RouteSource
 add esp,16
 cmp esp,SavedESP
 jne Failure
 push 022222222h
 push OFFSET ActorA
 push OFFSET ActorB
 push 011111111h
 call RouteSource
 add esp,16
 cmp esp,SavedESP
 jne Failure
 cmp CallIndex,2
 jne Failure
 cmp GuardA,0A5A5A5A5h
 jne Failure
 cmp AfterA,0A5A5A5A5h
 jne Failure
 cmp GuardB,0A5A5A5A5h
 jne Failure
 cmp AfterB,0A5A5A5A5h
 jne Failure
 mov eax,dword ptr [ActorA+248h]
 mov [Capture+40],eax
 mov eax,dword ptr [ActorA+0FCh]
 mov [Capture+44],eax
 mov eax,dword ptr [ActorB+248h]
 mov [Capture+48],eax
 mov eax,dword ptr [ActorB+0FCh]
 mov [Capture+52],eax
 fnstcw word ptr [Capture+56]
 fnstsw word ptr [Capture+58]
 stmxcsr [Capture+60]
'''
 s+=f' mov esi,OFFSET Capture\n mov edi,OFFSET Results+{k*64}\n mov ecx,16\n rep movsd\n'
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 1088
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,1088
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 1088
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 64
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
RouteBoth PROC
 mov eax,1
 jmp RecordRoute
RouteBoth ENDP
RouteSecond PROC
 mov eax,2
 jmp RecordRoute
RouteSecond ENDP
RouteFirst PROC
 mov eax,3
 jmp RecordRoute
RouteFirst ENDP
RouteNeither PROC
 mov eax,4
 jmp RecordRoute
RouteNeither ENDP
RecordRoute PROC
 mov edx,CallIndex
 cmp edx,2
 jae StubFailure
 imul edx,20
 mov [Capture+edx],eax
 cmp dword ptr [esp+4],011111111h
 jne StubFailure
 mov dword ptr [Capture+edx+4],1
 cmp dword ptr [esp+16],022222222h
 jne StubFailure
 mov dword ptr [Capture+edx+16],1
 cmp CallIndex,0
 jne ReverseOrder
 cmp dword ptr [esp+8],OFFSET ActorA
 jne StubFailure
 cmp dword ptr [esp+12],OFFSET ActorB
 jne StubFailure
 jmp OrderValid
ReverseOrder:
 cmp dword ptr [esp+8],OFFSET ActorB
 jne StubFailure
 cmp dword ptr [esp+12],OFFSET ActorA
 jne StubFailure
OrderValid:
 mov dword ptr [Capture+edx+8],1
 mov dword ptr [Capture+edx+12],1
 inc CallIndex
 ret
RecordRoute ENDP
StubFailure PROC
 push 7
 call _ExitProcess@4
StubFailure ENDP
RouteSource PROC
'''
def db(x):return ''.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+20])+'\n' for i in range(0,len(x),20))
pos=0
for address,target in [(0xe73e44,'RouteBoth'),(0xe73e51,'RouteSecond'),(0xe73e74,'RouteFirst'),(0xe73e81,'RouteNeither')]:
 off=address-a;assert b[off]==0xe9;s+=db(b[pos:off+1])+f' DD {target} - ($ + 4)\n';pos=off+5
s+=db(b[pos:])+'''RouteSource ENDP
END HostMain
''';(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'source.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S];m={'host_executed':False,'range':'E73E10..E73E85','patches':'four tail jumps to controlled route recording stubs','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'synthetic actor fields and explicit call args, no route callee/contact/callback or actor identity'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
