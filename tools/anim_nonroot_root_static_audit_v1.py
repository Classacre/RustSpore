"""Static pinned instruction/stack checks; no execution, compilation or writes outside target."""
import hashlib,json,sys
from pathlib import Path
R=Path(__file__).resolve().parent.parent;sys.path.insert(0,str(R/'tools'))
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
SRC=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
P=R/'target/animation/nonroot-root-host-packet-v2-final'; OUT=R/'target/animation/nonroot-root-static-audit-v1'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 assert sha(SRC)=='17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
 assert sha(P/'host-x86.asm')=='5cba9971b2290a7a17002740fe952a9f500f541603e111ba6a7ee04aa8351a23'
 assert sha(P/'input.bin')=='bd1d0f0180699b4443e709515d76d5e6d3181e72df2b2bf96b601a9b52187ad9'
 im=SourceImage(SRC);engine=decoder()
 from capstone import CS_GRP_CALL
 spans=[(0x9b0ca0,0x9b1b82),(0x9b0b90,0x9b0c91),(0x99bd80,0x99be6b),(0x99d74f,0x99dbfe),(0x99c8d0,0x99c993),(0x5992b0,0x5992cf),(0x99cb70,0x99cc74)]
 decoded={};hashes=[]
 for a,b in spans:
  raw=im.read(a,b-a);items=list(engine.disasm(raw,a));assert sum(x.size for x in items)==b-a
  for x in items:decoded[x.address]=(x.mnemonic,x.op_str)
  hashes.append({'start':f'{a:08X}','end_exclusive':f'{b:08X}','sha256':hashlib.sha256(raw).hexdigest()})
 checks={0x9b0d38:('test','eax, 0x200'),0x9b0d3d:('je','0x9b0dd4'),0x9b0dd4:('and','eax, 0x120'),0x9b0dd9:('cmp','eax, 0x100'),0x9b0dde:('jne','0x9b0f85'),0x9b1290:('test','al, 1'),0x9b1292:('je','0x9b17a7'),0x9b1856:('test','al, 1'),0x9b1858:('je','0x9b1a15'),0x9b1ace:('test','dword ptr [eax], 0x200'),0x9b1ad4:('je','0x9b1b21'),0x99d803:('cmp','dword ptr [0x1551a74], 0'),0x99d80a:('je','0x99d84f'),0x99d918:('test','byte ptr [edi + 0x10], 1'),0x99d91c:('je','0x99d99f'),0x9b0ca0:('sub','esp, 0xc8'),0x9b17b0:('call','0x9b0b90'),0x9b1a85:('call','0x99bd80'),0x99cb80:('sub','esp, 0x14')}
 for addr,want in checks.items():assert decoded[addr]==want,(hex(addr),decoded.get(addr),want)
 # All reachable leaf functions named below have no stack push/pop/call or ESP arithmetic.
 for a,b in [(0x9b0b90,0x9b0c91),(0x99bd80,0x99be6b),(0x99c8d0,0x99c993),(0x5992b0,0x5992cf)]:
  for i in engine.disasm(im.read(a,b-a),a):
   assert not i.group(CS_GRP_CALL) and i.mnemonic not in ('push','pop','enter','leave')
   assert not (i.mnemonic in ('add','sub','mov','and') and i.op_str.startswith('esp,'))
 host=(P/'host-x86.asm').read_text();assert 'mov dword ptr [PositionDescriptorBuffer],eax' in host and 'mov dword ptr [RotationDescriptorBuffer],032h' in host
 assert 'mov dword ptr [ChannelBuffer+088h],eax' in host and 'mov esp,OFFSET FrameBuffer+0800h' in host
 result={'schema':'nonroot-root-static-audit-v1','native_executed':False,'compiled':False,'host_asm_sha256':sha(P/'host-x86.asm'),'input_sha256':sha(P/'input.bin'),'source_spans':hashes,'checked_instructions':{f'{a:08X}':list(v) for a,v in checks.items()},
 'conditional_branch_summary':'POS 71 avoids 200 and 100 branches; CHAN word 0 skips relative POS and ROT sections; global 1551A74=0 and wrapper +10 bit0=0 skip remaining calls. Rotation flags 32 reach original 99BD80. Mask 9 is literal but 9B0B90 takes POS71 0x40 branch, not mask-dependent B1.',
 'stack_bound':{'frame_stack_start':'0800','conversion_owned_arguments':52,'conversion_return':4,'conversion_allocation':200,'conversion_saved_registers':16,'maximum_reachable_helper_arguments':20,'helper_return':4,'lowest_offset_hex':'06D8','calculation_bytes':296,'scope':'Assumes listed fixture branches and balanced source call sites; leaf no-stack checks performed. 99CB70 uses 24 bytes including return; accumulation helpers use less than conversion path.'},
 'not_proved':'No automated whole-CFG symbolic execution, external peer review, linked image, revised dedicated builder or runtime; review register/stack provenance and partial 99D entry separately.',
 'pins':{str(p.relative_to(R)):sha(p) for p in [Path(__file__),P/'host-x86.asm',P/'input.bin',P/'manifest.json']}}
 OUT.mkdir(parents=True,exist_ok=False);(OUT/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
