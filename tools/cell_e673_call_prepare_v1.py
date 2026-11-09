"""Build-only post-arbitration x87 handoff host; never execute original bytes."""
import hashlib,json,struct,subprocess
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'target/cell-e673-call-prep-v1';SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
sha=lambda b:hashlib.sha256(b).hexdigest();b=lambda f:struct.unpack('<I',struct.pack('<f',f))[0]
S='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';F='0ce625c52a945160508c11f0c76ebde1678592b8d4a11d63b3b54b313f39d6c6'
assert not OUT.exists();OUT.mkdir(parents=True);assert sha(SOURCE.read_bytes())==S
pe=pefile.PE(str(SOURCE));frag=pe.get_data(0xe6a548-pe.OPTIONAL_HEADER.ImageBase,75);assert sha(frag)==F and frag[7:11]==bytes.fromhex('e44b6b01')
(OUT/'fragment.bin').write_bytes(frag)
z=0;cases=[([b(3.),b(4.),z],[z,z,z]),([0x80000000,z,z],[z,z,z]),([1,z,z],[z,z,z]),([b(1.),b(1.),b(1.)],[z,z,z]),([b(16777216.),b(1.),b(1.)],[z,z,z]),([b(-3.),b(2.),b(7.)],[b(1.),b(-1.),b(7.)])]
corpus=b''.join(struct.pack('<6I',*a,*q) for a,q in cases);(OUT/'cases.bin').write_bytes(corpus)
def db(data):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in data[i:i+24]) for i in range(0,len(data),24))
blocks=[]
for i in range(len(cases)):
 c=i*24;o=i*28
 blocks.append(f''' fninit
 fldcw word ptr [CW]
 ldmxcsr dword ptr [MX]
 mov eax,dword ptr [Cases+{c}]
 mov [ActorA+90h],eax
 mov eax,dword ptr [Cases+{c+4}]
 mov [ActorA+94h],eax
 mov eax,dword ptr [Cases+{c+8}]
 mov [ActorA+98h],eax
 mov eax,dword ptr [Cases+{c+12}]
 mov [ActorB+90h],eax
 mov eax,dword ptr [Cases+{c+16}]
 mov [ActorB+94h],eax
 mov eax,dword ptr [Cases+{c+20}]
 mov [ActorB+98h],eax
 mov edi,OFFSET ActorA
 mov esi,OFFSET ActorB
 mov ebp,OFFSET Point
 mov ebx,{o}
 mov SavedESP,esp
 call SourceFragment
 cmp esp,SavedESP
 jne Failure''')
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
GuardRoot DWORD 0A5A5A5A5h
Root BYTE 5190h DUP(0)
RootLink DWORD OFFSET State
EndRoot DWORD 0A5A5A5A5h
GuardState DWORD 0A5A5A5A5h
State BYTE 74h DUP(0)
Current DWORD 012345678h
EndState DWORD 0A5A5A5A5h
RootPtr DWORD OFFSET Root
GuardA DWORD 0A5A5A5A5h
ActorA DWORD 000010000h
 BYTE 98h DUP(0)
EndA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
ActorB DWORD 000020001h
 BYTE 98h DUP(0)
EndB DWORD 0A5A5A5A5h
GuardPoint DWORD 0A5A5A5A5h
Point DWORD 11,22,33
EndPoint DWORD 0A5A5A5A5h
Cases LABEL BYTE
{db(corpus)}
Results DWORD {len(cases)*7} DUP(0CCCCCCCCh)
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
{chr(10).join(blocks)}
 cmp GuardRoot,0A5A5A5A5h
 jne Failure
 cmp EndRoot,0A5A5A5A5h
 jne Failure
 cmp GuardState,0A5A5A5A5h
 jne Failure
 cmp EndState,0A5A5A5A5h
 jne Failure
 cmp GuardA,0A5A5A5A5h
 jne Failure
 cmp EndA,0A5A5A5A5h
 jne Failure
 cmp GuardB,0A5A5A5A5h
 jne Failure
 cmp EndB,0A5A5A5A5h
 jne Failure
 cmp GuardPoint,0A5A5A5A5h
 jne Failure
 cmp EndPoint,0A5A5A5A5h
 jne Failure
 cmp RootLink,OFFSET State
 jne Failure
 cmp RootPtr,OFFSET Root
 jne Failure
 cmp Current,012345678h
 jne Failure
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push {len(cases)*28}
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,{len(cases)*28}
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push 7
 call _ExitProcess@4
HostMain ENDP
SourceFragment PROC
{db(frag[:7])}
 DD OFFSET RootPtr
{db(frag[11:])}
 mov [Results+ebx+8],eax
 mov [Results+ebx+4],ecx
 mov eax,[esp]
 mov [Results+ebx],eax
 mov eax,[esp+4]
 mov [Results+ebx+12],eax
 fnstcw word ptr [Results+ebx+20]
 fnstsw word ptr [Results+ebx+22]
 stmxcsr [Results+ebx+24]
 xor eax,eax
 cmp ebp,OFFSET Point
 sete al
 mov [Results+ebx+16],eax
 add esp,8
 ret
SourceFragment ENDP
END HostMain
'''
(OUT/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib')
cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{OUT}/host-x86.obj',str(OUT/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{OUT}/host-x86.exe',f'/MAP:{OUT}/host-x86.map',str(OUT/'host-x86.obj'),str(lib)]]
logs=[]
for cmd in cmds:
 p=subprocess.run(cmd,capture_output=True,text=True);logs.append({'args':cmd,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
 if p.returncode:(OUT/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failure')
paths=[Path(__file__),OUT/'fragment.bin',OUT/'cases.bin',OUT/'host-x86.asm',OUT/'host-x86.exe',OUT/'host-x86.obj',OUT/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,ROOT/'crates/spore-bevy/src/cell_source_contact_call_v1.rs',ROOT/'crates/spore-core/src/native_x87.rs']
m={'host_executed':False,'source_sha256':S,'fragment_sha256':F,'range':'E6A548..E6A592','patch':'global pointer operand at offset 7..10 -> synthetic RootPtr','pins':{str(p):sha(p.read_bytes()) for p in paths},'commands':logs,'limits':'Synthetic finite inputs; host/patch/ABI/runner require independent review; no live actors or callback effects.'}
(OUT/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
