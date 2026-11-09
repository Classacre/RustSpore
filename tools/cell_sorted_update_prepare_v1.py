"""Build-only whole BBB880 sorted-update host with pair helpers trapped."""
from pathlib import Path
import pefile,hashlib,struct,json,subprocess
R=Path(__file__).resolve().parents[1];O=R/'target/cell-sorted-update-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True);assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';p=pefile.PE(str(S),fast_load=True)
spec=[('UpdateSource',0xbbb880,381,[(0xbbb959,'MinLeft'),(0xbbb97e,'MaxRight'),(0xbbb9a7,'MinRight'),(0xbbb9d0,'MaxLeft')]),('MinLeft',0xbbb020,264,[(0xbbb097,'UnexpectedHelper')]),('MaxRight',0xbbb130,258,[(0xbbb1a5,'UnexpectedHelper')]),('MinRight',0xbbb240,209,[(0xbbb2b3,'UnexpectedHelper'),(0xbbb2ca,'UnexpectedHelper')]),('MaxLeft',0xbbb320,207,[(0xbbb391,'UnexpectedHelper'),(0xbbb3a8,'UnexpectedHelper')])]
frags=[]
for name,a,n,calls in spec:
 b=p.get_data(a-p.OPTIONAL_HEADER.ImageBase,n);assert len(b)==n and b[-1:]==b'\xc3';(O/(name+'.bin')).write_bytes(b);frags.append(b)
cases=[(0.,5.,2.,3.,2.5,1.,2.75,4.),(1.,4.,0.,5.,-1.,-2.,6.,7.),(0.,5.,2.,3.,-1.,1.,4.,6.),(0.,5.,2.,3.,-0.,0.,5.,5.)];corpus=b''.join(struct.pack('<8f',*c) for c in cases);(O/'cases.bin').write_bytes(corpus)
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
GuardX DWORD 0A5A5A5A5h
OldX DWORD 8 DUP(0)
AfterX DWORD 0A5A5A5A5h
GuardY DWORD 0A5A5A5A5h
OldY DWORD 8 DUP(0)
AfterY DWORD 0A5A5A5A5h
GuardP DWORD 0A5A5A5A5h
Pairs DWORD 4 DUP(0)
AfterP DWORD 0A5A5A5A5h
GuardV DWORD 0A5A5A5A5h
MinV DWORD 2 DUP(0)
MaxV DWORD 2 DUP(0)
AfterV DWORD 0A5A5A5A5h
Capture DWORD 31 DUP(0)
Results DWORD 124 DUP(0)
SavedESP DWORD 0
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
'''
for k,c in enumerate(cases):
 entries=sorted([(0,0,c[0]),(1,0,c[1]),(0,1,c[2]),(1,1,c[3])],key=lambda x:x[2]);ranks=[[0]*4 for _ in range(2)]
 for i,(flag,slot,v) in enumerate(entries):
  for axis in range(2):ranks[slot][axis+2*flag]=i
 s+=''' mov edi,OFFSET Manager
 xor eax,eax
 mov ecx,14
 rep stosd
 mov dword ptr [Manager],4
 mov dword ptr [Manager+8],2
 mov dword ptr [Manager+0Ch],OFFSET Pairs
 mov dword ptr [Manager+14h],OFFSET Slots
 mov dword ptr [Manager+20h],4
 mov dword ptr [Manager+24h],OFFSET OldX
 mov dword ptr [Manager+28h],OFFSET OldY
 mov dword ptr [Slots],10001h
 mov dword ptr [Slots+4],0
 mov dword ptr [Slots+16],20002h
 mov dword ptr [Slots+20],0
 mov edi,OFFSET Pairs
 mov eax,0CCCCCCCCh
 mov ecx,4
 rep stosd
'''
 for slot in range(2):
  for j in range(4):s+=f' mov word ptr [Slots+{slot*16+8+j*2}],{ranks[slot][j]}\n'
 for name in ['OldX','OldY']:
  for i,(flag,slot,v) in enumerate(entries):
   bits=struct.unpack('<I',struct.pack('<f',v))[0];s+=f' mov dword ptr [{name}+{8*i}],0{(slot<<16|flag):08X}h\n mov dword ptr [{name}+{8*i+4}],0{bits:08X}h\n'
 for j,v in enumerate(c[4:]):
  bits=struct.unpack('<I',struct.pack('<f',v))[0];s+=f' mov dword ptr [{"MinV" if j<2 else "MaxV"}+{(j%2)*4}],0{bits:08X}h\n'
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
 for g in ['GuardM','AfterM','GuardS','AfterS','GuardX','AfterX','GuardY','AfterY','GuardP','AfterP','GuardV','AfterV']:s+=f' cmp {g},0A5A5A5A5h\n jne Failure\n'
 for name,offset,count in [('Slots',0,8),('OldX',32,8),('OldY',64,8),('Pairs',96,4)]:s+=f' cld\n mov esi,OFFSET {name}\n mov edi,OFFSET Capture+{offset}\n mov ecx,{count}\n rep movsd\n'
 s+=f''' mov eax,[Manager+8]
 mov [Capture+112],eax
 fnstcw word ptr [Capture+116]
 fnstsw word ptr [Capture+118]
 stmxcsr [Capture+120]
 mov esi,OFFSET Capture
 mov edi,OFFSET Results+{k*124}
 mov ecx,31
 rep movsd
'''
s+=''' push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push 496
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,496
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push -11
 call _GetStdHandle@4
 mov ebx,eax
 push 0
 push OFFSET Written
 push 496
 push OFFSET Results
 push ebx
 call _WriteFile@20
 push 0
 push OFFSET Written
 push 124
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
 for address,target in calls:
  off=address-a;assert b[off]==0xe8;s+=db(b[pos:off+1])+f' DD {target} - ($ + 4)\n';pos=off+5
 s+=db(b[pos:])+name+' ENDP\n'
s+='END HostMain\n';(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),O/'cases.bin',O/'host-x86.asm',O/'host-x86.obj',O/'host-x86.exe',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S]+[O/(x[0]+'.bin') for x in spec];m={'host_executed':False,'functions':[(n,hex(a),l,[(hex(c),t) for c,t in cs]) for n,a,l,cs in spec],'patches':'four BBB880 calls to source helper copies, six helper pair-related calls to nonreturning trap','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'sorted-only synthetic two-slot finite unique endpoint snapshots; no pending branch, pair effect, aliasing or lifecycle'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
