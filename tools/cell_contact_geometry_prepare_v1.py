"""Build-only synthetic x86 contact continuation host. Never execute the fragment."""
import hashlib,json,struct,subprocess
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'target/cell-contact-geometry-prep-v2';SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
sha=lambda b:hashlib.sha256(b).hexdigest();b=lambda f:struct.unpack('<I',struct.pack('<f',f))[0]
S='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';F='c16003e09ec659ab4f2d729301fa51a4dbd5ce9cfa90756776149069e8f5ef74'
assert not OUT.exists();OUT.mkdir(parents=True);assert sha(SOURCE.read_bytes())==S
pe=pefile.PE(str(SOURCE));f=pe.get_data(0xe6b099-pe.OPTIONAL_HEADER.ImageBase,184);assert sha(f)==F
assert f[0x1c:0x22]==bytes.fromhex('0f87ef000000') and f[0x26:0x2a]==bytes.fromhex('d0614801') and f[0x8e:0x92]==bytes.fromhex('2c1b4701')
(OUT/'fragment.bin').write_bytes(f)
# Each tuple: Axyz,ra,Bxyz,rb,d2,dx,dy,dz,radius sum. All exact finite values.
z=b(0.);o=b(1.)
def row(A,ra,B,rb,d2,delta,rs):return struct.pack('<13I',*A,ra,*B,rb,d2,*delta,rs)
cases=[row([z,z,z],b(.5),[o,z,z],b(.5),o,[o,z,z],o),row([z,z,z],b(.25),[o,z,z],b(.25),o,[o,z,z],b(.5)),row([z,z,z],o,[o,z,z],b(.5),o,[o,z,z],b(1.5)),row([z,z,z],b(3.),[b(3.),b(4.),z],b(3.),b(25.),[b(3.),b(4.),z],b(6.)),row([z,z,z],o,[o,o,z],o,b(2.),[o,o,z],b(2.)),row([z,z,z],o,[b(.00390625),z,z],o,0x37800000,[b(.00390625),z,z],b(2.))]
(OUT/'cases.bin').write_bytes(b''.join(cases))
def db(d):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in d[i:i+24]) for i in range(0,len(d),24))
rows=[];blocks=[]
for i,c in enumerate(cases):
 rows.append(f'Guard{i} DWORD 0A5A5A5A5h\nCase{i} LABEL BYTE\n{db(c)}\nEnd{i} DWORD 0A5A5A5A5h')
 off=i*44
 blocks.append(f''' fninit
 fldcw [CW]
 ldmxcsr [MX]
 sub esp,80h
 mov dword ptr [esp],0A5A5A5A5h
 mov dword ptr [esp+7Ch],0A5A5A5A5h
 mov dword ptr [esp+38h],0CCCCCCCCh
 mov dword ptr [esp+3Ch],0CCCCCCCCh
 mov dword ptr [esp+40h],0CCCCCCCCh
 mov dword ptr [esp+44h],0CCCCCCCCh
 mov dword ptr [esp+48h],0CCCCCCCCh
 mov dword ptr [esp+4Ch],0CCCCCCCCh
 mov eax,dword ptr [Case{i}+32]
 mov [esp+1Ch],eax
 mov eax,dword ptr [Case{i}]
 mov [esp+2Ch],eax
 mov eax,dword ptr [Case{i}+4]
 mov [esp+30h],eax
 mov eax,dword ptr [Case{i}+8]
 mov [esp+34h],eax
 mov eax,dword ptr [Case{i}+28]
 mov [esp+58h],eax
 mov eax,dword ptr [Case{i}+12]
 mov [esp+5Ch],eax
 movss xmm5,dword ptr [Case{i}+16]
 movss xmm3,dword ptr [Case{i}+20]
 movss xmm7,dword ptr [Case{i}+24]
 movss xmm1,dword ptr [Case{i}+36]
 movss xmm6,dword ptr [Case{i}+40]
 movss xmm4,dword ptr [Case{i}+44]
 movss xmm0,dword ptr [Case{i}+48]
 call SourceFragment
 mov [Results+{off}],eax
 mov eax,[esp+10h]
 mov [Results+{off+4}],eax
 mov eax,[esp+14h]
 mov [Results+{off+8}],eax
 mov eax,[esp+38h]
 mov [Results+{off+12}],eax
 mov eax,[esp+3Ch]
 mov [Results+{off+16}],eax
 mov eax,[esp+40h]
 mov [Results+{off+20}],eax
 mov eax,[esp+44h]
 mov [Results+{off+24}],eax
 mov eax,[esp+48h]
 mov [Results+{off+28}],eax
 mov eax,[esp+4Ch]
 mov [Results+{off+32}],eax
 fnstcw word ptr [Results+{off+36}]
 fnstsw word ptr [Results+{off+38}]
 stmxcsr [Results+{off+40}]
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
CW WORD 027Fh
MX DWORD 01F80h
CGuard DWORD 0A5A5A5A5h
One DWORD 03F800000h
Half DWORD 03F000000h
CEnd DWORD 0A5A5A5A5h
Written DWORD 0
Results DWORD {len(cases)*11} DUP(0CCCCCCCCh)
{chr(10).join(rows)}
.code
PUBLIC HostMain
HostMain PROC
 cld
{chr(10).join(blocks)}
 cmp CGuard,0A5A5A5A5h
 jne Failure
 cmp One,03F800000h
 jne Failure
 cmp Half,03F000000h
 jne Failure
 cmp CEnd,0A5A5A5A5h
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
{db(f[:0x1e])}
 DD Separated - ($ + 4)
{db(f[0x22:0x26])}
 DD OFFSET One
{db(f[0x2a:0x8e])}
 DD OFFSET Half
{db(f[0x92:])}
 xor eax,eax
 ret
Separated:
 mov eax,1
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
paths=[Path(__file__),OUT/'fragment.bin',OUT/'cases.bin',OUT/'host-x86.asm',OUT/'host-x86.exe',OUT/'host-x86.obj',OUT/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,ROOT/'crates/spore-bevy/src/cell_source_feature_geometry_v1.rs',ROOT/'crates/spore-core/src/native_x87.rs',ROOT/'crates/spore-core/src/native_sse.rs']
m={'host_executed':False,'source_sha256':S,'fragment_sha256':F,'range':'E6B099..E6B150','patches':'rel32 branch operand 0x1e; absolute operands 0x26 and 0x8e','pins':{str(p):sha(p.read_bytes()) for p in paths},'commands':logs,'limits':'Synthetic finite inputs; host/patch/ABI/runner require independent review; no live contact or FP status parity.'}
(OUT/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
