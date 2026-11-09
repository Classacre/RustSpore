"""Build-only unmodified BBA700 endpoint comparator host."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-sweep-endpoint-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S));f=p.get_data(0xbba700-p.OPTIONAL_HEADER.ImageBase,29);assert h(f)=='b05619e6f5ded95238efc063c573fcbaf63b743a1d815931a032cfb25b2f8ee9';(O/'fragment.bin').write_bytes(f)
b=lambda x:struct.unpack('<I',struct.pack('<f',x))[0];cases=[(b(-2),b(1)),(b(1),b(-2)),(b(2),b(2)),(0,0x80000000),(0x80000000,0),(b(1),b(2)),(0x7f7fffff,b(1)),(b(1),0x7f7fffff)];(O/'cases.bin').write_bytes(b''.join(struct.pack('<II',*c) for c in cases))
def db(x):return '\n'.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+24]) for i in range(0,len(x),24))
blocks=[]
for i,(a,bits) in enumerate(cases):blocks.append(f''' mov dword ptr [EndpointA+4],0{a:08X}h
 mov dword ptr [EndpointB+4],0{bits:08X}h
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push OFFSET EndpointB
 push OFFSET EndpointA
 call SourceFragment
 add esp,8
 mov [Capture],eax
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
 fnstcw word ptr [Capture+4]
 fnstsw word ptr [Capture+6]
 stmxcsr [Capture+8]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{i*12}
 mov ecx,3
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
EndpointA DWORD 2 DUP(0)
EndA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
EndpointB DWORD 2 DUP(0)
EndB DWORD 0A5A5A5A5h
Capture DWORD 3 DUP(0)
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
 push 12
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
paths=[Path(__file__),O/'fragment.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-core/src/cell_sweep_endpoint_compare_v1.rs',R/'crates/spore-core/src/cell_source_sweep_pairs_v1.rs',R/'crates/spore-core/examples/cell_sweep_endpoint_compare_oracle_v1.rs',R/'target/debug/examples/cell_sweep_endpoint_compare_oracle_v1.exe'];m={'host_executed':False,'source_sha256':h(S.read_bytes()),'range':'BBA700..BBA71C','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Synthetic disjoint finite-normal/zero endpoint pairs; no CRT sort, merge, persistent collection, actor identity or gameplay.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
