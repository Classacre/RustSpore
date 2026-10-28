"""Read-only independent clock host/domain review. Executes no host or Rust."""
import hashlib,json,struct,sys
from fractions import Fraction
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path[:0]=[str(ROOT/'tools'),str(ROOT/'target/rea/capstone-runtime')]
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import PE,decoder
from capstone import CS_GRP_CALL,CS_GRP_JUMP
from capstone.x86 import X86_OP_IMM
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def rational(bits,width):
 f=struct.unpack('<f' if width==32 else '<d',bits.to_bytes(width//8,'little'))[0]
 return Fraction.from_float(f)
def power(e):return Fraction(1<<e) if e>=0 else Fraction(1,1<<-e)
def round_binary(value,precision,rc,minimum_quantum=None):
 if not value:return value
 negative=value<0;a=abs(value);e=a.numerator.bit_length()-a.denominator.bit_length()
 if a<power(e):e-=1
 quantum=e-precision+1
 if minimum_quantum is not None:quantum=max(quantum,minimum_quantum)
 unit=power(quantum);scaled=a/unit;q,r=divmod(scaled.numerator,scaled.denominator)
 inc= r*2>scaled.denominator or (r*2==scaled.denominator and q%2==1) if rc==0 else bool(r) and ((rc==1 and negative) or(rc==2 and not negative))
 result=(q+int(inc))*unit
 return -result if negative else result
def main(out):
 host=ROOT/'target/source-cell/native-runtime-clock-host-root-reproduction-v2';inputs=ROOT/'target/source-cell/runtime-clock-inputs-v1';original=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));audit=json.loads((host/'linked-audit.json').read_text());manifest=json.loads((host/'manifest.json').read_text());pe=PE(host/'host-x86.exe')
 assert sha(host/'host-x86.exe')=='1b50b88351a7b5eca1edc236decf8b0ddc773d5ea30451feed93ef09ec7503e9'
 assert audit['host_exe_sha256']==sha(host/'host-x86.exe') and audit['host_map_sha256']==sha(host/'host-x86.map')
 assert manifest['source_bytes']==146 and len(manifest['code_allowlist'])==4
 assert manifest['owned_only_change'].startswith('HostFailure restores ESP')
 expected=[(0x9A2BB0,0x9A2BCC),(0x9A1CDB,0x9A1CFF),(0x9A1D4E,0x9A1D57),(0x9A1D69,0x9A1DB2)];transfers=[]
 for row,(a,b) in zip(manifest['code_allowlist'],expected):
  assert (int(row['start'],16),int(row['end_exclusive'],16))==(a,b)
  raw=original.read(a,b-a);assert hashlib.sha256(raw).hexdigest()==row['sha256']
  symbol=int(audit['symbols'][f'Original_{a:08X}'],16);assert pe.read(symbol,b-a)==raw
  decoded=list(decoder().disasm(raw,a));assert sum(i.size for i in decoded)==b-a
  for ins in decoded:
   assert not ins.group(CS_GRP_CALL)
   if ins.group(CS_GRP_JUMP):assert ins.operands[0].type==X86_OP_IMM;transfers.append((ins.address,ins.mnemonic,ins.operands[0].imm))
 assert transfers==[(0x9A1CFD,'jb',0x9A1D4E),(0x9A1D55,'je',0x9A1D69)]
 assert original.read(0x14861D0,4)==b'\0\0\x80?'
 assembly=(host/'host-x86.asm').read_text();assert sha(host/'host-x86.asm')==manifest['host_source_sha256']
 #Validate exact entry/control/shadow writes in the generated host, independently
 #of native output. Source bytes above are compared against the linked PE.
 for needle in ['cmp esp,OFFSET FrameBuffer+07F4h','cmp esp,OFFSET FrameBuffer+07E0h','mov byte ptr [ClipBuffer+0ADh],0','fninit','fldcw word ptr [InputBuffer+4]','ldmxcsr SseControl','cmp ax,word ptr [InputBuffer+4]','test word ptr [ReplyBuffer+22],03800h','and eax,0FFFFFFC0h','cmp eax,01F80h','cmp ecx,0804h','cmp ecx,0818h','cmp ecx,0B0h','cmp ecx,0B4h','mov ecx,20','repe cmpsb']:
  assert needle in assembly,needle
 assert manifest['exits']=={'009A2BCC':'DeltaDone','009A1CFF':'HostFailure','009A1D57':'HostFailure','009A1DB2':'FrameDone'}
 #Independently inspect PE section protection bits and all linked guard extents.
 symbols={k:int(v,16) for k,v in audit['symbols'].items()};header=struct.unpack_from('<I',pe.data,0x3c)[0];section_count=struct.unpack_from('<H',pe.data,header+6)[0];optional_size=struct.unpack_from('<H',pe.data,header+20)[0];sections=[]
 for i in range(section_count):
  at=header+24+optional_size+40*i;virtual,rva=struct.unpack_from('<II',pe.data,at+8);flags=struct.unpack_from('<I',pe.data,at+36)[0];sections.append((pe.base+rva,pe.base+rva+virtual,flags&0xe0000000))
 def protection(address,size):return next(flags for lo,hi,flags in sections if lo<=address and address+size<=hi)
 assert pe.base==0x27000000 and protection(symbols['OwnedStart'],symbols['OwnedEnd']-symbols['OwnedStart'])==0x60000000
 extents=[]
 for name,size in [('Input',64),('Clip',0xc8),('Resource',0x130),('Context',16),('Frame',0x1000),('Reply',28)]:
  address=symbols[name+'Buffer'];lo=address-16;hi=address+size+16;assert symbols[name+'Before']==lo and symbols[name+'After']==address+size and protection(lo,hi-lo)==0xc0000000
  assert pe.read(lo,16)==bytes([0xa5])*16 and pe.read(address+size,16)==bytes([0xa5])*16
  assert all(hi<=a or b<=lo for a,b in extents);extents.append((lo,hi))
 for row in manifest['code_allowlist']:assert protection(symbols['Original_'+row['start']],row['bytes'])==0x40000000
 owned=list(decoder().disasm(pe.read(symbols['OwnedStart'],symbols['OwnedEnd']-symbols['OwnedStart']),symbols['OwnedStart']));assert sum(i.size for i in owned)==symbols['OwnedEnd']-symbols['OwnedStart']
 failure=symbols['HostFailure'];failure_rows=[i for i in owned if failure<=i.address<symbols['OwnedEnd']]
 assert [i.mnemonic for i in failure_rows]==['cmp','je','mov','push','call']
 assert failure_rows[0].op_str==f"dword ptr [0x{symbols['NativeStack']:x}], 0"
 assert failure_rows[1].operands[0].imm==failure_rows[3].address
 assert failure_rows[2].op_str==f"esp, dword ptr [0x{symbols['NativeStack']:x}]" and failure_rows[3].op_str=='7'
 boundaries={i.address for i in owned}
 imports=pe.imports();assert len(imports)==8 and all(dll.lower()=='kernel32.dll' for dll,name in imports.values())
 for ins in owned:
  if ins.group(CS_GRP_CALL) or ins.group(CS_GRP_JUMP):
   assert ins.operands[0].type==X86_OP_IMM;target=ins.operands[0].imm
   if target not in boundaries:
    thunk=pe.read(target,6);assert ins.mnemonic=='call' and thunk[:2]==b'\xff\x25' and struct.unpack_from('<I',thunk,2)[0] in imports
 assert {name for dll,name in imports.values()}=={'GetStdHandle','ReadFile','WriteFile','VirtualAlloc','VirtualProtect','GetLastError','FlushInstructionCache','ExitProcess'}
 raw=(inputs/'input.bin').read_bytes();assert len(raw)==1560*64;counts={};pc_counts={};rc_counts={}
 for at in range(0,len(raw),64):
  req=raw[at:at+64];mode,cw,reserved,t,p,d,f,loop=struct.unpack_from('<IHHIQQII',req)
  assert mode in [0,1] and cw&0xf0ff==0x7f and cw&0x300!=0x100 and reserved==0 and loop==0 and req[36:]==bytes(28)
  rc=cw>>10&3;precision={0:24,2:53,3:64}[(cw>>8)&3];duration=rational(d,64);frames=rational(f,32);time=rational(t,32)
  if mode:time=round_binary(time+rational(p,64),precision,rc);time=round_binary(time,24,rc,-149)
  assert 0<=time<duration and duration>0 and frames>=1
  assert abs(time)<=rational(0x7f7fffff,32)
  ratio=round_binary(round_binary(time/duration,precision,rc),24,rc,-149);assert 0<=ratio<=1
  frame=round_binary(round_binary(frames-1,24,0,-149)*ratio,24,0,-149);assert 0<=frame<=rational(0x7f7fffff,32)
  counts[f'{cw:04X}']=counts.get(f'{cw:04X}',0)+1;pc_counts[str(precision)]=pc_counts.get(str(precision),0)+1;rc_counts[str(rc)]=rc_counts.get(str(rc),0)+1
 static=json.loads((inputs/'static-preparation-review.json').read_text())
 for path,digest in static['pins'].items():assert sha(ROOT/path)==digest
 report=dict(schema='independent-clock-v2-linked-host-static-review-v1',review_result='STATIC PASS v2 linked146byte source/guard/control contract and conditional real-stack failure exit; execution/comparison remain separate',source_sha256=SOURCE_SHA256,source_bytes=146,conditional_transfers=[dict(at=f'{a:08X}',mnemonic=n,target=f'{b:08X}') for a,n,b in transfers],native_host_sha256=sha(host/'host-x86.exe'),host_asm_sha256=sha(host/'host-x86.asm'),linked_audit_sha256=sha(host/'linked-audit.json'),inputs_sha256=sha(inputs/'input.bin'),cases=1560,contexts=counts,precision_counts=pc_counts,rounding_counts=rc_counts,linked_conditional_failure_stack_restore_checked=True,independent_pe_section_guard_transfer_checks=True,exact_rational_domain_verified=True,arithmetic_output_oracle=False,new_native_execution=False,rust_execution=False,limitations=['Read-only review of author-generated linked audit/host plus independent source/PE span and exact rational domain checks; does not replace native replies','No sampler9A1590,event reset9A1690,loop/crossing,liveclip admission or originalprocess','Clock exact strictT32<duration64 branch is intentional; no rate/speed replacement helper','Complete x87status exception flags and highXMM lanes are excluded from projected arithmetic equality; TOP/CW/MXCSRbase controls guarded'])
 out=Path(out);out.resolve().relative_to((ROOT/'target').resolve());assert not out.exists();out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['contexts','limitations']}))
if __name__=='__main__':main(sys.argv[1])
