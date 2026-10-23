"""Source-instruction oracle for original FPGM PUSH/FDEF setup only.

Executes no native machine code. The closed integer model interprets the
original ordinary dispatch loop, PUSH handlers, FDEF scanner and checked
original switch/dispatch table reads. Bodies stay opaque. All font payloads,
source disassembly and generated canonical data remain under target.
"""
import argparse
from collections import Counter
import hashlib
import io
import json
from pathlib import Path
import re
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path[:0] = [str(ROOT / 'tools'), str(ROOT / 'target/rea/capstone-runtime')]
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG
from cell_schema_probe import SourceImage, SOURCE_SHA256
from fontTools.ttLib import TTFont
from native_archive_index_probe import decode as native_index
from refpack import decompress
from refpack_bulk import install as checked_bulk_copy
from spui_font_outline_probe import FACES
from spui_font_scaling_probe import Machine, STACK, signed

FRAME, STATE, PROGRAM, FUNCTIONS, OPERANDS = 0x300000, 0x310000, 0x320000, 0x330000, 0x340000
HANDLERS = {0x40: 0x8ADB40, 0x41: 0x8ADBD0, 0x2C: 0x8AE840}
HANDLERS.update({opcode: 0x8ADB00 for opcode in range(0xB0, 0xB8)})
HANDLERS.update({opcode: 0x8ADB80 for opcode in range(0xB8, 0xC0)})
SCANNER_TARGETS = {0x8AE1FB, 0x8AE205, 0x8AE20F, 0x8AE219, 0x8AE220}
ENTRIES = [0x8ABCA0, 0x8ADB00, 0x8ADB40, 0x8ADB80, 0x8ADBD0,
           0x8AE840, 0x8AE1E0, 0x8CD330, 0x8CBE80]
BASE_ORACLE_SHA256 = 'f40f532a3148b7b87cc392ec108d44a6399addec747860dbb901fd348260454c'
SAMPLES = [
    ('empty', b'', 3, 16),
    ('npushb0', bytes([0x40, 0]), 3, 16),
    ('npushw0', bytes([0x41, 0]), 3, 16),
    ('npushb3', bytes([0x40, 3, 0, 128, 255]), 3, 16),
    ('pushb2', bytes([0xB1, 45, 255]), 3, 16),
    ('signedwords', bytes([0x41, 3, 0xFF, 0xFF, 0x80, 0, 0x7F, 0xFF]), 3, 16),
    ('fixedwords', bytes([0xB9, 0xFF, 0xFF, 0x80, 0]), 3, 16),
    ('opaque_endf_data', bytes([0xB0, 2, 0x2C, 0x41, 1, 0x2D, 0x2D, 0xB1, 0x2D, 0x2D, 0x2D]), 3, 16),
    ('overwrite', bytes([0xB0, 1, 0x2C, 0x21, 0x2D, 0xB0, 1, 0x2C, 0, 0x2D]), 3, 16),
]


def sha(data):
    return hashlib.sha256(data).hexdigest()


class ProgramMachine(Machine):
    """Only the original setup's explicitly closed indirect transfers."""
    def __init__(self, instructions):
        super().__init__(instructions)
        self.dispatches = []
        self.peak = 0

    def apply(self, ins):
        if ins.address == 0x8ABCD2:
            latest = self.dispatches[-1]
            latest['next_pc'] = self.word(FRAME) - PROGRAM
            latest['stack_after'] = (self.word(FRAME + 0x20) - OPERANDS) // 4
            self.peak = max(self.peak, latest['stack_after'])
        if ins.mnemonic == 'nop':
            self.steps += 1
            return ins.address + ins.size
        if ins.mnemonic == 'call':
            self.steps += 1
            operand = ins.operands[0]
            target = self.operand(ins, operand)
            if ins.address == 0x8ABCD0:
                assert operand.type == X86_OP_REG and ins.op_str == 'ecx'
                opcode = self.word(FRAME + 4)
                assert opcode in HANDLERS and HANDLERS[opcode] == target
                count = (self.word(FRAME + 0x20) - OPERANDS) // 4
                index = signed(self.word(OPERANDS + (count-1)*4)) if opcode == 0x2C else None
                self.dispatches.append(dict(pc=self.word(FRAME)-PROGRAM-1, opcode=opcode,
                                             handler=target, defined_index=index))
            else:
                assert ins.address == 0x8AE885 and operand.type == X86_OP_IMM and target == 0x8AE1E0
            self.reg('esp', self.reg('esp')-4)
            self.word(self.reg('esp'), value=ins.address + ins.size)
            return target
        if ins.mnemonic == 'jmp' and ins.operands[0].type == X86_OP_MEM:
            self.steps += 1
            assert ins.address == 0x8AE1F4
            target = self.operand(ins, ins.operands[0])
            assert target in SCANNER_TARGETS
            return target
        return super().apply(ins)


