"""Original data census of serialized GMDL material IDs and parameter bytes."""
import argparse
from collections import Counter,defaultdict
import json
from pathlib import Path
import struct
from gmdl_skin_probe import indexes
from dbpf_extract import payload
from refpack_bulk import install


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('skin_report',type=Path)
    ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args()
    install()
    source=json.loads(args.skin_report.read_text())
    compact=[]
    for r in source['records']:
        compact.append((r['package'],r['key'],r['tail']['source_at'],r['tail']['materials']))
    del source
    entries={}
    ids=Counter();relations=Counter();examples=defaultdict(list);errors=[]
    for n,(package,key,at,materials) in enumerate(compact):
        try:
            if package not in entries:
                entries[package]={f"{e['type']:08X}:{e['group']:08X}:{e['instance']:016X}":e for e in indexes(Path(package))}
            data=payload(Path(package),entries[package][key])
            count=len(materials)
            refs,=struct.unpack_from('>I',data,4)
            mesh_count,=struct.unpack_from('<I',data,8+12*refs)
            assert count==mesh_count,(count,mesh_count)
            # This footer is independently decoded by gmdl.rs; count comes from
            # the complete shader-material tail, preserving absent v9 slots.
            material_ids=struct.unpack_from('<'+'I'*count,data,at-count*4)
            for i,(ident,material) in enumerate(zip(material_ids,materials)):
                raw=next((bytes.fromhex(raw) for typ,raw in material['shader_data'] if typ==0x210),None)
                flag=raw[0] if raw else None
                ids[ident]+=1
                relations[(ident,flag)]+=1
                if len(examples[ident])<3:
                    examples[ident].append(dict(package=package,key=key,mesh=i,parameter_byte=flag))
        except Exception as ex: errors.append(dict(package=package,key=key,error=str(ex)))
        if (n+1)%1000==0: print(f'measured {n+1}/{len(compact)}',flush=True)
    report=dict(summary=dict(models=len(compact),meshes=sum(ids.values()),unique_material_ids=len(ids),errors=len(errors)),
                errors=errors,materials=[dict(material_id=f'{i:08X}',meshes=n,
                    parameter_bytes={str(flag):v for (ident,flag),v in relations.items() if ident==i},examples=examples[i])
                    for i,n in ids.most_common()])
    args.out.write_text(json.dumps(report,indent=2))
    print(json.dumps(report['summary'],indent=2))
    if errors: raise SystemExit(1)


if __name__=='__main__':main()
