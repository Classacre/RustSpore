"""Build-only guarded BBAC30 removal host, no source execution."""
from pathlib import Path
import pefile,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-sweep-remove-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
p=pefile.PE(str(S),fast_load=True);b=p.get_data(0xbbac30-p.OPTIONAL_HEADER.ImageBase,202);assert h(b)=='5a5ab6971ec9b2363a5d75937810093321839498a986bbb05087c62dd0fdd43c';(O/'source.bin').write_bytes(b)
# [minX,minY,maxX,maxY], counts and current free head. Every selected
# bank/index is valid and distinct on that bank; slot 0 is a live fixture.
cases=[([0,0,1,1],2,2,0xffffffff),([0x8000,0x8000,0x8001,0x8001],2,2,0xffffffff),([0,0x8002,0x8001,3],1,2,1),([0x8003,2,0x8000,0x8001],2,2,0xffffffff)]
corpus=b''.join(struct.pack('<4HIII',*links,live,high,head) for links,live,high,head in cases);(O/'cases.bin').write_bytes(corpus)
s='''.686p
.xmm
.model flat
option casemap:none
EXTERN _GetStdHandle@4:PROC
EXTERN _WriteFile@20:PROC
EXTERN _ExitProcess@4:PROC
.data
CW WORD 027Fh
MX DWORD 01F80h
GuardM DWORD 0A5A5A5A5h
Manager DWORD 14 DUP(0)
AfterM DWORD 0A5A5A5A5h
GuardS DWORD 0A5A5A5A5h
Slots DWORD 8 DUP(0)
AfterS DWORD 0A5A5A5A5h
'''
for name in ['OldX','OldY','NewX','NewY']:s+=f'Guard{name} DWORD 0A5A5A5A5h\n{name} DWORD 8 DUP(0)\nAfter{name} DWORD 0A5A5A5A5h\n'
s+='''Capture DWORD 45 DUP(0)
Results DWORD 180 DUP(0)
SavedESP DWORD 0
Written DWORD 0
FailureCode DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
'''
for k,(links,live,high,head) in enumerate(cases):
 s+=f''' mov edi,OFFSET Manager
 xor eax,eax
 mov ecx,14
 rep stosd
 mov dword ptr [Manager+10h],{live}
 mov dword ptr [Manager+14h],OFFSET Slots
 mov dword ptr [Manager+18h],{high}
 mov dword ptr [Manager+1Ch],0{head:08X}h
 mov dword ptr [Manager+24h],OFFSET OldX
 mov dword ptr [Manager+28h],OFFSET OldY
 mov dword ptr [Manager+30h],OFFSET NewX
 mov dword ptr [Manager+34h],OFFSET NewY
 mov dword ptr [Slots],012345678h
 mov dword ptr [Slots+4],087654321h
 mov dword ptr [Slots+16],0{(0xffffffff if head==1 else 0x55667788):08X}h
 mov dword ptr [Slots+20],0{(0xffffffff if head==1 else 0x11223344):08X}h
 mov dword ptr [Slots+24],099AABB CCh
 mov dword ptr [Slots+28],0DDEEFF00h
'''.replace('099AABB CCh','099AABBCCh')
 for j,v in enumerate(links):s+=f' mov word ptr [Slots+{8+2*j}],0{v:04X}h\n'
 for bank,name in enumerate(['OldX','OldY','NewX','NewY']):
  for i in range(4):s+=f' mov dword ptr [{name}+{i*8}],0{0x11000000+bank*0x10000+i:08X}h\n mov dword ptr [{name}+{i*8+4}],0{0x3f000000+bank*0x10000+i:08X}h\n'
 s+=''' fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push 0
 push OFFSET Manager
 call RemoveSource
 add esp,8
 cmp esp,SavedESP
 jne Failure
'''
 for g in ['GuardM','AfterM','GuardS','AfterS']+[x+y for y in ['OldX','OldY','NewX','NewY'] for x in ['Guard','After']]:s+=f' cmp {g},0A5A5A5A5h\n jne Failure\n'
 s+=''' mov eax,[Manager+10h]
 mov [Capture],eax
 mov eax,[Manager+18h]
 mov [Capture+4],eax
 mov eax,[Manager+1Ch]
 mov [Capture+8],eax
 cld
 mov esi,OFFSET Slots
 mov edi,OFFSET Capture+12
 mov ecx,8
 rep movsd
'''
 for j,name in enumerate(['OldX','OldY','NewX','NewY']):s+=f' mov esi,OFFSET {name}\n mov edi,OFFSET Capture+{44+32*j}\n mov ecx,8\n rep movsd\n'
 s+=f''' fnstcw word ptr [Capture+172]
 fnstsw word ptr [Capture+174]
 stmxcsr [Capture+176]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{k*180}
 mov ecx,45
 rep movsd
'''
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 720
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,720
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 720
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 180
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 4
 push OFFSET FailureCode
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
RemoveSource PROC
'''
for i in range(0,len(b),20):s+=' DB '+','.join('0%02Xh'%v for v in b[i:i+20])+'\n'
s+='RemoveSource ENDP\nEND HostMain\n';(O/'host-x86.asm').write_text(s,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'source.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S];m={'host_executed':False,'range':'BBAC30..BBACF9','patches':[],'pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'guarded two-slot synthetic collection, explicit valid unique endpoint links, no allocation or identity, source does not check fixture safety'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
