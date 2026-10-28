"""Independent static source/actual-linked audit; never run the native host."""
import argparse,hashlib,json,struct,sys
from pathlib import Path
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
from native_crt_acos_probe import decoder
from cell_schema_probe import SourceImage
import anim_cid_node_init_native_prepare_v2 as prep
import anim_cid_node_init_native_prepare_v1 as hostprep
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def audit(host,exe=None,mapping=None,carrier=None):
 source=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));asm=(host/'host-x86.asm').read_text();assert asm==hostprep.host(source)
 manifest=json.loads((host/'manifest.json').read_text());assert manifest['host_sha256']==sha(host/'host-x86.asm')
 # Derived independently from actual instructions, not the generator's MASK.
 writes=set(range(0,0x2c))|{0x38,0x54,0x55,0x2a0}|set(range(0x3c,0x54))|set(range(0x58,0xa4))|set(range(0x144,0x164))
 assert bytes(int(i in writes)for i in range(700))==hostprep.MASK
 sys.path.insert(0,str(prep.ROOT/'target/rea/capstone-runtime'))
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_MEM,X86_REG_EAX,X86_REG_ESP,X86_OP_IMM
 decoded=list(decoder().disasm(source.read(0x9c52f4,445),0x9c52f4));bound={i.address for i in decoded};actual=set()
 for i in decoded:
  assert not i.group(CS_GRP_CALL)
  if i.group(CS_GRP_JUMP):assert i.operands[0].imm in bound
  if i.mnemonic in ['mov','movss','fstp']and i.operands[0].type==X86_OP_MEM:
   op=i.operands[0];assert op.mem.base==X86_REG_EAX and not op.mem.index
   actual.update(range(op.mem.disp,op.mem.disp+op.size))
 assert actual==writes
 for t in ['mov dword ptr [FrameBuffer+1FCh],09C52F4h','cmp esp,OFFSET FrameBuffer+200h','mov esp,NativeStack','fnstcw SavedCW','fnstsw SavedSW','stmxcsr SavedCSR','call GuardAll','cmp byte ptr [WriteMask+edx],0','cmp al,0A5h','mov Poison,-1','movaps xmm7,xmm0','cmp dword ptr [esi+4],OFFSET CIDBuffer','div ebx']:
  assert t in asm,t
 r=dict(schema='CID-node-init-independent-static-audit-v2',static_pass=True,native_executed=False,full_function=False,source_bytes=445,source_node_write_bytes=len(writes),all700_unwritten_bytes_guarded=True,source_loop_no_calls=True,stack_frame_exact_and_unchanged=True,actual_factory_scale_one_only=True,float_order='quaternionFLD/FSTP,positionMULSS,historyFLD/FSTP',canonical_pointers_only_after_source=True,pins={str(p):sha(p)for p in[Path(__file__),Path(prep.__file__),Path(hostprep.__file__),host/'host-x86.asm',host/'manifest.json']})
 if not exe:return r
 assert mapping and carrier;old=Image(exe);new=Image(carrier);symbols=syms(mapping)
 assert old.base==0x29000000 and new.base==0x670000
 # Reproduction closes every RVA/import edit, then independent checks inspect
 # complete original spans and actual section/import/owned instruction ranges.
 rebuilt,review=prep.build(exe,mapping,source);assert rebuilt==Path(carrier).read_bytes()
 ni,edits=new.imports();oi,oldedits=old.imports();assert ni==oi and edits==oldedits
 assert set(ni.values())=={'GetStdHandle','ReadFile','WriteFile','ExitProcess','GetModuleHandleA','GetLastError'}
 sections=sorted(new.sections,key=lambda z:z['va'])
 for x,y in zip(sections,sections[1:]):assert x['va']+x['vs']<=y['va']
 assert all(not(z['flags']&0x20000000 and z['flags']&0x80000000)for z in sections)
 assert all(b<=0x670000 or a>=0x671000 for a,b in [(0x9c52f4,0x9c54b1),(0x9c54b1,0x9c54b8),(0x14861d0,0x14861d4)])
 code=bytearray(b"\xcc"*65536);code[0x52f4:0x54b1]=source.read(0x9c52f4,445);code[0x54b1:0x54b8]=b"\xb8"+struct.pack("<I",symbols["OriginalDone"])+b"\xff\xe0"
 data=bytearray(65536);data[0x61d0:0x61d4]=source.read(0x14861d0,4)
 assert new.read(0x671000,0xf000)==bytes(0xf000) and new.perm(0x671000,0xf000)==0x40000000
 assert new.read(0x9c0000,65536)==code and new.perm(0x9c0000,65536)==0x60000000
 assert new.read(0x1480000,65536)==data and new.perm(0x1480000,65536)==0x40000000
 assert new.read(0x9c52f4,445)==source.read(0x9c52f4,445) and new.perm(0x9c52f4,445)==0x60000000
 assert new.read(0x14861d0,4)==bytes.fromhex('0000803f') and new.perm(0x14861d0,4)==0x40000000
 assert new.read(0x9c54b1,7)==b'\xb8'+struct.pack('<I',symbols['OriginalDone'])+b'\xff\xe0'
 start,end=symbols['OwnedStart'],symbols['OwnedEnd'];assert new.read(start,end-start)==old.read(start,end-start)
 rows=list(decoder().disasm(new.read(start,end-start),start));assert sum(i.size for i in rows)==end-start;boundaries={i.address for i in rows}
 for i in rows:
  if not(i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP)):continue
  assert i.operands[0].type==X86_OP_IMM;t=i.operands[0].imm
  if t in boundaries:continue
  assert i.mnemonic=='call';raw=new.read(t,6);assert raw[:2]==b'\xff\x25'and struct.unpack('<I',raw[2:])[0]in ni
 entries=[rows[j-1].operands[0].imm for j,i in enumerate(rows)if i.mnemonic=='ret'and j and rows[j-1].mnemonic=='push'and rows[j-1].operands[0].type==X86_OP_IMM]
 assert entries==[0x9c52f4]
 assert new.read(symbols['Data_014861D0'],4)==source.read(0x14861d0,4) and new.perm(symbols['Data_014861D0'],4)==0x40000000
 assert new.read(symbols['WriteMask'],700)==hostprep.MASK and new.perm(symbols['WriteMask'],700)==0x40000000
 for name,size in hostprep.SIZES.items():
  at=symbols[name+'Buffer'];assert symbols[name+'Before']==at-16 and symbols[name+'After']==at+size and new.perm(at-16,size+32)==0xc0000000
  assert new.perm(symbols[name+'Shadow'],size)==0xc0000000
 assert new.read(symbols['Original_009C52F4'],445)==source.read(0x9c52f4,445) and new.perm(symbols['Original_009C52F4'],445)==0x40000000
 r.update(actual_linked=True,carrier_sha256=sha(carrier),owned_sha256=sha(exe),map_sha256=sha(mapping),source_pages_exact=True,owned_instruction_bytes_preserved=True,no_WX=True,section_nonoverlap=True,imports_closed=True,all_buffers_guarded_and_RW=True,original_entry_only='009C52F4',exit_stub_exact=True)
 for p in[exe,mapping,carrier]:r['pins'][str(p)]=sha(p)
 return r
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--owned-exe',type=Path);p.add_argument('--owned-map',type=Path);p.add_argument('--carrier',type=Path);a=p.parse_args();a.out.resolve().relative_to((prep.ROOT/'target').resolve());r=audit(a.host_dir,a.owned_exe,a.owned_map,a.carrier)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(sha(a.out))
if __name__=='__main__':main()
