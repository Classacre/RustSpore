"""Census native ordered-ID/parent classification across all installed copies.

No package winner is chosen. Every matching parent copy contributes a possible
native presence mask; identical results can be proved independently of overlay
priority. Runtime lookup keys are retained as explicit low/scope word pairs.
"""
import argparse
from collections import Counter,defaultdict
import hashlib
import json
from pathlib import Path
import struct

from cell_capability_cache_probe import named_property_records,TAGS,PARENT
from dbpf_extract import payload
from gmdl_skin_probe import indexes


def find(ids,query):
    first=0;count=len(ids)
    while count>0:
        step=count>>1;candidate=first+step
        if ids[candidate]<query:first=candidate+1;count-=step+1
        else:count=step
    return first if first<len(ids)and query>=ids[first]else None


def native_class(mask):
    return next((kind for index,(_ident,kind)in enumerate(TAGS)if mask&(1<<index)),1)


def main():
    ap=argparse.ArgumentParser();ap.add_argument('install',type=Path)
    ap.add_argument('--census-dir',type=Path,required=True);ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args();target=(Path(__file__).resolve().parent.parent/'target').resolve()
    args.out.resolve().relative_to(target);args.census_dir.resolve().relative_to(target)
    args.out.mkdir(parents=True,exist_ok=False)
    census=json.loads((args.census_dir/'inventory.json').read_text());assert not census['failures']
    wanted=sorted({int(record['part_instance'],16)for record in census['part_props']});assert len(wanted)==654
    providers=defaultdict(list);packages=0
    for package in sorted(args.install.rglob('*.package')):
        packages+=1
        for entry in indexes(package):
            if entry['type']==0x00b1b104:providers[entry['instance']].append((package,entry))
    loaded={};failures=[];failed_keys=set();stats=Counter();maximum=Counter()
    pending=list(wanted)
    while pending:
        key=pending.pop()
        if key in loaded:continue
        records=[];loaded[key]=records
        for package,entry in providers[key]:
            try:
                raw=payload(package,entry);fields=named_property_records(raw);ids=[row[0]for row in fields]
                ordered=all(a<=b for a,b in zip(ids,ids[1:]));duplicates=len(ids)-len(set(ids))
                mask=sum(1<<index for index,(tag,_kind)in enumerate(TAGS)if find(ids,tag)is not None)
                parent_at=find(ids,PARENT);parent_key=None;parent_field=None;issues=[]
                if parent_at is not None:
                    ident,kind,spec,value=fields[parent_at]
                    parent_field=dict(type=f'{kind:04X}',spec=f'{spec:04X}',raw_hex=value.hex())
                    active=value
                    if spec in (0x1c,0x9c):
                        number,stride=struct.unpack_from('>2I',value)
                        if number==0 or stride<12:issues.append('unsupported_empty_or_short_parent_key_array')
                        else:active=value[8:8+stride]
                    if kind!=0x20 or len(active)<12:issues.append('unsupported_parent_property_type_or_extent')
                    elif not issues:
                        low,middle,scope=struct.unpack_from('<3I',active)
                        effective_scope=scope if scope else key>>32
                        parent_key=(effective_scope<<32)|low
                        parent_field.update(low=f'{low:08X}',ignored_middle=f'{middle:08X}',
                            explicit_scope=f'{scope:08X}',effective_scope=f'{effective_scope:08X}')
                        pending.append(parent_key)
                record=dict(package=package.relative_to(args.install).as_posix(),
                    legacy_archive_group_word=f'{entry["group"]:08X}',
                    low=f'{key&0xffffffff:08X}',scope=f'{key>>32:08X}',
                    sha256=hashlib.sha256(raw).hexdigest(),raw_bytes=len(raw),
                    ids=[f'{ident:08X}'for ident in ids],ids_sorted_unsigned=ordered,duplicate_ids=duplicates,
                    local_native_mask=f'{mask:03X}',parent_key=None if parent_key is None else f'{parent_key:016X}',
                    parent_field=parent_field,issues=issues)
                records.append(record);stats['property_copies']+=1;stats['unsorted_id_lists']+=not ordered
                stats['duplicate_id_fields']+=duplicates;maximum['property_fields']=max(maximum['property_fields'],len(ids))
            except Exception as exc:
                failed_keys.add(key)
                failures.append(dict(key=f'{key:016X}',package=package.relative_to(args.install).as_posix(),error=repr(exc)))
    solved={};active=[]
    def solve(key):
        if key in solved:return solved[key]
        if key in active:return set(),{f'parent_cycle:{key:016X}'},0
        active.append(key);masks=set();issues={'property_parse_failure'}if key in failed_keys else set();depth=1
        if not loaded[key]:masks.add(0);stats['missing_installed_parent_keys']+=1
        for record in loaded[key]:
            local=int(record['local_native_mask'],16)
            if record['issues']:issues.update(record['issues']);continue
            parent=record['parent_key']
            if parent is None:parent_masks={0};parent_issues=set();parent_depth=0
            else:parent_masks,parent_issues,parent_depth=solve(int(parent,16))
            issues.update(parent_issues);depth=max(depth,parent_depth+1)
            masks.update(local|value for value in parent_masks)
        active.pop();solved[key]=(masks,issues,depth);return solved[key]
    results=[];classes=Counter();all_classes=Counter();depths=Counter()
    for key in wanted:
        masks,issues,depth=solve(key);possible=sorted({native_class(mask)for mask in masks})
        supported=len(possible)==1 and not issues
        if supported:classes[str(possible[0])]+=1
        else:stats['unresolved_part_identities']+=1
        for kind in possible:all_classes[str(kind)]+=1
        depths[str(depth)]+=1
        results.append(dict(low=f'{key&0xffffffff:08X}',scope=f'{key>>32:08X}',
            direct_copies=len(loaded[key]),possible_native_masks=[f'{mask:03X}'for mask in sorted(masks)],
            possible_classes=possible,all_candidate_copies_agree=supported,
            maximum_parent_chain_lists=depth,issues=sorted(issues)))
    stats['reachable_property_keys']=len(loaded);maximum['parent_chain_lists']=max(int(key)for key in depths)
    result=dict(indexes=packages,source_part_identities=len(wanted),statistics=dict(stats),maximum=dict(maximum),
        agreed_native_classes_by_identity=dict(classes),possible_class_counts=dict(all_classes),depths=dict(depths),
        parts=results,properties={f'{key:016X}':value for key,value in sorted(loaded.items())},failures=failures,
        scope='All installed candidates under explicit(low,scope) keys. Native unsigned lower_bound and ordered parent-presence semantics. No archive priority winner, actual loadedpackage set, livepropertycache oractor binding chosen.')
    (args.out/'report.json').write_text(json.dumps(result,indent=2))
    print(json.dumps({name:result[name]for name in ['indexes','source_part_identities','statistics','maximum','agreed_native_classes_by_identity','depths','failures']},indent=2))
    raise SystemExit(1 if failures else 0)


if __name__=='__main__':main()
