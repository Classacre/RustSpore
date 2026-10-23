"""Census original creature rows and source predicates before Cell cache porting.

This read-only pass does not infer a runtime part-manager implementation,
choose conflicting overlay records, or substitute classes for unresolved props.
The exact native 128-byte CreatureData RLE framing is reused independently.
"""
import argparse
from collections import Counter,defaultdict
from functools import lru_cache
import hashlib
import json
import math
from pathlib import Path
import struct

from anim_rig_mapping_probe import key_values
from creature_header_probe import header
from dbpf_extract import payload
from gmdl_skin_probe import indexes
from prop_numeric_probe import ARRAYS,COMPOUNDS,PRIMITIVES,fields

TAGS=[(0x100f0f5a,12),(0x11b79a74,2),(0x11b79a70,3),(0x032f92e6,9),
    (0x11b79a72,10),(0x11b79a71,7),(0x11b79a78,4),(0x11b79a75,6),
    (0x11b79a76,5),(0x11b79a77,8),(0x11b79a73,11)]
PARENT=0x00b2cccb


def named_property_records(data):
    """Retain raw fields using the complete named/count-bounded PROP framing.

    Compound singles have two measured extents. Select their extent only when
    the entire declared list consumes the payload; anonymous recovery and a
    greedy choice at the next field are deliberately unsupported here.
    """
    if len(data)<4:raise ValueError('missing_item_count')
    count=struct.unpack_from('>I',data)[0]
    if count>10000:raise ValueError('item_count_exceeds_limit')
    decisions={}

    def separator(at):
        return data[at:at+4]==b'\1\0\0\0' or data[at+2:at+4] in (b'\x12\0',b'\xa0\0')

    def ends(start,kind,spec):
        if spec in ARRAYS:
            if start+8>len(data):return []
            number,stride=struct.unpack_from('>II',data,start)
            if number>200000 or stride>16384:return []
            end=start+8
            if kind in (0x12,0x13):
                for _ in range(number):
                    if end+4>len(data):return []
                    size=struct.unpack_from('>I',data,end)[0]
                    end+=4+size*(1 if kind==0x12 else 2)
                    if end>len(data):return []
            else:
                if kind in PRIMITIVES and stride<4:return []
                end+=number*stride
            return [end]if end<=len(data)else []
        if kind in PRIMITIVES:sizes=[4]
        elif kind==1:sizes=[1]
        elif kind==0x20:sizes=[16]
        elif kind in COMPOUNDS:
            sizes=[COMPOUNDS[kind]]
            if kind not in (0x33,0x34):sizes.append(COMPOUNDS[kind]+4)
        elif kind in (0x12,0x13):
            if start+4>len(data):return []
            size=struct.unpack_from('>I',data,start)[0]
            sizes=[4+size*(1 if kind==0x12 else 2)]
        elif kind==0:sizes=[spec]
        else:return []
        return [start+size for size in sizes if start+size<=len(data)]

    @lru_cache(None)
    def solve(at,remaining):
        while at+4<=len(data) and separator(at):at+=4
        if remaining==0:return at==len(data)
        if at+8>len(data):return False
        ident,kind,spec=struct.unpack_from('>IHH',data,at)
        for end in ends(at+8,kind,spec):
            if solve(end,remaining-1):
                decisions[at,remaining]=(end,ident,kind,spec)
                return True
        return False

    if not solve(4,count):raise ValueError('nonexact_declared_named_property_list')
    records=[];at=4;remaining=count
    while remaining:
        while at+4<=len(data) and separator(at):at+=4
        end,ident,kind,spec=decisions[at,remaining]
        records.append((ident,kind,spec,data[at+8:end]))
        at=end;remaining-=1
    return records


def source_rows(data):
    if len(data)<20 or struct.unpack_from('<I',data)[0]!=0xabb455b7:
        raise ValueError('invalid_native_creature_header')
    version=struct.unpack_from('<I',data,4)[0]
    if version not in (9,10):raise ValueError('unsupported_creature_version')
    count=3 if version==9 else 4
    counts=struct.unpack_from('<'+'I'*count,data,8)
    if counts[0]>256 or counts[1]>4096 or counts[2]>1024:
        raise ValueError('native_source_count_guard')
    at,decoded,_encoded,_chunks=header(data,8+4*count,128)
    end=at+counts[0]*140+counts[1]*5+counts[2]*12+4+(counts[3]*56 if version==10 else 0)
    if end!=len(data) or len(decoded)!=128:raise ValueError('nonexact_creature_framing')
    return [data[at+140*index:at+140*(index+1)]for index in range(counts[0])]


