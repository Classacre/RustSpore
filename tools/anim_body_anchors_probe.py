"""Read-only original-instruction oracle for canonical source body anchors.

Interprets9CCA09..9CCE20 and the actual9B5200 callee over fresh source rows,
the conformed rest/field head and accepted native capability masks. No native
host, game process or original module is executed.
"""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import struct
import sys

from anim_body_fields_probe import TailOracle
from anim_body_hierarchy_probe import constructor_inputs, package_entries
from anim_body_pose_probe import arithmetic
from anim_body_source_probe import SourceOracle, CONTROLS, fnv
from cell_schema_probe import SourceImage, SOURCE_SHA256
from creature_header_probe import header
from dbpf_extract import payload
from native_x87_probe import operand, quantize, sqrt_quantize, commit

CAPABILITIES_SHA = '582aca8c6e189c9d8385749d68798bb03efdd9bb8da0446f7ac3c01032c6b337'
START, END = 0x9CCA09, 0x9CCE21
OFFSETS = list(range(0x344, 0x36C, 4))


class AnchorOracle:
    def __init__(self, image):
        from native_crt_acos_probe import decoder
        from capstone.x86 import X86_OP_REG, X86_OP_MEM, X86_OP_IMM
        self.program, self.constants, self.evidence = {}, {}, []
        for name, start, end in [('canonical source anchors', START, END),
            ('source delta getter', 0x9B5200, 0x9B52A1)]:
            raw = image.read(start, end-start)
            instructions = list(decoder().disasm(raw, start))
            assert sum(i.size for i in instructions) == len(raw)
            for ins in instructions:
                arguments = []
                for v in ins.operands:
                    if v.type == X86_OP_REG: arguments.append(('reg', ins.reg_name(v.reg), v.size))
                    elif v.type == X86_OP_IMM: arguments.append(('imm', v.imm, v.size))
                    elif v.type == X86_OP_MEM:
                        arguments.append(('mem', ins.reg_name(v.mem.base), ins.reg_name(v.mem.index),
                            v.mem.scale, v.mem.disp, v.size))
                        if not v.mem.base and not v.mem.index:
                            assert v.mem.disp in {0x14719E4, 0x1471B2C}
                            self.constants[v.mem.disp] = image.read(v.mem.disp, 4)
                    else: raise ValueError('unknown operand kind')
                self.program[ins.address] = ins.mnemonic, arguments, ins.address+ins.size
            self.evidence.append(dict(name=name, start=f'{start:08X}', end_exclusive=f'{end:08X}',
                bytes=len(raw), instructions=len(instructions), sha256=hashlib.sha256(raw).hexdigest()))

    def evaluate(self, rows, rest, fields, masks, ordinal, precision, rounding):
        raw = rows[ordinal]
        parent = struct.unpack_from('<H', raw, 4)[0]
        if parent == 0xFFFF:
            return [0]*10, 'parentless_constructor_defaults'
        assert parent < len(rows)
        source, stack, blocks, creature = 0x10000, 0x20000, 0x30000, 0x90000
        memory = {}
        def put(at, data):
            for i, byte in enumerate(data): memory[at+i] = byte
        def word(at, v): put(at, struct.pack('<I', v & 0xFFFFFFFF))
        for i in {ordinal, parent}:
            put(source+i*140, rows[i])
        word(creature+0x98, source)
        for i in {ordinal, parent}:
            for j, v in enumerate(rest[i]): word(blocks+i*0x468+0x10C+j*4, v)
            for j, v in enumerate(fields[i][3:6]): word(blocks+i*0x468+0x144+j*4, v)
            word(blocks+i*0x468+0x150, masks[i])
        own = blocks+ordinal*0x468
        registers = dict(eax=source+ordinal*140, ecx=parent*0x468, edx=blocks,
            esi=own, ebx=creature, esp=stack, xmm2=0x80000000, xmm6=0x3F000000)
        floating, zero, less, steps = [], False, False, 0
        def address(v):
            return ((registers[v[1]] if v[1] else 0)+(registers[v[2]] if v[2] else 0)*v[3]+v[4]) & 0xFFFFFFFF
        def read(v):
            if v[0] == 'imm': return v[1]
            if v[0] == 'reg':
                if v[1] == 'ax': return registers['eax'] & 0xFFFF
                return registers[v[1]]
            at = address(v)
            if at in self.constants:
                assert v[-1] == 4
                return int.from_bytes(self.constants[at], 'little')
            return int.from_bytes(bytes(memory[at+j] for j in range(v[-1])), 'little')
        def write(v, data):
            if v[0] == 'reg': registers[v[1]] = data & 0xFFFFFFFF
            else: put(address(v), (data & ((1 << (v[-1]*8))-1)).to_bytes(v[-1], 'little'))
        def push(data):
            registers['esp'] -= 4
            word(registers['esp'], data)
        def pop():
            data = int.from_bytes(bytes(memory[registers['esp']+j] for j in range(4)), 'little')
            registers['esp'] += 4
            return data
        def from_word(v): return operand(v), bool(v & 0x80000000)
        def xread(v):
            if v[0] == 'reg' and v[1].startswith('st('): return floating[-1-int(v[1][3:-1])]
            return from_word(read(v))
        def rounded(exact, negative=False):
            value, raw = quantize(exact, precision, rounding, not exact and negative)
            return value, bool(raw[-1] & 0x80)
        def binary(a,b,operation):
            av,an=a;bv,bn=b
            exact=av*bv if operation=='mul' else av+bv
            negative=an != bn if operation=='mul' else rounding==1 or an and bn
            return rounded(exact,negative)
        pc=START
        while pc != END:
            steps += 1
            assert steps <= 400, 'bounded source instruction limit'
            operation, args, next_pc = self.program[pc]
            if operation in {'mov','movss','movaps','movzx'}: write(args[0],read(args[1]))
            elif operation == 'movsx':
                value=read(args[1]);bits=args[1][-1]*8
                write(args[0],value-(1 << bits) if value & (1 << (bits-1)) else value)
            elif operation == 'xorps': write(args[0],read(args[0])^read(args[1]))
            elif operation in {'mulss','subss','addss'}: write(args[0],arithmetic(operation,read(args[0]),read(args[1])))
            elif operation in {'cmp','test'}:
                left,right=read(args[0]),read(args[1]);bits=args[0][-1]*8
                if operation=='test': zero,less=(left & right)==0,bool(left & right & (1 << (bits-1)))
                else: zero,less=left==right,bool((left-right) & (1 << (bits-1)))
            elif operation in {'jz','je','jnz','jne','jl'}:
                take=zero if operation in {'jz','je'} else not zero if operation in {'jnz','jne'} else less
                if take: next_pc=read(args[0])
            elif operation == 'lea': write(args[0],address(args[1]))
            elif operation in {'add','imul'}:
                if operation=='add':write(args[0],read(args[0])+read(args[1]))
                else:write(args[0],read(args[1])*read(args[2]))
            elif operation == 'push': push(read(args[0]))
            elif operation == 'call':
                assert read(args[0]) == 0x9B5200
                push(next_pc);next_pc=read(args[0])
            elif operation == 'ret': next_pc=pop()
            elif operation == 'fld': floating.append(xread(args[0]))
            elif operation == 'fld1': floating.append((Fraction(1),False))
            elif operation == 'fstp':
                exact,negative=floating.pop();value=commit(exact,rounding,not exact and negative)
                assert value is not None,'nonfinite x87 store'
                write(args[0],value)
            elif operation=='fmul':
                assert [v[1] for v in args]==['st(0)','st(0)']
                floating[-1]=binary(floating[-1],floating[-1],'mul')
            elif operation in {'fmulp','faddp'}:
                i=int(args[0][1][3:-1]);assert i in (1,2)
                floating[-1-i]=binary(floating[-1-i],floating[-1],'mul' if operation=='fmulp' else 'add')
                floating.pop()
            elif operation=='fsqrt':
                value,negative=floating.pop();assert value>=0
                if value:
                    value,stored=sqrt_quantize(value,precision,rounding);negative=bool(stored[-1]&0x80)
                floating.append((value,negative))
            else:raise ValueError(f'unknown source instruction {pc:08X} {operation}')
            pc=next_pc
        assert not floating and registers['esp']==stack
        values=[int.from_bytes(bytes(memory[own+offset+j] for j in range(4)),'little') for offset in OFFSETS]
        assert all(v&0x7F800000 != 0x7F800000 for v in values),'nonfinite anchor output'
        return values, 'class5_midpoint' if raw[0xB]==5 else 'source_translation'


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('install',type=Path);p.add_argument('--source-exe',type=Path,required=True)
    p.add_argument('--capability-inputs',type=Path,required=True);p.add_argument('--capability-run',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    root=Path(__file__).resolve().parent.parent
    a.out.resolve().relative_to((root/'target').resolve());a.out.mkdir(parents=True,exist_ok=False)
    comparison_path=a.capability_run/'comparison-v1.json'
    assert json.loads(comparison_path.read_text())['exact']
    caps=(a.capability_run/'native-projection.bin').read_bytes()
    assert hashlib.sha256(caps).hexdigest()==CAPABILITIES_SHA
    resources_path=a.capability_inputs/'source-resources.json'
    by_key={(r['package'],r['key']):r for r in json.loads(resources_path.read_text())}
    sys.path.insert(0,str(root/'target/rea/capstone-runtime'))
    image=SourceImage(a.source_exe);head,tail,anchor=SourceOracle(image),TailOracle(image),AnchorOracle(image)
    statistics,branches,records,lines,failures=Counter(),Counter(),[],[],[]
    for package in sorted(a.install.rglob('*.package')):
        statistics['package_indexes']+=1
        for entry in package_entries(package):
            if entry['type']!=0x0F43029A:continue
            raw=payload(package,entry);initial=constructor_inputs(raw);count=initial['source_rows']
            key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:08X}"
            name=package.relative_to(a.install).as_posix();known=by_key[(name,key)]
            source_sha=hashlib.sha256(raw).hexdigest();assert source_sha==known['source_sha256']
            version=struct.unpack_from('<I',raw,4)[0];start=header(raw,20 if version==9 else 24)[0]
            rows=[raw[start+i*140:start+(i+1)*140] for i in range(count)]
            masks=[struct.unpack_from('<I',caps,(known['first_row']+i)*168)[0] for i in range(count)]
            parents=[struct.unpack_from('<H',row,4)[0] for row in rows]
            children=[sum(parent==i for parent in parents) for i in range(count)]
            for control,precision,rounding in CONTROLS:
                digest=fnv(b'native-body-source-anchors-v1\0'+struct.pack('<2I',control,count))
                try:
                    rest=[head.evaluate(row,precision,rounding)[0] for row in rows]
                    fields=[tail.evaluate(row,value) for row,value in zip(rows,rest)]
                    for i,row in enumerate(rows):
                        values,branch=anchor.evaluate(rows,rest,fields,masks,i,precision,rounding)
                        branches[f'{control:04X}:{branch}']+=1
                        parent=0xFFFFFFFF if parents[i]==0xFFFF else parents[i]
                        digest=fnv(struct.pack('<I',i)+row+struct.pack('<13I',masks[i],parent,children[i],*values),digest)
                        statistics['evaluated_rows']+=1
                    lines.append(f'{name} {key} {control:04X} {digest:016X}')
                except Exception as exc:
                    failures.append(dict(package=name,key=key,control=f'{control:04X}',error=repr(exc)))
                    print(json.dumps(failures[-1]),flush=True)
                    raise
            records.append(dict(package=name,key=key,source_sha256=source_sha,rows=count))
            statistics['resources']+=1;statistics['source_rows']+=count
            if statistics['resources']%128==0: print(f"resources={statistics['resources']} evaluated_rows={statistics['evaluated_rows']}",flush=True)
    text='\n'.join(sorted(lines))+'\n';(a.out/'source-digests.txt').write_text(text,newline='\n')
    report=dict(schema='canonical-source-body-anchor-instruction-oracle-v1',statistics=dict(statistics),branches=dict(branches),
        source_sha256=SOURCE_SHA256,head_regions=head.evidence,field_tail=tail.evidence,anchor_regions=anchor.evidence,
        capability_projection_sha256=CAPABILITIES_SHA,capability_comparison_sha256=hashlib.sha256(comparison_path.read_bytes()).hexdigest(),
        capability_resources_sha256=hashlib.sha256(resources_path.read_bytes()).hexdigest(),source_records=records,failures=failures,
        records=len(lines),normalized_digest_sha256=hashlib.sha256(text.encode()).hexdigest(),output_offsets=[f'{v:03X}' for v in OFFSETS],
        entry_seed='At9CCA09 EAX=raw palette row,ECX=remapped parent byte offset,EDX=runtime block base,ESI=current block,EBX=CreatureData source,ESP=owned frame;9CCA18 addsEDX toECX. XMM2=sign mask,XMM6=0.5. Rest/field inputs from conformed source instruction head/tail;native capability masks. Parentless rows retain exact constructor-zero anchors and length.',
        scope='Fresh canonical source reads and independently interpreted original instructions under three explicit nearest-even x87 precision contexts and integer binary32 SSE1F80 arithmetic. No original-byte host execution, optional-foot append, reroot/local-orientation change, finalizer class or evaluated animation binding.')
    (a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['schema','statistics','branches','records','normalized_digest_sha256','failures']},indent=2))
    assert statistics['package_indexes']==82 and statistics['resources']==1365 and statistics['source_rows']==46418
    assert statistics['evaluated_rows']==139254 and len(lines)==4095 and not failures


if __name__=='__main__':main()
