"""Build-only uninstrumented BBA7C0 activation-context comparison host."""
from pathlib import Path
import hashlib,json,subprocess
R=Path(__file__).resolve().parents[1];P=R/'target/cell-fresh-sort-prep-v2';O=R/'target/cell-fresh-sort-activated-prep-v1';S=Path(r'D:\Games\SPORE Collection\SporeBin\SporeApp.exe');D=Path(r'D:\Games\SPORE Collection\SporeBin\msvcr90.dll');M=Path(r'D:\Games\SPORE Collection\SporeBin\Microsoft.VC90.CRT.manifest');h=lambda b:hashlib.sha256(b).hexdigest();assert not O.exists();O.mkdir(parents=True)
assert h((P/'manifest.json').read_bytes())=='c0ddf9def312eb4ea4482cbee191393f54dd6e8b155bb9aad7ff8645e02a51da';assert h((P/'host-x86.asm').read_bytes())=='d96a665fe27611c97ffe3b36f02b12efffba1e5360abd32a3a074e1e702766ca';assert h(S.read_bytes())=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a';assert h(D.read_bytes())=='ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185';assert h(M.read_bytes())=='9e0220511d4ebdb014cc17ecb8319d57e3b0fea09681a80d8084aa8647196582'
s=(P/'host-x86.asm').read_text(encoding='ascii');s=s.replace('EXTERN _GetLastError@0:PROC','EXTERN _GetLastError@0:PROC\nEXTERN _CreateActCtxW@4:PROC\nEXTERN _ActivateActCtx@8:PROC',1)
def dw(path):return ','.join('0%04Xh'%ord(c) for c in str(path))+',0'
fields='''SourceExe WORD %s
AssemblyDir WORD %s
ActivationContext DWORD 32,0Ch,OFFSET SourceExe,0,OFFSET AssemblyDir,1,0,0
ActivationHandle DWORD 0
ActivationCookie DWORD 0
'''%(dw(S),dw(S.parent));s=s.replace('QsortName BYTE "qsort",0\n','QsortName BYTE "qsort",0\n'+fields,1)
needle=' call _SetErrorMode@4\n push 8\n';assert s.count(needle)==1;s=s.replace(needle,''' call _SetErrorMode@4
 push OFFSET ActivationContext
 call _CreateActCtxW@4
 cmp eax,0FFFFFFFFh
 je LoaderFailure
 mov ActivationHandle,eax
 push OFFSET ActivationCookie
 push eax
 call _ActivateActCtx@8
 test eax,eax
 jz LoaderFailure
 push 8
''',1);assert 'QsortTap' not in s;assert s.count(' DD OFFSET QsortPtr')==1
(O/'host-x86.asm').write_text(s,encoding='ascii');sdk=Path(r'C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x86');lib=Path(r'C:\Program Files (x86)\Windows Kits\10\Lib\10.0.28000.0\um\x86\kernel32.lib');cmds=[[str(sdk/'ml.exe'),'/nologo','/c','/coff',f'/Fo{O}/host-x86.obj',str(O/'host-x86.asm')],[str(sdk/'link.exe'),'/NOLOGO','/MACHINE:X86','/SUBSYSTEM:CONSOLE','/ENTRY:HostMain','/DYNAMICBASE:NO','/SAFESEH:NO',f'/OUT:{O}/host-x86.exe',f'/MAP:{O}/host-x86.map',str(O/'host-x86.obj'),str(lib)]];logs=[]
for cmd in cmds:
 q=subprocess.run(cmd,capture_output=True,text=True,creationflags=subprocess.CREATE_NO_WINDOW);logs.append({'args':cmd,'exit':q.returncode,'stdout':q.stdout,'stderr':q.stderr})
 if q.returncode:(O/'build-failure.json').write_text(json.dumps(logs,indent=2));raise SystemExit('build failed')
paths=[Path(__file__),P/'manifest.json',P/'host-x86.asm',P/'sort.bin',P/'probe.bin',P/'compare.bin',P/'cases.bin',O/'host-x86.asm',O/'host-x86.exe',O/'host-x86.obj',O/'host-x86.map',sdk/'ml.exe',sdk/'link.exe',lib,S,D,M,R/'target/cell-fresh-sort-result-v1/receipt.json',R/'target/cell-fresh-sort-diagnostic-result-v1/receipt.json',R/'target/cell-fresh-sort-activation-private-result-v1/receipt.json',R/'crates/spore-core/src/cell_fresh_sweep_registration_v1.rs',R/'crates/spore-core/src/cell_sweep_endpoint_compare_v1.rs',R/'crates/spore-core/examples/cell_fresh_sweep_sort_oracle_v1.rs',R/'target/debug/examples/cell_fresh_sweep_sort_oracle_v1.exe'];m={'host_executed':False,'activation':'ACTCTXW cbSize=32 flags=0x0c, lpSource=SporeApp.exe, lpAssemblyDirectory=SporeBin, resource ordinal 1, activate before direct qsort load','pins':{str(x):h(x.read_bytes()) for x in paths},'commands':logs,'limits':'Fresh old_count=0, unique finite disjoint buffers; installed CRT qsort loaded directly. Windows loader/in-memory integrity and activation lifetime admitted; private desktop runner required; no live actor or persistent state claim.'};(O/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m['pins'],indent=2))
