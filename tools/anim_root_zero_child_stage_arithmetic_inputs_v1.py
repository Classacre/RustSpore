"""Owned arithmetic-only supplement to accepted original zero-child stage.
No native/Rust execution or expected projection generation. Seed topology is immutable.
"""
import argparse,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
CONTROLS=[0x7f|p|r for p in(0,0x200,0x300) for r in(0,0x400,0x800,0xc00)]
THRESHOLDS=[('below',0x358637bc),('equal',0x358637bd),('above',0x358637be)]
CONTEX=[dict(name='commit_quarter_history',skip=0,smoothing=.25),dict(name='skip_threequarter_history',skip=1,smoothing=.75)]
# These are owned inputs, not expected outputs or original actor poses.
FAMILIES=[dict(name='alternate_only',pw=0.,apw=.5,rw=0.,arw=.5,ps=1,asign=1),dict(name='same_hemisphere_below_one',pw=.25,apw=.25,rw=.25,arw=.25,ps=1,asign=1),dict(name='opposite_hemisphere_below_one',pw=.25,apw=.25,rw=.25,arw=.25,ps=1,asign=-1),dict(name='same_negative_rest_hemisphere',pw=.25,apw=.25,rw=.25,arw=.25,ps=-1,asign=-1),dict(name='opposite_negative_rest_hemisphere',pw=.25,apw=.25,rw=.25,arw=.25,ps=-1,asign=1),dict(name='total_weight_exact_one',pw=.5,apw=.5,rw=.5,arw=.5,ps=1,asign=1),dict(name='total_weight_above_one',pw=.75,apw=.75,rw=.75,arw=.75,ps=1,asign=1),dict(name='asymmetric_opposite_weights',pw=.25,apw=.75,rw=.125,arw=.5,ps=1,asign=-1)]
P=[.5,-.25,.75];AP=[-.25,.5,.125];Q=[.25,-.5,.125,.75];AQ=[.125,.25,.375,.5]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def word(b,o):return struct.unpack_from('<I',b,o)[0]
def put(b,o,x):struct.pack_into('<I',b,o,x)
def bits(f):return struct.unpack('<I',struct.pack('<f',f))[0]
def f(v):return struct.unpack('<f',struct.pack('<I',v))[0]
def allowed():return set(range(0,4))|set(range(12,24))|set(range(32+0x58,32+0xa0))
def record(seed,family,cw,dt,control):
 raw=bytearray(seed);put(raw,0,cw);put(raw,12,dt);put(raw,16,bits(control['smoothing']));put(raw,20,control['skip'])
 for at,values in [(0x58,[v*family['pw'] for v in P]),(0x7c,[v*family['apw'] for v in AP]),(0x68,[v*family['rw']*family['ps'] for v in Q]),(0x8c,[v*family['arw']*family['asign'] for v in AQ])]:
  for i,v in enumerate(values):put(raw,32+at+i*4,bits(v))
 for at,name in [(0x64,'pw'),(0x88,'apw'),(0x78,'rw'),(0x9c,'arw')]:put(raw,32+at,bits(family[name]))
 assert all(a==b for i,(a,b) in enumerate(zip(raw,seed)) if i not in allowed())
 assert word(raw,4)==0 and word(raw,8)==1 and word(raw,1252)==0 and raw[32:48]==bytes(16)
 assert all(abs(f(word(raw,i)))<=4 for i in [12,16,24]+list(range(48,88,4))+list(range(92,192,4))+list(range(1000,1028,4)))
 return bytes(raw)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());a.out.mkdir(parents=True,exist_ok=False)
 original=ROOT/'target/animation/root-zero-child-stage-inputs-v3/input.bin'
 with original.open('rb') as stream:seed=stream.read(1256)
 accepted=ROOT/'target/animation/root-zero-child-stage-native-run-v1/rust-comparison-v1.json';approval=json.loads(accepted.read_text());assert approval['accepted'] and approval['records']==66636
 (a.out/'seed1256.bin').write_bytes(seed);rows=[]
 with(a.out/'input.bin').open('xb') as output:
  for family in FAMILIES:
   for control in CONTEX:
    for label,dt in THRESHOLDS:
     for cw in CONTROLS:
      raw=record(seed,family,cw,dt,control);output.write(raw);rows.append(dict(row=len(rows),family=family['name'],control=control['name'],dt_relation=label,dt_bits=f'{dt:08X}',cw=f'{cw:04X}',input_sha256=hashlib.sha256(raw).hexdigest(),owned_arithmetic=True,actor_association=False))
 (a.out/'index.json').write_text(json.dumps(rows,indent=2)+'\n');(a.out/'families.json').write_text(json.dumps(FAMILIES,indent=2)+'\n');(a.out/'controls.json').write_text(json.dumps(CONTEX,indent=2)+'\n')
 paths=[Path(__file__),original,accepted,ROOT/'target/animation/root-zero-child-stage-native-run-v1/native-v1/report.json',ROOT/'target/animation/root-zero-child-stage-native-v3/final-preparation-review.json',ROOT/'target/animation/root-zero-child-stage-promotion-v1/anim_root_zero_child_stage.rs',ROOT/'target/animation/root-zero-child-stage-promotion-v1/anim_root_zero_child_stage_verify.rs.pending',ROOT/'crates/spore-core/src/anim_root_zero_child_stage.rs',ROOT/'crates/spore-core/examples/anim_root_zero_child_stage_verify.rs']
 report=dict(schema='zero-child-stage-owned-arithmetic-inputs-v1',records=len(rows),families=len(FAMILIES),contexts=len(CONTEX),threshold_neighbors=3,x87_controls=12,seed='first admitted original1256B record',seed_sha256=sha(a.out/'seed1256.bin'),allowed_changed_byte_ranges=[[0,4],[12,24],[32+0x58,32+0xa0]],graph_topology_definition_and_reserved_bytes_unchanged=True,input_float_magnitude_limit=4,expected_numeric_projection_generated=False,new_native_execution=False,rust_execution=False,scope='Owned alternate position/rotation,hemisphere,weight clamp and delta threshold supplement. All inputs synthetic arithmetic modifications of one admitted seed; not new original source pose/actor/body observations.',pins={str(path.relative_to(ROOT)):sha(path) for path in paths},files={path.name:sha(path) for path in a.out.iterdir() if path.is_file()})
 (a.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(len(rows),sha(a.out/'input.bin'))
if __name__=='__main__':main()
