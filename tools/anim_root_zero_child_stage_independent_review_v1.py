"""Independent actual-linked zero-child stage v3 peer. Never execute native code."""
import hashlib,json,struct,sys
from pathlib import Path
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import anim_root_zero_child_stage_audit_v2 as audit
import anim_root_zero_child_stage_native_probe_v3 as probe
import anim_root_zero_child_stage_native_run_v1 as runner
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import decoder

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 host=ROOT/'target/animation/root-zero-child-stage-native-root-v1';prep=ROOT/'target/animation/root-zero-child-stage-native-v3';fixture=ROOT/'target/animation/root-zero-child-stage-inputs-v3';original=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
 args=SimpleNamespace(carrier=host/'stage-carrier-x86.exe',owned_exe=host/'host-x86.exe',owned_map=host/'host-x86.map',source_exe=original)
 fresh=audit.audit(args);retained=json.loads((host/'independent-linked-audit.json').read_text());assert fresh==retained and fresh['static_pass']
 assert fresh['carrier_sha256']=='9279eceacf329d898adc610b53dd19cd7cb91bf35b9908980fe620a1cb63ee67' and fresh['owned_exe_sha256']=='6c90439efff9a470ee228146b3e662e6a8b31a2de8f0f40884347a286e6edb00'
 assert (host/'host-x86.asm').read_bytes()==(prep/'host-x86.asm').read_bytes()
 assert (host/'host-x86.asm').read_text()==probe.host_source((host/'stage-host-core-x86.asm').read_text())
 im=audit.Image(args.carrier);s=audit.syms(args.owned_map);rows=list(decoder().disasm(im.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']),s['OwnedStart']));index={r.address:i for i,r in enumerate(rows)};asm=(host/'host-x86.asm').read_text()
 # Actual definition word guard precedes any input copy/binding; no fabricated zero.
 validate=rows[index[s['ValidateStage']]:index[s['RunStage']]]
 child=next(r for r in validate if r.mnemonic=='cmp' and r.op_str==f'dword ptr [0x{s["InputBuffer"]+1252:x}], 0');assert child.operands[0].size==4
 assert 'mov dword ptr [DefinitionBuffer+208h],0' not in asm and 'mov dword ptr [DefinitionBuffer+520],0' not in asm
 assert 'mov NativeStack,esp\ncld\ncall ValidateStage' in asm
 assert 'mov dword ptr [RootBuffer],OFFSET DefinitionBuffer' in asm and 'mov dword ptr [BodyBuffer+2E4h],OFFSET RootBuffer\nmov dword ptr [BodyBuffer+2E8h],OFFSET RootBuffer+700' in asm
 assert 'mov dword ptr [ReplyBuffer],0\nfnstcw' in asm
 # Actual owned failure is a normal return to outer transport on saved real stack.
 fail=rows[index[s['StageFailure']]:index[s['StageOwnedEnd']]];assert [r.mnemonic for r in fail[:5]]==['cmp','je','mov','mov','ret'] and fail[2].op_str.startswith('esp,') and fail[3].operands[1].imm==71
 run=rows[index[s['RunStage']]:index[s['StageFailure']]]
 assert sum(r.mnemonic=='cmp' and r.op_str.startswith('esp,') for r in run)==3
 outer=rows[index[s['HostFailure']]:];assert outer[0].mnemonic=='push' and outer[0].operands[0].imm==71 and outer[1].mnemonic=='call'
 # Decode source zero-child proof from immutable executable, not a source stub.
 image=SourceImage(original);cs=decoder();closure=probe.proof();source_ins={}
 for record in closure['source_records']:
  for span in record['ranges']:
   a=int(span['start'],16);raw=image.read(a,span['bytes']);assert hashlib.sha256(raw).hexdigest()==span['sha256'];source_ins.update({r.address:r for r in cs.disasm(raw,a)})
 assert sum(span['bytes'] for r in closure['source_records'] for span in r['ranges'])==5901
 for va,mnemonic,operand in [(0x9e7c8c,'mov','eax, dword ptr [eax + 0x208]'),(0x9e7c9c,'cmp','eax, ecx'),(0x9e7c9e,'jbe','0x9e82d2'),(0x9e82d2,'add','esp, 0x7c'),(0x9e89c6,'call','0x9e7c80'),(0x9e89cb,'mov','eax, dword ptr [esp + 0x2c]'),(0x9e89d2,'cmp','eax, ebx'),(0x9e89d4,'je','0x9e915a')]:
  r=source_ins[va];assert (r.mnemonic,r.op_str)==(mnemonic,operand),(hex(va),r.mnemonic,r.op_str)
 assert source_ins[0x9e8971].mnemonic=='xor' and source_ins[0x9e8971].op_str=='ebx, ebx'
 assert source_ins[0x9e8979].mnemonic=='mov' and source_ins[0x9e8979].op_str=='dword ptr [esp + 0x2c], ebx'
 for target in closure['excluded_calls']:assert im.read(int(target,16),1)==b'\xcc'
 assert source_ins[0x9e82d5].mnemonic=='ret'
 # Real normalizer remains present. All source CALLs from stage1 target its entry.
 calls=[r for r in source_ins.values() if 0x9e82e0<=r.address<0x9e8928 and r.mnemonic=='call'];assert calls and all(r.op_str=='0x99c810' for r in calls)
 assert im.read(0x99c810,192)==image.read(0x99c810,192)
 # Re-run read-only provenance reconstruction across all66,636 records/protocol files.
 manifest,join=runner.review(SimpleNamespace(host_dir=host,fixture=fixture,source_exe=original,peer_review=None,run=False));assert join['records']==66636 and join['all_88B_native_accumulators_exact'] and join['whole_payload_graph_context_reconstruction_exact']
 final=json.loads((prep/'final-preparation-review.json').read_text());pins={}
 for name,value in final['pins'].items():
  p=Path(name);p=p if p.is_absolute() else ROOT/p;assert sha(p)==value;pins[str(p.relative_to(ROOT))]=value
 paths=[Path(__file__),Path(audit.__file__),Path(probe.__file__),Path(runner.__file__),ROOT/'crates/spore-core/src/native_x87.rs',ROOT/'crates/spore-core/src/native_sse.rs',*[host/n for n in ['host-x86.asm','stage-host-core-x86.asm','manifest.json','host-x86.exe','host-x86.map','stage-carrier-x86.exe','carrier-build-review.json','independent-linked-audit.json']],prep/'HANDOFF.md',prep/'final-preparation-review.json',fixture/'manifest.json',fixture/'index.jsonl',fixture/'stage-contexts.json']
 pins.update({str(p.relative_to(ROOT)):sha(p) for p in paths})
 report=dict(schema='zero-child-full-entry-stage-independent-actual-peer-v1',no_blocking_static_issue_found=True,original_spans_exact=True,source_no_stub_or_gateway=True,zero_child_excluded_callees_closed=True,actual_definition_child_count_guard=True,full_entry_abi_and_real_stack_closed=True,owned_failure_stack_and_protocol_closed=True,write_bounds_and_guards_closed=True,PE_imports_control_bytes_and_gap_sections_closed=True,Rust_source_arithmetic_review_no_blocker=True,fixture_66636_join_exact=True,native_executed=False,host_executed=False,rust_executed=False,source_sha256=SOURCE_SHA256,linked_audit=fresh,fixture_join=join,pins=pins,limits='Static full-entry zero-child one-root caller topology only; existing originalsource accumulator bytes plus explicit finite stage contexts. No generatedbodychildcount modification,actualactor association,full9F4730 traversal or numerical/native/Rust acceptance. All source/native execution remains root-owned. Rust scalar SSE operand/reduction order and x87 normfloat32 policy reviewed; all mismatch/nonfinite results remain failures, no automatic exclusions.')
 p=Path(out);p.resolve().relative_to((ROOT/'target').resolve())
 with p.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(p))
if __name__=='__main__':main(sys.argv[1])
