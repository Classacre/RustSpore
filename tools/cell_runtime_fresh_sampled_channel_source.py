"""Prepare source-only full fresh-channel sampling closure; never compile/execute."""
import argparse,hashlib,json,struct,sys
from pathlib import Path
import cell_runtime_fresh_loop_native_probe as previous
import anim_quaternion_probe as quaternion
import anim_rotation_native_probe as rotation
import cell_runtime_curves_native_probe_v3 as scalar
import native_crt_acos_probe as acos
ROOT=Path(__file__).resolve().parent.parent
EXTRA=[(0x401930,0x401953),(0x99CC80,0x99CC9D),(0x99EA40,0x99EA78),(0x99EA80,0x99EAE1),
 (0x9A0B30,0x9A0BAE),(0x9A0BB0,0x9A0BE1),(0x9A1450,0x9A14BD),(0x9A14C0,0x9A158C),
 (0x99F6A0,0x9A0974),(0x99C6C0,0x99C772),(0x99F630,0x99F697),
 (0x9A3214,0x9A3232),(0x9A328F,0x9A32C4),(0x11E052E,0x11E0534),(0x11E0534,0x11E053A)]
APP=sorted(set(previous.APP_CODE+EXTRA+quaternion.SPANS+[quaternion.THUNK]))
CRT=sorted(set(previous.CRT_CODE+acos.CODE))
DATA=sorted(set(previous.DATA+[previous.ONE]+quaternion.CONSTANTS+rotation.DATA+scalar.DATA+[(0x15514F8,4)]))
EXITS=dict(previous.EXITS,**{})
EXITS.update({0x9A3232:'BorrowedChannelInstalled',0x9A32C4:'EpochAndCacheInitialized'})
TRAPS={a:why for a,why in previous.TRAPS.items()if a not in[0x9A14C0,0x99F6A0]}
TRAPS[0x9C1050]='trueeventcapacity>=2,resetlength0,atmost2authoredevents'
IATS={0x13CC478:'memset',0x13CC47C:'memcpy',0x13CC48C:'_CIacos',0x13CC4C0:'modf'}
MUTABLE=[(0x15514DC,4),(0x15514E0,4),(0x166C240,4)]
def digest(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def imports(images):
 pe=acos.PE(images.app_path);rva,n=pe.directories[1];out={}
 for i in range(n//20):
  oft,_,_,name,ft=struct.unpack('<5I',pe.read(pe.base+rva+20*i,20))
  if not name:break
  for j in range(1024):
   value=pe.u32(pe.base+(oft or ft)+4*j)
   if not value:break
   at=pe.base+ft+4*j
   if at in IATS:assert not value&0x80000000;out[at]=(pe.text(pe.base+name),pe.text(pe.base+value+2))
 assert out=={a:('MSVCR90.dll',name)for a,name in IATS.items()};return out

def audit(images):
 sys.path.insert(0,str(ROOT/'target/rea/capstone-runtime'));from capstone import CS_GRP_CALL,CS_GRP_JUMP
 from capstone.x86 import X86_OP_IMM,X86_OP_MEM,X86_REG_EBP
 for(a,b),(c,d)in zip(APP,APP[1:]):assert b<=c,(hex(a),hex(c))
 decoded=[list(acos.decoder().disasm(images.read(a,b-a),a))for a,b in APP];bounds={i.address for rows in decoded for i in rows};records=[]
 allowed=DATA+MUTABLE+[(a,4)for a in IATS]
 for(a,b),rows in zip(APP,decoded):
  assert sum(i.size for i in rows)==b-a;transfers=[]
  for i in rows:
   if i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP):
    op=i.operands[0]
    if op.type==X86_OP_IMM:target=op.imm;assert target in bounds|set(EXITS)|set(TRAPS),(hex(i.address),hex(target));target=f'{target:08X}'
    else:
     assert op.type==X86_OP_MEM
     if i.address==0x401941:assert op.mem.base==X86_REG_EBP and op.mem.disp==0x14;target='literalcallback0099CC80'
     else:assert op.mem.disp in IATS and not op.mem.base and not op.mem.index;target=f'IAT{op.mem.disp:08X}'
    transfers.append(dict(at=f'{i.address:08X}',kind=i.mnemonic,target=target))
   for op in i.operands:
    if op.type==X86_OP_MEM and not op.mem.base and not op.mem.index:
     assert any(lo<=op.mem.disp and op.mem.disp+op.size<=lo+n for lo,n in allowed),(hex(i.address),hex(op.mem.disp),op.size)
  records.append(dict(start=f'{a:08X}',end_exclusive=f'{b:08X}',bytes=b-a,sha256=hashlib.sha256(images.read(a,b-a)).hexdigest(),transfers=transfers))
 callback=next(rows for(a,b),rows in zip(APP,decoded)if a==0x99CC80)
 assert len(callback)==10 and callback[-1].address==0x99CC9C and callback[-1].mnemonic=='ret'and not any(i.group(CS_GRP_CALL)or i.group(CS_GRP_JUMP)for i in callback)
 assert images.read(0x99EAA6,5)==b'\x68\x80\xcc\x99\x00'
 assert images.read(0x11E052E,6)==b'\xff\x25\x78\xc4\x3c\x01'and images.read(0x11E0534,6)==b'\xff\x25\x7c\xc4\x3c\x01'
 assert struct.unpack('<4I',images.read(0x15514CC,16))==(20,80,100,40)
 assert[struct.unpack('<I',images.read(0x14477D8+32*i,4))[0]for i in range(9)]==[0,20,100,200,240,280,320,360,400]
 assert images.read(0x15514F8,4)==struct.pack('<I',1)
 # Reuse the accepted finite CRT instruction/table bound proof without changing it.
 finite,table=acos.audit_source(images.crt)
 return records,finite,table

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());images=previous.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));records,finite,table=audit(images);proof=imports(images);a.out.mkdir(exist_ok=False,parents=True)
 for lo,hi in APP+CRT:
  raw=images.read(lo,hi-lo);(a.out/f'{lo:08X}-original.bin').write_bytes(raw);rows=list(acos.decoder().disasm(raw,lo));(a.out/f'{lo:08X}-disassembly.txt').write_text('\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in rows)+'\n')
 for lo,n in DATA+[(acos.WINDOW[0],acos.WINDOW[1]-acos.WINDOW[0])]:(a.out/f'{lo:08X}-{n}-constant.bin').write_bytes(images.read(lo,n))
 m=dict(schema='source-fresh-first-channel-complete-sampler-closure-v1',static_source_pass=True,native_executed=False,source_sha256=previous.arithmetic.SOURCE_SHA256,crt_sha256=previous.arithmetic.CRT_SHA,game_bytes=sum(b-a for a,b in APP),crt_bytes=sum(b-a for a,b in CRT),code=records,finite_CRT_proof=finite,finite_CRT_table_bounds=table,imports={f'{k:08X}':dict(dll=v[0],name=v[1])for k,v in proof.items()},source_default_payload='Original fullconstructor sets440byteszero thenINFOword4=old15514DC++; no borrowed guessedpayload defaults.',explicit_partial_entries={'009A3214..009A3232':'Install realchannel pointer from resource148table with original count/index fields; allocation/refownership excluded.','009A328F..009A32C4':'Original zero genuine keyepoch count and initialize cachedINFOword; preallocated vectors explicit.'},callback={'entry':'0099CC80','end_exclusive':'0099CC9D','manager_procedure':None,'instructions':10,'positive_literal_source_target_and_straightline_RET':True,'failed_query84_retained':True},global_counters=['015514DC','015514E0','0166C240'],main_base='003F0000',CRT_pages_separate=True,owned_provider_contracts={'memset':'Exact original6Bthunk/IAT, source return99EAC9, value0/count440/destcurrentwrapper+4 only.','memcpy':'Exact original6Bthunk/IAT; cleanupNULL,NULL,0 without dereference or bounded20/40/80/100B readonlysource row to legalcurrentwrapper slot; cdecl unchanged.','acos':'Unchanged original11E06B8 thunk/IAT explicitly selects reviewed originalfiniteSSE2kernel785A35E0; source runtime producer unknown.','modf':'Reuse accepted unchanged original finiteCRT/provider ABI.'},typed_rotation_policy='ExplicitSourceImageInitialOne(15514F8); no retail historyclaim.',opaque_bytes='Wrapper1E8..1EF untouched/guarded. Event8Brecord middleU16copiedopaque, onlychannelU16+keyI32 projected. Whole440payload exactacrosssteps.',limits='Source-only; no actual host or numeric admission. Fullbinder/reference/allocation/actor registry and eventdispatch remain separate.',tool_sha256=digest(__file__))
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(m['game_bytes'],m['crt_bytes'])
if __name__=='__main__':main()
