"""Independent static source/design review of sparse pose carrier; never execute."""
import hashlib,json,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from cell_schema_probe import SourceImage,SOURCE_SHA256
import anim_root_pose_native_probe_v6 as v6

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def block(s,a,b):return s.split(a,1)[1].split(b,1)[0]
def main(out):
 prep=ROOT/'target/animation/root-pose-sparse-carrier-design-v1';asm=prep/'host-x86.asm';text=asm.read_text();old=(ROOT/'target/animation/root-pose-conversion-host-v6/host-x86.asm').read_text();image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));audit=v6.v3.source_audit(image)
 assert sha(v6.__file__)=='27370aa2b5b5147f72183b8797974b9c4135067fed86e89fb82ed52622d571b9'
 assert block(text,'ValidateRecord PROC','HostMain PROC')==block(old,'ValidateRecord PROC','HostMain PROC')
 assert block(text,'NextRecord:','HostMain ENDP')==block(old,'NextRecord:','HostMain ENDP')
 assert block(text,'HostFailure PROC','HostFailure ENDP')==block(old,'HostFailure PROC','HostFailure ENDP')
 imports=re.findall(r'^EXTERN _(\w+)@\d+:PROC$',text,re.M);assert set(imports)=={'GetStdHandle','ReadFile','WriteFile','ExitProcess','GetModuleHandleA'} and len(imports)==5
 startup=block(text,'HostMain PROC','NextRecord:');assert startup.startswith('\ncld\npush 0\ncall _GetModuleHandleA@4\ncmp eax,000590000h\njne HostFailure\n')
 assert not any(x in text for x in ['VirtualAlloc','VirtualProtect','FlushInstructionCache','MappingFailure'])
 checks=[]
 for a,b in v6.CODE:
  needle=f'mov esi,OFFSET Original_{a:08X}\nmov edi,0{a:08X}h\nmov ecx,{b-a}\nrepe cmpsb\njne HostFailure';assert needle in startup
  assert not(a<0x591000 and b>0x590000);checks.append(dict(start=f'{a:08X}',end=f'{b:08X}',sha256=hashlib.sha256(image.read(a,b-a)).hexdigest()))
 for d in audit['data']:
  a=int(d['address'],16);assert not(a<0x591000 and a+d['bytes']>0x590000);assert f'mov edi,0{a:08X}h\nmov ecx,{d["bytes"]}\nrepe cmpsb\njne HostFailure' in startup
 for a in audit['unreachable_calls']:assert f'cmp byte ptr [0{a}h],0CCh\njne HostFailure' in startup
 for a in v6.v3.GLOBALS:assert f'cmp dword ptr [0{a:08X}h],0\njne HostFailure' in startup
 assert 'CarrierExitBytes BYTE 0B8h\nDWORD OFFSET OriginalDone\nBYTE 0FFh,0E0h' in text
 assert 'mov esi,OFFSET CarrierExitBytes\nmov edi,00099DBFEh\nmov ecx,7\nrepe cmpsb\njne HostFailure' in startup
 assert 'cmp SavedStack,0\nje FailureStackReady\nmov esp,SavedStack\nFailureStackReady::\npush 71\ncall _ExitProcess@4' in text
 report=dict(schema='sparse-pose-carrier-independent-source-design-v1',source_sha256=SOURCE_SHA256,no_blocking_source_design_issue_found=False,blocking_issues=[dict(kind="unresolved_import",asm_line=1322,symbol="_GetLastError@0",reason="Retained ReadMore broken-pipe path calls GetLastError, but its EXTERN is removed and five-import contract excludes it. Retain explicit sixth import or reviewed owned protocol rewrite before linked admission.")],unchanged_v6_validate_runonce_return_failure_contract=True,readonly_startup_before_input=True,exact_five_import_declarations=True,import_call_closure=False,first_header_page_outside_approved_spans_and_data=True,source_spans=checks,unreachable_targets=len(audit['unreachable_calls']),source_pages_writable=False,new_native_execution=False,host_executed=False,linked_audit_performed=False,admitted_for_execution=False,pins=[dict(path=str(p.relative_to(ROOT)),sha256=sha(p)) for p in [ROOT/'tools/anim_root_pose_sparse_carrier_v1.py',asm,prep/'manifest.json',prep/'design-review.md',ROOT/'tools/anim_root_pose_native_probe_v6.py',ROOT/'target/animation/root-pose-conversion-host-v6/branch-register-ledger-v1.json']],metadata_reasoning='New ImageBase590000; high owned RVA+=29000000-590000 preserves original owned VAs; import directory/name/thunk RVA fields require same delta; source absolute bytes stay fixed. No unknown PE directories may survive. Actual converted PE remains unreviewed.',required_linked_checks=['Actual rebased five imports and distinct lookup/IAT tables','Owned instruction bytes unchanged, only declared import RVA DWORD edits','All six native spans/constants/globals/INT3/slack/gateway bytes exact','PE entry,header fields,section extents/raw offsets/protections/zero gaps','No ASLR/relocs/TLS/unapproved directories; full VA reservation OS behavior is untested'])
 p=Path(out);p.resolve().relative_to((ROOT/'target').resolve());p.write_text(json.dumps(report,indent=2)+'\n');print(sha(p))
if __name__=='__main__':main(sys.argv[1])
