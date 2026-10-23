"""Pinned CALL/LOOPCALL source-instruction foundation; no native execution.

Exhausts the loop-count prefix and executes complete original call kernels
only with an owned PUSHB1/ADD body. Installed FPGM range-resolution cases stop
before any body entry. This is not a trace or emulator of retail PREP/glyphs.
"""
import argparse
from collections import Counter
import hashlib
import io
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path[:0] = [str(ROOT/'tools'),str(ROOT/'target/rea/capstone-runtime')]
from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
from cell_schema_probe import SourceImage, SOURCE_SHA256
from fontTools.ttLib import TTFont
from native_archive_index_probe import decode as native_index
from refpack import decompress
from refpack_bulk import install as checked_bulk_copy
from spui_font_outline_probe import FACES
from spui_font_scaling_probe import Machine, STACK, signed
from spui_font_hint_program_probe import load_source as bootstrap_source, full_bootstrap

FRAME, STATE, MAIN, FONT_PROGRAM, FUNCTIONS, OPERANDS = 0x300000,0x310000,0x320000,0x350000,0x330000,0x340000
BODY = bytes([0xB0,1,0x60,0x2D])
ALIASES = [-2147483648,-2147483647,-65536,-32768,-32767,-1,65536,65537,2147483647]
CASES = [('call',0,0),('call',0,2147483647)]+[
    ('loop',count,7) for count in [0,1,2,32767,32768,32769,65535,-32768,65537,-2147483647]]


def sha(data):
    return hashlib.sha256(data).hexdigest()


class ControlMachine(Machine):
    def __init__(self,instructions):
        super().__init__(instructions)
        self.calls = 0
        self.dispatches = 0

    def apply(self,ins):
        if ins.mnemonic == 'nop':
            self.steps += 1
            return ins.address+ins.size
        if ins.mnemonic == 'call':
            self.steps += 1
            target = self.operand(ins,ins.operands[0])
            if ins.address in [0x8AE833,0x8AE903]:
                assert target == 0x8ABCA0
                assert ins.operands[0].type == (X86_OP_REG if ins.address == 0x8AE833 else X86_OP_MEM)
                assert self.word(self.reg('esp')) == FRAME
                self.calls += 1
            else:
                assert ins.address == 0x8ABCD0 and ins.op_str == 'ecx'
                opcode = self.word(FRAME+4)
                assert (opcode,target) in [(0xB0,0x8ADB00),(0x60,0x8ADF70)]
                self.dispatches += 1
            self.reg('esp',self.reg('esp')-4)
            self.word(self.reg('esp'),value=ins.address+ins.size)
            return target
        if ins.mnemonic == 'jmp' and ins.operands[0].type == X86_OP_MEM:
            self.steps += 1
            assert ins.address == 0x8ADF9B and self.word(FRAME+4) == 0x60
            target = self.operand(ins,ins.operands[0])
            assert target == 0x8AE041
            return target
        return super().apply(ins)


def context(instructions,tables,program=BODY,definitions=None,capacity=2):
    if definitions is None:
        definitions = [dict(index=0,offset=0,length=3,selector=1)]
    m = ControlMachine(instructions)
    m.region(STACK-4096,bytes(8192))
    frame,state = bytearray(0x60),bytearray(0xD0)
    struct.pack_into('<I',frame,0,0xFEEDFACE)
    struct.pack_into('<I',frame,0x28,STATE)
    struct.pack_into('<I',frame,0x50,0x8ABCA0)
    struct.pack_into('<I',state,0x8C,0x1432F30)
    struct.pack_into('<I',state,0x90,FUNCTIONS)
    struct.pack_into('<I',state,0x9C,MAIN)
    struct.pack_into('<I',state,0xA0,FONT_PROGRAM)
    state[0xC9] = 0
    bindings = bytearray(b'\xA5'*(capacity*8))
    for f in definitions:
        struct.pack_into('<IHH',bindings,f['index']*8,f['offset'],f['length'],f['selector'])
    m.region(FRAME,frame)
    m.region(STATE,state,False)
    m.region(FONT_PROGRAM,program,False)
    m.region(FUNCTIONS,bindings,False)
    m.region(OPERANDS,b'\xA5'*32)
    for at,raw in tables.items():
        m.region(at,raw,False)
    return m


