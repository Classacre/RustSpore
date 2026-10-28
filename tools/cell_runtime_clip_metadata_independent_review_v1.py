"""Independent static peer of actual64-byte source clip-metadata host. No execution."""
import hashlib,json,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
import cell_runtime_clip_metadata_native_probe as probe
from cell_schema_probe import SourceImage,SOURCE_SHA256
from native_crt_acos_probe import PE,decoder

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 host=ROOT/'target/source-cell/native-runtime-clip-metadata-host-root-reproduction-v1';inputs=ROOT/'target/source-cell/runtime-clip-metadata-inputs-v1';image=SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));source=probe.audit_source(image);assert sum(b-a for a,b in probe.CODE)==64
 assert (host/'host-x86.asm').read_text()==probe.host_source(image)
 probe.base.BUFFERS=probe.BUFFERS;fresh=probe.base.linked_audit(host/'host-x86.exe',host/'host-x86.map',image);retained=json.loads((host/'linked-audit.json').read_text());entryflag=retained.pop('pe_entrypoint_hostmain');assert entryflag and fresh==retained
 assert fresh['host_exe_sha256']=='0145de54218b405a0cab19fe17d894895dfe4d80455b8ec0e7b8dfebca249227'
 pe=PE(host/'host-x86.exe');s={k:int(v,16) for k,v in fresh['symbols'].items()};nt=struct.unpack_from('<I',pe.data,60)[0];op=nt+24;assert pe.base+struct.unpack_from('<I',pe.data,op+16)[0]==s['HostMain'];assert not struct.unpack_from('<H',pe.data,op+70)[0]&0x40
 sections=[]
 for i in range(struct.unpack_from('<H',pe.data,nt+6)[0]):
  at=op+224+40*i;vs,rva,rs,fo=struct.unpack_from('<4I',pe.data,at+8);flags=struct.unpack_from('<I',pe.data,at+36)[0];assert rva%4096==0 and fo%512==0 and fo+rs<=len(pe.data);sections.append((pe.base+rva,vs,flags))
 assert sections[0][0]==pe.base+4096 and all(a+((n+4095)&~4095)==b for (a,n,f),(b,m,g) in zip(sections,sections[1:]))
 def perm(a,n):return next(f&0xe0000000 for lo,size,f in sections if lo<=a and a+n<=lo+size)
 ext=[]
 for name,n in probe.BUFFERS:
  a=s[name+'Buffer'];assert s[name+'Before']==a-16 and s[name+'After']==a+n;assert pe.read(a-16,16)==pe.read(a+n,16)==b'\xa5'*16
  for lo,size in [(a-16,n+32)]+([] if name=='Reply' else [(s[name+'Shadow'],n)]):
   assert perm(lo,size)==0xc0000000 and all(lo+size<=x or y<=lo for x,y in ext);ext.append((lo,lo+size))
 assert perm(s['Expected'],16)==0xc0000000 and pe.read(s['SseControl'],4)==bytes.fromhex('801f0000') and pe.read(s['NativeStack'],4)==bytes(4)
 rows=list(decoder().disasm(pe.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']),s['OwnedStart']));by={r.address:i for i,r in enumerate(rows)}
 from capstone.x86 import X86_OP_IMM
 entries=[]
 for i,r in enumerate(rows):
  if r.mnemonic=='ret' and i and rows[i-1].mnemonic=='push' and rows[i-1].operands[0].type==X86_OP_IMM:
   entries.append(rows[i-1].operands[0].imm)
 assert entries==[0x9a3198,0x9a32d4]
 d=rows[by[s['DurationDone']]:by[s['LoopDone']]];assert [r.mnemonic for r in d[:4]]==['cmp','jne','mov','mov'];assert d[0].operands[1].imm==s['FrameBuffer']+0x800 and d[2].op_str.startswith('esp,')
 failure=rows[by[s['HostFailure']]:];assert [r.mnemonic for r in failure[:5]]==['cmp','je','mov','push','call'];assert failure[2].op_str.startswith('esp,') and failure[3].operands[0].imm==7
 # Independent literal source memory contract and skipped XORPS proof.
 assert image.read(0x9a31b0,2)==bytes.fromhex('eb03') and 0x9a31b2+3==0x9a31b5
 asm=(host/'host-x86.asm').read_text();assert 'cmp ecx,0ADh\nje NextClip\ncmp ecx,0B8h\njb CompareClip\ncmp ecx,0C0h\njb NextClip' in asm
 assert 'cmp ecx,07FCh\nje NextFrame\ncmp ecx,0820h\nje NextFrame' in asm
 assert 'mov esp,OFFSET FrameBuffer+0800h\npush 009A3198h\nret' in asm
 assert 'call _GetLastError@0\ncmp eax,109' in asm and 'AllocationFailure:\npush 10\ncall _ExitProcess@4' in asm
 inp=json.loads((inputs/'report.json').read_text());assert sha(inputs/'input.bin')==inp['input_sha256'] and sha(inputs/'cases.json')==inp['cases_sha256'];raw=(inputs/'input.bin').read_bytes();cases=json.loads((inputs/'cases.json').read_text());assert len(raw)==len(cases)*32==2304*32
 from cell_runtime_clip_metadata_inputs import bounded
 contexts=set();flags=set()
 for i,c in enumerate(cases):
  q=raw[i*32:(i+1)*32];cw,res,fl,fr,iv=struct.unpack_from('<HHIII',q);assert cw&0xf0ff==0x7f and cw&0x300!=0x100 and res==0 and q[16:]==bytes(16);assert bounded(fr,iv);assert (f'{cw:04X}',f'{fl:08X}',f'{fr:08X}',f'{iv:08X}')==(c['cw'],c['flags'],c['frames'],c['interval']);contexts.add(cw);flags.add(fl)
 assert len(contexts)==12 and flags=={0,8,0xfffffff7,0xffffffff}
 pins={name:sha(ROOT/name) for name in inp['pins']};assert pins==inp['pins']
 paths=[Path(__file__),Path(probe.__file__),Path(probe.base.__file__),ROOT/'tools/cell_runtime_clip_metadata_inputs.py',ROOT/'crates/spore-core/src/cell_runtime_clip_metadata.rs',ROOT/'crates/spore-core/examples/cell_runtime_clip_metadata_verify.rs.pending',*[host/n for n in ['host-x86.asm','manifest.json','host-x86.exe','host-x86.map','linked-audit.json']],inputs/'input.bin',inputs/'report.json',inputs/'cases.json']
 pins.update({str(p.relative_to(ROOT)):sha(p) for p in paths})
 report=dict(schema='source-cell-clip-metadata-independent-actual-peer-v1',no_blocking_static_issue_found=True,original_spans_exact=True,partial_entry_frame_registers_closed=True,guarded_memory_bounds_closed=True,owned_control_and_failure_stack_closed=True,protocol_guards_closed=True,PE_entry_import_sections_closed=True,host_executed=False,native_executed=False,source_sha256=SOURCE_SHA256,source_bytes=64,source_audit=source,cases=2304,contexts=12,pins=pins,limits='Static partial constructor metadata only: nonnull selected resource,frames>1,interval>0,finite positive committed product; explicit runtime-word inputs. No ANIM-header mapping,allocation/ownership,actualresource/actorselection,sampling/events,or native/Rust arithmetic acceptance. EDX resource initialization at second entry is redundant: first source MOV EDX,[EBX] defines it. Frame7FC is owned PUSH/RET landing scratch; source writes onlyFrame820 andClipB8..BF/AD.')
 p=Path(out);p.resolve().relative_to((ROOT/'target').resolve())
 with p.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(p))
if __name__=='__main__':main(sys.argv[1])
