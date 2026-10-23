"""Fresh source CreatureData rest-input census with an original-instruction oracle.

This does not launch the game. A bounded interpreter decodes the SHA-pinned
source loader geometry and its actual matrix-to-quaternion callee, retaining
x87 values as exact rational numbers rounded after each operation. Inputs are
read afresh through an independent DBPF/header reader. Source bytes stay local.
"""
from __future__ import annotations

import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import itertools
import json
from pathlib import Path
import struct
import sys

from anim_body_pose_probe import arithmetic, SIGN
from cell_schema_probe import SourceImage, SOURCE_SHA256
from creature_header_probe import header, inspect
from dbpf_extract import payload
from prop_numeric_probe import index as package_index
from native_x87_probe import operand, quantize, sqrt_quantize, commit


CONTROLS = ((0x007F, 24, 0), (0x027F, 53, 0), (0x037F, 64, 0))


class SourceOracle:
    def __init__(self, image):
        from capstone import Cs, CS_ARCH_X86, CS_MODE_32
        from capstone.x86 import X86_OP_REG, X86_OP_MEM, X86_OP_IMM
        decoder = Cs(CS_ARCH_X86, CS_MODE_32)
        decoder.detail = True
        self.image = image
        self.program = {}
        self.evidence = []
        self.constants = {}
        for name, start, end in (
            ('canonical source geometry', 0x9CC249, 0x9CC5A8),
            ('matrix to quaternion', 0x9A4DF0, 0x9A4F6C),
        ):
            raw = image.read(start, end - start)
            decoded = list(decoder.disasm(raw, start))
            if sum(i.size for i in decoded) != len(raw):
                raise ValueError('incomplete bounded decoding')
            for instruction in decoded:
                operands = []
                for value in instruction.operands:
                    if value.type == X86_OP_REG:
                        operands.append(('reg', instruction.reg_name(value.reg)))
                    elif value.type == X86_OP_IMM:
                        operands.append(('imm', value.imm))
                    elif value.type == X86_OP_MEM:
                        operands.append(('mem', instruction.reg_name(value.mem.base),
                                         instruction.reg_name(value.mem.index),
                                         value.mem.scale, value.mem.disp))
                    else:
                        raise ValueError('unsupported operand')
                self.program[instruction.address] = (instruction.mnemonic, operands,
                                                     instruction.address + instruction.size)
            self.evidence.append({'name': name, 'start': f'{start:08X}',
                                  'end_exclusive': f'{end:08X}', 'instructions': len(decoded),
                                  'source_bytes_sha256': hashlib.sha256(raw).hexdigest()})

    def evaluate(self, raw_row: bytes, precision: int, mode: int):
        row, stack, block = 0x10000, 0x20000, 0x30000
        memory = {row + i: struct.unpack_from('<I', raw_row, i)[0]
                  for i in range(0, 140, 4)}
        registers = {'edi': row, 'esi': block, 'esp': stack,
                     'eax': 0, 'ebx': 0, 'ecx': 0, 'edx': 0, 'ebp': 0}
        floating = []
        flags = {'zero': False, 'below': False, 'less': False}
        branch = None

        def address(value):
            _, base, index, scale, offset = value
            return (registers.get(base, 0) + registers.get(index, 0) * scale + offset) & 0xFFFFFFFF

        def read(value):
            if value[0] == 'reg':
                return registers[value[1]]
            if value[0] == 'imm':
                return value[1]
            at = address(value)
            if at in memory:
                return memory[at]
            if at not in self.constants:
                self.constants[at] = struct.unpack('<I', self.image.read(at, 4))[0]
            return self.constants[at]

        def write(value, word):
            if value[0] == 'reg':
                registers[value[1]] = word & 0xFFFFFFFF
            else:
                memory[address(value)] = word & 0xFFFFFFFF

        def push(word):
            registers['esp'] -= 4
            memory[registers['esp']] = word & 0xFFFFFFFF

        def pop():
            word = memory[registers['esp']]
            registers['esp'] += 4
            return word

        def xvalue(word):
            return operand(word), bool(word & SIGN)

        def rounded(value, negative_zero=False):
            exact, raw = quantize(value, precision, mode, negative_zero)
            return exact, bool(raw[-1] & 0x80)

        pc = 0x9CC249
        steps = 0
        while pc != 0x9CC5A8:
            steps += 1
            if steps > 1000:
                raise ValueError('bounded interpreter step limit')
            if pc == 0x9A4E32:
                branch = 'positive_trace'
            elif pc == 0x9A4EB2:
                branch = f'dominant_{registers["esi"]}'
            operation, values, next_pc = self.program[pc]
            if operation in ('mov', 'movss', 'movaps'):
                write(values[0], read(values[1]))
            elif operation in ('xor', 'xorps'):
                word = 0 if values[0] == values[1] else read(values[0]) ^ read(values[1])
                write(values[0], word)
            elif operation in ('addss', 'subss', 'mulss', 'divss'):
                write(values[0], arithmetic(operation, read(values[0]), read(values[1])))
            elif operation == 'comiss':
                left, right = map(lambda v: operand(read(v)), values)
                flags = {'zero': left == right, 'below': left < right, 'less': left < right}
            elif operation == 'cmp':
                left, right = read(values[0]) & 0xFFFFFFFF, read(values[1]) & 0xFFFFFFFF
                signed = lambda v: v - 0x100000000 if v & SIGN else v
                flags = {'zero': left == right, 'below': left < right,
                         'less': signed(left) < signed(right)}
            elif operation in ('add', 'sub', 'shl'):
                left, right = read(values[0]), read(values[1])
                word = {'add': lambda: left + right, 'sub': lambda: left - right,
                        'shl': lambda: left << right}[operation]()
                write(values[0], word)
            elif operation == 'inc':
                write(values[0], read(values[0]) + 1)
            elif operation == 'lea':
                write(values[0], address(values[1]))
            elif operation == 'cdq':
                registers['edx'] = 0xFFFFFFFF if registers['eax'] & SIGN else 0
            elif operation == 'idiv':
                numerator = (registers['edx'] << 32) | registers['eax']
                if numerator & (1 << 63):
                    numerator -= 1 << 64
                denominator = read(values[0])
                if denominator & SIGN:
                    denominator -= 1 << 32
                quotient = abs(numerator) // abs(denominator)
                if (numerator < 0) != (denominator < 0):
                    quotient = -quotient
                registers['eax'] = quotient & 0xFFFFFFFF
                registers['edx'] = (numerator - quotient * denominator) & 0xFFFFFFFF
            elif operation == 'push':
                push(read(values[0]))
            elif operation == 'pop':
                write(values[0], pop())
            elif operation == 'call':
                if read(values[0]) != 0x9A4DF0:
                    raise ValueError('unexpected callee')
                push(next_pc)
                next_pc = read(values[0])
            elif operation == 'ret':
                next_pc = pop()
            elif operation == 'jbe':
                if flags['below'] or flags['zero']:
                    next_pc = read(values[0])
            elif operation == 'jl':
                if flags['less']:
                    next_pc = read(values[0])
            elif operation == 'fld':
                floating.append(xvalue(read(values[0])))
            elif operation in ('fst', 'fstp'):
                exact, negative = floating[-1]
                word = commit(exact, mode, not exact and negative)
                if word is None:
                    raise ValueError('nonfinite x87 store')
                write(values[0], word)
                if operation == 'fstp':
                    floating.pop()
            elif operation in ('fadd', 'fmul', 'fsubr'):
                left, ln = floating.pop()
                right, rn = xvalue(read(values[0]))
                exact = {'fadd': lambda: left + right, 'fmul': lambda: left * right,
                         'fsubr': lambda: right - left}[operation]()
                negative = ((ln != rn) if operation == 'fmul' else
                            (mode == 1 or (ln and rn)) if operation == 'fadd' else
                            (mode == 1 or (rn and not ln)))
                floating.append(rounded(exact, not exact and negative))
            elif operation == 'fsqrt':
                exact, negative = floating.pop()
                if exact < 0:
                    raise ValueError('negative native square-root input')
                if not exact:
                    floating.append((exact, negative))
                else:
                    value, raw = sqrt_quantize(exact, precision, mode)
                    floating.append((value, bool(raw[-1] & 0x80)))
            else:
                raise ValueError(f'unsupported native operation {pc:08X}: {operation}')
            pc = next_pc
        if floating:
            raise ValueError('x87 stack not balanced')
        center = [memory[block + 0x10C + i * 4] for i in range(3)]
        quaternion = [memory[block + 0x118 + i * 4] for i in range(4)]
        if any((v >> 23) & 255 == 255 for v in center + quaternion):
            raise ValueError('nonfinite native rest input')
        return center + quaternion, branch


