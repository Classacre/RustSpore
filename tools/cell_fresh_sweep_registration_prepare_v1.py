"""Build-only fresh-append BBAB40 comparison host."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-fresh-registration-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
p=pefile.PE(str(S));f=p.get_data(0xbbab40-p.OPTIONAL_HEADER.ImageBase,239);assert h(f)=='047980ab870d8c5d32f1d76ddf5f7171a8627300ea05ef349d9bd9da2f22e094';(O/'fragment.bin').write_bytes(f)
b=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
cases=[[(0x10000,0,b(0),b(-2),b(1),b(2)),(0x20001,4,b(2),b(3),b(4),b(5))],[(0x80010000,0xffffffff,0x80000000,b(-3),b(1),b(-1)),(0xffffffff,9,b(-2),b(2),b(-1),b(6))]];(O/'cases.bin').write_bytes(b''.join(struct.pack('<6I',*row) for case in cases for row in case))
def db(x):return '\n'.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+24]) for i in range(0,len(x),24))
blocks=[]
for ci,case in enumerate(cases):
 calls=[]
 for ai,(actor,tag,x,y,X,Y) in enumerate(case):
  calls.append(f''' mov dword ptr [Min],0{x:08X}h
 mov dword ptr [Min+4],0{y:08X}h
 mov dword ptr [Max],0{X:08X}h
 mov dword ptr [Max+4],0{Y:08X}h
 push 0{tag:08X}h
 push OFFSET Max
 push OFFSET Min
 push 0{actor:08X}h
 push OFFSET Manager
 call SourceFragment
 add esp,20
 mov [Capture+{108+ai*4}],eax''')
 blocks.append(f''' mov edi,OFFSET Manager
 xor eax,eax
 mov ecx,14
 rep stosd
 mov edi,OFFSET Slots
 mov ecx,8
 rep stosd
 mov edi,OFFSET XAxis
 mov ecx,8
 rep stosd
 mov edi,OFFSET YAxis
 mov ecx,8
 rep stosd
 mov dword ptr [Manager+14h],OFFSET Slots
 mov dword ptr [Manager+30h],OFFSET XAxis
 mov dword ptr [Manager+34h],OFFSET YAxis
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
 cmp GuardX,0A5A5A5A5h
 jne Failure
 cmp EndX,0A5A5A5A5h
 jne Failure
 cmp GuardY,0A5A5A5A5h
 jne Failure
 cmp EndY,0A5A5A5A5h
 jne Failure
 mov eax,[Manager+10h]
 mov [Capture],eax
 mov eax,[Manager+18h]
 mov [Capture+4],eax
 mov eax,[Manager+2Ch]
 mov [Capture+8],eax
 mov esi,OFFSET Slots
 mov edi,OFFSET Capture+12
 mov ecx,8
 rep movsd
 mov esi,OFFSET XAxis
 mov edi,OFFSET Capture+44
 mov ecx,8
 rep movsd
 mov esi,OFFSET YAxis
 mov edi,OFFSET Capture+76
 mov ecx,8
 rep movsd
 fnstcw word ptr [Capture+116]
 fnstsw word ptr [Capture+118]
 stmxcsr [Capture+120]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{ci*124}
 mov ecx,31
 rep movsd''')
asm=f'''.686p
.xmm
.model flat
option casemap:none
EXTERN _GetStdHandle@4:PROC
EXTERN _WriteFile@20:PROC
EXTERN _ExitProcess@4:PROC
.data
CW WORD 027Fh
MX DWORD 01F80h
GuardM DWORD 0A5A5A5A5h
Manager DWORD 14 DUP(0)
AfterManager DWORD 0A5A5A5A5h
GuardS DWORD 0A5A5A5A5h
Slots DWORD 8 DUP(0)
AfterSlots DWORD 0A5A5A5A5h
GuardX DWORD 0A5A5A5A5h
XAxis DWORD 8 DUP(0)
EndX DWORD 0A5A5A5A5h
GuardY DWORD 0A5A5A5A5h
YAxis DWORD 8 DUP(0)
EndY DWORD 0A5A5A5A5h
Min DWORD 2 DUP(0)
Max DWORD 2 DUP(0)
Capture DWORD 31 DUP(0)
Results DWORD 62 DUP(0)
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
{chr(10).join(blocks)}
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 248
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,248
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 248
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 124
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
SourceFragment PROC
{db(f)}
SourceFragment ENDP
END HostMain
''';(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'fragment.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs',R/'crates/spore-core/examples/cell_fresh_sweep_registration_oracle_v1.rs',R/'target/debug/examples/cell_fresh_sweep_registration_oracle_v1.exe'];m={'host_executed':False,'source_sha256':h(S.read_bytes()),'range':'BBAB40..BBAC2E','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Fresh fixed-capacity append, 2 actors, x87 027F, guarded synthetic manager/arrays, stable disjoint inputs; no frees, reuse, persistence or live actor.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
