"""Fresh source intake and independent integer/rational finalizer references.

No original bytes are executed. Source-stage native requests use explicit
CW027F; all three precision contexts enter full-source digests and all twelve
masked precision/rounding contexts enter bounded synthetic native requests.
"""
import argparse
from collections import Counter
import copy
import hashlib
import itertools
import json
from pathlib import Path
import random
import struct
import sys

from anim_body_anchors_probe import AnchorOracle
from anim_body_fields_probe import TailOracle
from anim_body_hierarchy_probe import constructor_inputs, package_entries
from anim_body_optional_fixture import (CAPABILITIES_SHA, empty_block, reference as optional_reference,
    add, sub, mul, bits)
from anim_body_finalizer_native_probe import FIELDS, FIELD_INDEX, RECORD_BYTES, INPUT_BYTES, PROJECTION_BYTES
from anim_body_root_fixture import reference as root_reference
from anim_body_source_probe import SourceOracle, CONTROLS, fnv
from anim_order_probe import native_reference as order_reference
from cell_schema_probe import SourceImage, SOURCE_SHA256
from creature_header_probe import header
from dbpf_extract import payload
from native_x87_probe import operand, quantize, sqrt_quantize, commit

ALL_CONTROLS=[0x7F|p|r for p,r in itertools.product([0,0x200,0x300],[0,0x400,0x800,0xC00])]
ROOT, LIMB, SPINE = 0x746F6F72, 0x626D696C, 0x6E697073
CATEGORIES={ROOT:1,LIMB:2,0x746F6F66:4,SPINE:8,0x74667370:16,0x74756F6D:32,0x70737267:64,0x7274736E:128}

def rebuild(b):
    caps=bytearray(b['caps']);presence,nonzero=0,0
    for slot in range(32):
        ident=struct.unpack_from('<I',caps,8+4*slot)[0]
        if not ident:break
        category=CATEGORIES.get(ident,0);presence|=category
        if caps[136+slot]:nonzero|=category
    struct.pack_into('<2I',caps,0,presence,nonzero)
    b.update(caps=bytes(caps),presence=presence,nonzero=nonzero)

def ensure(b,ident,number):
    caps=bytearray(b['caps'])
    for slot in range(32):
        current=struct.unpack_from('<I',caps,8+4*slot)[0]
        if current in [0,ident]:
            struct.pack_into('<I',caps,8+4*slot,ident);caps[136+slot]=number;break
    b['caps']=bytes(caps);rebuild(b)

def norm(values,cw):
    precision={0:24,0x200:53,0x300:64}[cw&0x300];rounding=(cw>>10)&3
    x,y,z=map(operand,values)
    xx,yy,zz=[quantize(v*v,precision,rounding)[0] for v in [x,y,z]]
    xy=quantize(yy+xx,precision,rounding)[0]
    total=quantize(zz+xy,precision,rounding)[0]
    root=sqrt_quantize(total,precision,rounding)[0] if total else total
    output=commit(root,rounding);assert output is not None
    return output

def reroot(input_blocks,selected,cw):
    blocks=copy.deepcopy(input_blocks);ensure(blocks[selected],ROOT,1)
    if selected:
        parent_anchor=[blocks[selected][0x350+j*4] for j in range(3)]
        child_anchor=[blocks[selected][0x344+j*4] for j in range(3)]
        previous,index=0xFFFFFFFF,selected
        while index!=0xFFFFFFFF:
            b=blocks[index];old_parent=b[0x1FC]
            old_child=[b[0x344+j*4] for j in range(3)]
            old_anchor=[b[0x350+j*4] for j in range(3)]
            b[0x1FC]=previous
            for j in range(3):b[0x344+j*4],b[0x350+j*4]=parent_anchor[j],child_anchor[j]
            delta=[sub(parent_anchor[j],b[0x144+j*4]) if b['presence']&8 else parent_anchor[j] for j in range(3)]
            for j in range(3):b[0x35C+j*4]=delta[j]
            b[0x368]=norm(delta,cw)
            parent_anchor,child_anchor=old_anchor,old_child
            previous,index=index,old_parent
        blocks[0][0x208]-=1;blocks[selected][0x208]+=1
    for at in range(0x344,0x36C,4):blocks[selected][at]=0
    return blocks

