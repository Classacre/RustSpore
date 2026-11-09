"""Build only: early B7 E6A450 into original E59B40; unsupported calls trap."""
from pathlib import Path
import pefile,capstone,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-b7-producer-arbitration-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S),fast_load=True);d=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
a=0xe6a450;b=p.get_data(a-p.OPTIONAL_HEADER.ImageBase,0x983);arbaddr=0xe59b40;arb=p.get_data(arbaddr-p.OPTIONAL_HEADER.ImageBase,0xcc);assert h(b)=='7661135fa4612630df3533e6445e3faab28d7e74357e6c8f29b315ebfac917e0';assert h(arb)=='066e0ae76b87675e20d9f7f796d045688a20a011ebd77dcfdbc3ff3e291a73a9';(O/'producer.bin').write_bytes(b);(O/'arbitration.bin').write_bytes(arb)
# feature A/B, B4 A/B, player ID, initial selector, initial depth, candidate depth
cases=[(2,3,2,7,0x10001,0,0,0x3f000000),(2,3,2,7,0x99999,0xffffffff,0,0x3f000000),(2,3,2,7,0x99999,0,0x3f800000,0x3f000000),(9,3,2,7,0x10001,0,0,0x3f000000),(2,3,7,7,0x10001,19,0x3f800000,0x3f000000)];(O/'cases.bin').write_bytes(b''.join(struct.pack('<8I',*r) for r in cases))
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
GuardCell DWORD 0A5A5A5A5h
Cell BYTE 16672 DUP(0)
AfterCell DWORD 0A5A5A5A5h
CellGlobal DWORD OFFSET Cell
GuardA DWORD 0A5A5A5A5h
ActorA DWORD 10001h
AfterA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
ActorB DWORD 20002h
AfterB DWORD 0A5A5A5A5h
GuardPoint DWORD 0A5A5A5A5h
Point DWORD 03F800000h,040000000h,040400000h
AfterPoint DWORD 0A5A5A5A5h
GuardNormal DWORD 0A5A5A5A5h
Normal DWORD 040800000h,040A00000h,040C00000h
AfterNormal DWORD 0A5A5A5A5h
GuardOut DWORD 0A5A5A5A5h
Output DWORD 12 DUP(0)
AfterOut DWORD 0A5A5A5A5h
Capture DWORD 15 DUP(0)
Results DWORD 75 DUP(0)
CallCount DWORD 0
SavedESP DWORD 0
SavedEBX DWORD 0
SavedEBP DWORD 0
SavedESI DWORD 0
SavedEDI DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
'''
for k,(fa,fb,da,db,player,selector,olddepth,depth) in enumerate(cases):
 assert db==7 and (fa==9 or fb==9 or (fa!=9 and fb!=9))
 s+=f''' mov dword ptr [Cell+411Ch],0{player:08X}h
 mov edi,OFFSET Output
 mov eax,0CCCCCCCCh
 mov ecx,12
 rep stosd
 mov dword ptr [Output+8],0{selector:08X}h
 mov dword ptr [Output+14h],0{olddepth:08X}h
 mov CallCount,0
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 mov SavedEBX,ebx
 mov SavedEBP,ebp
 mov SavedESI,esi
 mov SavedEDI,edi
 push 1
 push {fb}
 push 0
 push 4
 push {db}
 push OFFSET ActorB
 push 0
 push {fa}
 push {da}
 push OFFSET ActorA
 push 0{depth:08X}h
 push OFFSET Normal
 push OFFSET Point
 push OFFSET Output
 call ProducerSource
 add esp,56
 cmp esp,SavedESP
 jne Failure
 cmp ebx,SavedEBX
 jne Failure
 cmp ebp,SavedEBP
 jne Failure
 cmp esi,SavedESI
 jne Failure
 cmp edi,SavedEDI
 jne Failure
'''
 for guard in ['GuardCell','AfterCell','GuardA','AfterA','GuardB','AfterB','GuardPoint','AfterPoint','GuardNormal','AfterNormal','GuardOut','AfterOut']:s+=f' cmp {guard},0A5A5A5A5h\n jne Failure\n'
 s+=f''' cld
 mov esi,OFFSET Output
 mov edi,OFFSET Capture
 mov ecx,12
 rep movsd
 mov eax,CallCount
 mov [Capture+48],eax
 fnstcw word ptr [Capture+52]
 fnstsw word ptr [Capture+54]
 stmxcsr [Capture+56]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{k*60}
 mov ecx,15
 rep movsd
'''
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 300
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,300
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 300
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 60
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
ArbitrateThunk PROC
 inc CallCount
 cmp CallCount,1
 jne UnexpectedCall
 jmp ArbitrationSource
ArbitrateThunk ENDP
UnexpectedCall PROC
 push 7
 call _ExitProcess@4
UnexpectedCall ENDP
ProducerSource PROC
'''
def db(x):return ''.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+20])+'\n' for i in range(0,len(x),20))
# Preserve the full function and all internal relative branches. Relocate all
# original direct calls, routing only two B7 sites to native arbitration thunk.
ins=list(d.disasm(b,a));assert sum(i.size for i in ins)==len(b);calls=[i for i in ins if i.mnemonic=='call'];assert len(calls)==42
patches=[(i.address-a,5,'call','ArbitrateThunk' if i.address in (0xe6a4b2,0xe6a4db) else 'UnexpectedCall') for i in calls]+[(0xe6a47b-a+2,4,'abs','CellGlobal')];patches.sort();pos=0
for off,n,kind,target in patches:
 if kind=='call':assert b[off]==0xe8;s+=db(b[pos:off+1])+f' DD {target} - ($ + 4)\n'
 else:s+=db(b[pos:off])+f' DD OFFSET {target}\n'
 pos=off+n
s+=db(b[pos:])+'''ProducerSource ENDP
ArbitrationSource PROC
''';off=0xe59bb8-arbaddr+2;s+=db(arb[:off])+' DD OFFSET CellGlobal\n'+db(arb[off+4:])+'''ArbitrationSource ENDP
END HostMain
''';(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'producer.bin',O/'arbitration.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S];m={'host_executed':False,'ranges':['E6A450..E6ADD2','E59B40..E59C0B'],'patches':'42 calls, B7 two to original arbitration via counter, remaining calls trap; two global refs to synthetic Cell','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'synthetic B7/feature9 only, finite depth, valid stable disjoint actor words and geometry, no source actor or callbacks; unadmitted control flow unsafe despite traps'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
