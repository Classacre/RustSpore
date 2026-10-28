"""Actual all1644 selector cohort; joins genuine original Z publisher rows."""
import argparse,json,struct,hashlib
from pathlib import Path
from anim_nonroot_selector_prepare_v1 import select,word,number,f,INDEX
ROOT=Path(__file__).resolve().parent.parent
OLD=ROOT/'target/animation/nonroot-selector-inputs-v3'
ZRUN=ROOT/'target/animation/selector-Z-producer-native-run-v2/native-v1'
ZINPUT=ROOT/'target/animation/selector-Z-producer-inputs-v1'
CONTROLS=[0x7f|p|r for p in(0,0x200,0x300)for r in(0,0x400,0x800,0xc00)]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def z_select(blocks,s,mirror,fields):
 assert s[0]in(0x4c,0x1040)and s[2:]==[0,0]
 hierarchy=s[0]==0x1040;candidates=select(blocks,[0 if hierarchy else 0xc,s[1],0,0],mirror)
 minimum=struct.unpack('<f',struct.pack('<I',fields[2]))[0];extent=struct.unpack('<f',struct.pack('<I',fields[5]))[0];mid=f(f(extent*.5)+minimum);out=[]
 for i in candidates:
  if number(blocks[i],INDEX[0x114])<mid:continue
  ordinal=i
  if hierarchy:
   parent=word(blocks[i],INDEX[0x1fc])
   if parent!=0xffffffff:
    while word(blocks[parent],4)&8==0:parent=word(blocks[parent],INDEX[0x1fc]);assert parent!=0xffffffff
    ordinal=parent
  if ordinal not in out:out.append(ordinal)
 return out

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.resolve().relative_to((ROOT/'target').resolve());a.out.mkdir(parents=True,exist_ok=False)
 assert sha(ZRUN/'native-projection44.bin')=='9c4d52db13730637c08f806a596aefd3f56299525a92f4c432aa9d6eca11336e'
 assert sha(ZRUN/'corpus-output.bin')=='91a573f56bb64e99fbf18666d54c2c6d7bffe433e79bf8a59789993408a2ab32'
 report=json.loads((ZRUN/'report.json').read_text());assert report['native_executed']and report['protocol_checks']==47 and report['observed']==dict(groups=1322,rows=15864)
 oldidx=json.loads((OLD/'body-index.json').read_text());zidx=json.loads((ZINPUT/'body-index.json').read_text());aliases=json.loads((OLD/'selector-aliases.json').read_text());oldselectors=json.loads((OLD/'selector-table.json').read_text());selectors=sorted({tuple(x['primary'])for x in aliases});assert len(aliases)==1644 and len(selectors)==61 and len(oldselectors)==59
 bounds=[]
 with(ZRUN/'native-projection44.bin').open('rb')as projection,(ZRUN/'corpus-output.bin').open('rb')as raw:
  for i,(b,z)in enumerate(zip(oldidx,zidx)):
   assert b['body']==z['body']==i and all(b[k]==z[k]for k in('native_case','package','key','source_sha256'));six=None;chosen=None
   for cw in CONTROLS:
    r=raw.read(52);v=projection.read(44);assert len(r)==52 and r[:44]==v and word(v,0)==i and word(v,4)==cw and v[42:44]==bytes(2)
    assert struct.unpack_from('<H',r,44)[0]==cw and struct.unpack_from('<H',r,46)[0]&0x3800==0 and word(r,48)&~0x3f==0x1f80
    fields=struct.unpack_from('<6I',v,8)
    if six is None:six=fields
    assert fields==six,'Published six fields differ between supplied CW policies'
    if cw==0x27f:chosen=v
   assert chosen is not None;bounds.append(dict(body=i,fields=list(six),projection44_hex=chosen.hex(),source_key=b['key'],source_sha256=b['source_sha256']))
  assert not raw.read(1)and not projection.read(1)
 oldmap={tuple(s):i for i,s in enumerate(oldselectors)};queries=nonempty=selected=oldpreserved=zqueries=0;bodyout=[]
 with(OLD/'input.bin').open('rb')as old,(OLD/'expected1032.bin').open('rb')as gold,(a.out/'input.bin').open('xb')as inp,(a.out/'expected1032.bin').open('xb')as expected:
  for bi,b in enumerate(oldidx):
   header=old.read(32);n=word(header,4);assert word(header,8)==118 and word(header,16)==bi;records=old.read(n*324);assert hashlib.sha256(records).hexdigest()==b['body_records_sha256'];blocks=[records[i*324:(i+1)*324]for i in range(n)];old.read(118*32);oldgold=[gold.read(1032)for _ in range(118)]
   offset=inp.tell();inp.write(struct.pack('<8I',0x314c4553,n,122,0x27f,bi,0,0,0));inp.write(records)
   for si,st in enumerate(selectors):
    s=list(st)
    for mirror in(0,1):
     qi=si*2+mirror;inp.write(struct.pack('<8I',*s,mirror,255,0,qi))
     if st in oldmap:
      oldref=oldgold[oldmap[st]*2+mirror];v=list(struct.unpack('<258I',oldref));assert v[0]==bi;v[1]=qi;oldpreserved+=1
     else:
      chosen=z_select(blocks,s,mirror,bounds[bi]['fields']);v=[bi,qi,len(chosen),*chosen,*([0xffffffff]*(255-len(chosen)))];zqueries+=1
     expected.write(struct.pack('<258I',*v));queries+=1;nonempty+=v[2]>0;selected+=v[2]
   bodyout.append(dict(**b,offset=offset,group_bytes=inp.tell()-offset,native_bounds_projection44_hex=bounds[bi]['projection44_hex']))
  assert not old.read(1)and not gold.read(1)
 assert queries==161284 and oldpreserved==155996 and zqueries==5288
 for name,obj in [('body-index.json',bodyout),('native-definition-bounds.json',bounds),('selector-table.json',selectors),('selector-aliases.json',[dict(x,admitted=True,exclusion=None,prior_exclusion=x['exclusion'])for x in aliases])]:
  (a.out/name).write_text(json.dumps(obj,indent=2)+'\n')
 pins=[OLD/'manifest.json',OLD/'input.bin',OLD/'expected1032.bin',OLD/'body-index.json',OLD/'selector-table.json',OLD/'selector-aliases.json',ZINPUT/'input.bin',ZINPUT/'body-index.json',ZINPUT/'manifest.json',ZRUN/'corpus-output.bin',ZRUN/'native-projection44.bin',ZRUN/'report.json',ZRUN/'execution-pins-v1.json',ZRUN/'immutability-v1.json',ROOT/'tools/anim_nonroot_selector_prepare_v1.py',ROOT/'crates/spore-core/src/anim_nonroot_selector.rs',ROOT/'crates/spore-core/src/anim_nonroot_selector_z.rs',Path(__file__)]
 m=dict(schema='actual-all1644-nonroot-selector-with-native-Z-inputs-v1',bodies=1322,blocks=46256,source_aliases=1644,admitted_source_aliases=1644,retained_Z_exclusions=0,body_upstream_exclusions=43,unique_selector_words=61,mirrors=2,queries=queries,old_155996_exact_slots_preserved=oldpreserved,new_Z_queries=zqueries,nonempty_reference_queries=nonempty,selected_reference_ordinals=selected,six_published_fields_identical_across12CWs=True,selected_bounds_CW='027F',native_Z_projected_sha256=sha(ZRUN/'native-projection44.bin'),input_sha256=sha(a.out/'input.bin'),expected_sha256=sha(a.out/'expected1032.bin'),pins={str(x.resolve()):sha(x)for x in pins},files={x.name:sha(x)for x in a.out.iterdir()if x.is_file()},native_combined_executed=False,scope='Actual source finalized body × all source selector policies. Bounds come solely from genuine original publisher output. New combined host must execute original producer, not inject these expected fields. No clip/actor association, full builder, AE ownership/pruning claim.')
 (a.out/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print(json.dumps({k:m[k]for k in('bodies','queries','new_Z_queries','nonempty_reference_queries','selected_reference_ordinals','input_sha256','expected_sha256')}))
if __name__=='__main__':main()
