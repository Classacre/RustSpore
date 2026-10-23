"""Check bounded body-pose math against source-pinned scalar-SSE instructions.

The original program is not launched. Three straight-line instruction regions
are decoded locally, interpreted with integer binary32 round-to-nearest/even,
and supplied only synthetic explicit poses. No hierarchy or channels are built.
Proprietary bytes/disassembly and generated fixtures remain in ignored target/.
"""
from __future__ import annotations

import argparse
import hashlib
import itertools
import json
from pathlib import Path
import struct
import sys

from cell_schema_probe import SourceImage, SOURCE_SHA256


SIGN = 0x80000000
ONE = 0x3F800000
NO_PALETTE = 0xFFFFFFFF


def bits(value: float) -> int:
    return struct.unpack('<I', struct.pack('<f', value))[0]


def number(value: int) -> tuple[int, int]:
    exponent = (value >> 23) & 255
    if exponent == 255:
        raise ValueError('oracle accepts finite operands only')
    mantissa = value & 0x7FFFFF
    if exponent:
        mantissa |= 0x800000
    return mantissa, exponent - 150 if exponent else -149


def rounded_quotient(numerator: int, denominator: int, shift: int) -> int:
    if shift >= 0:
        numerator <<= shift
    else:
        denominator <<= -shift
    quotient, remainder = divmod(numerator, denominator)
    twice = remainder * 2
    return quotient + (twice > denominator or (twice == denominator and quotient & 1))


def rounded(numerator: int, denominator: int, power: int, sign: int) -> int:
    """Round an exact nonnegative integer ratio times 2**power to binary32."""
    if numerator == 0:
        return sign
    exponent = numerator.bit_length() - denominator.bit_length()
    if exponent >= 0:
        exponent -= numerator < denominator << exponent
    else:
        exponent -= numerator << -exponent < denominator
    exponent += power
    if exponent < -126:
        mantissa = rounded_quotient(numerator, denominator, power + 149)
        return sign | mantissa
    mantissa = rounded_quotient(numerator, denominator, power - exponent + 23)
    if mantissa == 0x1000000:
        mantissa >>= 1
        exponent += 1
    if exponent > 127:
        return sign | 0x7F800000
    return sign | ((exponent + 127) << 23) | (mantissa & 0x7FFFFF)


def arithmetic(operation: str, left: int, right: int) -> int:
    if operation == 'subss':
        right ^= SIGN
        operation = 'addss'
    lm, le = number(left)
    rm, re = number(right)
    if operation == 'addss':
        power = min(le, re)
        exact = ((-lm if left & SIGN else lm) << (le - power))
        exact += ((-rm if right & SIGN else rm) << (re - power))
        sign = SIGN if exact < 0 or (exact == 0 and left & right & SIGN) else 0
        return rounded(abs(exact), 1, power, sign)
    sign = (left ^ right) & SIGN
    if operation == 'mulss':
        return rounded(lm * rm, 1, le + re, sign)
    if operation == 'divss':
        if not rm:
            raise ValueError('zero divisor')
        return rounded(lm, rm, le - re, sign)
    raise ValueError(operation)


