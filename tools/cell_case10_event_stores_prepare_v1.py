"""Build-only admitted E51D40 field-store slices; no host execution."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-case10-event-stores-prep-v3';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
p=pefile.PE(str(S));spans=[(0xe51d6f,0xe51dc8),(0xe51dd1,0xe51de2),(0xe51ded,0xe51dfe),(0xe51e09,0xe51e32),(0xe51e33,0xe51e3c)]
fragments=[p.get_data(a-p.OPTIONAL_HEADER.ImageBase,z-a) for a,z in spans]
for i,b in enumerate(fragments):(O/f'fragment-{i}.bin').write_bytes(b)
cases=[(37,0x1234,0x3f800000,10,11,12,20,21,22,23,0xa5),(0xffffffff,0,0x80000000,0,0xffffffff,0x7fc00001,0,1,0x80000000,0x7f800000,0x5a),(0,0xdeadbeef,0,0x11223344,0x55667788,0x99aabbcc,1,2,3,4,0xcc),(0x87654321,0xabcdef01,0x7f7fffff,4,5,6,7,8,9,10,0xff)]
(O/'cases.bin').write_bytes(b''.join(struct.pack('<11I',*v) for v in cases))
def db(b):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in b[i:i+24]) for i in range(0,len(b),24))
blocks=[]
for i,c in enumerate(cases):
 selected,definition,duration,*tail=c;vector=tail[:3];quat=tail[3:7];initial=tail[7];b=[]
 for j,v in enumerate(vector):b.append(f' mov dword ptr [Vector+{j*4}],0{v:08X}h')
 for j,v in enumerate(quat):b.append(f' mov dword ptr [Quat+{j*4}],0{v:08X}h')
 setup='\n'.join(b)
 slices=[f'Slice{i}_{j}::\n{db(f)}' for j,f in enumerate(fragments)]
 blocks.append(f'''{setup}
 mov edi,OFFSET Capture
 mov eax,0{initial*0x01010101:08X}h
 mov ecx,31
 rep stosd
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 sub esp,60h
 mov dword ptr [esp+14h],1
 mov dword ptr [esp+18h],0{duration:08X}h
 mov dword ptr [esp+1Ch],000010000h
 mov dword ptr [esp+20h],000020001h
 mov dword ptr [esp+24h],0{selected:08X}h
 mov dword ptr [esp+28h],0FFFFFFFFh
 mov dword ptr [esp+2Ch],2
 mov dword ptr [esp+30h],0{definition:08X}h
 mov dword ptr [esp+34h],0
 mov dword ptr [esp+38h],0
 mov dword ptr [esp+48h],0
 mov dword ptr [esp+4Ch],0
 mov dword ptr [esp+50h],0
 mov eax,OFFSET Capture
{slices[0]}
 mov edi,OFFSET Vector
{slices[1]}
 mov ecx,OFFSET Vector
{slices[2]}
 mov ecx,OFFSET Quat
{slices[3]}
 ; Original pop edi omitted, because wrapper owns this synthetic stack.
{slices[4]}
 add esp,60h
 cmp esp,SavedESP
 jne Failure
 fnstcw word ptr [Capture+124]
 fnstsw word ptr [Capture+126]
 stmxcsr [Capture+128]
 cmp GuardRecord,0A5A5A5A5h
 jne Failure
 cmp EndRecord,0A5A5A5A5h
 jne Failure
 cmp GuardVector,0A5A5A5A5h
 jne Failure
 cmp EndVector,0A5A5A5A5h
 jne Failure
 cmp GuardQuat,0A5A5A5A5h
 jne Failure
 cmp EndQuat,0A5A5A5A5h
 jne Failure
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{i*132}
 mov ecx,33
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
GuardRecord DWORD 0A5A5A5A5h
Capture DWORD 33 DUP(0)
EndRecord DWORD 0A5A5A5A5h
GuardVector DWORD 0A5A5A5A5h
Vector DWORD 3 DUP(0)
EndVector DWORD 0A5A5A5A5h
GuardQuat DWORD 0A5A5A5A5h
Quat DWORD 4 DUP(0)
EndQuat DWORD 0A5A5A5A5h
Results DWORD 132 DUP(0)
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
{chr(10).join('PUBLIC Slice%d_%d'%(i,j) for i in range(4) for j in range(5))}
HostMain PROC
 cld
{chr(10).join(blocks)}
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 528
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,528
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 528
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 132
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
END HostMain
'''
(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib')
cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]]
logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),*[O/f'fragment-{i}.bin' for i in range(5)],O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-bevy/src/cell_source_case10_event_v1.rs',R/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs',R/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs',R/'crates/spore-bevy/examples/cell_case10_event_stores_oracle_v1.rs',R/'target/debug/examples/cell_case10_event_stores_oracle_v1.exe']
m={'host_executed':False,'source_sha256':h(S.read_bytes()),'spans':[[hex(a),hex(z-1)] for a,z in spans],'pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Sliced field stores; skips null-pointer selection and prologue/epilogue, admits vector/quaternion buffers and initial record. No allocator, callback, dispatch, event lifetime or mutation.'}
(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
