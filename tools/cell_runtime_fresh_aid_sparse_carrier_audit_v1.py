"""Independent linked fresh-aid carrier audit; no native execution."""
import argparse,json,struct
from pathlib import Path
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms,sha,u32,roundup,IMPORTS
from cell_schema_probe import SourceImage
from native_crt_acos_probe import decoder
import cell_runtime_fresh_aid_native_probe as source
BASE=0x7c0000;OLD=0x27000000

def audit(a):
 new,old=Image(a.carrier),Image(a.owned_exe);s=syms(a.owned_map);image=SourceImage(a.source_exe);source.audit_source(image)
 assert new.base==BASE and old.base==OLD and new.base+u32(new.b,new.op+16)==s['HostMain']
 assert struct.unpack_from('<H',new.b,new.nt+22)[0]&1 and not struct.unpack_from('<H',new.b,new.op+70)[0]&0x40
 assert new.b[:new.nt]==old.b[:old.nt] and u32(new.b,new.op+64)==0
 assert all((r,n)==(0,0) for i,(r,n) in enumerate(new.dirs()) if i not in (1,12))
 ni,ne=new.imports();oi,oe=old.imports();assert ni==oi and ne==oe and set(ni.values())==IMPORTS and len(ni)==6
 for i in (1,12):
  r,n=old.dirs()[i];assert new.dirs()[i]==(r+OLD-BASE,n)
 high=[z for z in new.sections if z['va']>=OLD and z['rs']];assert len(high)==len(old.sections)
 for x,y in zip(old.sections,high):
  assert all(x[k]==y[k] for k in ('name','va','vs','rs','flags'));expected=bytearray(old.read(x['va'],x['vs']))
  for at in oe:
   if x['va']<=at<x['va']+x['vs']:
    j=at-x['va'];struct.pack_into('<I',expected,j,u32(expected,j)+OLD-BASE)
  assert new.read(y['va'],y['vs'])==expected and new.b[y['raw']+y['vs']:y['raw']+y['rs']]==old.b[x['raw']+x['vs']:x['raw']+x['rs']]
 assert all(not(s['OwnedStart']<=at<s['OwnedEnd']) for at in oe)
 # Independent payload reconstruction: padding, two approved code pages, ONE, mutable serial page.
 pages={0x7c1000:(bytearray(0xf000),0x40000040),0x7d0000:(bytearray(b'\xcc'*65536),0x60000020),0x9a0000:(bytearray(b'\xcc'*65536),0x60000020),0x1480000:(bytearray(65536),0x40000040),0x1550000:(bytearray(65536),0xc0000040)}
 for lo,hi in source.CODE:pages[lo&~65535][0][lo&65535:(lo&65535)+hi-lo]=image.read(lo,hi-lo)
 pages[0x1480000][0][0x61d0:0x61d4]=image.read(0x14861d0,4)
 low=[z for z in new.sections if z['va']<OLD and z['rs']];assert len(low)==5
 for z in low:
  expected,flags=pages[z['va']];assert z['vs']==len(expected) and z['flags']==flags and new.read(z['va'],z['vs'])==expected
 assert all(z['flags']==0x80 and z['raw']==0 for z in new.sections if not z['rs'])
 ext=[]
 for name,n in source.BUFFERS:
  at=s[name+'Buffer'];assert s[name+'Before']==at-16 and s[name+'After']==at+n and new.read(at-16,16)==new.read(at+n,16)==b'\xa5'*16
  for lo,size in [(at-16,n+32)]+([] if name=='Reply' else [(s[name+'Shadow'],n)]):
   assert new.perm(lo,size)==0xc0000000 and all(lo+size<=x or y<=lo for x,y in ext);ext.append((lo,lo+size))
 assert new.perm(s['SerialShadow'],65536)==0xc0000000 and all(s['SerialShadow']+65536<=x or y<=s['SerialShadow'] for x,y in ext)
 rows=list(decoder().disasm(new.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']),s['OwnedStart']));bounds={r.address for r in rows};assert sum(r.size for r in rows)==s['OwnedEnd']-s['OwnedStart'];by={r.address:i for i,r in enumerate(rows)}
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM
 calls=[]
 for r in rows:
  if not(r.group(CS_GRP_CALL) or r.group(CS_GRP_JUMP)):continue
  assert r.operands[0].type==X86_OP_IMM;t=r.operands[0].imm
  if t in bounds:continue
  assert r.mnemonic=='call';thunk=new.read(t,6);assert thunk[:2]==b'\xff\x25';calls.append((r.address,ni[u32(thunk,2)]))
 entries=[rows[i-1].operands[0].imm for i,r in enumerate(rows) if r.mnemonic=='ret' and i and rows[i-1].mnemonic=='push' and rows[i-1].operands[0].type==X86_OP_IMM];assert entries==[0x9a33c0,0x9a1690]
 module=next(x for x,n in calls if n=='GetModuleHandleA');read=next(x for x,n in calls if n=='ReadFile');j=by[module];assert rows[j+1].mnemonic=='cmp' and rows[j+1].operands[1].imm==BASE and rows[j+2].operands[0].imm==s['HostFailure'];assert module<read
 assert sum(1 for r in rows if module<r.address<read and r.mnemonic=='repe cmpsb')==7
 fail=rows[by[s['HostFailure']]:];assert [r.mnemonic for r in fail[:5]]==['cmp','je','mov','push','call'] and fail[2].op_str.startswith('esp,')
 # Per-record program exact comparison: carrier modifies startup/import declarations only.
 import cell_runtime_fresh_aid_sparse_carrier_v1 as builder
 original=source.host_source(image);actual=(a.owned_map.parent/'host-x86.asm').read_text();assert actual==builder.host_source(image)
 assert original[original.index('RunOnce PROC'):original.index('RunOnce ENDP')]==actual[actual.index('RunOnce PROC'):actual.index('RunOnce ENDP')]
 return dict(schema='fresh-aid-sparse-carrier-independent-linked-audit-v1',static_pass=True,native_executed=False,source_bytes=1049,source_byte_exact=True,owned_instruction_byte_exact=True,import_metadata_only_changes=True,section_virtual_ranges_adjacent=True,owned_padding_readonly_zero=True,low_payload_and_slack_exact=True,serial_page_owned_mutable_zero_initial=True,per_record_program_unchanged=True,guards_closed=True,owned_control_closed=True,failure_real_stack_closed=True,carrier_sha256=sha(a.carrier),owned_sha256=sha(a.owned_exe),map_sha256=sha(a.owned_map),virtual_image_size=new.image,imports=sorted(IMPORTS),limits='Static only; full fresh ctor/reset exclusions and original frozen inputs/models remain. Seven excluded calls are source-unreachable; no installed-seven-stubs claim. No native acceptance.')
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for n in ('carrier','owned-exe','owned-map','out'):p.add_argument('--'+n,type=Path,required=True)
 p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));a=p.parse_args();r=audit(a)
 with a.out.open('x') as f:json.dump(r,f,indent=2);f.write('\n')
 print(sha(a.out))
if __name__=='__main__':main()
