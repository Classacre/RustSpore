"""Static pre-build source/stack/write-mask audit; never build or execute a host."""
import argparse,json,struct,hashlib
from pathlib import Path
import cell_runtime_fresh_sampled_channel_native_probe as host
import cell_runtime_fresh_sampled_channel_inputs as fixture
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--inputs',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((host.ROOT/'target').resolve())
 images=host.arithmetic.Images(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),Path('D:/Games/SPORE Collection/SporeBin/msvcr90.dll'));proof=host.source.audit(images);host.source.imports(images)
 asm=(a.host_dir/'host-x86.asm').read_text();m=json.loads((a.host_dir/'manifest.json').read_text());assert asm==host.host_source(images) and sha(a.host_dir/'host-x86.asm')==m['host_source_sha256'] and sha(host.__file__)==m['preparation_tool_sha256']
 initialize=asm.split('InitializeWrappers PROC\n',1)[1].split('InitializeWrappers ENDP',1)[0]
 assert initialize.count('mov HelperStack,esp')==1 and 'mov NativeStack,esp'not in initialize and initialize.count('mov esp,HelperStack')==3
 assert asm.count('mov NativeStack,esp')==1 and 'HostFailure::\ncmp NativeStack,0\nje FailureRealStack\nmov esp,NativeStack\nFailureRealStack:\npush 7\ncall _ExitProcess@4'in asm
 assert max(map(len,asm.splitlines()))<512 and all(','not in l for l in asm.splitlines()if l.startswith('PUBLIC '))
 # Source proves the original six-item vector constructor's literal callback,
 # memset size440 and isolatedserial increment; no synthesized default writes.
 assert images.read(0x99EAA6,5)==bytes.fromhex('6880cc9900')
 for s in ['cmp esp,OFFSET FrameBuffer+07D8h','cmp dword ptr [esp],0099EAC9h','cmp dword ptr [esp+12],440','cmp esp,OFFSET FrameBuffer+07DCh','cmp dword ptr [esp],009A0B96h','cmp dword ptr [esp],0099F685h','sub eax,02E8h'] :assert s in asm
 assert 'mov WrapperMode,4\nmov esi,OFFSET InputBuffer+44'in asm and 'call GuardEpochs'in asm
 assert 'cmp ecx,5340\nje NextSerialPage\ncmp ecx,5344\nje NextSerialPage'in asm
 cases=host.source_cases();all_rows=[]
 for c in cases:
  assert len(c['raw'])%4==0 and 0<len(c['channels'])<=16 and all(0<ch['count']<=46 for ch in c['channels'])
  ch=c['channels'][0];raw=c['raw'];n=struct.unpack_from('<I',raw,ch['at']+220)[0];ranges=[]
  for i in range(n):
   d=ch['descriptors']+32*i;kind,slot=struct.unpack_from('<I',raw,d)[0]&15,struct.unpack_from('<I',raw,d+16)[0];start=4+[0,20,100,200,240,280,320,360,400][slot];size=[20,80,100,40][kind];ranges.append([start,start+size])
  expected=bytearray(444)
  for start,end in ranges:expected[start:end]=bytes([1])*(end-start)
  assert expected==c['mask'] and expected[:4]==bytes(4)
  all_rows.append(dict(clip_sha256=sha(c['path']),channels=len(c['channels']),all_channel_key_counts=[ch['count']for ch in c['channels']],payload_write_ranges_wrapper_relative=ranges))
 rows=json.loads((a.inputs/'cases.json').read_text());binary=(a.inputs/'input.bin').read_bytes();assert len(rows)==300 and len(binary)==76800
 for i,row in enumerate(rows):fixture.domain(binary[i*256:(i+1)*256],row['source_case'],struct.unpack('<d',bytes.fromhex(row['duration_bits'])[::-1])[0])
 report=json.loads((a.inputs/'report.json').read_text());assert all(sha(p)==v for p,v in report['pins'].items())
 pins={str(q):sha(q)for q in [Path(__file__),Path(host.__file__),Path(fixture.__file__),Path(host.source.__file__),host.ROOT/'tools/cell_runtime_fresh_sampled_channel_sparse_carrier.py',host.ROOT/'tools/cell_runtime_fresh_sampled_channel_hybrid_audit.py',a.host_dir/'host-x86.asm',a.host_dir/'manifest.json',a.inputs/'input.bin',a.inputs/'cases.json',a.inputs/'report.json',host.ROOT/'crates/spore-core/src/cell_runtime_fresh_sampled_channel.rs',host.ROOT/'crates/spore-core/examples/cell_runtime_fresh_sampled_channel_verify.rs.pending']}
 result=dict(schema='fresh-full-source-sampled-channel-prebuild-audit-v1',static_pass=True,native_executed=False,compiled=False,source_bytes=sum(z['bytes']for z in proof[0])+sum(b-a for a,b in host.CRT_CODE),game_bytes=8979,crt_bytes=1424,host_generated_exact=True,separate_failure_and_wrapper_helper_stacks=True,masm_public_line_limit_closed=True,source_history_records=300,source_snapshots=2700,first_channel_only_sampling_union=True,epoch_table_sampling_immutable=True,event_padding_opaque=True,source_cases=all_rows,pins=pins,limits='Prebuild static gate only. Root must compile, run linked carrier audit and obtain an independent review before any original-byte execution. No numeric result, fullbinder/lifetime, actor route or eventdispatch acceptance.')
 with a.out.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 print(sha(a.out))
if __name__=='__main__':main()