def machine(instructions, tables, program, stack_capacity, function_capacity):
    m = ProgramMachine(instructions)
    m.region(STACK-256, bytes(512))
    m.reg('esp', STACK)
    frame, state = bytearray(0x2C), bytearray(0xD0)
    struct.pack_into('<I', frame, 0, 0xFEEDFACE)
    struct.pack_into('<I', frame, 0x20, OPERANDS)
    struct.pack_into('<I', frame, 0x28, STATE)
    struct.pack_into('<I', state, 0x8C, 0x1432F30)
    struct.pack_into('<I', state, 0x90, FUNCTIONS)
    struct.pack_into('<I', state, 0xA0, PROGRAM)
    state[0xC9] = 1
    m.region(FRAME, frame)
    m.region(STATE, state, False)
    m.region(PROGRAM, program, False)
    m.region(FUNCTIONS, b'\xA5'*(function_capacity*8))
    m.region(OPERANDS, b'\xA5'*(stack_capacity*4))
    for address, raw in tables.items():
        m.region(address, raw, False)
    return m


def full_bootstrap(instructions, tables, program, function_capacity, stack_capacity):
    m = machine(instructions, tables, program, stack_capacity, function_capacity)
    sentinels = dict(ebx=0x11223344, esi=0x22334455, edi=0x33445566, ebp=0x44556677)
    for name, value in sentinels.items():
        m.reg(name, value)
    for n, value in enumerate([0xFFFFFFFF, FRAME, PROGRAM, PROGRAM + len(program)]):
        m.word(STACK+n*4, value=value)
    m.run(0x8ABCA0, 0xFFFFFFFF, 1000000)
    assert m.reg('esp') == STACK+4 and m.word(FRAME) == 0xFEEDFACE
    assert all(m.reg(name) == value for name, value in sentinels.items())
    top = m.word(FRAME+0x20)
    assert OPERANDS <= top <= OPERANDS+stack_capacity*4 and (top-OPERANDS) % 4 == 0
    stack = [signed(m.word(address)) for address in range(OPERANDS, top, 4)]
    defined = {step['defined_index'] for step in m.dispatches if step['defined_index'] is not None}
    functions = []
    for index in range(function_capacity):
        row = m.bytes(FUNCTIONS+index*8, 8)
        if index not in defined:
            assert row == b'\xA5'*8
            continue
        start, length, selector = struct.unpack('<IHH', row)
        assert selector == 1 and start + length < len(program) and program[start+length] == 0x2D
        functions.append(dict(index=index, offset=start, length=length, selector=selector,
                              body=program[start:start+length].hex()))
    return dict(functions=functions, dispatches=m.dispatches, remaining_stack=stack,
                peak=m.peak, steps=m.steps, unused_function_slots_preserved=True,
                cpu_nonvolatile_registers_restored=True, frame_pc_restored=True)


def scanner_end(instructions, tables, program):
    m = machine(instructions, tables, program, 0, 0)
    m.word(FRAME, value=PROGRAM+1)
    m.word(FRAME+4, value=program[0])
    m.word(STACK, value=0xFFFFFFFF)
    m.reg('esi', FRAME)
    m.run(0x8AE1E0, 0xFFFFFFFF)
    assert m.reg('esp') == STACK+4 and m.reg('esi') == FRAME
    return m.word(FRAME)-PROGRAM, m.steps


