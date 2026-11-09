"""Build a bounded x86 second-feature/pair-prefix host; never execute it."""
import hashlib,json,struct,subprocess
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'target/cell-feature-pair-prep-v2'
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
sha=lambda b:hashlib.sha256(b).hexdigest()
b=lambda f:struct.unpack('<I',struct.pack('<f',f))[0]
S='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
F='e1c99e797b887d15fe3b2cc6c6d9f93684c85554a062d31bb83318d84a250cde'
assert not OUT.exists();OUT.mkdir(parents=True);assert sha(SOURCE.read_bytes())==S
pe=pefile.PE(str(SOURCE));frag=pe.get_data(0xe6af72-pe.OPTIONAL_HEADER.ImageBase,282)
assert sha(frag)==F and frag[0x101:0x105]==bytes.fromhex('9c5c4801')
(OUT/'fragment.bin').write_bytes(frag)
Z=0;O=b(1.);I=[O,Z,Z,Z,O,Z,Z,Z,O]
def case(first,position,center=(0,0,0),radius=O,scale=O,flags=0,matrix=I):
 actor=bytearray(128);row=bytearray(72);actor[0x48]=flags
 struct.pack_into('<4I',actor,0x4c,*position,scale);struct.pack_into('<9I',actor,0x5c,*matrix)
 struct.pack_into('<3I',row,8,*center);struct.pack_into('<I',row,0x38,radius)
 return struct.pack('<4I',*first)+bytes(actor)+bytes(row)
first=[Z,Z,Z,O]
cases=[case(first,[Z,Z,Z]),case(first,[0x3b7fffff,Z,Z]),case(first,[0x3b800000,Z,Z]),case(first,[0x3b800001,Z,Z]),
 case([b(1.),b(2.),b(3.),b(2.)],[b(10.),b(20.),b(30.)],[b(1.),b(2.),b(3.)],b(2.),b(2.),2,list(map(b,[1.,2.,3.,4.,5.,6.,7.,8.,9.]))),
 case(first,[Z,Z,Z],[1,0x80000001,0x80000000],O,b(.5),0,I)]
(OUT/'cases.bin').write_bytes(b''.join(cases))
def db(d):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in d[i:i+24]) for i in range(0,len(d),24))
rows=[];blocks=[]
for i,c in enumerate(cases):
 rows.append(f'Guard{i} DWORD 0A5A5A5A5h\nCase{i} LABEL BYTE\n{db(c)}\nEnd{i} DWORD 0A5A5A5A5h')
 o=i*44
 blocks.append(f''' ldmxcsr [MX]
 sub esp,80h
 mov dword ptr [esp],0A5A5A5A5h
 mov dword ptr [esp+7Ch],0A5A5A5A5h
 mov eax,dword ptr [Case{i}+4]
 mov dword ptr [esp+30h],eax
 mov eax,dword ptr [Case{i}+8]
 mov dword ptr [esp+34h],eax
 mov eax,dword ptr [Case{i}+12]
 mov dword ptr [esp+5Ch],eax
 movss xmm6,dword ptr [Case{i}]
 mov esi,OFFSET Case{i}+16
 mov ebx,OFFSET Case{i}+160
 call SourceFragment
 pushfd
 pop eax
 mov [Results+{o+36}],eax
 movss [Results+{o}],xmm5
 movss [Results+{o+4}],xmm3
 movss [Results+{o+8}],xmm7
 mov eax,[esp+58h]
 mov [Results+{o+12}],eax
 movss [Results+{o+16}],xmm1
 movss [Results+{o+20}],xmm6
 movss [Results+{o+24}],xmm4
 mov eax,[esp+1Ch]
 mov [Results+{o+28}],eax
 movss [Results+{o+32}],xmm0
 stmxcsr [Results+{o+40}]
 cmp dword ptr [esp],0A5A5A5A5h
 jne Failure
 cmp dword ptr [esp+7Ch],0A5A5A5A5h
 jne Failure
 add esp,80h
 cmp Guard{i},0A5A5A5A5h
 jne Failure
 cmp End{i},0A5A5A5A5h
 jne Failure''')
asm=f'''.686p
.xmm
.model flat
option casemap:none
EXTERN _GetStdHandle@4:PROC
EXTERN _WriteFile@20:PROC
EXTERN _ExitProcess@4:PROC
.data
MX DWORD 01F80h
ThresholdGuard DWORD 0A5A5A5A5h
Threshold DWORD 037800000h
ThresholdEnd DWORD 0A5A5A5A5h
Written DWORD 0
Results DWORD {len(cases)*11} DUP(0CCCCCCCCh)
{chr(10).join(rows)}
.code
PUBLIC HostMain
HostMain PROC
 cld
{chr(10).join(blocks)}
 cmp ThresholdGuard,0A5A5A5A5h
 jne Failure
 cmp Threshold,037800000h
 jne Failure
 cmp ThresholdEnd,0A5A5A5A5h
 jne Failure
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push {len(cases)*44}
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,{len(cases)*44}
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push 7
 call _ExitProcess@4
HostMain ENDP
SourceFragment PROC
{db(frag[:0x101])}
 DD OFFSET Threshold
{db(frag[0x105:])}
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
 if p.returncode:(OUT/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),OUT/'fragment.bin',OUT/'cases.bin',OUT/'host-x86.asm',OUT/'host-x86.exe',OUT/'host-x86.obj',OUT/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,ROOT/'crates/spore-bevy/src/cell_source_feature_geometry_v1.rs']
m={'host_executed':False,'source_sha256':S,'fragment_sha256':F,'range':'E6AF72..E6B08B','patch':'operand bytes at fragment offset 0x101..0x104 point to guarded host threshold 37800000','pins':{str(p):sha(p.read_bytes()) for p in paths},'commands':logs,'limits':'Synthetic fixed cases; no native actor/contact, exception parity or file-stability proof. Independent host, patch, ABI and runner review required.'}
(OUT/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