def local_orientations(input_blocks):
    blocks=copy.deepcopy(input_blocks)
    for b in blocks:
        if b[0x1FC]==0xFFFFFFFF:continue
        parent=input_blocks[b[0x1FC]]
        cx,cy,cz,cw=[b[0x118+j*4] for j in range(4)]
        px,py,pz,pw=[parent[0x118+j*4] for j in range(4)]
        out=[sub(add(sub(mul(cx,pw),mul(cw,px)),mul(cy,pz)),mul(cz,py)),
            add(sub(sub(mul(cy,pw),mul(cw,py)),mul(cx,pz)),mul(cz,px)),
            add(sub(sub(mul(cx,py),mul(cw,pz)),mul(cy,px)),mul(cz,pw)),
            add(add(add(mul(cy,py),mul(cx,px)),mul(cw,pw)),mul(cz,pz))]
        for j,value in enumerate(out):b[0x128+j*4]=value
    return blocks

def relabel(input_blocks,starts,no_foot):
    blocks=copy.deepcopy(input_blocks)
    if not no_foot:
        for index in starts:
            while index:
                b=blocks[index]
                if sum(bool(child['nonzero']&2) for child in blocks[b[0x204]:b[0x204]+b[0x208]])>1:break
                ensure(b,LIMB,1);ensure(b,SPINE,0)
                index=b[0x1FC];assert index!=0xFFFFFFFF
    return blocks

def ordered(input_blocks,root):
    parents=[None if b[0x1FC]==0xFFFFFFFF else b[0x1FC] for b in input_blocks]
    result=order_reference(parents,[b['presence'] for b in input_blocks],root)
    blocks=[]
    for new,(old,parent,start,count,_,descendants) in enumerate(result['blocks']):
        b=copy.deepcopy(input_blocks[old]);b[0x1F8]=new;b[0x1FC]=0xFFFFFFFF if parent is None else parent
        b[0x200]=0xFFFFFFFF if b[0x200]==0xFFFFFFFF else result['old_to_new'][b[0x200]]
        b[0x204],b[0x208],b[0x20C]=start,count,descendants
        blocks.append(b)
    return blocks,result

def pipeline(blocks,selected,no_foot,cw):
    root_stage=reroot(blocks,selected,cw)
    order_stage,order=ordered(root_stage,selected)
    local_stage=local_orientations(order_stage)
    last=max(i for i,b in enumerate(blocks) if b['nonzero']&8)
    starts=[order['old_to_new'][0],order['old_to_new'][last]]
    final=relabel(local_stage,starts,no_foot)
    prefix=max((i+1 for i,b in enumerate(blocks) if b[0x21C]!=0xFFFFFFFF),default=0)
    return final,dict(order=order,starts=starts,prefix=prefix,root_stage=root_stage,order_stage=order_stage,local_stage=local_stage)

def record(b):return b['caps']+struct.pack('<39I',*(b.get(at,0) for at in FIELDS))
def request(blocks,mode,cw,selected=0,starts=(0,0),no_foot=0):
    data=struct.pack('<HH7I',cw,0,mode,len(blocks),selected,*starts,no_foot,0)+b''.join(record(b) for b in blocks)
    assert len(data)<=INPUT_BYTES
    return data+bytes(INPUT_BYTES-len(data))
def projection(blocks,raw_header):
    data=raw_header+b''.join(record(b) for b in blocks)
    return data+bytes(PROJECTION_BYTES-len(data))

