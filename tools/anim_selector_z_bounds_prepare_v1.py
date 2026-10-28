"""Source-only actual finalized-body Z publisher cohort; no native execution."""
import argparse,hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());a.out.mkdir(parents=True,exist_ok=False)
 upstream=ROOT/'target/animation/nonroot-selector-inputs-v3';raw=(upstream/'input.bin').read_bytes();idx=json.loads((upstream/'body-index.json').read_text());pos=0;groups=[];out=bytearray();policies=[0x7f|pc|rc for pc in(0,0x200,0x300)for rc in(0,0x400,0x800,0xc00)]
 for body in idx:
  h=struct.unpack_from('<8I',raw,pos);pos+=32;magic,n,q,cw,bodyid,*reserved=h;assert bodyid==body['body'] and n==body['finalized_blocks'] and not any(reserved)
  records=raw[pos:pos+n*324];pos+=n*324+q*32
  for k in range(n):
   fields=struct.unpack_from('<39I',records,k*324+168)
   for j in range(7,10):assert fields[j]&0x7fffffff<=0x49800000
   for j in range(10,14):assert fields[j]&0x7fffffff<=0x40000000
   for j in range(18,21):assert 0<fields[j]<=0x49800000
  offset=len(out);out+=struct.pack('<8I',0x315a4242,n,12,bodyid,0,0,0,0)+records+struct.pack('<12I',*policies)
  groups.append(dict(body=bodyid,native_case=body['native_case'],offset=offset,bytes=len(out)-offset,records_sha256=hashlib.sha256(records).hexdigest(),package=body['package'],key=body['key'],source_sha256=body['source_sha256'],blocks=n))
 assert pos==len(raw) and len(groups)==1322 and sum(x['blocks']for x in groups)==46256
 (a.out/'input.bin').write_bytes(out);(a.out/'body-index.json').write_text(json.dumps(groups,indent=2)+'\n')
 pins={str(x.relative_to(ROOT)):sha(x)for x in[upstream/'input.bin',upstream/'body-index.json',upstream/'manifest.json',Path(__file__),ROOT/'crates/spore-core/src/anim_selector_z_bounds.rs',ROOT/'tools/anim_selector_z_bounds_native_probe_v1.py',ROOT/'target/animation/selector-Z-producer-native-v1/source-closure.json']}
 report=dict(schema='actual-finalized-body-Z-fieldpublisher-inputs-v1',bodies=1322,blocks=46256,policies=policies,records=1322*12,input_sha256=sha(a.out/'input.bin'),pins=pins,scope='Actual accepted source-derived finalized fields; supplied 12 x87 policies. No actor association, live full definition, or whole wrapper claim.',expected_model_bytes=44,expected_reply_bytes=52,native_executed=False,rust_compiled=False,admitted=False)
 (a.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
