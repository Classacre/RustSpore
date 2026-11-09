"""Build-only E69B00 type-2 state branch host."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-e69b-state-prep-v3';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
p=pefile.PE(str(S));get=lambda a,n:p.get_data(a-p.OPTIONAL_HEADER.ImageBase,n)
f=get(0xe69b2d,68);table=get(0xe69c4c,52);assert h(f)=='be3f0d235b36619f5c007d98e20aba4ec99d779aba2f342841a4dd57894fe205';assert set(table)<={0,1}
(O/'fragment.bin').write_bytes(f);(O/'table.bin').write_bytes(table)
cases=list(range(54))+[0xffffffff];(O/'cases.bin').write_bytes(struct.pack('<55I',*cases))
def db(b):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in b[i:i+24]) for i in range(0,len(b),24))
assert f[0x26:0x29]==b'\x0f\xb6\x90' and f[0x2d:0x30]==b'\xff\x24\x95' and f[0x3c]==0xe8
frag=db(f[:0x29])+'\n DD OFFSET SourceTable\n'+db(f[0x2d:0x30])+'\n DD OFFSET Targets\n'+db(f[0x34:0x3d])+'\n DD OnWrite - ($ + 4)\n'+db(f[0x41:])
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
Guard DWORD 0A5A5A5A5h
Actor BYTE 190h DUP(0)
EndActor DWORD 0A5A5A5A5h
Selected DWORD 37
SourceTable LABEL BYTE
{db(table)}
Targets DWORD OFFSET SourceFragment+34h, OFFSET SourceFragment+44h
States LABEL DWORD
{chr(10).join(' DD '+','.join('0%08Xh'%x for x in cases[i:i+8]) for i in range(0,len(cases),8))}
Results DWORD 220 DUP(0)
Capture DWORD 4 DUP(0)
Counter DWORD 0
Bad DWORD 0
SavedESP DWORD 0
Written DWORD 0
Index DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
LoopCase:
 mov eax,Index
 mov ecx,[States+eax*4]
 mov dword ptr [Actor+18Ch],ecx
 mov Counter,0
 mov Bad,0
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 sub esp,60h
 mov dword ptr [esp+44h],2
 mov ebp,OFFSET Actor
 mov edi,OFFSET Selected
 call SourceFragment
 add esp,60h
 cmp esp,SavedESP
 jne Failure
 cmp Guard,0A5A5A5A5h
 jne Failure
 cmp EndActor,0A5A5A5A5h
 jne Failure
 cmp Bad,0
 jne Failure
 mov eax,dword ptr [Actor+18Ch]
 mov [Capture],eax
 mov eax,Counter
 mov [Capture+4],eax
 fnstcw word ptr [Capture+8]
 fnstsw word ptr [Capture+10]
 stmxcsr [Capture+12]
 mov eax,Index
 shl eax,4
 mov esi,OFFSET Capture
 lea edi,[Results+eax]
 mov ecx,4
 rep movsd
 inc Index
 cmp Index,55
 jb LoopCase
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 880
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,880
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 880
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
OnWrite PROC
 cmp dword ptr [esp+4],OFFSET Actor
 jne BadCall
 cmp dword ptr [esp+8],OFFSET Selected
 jne BadCall
 cmp dword ptr [Actor+18Ch],12
 jne BadCall
 inc Counter
 ret
BadCall:
 mov Bad,1
 ret
OnWrite ENDP
SourceFragment PROC
{frag}
 pop ebx
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
paths=[Path(__file__),O/'fragment.bin',O/'table.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-bevy/src/cell_source_case10_event_v1.rs',R/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs',R/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs',R/'crates/spore-bevy/examples/cell_e69b_state_oracle_v1.rs',R/'target/debug/examples/cell_e69b_state_oracle_v1.exe']
m={'host_executed':False,'source_sha256':h(S.read_bytes()),'source_range':'E69B2D..E69B70','patches':['E69B56 absolute table','E69B5D absolute jump table','E69B6A rel32 call'],'pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Synthetic actor; E57890 guarded stub, type=2, no first lookup, actor gate, second lookup, event or gameplay.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
