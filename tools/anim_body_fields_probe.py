"""Read-only source-pinned oracle for canonical body extents/origin fields.

Extends the already conformed seven-float rest boundary with unchanged decoded
SSE instructions9CC5A8..9CC800. No production port or optional-foot execution.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import sys

from anim_body_hierarchy_probe import package_entries
from anim_body_pose_probe import arithmetic
from anim_body_source_probe import SourceOracle, CONTROLS
from cell_schema_probe import SourceImage, SOURCE_SHA256
from creature_header_probe import header, inspect
from dbpf_extract import payload
from native_x87_probe import operand

START, END = 0x9CC5A8, 0x9CC801
CONSTANTS = {0x14719E4, 0x1471B2C, 0x1489324}
OFFSETS = [0x138,0x13C,0x140,0x144,0x148,0x14C,0x324,0x328,0x32C,0x330,0x334,0x338]


def fnv(data, digest=0xCBF29CE484222325):
    for byte in data: digest=((digest^byte)*0x100000001B3)&0xFFFFFFFFFFFFFFFF
    return digest


class TailOracle:
    def __init__(self,image):
        from native_crt_acos_probe import decoder
        from capstone.x86 import X86_OP_REG,X86_OP_MEM,X86_OP_IMM
        self.image,self.program,self.constants=image,{},{}
        raw=image.read(START,END-START)
        rows=list(decoder().disasm(raw,START))
        assert sum(i.size for i in rows)==len(raw)
        for i in rows:
            arguments=[]
            for v in i.operands:
                if v.type==X86_OP_REG: arguments.append(('reg',i.reg_name(v.reg)))
                elif v.type==X86_OP_IMM: arguments.append(('imm',v.imm))
                elif v.type==X86_OP_MEM:
                    arguments.append(('mem',i.reg_name(v.mem.base),i.reg_name(v.mem.index),v.mem.scale,v.mem.disp))
                    if not v.mem.base and not v.mem.index:
                        assert v.mem.disp in CONSTANTS
                        self.constants[v.mem.disp]=struct.unpack('<I',image.read(v.mem.disp,4))[0]
                else: raise AssertionError('unknown source operand')
            assert i.mnemonic in {'movss','movaps','xorps','mulss','addss','subss','lea','mov','comiss','jb','add','sub','jne'}
            self.program[i.address]=(i.mnemonic,arguments,i.address+i.size)
        self.evidence=dict(start=f'{START:08X}',end_exclusive=f'{END:08X}',bytes=len(raw),
            instructions=len(rows),sha256=hashlib.sha256(raw).hexdigest(),
            constants={f'{a:08X}':f'{v:08X}' for a,v in self.constants.items()},
            entry_seed='Exact earlier source stores:scaledMin@stack44/48/4C,scaledMax@30/34/38;XMM1=centerY at9CC594,XMM3=translationX-centerX at9CC590,XMM5=translationY until9CC5B0,XMM6=translationZ-centerZ at9CC580,XMM4=signMask;center/quaternion from conformed9CC249..5A8/9A4DF0.')

    def evaluate(self,row,rest):
        source,stack,block=0x10000,0x20000,0x30000
        memory={source+i:struct.unpack_from('<I',row,i)[0] for i in range(0,140,4)}
        registers={'edi':source,'esp':stack,'esi':block,'eax':0,'ecx':0}
        registers.update(xmm1=rest[1],xmm3=arithmetic('subss',memory[source+0x54],rest[0]),
            xmm5=memory[source+0x58],xmm6=arithmetic('subss',memory[source+0x5C],rest[2]),xmm4=0x80000000)
        for i,word in enumerate(rest): memory[block+0x10C+i*4]=word
        for n,at in enumerate([0x44,0x48,0x4C,0x30,0x34,0x38]):
            memory[stack+at]=arithmetic('mulss',memory[source+0x14+n*4],memory[source+0x2C])
        def address(v):
            return (registers.get(v[1],0)+registers.get(v[2],0)*v[3]+v[4])&0xFFFFFFFF
        def read(v):
            if v[0]=='reg': return registers[v[1]]
            if v[0]=='imm': return v[1]
            at=address(v)
            if at in memory:return memory[at]
            return self.constants[at] # Unknown source/frame reads are errors.
        def write(v,word):
            if v[0]=='reg':registers[v[1]]=word&0xFFFFFFFF
            else:memory[address(v)]=word&0xFFFFFFFF
        pc,zero,below,steps=START,False,False,0
        while pc!=END:
            operation,args,next_pc=self.program[pc];steps+=1
            assert steps<=180
            if operation in {'movss','movaps','mov'}:write(args[0],read(args[1]))
            elif operation=='xorps':write(args[0],read(args[0])^read(args[1]))
            elif operation in {'mulss','addss','subss'}:write(args[0],arithmetic(operation,read(args[0]),read(args[1])))
            elif operation=='lea':write(args[0],address(args[1]))
            elif operation=='comiss':
                left,right=[operand(read(v)) for v in args]
                zero,below=left==right,left<right
            elif operation in {'add','sub'}:
                left,right=[read(v) for v in args]
                result=left+right if operation=='add' else left-right
                write(args[0],result);zero=(result&0xFFFFFFFF)==0
            elif operation=='jb':
                if below:next_pc=read(args[0])
            elif operation=='jne':
                if not zero:next_pc=read(args[0])
            else:raise AssertionError(operation)
            pc=next_pc
        return [memory[block+i] for i in OFFSETS]


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('install',type=Path);p.add_argument('--source-exe',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    a=p.parse_args();root=Path(__file__).resolve().parent.parent
    a.out.resolve().relative_to((root/'target').resolve());a.out.mkdir(parents=True,exist_ok=False)
    sys.path.insert(0,str(root/'target/rea/capstone-runtime'))
    image=SourceImage(a.source_exe);head,tail=SourceOracle(image),TailOracle(image)
    stats,fields,records,lines,failures=Counter(),Counter(),[],[],[]
    for package in sorted(a.install.rglob('*.package')):
        entries=package_entries(package);stats['package_indexes']+=1
        for entry in entries:
            if entry['type']!=0x0F43029A:continue
            raw=payload(package,entry);inspect(raw,Counter())
            version,count=struct.unpack_from('<II',raw,4)
            at=header(raw,20 if version==9 else 24)[0]
            key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}"
            name=package.relative_to(a.install).as_posix()
            for cw,precision,rounding in CONTROLS:
                digest=fnv(b'native-body-source-fields-v1\0'+struct.pack('<2I',cw,count))
                try:
                    for i in range(count):
                        row=raw[at+i*140:at+(i+1)*140]
                        rest,branch=head.evaluate(row,precision,rounding)
                        values=tail.evaluate(row,rest)
                        assert all(x&0x7F800000!=0x7F800000 for x in values)
                        digest=fnv(struct.pack('<I',i)+row+struct.pack('<19I',*rest,*values),digest)
                        stats['evaluated_rows']+=1
                        fields[f'{cw:04X}:extent_contains_3DCCCCCD_rows']+=any(x==0x3DCCCCCD for x in values[:3])
                    lines.append(f'{name} {key} {cw:04X} {digest:016X}')
                except Exception as exc:failures.append(dict(package=name,key=key,cw=f'{cw:04X}',error=repr(exc)))
            stats['resources']+=1;stats['source_rows']+=count
            records.append(dict(package=name,key=key,source_sha256=hashlib.sha256(raw).hexdigest(),rows=count))
            if stats['resources']%128==0:print(f"resources={stats['resources']} evaluated_rows={stats['evaluated_rows']}",flush=True)
    normalized='\n'.join(sorted(lines))+'\n'
    (a.out/'source-digests.txt').write_text(normalized,newline='\n')
    report=dict(schema='canonical-source-body-field-instruction-oracle-v1',statistics=dict(stats),fields=dict(fields),
        source_sha256=SOURCE_SHA256,head_regions=head.evidence,tail_region=tail.evidence,
        output_offsets=[f'{x:03X}' for x in OFFSETS],source_records=records,failures=failures,
        source_digest_sha256=hashlib.sha256(normalized.encode()).hexdigest(),native_execution=False,
        scope='Source-pinned bounded instruction interpreter. Exact earlier rest boundary plus source-origin/extent/effects fields, all source rows and three explicit x87 contexts. No new Rust production comparison, optional-foot construction, full hierarchy or runtime animation claimed.')
    (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='source_records'},indent=2))
    assert not failures and stats['package_indexes']==82 and stats['resources']==1365 and stats['source_rows']==46418


if __name__=='__main__':main()
