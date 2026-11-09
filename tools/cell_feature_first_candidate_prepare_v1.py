"""Build-only one-row E6ADE0 host; records and traps at first E6A450."""
from pathlib import Path
import pefile,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-feature-first-candidate-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S),fast_load=True);a=0xe6ade0;b=p.get_data(a-p.OPTIONAL_HEADER.ImageBase,1018);assert h(b)=='d4e5c05b22a47109d5e9559d92fd1314321ae81d1ff9ea88057bee25dca8b3d8';(O/'source.bin').write_bytes(b)
# Distance 3 separated, 2 tangent, 1 overlap, exact threshold and below.
x=[0x40400000,0x40000000,0x3f800000,0x3b800000,0x3b7fffff];(O/'cases.bin').write_bytes(struct.pack('<5I',*x))
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
Threshold DWORD 037800000h
One DWORD 03F800000h
Half DWORD 03F000000h
GuardA DWORD 0A5A5A5A5h
ActorA BYTE 264 DUP(0)
AfterA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
ActorB BYTE 264 DUP(0)
AfterB DWORD 0A5A5A5A5h
GuardCA DWORD 0A5A5A5A5h
CacheA BYTE 76 DUP(0)
AfterCA DWORD 0A5A5A5A5h
GuardCB DWORD 0A5A5A5A5h
CacheB BYTE 76 DUP(0)
AfterCB DWORD 0A5A5A5A5h
GuardDA DWORD 0A5A5A5A5h
DefA BYTE 184 DUP(0)
AfterDA DWORD 0A5A5A5A5h
GuardDB DWORD 0A5A5A5A5h
DefB BYTE 184 DUP(0)
AfterDB DWORD 0A5A5A5A5h
Output DWORD 12 DUP(0)
Capture DWORD 24 DUP(0)
Results DWORD 120 DUP(0)
CaseIndex DWORD 0
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
 mov dword ptr [ActorA+0FCh],11111111h
 mov dword ptr [ActorA+100h],12121212h
 mov dword ptr [ActorA+104h],13131313h
 mov dword ptr [ActorA+58h],03F800000h
 mov dword ptr [ActorB+0FCh],22222222h
 mov dword ptr [ActorB+100h],23232323h
 mov dword ptr [ActorB+104h],24242424h
 mov dword ptr [ActorB+58h],03F800000h
 mov dword ptr [CacheA],1
 mov dword ptr [CacheA+8],2
 mov dword ptr [CacheA+3Ch],03F800000h
 mov dword ptr [CacheB],1
 mov dword ptr [CacheB+8],3
 mov dword ptr [CacheB+3Ch],03F800000h
 mov dword ptr [DefA+0B4h],2
 mov dword ptr [DefB+0B4h],3
 mov dword ptr [DefB+0B0h],4
'''
for k,v in enumerate(x):
 s+=f''' mov edi,OFFSET Capture
 xor eax,eax
 mov ecx,24
 rep stosd
 mov CaseIndex,{k}
 mov dword ptr [ActorB+4Ch],0{v:08X}h
 fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push OFFSET DefB
 push OFFSET DefA
 push OFFSET ActorB
 push OFFSET ActorA
 push OFFSET Output
 call LoopSource
 add esp,20
 jmp AfterSource
CaptureExit:
'''.replace('CaptureExit:\n','')
 # Candidate stub abandons the source stack and jumps to a per-case common
 # continuation via CaseReturn. Separated case returns normally.
 s+=f'''AfterCase{k}:
 cmp esp,SavedESP
 jne Failure
'''
 for g in ['GuardA','AfterA','GuardB','AfterB','GuardCA','AfterCA','GuardCB','AfterCB','GuardDA','AfterDA','GuardDB','AfterDB']:s+=f' cmp {g},0A5A5A5A5h\n jne Failure\n'
 s+=f''' fnstcw word ptr [Capture+88]
 fnstsw word ptr [Capture+90]
 stmxcsr [Capture+92]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{k*96}
 mov ecx,24
 rep movsd
