"""Build-only pinned 4098A0/41CAE0 local constructor host."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-e69b-local-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S));get=lambda a,n:p.get_data(a-p.OPTIONAL_HEADER.ImageBase,n)
f=get(0x4098a0,127);g=get(0x41cae0,0xd8);assert h(f)=='b37177c54324bed1456ba6ff14f4ab9a43153859066a325ec5032d67e850b54f';(O/'ctor.bin').write_bytes(f);(O/'copy.bin').write_bytes(g)
cases=[tuple(range(13)),(0x80000000,0x7fc00001,3,4,5,6,7,8,9,10,11,12,13),tuple([0xffffffff]*13),tuple([0x3f800000,0,0,0x7f800000,1,2,3,4,5,6,7,8,9])];(O/'cases.bin').write_bytes(b''.join(struct.pack('<13I',*c) for c in cases))
def db(b):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in b[i:i+24]) for i in range(0,len(b),24))
patches=[(0x2a,'OFFSET Vector'),(0x39,'OFFSET Vector+4'),(0x49,'OFFSET Vector+8'),(0x59,'OFFSET Scalar'),(0x6c,'OFFSET Matrix'),(0x74,'CopySource - ($ + 4)')];fr=[];last=0
for off,target in patches:fr.extend([db(f[last:off]),' DD '+target]);last=off+4
fr.append(db(f[last:]));frag='\n'.join(fr)
blocks=[]
for i,c in enumerate(cases):
 setup=[]
 for j,v in enumerate(c[:3]):setup.append(f' mov dword ptr [Vector+{j*4}],0{v:08X}h')
 setup.append(f' mov dword ptr [Scalar],0{c[3]:08X}h')
 for j,v in enumerate(c[4:]):setup.append(f' mov dword ptr [Matrix+{j*4}],0{v:08X}h')
 blocks.append(f'''{chr(10).join(setup)}
 mov edi,OFFSET Capture
 mov eax,0A5A5A5A5h
 mov ecx,14
 rep stosd
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 mov ecx,OFFSET Capture
 call CtorSource
 cmp eax,OFFSET Capture
 jne Failure
 cmp esp,SavedESP
 jne Failure
 cmp GuardOut,0A5A5A5A5h
 jne Failure
 cmp EndOut,0A5A5A5A5h
 jne Failure
 cmp GuardData,0A5A5A5A5h
 jne Failure
 cmp EndData,0A5A5A5A5h
 jne Failure
 fnstcw word ptr [Capture+56]
 fnstsw word ptr [Capture+58]
 stmxcsr [Capture+60]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{i*64}
 mov ecx,16
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
GuardOut DWORD 0A5A5A5A5h
Capture DWORD 16 DUP(0)
EndOut DWORD 0A5A5A5A5h
GuardData DWORD 0A5A5A5A5h
Vector DWORD 3 DUP(0)
Scalar DWORD 0
Matrix DWORD 9 DUP(0)
EndData DWORD 0A5A5A5A5h
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
 push 64
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
CtorSource PROC
{frag}
CtorSource ENDP
CopySource PROC
{db(g)}
CopySource ENDP
END HostMain
''';(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'ctor.bin',O/'copy.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-bevy/src/cell_source_case10_event_v1.rs',R/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs',R/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs',R/'crates/spore-bevy/examples/cell_e69b_local_oracle_v1.rs',R/'target/debug/examples/cell_e69b_local_oracle_v1.exe'];m={'host_executed':False,'source_sha256':h(S.read_bytes()),'ranges':['4098A0..40991E','41CAE0..41CBB7'],'patch_offsets':[hex(x) for x,_ in patches],'pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Admitted stable disjoint global words; no E68070, actors, callback or dispatch.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
