"""Interpret pinned selector-1 integer instructions; never execute native code.

The previous selector-0 oracle stays unchanged. This separate gate extends its
closed integer interpreter with signed CDQ/IDIV, exhausts five signed-word
domains and prepares all simple glyphs from the five canonical installed faces.
Selector 2, component assembly, glyph hints and rasterization remain excluded.
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
from cell_schema_probe import SourceImage, SOURCE_SHA256
from fontTools.ttLib import TTFont
from native_archive_index_probe import decode as native_index
from refpack import decompress
from refpack_bulk import install as checked_bulk_copy
from spui_font_outline_probe import FACES
from spui_font_scaling_probe import (
    Machine, signed, coefficients, vertical, phantom, source_word_table,
    STACK, X, Y, COEFF,
)

PPEMS = [1, 2, 3, 15, 16, 31, 511, 512, 1023, 32767, 32768, 32769, 65534, 65535]
UNITS = [1, 2, 3, 16, 31, 32, 63, 64, 125, 192, 384, 1000, 2048, 32767, 32768, 65535]
WORD_DOMAINS = [(15, 2048), (15, 1000), (1, 384), (32767, 192), (1, 65535)]
PARAMETER_FIELDS = ['selector', 'multiplier', 'shift', 'denominator', 'rounding_addend', 'fixed_multiplier']
BASE_ORACLE_SHA256 = 'f40f532a3148b7b87cc392ec108d44a6399addec747860dbb901fd348260454c'


class QuotientMachine(Machine):
    """Checked IA-32 CDQ/IDIV semantics over bounded model memory only."""
    def __init__(self, instructions):
        super().__init__(instructions)
        self.visits = Counter()

    def apply(self, ins):
        self.visits[ins.address] += 1
        if ins.mnemonic == 'cdq':
            assert not ins.operands
            self.steps += 1
            self.reg('edx', -1 if signed(self.reg('eax')) < 0 else 0)
            return ins.address + ins.size  # CDQ preserves flags.
        if ins.mnemonic == 'idiv':
            assert len(ins.operands) == 1 and ins.operands[0].size == 4
            self.steps += 1
            dividend = signed((self.reg('edx') << 32) | self.reg('eax'), 64)
            divisor = signed(self.operand(ins, ins.operands[0]))
            assert divisor, 'modeled IDIV divide by zero'
            quotient = abs(dividend) // abs(divisor)
            if (dividend < 0) != (divisor < 0):
                quotient = -quotient
            assert -(1 << 31) <= quotient < (1 << 31), 'modeled IDIV quotient overflow'
            remainder = dividend - quotient * divisor
            assert abs(remainder) < abs(divisor)
            assert remainder == 0 or (remainder < 0) == (dividend < 0)
            self.reg('eax', quotient)
            self.reg('edx', remainder)
            self.flags = dict(z=None, s=None, c=None, o=None)
            return ins.address + ins.size
        return super().apply(ins)


def stack_machine(instructions):
    machine = QuotientMachine(instructions)
    machine.region(STACK - 256, bytes(512))
    machine.reg('esp', STACK)
    return machine


def signed_division_semantics(instructions):
    """Explicit signed quotient/remainder facts, including CPU trap domains."""
    m = QuotientMachine(instructions)
    fixtures = [(7, 3, 2, 1), (-7, 3, -2, -1), (7, -3, -2, 1),
                (-7, -3, 2, -1), (-1, 3, 0, -1), (1, -3, 0, 1),
                (-2147483648, 1, -2147483648, 0),
                (2147483647, -1, -2147483647, 0), (0, -3, 0, 0)]
    for dividend, divisor, quotient, remainder in fixtures:
        flags = dict(z=True, s=False, c=True, o=False)
        m.flags = flags.copy()
        m.reg('eax', dividend)
        m.reg('ebp', divisor)
        m.apply(instructions[0x8BF3D4])
        assert m.flags == flags and signed(m.reg('edx')) == (-1 if dividend < 0 else 0)
        m.apply(instructions[0x8BF3D5])
        assert (signed(m.reg('eax')), signed(m.reg('edx'))) == (quotient, remainder)
        assert all(value is None for value in m.flags.values())
    for dividend, divisor in [(1, 0), (-2147483648, -1)]:
        m.reg('eax', dividend)
        m.reg('ebp', divisor)
        m.apply(instructions[0x8BF3D4])
        try:
            m.apply(instructions[0x8BF3D5])
        except AssertionError as error:
            assert str(error).startswith('modeled IDIV ')
        else:
            raise AssertionError('the modeled CPU trap was not rejected')
    return dict(signed_quotient_remainder_fixtures=len(fixtures), rejected_divide_traps=2,
                cdq_preserves_all_modeled_flags=True, idiv_flags_are_undefined=True)


def quotient_word_table(instructions, parameters, packed):
    assert parameters['selector'] == 1
    m = stack_machine(instructions)
    m.region(X, bytes(2))
    m.reg('ebx', parameters['rounding_addend'])
    m.reg('ebp', parameters['denominator'])
    m.reg('esi', 0)
    m.word(STACK + 0x14, value=X)
    m.word(STACK + 0xC, value=parameters['multiplier'])
    values = []
    for word in range(-32768, 32768):
        m.word(X, 2, word)
        m.run(0x8BF3C0, 0x8BF3E2, 16)
        values.append(signed(m.reg('eax')))
        assert m.reg('esp') == STACK and m.reg('esi') == 0
    assert m.visits[0x8BF3D1] == 32768 and m.visits[0x8BF3D9] == 32768
    assert m.visits[0x8BF3D5] == 32768 and m.visits[0x8BF3DE] == 32768
    samples = [-32768, -32767, -17, -16, -3, -1, 0, 1, 3, 16, 32767]
    full = stack_machine(instructions)
    full.region(COEFF, packed, False)
    design = struct.pack('<11h', *samples)
    full.region(X, design, False)
    full.region(Y, b'\xA5' * 44)
    full.reg('eax', COEFF)
    full.reg('edi', len(samples))
    for n, value in enumerate([0xFFFFFFFF, X, Y]):
        full.word(STACK + n * 4, value=value)
    full.run(0x8BF350, 0xFFFFFFFF)
    actual = struct.unpack('<11i', full.bytes(Y, 44))
    assert list(actual) == [values[v + 32768] for v in samples]
    assert full.reg('esp') == STACK + 4 and full.bytes(X, len(design)) == design
    assert full.bytes(COEFF, 20) == packed
    return values, m.steps + full.steps, dict(
        nonnegative_product_paths=m.visits[0x8BF3D1],
        negative_product_paths=m.visits[0x8BF3D9],
        positive_idiv=m.visits[0x8BF3D5], negative_magnitude_idiv=m.visits[0x8BF3DE],
        optional_neg=m.visits[0x8BF3E0], full_call_sample_points=len(samples))


def load_instructions(image):
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
    operands = {
        0x8BF3A5: ('mov', 'ebp, dword ptr [eax + 4]'),
        0x8BF3AE: ('movsx', 'eax, dx'),
        0x8BF3B1: ('mov', 'dword ptr [esp + 0xc], eax'),
        0x8BF3C0: ('mov', 'ecx, dword ptr [esp + 0x14]'),
        0x8BF3C4: ('movsx', 'ecx, word ptr [ecx + esi*2]'),
        0x8BF3C8: ('imul', 'ecx, dword ptr [esp + 0xc]'),
        0x8BF3CD: ('test', 'ecx, ecx'), 0x8BF3CF: ('jl', '0x8bf3d9'),
        0x8BF3D1: ('lea', 'eax, [ecx + ebx]'), 0x8BF3D4: ('cdq', ''),
        0x8BF3D5: ('idiv', 'ebp'), 0x8BF3D7: ('jmp', '0x8bf3e2'),
        0x8BF3D9: ('mov', 'eax, ebx'), 0x8BF3DB: ('sub', 'eax, ecx'),
        0x8BF3DD: ('cdq', ''), 0x8BF3DE: ('idiv', 'ebp'), 0x8BF3E0: ('neg', 'eax'),
        0x8BF3E2: ('mov', 'edx, dword ptr [esp + 0x18]'),
        0x8BF3E6: ('mov', 'dword ptr [edx + esi*4], eax'),
        0x8BF407: ('movsx', 'esi, word ptr [esi + eax*2]'),
        0x8BF40F: ('imul', 'esi, edx'), 0x8BF412: ('add', 'esi, ebx'),
        0x8BF414: ('sar', 'esi, cl'),
        0x8CC8C1: ('mov', 'ebx, dword ptr [esp + 0xc]'),
        0x8CC8C7: ('mov', 'esi, dword ptr [esp + 0x10]'),
        0x8CC8CB: ('shl', 'esi, 6'),
        0x8CD03D: ('mov', 'word ptr [esi + 8], ax'),
        0x8CD047: ('mov', 'word ptr [esi + 0xa], ax'),
        0x8CD051: ('mov', 'word ptr [esi + 0xc], ax'),
        0x8BF905: ('lea', 'ecx, [ebx + 0x140]'),
        0x8BF916: ('lea', 'eax, [ebx + 0x154]'),
        0x8C0596: ('lea', 'edi, [ebx + 4]'), 0x8C0599: ('call', '0x8bf350'),
    }
    for address, expected in operands.items():
        i = instructions[address]
        assert (i.mnemonic, i.op_str) == expected, f'operand pin mismatch at {address:08X}'
    return instructions, proofs, operands


def parameter_text(parameters):
    if parameters['selector'] == 2:
        return 'unsupported\t2'
    assert parameters['selector'] in (0, 1)
    return '\t'.join(str(parameters[key]) for key in PARAMETER_FIELDS)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    for path in [args.out, args.report]:
        path.resolve().relative_to((ROOT / 'target').resolve())
        assert not path.exists()
    assert hashlib.sha256((ROOT / 'tools/spui_font_scaling_probe.py').read_bytes()).hexdigest() == BASE_ORACLE_SHA256
    image = SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    instructions, proofs, operands = load_instructions(image)
    division_audit = signed_division_semantics(instructions)
    totals, state, word_tables, word_reports, domain_counts = Counter(), hashlib.sha256(), {}, [], Counter()
    checked_bulk_copy()
    with args.out.open('xb') as output:
        def emit(text):
            data = (text + '\n').encode('ascii')
            output.write(data)
            state.update(data)
            totals['rows'] += 1

        for ppem in PPEMS:
            for units in UNITS:
                params, _, steps = coefficients(instructions, ppem, units)
                totals['instruction_steps'] += steps
                domain_counts[str(params['selector'])] += 1
                emit(f'domain\t{ppem}\t{units}\t{parameter_text(params)}')

        for ppem, units in WORD_DOMAINS:
            params, packed, steps = coefficients(instructions, ppem, units)
            totals['instruction_steps'] += steps
            if params['selector'] == 0:
                table, steps = source_word_table(instructions, params, packed)
                paths = dict(full_call_sample_points=11, nonnegative_and_negative_sar_inputs=True)
            else:
                table, steps, paths = quotient_word_table(instructions, params, packed)
            totals['instruction_steps'] += steps
            word_tables[(ppem, units)] = table
            for word, result in zip(range(-32768, 32768), table):
                emit(f'word\t{ppem}\t{units}\t{word}\t{result}')
            word_reports.append(dict(ppem=ppem, units_per_em=units, parameters=params,
                                     signed_word_inputs=len(table), minimum=min(table), maximum=max(table),
                                     negative_results=sum(value < 0 for value in table),
                                     zero_results=table.count(0), positive_results=sum(value > 0 for value in table),
                                     source_branch_paths=paths))

        with args.package.open('rb') as archive:
            header = archive.read(96)
            count, length, offset = (struct.unpack_from('<I', header, p)[0] for p in [0x24, 0x2C, 0x40])
            assert header[:4] == b'DBPF'
            archive.seek(offset)
            block = archive.read(length)
            flags, records = native_index(block, count)
            faces = [r for r in records if r[1] == 0x027C5CEF and r[2] == 0x0248E873]
            assert len(faces) == 5 and {f'{r[0]:08X}' for r in faces} == set(FACES)
            face_reports = []
            for instance, kind, scope, at, stored, decoded, compression, _, _ in sorted(faces):
                archive.seek(at)
                raw = archive.read(stored)
                assert compression in (0, 0xFFFF)
                raw = raw if compression == 0 else decompress(raw, out_size=decoded, header=True)[0]
                digest = hashlib.sha256(raw).hexdigest()
                assert digest == FACES[f'{instance:08X}'] and len(raw) == decoded
                font = TTFont(io.BytesIO(raw))
                assert 'vmtx' not in font
                key = f'{kind:08X}:{scope:08X}:{instance:08X}'
                units = font['head'].unitsPerEm
                params, _, steps = coefficients(instructions, 15, units)
                totals['instruction_steps'] += steps
                vadvance, top, steps = vertical(instructions, font['hhea'])
                totals['instruction_steps'] += steps
                assert params['selector'] in (0, 1)
                table = word_tables[(15, units)]
                emit(f'{key}\tparams\t15\t{units}\t{parameter_text(params)}')
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
                    points = list(g.coordinates) + phantoms
                    encoded_scale = ';'.join(f'{table[x + 32768]},{table[y + 32768]}' for x, y in points)
                    counts.update(simple_phantoms=1, scaled_simple_glyphs=1,
                                  scaled_real_points=len(g.coordinates), scaled_phantom_points=4)
                    counts['scaled_selector_' + str(params['selector'])] += 1
                    emit(f'{key}\tglyph\t{gid}\tsimple\t{vadvance},{top}\t{encoded_phantoms}\t{encoded_scale}')
                totals.update(counts)
                face_reports.append(dict(key=key, source_sha256=digest, ppem=15, units_per_em=units,
                                         native_parameters=params, vertical_advance=vadvance,
                                         top_bearing=top, counts=dict(counts)))
    assert totals['rows'] == 379589 and totals['glyphs'] == 51680
    assert totals['simple_phantoms'] == totals['scaled_simple_glyphs'] == 37691
    assert totals['scaled_real_points'] == 2888616 and totals['scaled_phantom_points'] == 150764
    assert totals['scaled_selector_0'] == 37254 and totals['scaled_selector_1'] == 437
    assert totals['excluded_composite'] == 13939 and totals['excluded_empty'] == 50
    result = dict(
        schema='source-pinned-simple-font-integer-quotient-oracle-v1', source_sha256=SOURCE_SHA256,
        source_proofs=proofs, projected_operand_pins={f'{a:08X}': list(v) for a, v in operands.items()},
        oracle_tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        borrowed_selector0_oracle_sha256=BASE_ORACLE_SHA256,
        unexported_port_sha256=hashlib.sha256((ROOT / 'crates/spore-core/src/spui_font_scaling_quotient.rs').read_bytes()).hexdigest(),
        rust_example_sha256=hashlib.sha256((ROOT / 'crates/spore-bevy/examples/spui_font_scaling_quotient_verify.rs').read_bytes()).hexdigest(),
        parameter_domain=dict(ppems=PPEMS, units_per_em=UNITS, cases=len(PPEMS) * len(UNITS),
                              observed_native_selectors=dict(domain_counts),
                              positive_u16_callee_input_domain=True,
                              retail_style_conversion_full_domain_claim=False),
        signed_division_model_audit=division_audit, exhaustive_word_domains=word_reports,
        source_index_flags=f'{flags:08X}', source_index_sha256=hashlib.sha256(block).hexdigest(),
        faces=face_reports, totals=dict(totals), native_code_executed=False,
        canonical_bytes=args.out.stat().st_size, canonical_sha256=state.hexdigest(),
        scope='Actual pinned integer instruction interpreter; every signed word for five admitted '
              'coefficient classes, including negative half ties, maximum multiplier and maximum '
              'positive-u16 denominator. All five installed faces at15ppem: simple contours with '
              'four measured simple phantoms and selectors0/1. Selector2 explicitly unsupported. '
              'Composite and empty packets excluded. No flags, FPGM/PREP/glyph hint execution or raster comparison.')
    with args.report.open('x') as report:
        report.write(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(rows=totals['rows'], scaled_glyphs=totals['scaled_simple_glyphs'],
                          scaled_points=totals['scaled_real_points'] + totals['scaled_phantom_points'],
                          instruction_steps=totals['instruction_steps'],
                          canonical_bytes=result['canonical_bytes'], sha256=result['canonical_sha256'])))


if __name__ == '__main__':
    main()