class Oracle:
    """Only the straight-line scalar operations in the three measured regions."""

    def __init__(self, image: SourceImage):
        from capstone import Cs, CS_ARCH_X86, CS_MODE_32
        from capstone.x86 import X86_OP_REG, X86_OP_MEM, X86_OP_IMM
        decoder = Cs(CS_ARCH_X86, CS_MODE_32)
        decoder.detail = True
        self.image = image
        self.regions = {}
        self.evidence = []
        for name, start, end in (
            ('rest', 0x9C33D6, 0x9C36D8),
            ('basis', 0x9A4580, 0x9A46B1),
            ('product', 0x9C7AD0, 0x9C7DF8),
        ):
            raw = image.read(start, end - start)
            instructions = list(decoder.disasm(raw, start))
            if sum(i.size for i in instructions) != len(raw):
                raise ValueError('incomplete instruction decoding')
            compiled = []
            for instruction in instructions:
                if instruction.mnemonic not in ('mov', 'movss', 'movaps', 'xorps',
                                                 'addss', 'subss', 'mulss', 'add', 'sub'):
                    raise ValueError(f'unsupported instruction {instruction.address:08X}')
                operands = []
                for operand in instruction.operands:
                    if operand.type == X86_OP_REG:
                        operands.append(('reg', instruction.reg_name(operand.reg)))
                    elif operand.type == X86_OP_IMM:
                        operands.append(('imm', operand.imm))
                    elif operand.type == X86_OP_MEM:
                        operands.append(('mem', instruction.reg_name(operand.mem.base),
                                         instruction.reg_name(operand.mem.index),
                                         operand.mem.scale, operand.mem.disp))
                    else:
                        raise ValueError('unsupported operand')
                compiled.append((instruction.mnemonic, operands))
            self.regions[name] = compiled
            self.evidence.append({'name': name, 'start': f'{start:08X}', 'end_exclusive': f'{end:08X}',
                                  'instructions': len(compiled),
                                  'source_bytes_sha256': hashlib.sha256(raw).hexdigest()})

    def execute(self, name: str, registers: dict[str, int], memory: dict[int, int]):
        def address(operand):
            _, base, index, scale, displacement = operand
            return registers.get(base, 0) + registers.get(index, 0) * scale + displacement

        def read(operand):
            if operand[0] == 'imm':
                return operand[1]
            if operand[0] == 'reg':
                return registers[operand[1]]
            at = address(operand)
            if at not in memory:
                memory[at] = struct.unpack('<I', self.image.read(at, 4))[0]
            return memory[at]

        def write(operand, value):
            if operand[0] == 'reg':
                registers[operand[1]] = value
            else:
                memory[address(operand)] = value

        for operation, (left, right) in self.regions[name]:
            value = read(right)
            if operation in ('addss', 'subss', 'mulss'):
                value = arithmetic(operation, read(left), value)
            elif operation == 'xorps':
                value ^= read(left)
            elif operation == 'add':
                value += read(left)
            elif operation == 'sub':
                value = read(left) - value
            write(left, value)

    def rest(self, values: list[int]) -> list[int]:
        # Explicit rest center at block+10C, quaternion at +118.
        block, stack = 0x10000, 0x20000
        memory = {block + 0x10C + i * 4: value for i, value in enumerate(values)}
        self.execute('rest', {'edx': block, 'esp': stack,
                              'xmm0': SIGN, 'xmm3': bits(2.)}, memory)
        return [memory[stack + 0x84 + i * 4] for i in range(9)] + [
            memory[stack + 0x34 + i * 4] for i in range(3)]

    def basis(self, quaternion: list[int]) -> list[int]:
        source, destination, stack = 0x10000, 0x20000, 0x30000
        memory = {source + i * 4: value for i, value in enumerate(quaternion)}
        memory[stack + 4], memory[stack + 8] = destination, source
        self.execute('basis', {'esp': stack}, memory)
        return [memory[destination + i * 4] for i in range(9)]

    def product(self, rest: list[int], live: list[int]) -> list[int]:
        a, p, output, stack = 0x10000, 0x20000, 0x30000, 0x40000
        memory = {a + i * 4: value for i, value in enumerate(rest)}
        memory.update({p + i * 4: value for i, value in enumerate(live)})
        memory[stack + 0x3C] = output - a
        self.execute('product', {'eax': a + 0x28, 'ecx': p + 8,
                                 'edx': output + 0xC, 'esp': stack,
                                 'esi': p - a, 'edi': output - p}, memory)
        return [memory[output + i * 4] for i in range(12)]

    def evaluate(self, count: int, rest_rows, runtime_rows, scale: int):
        rest = [None] * count
        output = [None] * count
        for palette_id, values in rest_rows:
            if palette_id != NO_PALETTE:
                rest[palette_id] = self.rest(values)
        reciprocal = arithmetic('divss', ONE, scale)
        for palette_id, values in runtime_rows:
            if palette_id != NO_PALETTE:
                live = self.basis(values[3:]) + [arithmetic('mulss', v, reciprocal) for v in values[:3]]
                output[palette_id] = self.product(rest[palette_id], live)
        if any(row is None for row in rest + output):
            raise ValueError('incomplete synthetic coverage')
        return rest, output


