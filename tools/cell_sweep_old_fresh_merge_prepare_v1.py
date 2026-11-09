"""Build-only BBA7C0 old/fresh host. Does not execute original code."""
from pathlib import Path
import hashlib,json,struct,subprocess
R=Path(__file__).resolve().parents[1];P=R/'target/cell-fresh-sort-activated-prep-v1';O=R/'target/cell-old-fresh-merge-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=S.parent/'msvcr90.dll';M=S.parent/'Microsoft.VC90.CRT.manifest';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
assert h(D.read_bytes())=='ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185'
assert h(M.read_bytes())=='9e0220511d4ebdb014cc17ecb8319d57e3b0fea09681a80d8084aa8647196582'
# Slot records contain explicit links for both axes; tombstones are omitted from
# rank validation. Padding is 0xcc and output tails are compared in full.
def e(flag,slot,value):return struct.pack('<HHf',flag,slot,value)
def slot(actor,links):return struct.pack('<II4H',actor,0,*links)
def row(axis,slots,old,fresh):
 assert len(slots)==4 and len(old)<=8 and len(fresh)<=8
 return struct.pack('<III',axis,len(old),len(fresh))+b''.join(slots)+(b''.join(old)+b'\xcc'*64)[:64]+(b''.join(fresh)+b'\xcc'*64)[:64]
A=lambda a,b,c,d:slot(0x10000,[a,b,c,d]);B=lambda a,b,c,d:slot(0x20001,[a,b,c,d]);C=lambda a,b,c,d:slot(0x30002,[a,b,c,d]);Z=slot(0x40003,[71,72,73,74]);T=lambda v:e(0xfffe,0,v)
cases=[
 row(0,[A(0,81,1,82),B(61,62,63,64),C(65,66,67,68),Z],[e(0,0,1),e(1,0,8)],[]),
 row(0,[A(0,81,3,82),B(1,61,2,62),C(0x8000,65,0x8001,66),Z],[e(0,0,1),e(0,1,3),e(1,1,9),e(1,0,10)],[e(1,2,4),e(0,2,2)]),
 row(1,[A(81,0,82,2),B(61,0x8000,62,0x8001),C(65,66,67,68),Z],[e(0,0,1),T(2),e(1,0,9)],[e(1,1,4),T(5),e(0,1,3)]),
 row(0,[A(0x8000,81,0x8001,82),B(0x8002,61,0x8003,62),C(65,66,67,68),Z],[],[e(1,1,8),e(0,0,1),e(1,0,5),e(0,1,2)]),
 row(1,[A(80,81,82,83),B(61,62,63,64),C(65,66,67,68),Z],[],[])]
corpus=b''.join(cases);(O/'cases.bin').write_bytes(corpus)
s=(P/'host-x86.asm').read_text(encoding='ascii');s=s.replace('Slots DWORD 8 DUP(0)','Slots DWORD 16 DUP(0)').replace('NewX DWORD 8 DUP(0)','NewX DWORD 16 DUP(0)').replace('OutX DWORD 8 DUP(0)','OutX DWORD 16 DUP(0)').replace('Capture DWORD 28 DUP(0)','Capture DWORD 35 DUP(0)').replace('Results DWORD 84 DUP(0)','Results DWORD 175 DUP(0)')
start=s.index(' mov edi,OFFSET Manager\n');end=s.index('LoaderFailure:',start)
body=[]
def add(x):body.append(x)
for case_no,case in enumerate(cases):
 axis,oldn,newn=struct.unpack_from('<III',case);add(' mov edi,OFFSET Manager\n xor eax,eax\n mov ecx,14\n rep stosd\n mov dword ptr [Manager+14h],OFFSET Slots\n')
 for name,offset,size in [('Slots',12,64),('OutX',76,64),('NewX',140,64)]:
  for j in range(0,size,4):add(f' mov dword ptr [{name}+{j}],0{struct.unpack_from("<I",case,offset+j)[0]:08X}h\n')
 add(f''' fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push {axis}
 push {newn}
 push OFFSET NewX
 push {oldn}
 push OFFSET OutX
 push OFFSET Manager
 call SortSource
 add esp,24
 cmp esp,SavedESP
 jne Failure
 mov [Capture+128],eax
''')
 for guard in ['GuardM','AfterManager','GuardS','AfterSlots','GuardNX','AfterNX','GuardOX','AfterOX']:add(f' cmp {guard},0A5A5A5A5h\n jne Failure\n')
 add(f''' cld
 mov esi,OFFSET Slots
 mov edi,OFFSET Capture
 mov ecx,16
 rep movsd
 mov esi,OFFSET OutX
 mov edi,OFFSET Capture+64
 mov ecx,16
 rep movsd
 fnstcw word ptr [Capture+132]
 fnstsw word ptr [Capture+134]
 stmxcsr [Capture+136]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{case_no*140}
 mov ecx,35
 rep movsd
''')
add(''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 700
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,700
 jne Failure
 push 0
 call _ExitProcess@4
''')
s=s[:start]+''.join(body)+s[end:];s=s.replace(' push 336\n push OFFSET Results',' push 700\n push OFFSET Results').replace(' push 112\n push OFFSET Capture',' push 140\n push OFFSET Capture')
(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),P/'host-x86.asm',O/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S,D,M,R/'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs',R/'crates/spore-core/src/cell_sweep_endpoint_compare_v1.rs',R/'crates/spore-core/src/cell_sweep_old_fresh_merge_v1.rs',R/'crates/spore-core/examples/cell_sweep_old_fresh_merge_oracle_v1.rs',R/'target/debug/examples/cell_sweep_old_fresh_merge_oracle_v1.exe'];manifest={'host_executed':False,'case_count':5,'ranges':['BBA7C0..BBA9BE','11E04F0..11E051A','BBA700..BBA71C'],'limits':'synthetic guarded <=4 slots and 8 endpoint capacity, finite unique, disjoint, no source allocation or lifecycle; installed qsort and source activation; loader and in-memory integrity admitted','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs};(O/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest['pins'],indent=2))
