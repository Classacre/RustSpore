"""Build-only guarded original BBB880 pending update host; no execution."""
from pathlib import Path
import pefile,hashlib,subprocess,json,struct
R=Path(__file__).resolve().parents[1];O=R/'target/cell-pending-endpoint-update-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True)
assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
p=pefile.PE(str(S),fast_load=True);b=p.get_data(0xbbb880-p.OPTIONAL_HEADER.ImageBase,381);assert h(b)=='e053b4bd03b795e4674d7e61967b93050c77e72bc1ec6ba8b932f3bcbe56993d';(O/'source.bin').write_bytes(b)
# links [minX,minY,maxX,maxY] low 15 bits index own axis; input floats
# raw finite normal/zero to preserve exact x87 load/store without exceptions.
cases=[([0,0,1,1],[0x3f800000,0x40000000,0x40400000,0x40800000]),([3,2,1,0],[0x80000000,0,0xc0200000,0x40a00000]),([2,3,0,1],[0x00800000,0x80800000,0x7f7fffff,0xff7fffff]),([1,1,3,3],[0x41200000,0xc1200000,0x3f000000,0xbf000000])]
corpus=b''.join(struct.pack('<4H4I',*links,*vals) for links,vals in cases);(O/'cases.bin').write_bytes(corpus)
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
Slot DWORD 4 DUP(0)
AfterS DWORD 0A5A5A5A5h
GuardX DWORD 0A5A5A5A5h
NewX DWORD 8 DUP(0)
AfterX DWORD 0A5A5A5A5h
GuardY DWORD 0A5A5A5A5h
NewY DWORD 8 DUP(0)
AfterY DWORD 0A5A5A5A5h
GuardV DWORD 0A5A5A5A5h
MinV DWORD 2 DUP(0)
MaxV DWORD 2 DUP(0)
AfterV DWORD 0A5A5A5A5h
Capture DWORD 22 DUP(0)
Results DWORD 88 DUP(0)
SavedESP DWORD 0
Written DWORD 0
FailureCode DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
'''
for k,(links,vals) in enumerate(cases):
 s+=' mov edi,OFFSET Manager\n xor eax,eax\n mov ecx,14\n rep stosd\n mov dword ptr [Manager+14h],OFFSET Slot\n mov dword ptr [Manager+30h],OFFSET NewX\n mov dword ptr [Manager+34h],OFFSET NewY\n mov dword ptr [Slot],012345678h\n mov dword ptr [Slot+4],087654321h\n'
 for j,v in enumerate(links):s+=f' mov word ptr [Slot+{8+2*j}],0{0x8000|v:04X}h\n'
 for axis,name in enumerate(['NewX','NewY']):
  for i in range(4):s+=f' mov dword ptr [{name}+{i*8}],0{(axis*16+i):08X}h\n mov dword ptr [{name}+{i*8+4}],0{(0x3e800000+axis*0x10000+i):08X}h\n'
 for j,v in enumerate(vals):s+=f' mov dword ptr [{"MinV" if j<2 else "MaxV"}+{(j%2)*4}],0{v:08X}h\n'
 s+=''' fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push OFFSET MaxV
 push OFFSET MinV
 push 0
 push OFFSET Manager
 call UpdateSource
 add esp,16
 cmp esp,SavedESP
 jne Failure
'''
 for g in ['GuardM','AfterM','GuardS','AfterS','GuardX','AfterX','GuardY','AfterY','GuardV','AfterV']:s+=f' cmp {g},0A5A5A5A5h\n jne Failure\n'
 s+=f''' cld
 mov esi,OFFSET Slot
 mov edi,OFFSET Capture
 mov ecx,4
 rep movsd
 mov esi,OFFSET NewX
 mov edi,OFFSET Capture+16
 mov ecx,8
 rep movsd
 mov esi,OFFSET NewY
 mov edi,OFFSET Capture+48
 mov ecx,8
 rep movsd
 fnstcw word ptr [Capture+80]
 fnstsw word ptr [Capture+82]
 stmxcsr [Capture+84]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{k*88}
 mov ecx,22
 rep movsd
'''
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 352
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,352
 jne Failure
 push 0
 call _ExitProcess@4
Trap:
 mov FailureCode,0BAD00001h
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 352
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 88
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
UnexpectedHelper PROC
 push 7
 call _ExitProcess@4
UnexpectedHelper ENDP
UpdateSource PROC
'''
# Patch only four helper CALL rel32 operands, to a failing trap. Remaining source
# bytes and complete instruction boundaries must be verified by runner.
offs=[0xbbb959-0xbbb880,0xbbb97e-0xbbb880,0xbbb9a7-0xbbb880,0xbbb9d0-0xbbb880];pos=0
for off in offs:
 assert b[off]==0xe8
 def db(x):return ''.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+20])+'\n' for i in range(0,len(x),20))
 s+=db(b[pos:off+1])+' DD UnexpectedHelper - ($ + 4)\n';pos=off+5
s+=db(b[pos:])+'''UpdateSource ENDP
END HostMain
''';(O/'host-x86.asm').write_text(s,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'source.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S];manifest={'host_executed':False,'range':'BBB880..BBB9FC','patches':'four sorted-branch helper calls redirected to failure trap','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'pending high-bit branch, synthetic disjoint slot/arrays, finite normal/zero x87 default; no actor identity or sorted crossing helpers'};(O/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest['pins'],indent=2))
