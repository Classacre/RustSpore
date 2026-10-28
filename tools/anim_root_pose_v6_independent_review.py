"""Independent read-only posev6 source/entry/guard review. Never executes host."""
import hashlib,json,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path[:0]=[str(ROOT/'tools'),str(ROOT/'target/rea/capstone-runtime')]
import anim_root_pose_native_probe_v6 as tool
from cell_schema_probe import SourceImage
from native_crt_acos_probe import PE,decoder
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 prep=ROOT/'target/animation/root-pose-conversion-host-v6';host=ROOT/'target/animation/root-pose-conversion-host-root-reproduction-v3';ledger_path=prep/'branch-register-ledger-v1.json';ledger=json.loads(ledger_path.read_text());manifest=json.loads((prep/'manifest.json').read_text());image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
 for path,digest in ledger['pins'].items():assert sha(ROOT/path)==digest,path
 assert sha(host/'host-x86.exe')=='e14178873148a7dd6a66b28c8554804be51ed6c5c1af5dab2704271caf03bb0e'
 audit=tool.v3.source_audit(image);assert audit==manifest['source_audit'];source=tool.host_source(image,audit);assert source==(prep/'host-x86.asm').read_text()==(host/'host-x86.asm').read_text()
 linked=tool.linked_audit(host/'host-x86.exe',host/'host-x86.map',image,audit);assert linked['source_and_data_byte_exact'] and linked['sections_checked'] and linked['guarded_buffers_shadows_nonoverlapping'] and linked['owned_failure_restores_saved_stack']
 assert linked['owned_code_sha256']=='546d598faaf860ee50278c04f5544439aade5589c7364c71f8bf08564c239866'
 rows={}
 for part in audit['code']:
  a=int(part['start'],16);b=int(part['end_exclusive'],16);decoded=list(decoder().disasm(image.read(a,b-a),a));assert sum(i.size for i in decoded)==b-a;rows.update({i.address:i for i in decoded})
 assert not any(i.mnemonic in ['std','popfd','iret','fldcw','ldmxcsr'] for i in rows.values())
 expected_gates={0x9b0d38:('test','eax, 0x200'),0x9b0d3d:('je','0x9b0dd4'),0x9b1290:('test','al, 1'),0x9b1292:('je','0x9b17a7'),0x9b1856:('test','al, 1'),0x9b1858:('je','0x9b1a15'),0x9b1ace:('test','dword ptr [eax], 0x200'),0x9b1ad4:('je','0x9b1b21'),0x99d803:('cmp','dword ptr [0x1551a74], 0'),0x99d80a:('je','0x99d84f'),0x99d918:('test','byte ptr [edi + 0x10], 1'),0x99d91c:('je','0x99d99f')}
 for at,pair in expected_gates.items():assert (rows[at].mnemonic,rows[at].op_str)==pair,(hex(at),rows[at].op_str)
 assert rows[0x9b1209].op_str=='edi, dword ptr [esp + 0xdc]'
 assert len(audit['unreachable_calls'])==len(ledger['unreachable_calls'])==9
 #13cdecl args are pushed in reverse order; one full ordinary conversion CALL
 #then a root-index0 partial entry with explicitly seeded registers/stack.
 for text in ['push dword ptr [InputBuffer+12]','push OFFSET ConvertedBuffer+12','push OFFSET ConvertedBuffer','push OFFSET ReferenceRotationBuffer','push OFFSET ReferencePositionBuffer','push OFFSET PoseBuffer','push OFFSET OwnerBuffer','push OFFSET WrapperBuffer+068h','push OFFSET RotationDescriptorBuffer','push OFFSET WrapperBuffer+018h','push OFFSET PositionDescriptorBuffer','push OFFSET SecondaryPositionBuffer','push OFFSET ChannelBuffer+088h','add esp,52','mov esi,OFFSET PoseBuffer','mov edi,OFFSET WrapperBuffer','mov ebp,OFFSET OwnerBuffer','mov ebx,2','mov eax,0099D74Fh','mov dword ptr [FrameBuffer+02E0Ch],OFFSET OwnerBuffer','mov byte ptr [FrameBuffer+0813h],1','mov byte ptr [FrameBuffer+0812h],2','cmp esp,OFFSET FrameBuffer+0800h','mov esp,SavedStack']:
  assert text in source,text
 assert rows[0x5992cc].mnemonic=='ret' and rows[0x5992cc].op_str=='4'
 assert rows[0x9b1b81].mnemonic=='ret' and rows[0x9b1b81].op_str==''
 inputs=ROOT/'target/animation/root-pose-conversion-inputs-v1';input_manifest=json.loads((inputs/'manifest.json').read_text());raw=(inputs/'inputs.bin').read_bytes();assert len(raw)==22212*144
 for name,digest in input_manifest['files'].items():assert sha(inputs/name)==digest
 controls={};finite_rows=0
 for at in range(0,len(raw),144):
  words=struct.unpack_from('<36I',raw,at);cw,pos,chan,mirror=words[:4]
  assert cw&0xfffff0ff==0x7f and cw&0x300!=0x100 and pos in [0x31,0x71,0xb1] and chan in [0,0x10] and mirror in [0,1]
  assert words[10] in [0,8] and words[15]&0xff000000==0 and words[15]&6==6
  assert words[4]&0x80000000==0 and words[4]&0x7fffffff!=0
  for i in list(range(4,10))+[i for i in range(11,36) if i!=15]:assert words[i]&0x7f800000!=0x7f800000
  controls[f'{cw:04X}']=controls.get(f'{cw:04X}',0)+1;finite_rows+=1
 pins={str(p.relative_to(ROOT)):sha(p) for p in [Path(tool.__file__),ROOT/'tools/anim_root_pose_native_probe.py',prep/'host-x86.asm',prep/'manifest.json',ledger_path,host/'host-x86.exe',host/'linked-audit.json',inputs/'manifest.json',inputs/'inputs.bin']}
 report=dict(schema='independent-pose-v6-peer-review-v1',review_result='STATIC PASS for declared partial-entry/root0/nonrelative coordinate+accumulation fixture; runtime/numerical acceptance separate',no_blocking_static_issue_found=True,original_spans_exact=True,unreachable_calls_closed=True,partial_entry_frame_registers_closed=True,guarded_memory_bounds_closed=True,host_executed=False,rust_executed=False,pins=pins,source_bytes=sum(p['bytes'] for p in audit['code']),source_spans=len(audit['code']),finite_positive_blend_requests=finite_rows,input_controls=controls,source_gate_proof=['POS200clear skips both orientation99BE70 sites','CHANbit0clear skips relativePOS/ROT block families covering7 excluded callees','debug1551A74zero skips99D821; Wrapper10zero skips99D949/99D997','Rootordinal0,Frame2E0Cowner restore,17guarded buffers and published frame/pose/owner write windows match owned setup','5992B0thiscall RET4 and other ordinaryRET helpers match caller cleanup; fakeESP failures restore saved real stack before ExitProcess; mapping failures stay pre-input'],limits=['No original host executed by reviewer; static author linked-audit re-evaluated plus independent source gates/domain/entries checked','Raw finite input fields and positiveblend only; output finite/numeric equivalence belongs to subsequent native/Rust gate','Three named reference contexts are explicit fixtures, not source bodies/actors','No staged evaluation,children,constraints,finalcommit or visible animation claim'])
 out=Path(output);out.resolve().relative_to((ROOT/'target').resolve());assert not out.exists();out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['pins','limits','source_gate_proof']}))
if __name__=='__main__':main(sys.argv[1])
