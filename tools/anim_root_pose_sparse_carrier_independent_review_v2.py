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
 prep=ROOT/'target/animation/root-pose-sparse-carrier-root-v1';asm=prep/'host-x86.asm';text=asm.read_text();old=(ROOT/'target/animation/root-pose-conversion-host-v6/host-x86.asm').read_text();image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));audit=v6.v3.source_audit(image)
 assert sha(v6.__file__)=='27370aa2b5b5147f72183b8797974b9c4135067fed86e89fb82ed52622d571b9'
 assert block(text,'ValidateRecord PROC','HostMain PROC')==block(old,'ValidateRecord PROC','HostMain PROC')
 assert block(text,'NextRecord:','HostMain ENDP')==block(old,'NextRecord:','HostMain ENDP')
 assert block(text,'HostFailure PROC','HostFailure ENDP')==block(old,'HostFailure PROC','HostFailure ENDP')
 imports=re.findall(r'^EXTERN _(\w+)@\d+:PROC$',text,re.M);assert set(imports)=={'GetStdHandle','ReadFile','WriteFile','ExitProcess','GetModuleHandleA','GetLastError'} and len(imports)==6
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
 report=dict(schema='sparse-pose-carrier-independent-actual-linked-peer-v2',source_sha256=SOURCE_SHA256,no_blocking_source_design_issue_found=True,blocking_issues=[],unchanged_v6_validate_runonce_return_failure_contract=True,readonly_startup_before_input=True,exact_six_import_declarations=True,import_call_closure=True,first_header_page_outside_approved_spans_and_data=True,source_spans=checks,unreachable_targets=len(audit['unreachable_calls']),source_pages_writable=False,new_native_execution=False,host_executed=False,linked_audit_performed=False,admitted_for_execution=False,pins=[dict(path=str(p.relative_to(ROOT)),sha256=sha(p)) for p in [ROOT/'tools/anim_root_pose_sparse_carrier_v2.py',asm,prep/'manifest.json',ROOT/'tools/anim_root_pose_native_probe_v6.py',ROOT/'target/animation/root-pose-conversion-host-v6/branch-register-ledger-v1.json']],metadata_reasoning='New ImageBase590000; high owned RVA+=29000000-590000 preserves original owned VAs; import directory/name/thunk RVA fields require same delta; source absolute bytes stay fixed. No unknown PE directories may survive. Actual converted PE remains unreviewed.',required_linked_checks=['Actual rebased five imports and distinct lookup/IAT tables','Owned instruction bytes unchanged, only declared import RVA DWORD edits','All six native spans/constants/globals/INT3/slack/gateway bytes exact','PE entry,header fields,section extents/raw offsets/protections/zero gaps','No ASLR/relocs/TLS/unapproved directories; full VA reservation OS behavior is untested'])
 from types import SimpleNamespace
 import anim_root_pose_sparse_carrier_audit_v1 as linked
 fresh=linked.audit(SimpleNamespace(carrier=prep/'carrier-x86.exe',owned_exe=prep/'host-x86.exe',owned_map=prep/'host-x86.map',source_exe=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')))
 retained=json.loads((prep/'independent-linked-audit.json').read_text());assert fresh==retained and fresh['static_pass'] and not fresh['native_executed']
 assert fresh['carrier_sha256']=='4a3c18ffa486543e853d1c1a15cc3ddccfa74d36da274c344da1b5d60ef7fb2c'
 assert fresh['owned_sha256']=='c9175b6c6ac179a7c2e1cd73cb3f948d1816ee83eb349f63ec43756575da6c03'
 peer_path=ROOT/'target/animation/root-pose-v6-independent-font-review-v1.json';prior=json.loads(peer_path.read_text())
 for key in ['no_blocking_static_issue_found','original_spans_exact','unreachable_calls_closed','partial_entry_frame_registers_closed','guarded_memory_bounds_closed']:assert prior[key] is True
 for key in ['source_byte_exact','owned_instruction_byte_exact','source_pages_and_slack_exact','imports_OFT_IAT_closed','unknown_RVA_directories_rejected','owned_control_closed','guards_closed']:assert fresh[key] is True
 im=linked.Image(prep/'carrier-x86.exe');sy=linked.syms(prep/'host-x86.map');imports_map,edits=im.imports()
 # Actual lookup/IAT tables must be distinct; all six imported thunk sites resolve.
 cursor=im.base+im.dirs()[1][0]
 import struct
 while True:
  fields=struct.unpack('<5I',im.read(cursor,20))
  if not any(fields):break
  assert fields[0]!=fields[4];cursor+=20
 from native_crt_acos_probe import decoder
 failure=list(decoder().disasm(im.read(sy['HostFailure'],sy['OwnedEnd']-sy['HostFailure']),sy['HostFailure']))
 assert [r.mnemonic for r in failure[:5]]==['cmp','je','mov','push','call']
 assert failure[2].op_str.startswith('esp,') and failure[3].operands[0].imm==71
 report.update(no_blocking_static_issue_found=True,original_spans_exact=True,unreachable_calls_closed=True,partial_entry_frame_registers_closed=True,guarded_memory_bounds_closed=True,native_executed=False,host_executed=False,linked_audit_performed=True,admitted_for_execution=False,static_admission_for_root_owned_smoke=True,prior_v1_blocker_fixed=True,linked_audit=fresh,metadata_reasoning='Actual converted PE reproduces the reviewed independent audit exactly; six imports include retained broken-pipe GetLastError. Every owned VA and instruction stays unchanged, source bytes/slack exact, only declared import metadata RVAs change. Distinct lookup/IAT tables verified. Conditional SavedStack restore decoded in actual linked failure exit.',required_linked_checks=[])
 paths=[ROOT/'tools/anim_root_pose_sparse_carrier_v2.py',ROOT/'tools/anim_root_pose_sparse_carrier_audit_v1.py',Path(__file__),ROOT/'tools/anim_root_pose_sparse_carrier_native_run_v1.py',ROOT/'tools/anim_root_pose_native_probe_v6.py',ROOT/'tools/anim_root_pose_native_probe.py',peer_path,ROOT/'target/animation/root-pose-conversion-host-v6/branch-register-ledger-v1.json',*[prep/name for name in ['host-x86.asm','manifest.json','host-x86.exe','host-x86.map','carrier-x86.exe','carrier-build-review.json','independent-linked-audit.json']]]
 report['pins']={str(p.relative_to(ROOT)):sha(p) for p in paths}
 p=Path(out);p.resolve().relative_to((ROOT/'target').resolve())
 with p.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(p))
if __name__=='__main__':main(sys.argv[1])