def reset(m,kind,count=0,seed=7,index=0):
    m.reg('esp',STACK)
    for name,value in dict(ebx=0x11223344,esi=0x22334455,edi=0x33445566,ebp=0x44556677).items():
        m.reg(name,value)
    m.word(STACK,value=0xFFFFFFFF)
    m.word(STACK+4,value=FRAME)
    m.word(FRAME,value=0xFEEDFACE)
    m.word(FRAME+4,value=0x2A if kind == 'loop' else 0x2B)
    values = [seed,count,index] if kind == 'loop' else [seed,index]
    for n,value in enumerate(values):
        m.word(OPERANDS+n*4,value=value)
    m.word(FRAME+0x20,value=OPERANDS+len(values)*4)


def count_prefix(m,count):
    before = m.steps
    reset(m,'loop',count)
    m.run(0x8AE8B0,0x8AE8FE)
    iterations = 0 if m.condition('jl') else m.reg('ebx')+1
    assert m.word(FRAME+0x20) == OPERANDS+4 and m.word(OPERANDS) == 7
    assert m.reg('ebp') == FONT_PROGRAM and m.reg('edi') == FONT_PROGRAM+3
    assert m.word(STACK+4) == 0x8ABCA0 and m.word(FRAME) == 0xFEEDFACE
    return iterations,m.steps-before


def full_owned(m,kind,count,seed):
    reset(m,kind,count,seed)
    before_steps,before_calls,before_dispatches = m.steps,m.calls,m.dispatches
    m.run(0x8AE8B0 if kind == 'loop' else 0x8AE800,0xFFFFFFFF,10000000)
    calls,dispatches = m.calls-before_calls,m.dispatches-before_dispatches
    assert m.reg('esp') == STACK+4
    assert all(m.reg(name) == value for name,value in dict(ebx=0x11223344,esi=0x22334455,edi=0x33445566,ebp=0x44556677).items())
    assert m.word(FRAME) == 0xFEEDFACE and m.word(FRAME+0x20) == OPERANDS+4
    assert m.word(STATE+0xC9,1) == 0 and m.word(FRAME+4) == (0x60 if calls else 0x2A)
    assert dispatches == calls*2
    result = dict(kind=kind,count=count,seed=seed,calls=calls,body_dispatches=dispatches,
        value=signed(m.word(OPERANDS)),cursor=m.word(FRAME),last_opcode=m.word(FRAME+4),
        state_program_selector=m.word(STATE+0xC9,1),stack_elements=1,
        caller_arg_slot=m.word(STACK+4),steps=m.steps-before_steps)
    return result


