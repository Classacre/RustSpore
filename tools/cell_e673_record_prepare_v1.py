"""Build-only isolated E673A0 record slices; never run original bytes."""
import hashlib,json,struct,subprocess
from pathlib import Path
import pefile
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'target/cell-e673-record-prep-v2';SOURCE=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe')
sha=lambda b:hashlib.sha256(b).hexdigest();b=lambda f:struct.unpack('<I',struct.pack('<f',f))[0]
S='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
ranges=[(0xe616c0,0xe6170d,'233dbf636987d15096f54508905e9569f4bd48990ef6063a40dfebd9c673e1f9'),(0xe674dd,0xe674e0,'eb5237e9a61ef8c9bb4bdfefa8f1396104a56dd75c52cb2fb154888bcb01084a'),(0xe674fe,0xe6750d,'325a2d2cc8273841f710514f209a19baa41f981f2ee87622d94771aeb34d2684'),(0xe67524,0xe6756e,'fe80d46b69fdf42b835139dfbc5ccfd3120aa9de05cd2944f83ce75ba823199b')]
assert not OUT.exists();OUT.mkdir(parents=True);assert sha(SOURCE.read_bytes())==S
pe=pefile.PE(str(SOURCE));frags=[]
for i,(a,z,h) in enumerate(ranges):
 f=pe.get_data(a-pe.OPTIONAL_HEADER.ImageBase,z-a);assert sha(f)==h;frags.append(f);(OUT/f'fragment{i}.bin').write_bytes(f)
def db(v):return '\n'.join(' DB '+','.join('0%02Xh'%x for x in v[i:i+24]) for i in range(0,len(v),24))
keys=[0x80000000,0x10002,0x7fffffff];queries=[0x80000000,0x80000001,0x10002,0x10003,0x7fffffff,0xffffffff]
old=bytearray([0xa5]*44);struct.pack_into('<8I',old,0,0x10002,0x90001,0xa0002,5,6,1,2,3);old[32]=1;struct.pack_into('<2I',old,36,8,9)
current=42;fresh=[4,5,6]
newcases=[([0x20002,0x10001],[4,5,6],42,b(5.),b(5.)),([0x20002,0x10001],[4,5,6],42,b(5.),b(4.)),([0xa0002,0x90001],[7,8,9],99,b(5.),b(6.)),([0x20002,0x10001],[4,5,6],42,0x80000000,0)]
corpus=struct.pack('<9I',*keys,*queries)+bytes(old)+struct.pack('<4I',current,*fresh)+b''.join(struct.pack('<8I',*ids,*point,cur,dist,thresh) for ids,point,cur,dist,thresh in newcases)
(OUT/'cases.bin').write_bytes(corpus)
lookup_rows=b''.join(struct.pack('<I',k)+bytes(40) for k in keys)
lookup_blocks=[]
for i,q in enumerate(queries):
 lookup_blocks.append(f''' mov LookKey,0{q:08X}h
 push {i%2}
 push OFFSET LookKey
 push OFFSET LookRows+132
 push OFFSET LookRows
 call LookupFragment
 add esp,16
 cmp eax,OFFSET LookRows+132
 je LookupMiss{i}
 mov edx,LookKey
 cmp dword ptr [eax],edx
 jne LookupMiss{i}
 sub eax,OFFSET LookRows
 xor edx,edx
 mov ecx,44
 div ecx
 jmp LookupDone{i}
LookupMiss{i}:
 mov eax,0FFFFFFFFh
LookupDone{i}:
 mov dword ptr [Results+{i*4}],eax''')
newdata=[];newblocks=[]
for i,(ids,point,cur,dist,thresh) in enumerate(newcases):
 lo,hi=sorted(ids);key=((lo&65535)<<16)|(hi&65535);entry=bytearray([0xa5]*44);struct.pack_into('<I',entry,0,key)
 newdata.append(f'NG{i} DWORD 0A5A5A5A5h\nEntry{i} LABEL BYTE\n{db(entry)}\nNE{i} DWORD 0A5A5A5A5h\nPG{i} DWORD 0A5A5A5A5h\nPoint{i} DWORD '+','.join(f'0{x:08X}h' for x in point)+f'\nPE{i} DWORD 0A5A5A5A5h\nTG{i} DWORD 0A5A5A5A5h\nThreshold{i} BYTE 70h DUP(0)\nValue{i} DWORD 0{thresh:08X}h\nTE{i} DWORD 0A5A5A5A5h')
 # Result byte offset: lookup 24, existing 44, then each record 44 and MXCSR 4.
 o=68+i*48
 newblocks.append(f''' ldmxcsr [MX]
 sub esp,40h
 mov dword ptr [esp],0A5A5A5A5h
 mov dword ptr [esp+3Ch],0A5A5A5A5h
 mov dword ptr [esp+14h],OFFSET Threshold{i}
 mov dword ptr [esp+20h],0{hi:08X}h
 mov dword ptr [esp+28h],0{dist:08X}h
 mov dword ptr [esp+2Ch],0{cur:08X}h
 mov eax,OFFSET Entry{i}+4
 mov ebp,0{lo:08X}h
 mov edi,OFFSET Point{i}
 call NewFragment
 cmp dword ptr [esp],0A5A5A5A5h
 jne Failure
 cmp dword ptr [esp+3Ch],0A5A5A5A5h
 jne Failure
 add esp,40h
 cmp NG{i},0A5A5A5A5h
 jne Failure
 cmp NE{i},0A5A5A5A5h
 jne Failure
 cmp PG{i},0A5A5A5A5h
 jne Failure
 cmp PE{i},0A5A5A5A5h
 jne Failure
 cmp TG{i},0A5A5A5A5h
 jne Failure
 cmp TE{i},0A5A5A5A5h
 jne Failure
 cmp Value{i},0{thresh:08X}h
 jne Failure
 mov esi,OFFSET Entry{i}
 mov edi,OFFSET Results+{o}
 mov ecx,11
 rep movsd
 stmxcsr dword ptr [Results+{o+44}]''')
