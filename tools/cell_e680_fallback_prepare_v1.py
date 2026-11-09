"""Build only 537EE0 and four original comparison helpers."""
from pathlib import Path
import hashlib,json,struct,subprocess,pefile
R=Path(__file__).resolve().parents[1];O=R/'target/cell-e680-fallback-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S));get=lambda a,n:p.get_data(a-p.OPTIONAL_HEADER.ImageBase,n)
spans=[(0x537ee0,0x537fb7,'Fallback'),(0x41dbd0,0x41dc28,'VectorCompare'),(0x41dc30,0x41dc4f,'MatrixCompare'),(0x422d90,0x422dfc,'RowsCompare'),(0x422e00,0x422e58,'RowCompare')];frags=[get(a,z-a) for a,z,_ in spans]
for i,f in enumerate(frags):(O/f'fragment-{i}.bin').write_bytes(f)
def obj(flags=0,rev=0,words=None):return struct.pack('<HH13I',flags,rev,*(words or [0]*13))
def withword(data,i,v):x=bytearray(data);struct.pack_into('<I',x,4+i*4,v);return bytes(x)
base=obj();m2=obj(2);cases=[]
cases.append((base,base));cases.append((base,withword(base,0,0x80000000)));cases.append((base,obj(2)));cases.append((base,withword(base,0,0x3f800000)));cases.append((base,withword(base,3,0x3f800000)));cases.append((m2,withword(m2,4,0x3f800000)));cases.append((base,withword(base,4,0x3f800000)));cases.append((obj(0,0xffff),withword(base,1,0x40000000)));cases.append((base,withword(base,2,0x3f800000)));cases.append((m2,withword(m2,12,0x40000000)))
corpus=b''.join(a+b for a,b in cases);(O/'cases.bin').write_bytes(corpus)
def db(b):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in b[i:i+24]) for i in range(0,len(b),24))
patches={0:[(0x537f07,'VectorCompare'),(0x537f44,'MatrixCompare')],2:[(0x41dc3b,'RowsCompare')],3:[(0x422d9c,'RowCompare'),(0x422db9,'RowCompare'),(0x422dd6,'RowCompare')]};procs=[]
for i,(a,z,name) in enumerate(spans):
 f=frags[i];last=0;parts=[]
 for addr,target in patches.get(i,[]):
  off=addr-a;assert f[off]==0xe8;parts.extend([db(f[last:off+1]),' DD '+target+' - ($ + 4)']);last=off+5
 parts.append(db(f[last:]));procs.append(name+' PROC\n'+'\n'.join(parts)+'\n'+name+' ENDP')
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
Capture DWORD 16 DUP(0)
EndA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
Source DWORD 14 DUP(0)
EndB DWORD 0A5A5A5A5h
Cases LABEL BYTE
{db(corpus)}
Results DWORD 160 DUP(0)
SavedESP DWORD 0
Index DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
LoopCase:
 mov eax,Index
 imul eax,eax,112
 lea esi,[Cases+eax]
 mov edi,OFFSET Capture
 mov ecx,14
 rep movsd
 mov edi,OFFSET Source
 mov ecx,14
 rep movsd
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 mov ecx,OFFSET Capture
 push OFFSET Source
 call Fallback
 cmp eax,OFFSET Capture
 jne Failure
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
 fnstcw word ptr [Capture+56]
 fnstsw word ptr [Capture+58]
 stmxcsr [Capture+60]
 mov eax,Index
 shl eax,6
 mov esi,OFFSET Capture
 lea edi,[Results+eax]
 mov ecx,16
 rep movsd
 inc Index
 cmp Index,10
 jb LoopCase
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 640
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,640
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 640
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
{chr(10).join(procs)}
END HostMain
''';(O/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),*[O/f'fragment-{i}.bin' for i in range(5)],O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,R/'crates/spore-bevy/src/cell_source_case10_event_v1.rs',R/'crates/spore-bevy/src/cell_source_pair_coordinator_v1.rs',R/'crates/spore-bevy/src/cell_source_player_pool_fixture_v1.rs',R/'crates/spore-bevy/examples/cell_e680_fallback_oracle_v1.rs',R/'target/debug/examples/cell_e680_fallback_oracle_v1.exe'];m={'host_executed':False,'source_sha256':h(S.read_bytes()),'spans':[[hex(a),hex(z-1),n] for a,z,n in spans],'patches':[[hex(addr),target] for v in patches.values() for addr,target in v],'pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Finite normal/zero, masked controls, guarded disjoint buffers; no E68070 routing, actor/cache lookup, callbacks or live gameplay.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
