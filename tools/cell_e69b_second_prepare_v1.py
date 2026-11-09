"""Build only E69B85..E69BA4 synthetic second-lookup result host."""
from pathlib import Path
import hashlib,json,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-e69b-second-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
p=pefile.PE(str(S));f=p.get_data(0xe69b85-p.OPTIONAL_HEADER.ImageBase,32);assert h(f)=='29b3ab8bf80c0278a54cd8d95376e6a9ff9348e47283f2c1f727cc185a3614e0'
(O/'fragment.bin').write_bytes(f);cases=[(0,0,0),(0,0xa5,0x5a),(1,0xa5,0x5a),(1,0xff,0xff)];(O/'cases.bin').write_bytes(b''.join(bytes(c) for c in cases))
def db(b):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in b[i:i+24]) for i in range(0,len(b),24))
assert f[4:6]==b'\x0f\x84';frag=db(f[:6])+'\n DD Missing - ($ + 4)\n'+db(f[10:])
blocks=[]
for i,(present,a,b) in enumerate(cases):
 blocks.append(f''' mov byte ptr [ActorA+178h],{a}
 mov byte ptr [ActorB+179h],{b}
 mov byte ptr [ActorB+17Fh],{b}
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 mov ebp,OFFSET ActorA
 mov ebx,1
 mov eax,{('OFFSET ActorB' if present else '0')}
 call SourceFragment
 mov [Capture+12],eax
 movzx eax,byte ptr [ActorA+178h]
 mov [Capture],eax
 movzx eax,byte ptr [ActorB+179h]
 mov [Capture+4],eax
 movzx eax,byte ptr [ActorB+17Fh]
 mov [Capture+8],eax
 fnstcw word ptr [Capture+16]
 fnstsw word ptr [Capture+18]
 stmxcsr [Capture+20]
 cmp esp,SavedESP
 jne Failure
 cmp GuardA,0A5A5A5A5h
 jne Failure
 cmp EndA,0A5A5A5A5h
 jne Failure
 cmp GuardB,0A5A5A5A5h
 jne Failure
 cmp EndB,0A5A5A5A5h
 jne Failure
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{i*24}
 mov ecx,6
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
GuardA DWORD 0A5A5A5A5h
ActorA BYTE 180h DUP(0)
EndA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
ActorB BYTE 180h DUP(0)
EndB DWORD 0A5A5A5A5h
Capture DWORD 6 DUP(0)
Results DWORD 24 DUP(0)
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
 push 96
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,96
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 96
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 24
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
SourceFragment PROC
{frag}
 xor eax,eax
 ret
Missing:
 mov eax,1
 ret
SourceFragment ENDP
END HostMain
'''
(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib')
cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'fragment.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-bevy/src/cell_source_case10_event_v1.rs',R/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs',R/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs',R/'crates/spore-bevy/examples/cell_e69b_second_oracle_v1.rs',R/'target/debug/examples/cell_e69b_second_oracle_v1.exe']
m={'host_executed':False,'source_sha256':h(S.read_bytes()),'range':'E69B85..E69BA4','patch':'E69B89 near JE to synthetic missing return','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Admitted lookup result, guarded actors, BL=1. No B71CC0, 4098A0, E57890, dispatch or actor lifetime.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