asm=f'''.686p
.xmm
.model flat
option casemap:none
EXTERN _GetStdHandle@4:PROC
EXTERN _WriteFile@20:PROC
EXTERN _ExitProcess@4:PROC
.data
MX DWORD 01F80h
GuardLook DWORD 0A5A5A5A5h
LookRows LABEL BYTE
{db(lookup_rows)}
LookEnd DWORD 0A5A5A5A5h
LookKey DWORD 0
EG DWORD 0A5A5A5A5h
Existing LABEL BYTE
{db(old)}
EE DWORD 0A5A5A5A5h
FG DWORD 0A5A5A5A5h
Fresh DWORD 4,5,6
FE DWORD 0A5A5A5A5h
{chr(10).join(newdata)}
Results BYTE {68+len(newcases)*48} DUP(0CCh)
Written DWORD 0
.code
PUBLIC HostMain
HostMain PROC
 cld
{chr(10).join(lookup_blocks)}
 cmp GuardLook,0A5A5A5A5h
 jne Failure
 cmp LookEnd,0A5A5A5A5h
 jne Failure
 mov eax,OFFSET Existing+4
 mov ecx,{current}
 call ExistingStamp
 mov eax,OFFSET Existing+4
 mov esi,Fresh
 mov ebx,Fresh+4
 mov edi,Fresh+8
 call ExistingPoint
 cmp EG,0A5A5A5A5h
 jne Failure
 cmp EE,0A5A5A5A5h
 jne Failure
 cmp FG,0A5A5A5A5h
 jne Failure
 cmp FE,0A5A5A5A5h
 jne Failure
 mov esi,OFFSET Existing
 mov edi,OFFSET Results+24
 mov ecx,11
 rep movsd
{chr(10).join(newblocks)}
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET Written
 push {68+len(newcases)*48}
 push OFFSET Results
 push eax
 call _WriteFile@20
 test eax,eax
 jz Failure
 cmp Written,{68+len(newcases)*48}
 jne Failure
 push 0
 call _ExitProcess@4
Failure:
 push 7
 call _ExitProcess@4
HostMain ENDP
LookupFragment PROC
{db(frags[0])}
LookupFragment ENDP
ExistingStamp PROC
{db(frags[1])}
 ret
ExistingStamp ENDP
ExistingPoint PROC
{db(frags[2])}
 ret
ExistingPoint ENDP
NewFragment PROC
{db(frags[3])}
 ret
NewFragment ENDP
END HostMain
'''
(OUT/'host-x86.asm').write_text(asm,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib')
cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{OUT}/host-x86.obj',str(OUT/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{OUT}/host-x86.exe',f'/MAP:{OUT}/host-x86.map',str(OUT/'host-x86.obj'),str(lib)]]
logs=[]
for cmd in cmds:
 p=subprocess.run(cmd,capture_output=True,text=True);logs.append({'args':cmd,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
 if p.returncode:(OUT/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failure')
paths=[Path(__file__),OUT/'cases.bin',OUT/'host-x86.asm',OUT/'host-x86.exe',OUT/'host-x86.obj',OUT/'host-x86.map',*[OUT/f'fragment{i}.bin' for i in range(4)],sdk/'ml.exe',sdk/'link.exe',lib,ROOT/'crates/spore-bevy/src/cell_source_contact_record_v1.rs',ROOT/'crates/spore-core/src/native_sse.rs']
m={'host_executed':False,'source_sha256':S,'ranges':[{'start':hex(a),'end_exclusive':hex(z),'sha256':h} for a,z,h in ranges],'pins':{str(p):sha(p.read_bytes()) for p in paths},'commands':logs,'limits':'Independent fixed synthetic fragments with admitted stable lookup and record pointers; no E65810/provider/callback, actual actor identity, failure effects or live contact.'}
(OUT/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
