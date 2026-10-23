"""Stream original optional-foot source/synthetic inputs and integer-SSE results."""
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
from anim_body_optional_native_probe import FIELDS, FIELD_INDEX, RECORD_BYTES, INPUT_BYTES, PROJECTION_BYTES
from anim_body_pose_probe import arithmetic, bits
from anim_body_source_probe import SourceOracle, CONTROLS, fnv
from cell_schema_probe import SourceImage, SOURCE_SHA256
from creature_header_probe import header
from dbpf_extract import payload

CAPABILITIES_SHA='582aca8c6e189c9d8385749d68798bb03efdd9bb8da0446f7ac3c01032c6b337'
ALL_CONTROLS=[0x7F|p|r for p,r in itertools.product([0,0x200,0x300],[0,0x400,0x800,0xC00])]
SIGN=0x80000000


def nonfinite(v):return v&0x7F800000==0x7F800000
def nan(v):return nonfinite(v) and v&0x7FFFFF!=0
def number(v):return struct.unpack('<f',struct.pack('<I',v))[0]
def sse(op,left,right):
    # Nonfinite values here occur only in min/threshold intermediates, never
    # serialized output coordinates. All unordered comparisons branch JBE.
    if nan(left) or nan(right):return 0xFFC00000
    if op=='subss':return sse('addss',left,right^SIGN)
    if nonfinite(left) or nonfinite(right):
        if op=='addss':
            if nonfinite(left) and nonfinite(right) and (left^right)&SIGN:return 0xFFC00000
            return left if nonfinite(left) else right
        if op=='mulss':
            if left&0x7FFFFFFF==0 or right&0x7FFFFFFF==0:return 0xFFC00000
            return ((left^right)&SIGN)|0x7F800000
        raise ValueError(op)
    return arithmetic(op,left,right)
def add(a,b):return sse('addss',a,b)
def sub(a,b):return sse('subss',a,b)
def mul(a,b):return sse('mulss',a,b)
def less(a,b):return not nan(a) and not nan(b) and number(a)<number(b)
def greater(a,b):return not nan(a) and not nan(b) and number(a)>number(b)


def minimum_height(b):
    a=mul(b[0x13C],b[0x134]&~SIGN)
    c=mul(b[0x140],b[0x138]&~SIGN)
    d=mul(b[0x138],b[0x130]&~SIGN)
    return sub(b[0x114],add(add(a,c),d))
def eligible(b):return b[0x208]==0 and b['presence']&2 and not b['presence']&4
def limb_minimum(blocks):
    best,selected=0x7F7FFFFF,None
    for i,b in enumerate(blocks):
        if eligible(b):
            v=minimum_height(b)
            if less(v,best):best,selected=v,i
    return selected,best


def rotation(q,v):
    x,y,z,w=q;vx,vy,vz=v
    nx,ny,nz=[mul(i,i)^SIGN for i in [x,y,z]]
    xy,xz,yz,wx,wy,wz=[mul(a,b) for a,b in [(y,x),(z,x),(z,y),(x,w),(y,w),(z,w)]]
    return [add(mul(add(add(mul(add(nz,ny),vx),mul(add(xz,wy),vz)),mul(sub(xy,wz),vy)),bits(2.)),vx),
        add(mul(add(add(mul(add(xy,wz),vx),mul(sub(yz,wx),vz)),mul(add(nz,nx),vy)),bits(2.)),vy),
        add(mul(add(add(mul(sub(xz,wy),vx),mul(add(ny,nx),vz)),mul(add(yz,wx),vy)),bits(2.)),vz)]
