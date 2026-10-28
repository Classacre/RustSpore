"""Independent actuallinked low-sparse/high-CRT hybrid audit; never execute."""
import argparse,json,struct
from pathlib import Path
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms,sha,u32
from native_crt_acos_probe import decoder
import cell_runtime_fresh_loop_native_probe as source
BASE=source.BASE;OLD=source.OWNED_BASE

def audit(a):
 new,old=Image(a.carrier),Image(a.owned_exe);s=syms(a.owned_map);images=source.arithmetic.Images(a.source_exe,a.source_crt);proof=source.audit_source(images);owned=source.linked_audit(a.owned_exe,a.owned_map,images)
 assert new.base==BASE and old.base==OLD and new.base+u32(new.b,new.op+16)==s['HostMain'];assert new.b[:new.nt]==old.b[:old.nt]
 assert struct.unpack_from('<H',new.b,new.nt+22)[0]&1 and not struct.unpack_from('<H',new.b,new.op+70)[0]&0x40 and u32(new.b,new.op+64)==0
 assert all((r,n)==(0,0)for i,(r,n)in enumerate(new.dirs())if i not in[1,12]);ni,ne=new.imports();oi,oe=old.imports();assert ni==oi and ne==oe and len(ni)==9 and set(ni.values())==source.IMPORTS
 for i in[1,12]:r,n=old.dirs()[i];assert new.dirs()[i]==(r+OLD-BASE,n)
 high=[z for z in new.sections if z['va']>=OLD and z['rs']];assert len(high)==len(old.sections)
 for x,y in zip(old.sections,high):
  assert all(x[k]==y[k]for k in['name','va','vs','rs','flags']);raw=bytearray(old.read(x['va'],x['vs']))
  for at in oe:
   if x['va']<=at<x['va']+x['vs']:j=at-x['va'];struct.pack_into('<I',raw,j,u32(raw,j)+OLD-BASE)
  assert new.read(y['va'],y['vs'])==raw and new.b[y['raw']+y['vs']:y['raw']+y['rs']]==old.b[x['raw']+x['vs']:x['raw']+x['rs']]
 assert all(not(s['OwnedStart']<=at<s['OwnedEnd'])for at in oe)
 pages={at:bytearray(b'\xcc'*65536 if f&0x20000000 else bytes(65536))for at,f in source.LOW_PAGES.items()}
 for lo,hi in source.APP_CODE:pages[lo&~65535][lo&65535:(lo&65535)+hi-lo]=images.read(lo,hi-lo)
 for lo,n in source.DATA+[source.ONE]:
  if lo<0x78000000:pages[lo&~65535][lo&65535:(lo&65535)+n]=images.read(lo,n)
 struct.pack_into('<I',pages[0x13C0000],0xC4C0,s['ModfProvider'])
 for at,label in source.EXITS.items():
  if at<0x78000000:pages[at&~65535][at&65535:(at&65535)+7]=b'\xb8'+struct.pack('<I',s[label])+b'\xff\xe0'
 expected={BASE+0x1000:(bytes(0xF000),0x40000040)};expected.update({at:(bytes(raw),source.LOW_PAGES[at])for at,raw in pages.items()});low=[z for z in new.sections if z['va']<OLD and z['rs']];assert len(low)==len(expected)
 for z in low:raw,flags=expected[z['va']];assert z['vs']==len(raw)and z['flags']==flags and new.read(z['va'],z['vs'])==raw
 assert all(z['flags']==0x80 and z['raw']==0 for z in new.sections if not z['rs']);assert not any(z['flags']&0x20000000 and z['flags']&0x80000000 for z in new.sections);assert max(z['va']+z['vs']for z in new.sections)<0x78000000
 rows=list(decoder().disasm(new.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']),s['OwnedStart']));by={r.address:i for i,r in enumerate(rows)};assert sum(r.size for r in rows)==s['OwnedEnd']-s['OwnedStart']
 from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM
 calls=[]
 for r in rows:
  if not(r.group(CS_GRP_CALL)or r.group(CS_GRP_JUMP)):continue
  assert r.operands[0].type==X86_OP_IMM;t=r.operands[0].imm
  if t in by:continue
  assert r.mnemonic=='call';raw=new.read(t,6);assert raw[:2]==b'\xff\x25';calls.append((r.address,ni[u32(raw,2)]))
 startup=s['HostMain'];read=next(r.address for r in rows if r.address>=startup and r.mnemonic=='call'and r.operands[0].type==X86_OP_IMM and r.operands[0].imm==s['ReadRecord']);before=[(at,name)for at,name in calls if startup<=at<read];assert sum(name=='VirtualAlloc'for at,name in before)==3 and sum(name=='VirtualProtect'for at,name in before)==3 and sum(name=='FlushInstructionCache'for at,name in before)==1
 allocations=[]
 for at,name in before:
  if name=='VirtualAlloc':
   j=by[at];assert rows[j-1].mnemonic=='push'and rows[j-1].operands[0].type==X86_OP_IMM;allocations.append(rows[j-1].operands[0].imm)
 assert allocations==list(source.HIGH_PAGES);module=next(at for at,name in before if name=='GetModuleHandleA');j=by[module];assert rows[j+1].mnemonic=='cmp'and rows[j+1].operands[1].imm==BASE and rows[j+2].operands[0].imm==s['HostFailure']
 assert sum(r.mnemonic=='repe cmpsb'for r in rows if module<r.address<read)==len(source.CODE)
 entries=[rows[j-1].operands[0].imm for j,r in enumerate(rows)if r.mnemonic=='ret'and j and rows[j-1].mnemonic=='push'and rows[j-1].operands[0].type==X86_OP_IMM];assert sorted(entries)==sorted([0x785A3350,0x9A33C0,0x9A1690,0x9A3198,0x9A32D4,0xA00310,0x9A2BE0,0x9A1CB0,0x9A2BB0])
 fail=rows[by[s['HostFailure']]:];assert [r.mnemonic for r in fail[:5]]==['cmp','je','mov','push','call']and fail[2].op_str.startswith('esp,')
 asm=(a.owned_map.parent/'host-x86.asm').read_text();assert asm==source.host_source(images)
 assert 'cmp ProviderCalls,9' in asm and f'push 0{0x9A2BE0:X}h\nret' in asm
 for name,n in source.BUFFERS:
  at=s[name+'Buffer'];assert s[name+'Before']==at-16 and s[name+'After']==at+n and new.perm(at-16,n+32)==0xc0000000
  if name!='Reply':assert new.perm(s[name+'Shadow'],n)==0xc0000000
 for name in['SerialShadow','GenerationShadow']:assert new.perm(s[name],65536)==0xc0000000
 for lo in[0x1550000,0x1660000]:assert new.perm(lo,65536)==0xc0000000 and new.read(lo,65536)==bytes(65536)
 return dict(schema='fresh-loop-hybrid-independent-linked-audit-v1',static_pass=True,native_executed=False,source_bytes=2442,game_bytes=2225,crt_bytes=217,low_payload_exact=True,owned_instruction_bytes_preserved=True,import_metadata_only_edits=True,entrypoint_hostmain=True,no_WX_sections=True,no_RWX_virtual_gap_payload=True,primary_image_stops_below_CRT=True,high_runtime_allocations=[f'{a:08X}'for a in allocations],only_high_CRT_mapped_before_ReadRecord=True,all_source_code_startup_compares_before_ReadRecord=True,original_full_entry_transfers_closed=True,provider_clock_call_count_nine_guard_present=True,guarded_buffer_permissions_closed=True,serial_generation_pages_mutable_zero_initial=True,failure_real_stack_closed=True,carrier_sha256=sha(a.carrier),owned_sha256=sha(a.owned_exe),map_sha256=sha(a.owned_map),owned_audit=owned,imports=sorted(source.IMPORTS),limits='Static only. RuntimehighCRTmapping/copy/protection andnumericoutputs remainrootgates. Actualresourcecounts/fourzero masks/unboundfreshowner/isolatedglobal snapshots/explicitSSE2policy are typedboundary. No sampled/liveclip/eventdispatch/registry/scheduling claim.')

def main():
 p=argparse.ArgumentParser(description=__doc__)
 for n in['carrier','owned-exe','owned-map','out']:p.add_argument('--'+n,type=Path,required=True)
 p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--source-crt',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));a=p.parse_args();r=audit(a)
 with a.out.open('x')as f:json.dump(r,f,indent=2);f.write('\n')
 print(sha(a.out))
if __name__=='__main__':main()