def self_check():
    # Exact halfway ties, signed zeros, gradual underflow, carry and division.
    assert arithmetic('addss', ONE, bits(2. ** -24)) == ONE
    assert arithmetic('addss', ONE + 1, bits(2. ** -24)) == ONE + 2
    assert arithmetic('addss', SIGN, SIGN) == SIGN
    assert arithmetic('subss', SIGN, 0) == SIGN
    assert arithmetic('addss', ONE, ONE ^ SIGN) == 0
    assert arithmetic('mulss', bits(2. ** -126), bits(.5)) == 0x00400000
    assert arithmetic('mulss', 1, bits(.5)) == 0
    assert arithmetic('mulss', 3, bits(.5)) == 2
    assert arithmetic('divss', ONE, bits(3.)) == 0x3EAAAAAB
    # Compare ordinary binary32 addition/multiplication against independently
    # rounded binary64 arithmetic where exact binary32 operands fit its width.
    candidates = [bits(v) for v in (0., -0., 1., -1., 2. ** -126, 2. ** -20,
                                   .1, .5, 1.5, 2. ** 20)] + [1, 2, 3, ONE + 1]
    for operation, left, right in itertools.product(('addss', 'subss', 'mulss'), candidates, candidates):
        lv = struct.unpack('<f', struct.pack('<I', left))[0]
        rv = struct.unpack('<f', struct.pack('<I', right))[0]
        expected = {'addss': lambda: lv + rv, 'subss': lambda: lv - rv,
                    'mulss': lambda: lv * rv}[operation]()
        assert arithmetic(operation, left, right) == bits(expected)


def fixtures():
    scales = [bits(v) for v in (1., 2., .5, 3., -2., 10., .1, 2. ** -8, 2. ** 8)] + [ONE - 1, ONE + 1]
    quaternions = [[bits(v) for v in values] for values in (
        (0., 0., 0., 1.), (-0., 0., -0., -1.),
        (.7071067811865476, 0., 0., .7071067811865476),
        (0., .7071067811865476, 0., .7071067811865476),
        (0., 0., .7071067811865476, .7071067811865476),
        (.25, -.5, .75, 1.25), (2. ** -20, -2. ** -16, 2. ** -18, 1.),
        (0., 0., 1., 1.), (.4999999701976776, -.5000000596046448, .5, -.5),
    )]
    state = 0x6F173AF2

    def random_word():
        nonlocal state
        state = (state * 1664525 + 1013904223) & 0xFFFFFFFF
        return state

    def scalar():
        word = random_word()
        return (word & 0x807FFFFF) | ((117 + (word >> 24) % 21) << 23)

    for case in range(297):
        count = (0, 1, 2, 3, 7, 19, 100, 255, 11)[case % 9]
        scale = scales[case % len(scales)]
        rest_rows, runtime_rows = [], []
        for index in range(count):
            center = [scalar() for _ in range(3)]
            position = [scalar() for _ in range(3)]
            if case % 11 == 0:
                center = [0, SIGN, bits(1. / 3.)]
                position = [bits(1.), bits(-2.), bits(1. / 3.)]
            rest_q = quaternions[(case + index) % len(quaternions)]
            live_q = quaternions[(case * 3 + index * 5) % len(quaternions)]
            if case % 5 == 0:
                rest_q = [scalar() for _ in range(4)]
                live_q = [scalar() for _ in range(4)]
            rest_rows.append((index, center + rest_q))
            runtime_rows.append((index, position + live_q))
        # Independent rest/runtime permutations force ID-based association.
        if count:
            rest_rows = rest_rows[case % count:] + rest_rows[:case % count]
            if case & 1:
                rest_rows.reverse()
            runtime_rows.reverse()
            if case % 3 == 0:
                # Sentinel fields are intentionally nonfinite: native skips
                # their matrix production before reading these fields.
                sentinel = (NO_PALETTE, [0x7FC12345] * 7)
                rest_rows.insert(0, sentinel)
                runtime_rows.append(sentinel)
        yield scale, count, rest_rows, runtime_rows