def word_table(instructions):
    m = Machine(instructions)
    m.region(PROGRAM, bytes(2))
    m.region(OPERANDS, bytes(4))
    m.reg('edx', OPERANDS)
    m.reg('ecx', 0xA5A50000)
    m.reg('ebx', 0x5A5A0000)
    values = []
    for raw in range(65536):
        m.bytes(PROGRAM, 2, raw.to_bytes(2, 'big'))
        m.reg('eax', PROGRAM)
        for pc in [0x8ADBA0, 0x8ADBA4, 0x8ADBA9, 0x8ADBAC, 0x8ADBAD, 0x8ADBB0, 0x8ADBB3, 0x8ADBB6]:
            m.apply(instructions[pc])
        values.append(signed(m.word(OPERANDS)))
    assert [values[n] for n in [0, 255, 32767, 32768, 65535]] == [0, 255, 32767, -32768, -1]
    return values, m.steps


def load_source(image):
    frontier = json.loads((ROOT/'target/source-ui/font-fixture-frontier-v7.json').read_text())
    cs = Cs(CS_ARCH_X86, CS_MODE_32)
    cs.detail = True
    instructions, proofs = {}, []
    for entry in ENTRIES:
        body = next(body for body in frontier['functions'] if int(body['entry'],16) == entry)
        code = b''.join(image.read(int(a,16),int(b,16)-int(a,16)) for a,b in body['ranges'])
        assert sha(code) == body['sha256']
        for a,b in body['ranges']:
            start,end = int(a,16),int(b,16)
            decoded = list(cs.disasm(image.read(start,end-start),start))
            assert sum(row.size for row in decoded) == end-start
            instructions.update({row.address:row for row in decoded})
        proofs.append(dict(entry=f'{entry:08X}',ranges=body['ranges'],sha256=body['sha256']))
    tables = {0x1432F30:image.read(0x1432F30,1024), 0x8AE224:image.read(0x8AE224,20),
              0x8AE238:image.read(0x8AE238,128)}
    dispatch = struct.unpack('<256I',tables[0x1432F30])
    assert all(dispatch[opcode] == target for opcode,target in HANDLERS.items())
    targets = struct.unpack('<5I',tables[0x8AE224])
    assert set(targets) == SCANNER_TARGETS
    for opcode in range(0x40,0xC0):
        expected = 0x8AE1FB if opcode == 0x40 else 0x8AE205 if opcode == 0x41 else 0x8AE20F if 0xB0 <= opcode <= 0xB7 else 0x8AE219 if 0xB8 <= opcode <= 0xBF else 0x8AE220
        assert targets[tables[0x8AE238][opcode-0x40]] == expected
    pins = {0x8ABCC3:('movzx','ecx, byte ptr [eax]'),
            0x8ABCC9:('mov','ecx, dword ptr [edi + ecx*4]'), 0x8ABCD0:('call','ecx'),
            0x8ADB60:('movzx','edi, byte ptr [edx]'), 0x8ADBB3:('movsx','ecx, cx'),
            0x8ADC03:('movsx','ecx, cx'), 0x8AE859:('lea','edi, [edx + ecx*8]'),
            0x8AE864:('mov','word ptr [edi + 6], cx'), 0x8AE874:('mov','dword ptr [edi], edx'),
            0x8AE8A1:('mov','word ptr [edi + 4], cx'),
            0x8CD3AD:('mov','word ptr [esi + 0x18], ax'),
            0x8CD3C1:('mov','word ptr [esi + 0x1c], ax')}
    for pc,expected in pins.items():
        assert (instructions[pc].mnemonic,instructions[pc].op_str) == expected
    return instructions,tables,proofs


