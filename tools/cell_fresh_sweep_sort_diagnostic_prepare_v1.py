"""Build-only diagnostic variant of fresh-sort host; no execution."""
from pathlib import Path
import hashlib,json,subprocess,re
R=Path(__file__).resolve().parents[1];P=R/'target/cell-fresh-sort-prep-v2';O=R/'target/cell-fresh-sort-diagnostic-prep-v2';h=lambda b:hashlib.sha256(b).hexdigest()
assert not O.exists();O.mkdir(parents=True)
assert h((P/'manifest.json').read_bytes())=='c0ddf9def312eb4ea4482cbee191393f54dd6e8b155bb9aad7ff8645e02a51da'
assert h((P/'host-x86.asm').read_bytes())=='d96a665fe27611c97ffe3b36f02b12efffba1e5360abd32a3a074e1e702766ca'
s=(P/'host-x86.asm').read_text(encoding='ascii')
macro='''TraceMarker MACRO marker
 pushfd
 pushad
 mov eax,marker
 call LogMarker
 popad
 popfd
ENDM
'''
s=s.replace('.data\n',macro+'.data\n',1).replace('QsortPtr DWORD 0\n','QsortPtr DWORD 0\nQsortTapPtr DWORD OFFSET QsortTap\nTraceWord DWORD 0\nTraceWritten DWORD 0\n',1)
assert s.count(' DD OFFSET QsortPtr')==1;s=s.replace(' DD OFFSET QsortPtr',' DD OFFSET QsortTapPtr',1)
s=s.replace('HostMain PROC\n cld','HostMain PROC\n cld\n TraceMarker 0A7000001h',1)
for idx,name in enumerate(['_SetErrorMode@4','_LoadLibraryExW@12','_GetProcAddress@8','_GetModuleHandleExW@12','_GetModuleFileNameW@12','_lstrcmpiW@8'],2):
 old=' call '+name;assert s.count(old)==1;s=s.replace(old,old+'\n TraceMarker 0A700%04Xh'%idx,1)
assert s.count(' call SortSource')==6
counter=[0]
def mark_call(match):
 idx=counter[0];counter[0]+=1
 return ' TraceMarker 0A700%04Xh\n call SortSource ; traced\n TraceMarker 0A700%04Xh'%(0x100+idx,0x200+idx)
s=re.sub(r'(?m)^ call SortSource$',mark_call,s)
assert counter[0]==6
helpers='''LogMarker PROC
 mov TraceWord,eax
 push -11
 call _GetStdHandle@4
 push 0
 push OFFSET TraceWritten
 push 4
 push OFFSET TraceWord
 push eax
 call _WriteFile@20
 ret
LogMarker ENDP
QsortTap PROC
 TraceMarker 0A7000300h
 push dword ptr [esp+10h]
 push dword ptr [esp+10h]
 push dword ptr [esp+10h]
 push dword ptr [esp+10h]
 call dword ptr [QsortPtr]
 add esp,16
 TraceMarker 0A7000301h
 ret
QsortTap ENDP
'''
assert s.count('SortSource PROC')==1;s=s.replace('SortSource PROC',helpers+'SortSource PROC',1)
(O/'host-x86.asm').write_text(s,encoding='ascii')
sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),P/'manifest.json',P/'host-x86.asm',P/'sort.bin',P/'probe.bin',P/'compare.bin',P/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,Path(r'D:\Games\SPORE Collection\SporeBin\msvcr90.dll')];m={'host_executed':False,'reference_timeout_receipt':'target/cell-fresh-sort-result-v1/receipt.json','markers':'A7000001 start; 2..7 each loader call returned; A7000100..105 before each sort; A7000200..205 after each sort; A7000300 qsort entered; A7000301 qsort returned','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Diagnostic instrumentation changes qsort call stack and flushes markers. No parity assertion. No native execution before review.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