def digest(words, rest, output):
    feed = bytearray(b'native-body-pose-v1\0')
    for value in words + list(itertools.chain.from_iterable(rest + output)):
        feed.extend(struct.pack('<I', value))
    value = 0xCBF29CE484222325
    for byte in feed:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f'{value:016X}'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--capstone-runtime', type=Path, default=Path('target/rea/capstone-runtime'))
    parser.add_argument('--fixtures', type=Path, required=True)
    parser.add_argument('--digest-out', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--rust-digest', type=Path)
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    for path in (args.fixtures, args.digest_out, args.out):
        path.resolve().relative_to(target)
        path.parent.mkdir(parents=True, exist_ok=True)
    sys.path.insert(0, str(args.capstone_runtime.resolve()))
    self_check()
    oracle = Oracle(SourceImage(args.exe))
    lines = []
    matrices = sentinels = 0
    with args.fixtures.open('w', encoding='utf-8', newline='\n') as stream:
        stream.write('# id; then hex u32 scale, palette_count, rest_count, live_count; then id + seven float bits per row\n')
        for case, (scale, count, rest_rows, runtime_rows) in enumerate(fixtures()):
            words = [scale, count, len(rest_rows), len(runtime_rows)]
            words += [value for palette_id, values in rest_rows + runtime_rows for value in (palette_id, *values)]
            case_id = f'{case:06d}'
            stream.write(case_id + '\t' + '\t'.join(f'{value:08X}' for value in words) + '\n')
            rest, output = oracle.evaluate(count, rest_rows, runtime_rows, scale)
            lines.append(case_id + '\t' + digest(words, rest, output))
            matrices += count
            sentinels += sum(index == NO_PALETTE for index, _ in rest_rows + runtime_rows)
    normalized = '\n'.join(lines) + '\n'
    args.digest_out.write_text(normalized, encoding='utf-8', newline='\n')
    rust = args.rust_digest.read_text(encoding='utf-8').splitlines() if args.rust_digest else None
    problems = [] if rust is None else [
        {'line': i + 1, 'expected': a, 'rust': b}
        for i, (a, b) in enumerate(itertools.zip_longest(lines, rust)) if a != b]
    report = {
        'scope': 'explicit synthetic rest/live poses, retained IDs and scale; no hierarchy or channel inference',
        'oracle': 'source-pinned Capstone instructions; exact integer binary32 RNE; gradual underflow; no MXCSR provenance claim',
        'source_executable_sha256': SOURCE_SHA256,
        'instruction_regions': oracle.evidence,
        'cases': len(lines), 'rest_matrices': matrices, 'output_matrices': matrices,
        'skipped_sentinel_rows': sentinels,
        'fixture_sha256': hashlib.sha256(args.fixtures.read_bytes()).hexdigest(),
        'normalized_digest_sha256': hashlib.sha256(normalized.encode()).hexdigest(),
        'rust_exact_match': None if rust is None else not problems,
        'rust_records': None if rust is None else len(rust),
        'problem_count': len(problems), 'problems': problems[:20],
    }
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({key: report[key] for key in ('cases', 'rest_matrices', 'skipped_sentinel_rows',
                                                'normalized_digest_sha256', 'rust_exact_match', 'problem_count')}))
    if problems:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