def source_digest(rows,initial,called,optional_header,blocks,cw):
    digest=fnv(b'native-body-finalizer-fields-source-v1\0')
    for word in [cw,0x3DCCCCCD,2,len(rows),initial['root_nomination'],int(called),*optional_header]:
        digest=fnv(struct.pack('<I',word),digest)
    for name in ['source_to_initial','initial_to_source']:
        digest=fnv(struct.pack('<'+'I'*len(rows),*initial[name]),digest)
    digest=fnv(b''.join(rows),digest)
    for b in blocks:digest=fnv(record(b),digest)
    root_data=None;meta=None;final=None
    if not blocks[0]['nonzero']&8:admission=1
    else:
        root_blocks=[dict(parent=None if b[0x1FC]==0xFFFFFFFF else b[0x1FC],
            presence=b['presence'],nonzero=b['nonzero'],center=[b[at] for at in [0x10C,0x110,0x114]]) for b in blocks]
        root_data,selection=root_reference(root_blocks,2)
        admission={'Ok':0,'SelectedRootMissingNonzeroSpine':2,'NoSelectedRoot':3}[selection['admission']]
    digest=fnv(struct.pack('<I',admission),digest)
    if not admission:
        root,no_foot=struct.unpack_from('<2I',root_data)
        final,meta=pipeline(blocks,root,bool(no_foot),cw)
        digest=fnv(root_data,digest)
        for name in ['new_to_old','old_to_new']:
            digest=fnv(struct.pack('<'+'I'*len(blocks),*meta['order'][name]),digest)
        digest=fnv(struct.pack('<4I',*meta['starts'],meta['prefix'],len(final)),digest)
        for b in final:digest=fnv(record(b),digest)
    return digest,admission,root_data,final,meta

def source_inputs(install,image,capability_inputs,capability_run,statistics):
    caps=(capability_run/'native-projection.bin').read_bytes()
    assert hashlib.sha256(caps).hexdigest()==CAPABILITIES_SHA
    assert json.loads((capability_run/'comparison-v1.json').read_text())['exact']
    by_key={(r['package'],r['key']):r for r in json.loads((capability_inputs/'source-resources.json').read_text())}
    head,tail,anchor=SourceOracle(image),TailOracle(image),AnchorOracle(image)
    for package in sorted(install.rglob('*.package')):
        statistics['package_indexes']+=1
        for entry in package_entries(package):
            if entry['type']!=0x0F43029A:continue
            raw=payload(package,entry);initial=constructor_inputs(raw);count=initial['source_rows']
            key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}"
            name=package.relative_to(install).as_posix();known=by_key[(name,key)]
            assert hashlib.sha256(raw).hexdigest()==known['source_sha256']
            at=header(raw,20 if struct.unpack_from('<I',raw,4)[0]==9 else 24)[0]
            rows=[raw[at+i*140:at+(i+1)*140] for i in range(count)]
            source_caps=[caps[(known['first_row']+i)*168:(known['first_row']+i+1)*168] for i in range(count)]
            masks=[struct.unpack_from('<I',value)[0] for value in source_caps]
            called=initial['root_nomination']==0 and not any(struct.unpack_from('<I',value,4)[0]&4 for value in source_caps)
            for control,precision,rounding in CONTROLS:
                rest=[head.evaluate(row,precision,rounding)[0] for row in rows]
                fields=[tail.evaluate(row,value) for row,value in zip(rows,rest)]
                anchors=[anchor.evaluate(rows,rest,fields,masks,i,precision,rounding)[0] for i in range(count)]
                blocks=[]
                for b in initial['blocks']:
                    i=b['source_palette_id'];r=rows[i]
                    part=empty_block(b['initial_ordinal'],b['parent'],*struct.unpack_from('<2I',source_caps[i]),rest[i][:3])
                    part['caps']=source_caps[i]
                    part.update({0x104:struct.unpack_from('<I',r,0x88)[0],0x108:struct.unpack_from('<I',r,0x84)[0],
                        0x21C:i,0x200:b['symmetric_word_written'],0x208:b['initial_child_count'],
                        0x378:struct.unpack_from('<I',r,0x2C)[0],0x37C:struct.unpack_from('<I',r,0x6C)[0],0x204:0,0x20C:0})
                    for j,v in enumerate(rest[i][3:]):part[0x118+j*4]=v
                    for j,v in enumerate(fields[i][:6]):part[0x138+j*4]=v
                    for j,v in enumerate(anchors[i]):part[0x344+j*4]=v
                    blocks.append(part)
                if called:blocks,optional_header=optional_reference(blocks,0x3DCCCCCD,0)
                else:optional_header=(len(blocks),0,0,0)
                for b in blocks:b.setdefault(0x204,0);b.setdefault(0x20C,0)
                statistics['evaluated_source_rows']+=count
                yield name,key,rows,initial,called,optional_header,blocks,control
            statistics['resources']+=1;statistics['source_rows']+=count
            if statistics['resources']%64==0:print(f"resources={statistics['resources']} rows={statistics['source_rows']}",flush=True)

