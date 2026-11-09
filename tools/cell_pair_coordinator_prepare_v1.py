"""Build-only E7F170 coordinator host with deterministic stubs. Never execute source bytes."""
import hashlib,json,struct,subprocess
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'target/cell-pair-coordinator-prep-v2';SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
sha=lambda b:hashlib.sha256(b).hexdigest();S='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';F='f69a63d2cfddb13fc1aa2454e1b6edd6cadc2c12bbacebfead9ce278969642ed'
assert not OUT.exists();OUT.mkdir(parents=True);assert sha(SOURCE.read_bytes())==S
p=pefile.PE(str(SOURCE));f=p.get_data(0xe7f170-p.OPTIONAL_HEADER.ImageBase,284);assert sha(f)==F
(OUT/'fragment.bin').write_bytes(f)
# equal IDs, A/B +B4, first/second scripted selector, float bits.
cases=[(1,2,2,9,10,0x80000000),(0,0,2,9,10,0x80000000),(0,2,0,9,10,0x80000000),(0,2,2,9,0,0x80000000),(0,2,2,9,22,0x80000000),(0,2,2,9,10,0x80000000),(0,2,2,0,19,0x80000000)]
corpus=b''.join(struct.pack('<6I',*x) for x in cases);(OUT/'cases.bin').write_bytes(corpus)
def db(v):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in v[i:i+24]) for i in range(0,len(v),24))
# Replace only relative call/jump operands; each opcode and all source-internal
# branches retain the original byte positions. Stub labels are same section.
patches=[(0xe7f18e,'HolderInit'),(0xe7f19f,'Resolve'),(0xe7f1ad,'HolderInit'),(0xe7f1be,'Resolve'),(0xe7f203,'Selector'),(0xe7f211,'Selector'),(0xe7f233,'Event'),(0xe7f23f,'Cleanup'),(0xe7f24f,'Cleanup'),(0xe7f258,'Cleanup'),(0xe7f268,'Cleanup'),(0xe7f271,'Cleanup'),(0xe7f281,'Cleanup')]
fr=[];last=0
for address,target in patches:
 off=address-0xe7f170;assert f[off] in (0xe8,0xe9);fr.append(db(f[last:off+1]));fr.append(f' DD {target} - ($ + 4)');last=off+5