def fnv(data: bytes, value=0xCBF29CE484222325):
    for byte in data:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', nargs='?', type=Path, default=Path('D:/Games/SPORE Collection'))
    parser.add_argument('--exe', type=Path, default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    parser.add_argument('--capstone-runtime', type=Path, default=Path('target/rea/capstone-runtime'))
    parser.add_argument('--digest-out', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--rust-digest', type=Path)
    parser.add_argument('--reference', type=Path,
                        help='compare retained oracle digests without repeating the instruction census')
    args = parser.parse_args()
    target = (Path(__file__).resolve().parent.parent / 'target').resolve()
    for path in (args.digest_out, args.out):
        path.resolve().relative_to(target)
        path.parent.mkdir(parents=True, exist_ok=True)
    if args.reference:
        if not args.rust_digest:
            parser.error('--reference requires --rust-digest')
        reference_bytes = args.reference.read_bytes()
        reference = json.loads(reference_bytes)
        expected_bytes = args.digest_out.read_bytes()
        if hashlib.sha256(expected_bytes).hexdigest() != reference['normalized_digest_sha256']:
            raise ValueError('retained oracle digest file hash differs from its report')
        # Recheck the executable pin. The fresh Rust corpus digest includes
        # every complete source row, so changed source inputs cannot pass by
        # silently reusing an expected pose for different source bytes.
        SourceImage(args.exe)
        expected = expected_bytes.decode().splitlines()
        rust = sorted(args.rust_digest.read_text().splitlines())
        problems = [{'line': i + 1, 'expected': a, 'rust': b}
                    for i, (a, b) in enumerate(itertools.zip_longest(expected, rust)) if a != b]
        report = dict(reference)
        report.update(reference_report=str(args.reference),
                      reference_report_sha256=hashlib.sha256(reference_bytes).hexdigest(),
                      comparison_scope='retained source-instruction oracle versus fresh Rust package reads',
                      rust_exact_match=not problems, rust_records=len(rust),
                      rust_digest_sha256=hashlib.sha256(args.rust_digest.read_bytes()).hexdigest(),
                      problem_count=len(problems), problems=problems[:20])
        args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        print(json.dumps({key: report[key] for key in ('statistics', 'normalized_digest_sha256',
                                                    'rust_exact_match', 'problem_count')}))
        if problems:
            raise SystemExit(1)
        return
    sys.path.insert(0, str(args.capstone_runtime.resolve()))
    oracle = SourceOracle(SourceImage(args.exe))
    stats, branches = Counter(), Counter()
    lines, records, failures, excluded = [], [], [], []
    for package in sorted(args.root.rglob('*.package')):
        try:
            entries = package_index(package)
        except Exception as error:
            excluded.append({'package': str(package), 'error': repr(error)})
            continue
        stats['indexes_opened'] += 1
        for entry in entries:
            if entry['type'] != 0x0F43029A:
                continue
            key = f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            stats['source_resources'] += 1
            try:
                raw = payload(package, entry)
                inspect(raw, Counter())
                version, count = struct.unpack_from('<II', raw, 4)
                start, _, _, _ = header(raw, 20 if version == 9 else 24)
                rows = [raw[start + i * 140:start + (i + 1) * 140] for i in range(count)]
                stats['source_rows'] += count
                for control, precision, mode in CONTROLS:
                    digest = fnv(b'native-source-rest-v1\0' + struct.pack('<2I', control, count))
                    for index, row in enumerate(rows):
                        values, branch = oracle.evaluate(row, precision, mode)
                        branches[f'{control:04X}:{branch}'] += 1
                        digest = fnv(struct.pack('<I', index) + row + struct.pack('<7I', *values), digest)
                        stats['evaluated_rows'] += 1
                    lines.append(f'{package.name} {key} {control:04X} {digest:016X}')
                    stats['prepared_resource_contexts'] += 1
                records.append({'package': package.name, 'key': key, 'source_rows': count,
                                'source_sha256': hashlib.sha256(raw).hexdigest()})
            except Exception as error:
                failures.append({'package': package.name, 'key': key, 'error': repr(error)})
            if stats['source_resources'] % 128 == 0:
                print(f"processed_resources={stats['source_resources']} evaluated_rows={stats['evaluated_rows']}", flush=True)
    lines.sort()
    normalized = '\n'.join(lines) + '\n'
    args.digest_out.write_text(normalized, encoding='utf-8', newline='\n')
    rust = args.rust_digest.read_text().splitlines() if args.rust_digest else None
    problems = [] if rust is None else [
        {'line': i + 1, 'expected': a, 'rust': b}
        for i, (a, b) in enumerate(itertools.zip_longest(lines, sorted(rust))) if a != b]
    report = {
        'scope': 'canonical source-row rest center/orientation; no finalized hierarchy or runtime channels',
        'oracle': 'original decoded instructions; exact integer binary32 SSE RNE; rational/integer x87 rounding and sqrt',
        'source_executable_sha256': SOURCE_SHA256, 'instruction_regions': oracle.evidence,
        'explicit_contexts': [f'{v[0]:04X}' for v in CONTROLS],
        'retail_control_word_known': False, 'retail_mxcsr_known': False,
        'statistics': dict(stats), 'quaternion_branch_counts': dict(branches),
        'normalized_digest_sha256': hashlib.sha256(normalized.encode()).hexdigest(),
        'source_records': records, 'failures': failures, 'excluded_packages': excluded,
        'rust_exact_match': None if rust is None else not problems,
        'rust_records': None if rust is None else len(rust),
        'problem_count': len(problems), 'problems': problems[:20],
    }
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({key: report[key] for key in ('statistics', 'normalized_digest_sha256',
                                                'rust_exact_match', 'problem_count')}))
    if failures or excluded or not lines or problems:
        print(json.dumps({'failures': failures[:10], 'excluded': excluded[:10]}))
        raise SystemExit(1)


if __name__ == '__main__':
    main()