def synthetic_cases():
    rng=random.Random(0x9C3E72)
    for kind in ['chain3','chain255','presence_spine','nonspine','opaque_root','nonunit','subnormal',
        'saturated','hole_stale','duplicate_spine','branch_stop','presence_limb_children','both_paths']:
        n=255 if kind=='chain255' else 7 if kind in ['branch_stop','presence_limb_children','both_paths'] else 3
        parents=[None]+list(range(n-1))
        if n==7:parents=[None,0,0,1,1,2,2]
        blocks=[empty_block(i,parent,8,8,[bits(float(i)),0,0]) for i,parent in enumerate(parents)]
        for b in blocks:b[0x204]=0;b[0x20C]=0
        for i,b in enumerate(blocks):
            for j in range(3):b[0x344+j*4]=bits((i+1)*(j+1)/8);b[0x350+j*4]=bits(-(i+1)*(j+2)/16)
            b[0x144]=bits(0.25);b[0x148]=bits(-0.5)
            b[0x368]=bits(-42.);b[0x21C]=0xFFFFFFFF if i==n-1 else i
        for b in blocks:
            if b[0x1FC]!=0xFFFFFFFF:blocks[b[0x1FC]][0x208]+=1
        selected=2 if n==3 else 3 if n==7 else n-1
        if kind in ['presence_spine','nonspine']:
            caps=bytearray(blocks[1]['caps']);caps[136]=0
            if kind=='nonspine':struct.pack_into('<I',caps,8,0)
            blocks[1]['caps']=bytes(caps);rebuild(blocks[1])
        if kind=='opaque_root':
            selected=0
            for at in range(0x344,0x36C,4):blocks[0][at]=0x7FC12345
            for at in range(0x128,0x138,4):blocks[0][at]=0x7F800001
        if kind=='nonunit':
            for b in blocks:
                for at in range(0x118,0x128,4):b[at]=bits(rng.uniform(-2,2))
        if kind=='subnormal':
            for b in blocks:
                b[0x344],b[0x348],b[0x34C]=1,0x80000001,0x80000000
                b[0x350],b[0x354],b[0x358]=16,0x80000008,1
                b[0x144],b[0x148],b[0x14C]=8,0x80000001,0
                b[0x118],b[0x11C],b[0x120]=1,0x80000001,0x80000000
        if kind in ['saturated','hole_stale','duplicate_spine']:
            for b in blocks:
                caps=bytearray(b['caps'])
                for slot in range(1,32):struct.pack_into('<I',caps,8+slot*4,0x1000+slot);caps[136+slot]=0xA5
                if kind=='hole_stale':struct.pack_into('<I',caps,12,0);struct.pack_into('<I',caps,16,LIMB)
                if kind=='duplicate_spine':struct.pack_into('<I',caps,12,SPINE);caps[137]=2
                b['caps']=bytes(caps);rebuild(b)
        if kind in ['branch_stop','presence_limb_children','both_paths']:
            for i in [3,4,5,6]:ensure(blocks[i],LIMB,0 if kind=='presence_limb_children' else 1)
        yield kind,blocks,selected

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('install',type=Path);p.add_argument('--source-exe',type=Path,required=True)
    p.add_argument('--capability-inputs',type=Path,required=True);p.add_argument('--capability-run',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    a.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    a.out.mkdir(parents=True,exist_ok=False)
    sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'target/rea/capstone-runtime'))
    image=SourceImage(a.source_exe)
    statistics,families,admissions=Counter(),Counter(),Counter();digests=[];native_rows=0
    input_sha,output_sha=hashlib.sha256(),hashlib.sha256()
    with (a.out/'inputs.bin').open('xb') as binary,(a.out/'python-output.bin').open('xb') as output, \
            (a.out/'cases.tsv').open('x',newline='\n') as index:
        index.write('case\tfamily\tmode\tcw\tresource\n')
        def emit(blocks,mode,cw,selected=0,starts=(0,0),no_foot=0,family='',resource='-'):
            nonlocal native_rows
            raw=request(blocks,mode,cw,selected,starts,no_foot)
            expected=reroot(blocks,selected,cw) if mode==0 else local_orientations(blocks) if mode==1 else relabel(blocks,starts,no_foot)
            projected=projection(expected,raw[:32])
            binary.write(raw);output.write(projected);input_sha.update(raw);output_sha.update(projected)
            index.write(f'{native_rows}\t{family}\t{mode}\t{cw:04X}\t{resource}\n')
            native_rows+=1;families[family]+=1
        for name,key,rows,initial,called,optional_header,blocks,cw in source_inputs(a.install,image,a.capability_inputs,a.capability_run,statistics):
            digest,admission,root_data,final,meta=source_digest(rows,initial,called,optional_header,blocks,cw)
            digests.append(f'{name} {key} {cw:04X} {admission} {digest:016X}')
            admissions[str(admission)]+=1
            if not admission and cw==0x027F:
                selected,no_foot=struct.unpack_from('<2I',root_data)
                emit(blocks,0,cw,selected=selected,family='source_reroot',resource=key)
                emit(meta['order_stage'],1,cw,family='source_local',resource=key)
                emit(meta['local_stage'],2,cw,starts=meta['starts'],no_foot=no_foot,family='source_relabel',resource=key)
        for family,blocks,selected in synthetic_cases():
            for cw in ALL_CONTROLS:
                _,meta=pipeline(blocks,selected,False,cw)
                emit(blocks,0,cw,selected=selected,family='synthetic_'+family+'_reroot')
                emit(meta['order_stage'],1,cw,family='synthetic_'+family+'_local')
                for no_foot in [0,1]:
                    emit(meta['local_stage'],2,cw,starts=meta['starts'],no_foot=no_foot,family='synthetic_'+family+f'_relabel_skip{no_foot}')
    normalized='\n'.join(sorted(digests))+'\n'
    (a.out/'source-digests.txt').write_text(normalized,newline='\n')
    report=dict(schema='native-body-finalizer-fields-inputs-v1',native_executed=False,statistics=dict(statistics),
        rows=native_rows,families=dict(families),source_admission_contexts=dict(admissions),
        source_digest_records=len(digests),source_digest_sha256=hashlib.sha256(normalized.encode()).hexdigest(),
        input_sha256=input_sha.hexdigest(),expected_projection_sha256=output_sha.hexdigest(),
        source_sha256=SOURCE_SHA256,input_bytes=INPUT_BYTES,record_bytes=RECORD_BYTES,projection_bytes=PROJECTION_BYTES,
        capability_projection_sha256=CAPABILITIES_SHA,tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        scope='All1365fresh canonical source intakes under3nearest x87 precision contexts, explicit0.1 threshold/initial3F5false/rootpolicy2. All admission results retained, including original nonzero-spin failures. Only CW027F accepted source stages enter native fixtures; all12CWs enter synthetic stages. Integer binary32/rational x87 references. No original bytes executed during preparation.')
    (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
    assert statistics['package_indexes']==82 and statistics['resources']==1365 and statistics['source_rows']==46418
    assert statistics['evaluated_source_rows']==139254 and len(digests)==4095

if __name__=='__main__':main()
