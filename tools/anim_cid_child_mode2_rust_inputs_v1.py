"""Project 64 reviewed source-body/synthetic-stage fixtures for a Rust comparison.
No game/native execution or numeric expected output is generated.
"""
import argparse,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
FIX=ROOT/'target/animation/cid-child-mode2-fixtures-v1'
BODY=ROOT/'target/animation/cid-node-init-actual-body-inputs-v1'
NATIVE=ROOT/'target/animation/cid-child-mode2-native-execution-v3'
EXPECTED_NATIVE='d87b75038dda6ca3b58282b204b9e7390bd772aa54b9e58111c355688d8bc4e6'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def word(b,o):return struct.unpack_from('<I',b,o)[0]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve())
 fixture=json.loads((FIX/'fixtures.json').read_text());fm=json.loads((FIX/'manifest.json').read_text());body=json.loads((BODY/'manifest.json').read_text());index=json.loads((BODY/'index.json').read_text());native=json.loads((NATIVE/'report.json').read_text())
 assert fixture['synthetic_stage_state'] and len(fixture['fixtures'])==64 and sha(FIX/'fixtures.json')==fm['fixtures_sha256']
 assert sha(BODY/'input.bin')==body['input_sha256'] and sha(BODY/'expected.bin')==body['expected_sha256']
 assert native['native_executed'] and native['synthetic_stage'] and native['fixtures']==64 and not native['rust_comparison'] and native['native_sha256']==EXPECTED_NATIVE and sha(NATIVE/'native-output.bin')==EXPECTED_NATIVE
 raw=(BODY/'input.bin').read_bytes();out=bytearray();rows=[]
 for ordinal,case in enumerate(fixture['fixtures']):
  info=index[case['body']];assert info['body']==case['body'] and info['key']==case['key'] and info['source_sha256']==case['source_sha256']
  off=info['input_offset'];packet=raw[off:off+info['input_bytes']];assert hashlib.sha256(packet[32:]).hexdigest()==info['block_projection_sha256']
  n=info['count'];parent=case['parent_ordinal'];child=case['first_child_ordinal'];count=case['source_child_count'];assert parent<n and 1<=count<=255 and child+count<=n
  pb=packet[32+1128*parent:32+1128*(parent+1)];cb=packet[32+1128*child:32+1128*(child+1)];assert word(pb,0x204)==child and word(pb,0x208)==count and word(cb,0x1fc)==parent
  for block,at,name in [(pb,0x344,'parent_definition_344_bits'),(cb,0x128,'child_definition_128_bits'),(cb,0x350,'child_definition_350_bits')]:
   assert [word(block,at+4*i)for i in range(len(case[name]))]==case[name]
  s=case['synthetic_stage'];assert s['cid_scale_bits']==0x3f800000 and s['mode_238']==2 and s['flags_1ac_bytes']==[0,0,255,255] and s['other_child_flags_low3']==4 and not any(s['initial_position_bits']+s['initial_quaternion_bits']+s['initial_counters'])
  values=[count]+case['parent_definition_344_bits']+case['child_definition_128_bits']+case['child_definition_350_bits']+s['child_quaternion_19c_bits']+s['child_position_1c8_bits'];assert len(values)==18 and all((v&0x7f800000)!=0x7f800000 for v in values[1:])
  out+=struct.pack('<18I',*values);rows.append(dict(ordinal=ordinal,body=case['body'],key=case['key'],parent_ordinal=parent,first_child_ordinal=child,source_child_count=count))
 assert len(out)==64*72 and (NATIVE/'native-output.bin').stat().st_size==64*60
 a.out.mkdir(parents=True,exist_ok=False);(a.out/'input.bin').write_bytes(out);(a.out/'index.json').write_text(json.dumps(rows,indent=2)+'\n')
 pins=[Path(__file__),FIX/'fixtures.json',FIX/'manifest.json',BODY/'input.bin',BODY/'expected.bin',BODY/'index.json',BODY/'manifest.json',NATIVE/'report.json',NATIVE/'native-output.bin',ROOT/'crates/spore-core/src/anim_cid_child_mode2_v1.rs']
 m=dict(schema='cid-child-mode2-rust-inputs-v1',cases=64,input_bytes_per_case=72,input_sha256=sha(a.out/'input.bin'),index_sha256=sha(a.out/'index.json'),native_sha256=EXPECTED_NATIVE,synthetic_stage=True,baseline='Each record packs 18 little-endian u32: source child count, parent Definition+344[3], child Definition+128[4], child Definition+350[3], synthetic node+19C quaternion[4], synthetic node+1C8 position[3]. Parent/child topology and source bits rechecked; no dynamic pose source claim.',comparison='Full 60-byte native record, including counters, aggregate and node flags, node+210, x87 CW/SW and full MXCSR. Reset 027F/1F80 for each row; exact differences are failures.',native_comparison_executed=False,rust_executed=False,pins={str(f.resolve()):sha(f)for f in pins});(a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps(m))
if __name__=='__main__':main()
