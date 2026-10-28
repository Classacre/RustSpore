"""Independent actual-linked static peers; never launches a native host."""
import hashlib,importlib,json,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import PE,decoder
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def review(kind,expected,count,nbytes):
 p=importlib.import_module('cell_runtime_'+kind+'_native_probe');host=ROOT/('target/source-cell/native-runtime-'+kind.replace('_','-')+'-host-root-reproduction-v1');inputs=ROOT/('target/source-cell/runtime-'+kind.replace('_','-')+'-inputs-v1')
 image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));src=p.audit_source(image);assert sum(b-a for a,b in p.CODE)==nbytes
 asm=(host/'host-x86.asm').read_text();assert asm==p.host_source(image)
 audit=p.linked_audit(host/'host-x86.exe',host/'host-x86.map',image);assert audit==json.loads((host/'linked-audit.json').read_text());assert sha(host/'host-x86.exe')==expected
 pe=PE(host/'host-x86.exe');s={k:int(v,16) for k,v in audit['symbols'].items()};nt=struct.unpack_from('<I',pe.data,60)[0];op=nt+24
 assert pe.base+struct.unpack_from('<I',pe.data,op+16)[0]==s['HostMain'];assert not struct.unpack_from('<H',pe.data,op+70)[0]&0x40
 sections=[]
 for i in range(struct.unpack_from('<H',pe.data,nt+6)[0]):
  at=op+224+40*i;vs,rva,rs,fo=struct.unpack_from('<4I',pe.data,at+8);flags=struct.unpack_from('<I',pe.data,at+36)[0];assert rva%4096==0 and fo%512==0 and fo+rs<=len(pe.data);sections.append((pe.base+rva,vs,flags))
 assert sections[0][0]==pe.base+4096 and all(a+((n+4095)&~4095)==b for (a,n,f),(b,m,g) in zip(sections,sections[1:]))
 def perm(a,n):return next(f&0xe0000000 for lo,size,f in sections if lo<=a and a+n<=lo+size)
 ext=[]
 for name,n in p.BUFFERS:
  a=s[name+'Buffer'];assert s[name+'Before']==a-16 and s[name+'After']==a+n;assert pe.read(a-16,16)==pe.read(a+n,16)==b'\xa5'*16
  for lo,size in [(a-16,n+32)]+([] if name=='Reply' else [(s[name+'Shadow'],n)]):
   assert perm(lo,size)==0xc0000000 and all(lo+size<=x or y<=lo for x,y in ext);ext.append((lo,lo+size))
 assert pe.read(s['NativeStack'],4)==bytes(4) and pe.read(s['SseControl'],4)==bytes.fromhex('801f0000')
 rows=list(decoder().disasm(pe.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']),s['OwnedStart']));by={r.address:i for i,r in enumerate(rows)}
 from capstone.x86 import X86_OP_IMM
 entries=[rows[i-1].operands[0].imm for i,r in enumerate(rows) if r.mnemonic=='ret' and i and rows[i-1].mnemonic=='push' and rows[i-1].operands[0].type==X86_OP_IMM]
 assert entries==([0xa00310] if kind=='controller_loop' else [0x9a33c0,0x9a1690])
 fail=rows[by[s['HostFailure']]:];assert [r.mnemonic for r in fail[:5]]==['cmp','je','mov','push','call'];assert fail[2].op_str.startswith('esp,') and fail[3].operands[0].imm==7
 for name,offset in ([('SetterDone',0x804)] if kind=='controller_loop' else [('ConstructorDone',0x804),('ResetDone',0x808)]):
  q=rows[by[s[name]]:];assert [r.mnemonic for r in q[:3]]==['cmp','jne','mov'];assert q[0].operands[1].imm==s['FrameBuffer']+offset and q[2].op_str.startswith('esp,')
 assert 'call _GetLastError@0\ncmp eax,109' in asm and 'AllocationFailure:\npush 10\ncall _ExitProcess@4' in asm
 report=json.loads((inputs/'report.json').read_text());raw=(inputs/'input.bin').read_bytes();assert len(raw)==count*64 and sha(inputs/'input.bin')==report['input_sha256'] and sha(inputs/'cases.json')==report['cases_sha256'];assert report['cases']==count
 for i in range(count):
  v=struct.unpack('<16I',raw[i*64:(i+1)*64])
  if kind=='controller_loop':
   assert v[1]<16 and v[3]<16 and v[1]!=v[3] and v[7] in (0,1) and v[8] in (0,1) and all(x<=255 for x in v[9:13]) and v[13]==int(v[6]==1) and v[14:]==(0,0)
   assert not v[0] or v[0]&255==v[1]+1
   if v[0] and v[0]>>8==v[2] and v[5]:assert v[5]&255==v[3]+1
  else:
   cw=v[1]&65535;assert v[1]>>16==0 and cw&0xf0ff==0x7f and cw&0x300!=0x100 and v[2] in (0,1) and 0<=v[3]<=v[4]<=16 and not any(v[5:]);assert v[2] or v[3:5]==(0,0)
 pins=report['pins'].copy();assert all(sha(ROOT/k)==v for k,v in pins.items())
 paths=[Path(__file__),*[host/n for n in ['host-x86.asm','host-x86.exe','host-x86.map','manifest.json','linked-audit.json']],*[inputs/n for n in ['input.bin','cases.json','report.json']]];pins.update({str(x.relative_to(ROOT)):sha(x) for x in paths})
 limits='Static admission only. Dual RunOnce changes Poison global but does not load XMM poison; no empirical distinct-XMM-state claim. Controller is integer-only; fresh constructor defines used XMM0/1 before use. No runtime/live-actor/resource admission or native arithmetic acceptance.'
 notes=('Resolver establishes primary before secondary; epoch mismatch performs no mutation. Secondary slot/Aid are lookup-only; only resolved primary+D4 and its nonnull Aid+AD may change.' if kind=='controller_loop' else 'Constructor preclears three references, wrapper and event vectors before reuse helper. Seven excluded calls are unreachable source edges, not seven installed INT3 stubs: only allocated source-page slack is CC. Reset0 uses last=end, skips record copying, modifies end only and preserves begin/capacity/records. Entire serial RW page is shadowed except source INC DWORD at15514E0.')
 result=dict(schema='source-cell-'+kind.replace('_','-')+'-independent-actual-peer-v1',no_blocking_static_issue_found=True,original_spans_exact=True,full_entry_ABI_closed=True,guarded_memory_bounds_closed=True,owned_control_and_failure_stack_closed=True,protocol_guards_closed=True,PE_entry_import_sections_closed=True,rust_model_source_review_passed=True,host_executed=False,native_executed=False,source_sha256=SOURCE_SHA256,source_bytes=nbytes,cases=count,source_audit=src,pins=pins,source_contract_notes=notes,limits=limits)
 out=ROOT/('target/source-cell/runtime-'+kind.replace('_','-')+'-independent-font-review-v1.json')
 with out.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(str(out),sha(out))
if __name__=='__main__':
 review('controller_loop','978d3c657688d5114abb6dbd83a4f892e666ea55e9ce7a458edc9c7ec0ede940',536,183)
 review('fresh_aid','aa5c95a8e25dc8a1a5ac19e690281ed5daebe29b038ad3106ef025b2026a1b6c',288,1049)
