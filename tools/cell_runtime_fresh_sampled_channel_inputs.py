"""Derive actual source histories for nonzero first-channel sampling; no execution."""
import argparse,json,struct,hashlib,shutil
from pathlib import Path
import cell_runtime_fresh_sampled_channel_native_probe as host
import cell_runtime_fresh_loop_inputs as clock
ROOT=host.ROOT
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def word(b,a):return struct.unpack_from('<I',b,a)[0]
def domain(b,case,duration):
 assert len(b)==256 and word(b,16)==16 and word(b,44)==word(b,52)==1 and word(b,48)==word(b,56)==0
 assert word(b,72)==case and word(b,80)==1 and b[84:128]==bytes(44)
 legacy=bytearray(b);legacy[44:60]=bytes(16);legacy[72:128]=bytes(56)
 return clock.domain(legacy,duration)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--host-dir',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 base=ROOT/'target/source-cell/runtime-fresh-loop-inputs-v1';cases=json.loads((base/'cases.json').read_text());raw=(base/'input.bin').read_bytes();assert len(raw)==len(cases)*256==76800
 m=json.loads((a.host_dir/'manifest.json').read_text());assert sha(a.host_dir/'host-x86.asm')==m['host_source_sha256']
 a.out.mkdir(parents=True,exist_ok=False);shutil.copytree(base/'clips',a.out/'clips');indices={str(c['path'].relative_to(base)):i for i,c in enumerate(host.source_cases())};rows=[];out=bytearray()
 for i,row in enumerate(cases):
  b=bytearray(raw[i*256:(i+1)*256]);case=indices[row['clip']]
  for at,v in[(16,16),(44,1),(52,1),(72,case),(76,[0,1,0xfffffffe,0xffffffff,0x12345678][i%5]),(80,1)]:struct.pack_into('<I',b,at,v)
  duration=struct.unpack('<d',bytes.fromhex(row['duration_bits'])[::-1])[0]
  domain(b,case,duration);out+=b;rows.append(dict(row,event_capacity=16,source_case=case,wrapper_serial_seed=f'{word(b,76):08X}'))
 (a.out/'input.bin').write_bytes(out);(a.out/'cases.json').write_text(json.dumps(rows,indent=2)+'\n');shutil.copyfile(base/'cases.tsv',a.out/'cases.tsv')
 valid=out[:256];protocols={}
 for name,at,v in [('capacity',16,1),('first_lookup_mask',44,0),('second_sample_mask',52,0),('source_case',72,5),('rotation_policy',80,0),('reserved_tail',84,1),('source_flags',24,word(valid,24)^1),('source_count',36,word(valid,36)+1)]:
  b=bytearray(valid);struct.pack_into('<I',b,at,v);path=a.out/(name+'.bin');path.write_bytes(b);protocols[name]=sha(path)
 for name in ['cw_unmasked','cw_reserved','reserved_header','fresh_mode','initial_event_count','interval_zero','frames_one','channel_zero','channel_over_bound','step_count','crt_policy','property_mode','owner_boundary','step_mode','step_reserved','negative_absolute','nonfinite_absolute','counter_indefinite_domain','negative_first_delta','pending_count']:
  old=(base/(name+'.bin')).read_bytes();b=bytearray(valid)
  for at,(x,y)in enumerate(zip(raw[:256],old)):
   if x!=y:b[at]=y
  path=a.out/(name+'.bin');path.write_bytes(b);protocols[name]=sha(path)
 pins={str(q):sha(q)for q in [Path(__file__),base/'report.json',base/'input.bin',base/'cases.json',a.host_dir/'host-x86.asm',a.host_dir/'manifest.json',ROOT/'crates/spore-core/src/cell_runtime_fresh_sampled_channel.rs',ROOT/'crates/spore-core/examples/cell_runtime_fresh_sampled_channel_verify.rs.pending']}
 pins.update({str(q):sha(q)for q in (a.out/'clips').iterdir()})
 report=dict(schema='fresh-full-source-first-channel-sampling-inputs-v1',records=300,snapshots=2700,contexts=12,source_clips=5,input_sha256=sha(a.out/'input.bin'),pins=pins,protocols=protocols,no_execution=True,source_history='Exact accepted clock histories, source resource bytes unchanged; capacity16, channel0 lookup/sample masks, isolated wrapper serial and source-image rotation policy explicit.')
 (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(300)
if __name__=='__main__':main()