def parent_anchor(q,d):
    x,y,z,w=q;dx,dy,dz=d
    nx,ny,nz=[mul(i,i)^SIGN for i in [x,y,z]]
    nwx,nwy,nwz=[mul(i,w)^SIGN for i in [x,y,z]]
    xy,xz,yz=[mul(a,b) for a,b in [(y,x),(z,x),(z,y)]]
    return [add(mul(add(add(mul(sub(xy,nwz),dy),mul(add(xz,nwy),dz)),mul(add(nz,ny),dx)),bits(2.)),dx),
        add(mul(add(add(mul(add(nz,nx),dy),mul(sub(yz,nwx),dz)),mul(add(xy,nwz),dx)),bits(2.)),dy),
        add(mul(add(add(mul(add(ny,nx),dz),mul(add(yz,nwx),dy)),mul(sub(xz,nwy),dx)),bits(2.)),dz)]


def empty_block(ordinal,parent,presence,nonzero,center):
    b={offset:0 for offset in FIELDS}
    caps=bytearray(168);struct.pack_into('<2I',caps,0,presence,nonzero)
    slot=0
    for ident,mask in [(b'root',1),(b'limb',2),(b'foot',4),(b'spin',8),(b'psft',16),(b'mout',32),(b'grsp',64),(b'nstr',128)]:
        if presence&mask:
            caps[8+slot*4:12+slot*4]=ident;caps[136+slot]=int(bool(nonzero&mask));slot+=1
    b.update(caps=bytes(caps),presence=presence,nonzero=nonzero)
    b.update({0x1F8:ordinal,0x1FC:0xFFFFFFFF if parent is None else parent,0x200:0xFFFFFFFF,0x21C:ordinal,
        0x124:bits(1.),0x134:bits(1.),0x138:bits(0.5),0x13C:bits(0.5),0x140:bits(0.5),0x36C:0x7F7FFFFF,
        0x378:bits(1.),0x37C:bits(0.5)})
    for i,v in enumerate(center):b[0x10C+i*4]=v
    return b


def reference(input_blocks,threshold,initial_flag):
    blocks=copy.deepcopy(input_blocks);batches=0
    spine=0x7F7FFFFF
    for b in blocks:
        if b['presence']&8:
            h=minimum_height(b)
            if less(h,spine):spine=h
    selected,minimum=limb_minimum(blocks);initial=minimum
    maximum_batches=sum(bool(eligible(b)) for b in blocks)
    while selected is not None:
        if not greater(spine,minimum):break
        if not greater(mul(sub(spine,initial),threshold),sub(minimum,initial)):break
        assert batches<maximum_batches
        b=blocks[selected];assert not b['presence']&0x60
        r=rotation([b[0x118+i*4] for i in range(4)],[b[0x144+i*4] for i in range(3)])
        point=[sub(b[0x10C+i*4],r[i]) for i in range(3)]
        mirror=None if b[0x200]==0xFFFFFFFF else b[0x200]
        first=len(blocks)
        for n,parent in enumerate([selected]+([] if mirror is None else [mirror])):
            center=point.copy()
            if n:center[0]^=SIGN
            foot=empty_block(len(blocks),parent,4,4,center)
            foot[0x104]=0x6B2C0771;foot[0x108]=0x40600000;foot[0x21C]=0xFFFFFFFF
            foot[0x200]=0xFFFFFFFF if mirror is None else first+1 if n==0 else first
            for at in [0x138,0x13C,0x140]:foot[at]=0x3C23D70A
            p=blocks[parent]
            d=[sub(center[i],p[0x10C+i*4]) for i in range(3)]
            a=parent_anchor([p[0x118+i*4] for i in range(4)],d)
            for i,v in enumerate(a):foot[0x350+i*4]=v
            p[0x208]+=1;blocks.append(foot)
        batches+=1
        selected,minimum=limb_minimum(blocks)
    assert len(blocks)<=255
    return blocks,(len(blocks),len(blocks)-len(input_blocks),0 if batches else initial_flag,batches)


def record(b):return b['caps']+struct.pack('<'+str(len(FIELDS))+'I',*(b[at] for at in FIELDS))
def request(blocks,threshold,flag,cw):
    data=struct.pack('<HHIII',cw,0,len(blocks),threshold,flag)+b''.join(record(b) for b in blocks)
    data+=bytes(INPUT_BYTES-len(data));assert len(data)==INPUT_BYTES
    return data
