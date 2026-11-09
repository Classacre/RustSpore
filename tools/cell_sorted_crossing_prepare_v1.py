"""Build-only guarded pair-disabled sorted endpoint crossing host."""
from pathlib import Path
import pefile,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-sorted-crossing-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S),fast_load=True)
spec=[('MinLeft',0xbbb020,264,[0xbbb097]),('MaxRight',0xbbb130,258,[0xbbb1a5]),('MinRight',0xbbb240,209,[0xbbb2b3,0xbbb2ca]),('MaxLeft',0xbbb320,207,[0xbbb391,0xbbb3a8])]
frags=[]
for name,a,n,calls in spec:
 b=p.get_data(a-p.OPTIONAL_HEADER.ImageBase,n);assert len(b)==n and b[-1:]==b'\xc3';(O/(name+'.bin')).write_bytes(b);frags.append(b)
# axis, selected slot0 side/direction implied by case number, entries (flag,slot,f32).
def e(f,slot,x):return struct.pack('<HHf',f,slot,x)
cases=[(0,[2,3,0,1],[e(0,1,1),e(1,1,2),e(0,0,0),e(1,0,4)]),(1,[0,1,2,3],[e(0,0,0),e(1,0,5),e(0,1,2),e(1,1,4)]),(0,[0,3,1,2],[e(0,0,3),e(0,1,1),e(1,1,2),e(1,0,4)]),(1,[0,3,1,2],[e(0,0,0),e(0,1,2),e(1,1,3),e(1,0,1)])]
corpus=b''.join(struct.pack('<I4H',axis,*ranks)+b''.join(es) for axis,ranks,es in cases);(O/'cases.bin').write_bytes(corpus)
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
GuardE DWORD 0A5A5A5A5h
Endpoints DWORD 8 DUP(0)
AfterE DWORD 0A5A5A5A5h
GuardP DWORD 0A5A5A5A5h
Pairs DWORD 4 DUP(0CCCCCCCCh)
AfterP DWORD 0A5A5A5A5h
Capture DWORD 23 DUP(0)
Results DWORD 92 DUP(0)
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
'''
for k,(axis,ranks,entries) in enumerate(cases):
 s+=f''' mov edi,OFFSET Manager
 xor eax,eax
 mov ecx,14
 rep stosd
 mov dword ptr [Manager],4
 mov dword ptr [Manager+8],2
 mov dword ptr [Manager+0Ch],OFFSET Pairs
 mov dword ptr [Manager+14h],OFFSET Slots
 mov dword ptr [Manager+20h],4
 mov dword ptr [Manager+{0x24+4*axis:02X}h],OFFSET Endpoints
 mov dword ptr [Slots],10001h
 mov dword ptr [Slots+4],0
 mov dword ptr [Slots+16],20002h
 mov dword ptr [Slots+20],0
 mov dword ptr [Slots+8],00420041h
 mov dword ptr [Slots+12],00440043h
 mov dword ptr [Slots+24],00520051h
 mov dword ptr [Slots+28],00540053h
 mov edi,OFFSET Pairs
 mov eax,0CCCCCCCCh
 mov ecx,4
 rep stosd
'''
 for slot in range(2):
  for side in range(2):s+=f' mov word ptr [Slots+{slot*16+8+axis*2+side*4}],{ranks[slot*2+side]}\n'
 for i,entry in enumerate(entries):
  w,v=struct.unpack('<II',entry);s+=f' mov dword ptr [Endpoints+{i*8}],0{w:08X}h\n mov dword ptr [Endpoints+{i*8+4}],0{v:08X}h\n'
 s+=f''' fninit
 fldcw word ptr [CW]
 ldmxcsr [MX]
 mov SavedESP,esp
 push 0
 push 0
 push {axis}
 push OFFSET Manager
 call {spec[k][0]}
 add esp,16
 cmp esp,SavedESP
 jne Failure
'''
 for g in ['GuardM','AfterM','GuardS','AfterS','GuardE','AfterE','GuardP','AfterP']:s+=f' cmp {g},0A5A5A5A5h\n jne Failure\n'
 s+=f''' cld
 mov esi,OFFSET Slots
 mov edi,OFFSET Capture
 mov ecx,8
 rep movsd
 mov esi,OFFSET Endpoints
 mov edi,OFFSET Capture+32
 mov ecx,8
 rep movsd
 mov esi,OFFSET Pairs
 mov edi,OFFSET Capture+64
 mov ecx,4
 rep movsd
 mov eax,[Manager+8]
 mov [Capture+80],eax
 fnstcw word ptr [Capture+84]
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
 push 368
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,368
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 368
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
UnexpectedHelper PROC
 push 7
 call _ExitProcess@4
UnexpectedHelper ENDP
'''
def db(x):return ''.join(' DB '+','.join('0%02Xh'%v for v in x[i:i+20])+'\n' for i in range(0,len(x),20))
for (name,a,n,calls),b in zip(spec,frags):
 s+=name+' PROC\n';pos=0
 for call in calls:
  off=call-a;assert b[off]==0xe8;s+=db(b[pos:off+1])+' DD UnexpectedHelper - ($ + 4)\n';pos=off+5
 s+=db(b[pos:])+name+' ENDP\n'
s+='END HostMain\n';(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S]+[O/(x[0]+'.bin') for x in spec];m={'host_executed':False,'functions':[(n,hex(a),l,[hex(c) for c in cs]) for n,a,l,cs in spec],'patches':'all six pair-related calls redirected to exit-7 trap','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'synthetic two-slot four-endpoint unique finite records; pair effect flag 0, no tombstones, sorted update caller or lifecycle'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
