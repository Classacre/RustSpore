"""Prepare boundedfullfreshconstructor/reset inputpacket; never execute."""
import argparse,json,struct,hashlib
from pathlib import Path
from cell_runtime_curves_fixture import CONTEXTS
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def request(serial=0,cw=0x027f,mode=0,count=0,capacity=0):
 raw=struct.pack('<IHHIII',serial,cw,0,mode,count,capacity)+bytes(44);assert len(raw)==64;return raw
def validate(raw):
 assert len(raw)==64;serial,cw,reserved,mode,count,capacity=struct.unpack_from('<IHHIII',raw)
 assert cw in CONTEXTS and reserved==0 and mode in[0,1]and 0<=count<=capacity<=16 and raw[20:]==bytes(44)
 if mode==0:assert count==capacity==0
 return True
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());m=json.loads((a.host_dir/'manifest.json').read_text());assert sha(a.host_dir/'host-x86.asm')==m['host_source_sha256'];a.out.mkdir(parents=True,exist_ok=False);cases=[]
 with(a.out/'input.bin').open('xb')as f:
  def emit(raw,family):validate(raw);f.write(raw);cases.append(dict(case=len(cases),family=family,raw_hex=raw.hex()))
  for cw in CONTEXTS:
   for serial in[0,1,0xfffffffe,0xffffffff]:
    emit(request(serial,cw),'fresh_constructor')
    for count,capacity in[(0,0),(0,16),(1,1),(8,16),(16,16)]:emit(request(serial,cw,1,count,capacity),'fresh_then_bounded_pending_reset')
 assert len(cases)==288;(a.out/'cases.json').write_text(json.dumps(cases,indent=2)+'\n');valid=request(mode=1,count=8,capacity=16);protocols={}
 invalid={'cw_unmasked':(4,struct.pack('<H',0x027e)),'cw_reserved_precision':(4,struct.pack('<H',0x017f)),'reserved_header':(6,struct.pack('<H',1)),'invalid_mode':(8,struct.pack('<I',2)),'capacity_out_of_bounds':(16,struct.pack('<I',17)),'count_exceeds_capacity':(12,struct.pack('<I',17)),'fresh_nonempty_vector':(8,struct.pack('<I',0)),'reserved_tail':(20,struct.pack('<I',1))}
 for name,(at,value)in invalid.items():raw=bytearray(valid);raw[at:at+len(value)]=value;path=a.out/(name+'.bin');path.write_bytes(raw);protocols[name]=sha(path)
 paths=[Path(__file__),ROOT/'tools/cell_runtime_fresh_aid_native_probe.py',ROOT/'tools/cell_runtime_clip_metadata_native_probe.py',ROOT/'tools/cell_runtime_clock_native_probe.py',ROOT/'crates/spore-core/src/cell_runtime_fresh_aid.rs',ROOT/'crates/spore-core/examples/cell_runtime_fresh_aid_verify.rs.pending',a.host_dir/'manifest.json',a.host_dir/'host-x86.asm']
 paths +=[ROOT/f'target/rea/20261027-source-runtime-v4/{n}-analyze_function.json'for n in[56,57,58,59,65,66]]
 (a.out/'report.json').write_text(json.dumps(dict(schema='source-cell-full-fresh-aid-inputs-v1',cases=288,contexts=len(CONTEXTS),protocols=protocols,input_sha256=sha(a.out/'input.bin'),cases_sha256=sha(a.out/'cases.json'),pins={str(p):sha(p)for p in paths},scope='Full1049originalBfreshconstructor/reset; explicitglobalserialsnapshot,12CW,4serialseedsincludingwrap,5boundedpendingvector shapes; noresourcebinding/recycledstate/clockadvance/eventdispatch/liveactorclaim.',no_execution=True),indent=2)+'\n');print(288)
if __name__=='__main__':main()
