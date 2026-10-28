"""Static original-source event-tail counterexample; never execute native code."""
import json,struct,hashlib
from pathlib import Path
import cell_runtime_fresh_sampled_channel_native_probe as source
ROOT=source.ROOT
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 out=ROOT/'target/source-cell/runtime-fresh-sampled-channel-event-tail-diagnosis-v1';out.mkdir(exist_ok=True);assert not(out/'diagnosis.json').exists()
 packet=ROOT/'target/source-cell/native-runtime-fresh-sampled-channel-source-v1';host=ROOT/'target/source-cell/native-runtime-fresh-sampled-channel-root-v2/host-x86.asm';asm=host.read_text();start=asm.index('StepLoop:\n');assert 'mov esi,OFFSET EventsBuffer'not in asm[start:asm.index('RunOnce ENDP',start)]
 assert 'GuardEventByte:\ncmp ecx,edx\njb NextEventByte\nmov al,byte ptr [EventsBuffer+ecx]\ncmp al,byte ptr [EventsShadow+ecx]'in asm
 # Original event lookup only READS keyepochs. Zero reset passes range[end,end)
 # to7D0230: its copyloop is skipped and only the vector endpointer changes.
 images=source.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'))
 assert images.read(0x9A154C,3)==bytes.fromhex('390cb0')
 assert images.read(0x7D0242,4)==bytes.fromhex('3bfb7416') and images.read(0x7D0269,3)==bytes.fromhex('017904')
 assert images.read(0x9A0BC6,13)==bytes.fromhex('8b1189108b4904894804c20400')
 cases=json.loads((ROOT/'target/source-cell/runtime-fresh-loop-inputs-v1/cases.json').read_text());old=(ROOT/'target/source-cell/runtime-fresh-loop-native-run-v1/corpus-output.bin').read_bytes();assert Path(cases[240]['clip']).name.startswith('8d26f08b5e')
 c=source.source_cases()[4];ch=c['channels'][0];raw=c['raw'];u=lambda at:struct.unpack_from('<I',raw,at)[0];stride=u(ch['descriptors']+12);events=[dict(key=i,timestamp=u(ch['keys']+i*stride),event_byte=raw[ch['keys']+i*stride+10])for i in range(ch['count'])if raw[ch['keys']+i*stride+10]];assert events==[dict(key=2,timestamp=6,event_byte=1),dict(key=7,timestamp=216,event_byte=3)]
 buffer=bytearray(128);initial=bytes(buffer);prior=bytes(buffer);trace=[]
 for j in range(9):
  snap=old[240*864+j*96:240*864+(j+1)*96];frame=struct.unpack_from('<f',snap,16)[0];keys=[e['key']for e in events if e['timestamp']<=frame and struct.unpack_from('<I',snap,32)[0]>0]
  # Only the six defined event bytes are needed to prove the counterexample;
  # opaque padding remains symbolic/preserved, never assigned semantic meaning.
  for i,key in enumerate(keys):struct.pack_into('<H',buffer,8*i,0);struct.pack_into('<I',buffer,8*i+4,key)
  n=8*len(keys);old_guard=buffer[n:]==initial[n:];new_guard=buffer[n:]==prior[n:]
  trace.append(dict(snapshot=j,accepted_zero_mask_clock_frame_bits=f'{struct.unpack_from("<I",snap,16)[0]:08X}',frame=frame,source_event_keys=keys,old_initial_shadow_guard_pass=old_guard,correct_preclock_shadow_guard_pass=new_guard,retained_defined_nonzero_tail_offsets=[i for i in range(n,128)if buffer[i]]));prior=bytes(buffer)
 first=next(r for r in trace if not r['old_initial_shadow_guard_pass']);assert first['snapshot']==5 and first['source_event_keys']==[] and all(r['correct_preclock_shadow_guard_pass']for r in trace)
 pins={str(p):sha(p)for p in [Path(__file__),host,ROOT/'target/source-cell/runtime-fresh-sampled-channel-native-run-v1/failure.json',ROOT/'target/source-cell/runtime-fresh-loop-native-run-v1/corpus-output.bin',ROOT/'target/source-cell/runtime-fresh-loop-native-run-v1/rust-comparison-v1.json',ROOT/'target/source-cell/runtime-fresh-loop-inputs-v1/cases.json',c['path']]+[packet/f'{name}-disassembly.txt'for name in ['009A14C0','009A0BB0','009A1690','007D0230','009A1CB0']]}
 report=dict(schema='source-event-retained-tail-owned-guard-counterexample-v1',static_source_proof=True,native_executed=False,new_numeric_observation=False,accepted_zero_mask_clock_frames_used=True,actual_failure_exit=7,source_events=events,trace=trace,first_source_derived_bad_guard_snapshot=5,source_lookup_does_not_write_epoch=True,event_capacity_two_per_call_still_valid=True,original_reset_preserves_event_data=True,rootcause='Owned GuardEvents compared retained bytes beyond currentevent length to RunOnceinitialzeroEventsShadow. Source reset changes only endpointer; after wrappingframe0 stale defined keybytes legitimatelyremain. SnapshotEventsShadow immediatelybefore every sourceclock call; allow only currentappendprefix and compare everytailbyte to that preciseincomingstate.',limits='The exit7 failure itself does not localize an instruction. This is an independently reproducible unavoidable source-derived rejection in the frozenhost; corrected execution remains separate.',pins=pins)
 (out/'diagnosis.json').write_text(json.dumps(report,indent=2)+'\n');print(sha(out/'diagnosis.json'))
if __name__=='__main__':main()