def projection(blocks,header):
    data=struct.pack('<4I',*header)+b''.join(record(b) for b in blocks)
    data+=bytes(PROJECTION_BYTES-len(data));assert len(data)==PROJECTION_BYTES
    return data


def check_domain(blocks,threshold):
    assert 0<len(blocks)<=255 and not nonfinite(threshold)
    assert blocks[0][0x1FC]==0xFFFFFFFF
    children=[0]*len(blocks)
    for i,b in enumerate(blocks):
        assert b[0x1F8]==i and b[0x21C]==i
        assert b['presence']<=255 and not b['nonzero']&~b['presence'] and not b['nonzero']&4
        p=b[0x1FC]
        if p!=0xFFFFFFFF:assert p<len(blocks);children[p]+=1
        else:assert i==0
        assert b[0x200]==0xFFFFFFFF or b[0x200]<len(blocks)
        for at in range(0x10C,0x150,4):
            v=b[at];assert not nonfinite(v)
            assert v&~SIGN <= (0x40000000 if 0x118<=at<0x138 else 0x71800000)
        assert all(0<b[at]<0x80000000 for at in [0x138,0x13C,0x140])
        assert not eligible(b) or not b['presence']&0x60
        seen=set();at=i
        while at!=0xFFFFFFFF:
            assert at not in seen;seen.add(at);at=blocks[at][0x1FC]
    assert children==[b[0x208] for b in blocks]
    assert len(blocks)+2*sum(bool(eligible(b)) for b in blocks)<=255


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('install',type=Path);p.add_argument('--source-exe',type=Path,required=True)
    p.add_argument('--capability-inputs',type=Path,required=True);p.add_argument('--capability-run',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    root=Path(__file__).resolve().parent.parent;a.out.resolve().relative_to((root/'target').resolve())
    a.out.mkdir(parents=True,exist_ok=False);sys.path.insert(0,str(root/'target/rea/capstone-runtime'))
    caps=(a.capability_run/'native-projection.bin').read_bytes()
    assert hashlib.sha256(caps).hexdigest()==CAPABILITIES_SHA
    assert json.loads((a.capability_run/'comparison-v1.json').read_text())['exact']
    by_key={(r['package'],r['key']):r for r in json.loads((a.capability_inputs/'source-resources.json').read_text())}
    image=SourceImage(a.source_exe);head,tail,anchor=SourceOracle(image),TailOracle(image),AnchorOracle(image)
    statistics,source_records,index,digests,output_counts=Counter(),[],[],[],Counter()
    input_sha,output_sha=hashlib.sha256(),hashlib.sha256()
    with (a.out/'inputs.bin').open('xb') as stream,(a.out/'python-output.bin').open('xb') as expected:
        def emit(blocks,threshold,flag,control,family,resource=None):
            check_domain(blocks,threshold)
            output,values=reference(blocks,threshold,flag)
            request_bytes=request(blocks,threshold,flag,control);output_bytes=projection(output,values)
            stream.write(request_bytes);expected.write(output_bytes);input_sha.update(request_bytes);output_sha.update(output_bytes)
            index.append(dict(row=len(index),family=family,resource=resource,control=f'{control:04X}',
                flag=flag,threshold=f'{threshold:08X}',source_blocks=len(blocks),result_blocks=values[0],appended=values[1],batches=values[3]))
            statistics[family]+=1;output_counts[f'appended_{values[1]}']+=1
        for package in sorted(a.install.rglob('*.package')):
            statistics['package_indexes']+=1
            for entry in package_entries(package):
                if entry['type']!=0x0F43029A:continue
                raw=payload(package,entry);initial=constructor_inputs(raw);count=initial['source_rows']
                key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}"
                name=package.relative_to(a.install).as_posix();known=by_key[(name,key)]
                sha=hashlib.sha256(raw).hexdigest();assert sha==known['source_sha256']
                version=struct.unpack_from('<I',raw,4)[0];at=header(raw,20 if version==9 else 24)[0]
                rows=[raw[at+i*140:at+(i+1)*140] for i in range(count)]
                source_caps=[caps[(known['first_row']+i)*168:(known['first_row']+i+1)*168] for i in range(count)]
                masks=[struct.unpack_from('<I',value)[0] for value in source_caps]
                called=initial['root_nomination']==0 and not any(struct.unpack_from('<I',value,4)[0]&4 for value in source_caps)
                resource=len(source_records)
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
                            0x378:struct.unpack_from('<I',r,0x2C)[0],0x37C:struct.unpack_from('<I',r,0x6C)[0]})
                        for j,v in enumerate(rest[i][3:]):part[0x118+j*4]=v
                        for j,v in enumerate(fields[i][:6]):part[0x138+j*4]=v
                        for j,v in enumerate(anchors[i]):part[0x344+j*4]=v
                        blocks.append(part)
                    for flag in [0,1]:
                        if called:
                            emit(blocks,0x3DCCCCCD,flag,control,'source_call_gate',resource)
                            output,values=reference(blocks,0x3DCCCCCD,flag)
                        else:output,values=blocks,(len(blocks),0,flag,0)
                        digest=fnv(b'native-body-optional-source-v1\0'+struct.pack('<5I',control,flag,initial['root_nomination'],count,called))
                        for mapping in [initial['source_to_initial'],initial['initial_to_source']]:digest=fnv(struct.pack('<'+str(count)+'I',*mapping),digest)
                        digest=fnv(b''.join(rows)+struct.pack('<4I',*values),digest)
                        for b in output:digest=fnv(record(b),digest)
                        digests.append(f'{name} {key} {control:04X} {flag} {digest:016X}')
                    statistics['evaluated_source_rows']+=count
                statistics['source_resources']+=1;statistics['source_rows']+=count
                statistics['source_call_resources']+=called
                source_records.append(dict(package=name,key=key,source_sha256=sha,rows=count,driver_called=called,
                    root_nomination=initial['root_nomination']))
                if statistics['source_resources']%128==0:print(f"resources={statistics['source_resources']} native_rows={len(index)}",flush=True)
        rng=random.Random(0x9CBE70)
        families=[]
        for mirror in [False,True]:
            for threshold in [bits(0.),bits(-0.),bits(-0.1),bits(0.1),bits(1.),0x7F7FFFFF]:
                b=[empty_block(0,None,8,8,[0,0,bits(10.)]),empty_block(1,0,2,2,[bits(2.),bits(3.),0])]
                b[0][0x208]=1
                if mirror:
                    b.append(empty_block(2,0,2,2,[bits(-2.),bits(3.),0]));b[0][0x208]=2
                    b[1][0x200]=2;b[2][0x200]=1
                families.append((f'synthetic_mirror{int(mirror)}_threshold{threshold:08X}',b,threshold))
        for kind in ['empty255','no_spines','presence_only','threshold_unordered','height_overflow','source_opaque_nonfinite','signed_subnormal','nonunit_rotations']:
            if kind=='empty255':
                b=[empty_block(i,None if i==0 else i-1,8,8,[0,0,bits(float(i))]) for i in range(255)]
                for part in b[:-1]:part[0x208]=1
            else:
                b=[empty_block(0,None,8,8,[0,0,bits(10.)]),empty_block(1,0,2,2,[bits(1.),bits(2.),0])];b[0][0x208]=1
                if kind=='no_spines':b[0]['presence']=b[0]['nonzero']=0;b[0]['caps']=bytes(168)
                if kind=='presence_only':
                    b[0]['nonzero']=0;c=bytearray(b[0]['caps']);struct.pack_into('<I',c,4,0);c[136]=0;b[0]['caps']=bytes(c)
                if kind in ['threshold_unordered','height_overflow']:
                    for part in ([b[1]] if kind=='threshold_unordered' else b):
                        for at in [0x138,0x13C,0x140]:part[at]=0x71800000
                if kind=='source_opaque_nonfinite':
                    for n,at in enumerate(list(range(0x344,0x36C,4))+[0x36C,0x378,0x37C]):b[0][at]=[0x7FC12345,0xFF800000,0x7F800000][n%3]
                if kind=='signed_subnormal':
                    b[1][0x10C]=16;b[1][0x144]=8;b[1][0x110]=SIGN;b[1][0x148]=SIGN
                if kind=='nonunit_rotations':
                    for part in b:
                        for at in range(0x118,0x138,4):part[at]=bits(rng.uniform(-1.8,1.8))
                        for at in range(0x144,0x150,4):part[at]=bits(rng.uniform(-4,4))
            families.append((f'synthetic_{kind}',b,bits(0.1)))
        for kind in ['exact_capacity255','repeated_threshold_stop','repeated_threshold_equal','presence_foot_zero_number']:
            if kind=='exact_capacity255':
                b=[empty_block(i,None if i==0 else i-1,8,8,[0,0,bits(10.)]) for i in range(252)]
                for part in b[:-1]:part[0x208]=1
                b.append(empty_block(252,0,2,2,[bits(1.),0,0]));b[0][0x208]=2;b[252][0x200]=251
            elif kind=='presence_foot_zero_number':
                b=[empty_block(0,None,8,8,[0,0,bits(10.)]),empty_block(1,0,6,2,[0,0,0])];b[0][0x208]=1
            else:
                b=[empty_block(0,None,8,8,[0,0,bits(10.)])]
                for i,z in enumerate([0.,0.5 if kind=='repeated_threshold_stop' else 1.,5.]):
                    b.append(empty_block(i+1,0,2,2,[0,0,bits(z)]))
                b[0][0x208]=3
            families.append((f'synthetic_{kind}',b,bits(0.1)))
        for family,blocks,threshold in families:
            for control,flag in itertools.product(ALL_CONTROLS,[0,1]):emit(blocks,threshold,flag,control,family)
    normalized='\n'.join(sorted(digests))+'\n';(a.out/'source-digests.txt').write_text(normalized,newline='\n')
    report=dict(schema='bounded-optional-foot-source-and-synthetic-inputs-v1',statistics=dict(statistics),
        rows=len(index),input_bytes=INPUT_BYTES,projection_bytes=PROJECTION_BYTES,input_sha256=input_sha.hexdigest(),
        expected_projection_sha256=output_sha.hexdigest(),source_digest_sha256=hashlib.sha256(normalized.encode()).hexdigest(),
        source_digest_records=len(digests),output_counts=dict(output_counts),source_sha256=SOURCE_SHA256,
        capability_projection_sha256=CAPABILITIES_SHA,head_regions=head.evidence,field_tail=tail.evidence,anchor_regions=anchor.evidence,
        source_records=source_records,index=index,source_exclusions=[],
        scope='All1365canonical source intakes under3x87contexts and2initial3F5flags;only106callgates enter nativefixture. Expliciton-disk threshold0.1,notretailruntimepolicy. All12CWs on bounded synthetics. Independentinteger-SSE arithmetic and source-explicit branch/order; full output streamed, not retained as casewise integer lists. No original bytes executed.')
    (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['schema','statistics','rows','output_counts','input_sha256','expected_projection_sha256','source_digest_sha256','source_digest_records']},indent=2))
    assert statistics['package_indexes']==82 and statistics['source_resources']==1365 and statistics['source_rows']==46418
    assert statistics['source_call_resources']==106 and statistics['evaluated_source_rows']==139254 and len(digests)==8190


if __name__=='__main__':main()