def independent_assembly_ids(font):
    if 'fpgm' not in font:
        return []
    assembly = font['fpgm'].program.getAssembly()
    stack, ids, cursor = [], [], 0
    while cursor < len(assembly):
        line = assembly[cursor]
        if line.startswith(('NPUSHB[','NPUSHW[','PUSHB[','PUSHW[')):
            cursor += 1
            while cursor < len(assembly) and re.fullmatch(r'-?\d+',assembly[cursor]):
                stack.append(int(assembly[cursor]))
                cursor += 1
        else:
            assert line.startswith('FDEF[') and stack
            ids.append(stack.pop())
            cursor += 1
            while not assembly[cursor].startswith('ENDF['):
                assert not assembly[cursor].startswith('FDEF[')
                cursor += 1
            cursor += 1
    assert not stack
    return ids


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True)
    args = parser.parse_args()
    for path in [args.out,args.report]:
        path.resolve().relative_to((ROOT/'target').resolve())
        assert not path.exists()
    assert sha((ROOT/'tools/spui_font_scaling_probe.py').read_bytes()) == BASE_ORACLE_SHA256
    image = SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    instructions,tables,proofs = load_source(image)
    totals,state = Counter(),hashlib.sha256()
    checked_bulk_copy()
    with args.out.open('xb') as output:
        def emit(line):
            data = (line+'\n').encode('ascii')
            output.write(data)
            state.update(data)
            totals['rows'] += 1

        def emit_setup(key,program,capacity,stack_capacity,setup):
            emit(f'{key}\tlimits\t{capacity}\t{stack_capacity}\t{len(program)}\t{setup["peak"]}\t'+','.join(map(str,setup['remaining_stack'])))
            for ordinal,step in enumerate(setup['dispatches']):
                defined = '-' if step['defined_index'] is None else str(step['defined_index'])
                emit(f'{key}\tstep\t{ordinal}\t{step["pc"]}\t{step["opcode"]:02X}\t{step["next_pc"]}\t{step["stack_after"]}\t{defined}')
            for f in setup['functions']:
                emit(f'{key}\tfunction\t{f["index"]}\t{f["offset"]}\t{f["length"]}\t{f["selector"]}\t{f["body"]}')
            totals['instruction_steps'] += setup['steps']

        for opcode in range(256):
            counts = range(256) if opcode in [0x40,0x41] else [-1]
            for count in counts:
                if opcode in [0x40,0x41]:
                    program = bytes([opcode,count])+b'\x2D'*(count*(1 if opcode == 0x40 else 2))
                elif 0xB0 <= opcode <= 0xB7:
                    program = bytes([opcode])+b'\x2D'*(opcode-0xAF)
                elif 0xB8 <= opcode <= 0xBF:
                    program = bytes([opcode])+b'\x2D'*(2*(opcode-0xB7))
                else:
                    program = bytes([opcode])
                end,steps = scanner_end(instructions,tables,program)
                assert end == len(program)
                totals.update(instruction_steps=steps,scanner_fixtures=1)
                emit(f'scan\t{opcode:02X}\t{count}\t{end}')
        words,steps = word_table(instructions)
        totals['instruction_steps'] += steps
        for raw,value in enumerate(words):
            emit(f'word\t{raw}\t{value}')
        for name,program,capacity,stack_capacity in SAMPLES:
            setup = full_bootstrap(instructions,tables,program,capacity,stack_capacity)
            emit_setup('sample:'+name,program,capacity,stack_capacity,setup)

        with args.package.open('rb') as archive:
            header = archive.read(96)
            count,length,offset = (struct.unpack_from('<I',header,p)[0] for p in [0x24,0x2C,0x40])
            assert header[:4] == b'DBPF'
            archive.seek(offset)
            block = archive.read(length)
            flags,records = native_index(block,count)
            faces = [r for r in records if r[1] == 0x027C5CEF and r[2] == 0x0248E873]
            assert len(faces) == 5 and {f'{r[0]:08X}' for r in faces} == set(FACES)
            reports = []
            for instance,kind,scope,at,stored,decoded,compression,_,_ in sorted(faces):
                archive.seek(at)
                data = archive.read(stored)
                assert compression in [0,0xFFFF]
                data = data if compression == 0 else decompress(data,out_size=decoded,header=True)[0]
                assert sha(data) == FACES[f'{instance:08X}'] and len(data) == decoded
                font = TTFont(io.BytesIO(data))
                maxp = font.getTableData('maxp')
                assert len(maxp) >= 32 and maxp[:4] == b'\0\1\0\0'
                capacity = struct.unpack_from('>H',maxp,20)[0]
                stack_capacity = struct.unpack_from('>H',maxp,24)[0]
                program = font.getTableData('fpgm') if 'fpgm' in font else b''
                setup = full_bootstrap(instructions,tables,program,capacity,stack_capacity)
                key = f'{kind:08X}:{scope:08X}:{instance:08X}'
                emit_setup(key,program,capacity,stack_capacity,setup)
                ids = independent_assembly_ids(font)
                assert ids == [step['defined_index'] for step in setup['dispatches'] if step['defined_index'] is not None]
                assert capacity == font['maxp'].maxFunctionDefs and stack_capacity == font['maxp'].maxStackElements
                assert not setup['remaining_stack']
                if instance == 0x511B49CA:
                    native_header = ROOT/'target/source-ui/font-glyph-native-v2/native-font-payload.h'
                    text = native_header.read_text().split('font_definitions[] = {',1)[1].split('};',1)[0]
                    entries = [tuple(map(int,row)) for row in re.findall(r'\{(\d+)UL,(\d+)UL,(\d+)UL\}',text)]
                    assert entries == [(f['index'],f['offset'],f['length']) for f in setup['functions']]
                reports.append(dict(key=key,font_sha256=sha(data),has_fpgm='fpgm' in font,
                    fpgm_bytes=len(program),fpgm_sha256=sha(program),max_functions=capacity,
                    max_stack_elements=stack_capacity,peak=setup['peak'],
                    top_level_dispatch_count=len(setup['dispatches']),definitions=setup['functions'],
                    top_level_handlers=sorted({f'{step["handler"]:08X}' for step in setup['dispatches']}),
                    independent_fonttools_definition_order_agrees=True,remaining_stack_elements=0,
                    unused_native_table_model_slots_A5_preserved=setup['unused_function_slots_preserved']))
                totals.update(source_faces=1,source_definitions=len(setup['functions']),
                              source_dispatches=len(setup['dispatches']))
    assert totals['scanner_fixtures'] == 766 and totals['source_definitions'] == 137
    assert totals['source_dispatches'] == 159 and totals['source_faces'] == 5
    report = dict(schema='source-instruction-fpgm-bootstrap-oracle-v1',source_sha256=SOURCE_SHA256,
        source_proofs=proofs,source_data_proofs=[dict(address=f'{at:08X}',bytes=len(raw),sha256=sha(raw)) for at,raw in tables.items()],
        native_handlers={f'{opcode:02X}':f'{target:08X}' for opcode,target in HANDLERS.items()},
        borrowed_integer_model_sha256=BASE_ORACLE_SHA256,
        tool_sha256=sha(Path(__file__).read_bytes()),
        unexported_port_sha256=sha((ROOT/'crates/spore-core/src/spui_font_hint_program.rs').read_bytes()),
        rust_example_sha256=sha((ROOT/'crates/spore-bevy/examples/spui_font_hint_program_verify.rs').read_bytes()),
        source_index_flags=f'{flags:08X}',source_index_sha256=sha(block),faces=reports,totals=dict(totals),
        signed_push_word_inputs=65536,owned_setup_samples=len(SAMPLES),canonical_bytes=args.out.stat().st_size,
        canonical_sha256=state.hexdigest(),native_machine_code_executed=False,
        full_original_dispatch_loop_and_push_fdef_handlers_interpreted=True,
        source_font0_native_host_definition_assertions_overlap=53,
        native_font0_header_sha256=sha((ROOT/'target/source-ui/font-glyph-native-v2/native-font-payload.h').read_bytes()),
        scope='Initially empty operand stack, explicit maxp1.0 table/stack capacities, FPGM selector1. '
              'All four installed FPGM streams use only PUSH/FDEF top-level; absent FPGM modeled empty. '
              'Function bodies remain borrowed/opaque and never execute. Original native font0 host '
              'already asserts all53 defined entries; other faces and synthetic cases use this source '
              'instruction reference only. No claim for unused native table initialization, malformed '
              'native recovery, CALL/PREP/glyph VM state, point hinting, rasterization or retail dispatch.')
    with args.report.open('x') as handle:
        handle.write(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(totals=dict(totals),canonical_bytes=report['canonical_bytes'],sha256=report['canonical_sha256'])))


if __name__ == '__main__':
    main()