fr.append(db(f[last:]));fragment='\n'.join(fr)
blocks=[]
for i,(equal,aa,bb,s1,s2,fl) in enumerate(cases):
 off=i*180
 blocks.append(f''' mov dword ptr [ActorB],0{(0x10000 if equal else 0x20001):08X}h
 mov dword ptr [DefA+0B4h],0{aa:08X}h
 mov dword ptr [DefB+0B4h],0{bb:08X}h
 mov FirstSelect,0{s1:08X}h
 mov SecondSelect,0{s2:08X}h
 mov Bad,0
 mov edi,OFFSET Capture
 mov eax,0CCCCCCCCh
 mov ecx,45
 rep stosd
 mov dword ptr [Capture],0
 mov dword ptr [Capture+4],0
 mov dword ptr [Capture+8],0
 mov dword ptr [Capture+12],0
 mov dword ptr [Capture+16],0
 mov dword ptr [Capture+20],0
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 sub esp,100h
 mov edi,esp
 mov eax,0A5A5A5A5h
 mov ecx,64
 rep stosd
 add esp,0C0h
 lea eax,[esp-40h]
 mov ExpectedSelection,eax
 lea eax,[esp-0Ch]
 mov ExpectedHolderA,eax
 lea eax,[esp-08h]
 mov ExpectedHolderB,eax
 push 0{fl:08X}h
 push OFFSET ActorB
 push OFFSET ActorA
 call SourceFragment
 add esp,0Ch
 sub esp,0C0h
 cmp dword ptr [esp],0A5A5A5A5h
 jne Failure
 cmp dword ptr [esp+0FCh],0A5A5A5A5h
 jne Failure
 add esp,100h
 cmp esp,SavedESP
 jne Failure
 cmp Bad,0
 jne Failure
 fnstcw word ptr [Capture+172]
 fnstsw word ptr [Capture+174]
 stmxcsr [Capture+176]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{off}
 mov ecx,45
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
ActorA DWORD 000010000h
 BYTE 104h DUP(0)
RefA DWORD 11
EndA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
ActorB DWORD 000020001h
 BYTE 104h DUP(0)
RefB DWORD 22
EndB DWORD 0A5A5A5A5h
DefAGuard DWORD 0A5A5A5A5h
DefA BYTE 0B4h DUP(0)
DefAB4 DWORD 0
DefAEnd DWORD 0A5A5A5A5h
DefBGuard DWORD 0A5A5A5A5h
DefB BYTE 0B4h DUP(0)
DefBB4 DWORD 0
DefBEnd DWORD 0A5A5A5A5h
Capture DWORD 45 DUP(0)
Results DWORD {len(cases)*45} DUP(0)
ExpectedSelection DWORD 0
ExpectedHolderA DWORD 0
ExpectedHolderB DWORD 0
FirstSelect DWORD 0
SecondSelect DWORD 0
SavedESP DWORD 0
Bad DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
{chr(10).join(blocks)}
 cmp GuardA,0A5A5A5A5h
 jne Failure
 cmp EndA,0A5A5A5A5h
 jne Failure
 cmp GuardB,0A5A5A5A5h
 jne Failure
 cmp EndB,0A5A5A5A5h
 jne Failure
 cmp DefAGuard,0A5A5A5A5h
 jne Failure
 cmp DefAEnd,0A5A5A5A5h
 jne Failure
 cmp DefBGuard,0A5A5A5A5h
 jne Failure
 cmp DefBEnd,0A5A5A5A5h
 jne Failure
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push {len(cases)*180}
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,{len(cases)*180}
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 ; Preserve completed rows and current partial capture even on guard failure.
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 1260
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 180
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
HolderInit PROC
 mov eax,ecx
 cmp ecx,ExpectedHolderA
 je InitOk
 cmp ecx,ExpectedHolderB
 jne StubBad
InitOk:
 mov dword ptr [ecx],0
 ret
HolderInit ENDP
Resolve PROC
 mov eax,[esp+4]
 mov edx,[esp+8]
 cmp eax,11
 jne ResolveB
 cmp edx,ExpectedHolderA
 jne StubBad
 mov dword ptr [edx],11110000h
 mov eax,OFFSET DefA
 shl dword ptr [Capture+4],4
 or dword ptr [Capture+4],1
 inc dword ptr [Capture]
 ret
ResolveB:
 cmp eax,22
 jne StubBad
 cmp edx,ExpectedHolderB
 jne StubBad
 mov dword ptr [edx],22220000h
 mov eax,OFFSET DefB
 shl dword ptr [Capture+4],4
 or dword ptr [Capture+4],2
 inc dword ptr [Capture]
 ret
Resolve ENDP
Selector PROC
 mov eax,[esp+4]
 cmp eax,ExpectedSelection
 jne StubBad
 cmp dword ptr [Capture+8],0
 jne SelectorSecond
 cmp dword ptr [esp+8],OFFSET ActorA
 jne StubBad
 cmp dword ptr [esp+12],OFFSET ActorB
 jne StubBad
 cmp dword ptr [esp+16],OFFSET DefA
 jne StubBad
 cmp dword ptr [esp+20],OFFSET DefB
 jne StubBad
 push esi
 push edi
 mov esi,eax
 mov edi,OFFSET Capture+24
 mov ecx,12
 rep movsd
 pop edi
 pop esi
 mov eax,ExpectedSelection
 mov ecx,FirstSelect
 mov [eax+8],ecx
 mov dword ptr [Capture+12],1
 inc dword ptr [Capture+8]
 ret
SelectorSecond:
 cmp dword ptr [Capture+8],1
 jne StubBad
 cmp dword ptr [esp+8],OFFSET ActorB
 jne StubBad
 cmp dword ptr [esp+12],OFFSET ActorA
 jne StubBad
 cmp dword ptr [esp+16],OFFSET DefB
 jne StubBad
 cmp dword ptr [esp+20],OFFSET DefA
 jne StubBad
 push esi
 push edi
 mov esi,eax
 mov edi,OFFSET Capture+72
 mov ecx,12
 rep movsd
 pop edi
 pop esi
 mov eax,ExpectedSelection
 mov ecx,SecondSelect
 mov [eax+8],ecx
 shl dword ptr [Capture+12],4
 or dword ptr [Capture+12],2
 inc dword ptr [Capture+8]
 ret
Selector ENDP
Event PROC
 mov eax,[esp+4]
 cmp eax,ExpectedSelection
 jne StubBad
 push esi
 push edi
 mov esi,eax
 mov edi,OFFSET Capture+120
 mov ecx,12
 rep movsd
 pop edi
 pop esi
 mov eax,[esp+8]
 mov [Capture+168],eax
 inc dword ptr [Capture+16]
 ret
Event ENDP
Cleanup PROC
 cmp ecx,ExpectedHolderA
 je CleanupOk
 cmp ecx,ExpectedHolderB
 jne StubBad
CleanupOk:
 inc dword ptr [Capture+20]
 ret
Cleanup ENDP
StubBad:
 mov Bad,1
 xor eax,eax
 ret
SourceFragment PROC
{fragment}
SourceFragment ENDP
END HostMain
'''
(OUT/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib')
cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{OUT}/host-x86.obj',str(OUT/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{OUT}/host-x86.exe',f'/MAP:{OUT}/host-x86.map',str(OUT/'host-x86.obj'),str(lib)]]
logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(OUT/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failure')
paths=[Path(__file__),OUT/'fragment.bin',OUT/'cases.bin',OUT/'host-x86.asm',OUT/'host-x86.exe',OUT/'host-x86.obj',OUT/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,ROOT/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs',ROOT/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs']
m={'host_executed':False,'source_sha256':S,'fragment_sha256':F,'range':'E7F170..E7F28B','patched_call_sites':[[hex(a),t] for a,t in patches],'pins':{str(x):sha(x.read_bytes()) for x in paths},'commands':logs,'limits':'Synthetic stub script, no original resolver/selector/event/cleanup or gameplay. Independent whole-host patch, CFG, stack and ABI review required.'}
(OUT/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
