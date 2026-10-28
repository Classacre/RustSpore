"""Prepare deterministic guarded full-selector protocol inputs; never execute."""
import argparse,json,struct,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
DEFAULT=ROOT/'target/animation/nonroot-selector-inputs-v3'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--inputs',type=Path,default=DEFAULT);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());bodies=json.loads((a.inputs/'body-index.json').read_text());chosen=None
 with(a.inputs/'input.bin').open('rb')as inp,(a.inputs/'expected1032.bin').open('rb')as exp:
  while h:=inp.read(32):
   magic,n,q,cw,bid,*_=struct.unpack('<8I',h);records=inp.read(n*324);queries=inp.read(q*32)
   for qi in range(q):
    reply=exp.read(1032)
    if chosen is None and struct.unpack_from('<I',reply,8)[0]>0 and qi%2==1:chosen=(h,records,queries[qi*32:(qi+1)*32],qi,reply,n,bid)
   if chosen:break
 assert chosen;h,records,query,qi,expected,n,bid=chosen;header=bytearray(h);struct.pack_into('<I',header,8,1);reference=bytes(header)+records+query;a.out.mkdir(parents=True,exist_ok=False);(a.out/'reference-input.bin').write_bytes(reference);(a.out/'reference-expected1032.bin').write_bytes(expected)
 mutations=[(0,0,'bad_magic'),(4,0,'count0'),(4,256,'count256'),(8,0,'queries0'),(8,257,'queries257'),(12,0x17f,'reserved_precision'),(20,1,'reserved_header'),(32,struct.unpack_from('<I',records,0)[0]^1,'presence_mask_mismatch'),(32+180,0,'root_parent_self'),(32+196,0x7fc00000,'NaN_center')];start=32+n*324
 mutations += [(start,struct.unpack_from('<I',query,0)[0]|0x40,'unsupported_Z'),(start,0x1800,'unsupported_hierarchy'),(start+8,1,'alternate_capability'),(start+12,1,'reserved_selector'),(start+16,2,'mirror2'),(start+20,256,'capacity256'),(start+24,1,'expansion_disabled')]
 rows=[]
 for at,value,name in mutations:
  raw=bytearray(reference);struct.pack_into('<I',raw,at,value);(a.out/(name+'.bin')).write_bytes(raw);rows.append(dict(name=name,expected_exit=71,expected_output_bytes=0,input_sha256=hashlib.sha256(raw).hexdigest()))
 for capacity in(0,1,255):
  raw=bytearray(reference);struct.pack_into('<I',raw,start+20,capacity);(a.out/f'capacity{capacity}-input.bin').write_bytes(raw)
 report=dict(schema='actual-nonroot-selector-protocol-preparation-v1',native_executed=False,tool_sha256=sha(__file__),upstream_manifest_sha256=sha(a.inputs/'manifest.json'),reference=dict(body=bid,query_id=qi,source_body=bodies[bid],expected_count=struct.unpack_from('<I',expected,8)[0],bytes=len(reference),input_sha256=hashlib.sha256(reference).hexdigest(),expected_sha256=hashlib.sha256(expected).hexdigest()),negative_cases=rows,positive_capacity_cases=[0,1,255],stream='empty->exit0; partialheader/payload->71/empty; complete reference group+partial nextgroup->71/firstreply retained; repeatgroup->two identical replies',scope='Genuine source finalizer body with positive capability match; mirror1 enters original159Bmirror and bounded CRTcopy provider. No actor association.')
 (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
if __name__=='__main__':main()
