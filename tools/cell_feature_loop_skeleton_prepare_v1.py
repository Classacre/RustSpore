"""Build-only E6ADE0 loop-skeleton host with explicit geometry cutouts."""
from pathlib import Path
import pefile,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-feature-loop-skeleton-prep-v2';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S),fast_load=True);a=0xe6ade0;b=p.get_data(a-p.OPTIONAL_HEADER.ImageBase,1018);assert h(b)=='d4e5c05b22a47109d5e9559d92fd1314321ae81d1ff9ea88057bee25dca8b3d8';(O/'source.bin').write_bytes(b)
cases=[(0,3,[0,0,0],[0,0,0]),(3,2,[1,1,1],[0,0,0]),(3,3,[0,1,2],[1,0,2]),(3,3,[0,0,0],[0,0,0]),(2,0,[0,2,0],[0,0,0])];(O/'cases.bin').write_bytes(b''.join(struct.pack('<8I',na,nb,*ca,*cb) for na,nb,ca,cb in cases))
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
GuardA DWORD 0A5A5A5A5h
ActorA BYTE 264 DUP(0)
AfterA DWORD 0A5A5A5A5h
GuardB DWORD 0A5A5A5A5h
ActorB BYTE 264 DUP(0)
AfterB DWORD 0A5A5A5A5h
GuardCA DWORD 0A5A5A5A5h
CacheA BYTE 220 DUP(0)
AfterCA DWORD 0A5A5A5A5h
GuardCB DWORD 0A5A5A5A5h
CacheB BYTE 220 DUP(0)
AfterCB DWORD 0A5A5A5A5h
Capture DWORD 23 DUP(0)
Results DWORD 115 DUP(0)
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
 mov dword ptr [ActorA+0FCh],11111111h
 mov dword ptr [ActorA+100h],12121212h
 mov dword ptr [ActorA+104h],13131313h
 mov dword ptr [ActorB+0FCh],22222222h
 mov dword ptr [ActorB+100h],23232323h
 mov dword ptr [ActorB+104h],24242424h
'''
for k,(na,nb,ca,cb) in enumerate(cases):
 s+=' mov edi,OFFSET Capture\n xor eax,eax\n mov ecx,23\n rep stosd\n'
 for name,count,classes in [('CacheA',na,ca),('CacheB',nb,cb)]:
  s+=f' mov dword ptr [{name}],{count}\n'
  for j,c in enumerate(classes):s+=f' mov dword ptr [{name}+{8+j*72}],{c}\n'
 s+=''' fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push 022222222h
 push OFFSET ActorB
 push OFFSET ActorA
 push 011111111h
 call LoopSource
 add esp,16
 cmp esp,SavedESP
 jne Failure
'''
 for g in ['GuardA','AfterA','GuardB','AfterB','GuardCA','AfterCA','GuardCB','AfterCB']:s+=f' cmp {g},0A5A5A5A5h\n jne Failure\n'
 s+=f''' fnstcw word ptr [Capture+84]
 fnstsw word ptr [Capture+86]
 stmxcsr [Capture+88]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{k*92}
 mov ecx,23
 rep movsd
'''
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 460
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,460
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 460
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 92
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
OuterCut PROC
 mov eax,[esp+28h]
 xor ebp,ebp
 cmp dword ptr [eax],0
 jle EmptyInner
 lea ebx,[eax+14h]
 jmp InnerCheck
EmptyInner:
 mov ebx,[esp+2Ch]
 mov edx,[esp+1Ch]
 jmp OuterTail
OuterCut ENDP
CandidateCut PROC
 push eax
 push edx
 mov edx,[Capture+8]
 cmp edx,9
 jae StubFailure
 mov eax,[esp+18h]
 mov [Capture+edx*8+12],eax
 mov [Capture+edx*8+16],ebp
 inc dword ptr [Capture+8]
 pop edx
 pop eax
 jmp InnerTail
CandidateCut ENDP
StubFailure PROC
 push 7
 call _ExitProcess@4
StubFailure ENDP
LoopSource PROC
'''
# External calls are stubs, two whole-instruction geometry regions become
# cutouts; E6A450 is also redirected to trap if a cutout fails unexpectedly.
# Internal branch bytes stay unchanged; labels expose three source continuations.
patches={0xe6adf1:('call','RelationStub',5),0xe6ae17:('call','CacheStub',5),0xe6ae3e:('call','CacheStub',5),0xe6ae6c:('cut','OuterCut',9),0xe6af72:('cut','CandidateCut',9),0xe6b19a:('call','StubFailure',5)};labels={0xe6af66:'InnerCheck',0xe6b1aa:'InnerTail',0xe6b1ba:'OuterTail'}
def db(x):return ''.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+20])+'\n' for i in range(0,len(x),20))
pos=0
for address in sorted(set(patches)|set(labels)):
 off=address-a;s+=db(b[pos:off]);pos=off
 if address in labels:s+=labels[address]+'::\n'
 if address in patches:
  kind,target,n=patches[address]
  if kind=='call':assert b[off]==0xe8;s+=' DB 0E8h\n'+f' DD {target} - ($ + 4)\n'
  else:s+=' DB 0E9h\n'+f' DD {target} - ($ + 4)\n'+' DB '+','.join(['090h']*(n-5))+'\n'
  pos=off+n
s+=db(b[pos:])+'''LoopSource ENDP
END HostMain
''';(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'source.bin',O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S];m={'host_executed':False,'range':'E6ADE0..E6B1D9','patches':'three relation/cache calls to stubs, two 9-byte geometry cutouts, E6A450 to trap','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'instrumented traversal only; cache/actor identities and content admitted; no feature projection, pair geometry, E6A450, callbacks or gameplay'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
