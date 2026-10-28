"""Build exploratory mode-2 child contexts from real body projections, without executing source code.

Explicit synthetic stage pose/flags/mode are not observed animation state.
"""
import argparse,hashlib,json,math,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
INPUT=ROOT/"target/animation/cid-node-init-actual-body-inputs-v1"
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def word(b,o):return struct.unpack_from("<I",b,o)[0]
def fbits(b,start,n):return [word(b,start+4*i) for i in range(n)]
def finite(bits):return all(math.isfinite(struct.unpack("<f",struct.pack("<I",v))[0]) for v in bits)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--out",type=Path,required=True);p.add_argument("--limit",type=int,default=64);a=p.parse_args();a.out.resolve().relative_to((ROOT/"target").resolve());assert 1<=a.limit<=1322
 idx=json.loads((INPUT/"index.json").read_text());manifest=json.loads((INPUT/"manifest.json").read_text());assert len(idx)==1322 and sha(INPUT/"input.bin")==manifest["input_sha256"] and sha(INPUT/"expected.bin")==manifest["expected_sha256"]
 raw=(INPUT/"input.bin").read_bytes();expected=(INPUT/"expected.bin").read_bytes();ep=0;fixtures=[]
 for body in idx:
  n=body["count"];off=body["input_offset"];packet=raw[off:off+body["input_bytes"]];projection=expected[ep:ep+body["projection_bytes"]];ep+=body["projection_bytes"]
  assert hashlib.sha256(packet[32:]).hexdigest()==body["block_projection_sha256"] and hashlib.sha256(projection).hexdigest()==body["projection_sha256"]
  blocks=[packet[32+i*1128:32+(i+1)*1128] for i in range(n)]
  for parent,b in enumerate(blocks):
   child=word(b,0x204);count=word(b,0x208)
   if not count or child+count>n or word(blocks[child],0x1fc)!=parent:continue
   pb=fbits(b,0x344,3);cb=blocks[child];cq=fbits(cb,0x128,4);cp=fbits(cb,0x350,3)
   if not finite(pb+cq+cp) or not any(v&0x7fffffff for v in pb+cp):continue
   fixtures.append(dict(body=body["body"],key=body["key"],source_sha256=body["source_sha256"],parent_ordinal=parent,first_child_ordinal=child,source_child_count=count,parent_definition_344_bits=pb,child_definition_128_bits=cq,child_definition_350_bits=cp,synthetic_stage=dict(cid_scale_bits=0x3f800000,flags_1ac_bytes=[0,0,255,255],mode_238=2,child_quaternion_19c_bits=[0,0,0x3ec3ef15,0x3f6c835e],child_position_1c8_bits=[0x3e000000,0xbe800000,0x3f000000],other_child_flags_low3=4,initial_position_bits=[0,0,0],initial_quaternion_bits=[0,0,0,0],initial_counters=[0,0,0]),actual_actor_association=False))
   break
  if len(fixtures)>=a.limit:break
 assert len(fixtures)==a.limit,(len(fixtures),a.limit)
 a.out.mkdir(parents=True,exist_ok=False);path=a.out/"fixtures.json"
 path.write_text(json.dumps(dict(schema="cid-child-mode2-fixtures-v1",native_executed=False,source_body_contexts=True,synthetic_stage_state=True,meaning="One active first child in a genuine parent/child body topology; dynamic stage state is deliberately supplied, not recovered from animation",fixtures=fixtures),indent=2)+"\n")
 report=dict(schema="cid-child-mode2-fixtures-manifest-v1",fixtures=len(fixtures),fixtures_sha256=sha(path),native_executed=False,rust_executed=False,actual_actor_association=False,coverage="Exploratory contexts only; no native result, expected output, or 9F4730 acceptance",pins={str(p):sha(p)for p in [Path(__file__),INPUT/"input.bin",INPUT/"expected.bin",INPUT/"index.json",INPUT/"manifest.json"]})
 (a.out/"manifest.json").write_text(json.dumps(report,indent=2)+"\n");print(json.dumps(report))
if __name__=="__main__":main()
