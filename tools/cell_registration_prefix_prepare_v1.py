"""Build only conditional E74F93..E7501D registration-prefix host."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-registration-prefix-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S));f=p.get_data(0xe74f93-p.OPTIONAL_HEADER.ImageBase,139);assert h(f)=='1dfb287d6f32cb234db45c1077978173465b5b8115aecf428d8cdc3beed22aa8';(O/'fragment.bin').write_bytes(f)
b=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
cases=[(9,9,0,2,0x10000,b(100),b(-2),b(.5)),(9,10,0,2,0x10000,b(2),b(3),b(1)),(9,9,0xffffffff,5,0x20001,b(4),b(5),b(1)),(9,9,0xffffffff,7,0x20001,b(4),b(5),b(1)),(9,9,0xffffffff,2,0x20001,b(4),b(5),b(1)),(9,9,22,2,0x20001,b(16777216),b(5),b(1)),(9,9,0,2,0x20001,0x80000000,0,0),(9,9,0,2,0x20001,b(1),b(-1),b(1))];(O/'cases.bin').write_bytes(b''.join(struct.pack('<8I',*x) for x in cases))
def db(b):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in b[i:i+24]) for i in range(0,len(b),24))
assert f[10:12]==b'\x0f\x85';frag=db(f[:12])+'\n DD Skipped - ($ + 4)\n'+db(f[16:])
blocks=[]
for i,(orig,current,arg,b4,handle,x,y,r) in enumerate(cases):
 blocks.append(f''' mov dword ptr [Manager+40FCh],0{current:08X}h
 mov dword ptr [Definition+0B4h],0{b4:08X}h
 mov dword ptr [Actor],0{handle:08X}h
 mov Tag,0CCCCCCCCh
 mov edi,OFFSET Capture
 mov eax,0CCCCCCCCh
 mov ecx,8
 rep stosd
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 sub esp,100h
 mov edi,esp
 mov eax,0A5A5A5A5h
 mov ecx,64
 rep stosd
 add esp,50h
 mov dword ptr [esp+0Ch],OFFSET Definition
 mov dword ptr [esp+84h],0{arg:08X}h
 mov dword ptr [esp+2Ch],0{x:08X}h
 mov dword ptr [esp+30h],0{y:08X}h
 mov dword ptr [esp+7Ch],0{r:08X}h
 mov eax,OFFSET Manager
 mov esi,0{orig:08X}h
 mov edi,0{handle:08X}h
 mov ebp,OFFSET Actor
 call SourceFragment
 mov [Capture],eax
 test eax,eax
 jz SkipCopy{i}
 mov eax,Tag
 mov [Capture+4],eax
 mov eax,[esp+14h]
 mov [Capture+8],eax
 mov eax,[esp+18h]
 mov [Capture+12],eax
 mov eax,[esp+24h]
 mov [Capture+16],eax
 mov eax,[esp+28h]
 mov [Capture+20],eax
SkipCopy{i}:
 sub esp,50h
 cmp dword ptr [esp],0A5A5A5A5h
 jne Failure
 cmp dword ptr [esp+0FCh],0A5A5A5A5h
 jne Failure
 add esp,100h
 cmp esp,SavedESP
 jne Failure
 cmp GuardManager,0A5A5A5A5h
 jne Failure
 cmp EndManager,0A5A5A5A5h
 jne Failure
 cmp GuardDefinition,0A5A5A5A5h
 jne Failure
 cmp EndDefinition,0A5A5A5A5h
 jne Failure
 cmp GuardActor,0A5A5A5A5h
 jne Failure
 cmp EndActor,0A5A5A5A5h
 jne Failure
 fnstcw word ptr [Capture+24]
 fnstsw word ptr [Capture+26]
 stmxcsr [Capture+28]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{i*32}
 mov ecx,8
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
GuardManager DWORD 0A5A5A5A5h
Manager BYTE 4100h DUP(0)
EndManager DWORD 0A5A5A5A5h
GuardDefinition DWORD 0A5A5A5A5h
Definition BYTE 0B8h DUP(0)
EndDefinition DWORD 0A5A5A5A5h
GuardActor DWORD 0A5A5A5A5h
Actor DWORD 0
EndActor DWORD 0A5A5A5A5h
Tag DWORD 0
Capture DWORD 8 DUP(0)
Results DWORD 64 DUP(0)
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
 push 256
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,256
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 256
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 32
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
SourceFragment PROC
{frag}
 mov eax,[esp+8]
 mov Tag,eax
 add esp,12
 mov eax,1
 ret
Skipped:
 xor eax,eax
 ret
SourceFragment ENDP
END HostMain
''';(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'fragment.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-core/src/cell_registration_prefix_v1.rs',R/'crates/spore-core/examples/cell_registration_prefix_oracle_v1.rs',R/'target/debug/examples/cell_registration_prefix_oracle_v1.exe'];m={'host_executed':False,'source_sha256':h(S.read_bytes()),'range':'E74F93..E7501D','patch':'E74F9D jne to synthetic skipped exit','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Synthetic manager, actor, stack, stable disjoint inputs; no E86960, pool or BBB7F0.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
