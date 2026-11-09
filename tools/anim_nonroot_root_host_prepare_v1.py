"""Prepare uncompiled source-body root + 99CB70 proposal, never execute."""
import hashlib,json,struct,sys
from pathlib import Path
R=Path(__file__).resolve().parent.parent;sys.path.insert(0,str(R/'tools'))
import anim_root_pose_sparse_carrier_v3 as carrier
from cell_native_math_probe import db
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
OUT=R/'target/animation/nonroot-root-host-packet-v1'
FIX=R/'target/animation/nonroot-root-reference-preparation-v1'
SOURCE=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(FIX/'pending-root-input.bin')=='bd1d0f0180699b4443e709515d76d5e6d3181e72df2b2bf96b601a9b52187ad9'
 assert sha(R/'tools/anim_root_pose_sparse_carrier_v3.py')=='8384485a7318a2e8b9f7ee0207ffe695bcf850c0054e156acaccf8865a93540a'
 image=SourceImage(SOURCE);audit=carrier.v6.v3.source_audit(image);root=carrier.host_source(image,audit);lines=root.splitlines()
 request=(FIX/'pending-root-input.bin').read_bytes();assert len(request)==144 and struct.unpack_from('<I',request,40)[0]==9
 start,end=0x99cb70,0x99cc74;raw=image.read(start,end-start);inst=list(decoder().disasm(raw,start));assert sum(x.size for x in inst)==len(raw)
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM
 boundaries={x.address for x in inst}
 for x in inst:
  if x.group(CS_GRP_CALL) or x.group(CS_GRP_JUMP):
   assert x.operands[0].type==X86_OP_IMM and x.operands[0].imm in boundaries
  for op in x.operands:
   if op.type==X86_OP_MEM and not op.mem.base:assert op.mem.index==0 and op.mem.disp==0x14861d0
 def one(old,new):
  assert lines.count(old)==1,(old,lines.count(old));i=lines.index(old);lines[i:i+1]=new
 # Restrict input to reviewed packet and admit literal prepared mask9.
 i=lines.index('.code');lines[i:i]=db('FixtureInput',request)+db('Original_0099CB70',raw)
 i=lines.index('ValidateRecord PROC')+1;lines[i:i]=['mov esi,OFFSET InputBuffer','mov edi,OFFSET FixtureInput','mov ecx,144','repe cmpsb','jne HostFailure']
 one('and eax,0FFFFFFF7h',['cmp eax,9'])
 # The sparse image must map these bytes at their original VA. Check on startup.
 i=lines.index('NextRecord:');lines[i:i]=['mov esi,OFFSET Original_0099CB70','mov edi,0099CB70h',f'mov ecx,{len(raw)}','repe cmpsb','jne HostFailure']
 one('ReplyBuffer BYTE 176 DUP(0)',['ReplyBuffer BYTE 212 DUP(0)'])
 one('Expected BYTE 156 DUP(0)',['Expected BYTE 192 DUP(0)'])
 i=lines.index('mov edi,OFFSET ReplyBuffer');assert lines[i+1]=='mov ecx,44';lines[i+1]='mov ecx,53'
 i=lines.index('OriginalDone::');assert lines[i+1:i+4]==['cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure','mov esp,SavedStack']
 lines[i+3:i+3]=['mov esi,OFFSET FrameBuffer+0A28h','mov edi,OFFSET ReplyBuffer+176','mov ecx,9','rep movsd','mov ecx,OFFSET FrameBuffer+0A28h','mov eax,0099CB70h','call eax','cmp esp,OFFSET FrameBuffer+0800h','jne HostFailure']
 # Compare pre-normalization cache independently in both poison passes.
 i=lines.index('mov edi,OFFSET Expected');assert lines[i+1:i+3]==['mov ecx,156','rep movsb'];lines[i+3:i+3]=['mov esi,OFFSET ReplyBuffer+176','mov edi,OFFSET Expected+156','mov ecx,36','rep movsb']
 i=lines.index('repe cmpsb',lines.index('mov edi,OFFSET Expected',i+4));assert lines[i+1]=='jne HostFailure';lines[i+2:i+2]=['mov esi,OFFSET ReplyBuffer+176','mov edi,OFFSET Expected+156','mov ecx,36','repe cmpsb','jne HostFailure']
 one('mov Remaining,176',['mov Remaining,212'])
 OUT.mkdir(parents=True,exist_ok=False);(OUT/'input.bin').write_bytes(request);(OUT/'host-x86.asm').write_text('\n'.join(lines)+'\n')
 record={'start':f'{start:08X}','end_exclusive':f'{end:08X}','bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'calls':[],'branches':[{'at':f'{x.address:08X}','target':f'{x.operands[0].imm:08X}'} for x in inst if x.group(CS_GRP_JUMP)],'absolute_memory':'014861D0 only','disassembly':[f'{x.address:08X}: {x.bytes.hex()} {x.mnemonic} {x.op_str}' for x in inst]}
 report={'schema':'nonroot-source-root-host-proposal-v1','compiled':False,'native_executed':False,'admitted_for_execution':False,'asm_sha256':sha(OUT/'host-x86.asm'),'input_sha256':sha(OUT/'input.bin'),'source_sha256':sha(SOURCE),'source_audit':audit,'additional_original_span':record,'exit_stub':'99DBFE to owned OriginalDone, then source 99CB70 call','excluded_calls':audit['unreachable_calls'],'globals':audit['globals'],'reply_bytes':212,'reply_layout':'0..155 converted/pose/normalized cache/bitmap; 156..163 FP state; 164..175 reserved; 176..211 pre-normalization root cache','assumptions':'only pass0 root row0 in zero cache; unit blend/scale/dimensions, root source +154=9, root initial reference is source root rest, globals zero, binding +10 stable zero; earlier retained sample is the same source clip frame 0','not_established':'full 99D190 traversal, actual bind/actor/clock, source root relative changes from prior frames, whole child chain, live animation','builder':'old sparse-carrier CLI pins original assembly; new builder must add exact 99CB70 source span and audit revised host and pages. This preparation does not provide such builder.','pins':{str(p.relative_to(R)):sha(p) for p in [Path(__file__),FIX/'manifest.json',FIX/'pending-root-input.bin',R/'tools/anim_root_pose_sparse_carrier_v3.py',R/'tools/anim_root_pose_native_probe_v6.py',R/'tools/anim_root_pose_native_probe.py']}}
 (OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ('compiled','native_executed','admitted_for_execution','asm_sha256','input_sha256','additional_original_span')},indent=2))
if __name__=='__main__':main()