def main():
    ap=argparse.ArgumentParser();ap.add_argument('install',type=Path);ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    args.out.mkdir(parents=True,exist_ok=False)
    stats=Counter();failures=[];wanted=set();all_props=defaultdict(list);records=[];words6=Counter()
    with (args.out/'source-rows.jsonl').open('w',encoding='utf-8')as row_log:
        for package in sorted(args.install.rglob('*.package')):
            relative=package.relative_to(args.install).as_posix();stats['indexes']+=1
            for entry in indexes(package):
                if entry['type']==0x00b1b104:all_props[entry['instance']].append((package,entry))
                if entry['type']!=0x0f43029a:continue
                key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
                try:
                    data=payload(package,entry);rows=source_rows(data)
                except Exception as exc:
                    failures.append(dict(package=relative,key=key,error=repr(exc)));continue
                stats['creature_resources']+=1;stats['all_source_rows']+=len(rows)
                stats['clamped_source_resources']+=len(rows)>64
                stats['rows_omitted_by_native64_limit']+=max(len(rows)-64,0)
                stats['cache_source_rows']+=min(len(rows),64)
                records.append(dict(package=relative,key=key,sha256=hashlib.sha256(data).hexdigest(),
                    source_rows=len(rows),native_cache_rows=min(len(rows),64)))
                for ordinal,row in enumerate(rows[:64]):
                    high,low=struct.unpack_from('<2I',row,132)
                    # Source row stores high+84 and low+88, unlike PROP's key
                    # tuple. Keep the full64-bit original part instance.
                    instance=(high<<32)|low;wanted.add(instance)
                    word6=struct.unpack_from('<H',row,6)[0];words6[f'{word6:04X}']+=1
                    values=[struct.unpack_from('<f',row,at)[0]for at in [*range(20,48,4),*range(48,96,4)]]
                    stats['nonfinite_row_geometry_words']+=any(not math.isfinite(v)for v in values)
                    row_log.write(json.dumps(dict(package=relative,key=key,row=ordinal,
                        part_instance=f'{instance:016X}',word6=f'{word6:04X}',raw_hex=row.hex()))+'\n')
                if stats['creature_resources']%250==0:
                    print(f"CACHE_CENSUS resources={stats['creature_resources']} rows={stats['cache_source_rows']}",flush=True)
    props=[];by_instance=defaultdict(list)
    for instance in sorted(wanted):
        for package,entry in all_props[instance]:
            relative=package.relative_to(args.install).as_posix()
            key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            try:
                data=payload(package,entry);fields(data,set())
                parsed=named_property_records(data)
                if len(parsed)!=struct.unpack_from('>I',data)[0] or any(ident is None for ident,*_ in parsed):
                    raise ValueError('nonexact_named_property_walk')
                selected=[];present=0;true_bool=0;unclassified=0;parents=[]
                for ident,kind,spec,raw in parsed:
                    if ident==PARENT:parents.extend(key_values(kind,spec,raw))
                    for bit,(tag,_class)in enumerate(TAGS):
                        if ident!=tag:continue
                        present|=1<<bit
                        # Retain raw predicate-bearing fields, independently of
                        # the still-unresolved vtable+1C predicate semantics.
                        if kind==1 and spec not in (0x1c,0x9c) and len(raw)==1:
                            if raw[0]:true_bool|=1<<bit
                        else:unclassified|=1<<bit
                        selected.append(dict(id=f'{ident:08X}',type=f'{kind:04X}',spec=f'{spec:04X}',raw_hex=raw.hex()))
                direct_class=next((kind for bit,(_tag,kind)in enumerate(TAGS)if present&(1<<bit)),1)
                bool_class=next((kind for bit,(_tag,kind)in enumerate(TAGS)if true_bool&(1<<bit)),1)
                item=dict(package=relative,key=key,part_instance=f'{instance:016X}',
                    sha256=hashlib.sha256(data).hexdigest(),predicate_fields=selected,
                    presence_mask=f'{present:03X}',true_bool_mask=f'{true_bool:03X}',unclassified_predicate_mask=f'{unclassified:03X}',
                    presence_class_hypothesis=direct_class,true_bool_class_hypothesis=bool_class,
                    parent_refs=[dict(instance=f'{parent:016X}',type=f'{kind:08X}')for parent,kind in parents])
                props.append(item);by_instance[instance].append(item)
            except Exception as exc:failures.append(dict(package=relative,key=key,error=repr(exc)))
    classes=Counter();reasons=Counter()
    for instance in sorted(wanted):
        matches=by_instance[instance]
        if not matches:reasons['no_direct_PROP_candidate']+=1;continue
        candidates={row['presence_class_hypothesis']for row in matches}
        if len(candidates)!=1:reasons['conflicting_direct_presence_classes']+=1
        else:classes[str(next(iter(candidates)))]+=1
        reasons['presence_vs_bool_differs']+=any(row['presence_class_hypothesis']!=row['true_bool_class_hypothesis']for row in matches)
        reasons['unclassified_predicate_type']+=any(int(row['unclassified_predicate_mask'],16)for row in matches)
        reasons['has_parent_reference']+=any(row['parent_refs']for row in matches)
    report=dict(statistics=dict(stats),unique_part_instances=len(wanted),direct_PROP_copies=len(props),
        part_word6_counts=dict(words6),direct_presence_classes_by_unique_instance=dict(classes),
        unresolved_or_hypothesis_counts=dict(reasons),predicate_order=[dict(id=f'{ident:08X}',native_class=kind)for ident,kind in TAGS],
        creatures=records,part_props=props,failures=failures,
        scope='All source creature copies, first64native rows each; direct PROP/tag census only. Vtable1C semantics, inheritance/overlay/manager binding remain explicit gates; no class hypothesis selects live actor data.')
    (args.out/'inventory.json').write_text(json.dumps(report,indent=2))
    print(json.dumps({k:v for k,v in report.items()if k not in ['creatures','part_props','part_word6_counts']},indent=2))
    if failures:raise SystemExit(1)


if __name__=='__main__':main()