'''
# AfterSource executes after normal source return and before each AfterCase;
# replace per-case jmp with direct case-specific continuation.
for k in range(5):s=s.replace(' jmp AfterSource\nAfterCase'+str(k)+':',' jmp AfterCase'+str(k)+'\nAfterCase'+str(k)+':',1)
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 480
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,480
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 480
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 96
 push OFFSET Capture
 push ebx
 call _WriteFile@20
 push 7
 call _ExitProcess@4
HostMain ENDP
RelationStub PROC
 cmp dword ptr [esp+4],OFFSET ActorA
 jne StubFailure
 cmp dword ptr [esp+8],OFFSET ActorB
 jne StubFailure
 inc dword ptr [Capture]
 xor eax,eax
 ret
RelationStub ENDP
CacheStub PROC
 mov edx,SavedESP
 sub edx,512
 cmp eax,edx
 jb StubFailure
 mov edx,SavedESP
 sub edx,12
 cmp eax,edx
 ja StubFailure
 cmp dword ptr [Capture+4],0
 jne CacheSecond
 cmp dword ptr [eax],11111111h
 jne StubFailure
 cmp dword ptr [eax+4],12121212h
 jne StubFailure
 cmp dword ptr [eax+8],13131313h
 jne StubFailure
 inc dword ptr [Capture+4]
 mov eax,OFFSET CacheA
 ret
CacheSecond:
 cmp dword ptr [Capture+4],1
 jne StubFailure
 cmp dword ptr [eax],22222222h
 jne StubFailure
 cmp dword ptr [eax+4],23232323h
 jne StubFailure
 cmp dword ptr [eax+8],24242424h
 jne StubFailure
 inc dword ptr [Capture+4]
 mov eax,OFFSET CacheB
 ret
CacheStub ENDP
CandidateStub PROC
 cmp dword ptr [esp+4],OFFSET Output
 jne StubFailure
 cmp dword ptr [esp+14h],OFFSET ActorA
 jne StubFailure
 cmp dword ptr [esp+24h],OFFSET ActorB
 jne StubFailure
 inc dword ptr [Capture+8]
 mov eax,[esp+10h]
 mov [Capture+12],eax
 mov eax,[esp+18h]
 mov [Capture+40],eax
 mov eax,[esp+1Ch]
 mov [Capture+44],eax
 mov eax,[esp+20h]
 mov [Capture+48],eax
 mov eax,[esp+28h]
 mov [Capture+52],eax
 mov eax,[esp+2Ch]
 mov [Capture+56],eax
 mov eax,[esp+30h]
 mov [Capture+60],eax
 mov eax,[esp+34h]
 mov [Capture+64],eax
 mov eax,[esp+38h]
 mov [Capture+68],eax
 mov dword ptr [Capture+72],1
 mov dword ptr [Capture+76],1
 mov dword ptr [Capture+80],1
 cmp CaseIndex,4
 je OpaqueSmall
 mov eax,SavedESP
 sub eax,512
 mov ecx,SavedESP
 sub ecx,12
 mov edx,[esp+8]
 cmp edx,eax
 jb StubFailure
 cmp edx,ecx
 ja StubFailure
 mov eax,[edx]
 mov [Capture+16],eax
 mov eax,[edx+4]
 mov [Capture+20],eax
 mov eax,[edx+8]
 mov [Capture+24],eax
 mov eax,SavedESP
 sub eax,512
 mov edx,[esp+0Ch]
 cmp edx,eax
 jb StubFailure
 cmp edx,ecx
 ja StubFailure
 mov eax,[edx]
 mov [Capture+28],eax
 mov eax,[edx+4]
 mov [Capture+32],eax
 mov eax,[edx+8]
 mov [Capture+36],eax
 jmp CandidateDone
OpaqueSmall:
 mov dword ptr [Capture+84],1
CandidateDone:
 mov esp,SavedESP
 cmp CaseIndex,0
 je Case0
 cmp CaseIndex,1
 je Case1
 cmp CaseIndex,2
 je Case2
 cmp CaseIndex,3
 je Case3
 jmp AfterCase4
Case0: jmp AfterCase0
Case1: jmp AfterCase1
Case2: jmp AfterCase2
Case3: jmp AfterCase3
CandidateStub ENDP
StubFailure PROC
 push 7
 call _ExitProcess@4
StubFailure ENDP
LoopSource PROC
'''
# Relocate 3 calls and candidate plus 3 absolute constants, nothing else.
patches=[(0xe6adf1,'call','RelationStub',5),(0xe6ae17,'call','CacheStub',5),(0xe6ae3e,'call','CacheStub',5),(0xe6b06f+4,'abs','Threshold',4),(0xe6b0bb+4,'abs','One',4),(0xe6b123+4,'abs','Half',4),(0xe6b19a,'call','CandidateStub',5)]
def db(x):return ''.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+20])+'\n' for i in range(0,len(x),20))
pos=0
for addr,kind,target,n in patches:
 off=addr-a
 if kind=='call':assert b[off]==0xe8;s+=db(b[pos:off+1])+f' DD {target} - ($ + 4)\n'
 else:s+=db(b[pos:off])+f' DD OFFSET {target}\n'
 pos=off+n
s+=db(b[pos:])+'''LoopSource ENDP
END HostMain
''';(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'source.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S];m={'host_executed':False,'range':'E6ADE0..E6B1D9','patches':'4 source calls and 3 constant references','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'one synthetic row each, geometry relation/cache stubs, first-candidate trap, small locals opaque, no real actor/cache/callback'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