def extend_source(image,instructions,tables,proofs):
    frontier = json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text())
    cs = Cs(CS_ARCH_X86,CS_MODE_32)
    cs.detail = True
    for entry in [0x8AE800,0x8AE8B0,0x8ADF70]:
        f = next(f for f in frontier['functions'] if int(f['entry'],16) == entry)
        raw = b''.join(image.read(int(a,16),int(b,16)-int(a,16)) for a,b in f['ranges'])
        assert sha(raw) == f['sha256']
        for a,b in f['ranges']:
            a,b = int(a,16),int(b,16)
            decoded = list(cs.disasm(image.read(a,b-a),a))
            assert sum(row.size for row in decoded) == b-a
            instructions.update({row.address:row for row in decoded})
        proofs.append(dict(entry=f'{entry:08X}',ranges=f['ranges'],sha256=f['sha256']))
    tables[0x8AE0A4] = image.read(0x8AE0A4,121)
    index = tables[0x8AE0A4][0x3C+0x60-0x50]
    assert struct.unpack_from('<I',tables[0x8AE0A4],index*4)[0] == 0x8AE041
    assert struct.unpack_from('<I',tables[0x1432F30],0x60*4)[0] == 0x8ADF70
    pins = {0x8AE804:('add','dword ptr [eax + 0x20], -4'),
        0x8AE81A:('movzx','esi, word ptr [edx + 6]'),
        0x8AE81E:('mov','ecx, dword ptr [ecx + esi*4 + 0x9c]'),
        0x8AE830:('mov','eax, dword ptr [eax + 0x50]'),0x8AE833:('call','eax'),
        0x8AE8E5:('mov','ax, word ptr [eax]'),0x8AE8EA:('dec','ax'),
        0x8AE8EF:('movzx','ebx, ax'),0x8AE8F7:('mov','dword ptr [esp + 0x14], edx'),
        0x8AE8FB:('test','bx, bx'),0x8AE8FE:('jl','0x8ae910'),
        0x8AE903:('call','dword ptr [esp + 0x20]'),0x8AE907:('dec','ebx'),
        0x8AE90B:('test','bx, bx'),0x8AE90E:('jge','0x8ae900'),
        0x8ABCAF:('mov','ebp, dword ptr [esi]'),0x8ABCD9:('pop','edi'),
        0x8ABCDA:('mov','dword ptr [esi], ebp')}
    for at,expected in pins.items():
        assert (instructions[at].mnemonic,instructions[at].op_str) == expected
    return pins


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--package',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--report',type=Path,required=True)
    args = p.parse_args()
    for path in [args.out,args.report]:
        path.resolve().relative_to((ROOT/'target').resolve())
        assert not path.exists()
    image = SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    instructions,tables,proofs = bootstrap_source(image)
    pins = extend_source(image,instructions,tables,proofs)
    totals,state = Counter(),hashlib.sha256()
    checked_bulk_copy()
    with args.out.open('xb') as output:
        def emit(line):
            data = (line+'\n').encode('ascii')
            output.write(data)
            state.update(data)
            totals['rows'] += 1
        prefix = context(instructions,tables)
        distribution = Counter()
        for count in list(range(65536))+ALIASES:
            iterations,steps = count_prefix(prefix,count)
            totals['instruction_steps'] += steps
            distribution['zero' if iterations == 0 else 'positive'] += 1
            emit(f'count\t{count}\t{iterations}')
        own = context(instructions,tables)
        owned_reports = []
        for kind,count,seed in CASES:
            actual = full_owned(own,kind,count,seed)
            owned_reports.append(actual)
            totals['instruction_steps'] += actual['steps']
            emit(f'owned\t{kind}\t{count}\t{seed}\t{actual["calls"]}\t{actual["value"]}\t{actual["cursor"]}\t{actual["last_opcode"]}\t{actual["state_program_selector"]}\t{actual["stack_elements"]}')
        reports = []
        with args.package.open('rb') as archive:
            header = archive.read(96)
            count,length,offset = (struct.unpack_from('<I',header,at)[0] for at in [0x24,0x2C,0x40])
            assert header[:4] == b'DBPF'
            archive.seek(offset)
            block = archive.read(length)
            flags,records = native_index(block,count)
            faces = [row for row in records if row[1] == 0x027C5CEF and row[2] == 0x0248E873]
            assert len(faces) == 5 and {f'{r[0]:08X}' for r in faces} == set(FACES)
            for instance,kind,scope,at,stored,decoded,compression,_,_ in sorted(faces):
                archive.seek(at)
                raw = archive.read(stored)
                assert compression in [0,0xFFFF]
                raw = raw if compression == 0 else decompress(raw,out_size=decoded,header=True)[0]
                assert sha(raw) == FACES[f'{instance:08X}']
                font = TTFont(io.BytesIO(raw))
                program = font.getTableData('fpgm') if 'fpgm' in font else b''
                limits = font['maxp']
                setup = full_bootstrap(instructions,{k:v for k,v in tables.items() if k != 0x8AE0A4},program,limits.maxFunctionDefs,limits.maxStackElements)
                totals['instruction_steps'] += setup['steps']
                key = f'{kind:08X}:{scope:08X}:{instance:08X}'
                emit(f'{key}\tbindings\t{len(setup["functions"])}')
                record_count = 0
                for f in setup['functions']:
                    m = context(instructions,tables,program,setup['functions'],limits.maxFunctionDefs)
                    reset(m,'call',index=f['index'])
                    m.run(0x8AE800,0x8AE833)
                    stack = m.reg('esp')
                    actual = [m.word(stack+n*4) for n in range(3)]
                    assert actual == [FRAME,FONT_PROGRAM+f['offset'],FONT_PROGRAM+f['offset']+f['length']]
                    assert m.word(FRAME+0x20) == OPERANDS+4 and m.word(FRAME) == 0xFEEDFACE
                    assert m.calls == m.dispatches == 0 and m.word(STATE+0xC9,1) == 0
                    totals['instruction_steps'] += m.steps
                    emit(f'{key}\tresolve\t{f["index"]}\t{actual[1]}\t{actual[2]}\t{f["selector"]}\t1')
                    record_count += 1
                totals['resolved_installed_bindings'] += record_count
                reports.append(dict(key=key,font_sha256=sha(raw),program_bytes=len(program),
                                    defined_bindings=record_count,body_entry_stopped=True))
    assert totals['resolved_installed_bindings'] == 137
    result = dict(schema='source-instruction-ordinary-hint-call-foundation-v1',source_sha256=SOURCE_SHA256,
        tool_sha256=sha(Path(__file__).read_bytes()),
        unexported_port_sha256=sha((ROOT/'crates/spore-core/src/spui_font_hint_control.rs').read_bytes()),
        pending_verifier_sha256=sha((ROOT/'crates/spore-bevy/examples/spui_font_hint_control_verify.rs.pending').read_bytes()),
        borrowed_bootstrap_tool_sha256=sha((ROOT/'tools/spui_font_hint_program_probe.py').read_bytes()),
        source_code_proofs=proofs,source_data_proofs=[dict(address=f'{at:08X}',bytes=len(data),sha256=sha(data)) for at,data in tables.items()],
        projected_operands={f'{at:08X}':list(pair) for at,pair in pins.items()},
        source_index_flags=f'{flags:08X}',source_index_sha256=sha(block),faces=reports,
        count_word_inputs=65536,additional_upper_word_aliases=ALIASES,count_distribution=dict(distribution),
        owned_body=BODY.hex(),owned_body_is_source_font_payload=False,owned_body_full_cases=owned_reports,
        native_call_argument_slot_overwrite_observed_in_model=True,source_font_bodies_executed=False,
        totals=dict(totals),canonical_bytes=args.out.stat().st_size,canonical_sha256=state.hexdigest(),
        native_machine_code_executed=False,
        scope='CALL/LOOPCALL preparation plus original ordinary callback/shared operand stack/cursor restoration. '
              'Full execution uses only the owned PUSHB1/ADD body; shared ADD entry is admitted only for constant opcode60. '
              'Installed FPGM cases stop at source CALL before body entry. Source selector byte remains caller0 while target '
              'record selects FPGM1. Actual PREP, source function-body execution, glyph hints, other arithmetic handlers, '
              'native recovery and raster outputs remain outside this gate. Modeled addresses are explicit fixture inputs.')
    with args.report.open('x') as handle:
        handle.write(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(rows=totals['rows'],instruction_steps=totals['instruction_steps'],
                          canonical_bytes=result['canonical_bytes'],sha256=result['canonical_sha256'])))


if __name__ == '__main__':
    main()
