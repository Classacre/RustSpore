"""Source-only full9A1E00 entry cohort/reference using genuine native selections."""
import argparse,json,struct,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
COHORT=ROOT/'target/animation/nonroot-selector-Z-inputs-v1-complete'
NATIVE=ROOT/'target/animation/nonroot-selector-Z-native-run-v1/native-v1'
WHOLE=ROOT/'target/animation/root-whole-builder-followup-v1'
BODIES=[603,1182,472,833,895]
CAPACITY=7140;PROJECTION=44+CAPACITY*16
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def word(b,o):return struct.unpack_from('<I',b,o)[0]
def value(b,o):return struct.unpack_from('<f',b,o)[0]
def construct(ch,ci,i,mirror,claims,passid):
 mask=0;admitted=False;flags=2 if mirror else 0;owner=ch['ownerAE']
 for di,d in enumerate(ch['descriptors']):
  slot=d[4]
  if slot>=9 or claims[slot]not in(255,owner):continue
  admitted=True
  if d[0]&0x10==0:continue
  claims[slot]=owner;mask|=1<<di
  if d[0]&15==1:flags|=0x20
 fallback=mask==0 and admitted and ch['metadataAC']&8!=0
 if mask==0 and not fallback:return None
 if fallback:flags|=8
 return[1<<passid,0,mask,i|(ci<<8)|(255<<16)|(flags<<24)],fallback

