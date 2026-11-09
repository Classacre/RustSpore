"""Prepare a source-pinned nonroot host proposal; never compile or execute."""
import hashlib,json,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import anim_root_pose_sparse_carrier_v3 as carrier
from cell_schema_probe import SourceImage
from cell_native_math_probe import db
OUT=ROOT/'target/animation/nonroot-pose-host-packet-v1'
PACKET=ROOT/'target/animation/nonroot-source-fixture-v1'
ROT=ROOT/'target/animation/nonroot-rotation-native-run-v1/reply.bin'
COMPARE=ROOT/'target/animation/nonroot-rotation-rust-compare-v1/comparison.json'
SOURCE=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def w(b,o):return struct.unpack_from('<I',b,o)[0]
def main():
 assert sha(ROOT/'tools/anim_root_pose_sparse_carrier_v3.py')=='8384485a7318a2e8b9f7ee0207ffe695bcf850c0054e156acaccf8865a93540a'
 assert sha(carrier.v6.__file__)=='27370aa2b5b5147f72183b8797974b9c4135067fed86e89fb82ed52622d571b9'
 assert sha(ROOT/'tools/anim_root_pose_native_probe.py')=='212980f1b37a332b1f668d6f7f7a1a435d1fa07af20a814c5c62eb34dcf78603'
 assert sha(PACKET/'manifest.json')=='eeb9516a99f26b4ec167574c25ed811ba206e09151b092c4f90024519785a89a'
 assert sha(ROT)=='165fd1eee514a1e9d2a9fe601e006769667a5f3cc378eb701faeeb9c135625ce'
 assert sha(COMPARE)=='5d7b02ec19b612863d45369ba5887f1e96fd6afeebd5bcd829edee977b279173'
 compare=json.loads(COMPARE.read_text());assert compare['exact_for_compared_rows'] and compare['compared_rows']==1 and not compare['problems']
 m=json.loads((PACKET/'manifest.json').read_text())
 for name,expected in m['files'].items():assert sha(PACKET/name)==expected
 root=(PACKET/'root-definition-projection.bin').read_bytes();selected=(PACKET/'selected-definition-projection.bin').read_bytes();pos=(PACKET/'accepted-pos-reply.bin').read_bytes();rot=ROT.read_bytes()
 fields=[0x104,0x108,0x1f8,0x1fc,0x200,0x21c,0x208,0x10c,0x110,0x114,0x118,0x11c,0x120,0x124,0x128,0x12c,0x130,0x134,0x138,0x13c,0x140,0x144,0x148,0x14c,0x344,0x348,0x34c,0x350,0x354,0x358,0x35c,0x360,0x364,0x368,0x36c,0x378,0x37c,0x204,0x20c]
 def field(b,o):return w(b,168+4*fields.index(o))
 assert w(selected,4)==4 and w(root,4)==9
 selected_pos=[field(selected,x) for x in (0x10c,0x110,0x114)];selected_q=[field(selected,x) for x in (0x118,0x11c,0x120,0x124)];root_q=[field(root,x) for x in (0x118,0x11c,0x120,0x124)]
 pv=list(struct.unpack_from('<4I',pos,20));qv=list(struct.unpack_from('<5I',rot,100));one=0x3f800000
 words=[0x27f,0x71,0,0,one,one,one,one,one,one,4,one,*selected_pos,14,*selected_q,0,0,0,0,0,0,one,*pv,*qv]
 assert len(words)==36;request=struct.pack('<36I',*words)
 image=SourceImage(SOURCE);audit=carrier.v6.v3.source_audit(image)
 lines=carrier.host_source(image,audit).splitlines()
 def replace(old,new):
  assert lines.count(old)==1,(old,lines.count(old));i=lines.index(old);lines[i:i+1]=new
 # Root reference Definition differs from the selected node Definition.
 at=lines.index('mov esi,OFFSET InputBuffer+80');lines[at:at]=[f'mov dword ptr [RootReferenceBuffer+0{0x118+i*4:X}h],0{value:08X}h' for i,value in enumerate(root_q)]
 replace('and eax,0FFFFFFF7h',['cmp eax,4'])
 # Actual node ordinal 19 selects bitmap byte 4 and even/odd bits 40/80.
 replace('mov byte ptr [FrameBuffer+0813h],1',['mov byte ptr [FrameBuffer+0813h],040h'])
 replace('mov byte ptr [FrameBuffer+0812h],2',['mov byte ptr [FrameBuffer+0812h],080h'])
 replace('mov dword ptr [FrameBuffer+0838h],4',['mov dword ptr [FrameBuffer+0838h],4','mov dword ptr [FrameBuffer+0830h],4'])
 replace('cmp ecx,01B8h',['cmp ecx,01BCh'])
 replace('mov eax,dword ptr [OwnerBuffer+01B8h]',['mov eax,dword ptr [OwnerBuffer+01BCh]'])
 # Restrict this proposal to the exact source-bound request.
 at=lines.index('.code');lines[at:at]=db('FixtureInput',request)
 at=lines.index('ValidateRecord PROC')+1;lines[at:at]=['mov esi,OFFSET InputBuffer','mov edi,OFFSET FixtureInput','mov ecx,144','repe cmpsb','jne HostFailure']
 OUT.mkdir(parents=True,exist_ok=False);(OUT/'input.bin').write_bytes(request);(OUT/'host-x86.asm').write_text('\n'.join(lines)+'\n')
 report={'schema':'nonroot-pose-host-proposal-v1','compiled':False,'native_executed':False,'admitted_for_execution':False,'source_sha256':sha(SOURCE),'host_asm_sha256':sha(OUT/'host-x86.asm'),'input_sha256':sha(OUT/'input.bin'),'source_audit':audit,'original_spans':carrier.v6.CODE,'exit_stub':'99DBFE -> owned OriginalDone; reviewed carrier machinery, newly scoped proposal','excluded_calls':audit['unreachable_calls'],'globals':audit['globals'],'selected_pose':19,'bitmap_byte':4,'bitmap_bits':['40','80'],'selected_nonzero_mask':4,'selected_position_words':[f'{x:08X}' for x in selected_pos],'selected_quaternion_words':[f'{x:08X}' for x in selected_q],'root_quaternion_words':[f'{x:08X}' for x in root_q],'assumed':'unit owner scale and dimensions, zero prior pose, identity reference, effective blend 1, globals zero; +154 gate pre-applied and wrapper state explicitly supplied', 'limitations':'Partial 99D74F..DBFE, not full record traversal; no staging, actor, live selection, visible pose. This proposal requires independent source, linked and runner review; not executable.', 'pins':{str(p.relative_to(ROOT)):sha(p) for p in [Path(__file__),PACKET/'manifest.json',PACKET/'root-definition-projection.bin',PACKET/'selected-definition-projection.bin',PACKET/'accepted-pos-reply.bin',ROT,COMPARE,ROOT/'tools/anim_root_pose_sparse_carrier_v3.py',ROOT/'tools/anim_root_pose_native_probe_v6.py',ROOT/'tools/anim_root_pose_native_probe.py']}}
 (OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ('compiled','native_executed','admitted_for_execution','host_asm_sha256','input_sha256','original_spans','excluded_calls')},indent=2))
if __name__=='__main__':main()
