"""Source-pinned integer instruction oracle for simple glyph scale inputs.

No native code runs. A small, fail-closed interpreter evaluates the actual
pinned integer instructions for coefficient building, fixed division, hhea
fallback and the simple phantom suffix. Selector-0's four data-flow
instructions are evaluated over every signed word, then used to census real
font packets. Other selectors and non-simple glyphs remain explicit exclusions.
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
sys.path[:0] = [str(ROOT / 'tools'), str(ROOT / 'target/rea/capstone-runtime')]
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG
from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_archive_index_probe import decode as native_index
from refpack import decompress
from refpack_bulk import install as checked_bulk_copy
from spui_font_outline_probe import FACES
from fontTools.ttLib import TTFont


def u32(value):
    return value & 0xFFFFFFFF


def signed(value, bits=32):
    value &= (1 << bits) - 1
    return value - (1 << bits) if value & (1 << (bits - 1)) else value


ALIASES = {r: (r, 32, 0) for r in ['eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'esp', 'ebp']}
for short, full in [('ax', 'eax'), ('bx', 'ebx'), ('cx', 'ecx'), ('dx', 'edx'),
                    ('si', 'esi'), ('di', 'edi'), ('sp', 'esp'), ('bp', 'ebp')]:
    ALIASES[short] = full, 16, 0
for prefix in 'abcd':
    ALIASES[prefix + 'l'] = 'e' + prefix + 'x', 8, 0
    ALIASES[prefix + 'h'] = 'e' + prefix + 'x', 8, 8


class Machine:
    """Only the integer operations used by the reviewed source slices."""
    def __init__(self, instructions):
        self.instructions = instructions
        self.r = {r: 0 for r in ['eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'esp', 'ebp']}
        self.memory = []
        self.flags = dict(z=False, s=False, c=False, o=False)
        self.steps = 0

    def region(self, address, data, writable=True):
        assert all(address + len(data) <= a or a + len(b) <= address for a, b, _ in self.memory)
        self.memory.append((address, bytearray(data), writable))

    def bytes(self, address, count, value=None):
        for start, data, writable in self.memory:
            if start <= address and address + count <= start + len(data):
                at = address - start
                if value is not None:
                    assert writable and len(value) == count
                    data[at:at + count] = value
                return bytes(data[at:at + count])
        raise AssertionError(f'unmapped model memory {address:08X}+{count}')

    def word(self, address, size=4, value=None):
        if value is not None:
            self.bytes(address, size, (value & ((1 << (size * 8)) - 1)).to_bytes(size, 'little'))
        return int.from_bytes(self.bytes(address, size), 'little')

    def reg(self, name, value=None):
        base, bits, shift = ALIASES[name]
        mask = ((1 << bits) - 1) << shift
        if value is not None:
            self.r[base] = u32((self.r[base] & ~mask) | ((value << shift) & mask))
        return (self.r[base] & mask) >> shift

    def address(self, ins, operand):
        m = operand.mem
        return u32(m.disp + (self.reg(ins.reg_name(m.base)) if m.base else 0)
                   + (self.reg(ins.reg_name(m.index)) * m.scale if m.index else 0))

    def operand(self, ins, operand, value=None):
        if operand.type == X86_OP_REG:
            return self.reg(ins.reg_name(operand.reg), value)
        if operand.type == X86_OP_MEM:
            return self.word(self.address(ins, operand), operand.size, value)
        assert operand.type == X86_OP_IMM and value is None
        return operand.imm

    def condition(self, name):
        def flag(key):
            value = self.flags[key]
            assert value is not None, f'branch consumed undefined flag {key}'
            return value
        return {'je': lambda: flag('z'), 'jne': lambda: not flag('z'),
                'jb': lambda: flag('c'), 'ja': lambda: not flag('c') and not flag('z'),
                'jbe': lambda: flag('c') or flag('z'), 'jae': lambda: not flag('c'),
                'jle': lambda: flag('z') or flag('s') != flag('o'),
                'jl': lambda: flag('s') != flag('o'), 'jge': lambda: flag('s') == flag('o'),
                'jns': lambda: not flag('s')}[name]()

    def arithmetic(self, name, a, b, bits):
        mask, sign = (1 << bits) - 1, 1 << (bits - 1)
        a, b = a & mask, b & mask
        if name in ('sub', 'cmp'):
            result = (a - b) & mask
            carry, overflow = a < b, bool((a ^ b) & (a ^ result) & sign)
        elif name == 'add':
            result = (a + b) & mask
            carry, overflow = a + b > mask, bool(~(a ^ b) & (a ^ result) & sign)
        else:
            result = {'xor': a ^ b, 'or': a | b, 'test': a & b}[name]
            carry = overflow = False
        self.flags = dict(z=result == 0, s=bool(result & sign), c=carry, o=overflow)
        return result

    def apply(self, ins):
        """Execute one decoded source instruction, returning its next PC."""
        self.steps += 1
        op, args = ins.mnemonic, ins.operands
        read = lambda n: self.operand(ins, args[n])
        write = lambda n, value: self.operand(ins, args[n], value)
        next_pc = ins.address + ins.size
        if op == 'mov':
            write(0, read(1))
        elif op in ('movsx', 'movzx'):
            value = read(1)
            write(0, signed(value, args[1].size * 8) if op == 'movsx' else value)
        elif op == 'lea':
            write(0, self.address(ins, args[1]))
        elif op in ('add', 'sub', 'xor', 'or', 'cmp', 'test'):
            result = self.arithmetic(op, read(0), read(1), args[0].size * 8)
            if op not in ('cmp', 'test'):
                write(0, result)
        elif op in ('inc', 'dec', 'neg'):
            old_carry = self.flags['c']
            if op == 'neg':
                result = self.arithmetic('sub', 0, read(0), args[0].size * 8)
            else:
                result = self.arithmetic('add' if op == 'inc' else 'sub', read(0), 1, args[0].size * 8)
                self.flags['c'] = old_carry
            write(0, result)
        elif op in ('shl', 'shr', 'sar'):
            count = read(1) & 31
            if count:
                value, bits = read(0), args[0].size * 8
                assert count <= bits
                result = value << count if op == 'shl' else value >> count if op == 'shr' else signed(value, bits) >> count
                write(0, result)
                narrowed = result & ((1 << bits) - 1)
                carry = bool(value & (1 << (bits - count if op == 'shl' else count - 1)))
                overflow = None if count != 1 else bool(narrowed >> (bits - 1)) != carry if op == 'shl' else bool(value >> (bits - 1)) if op == 'shr' else False
                self.flags = dict(z=narrowed == 0, s=bool(narrowed >> (bits - 1)), c=carry, o=overflow)
        elif op == 'imul':
            if len(args) == 1:
                product = signed(self.reg('eax')) * signed(read(0))
                self.reg('eax', product)
                self.reg('edx', product >> 32)
            else:
                assert len(args) == 2
                product = signed(read(0)) * signed(read(1))
                write(0, product)
            overflow = not -(1 << 31) <= product < (1 << 31)
            self.flags = dict(z=None, s=None, c=overflow, o=overflow)
        elif op == 'div':
            divisor = read(0)
            assert divisor
            quotient, remainder = divmod((self.reg('edx') << 32) | self.reg('eax'), divisor)
            assert quotient <= 0xFFFFFFFF
            self.reg('eax', quotient)
            self.reg('edx', remainder)
            self.flags = dict(z=None, s=None, c=None, o=None)
        elif op == 'cwde':
            self.reg('eax', signed(self.reg('ax'), 16))
        elif op == 'push':
            value = read(0)
            self.reg('esp', self.reg('esp') - 4)
            self.word(self.reg('esp'), value=value)
        elif op == 'pop':
            value = self.word(self.reg('esp'))
            self.reg('esp', self.reg('esp') + 4)
            write(0, value)
        elif op == 'call':
            assert args[0].type == X86_OP_IMM and read(0) == 0x8D0DB0
            self.reg('esp', self.reg('esp') - 4)
            self.word(self.reg('esp'), value=next_pc)
            next_pc = read(0)
        elif op == 'ret':
            assert not args
            next_pc = self.word(self.reg('esp'))
            self.reg('esp', self.reg('esp') + 4)
        elif op == 'jmp':
            assert args[0].type == X86_OP_IMM
            next_pc = read(0)
        elif op.startswith('j'):
            assert args[0].type == X86_OP_IMM
            if self.condition(op):
                next_pc = read(0)
        else:
            raise AssertionError(f'unadmitted model instruction {ins.address:08X} {op}')
        return next_pc

    def run(self, start, stop, limit=1000):
        pc = start
        for _ in range(limit):
            if pc == stop:
                return
            assert pc in self.instructions, f'unadmitted PC {pc:08X}'
            pc = self.apply(self.instructions[pc])
        raise AssertionError('bounded instruction model exceeded step limit')


STACK, HEADER, FONT, X, Y, COEFF = 0x50000, 0x100000, 0x110000, 0x120000, 0x130000, 0x140000


def with_stack(instructions):
    m = Machine(instructions)
    m.region(STACK - 256, bytes(512))
    m.reg('esp', STACK)
    return m


def coefficients(instructions, ppem, units):
    m = with_stack(instructions)
    m.region(COEFF, bytes([0xA5] * 20))
    for n, value in enumerate([0xFFFFFFFF, ppem, units, COEFF]):
        m.word(STACK + n * 4, value=value)
    m.run(0x8CC8C0, 0xFFFFFFFF)
    assert m.reg('esp') == STACK + 4 and m.bytes(COEFF + 18, 2) == b'\xA5\xA5'
    raw = m.bytes(COEFF, 20)
    mul, shift, den, add, fixed, selector = struct.unpack_from('<hhiiih', raw)
    return dict(multiplier=mul, shift=shift, denominator=den, rounding_addend=add,
                fixed_multiplier=fixed, selector=selector), raw, m.steps


def vertical(instructions, hhea):
    m = with_stack(instructions)
    head = bytearray(40)
    struct.pack_into('<3h', head, 8, hhea.ascent, hhea.descent, hhea.lineGap)
    font = bytearray(112)
    struct.pack_into('<I', font, 0x60, HEADER)
    m.region(HEADER, head, False)
    m.region(FONT, font, False)
    m.reg('esi', FONT)
    m.run(0x8CE8D2, 0x8CE8FF)
    return m.reg('ebp'), m.reg('edi'), m.steps


def phantom(instructions, glyph, advance, bearing, vadvance, top):
    m = with_stack(instructions)
    header = bytearray(0x78)
    struct.pack_into('<h', header, 0x6C, glyph.xMin)
    struct.pack_into('<h', header, 0x72, glyph.yMax)
    m.region(HEADER, header, False)
    count = len(glyph.coordinates)
    xs = b''.join(struct.pack('<h', x) for x, _ in glyph.coordinates)
    ys = b''.join(struct.pack('<h', y) for _, y in glyph.coordinates)
    m.region(X, xs + b'\xA5' * 8)
    m.region(Y, ys + b'\xA5' * 8)
    for offset, value in [(0x20, Y), (0x24, X), (0x30, bearing), (0x34, advance), (0x38, top), (0x3C, vadvance)]:
        m.word(STACK + offset, value=value)
    m.reg('ebp', HEADER)
    m.reg('edi', count)
    m.run(0x8CE1C4, 0x8CE228)
    assert m.bytes(X, len(xs)) == xs and m.bytes(Y, len(ys)) == ys
    assert m.reg('esp') == STACK and m.reg('edi') == count
    x = struct.unpack('<4h', m.bytes(X + count * 2, 8))
    y = struct.unpack('<4h', m.bytes(Y + count * 2, 8))
    return list(zip(x, y)), m.steps


def source_word_table(instructions, parameters, raw):
    m = Machine(instructions)
    m.region(X, bytes(2))
    m.reg('edx', parameters['multiplier'])
    m.reg('ebx', parameters['rounding_addend'])
    m.reg('ecx', parameters['shift'])
    m.reg('eax', 0)
    values = []
    for word in range(-32768, 32768):
        m.word(X, 2, word)
        m.reg('esi', X)
        for pc in [0x8BF407, 0x8BF40F, 0x8BF412, 0x8BF414]:
            m.apply(instructions[pc])
        values.append(signed(m.reg('esi')))
    # Independently exercise the full selector-0 control path/loop/store for a
    # bounded eleven-point provider. The four-instruction slice has the same
    # data dependencies; the other loop instructions load the output base,
    # increment/check the index and store the result.
    samples = [-32768, -32767, -17, -16, -1, 0, 1, 15, 16, 17, 32767]
    full = with_stack(instructions)
    full.region(COEFF, raw, False)
    full.region(X, struct.pack('<11h', *samples), False)
    full.region(Y, b'\xA5' * 44)
    full.reg('eax', COEFF)
    full.reg('edi', len(samples))
    for n, value in enumerate([0xFFFFFFFF, X, Y]):
        full.word(STACK + n * 4, value=value)
    full.run(0x8BF350, 0xFFFFFFFF)
    actual = struct.unpack('<11i', full.bytes(Y, 44))
    assert list(actual) == [values[v + 32768] for v in samples]
    assert full.reg('esp') == STACK + 4
    return values, m.steps + full.steps


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    for p in [args.out, args.report]:
        p.resolve().relative_to((ROOT / 'target').resolve())
        assert not p.exists()
    image = SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    frontier = json.loads((ROOT / 'target/source-ui/font-fixture-frontier-v7.json').read_text())
    cs = Cs(CS_ARCH_X86, CS_MODE_32)
    cs.detail = True
    instructions, proofs = {}, []
    for entry in [0x8BF350, 0x8CC8C0, 0x8D0DB0, 0x8CD700, 0x8CE6C0, 0x8CD020, 0x8BF6E0, 0x8BFE40]:
        body = next(f for f in frontier['functions'] if int(f['entry'], 16) == entry)
        code = b''.join(image.read(int(a, 16), int(b, 16) - int(a, 16)) for a, b in body['ranges'])
        assert hashlib.sha256(code).hexdigest() == body['sha256']
        for a, b in body['ranges']:
            start, end = int(a, 16), int(b, 16)
            rows = list(cs.disasm(image.read(start, end - start), start))
            assert sum(r.size for r in rows) == end - start
            instructions.update({r.address: r for r in rows})
        proofs.append(dict(entry=f'{entry:08X}', ranges=body['ranges'], sha256=body['sha256']))
    # Pin each projected operand, not an algebraic replacement formula.
    for address, expected in {0x8BF407: ('movsx', 'esi, word ptr [esi + eax*2]'),
                              0x8BF40F: ('imul', 'esi, edx'), 0x8BF412: ('add', 'esi, ebx'),
                              0x8BF414: ('sar', 'esi, cl'),
                              0x8CD03D: ('mov', 'word ptr [esi + 8], ax'),
                              0x8CD047: ('mov', 'word ptr [esi + 0xa], ax'),
                              0x8CD051: ('mov', 'word ptr [esi + 0xc], ax'),
                              0x8BF905: ('lea', 'ecx, [ebx + 0x140]'),
                              0x8BF916: ('lea', 'eax, [ebx + 0x154]'),
                              0x8C0596: ('lea', 'edi, [ebx + 4]'),
                              0x8C0599: ('call', '0x8bf350')}.items():
        i = instructions[address]
        assert (i.mnemonic, i.op_str) == expected
    checked_bulk_copy()
    with args.package.open('rb') as archive:
        header = archive.read(96)
        count, length, offset = (struct.unpack_from('<I', header, p)[0] for p in [0x24, 0x2C, 0x40])
        assert header[:4] == b'DBPF'
        archive.seek(offset)
        block = archive.read(length)
        flags, records = native_index(block, count)
        faces = [r for r in records if r[1] == 0x027C5CEF and r[2] == 0x0248E873]
        assert len(faces) == 5 and {f'{r[0]:08X}' for r in faces} == set(FACES)
        totals, face_reports, state = Counter(), [], hashlib.sha256()
        word_tables = {}
        with args.out.open('xb') as output:
            def emit(text):
                data = (text + '\n').encode('ascii')
                output.write(data)
                state.update(data)
                totals['rows'] += 1
            for instance, kind, scope, at, stored, decoded, compression, _, _ in sorted(faces):
                archive.seek(at)
                raw = archive.read(stored)
                assert compression in [0, 0xFFFF]
                raw = raw if compression == 0 else decompress(raw, out_size=decoded, header=True)[0]
                digest = hashlib.sha256(raw).hexdigest()
                assert digest == FACES[f'{instance:08X}'] and len(raw) == decoded
                font = TTFont(io.BytesIO(raw))
                assert 'vmtx' not in font
                key = f'{kind:08X}:{scope:08X}:{instance:08X}'
                params, packed, steps = coefficients(instructions, 15, font['head'].unitsPerEm)
                totals['instruction_steps'] += steps
                vadvance, top, steps = vertical(instructions, font['hhea'])
                totals['instruction_steps'] += steps
                selector = params['selector']
                if selector == 0:
                    cache = tuple(params.values())
                    if cache not in word_tables:
                        table, steps = source_word_table(instructions, params, packed)
                        word_tables[cache] = table
                        totals['instruction_steps'] += steps
                    table = word_tables[cache]
                    emit(f'{key}\tparams\t15\t{font["head"].unitsPerEm}\t0\t{params["multiplier"]}\t{params["shift"]}\t{params["denominator"]}\t{params["rounding_addend"]}\t{params["fixed_multiplier"]}')
                else:
                    assert selector in (1, 2)
                    emit(f'{key}\tparams\t15\t{font["head"].unitsPerEm}\tunsupported\t{selector}')
                counts = Counter()
                for gid, name in enumerate(font.getGlyphOrder()):
                    g = font['glyf'][name]
                    counts['glyphs'] += 1
                    if g.numberOfContours <= 0:
                        excluded = 'composite' if g.numberOfContours < 0 else 'empty'
                        counts['excluded_' + excluded] += 1
                        emit(f'{key}\tglyph\t{gid}\texcluded:{excluded}')
                        continue
                    advance, bearing = font['hmtx'].metrics[name]
                    phantoms, steps = phantom(instructions, g, advance, bearing, vadvance, top)
                    totals['instruction_steps'] += steps
                    encoded_phantoms = ';'.join(f'{x},{y}' for x, y in phantoms)
                    counts['simple_phantoms'] += 1
                    if selector != 0:
                        encoded_scale = f'unsupported:{selector}'
                        counts['excluded_scale_selector_' + str(selector)] += 1
                    else:
                        points = list(g.coordinates) + phantoms
                        encoded_scale = ';'.join(f'{table[x + 32768]},{table[y + 32768]}' for x, y in points)
                        counts.update(scaled_simple_glyphs=1, scaled_real_points=len(g.coordinates), scaled_phantom_points=4)
                    emit(f'{key}\tglyph\t{gid}\tsimple\t{vadvance},{top}\t{encoded_phantoms}\t{encoded_scale}')
                totals.update(counts)
                face_reports.append(dict(key=key, source_sha256=digest, ppem=15, units_per_em=font['head'].unitsPerEm,
                                         native_parameters=params, vertical_advance=vadvance, top_bearing=top,
                                         counts=dict(counts)))
    assert totals['glyphs'] == 51680 and totals['simple_phantoms'] == 37691
    assert totals['scaled_simple_glyphs'] == 37254 and totals['scaled_real_points'] == 2858480
    assert totals['scaled_phantom_points'] == 149016 and totals['excluded_scale_selector_1'] == 437
    result = dict(schema='source-pinned-simple-font-scaling-instruction-oracle-v1',
                  source_sha256=SOURCE_SHA256, source_proofs=proofs, source_index_flags=f'{flags:08X}',
                  oracle_tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  unexported_port_sha256=hashlib.sha256((ROOT / 'crates/spore-core/src/spui_font_scaling.rs').read_bytes()).hexdigest(),
                  rust_example_sha256=hashlib.sha256((ROOT / 'crates/spore-bevy/examples/spui_font_scaling_verify.rs').read_bytes()).hexdigest(),
                  source_index_sha256=hashlib.sha256(block).hexdigest(), faces=face_reports, totals=dict(totals),
                  native_code_executed=False, coefficient_and_phantom_source_instructions_interpreted=True,
                  signed_word_domain_per_scale=65536, unique_selector0_word_tables=len(word_tables),
                  full_selector0_call_sample_points=11, canonical_bytes=args.out.stat().st_size,
                  canonical_sha256=state.hexdigest(),
                  scope='Original positive-contour ordinary packets only. All five faces supply phantom inputs; '
                  'only three 2048-UPE faces use admitted selector0 scaling at15ppem. Two1000-UPE faces '
                  'are explicitly unsupported for scaling. Composite and empty/zero-contour paths are excluded. '
                  'Source integer instruction model with strict model memory and closed direct-call paths, '
                  'not native machine-code execution. No phantom flags, hint execution or raster comparison.')
    args.report.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