def reference(blocks,channels,selected):
 rows=[];zero=fallbacks=0;live=components=0
 for passid in(0,1):
  claims=[[255]*9 for _ in blocks]
  for ci,ch in enumerate(channels):
   for i in selected(ci,passid):
    mode=ch['metadataAC']&0x32;assert mode in(0,0x10)
    mirror=bool(passid)if mode==0 else value(blocks[i],168+7*4)<=0
    result=construct(ch,ci,i,mirror,claims[i],passid)
    if result is None:continue
    row,fallback=result;zero+=i==0;fallbacks+=fallback;live|=1<<ci
    if row[2]&0xffffff:components|=1<<ci
    rows.append(row)
 search=zero
 for i in range(zero):
  if rows[i][3]&255:
   while search<len(rows)and rows[search][3]&255:search+=1
   if search<len(rows):rows[i],rows[search]=rows[search],rows[i]
 assert len(rows)<=CAPACITY
 return rows,[live&0xffffffff,live>>32,components&0xffffffff,components>>32,0,2 if rows else 1,1],zero,fallbacks

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());a.out.mkdir(parents=True,exist_ok=False)
 assert json.loads((NATIVE/'rust-comparison-v1.json').read_text())['accepted']and sha(NATIVE/'native-projection1032.bin')=='4cdceca9ffb0bb2d28b026cd13919a7bc77333734b487b0f6757eabb00a52259'
 idx=json.loads((COHORT/'body-index.json').read_text());selectors=json.loads((COHORT/'selector-table.json').read_text());mapping={tuple(s):i for i,s in enumerate(selectors)};keys={tuple(x['key'])for x in json.loads((WHOLE/'first-root-static-retention-v3.json').read_text())['candidates']};clips=[c for c in json.loads((WHOLE/'all818-whole-channel-prepared-v2.json').read_text())if tuple(c['key'])in keys];assert len(clips)==217 and sum(len(c['channels'])for c in clips)==1861
 # Fresh native output alone determines useful bodycoverage; no synthetic body.
 coverage=[]
 with(NATIVE/'native-projection1032.bin').open('rb')as native:
  for bi in range(1322):coverage.append({q for q in range(122)if word(native.read(1032),8)>0})
 covered=set().union(*(coverage[i]for i in BODIES));assert covered==set().union(*coverage)and len(covered)==118
 index=[];totalrows=totalfallbacks=0
 with(COHORT/'input.bin').open('rb')as source,(NATIVE/'native-projection1032.bin').open('rb')as native,(a.out/'input.bin').open('xb')as inp,(a.out/'expected114284.bin').open('xb')as expected:
  for bi in BODIES:
   b=idx[bi];source.seek(b['offset']);h=source.read(32);n=word(h,4);records=source.read(n*324);assert hashlib.sha256(records).hexdigest()==b['body_records_sha256'];blocks=[records[i*324:(i+1)*324]for i in range(n)];native.seek(bi*122*1032);selections=[]
   for q in range(122):
    ref=native.read(1032);assert word(ref,0)==bi and word(ref,4)==q;selections.append(list(struct.unpack_from(f'<{word(ref,8)}I',ref,12)))
   for clipid,c in enumerate(clips):
    channels=c['channels'];assert 1<=len(channels)<=14 and all(ch['enabled']and ch['groupAF']==0 and ch['secondary']==[2,0,0,0]and ch['metadataAC']&4==0 for ch in channels)
    assert[c['channel']for c in channels]==list(range(len(channels)))
    descriptors=sum(len(ch['descriptors'])for ch in channels);offset=inp.tell();inp.write(struct.pack('<8I',0x31444c42,n,len(channels),0x27f,bi,clipid,descriptors,0));inp.write(records)
    for ch in channels:
     words=ch['raw_metadata_words_88_d0'];assert len(words)==19 and words[1:5]==ch['primary']and words[5:9]==ch['secondary']and words[9]==ch['metadataAC'];assert((words[9]>>16)&255)==ch['ownerAE']and words[9]>>24==0;inp.write(struct.pack('<20I',*words,len(ch['descriptors'])))
     for d in ch['descriptors']:assert len(d)==8 and d[4]<9 and not(d[0]&15==3 and d[1]==0x70e47545);inp.write(struct.pack('<8I',*d))
    def selected(ci,mirror):
     s=channels[ci]['primary']
     if s in([1,0,0,0],[5,0,0,0]):return[0]
     return selections[mapping[tuple(s)]*2+mirror]
    rows,fields,zero,fallback=reference(blocks,channels,selected);projection=struct.pack('<11I',bi,clipid,1,len(rows),*fields)+b''.join(struct.pack('<4I',*r)for r in rows)+b'\xff'*((CAPACITY-len(rows))*16);assert len(projection)==PROJECTION;expected.write(projection);totalrows+=len(rows);totalfallbacks+=fallback
    index.append(dict(case=len(index),body=bi,clip=clipid,source_body_key=b['key'],source_body_sha256=b['source_sha256'],source_clip_key=c['key'],source_clip_sha256=c['sha256'],input_offset=offset,input_bytes=inp.tell()-offset,blocks=n,channels=len(channels),descriptors=descriptors,record_count=len(rows),selected_zero_count=zero,empty_fallback_count=fallback,fields=fields,projection_sha256=hashlib.sha256(projection).hexdigest()))
 assert len(index)==1085
 (a.out/'cases.json').write_text(json.dumps(index,indent=2)+'\n');(a.out/'clips.json').write_text(json.dumps(clips,indent=2)+'\n');(a.out/'bodies.json').write_text(json.dumps([idx[i]for i in BODIES],indent=2)+'\n')
 pins=[COHORT/'input.bin',COHORT/'body-index.json',COHORT/'selector-table.json',COHORT/'manifest.json',NATIVE/'native-projection1032.bin',NATIVE/'report.json',NATIVE/'rust-comparison-v1.json',NATIVE/'immutability-v1.json',WHOLE/'first-root-static-retention-v3.json',WHOLE/'all818-whole-channel-prepared-v2.json',ROOT/'crates/spore-core/src/anim_ungrouped_binding_builder.rs',Path(__file__)]
 m=dict(schema='whole9A1E00-actual217-fivebody-entry-inputs-v1',cases=1085,clips=217,actual_body_ids=BODIES,retained_body_upstream_exclusions=43,source_enabled_channel_instances=1861,source_nonroot_channel_instances=1644,source_descriptors=7956,case_source_channel_instances=9305,case_source_descriptors=39780,positive_selector_queries_covered=118,total_selector_query_policies=122,retained_genuine_empty_query_policies=[32,33,44,45],reference_record_count=totalrows,reference_empty_fallback_count=totalfallbacks,record_capacity=7140,record_capacity_bytes=114240,header_bytes=32,body_record_bytes=324,channel_wire_header_bytes=80,descriptor_bytes=32,projection_bytes=PROJECTION,reply_bytes=PROJECTION+8,input_sha256=sha(a.out/'input.bin'),expected_sha256=sha(a.out/'expected114284.bin'),native_fullbuilder_executed=False,original_actor_association=False,scope='Bounded prepared full9A1E00 entry crossproduct:actual217 sourceclips×5actualsourcebodyfieldprojections,allchannel/order/AE/descriptors. Freshnativeoriginalselector results feed independent reference, not authored endpoints. Original fullbuilder kernel not yet admitted.',pins={str(p.resolve()):sha(p)for p in pins},files={p.name:sha(p)for p in a.out.iterdir()if p.is_file()})
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps({k:m[k]for k in('cases','reference_record_count','reference_empty_fallback_count','input_sha256','expected_sha256')}))
if __name__=='__main__':main()

