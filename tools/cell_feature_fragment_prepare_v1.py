"""Preparation only: builds but never runs an original-byte x86 host."""
import hashlib, json, struct, subprocess
from pathlib import Path
import pefile

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'target/cell-feature-fragment-prep-v2'
SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
EXPECTED_SOURCE='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
EXPECTED_FRAGMENT='addffaaee26f1051ffeaf1fca15f1fb463449e72c1a14e7c2e2fa50d6dbaefe2'
sha=lambda b:hashlib.sha256(b).hexdigest()
def b(f):return struct.unpack('<I',struct.pack('<f',f))[0]
def case(flags, center, radius, pos, scale, matrix):
    actor=bytearray(0x80);row=bytearray(72)
    actor[0x48]=flags
    struct.pack_into('<4I',actor,0x4c,*pos,scale)
    struct.pack_into('<9I',actor,0x5c,*matrix)
    struct.pack_into('<3I',row,8,*center);struct.pack_into('<I',row,0x38,radius)
    return bytes(actor),bytes(row)
Z=b(0.);O=b(1.)
I=[O,Z,Z,Z,O,Z,Z,Z,O]
Y=[Z,O,Z,Z,O,Z,Z,O,Z]
cases=[
 case(0,list(map(b,[3.,4.,5.])),b(2.),list(map(b,[10.,20.,30.])),b(2.),I),
 case(2,list(map(b,[3.,4.,5.])),b(2.),list(map(b,[10.,20.,30.])),b(2.),I),
 case(2,list(map(b,[1.,2.,3.])),b(2.),[Z,Z,Z],O,list(map(b,[1.,2.,3.,4.,5.,6.,7.,8.,9.]))),
 case(2,list(map(b,[3.,1e20,-1e20])),O,[Z,Z,Z],O,Y),
 case(0,[1,0x80000001,0x80000000],O,[0,0,0],b(.5),I),
 case(0,[0x80000000]*3,0x80000000,[0x80000000]*3,O,I),
]
assert not OUT.exists(),'choose fresh output directory; never overwrite'
OUT.mkdir(parents=True)
assert sha(SOURCE.read_bytes())==EXPECTED_SOURCE
pe=pefile.PE(str(SOURCE));fragment=pe.get_data(0xe6ae6c-pe.OPTIONAL_HEADER.ImageBase,200)
assert sha(fragment)==EXPECTED_FRAGMENT and fragment[18:20]==bytes.fromhex('7472')
(OUT/'fragment.bin').write_bytes(fragment)
(OUT/'cases.bin').write_bytes(b''.join(a+r for a,r in cases))
def db(data):return ','.join('0%02Xh'%x for x in data)
def bytes_lines(data):return '\n'.join(' DB '+db(data[i:i+24]) for i in range(0,len(data),24))
rows=[]
for i,(a,r) in enumerate(cases):
 rows.append(f'A{i}Guard DWORD 0A5A5A5A5h\nA{i} LABEL BYTE\n{bytes_lines(a)}\nA{i}End DWORD 0A5A5A5A5h\nR{i}Guard DWORD 0A5A5A5A5h\nR{i} LABEL BYTE\n{bytes_lines(r)}\nR{i}End DWORD 0A5A5A5A5h')
blocks=[]
for i in range(len(cases)):
 o=i*20
 blocks.append(f''' ldmxcsr dword ptr [MX]
 mov edi,OFFSET A{i}
 mov edx,OFFSET R{i}+16
 sub esp,40h
 mov dword ptr [esp],0A5A5A5A5h
 mov dword ptr [esp+3Ch],0A5A5A5A5h
 call SourceFragment
 movss dword ptr [Results+{o}],xmm6
 mov eax,[esp+30h]
 mov [Results+{o+4}],eax
 movss dword ptr [Results+{o+8}],xmm1
 movss dword ptr [Results+{o+12}],xmm0
 stmxcsr dword ptr [Results+{o+16}]
 cmp dword ptr [esp],0A5A5A5A5h
 jne Failure
 cmp dword ptr [esp+3Ch],0A5A5A5A5h
 jne Failure
 add esp,40h
 cmp A{i}Guard,0A5A5A5A5h
 jne Failure
 cmp A{i}End,0A5A5A5A5h
 jne Failure
 cmp R{i}Guard,0A5A5A5A5h
 jne Failure
 cmp R{i}End,0A5A5A5A5h
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
Written DWORD 0
Results DWORD {len(cases)*5} DUP(0CCCCCCCCh)
{chr(10).join(rows)}
.code
PUBLIC HostMain
HostMain PROC
 cld
{chr(10).join(blocks)}
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push {len(cases)*20}
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,{len(cases)*20}
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push 7
 call _ExitProcess@4
HostMain ENDP
SourceFragment PROC
{bytes_lines(fragment)}
 ret
SourceFragment ENDP
END HostMain
'''
(OUT/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86')
lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib')
commands=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{OUT}/host-x86.obj',str(OUT/'host-x86.asm')],
[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{OUT}/host-x86.exe',f'/MAP:{OUT}/host-x86.map',str(OUT/'host-x86.obj'),str(lib)]]
logs=[]
for cmd in commands:
 p=subprocess.run(cmd,capture_output=True,text=True);logs.append({'args':cmd,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
 if p.returncode:
  (OUT/'build-failure.json').write_text(json.dumps(logs,indent=2),encoding='utf8');raise SystemExit('host build failed')
exe=(OUT/'host-x86.exe').read_bytes();assert exe.count(fragment)==1
paths=[Path(__file__),OUT/'fragment.bin',OUT/'cases.bin',OUT/'host-x86.asm',OUT/'host-x86.obj',OUT/'host-x86.exe',OUT/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,ROOT/'crates/spore-bevy/src/cell_source_feature_geometry_v1.rs',ROOT/'crates/spore-core/src/native_sse.rs']
manifest={'host_executed':False,'source_sha256':EXPECTED_SOURCE,'fragment_range':'E6AE6C..E6AF33','fragment_sha256':EXPECTED_FRAGMENT,'pins':{str(p):sha(p.read_bytes()) for p in paths},'commands':logs,'cases':len(cases),'outputs':'x,y,z,radius,mxcsr, five little-endian u32 per case','limits':'Synthetic fragment; no native identity, gameplay, exceptions parity or immutable-file guarantee. Independent disassembly, ABI, memory, guard and control-flow review required before execution.'}
(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf8')
print(json.dumps(manifest['pins'],indent=2))
