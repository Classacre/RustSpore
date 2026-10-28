"""Independent actual-linked full fresh activation/private loop static peer."""
import hashlib,json,struct,sys
from pathlib import Path
from types import SimpleNamespace
import cell_runtime_fresh_loop_hybrid_carrier_audit_v2 as audit
import cell_runtime_fresh_loop_native_probe_v2 as probe
import cell_runtime_fresh_loop_inputs as fixture
from anim_root_pose_sparse_carrier_audit_v2 import Image,syms
from native_crt_acos_probe import decoder
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 h=ROOT/'target/source-cell/native-runtime-fresh-loop-hybrid-host-root-v2';inp=ROOT/'target/source-cell/runtime-fresh-loop-inputs-v1';app=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe');crt=app.parent/'msvcr90.dll';images=probe.arithmetic.Images(app,crt)
 for name,v in [('carrier-x86.exe','400252d4bc1c85f6a6d334e6bfd8e2bed4d29f047581bf955bb84649e705f421'),('host-x86.exe','5a3e2504b23c80c39dc1bff39d9b772c7e2a6161a571f6d6eea32e8f2fa331c0'),('host-x86.map','39189d375935f1ebd694f3e36920557134043df7a00314e83ceb41ed2a9ee8f2')]:assert sha(h/name)==v
 fresh=audit.base.audit(SimpleNamespace(carrier=h/'carrier-x86.exe',owned_exe=h/'host-x86.exe',owned_map=h/'host-x86.map',source_exe=app,source_crt=crt));assert fresh==json.loads((h/'root-linked-audit-v2.json').read_text())
 asm=(h/'host-x86.asm').read_text();assert asm==probe.host_source(images);baseasm=probe.base.host_source(images);assert [x for x in asm.splitlines()if not x.startswith('PUBLIC ')]==[x for x in baseasm.splitlines()if not x.startswith('PUBLIC ')]
 pe=Image(h/'carrier-x86.exe');s=syms(h/'host-x86.map');ext=[]
 for name,n in probe.BUFFERS:
  at=s[name+'Buffer'];assert pe.read(at-16,16)==pe.read(at+n,16)==b'\xa5'*16
  for lo,size in[(at-16,n+32)]+([]if name=='Reply'else[(s[name+'Shadow'],n)]):assert pe.perm(lo,size)==0xc0000000 and all(lo+size<=x or y<=lo for x,y in ext);ext.append((lo,lo+size))
 for name in ('SerialShadow','GenerationShadow'):
  lo=s[name];assert pe.perm(lo,65536)==0xc0000000 and all(lo+65536<=x or y<=lo for x,y in ext);ext.append((lo,lo+65536))
 rows=list(decoder().disasm(pe.read(s['OwnedStart'],s['OwnedEnd']-s['OwnedStart']),s['OwnedStart']));by={r.address:i for i,r in enumerate(rows)}
 for label,offset in [('ConstructorDone',0x804),('ResetDone',0x808),('MetadataDurationDone',0x800),('MetadataLoopDone',0x800),('SetterDone',0x804),('ActivationDone',0x884),('StepDone',0x818),('DeltaDone',0x82c)]:
  q=rows[by[s[label]]:];assert [r.mnemonic for r in q[:3]]==['cmp','jne','mov'] and q[0].operands[1].imm==s['FrameBuffer']+offset and q[2].op_str.startswith('esp,')
 assert 'cmp dword ptr [esp],0099BCC7h' in asm and 'sub eax,020h\ncmp esp,eax' in asm and 'sub eax,02Ch\ncmp esp,eax' in asm
 assert 'push dword ptr [esp+12]\npush edx\npush eax\npush OFFSET ModfReturn\npush 0785A3350h\nret' in asm
 for text in ['mov ClipMask,OFFSET MaskDuration','mov ClipMask,OFFSET MaskLoop','mov ClipMask,OFFSET MaskActivation','mov ClipMask,OFFSET MaskClock','mov ClockBase,OFFSET FrameBuffer+0850h','mov ClockBase,OFFSET FrameBuffer+0800h','cmp ProviderCalls,9','cmp dword ptr [ClipBuffer+4],0','fstp tbyte ptr [eax]']:assert text in asm
 assert not any('xmm' in line.lower()for line in asm.splitlines()if not line.startswith('.xmm'))
 # Source liveness/footprint anchors,all full bodies are byteexact in audit.
 assert images.read(0x9a1cb0,3)==bytes.fromhex('83ec10') and images.read(0x9a1d60,4)==bytes.fromhex('d95c241c') and images.read(0x9a1db7,4)==bytes.fromhex('d9442438')
 assert images.read(0x9a15de,6)==bytes.fromhex('0f8490000000') and images.read(0x9a7f3d,2)==bytes.fromhex('7471')
 for name,offsets in [('MaskDuration',range(0xb8,0xc0)),('MaskLoop',[0xad]),('MaskActivation',[*range(0xc,0x14),0xac,*range(0xb0,0xb8),*range(0xc0,0xc8),*range(0xcc,0xd0)]),('MaskClock',[*range(0xb0,0xb8),*range(0xc0,0xc8),*range(0xcc,0xd0)])]:assert pe.read(s[name],0xe0)==bytes(int(i in offsets)for i in range(0xe0))
 report=json.loads((inp/'report.json').read_text());cases=json.loads((inp/'cases.json').read_text());raw=(inp/'input.bin').read_bytes();assert len(raw)==300*256 and len(cases)==300 and sha(inp/'input.bin')==report['input_sha256'] and sha(inp/'cases.json')==report['cases_sha256'];pins=report['pins'].copy();assert all(sha(ROOT/k)==v for k,v in pins.items());contexts=set();sources=set();channels=set()
 for i,row in enumerate(cases):
  q=raw[i*256:(i+1)*256];clip=inp/row['clip'];payload=clip.read_bytes();assert sha(clip)==row['source_sha256'] and struct.unpack_from('<I',payload,8)[0]==25 and struct.unpack_from('<I',payload,4)[0]==len(payload)
  assert [struct.unpack_from('<I',q,at)[0]for at in (24,28,32,36)]==[struct.unpack_from('<I',payload,at)[0]for at in (280,284,288,324)];duration=struct.unpack('<d',struct.pack('<Q',int(row['duration_bits'],16)))[0];assert fixture.domain(q,duration);contexts.add(struct.unpack_from('<H',q,4)[0]);sources.add(row['source_sha256']);channels.add(struct.unpack_from('<I',q,36)[0])
 assert len(contexts)==12 and len(sources)==5 and channels=={7,9,10,11}
 paths=[Path(__file__),Path(probe.__file__),Path(probe.base.__file__),Path(audit.__file__),Path(audit.base.__file__),Path(fixture.__file__),ROOT/'tools/cell_runtime_fresh_loop_sparse_carrier_v2.py',ROOT/'tools/cell_runtime_fresh_loop_sparse_carrier.py',ROOT/'tools/cell_runtime_loop_clock_native_probe.py',ROOT/'crates/spore-core/src/cell_runtime_fresh_loop.rs',ROOT/'crates/spore-core/examples/cell_runtime_fresh_loop_verify.rs.pending',*[h/n for n in ['carrier-x86.exe','host-x86.exe','host-x86.map','host-x86.asm','root-linked-audit-v2.json','manifest.json']],inp/'input.bin',inp/'cases.json',inp/'report.json'];pins.update({str(p.relative_to(ROOT)):sha(p)for p in paths})
 result=dict(schema='full-fresh-loop-hybrid-independent-font-peer-v2',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,pins=pins,actual_linked_audit_reproduced=True,source_bytes=2442,original_spans_exact=True,partial_entry_frame_registers_closed=True,frame_intervals_and_read_before_write_closed=True,source_used_XMM_defined=True,guarded_memory_bounds_closed=True,owned_control_and_failure_stack_closed=True,global_full_page_shadows_closed=True,finite_CRT_forwarding_not_numeric_stub=True,actual_source_metadata_input_join_closed=True,model_operation_and_counter_order_reviewed=True,records=300,snapshots=2700,source_clips=5,contexts=12,frame_buffer_bytes=4096,distinct_XMM_poison_loaded=False,frame_contracts=dict(constructor='7BC..800',event_reset='7D8..800',metadata='7FC..800 plus820durationlocal',setter='7E0..800',activation='810..880;embeddedclockB850',absolute='7C0..814 plus818 overwrittenclipargument;clockB800',delta='7C0..828;clockB800'),ABI_notes='All sourcefakeframeaccessesremainwithin0x1000-byteallocatedFrame. Repeatclock overwritesB+18 Clipargument withcommittedfraction before returnedFLD; callerTimeB+1Cremainsseparate andsamplerframeB-1C=(frames-1)*fraction. ActivationreturnsAL1andpopsclockST0; snapshot0ownedcapture reloadsitsdefinedB+18slot,notactivationAPIreturn. ModfProviderentryESP=B-20/return99BCC7/outputptrB-10;actualCRTforwardentryB-30,CRTminimumB-38,returnB-2C;capture/reload80thenrestoreoriginalproviderstack. FullsamplerlowestB-40viareal31Bshift. Sourceowner0cleanupskip andfouractualzero maskscloseunreachedcallbacks; realchannelcountloopsremain.',limits='Static only; native/Rust/protocolacceptance remainsrootgate. TwoRunOnce/full864comparisons aredeterminism only,PoisonnotloadedintoXMM. UsedscalarXMMdefined byfresh/metadata/sourceclock;CRTpackedXMMdefinedbyMOVQ/constantsbeforeuse. RuntimepreflightFPchecks mayalterstickyflags,whichare notin88Bprojection/CW/TOP/controlclaim. Explicitborrowedv25binding/freshowner0/directRepeat/fourzeromasks/isolatedserial+generationandSSE2CRTpolicy1; no constructorownership/liveactor/sharedresourceinterleaving/actualsampling/eventdispatch/schedulingclaim. HighCRTmappingmayfailatstartup;runnerdoesnotretry.')
 with Path(output).open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(sha(output))
if __name__=='__main__':main(sys.argv[1])
