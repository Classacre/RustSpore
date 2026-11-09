"""Build only; original gate slice is never executed by this script."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-case10-gate-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
h=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
assert not O.exists();O.mkdir(parents=True)
assert h(S)=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
p=pefile.PE(str(S));f=p.get_data(0xe7e050-p.OPTIONAL_HEADER.ImageBase,0x47)
assert hashlib.sha256(f).hexdigest()=='ba4cd95e8ff5272fd51d096672ca2dbdc70695d850ddfb8192ca6c689971be0b'
(O/'fragment.bin').write_bytes(f)
cases=[(1,1,1,1,1),(0,1,1,1,1),(0,2,1,1,1),(0,2,0,1,1),(0,2,0,0,1),(0,2,0,0,0),(255,0,0,0,0),(0,255,0,0,0)]
(O/'cases.bin').write_bytes(b''.join(bytes(c) for c in cases))
def db(b):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in b[i:i+24]) for i in range(0,len(b),24))
# Relocate only the near JNE true exit and E572E0 call displacements.
assert f[0x12:0x14]==b'\x0f\x85' and f[0x3b]==0xe8
fragment=db(f[:0x14])+'\n DD GateTrue - ($ + 4)\n'+db(f[0x18:0x3c])+'\n DD Relation - ($ + 4)\n'+db(f[0x40:])
blocks=[]
for i,(b179,b111,b17b,a178,rel) in enumerate(cases):
 blocks.append(f''' mov byte ptr [ActorB+179h],{b179}
 mov byte ptr [ActorB+111h],{b111}
 mov byte ptr [ActorB+17Bh],{b17b}
 mov byte ptr [ActorA+178h],{a178}
 mov RelationValue,{rel}
 mov RelationCount,0
 mov Bad,0
 mov dword ptr [Capture],0CCCCCCCCh
 mov dword ptr [Capture+4],0CCCCCCCCh
 mov dword ptr [Capture+8],0CCCCCCCCh
 mov dword ptr [Capture+12],0CCCCCCCCh
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 mov esi,OFFSET ActorB
 push 2
 push 37
 push OFFSET ActorA
 call SourceFragment
 add esp,12
 movzx eax,al
 mov [Capture],eax
 mov eax,RelationCount
 mov [Capture+4],eax
 fnstcw word ptr [Capture+8]
 fnstsw word ptr [Capture+10]
 stmxcsr [Capture+12]
 cmp esp,SavedESP
 jne Failure
 cmp Bad,0
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
 mov edi,OFFSET Results+{i*16}
 mov ecx,4
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
RelationValue DWORD 0
RelationCount DWORD 0
Bad DWORD 0
SavedESP DWORD 0
Capture DWORD 4 DUP(0)
Results DWORD {len(cases)*4} DUP(0)
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
 push 128
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,128
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 128
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 16
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
Relation PROC
 cmp dword ptr [esp+4],OFFSET ActorA
 jne RelationBad
 cmp dword ptr [esp+8],OFFSET ActorB
 jne RelationBad
 inc RelationCount
 mov eax,RelationValue
 ret
RelationBad:
 mov Bad,1
 mov eax,1
 ret
Relation ENDP
SourceFragment PROC
{fragment}
 ; The original continuation begins here; it is deliberately not run.
 mov al,2
 pop ebp
 pop ebx
 pop ecx
 ret
GateTrue:
 mov al,1
 pop ebp
 pop ebx
 pop ecx
 ret
SourceFragment ENDP
END HostMain
'''
(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib')
cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]]
logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'fragment.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs',R/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs',R/'crates/spore-bevy/examples/cell_case10_gate_oracle_v1.rs',R/'target/debug/examples/cell_case10_gate_oracle_v1.exe']
manifest={'host_executed':False,'source_sha256':h(S),'range':'E7E050..E7E096','patches':[['0xe7e062','GateTrue'],['0xe7e08b','Relation']],'pins':{str(x):h(x) for x in paths},'commands':logs,'limits':'Guarded synthetic actors; relation result stub only. No event creation, ownership or callbacks. Review linked bytes, ABI and stubs before execution.'}
(O/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest['pins'],indent=2))
