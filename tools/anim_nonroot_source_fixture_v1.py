"""Prepare one source-derived nonroot input; never execute a host or compile."""
import csv, hashlib, json, struct
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from cell_runtime_curves_fixture import package_index, source_payload
from anim_root_pose_curve_fixture import make_window

ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'target/animation/nonroot-source-fixture-v1'
INPUT=ROOT/'target/animation/ungrouped-binding-builder-inputs-v1/input.bin'
REPLY=ROOT/'target/animation/ungrouped-binding-builder-native-run-v2/native-v1/corpus-output.bin'
CASES=ROOT/'target/animation/ungrouped-binding-builder-inputs-v1/cases.json'
POS=ROOT/'target/animation/source-position-fixture-v1'
PACKAGE=Path('D:/Games/SPORE Collection/Data/PatchData.package')
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1<<20),b''):h.update(b)
 return h.hexdigest()
def word(b,o):return struct.unpack_from('<I',b,o)[0]
def exact(f,n):
 b=f.read(n)
 if len(b)!=n:raise ValueError('truncated input')
 return b

def main():
 assert sha(INPUT)=='d9d009821bd97d0217af832b3c90d6d5012a53d4ac80451f31418a9b61696319'
 assert sha(REPLY)=='5b5d790761cb63e30ce4b953f2062b23a483bf1eb5fbac46b0d689d9d6eff4d9'
 case=json.loads(CASES.read_text())[0]
 assert (case['body'],case['clip'],case['input_offset'],case['blocks'],case['channels'])==(603,0,0,20,10)
 with INPUT.open('rb') as f:
  h=exact(f,32); blocks=[exact(f,324) for _ in range(word(h,4))]; channels=[]
  for i in range(word(h,8)):
   header=exact(f,80);descs=[exact(f,32) for _ in range(word(header,76))];channels.append((header,descs))
  assert f.tell()==case['input_bytes']
 with REPLY.open('rb') as f: reply=exact(f,114292)
 assert word(reply,12)==case['record_count']==14
 row=reply[44+5*16:44+6*16]; assert (row[12],row[13],row[14],word(row,8))==(19,8,255,14)
 header,ds=channels[8]; assert word(header,0)&1==0
 assert ds[1][4:8]==b'POS\0' and word(ds[1],0)==0x71 and ds[2][4:8]==b'ROT\0' and word(ds[2],0)==0x32
 assert all(word(row,8)&(1<<i) for i in (1,2))
 before=sha(PACKAGE);index,_=package_index(PACKAGE);raw=source_payload(PACKAGE,index[(0x71d0a3d5,0xee17c6ad,0)])
 assert sha(PACKAGE)==before
 assert hashlib.sha256(raw).hexdigest()==case['source_clip_sha256']=='9daf9fe5bce32a039ff4a4de2ae70f9090b053b570c17da7d9141d61f83ee10a'
 start=word(raw,word(raw,328)+8*4); assert raw[start:start+4]==b'CHAN'
 n,at=struct.unpack_from('<2I',raw,start+220)
 assert n==len(ds) and [raw[at+i*32:at+(i+1)*32] for i in range(n)]==ds
 assert raw[start+136:start+212]==header[:76]
 inv=json.loads((POS/'inventory.json').read_text());cmp=json.loads((POS/'rust-comparison.json').read_text())
 assert cmp['byte_identical'] and cmp['full_source_selection']
 assert sha(POS/'output.bin')==inv['reply_report']['output_sha256']
 assert sha(POS/'expected.tsv')==cmp['expected_sha256']==inv['reply_report']['expected_sha256']
 with (POS/'expected.tsv').open(newline='') as f:
  matches=[x for x in csv.DictReader(f,delimiter='\t') if x['package']=='Data/PatchData.package' and x['instance']=='0000000071D0A3D5' and x['channel']=='8' and x['frame_bits']=='00000000' and x['cw']=='027F']
 assert len(matches)==1; item=matches[0]
 with (POS/'output.bin').open('rb') as f:f.seek(int(item['native_case_ordinal'])*124);pos=exact(f,124)
 bits=struct.unpack_from('<4I',pos,20);assert bits==tuple(int(item[k],16) for k in ('x_bits','y_bits','z_bits','weight_bits'))
 rot, detail=make_window(raw,start,struct.unpack('<8I',ds[2]),0)
 OUT.mkdir(parents=True,exist_ok=False)
 data={'root-definition-projection.bin':blocks[0], 'selected-definition-projection.bin':blocks[19], 'binding-row.bin':row, 'channel-header.bin':header, 'descriptors.bin':b''.join(ds), 'source-anim.bin':raw, 'accepted-pos-reply.bin':pos, 'pending-rot-request.bin':rot}
 for name,value in data.items():(OUT/name).write_bytes(value)
 pins={str(p.relative_to(ROOT)):sha(p) for p in [Path(__file__),INPUT,REPLY,CASES,POS/'inventory.json',POS/'rust-comparison.json',POS/'expected.tsv',POS/'output.bin']}
 report={'schema':'nonroot-source-fixture-v1','native_executed':False,'compiled':False,'case':case,'selected_pose_ordinal':19,'wrapper_ordinal':8,'record_ordinal':5,'binding_words':[f'{word(row,i):08X}' for i in range(0,16,4)],'source_anim_channel':8,'frame_bits':'00000000','pos_native_case':int(item['native_case_ordinal']),'pos_sample_bits':[f'{x:08X}' for x in bits],'rotation_status':'pending original sampler; request only, no result claimed','rotation_window':detail,'package_sha256_before_after':before,'package_stability':'before/after hashes do not prove no transient replacement; extracted payload hash pinned','context':{'blend_bits':'3F800000','prior_accumulator':'explicit all-zero proposed only','reference':'identity proposed only','scale':'explicit 1 proposed only','cw':'027F proposed','mxcsr':'1F80 proposed'},'boundaries':['body records are 324-byte projections, not complete source definition blocks','actor and resource association unestablished','not a native nonroot conversion or stage result','no actual animation timing, playback or visible motion'],'files':{name:hashlib.sha256(value).hexdigest() for name,value in data.items()},'pins':pins}
 (OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps({'output':str(OUT),'frame_bits':report['frame_bits'],'pos_sample_bits':report['pos_sample_bits'],'rotation_status':report['rotation_status'],'files':report['files']},indent=2))
if __name__=='__main__':main()
