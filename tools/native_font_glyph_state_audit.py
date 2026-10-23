"""Independently inspect retained native glyph replies and hint-state changes.

This runs no native host and imports neither the retained reply parser nor the
corpus comparator. Source code/data, font programs and reports remain target/
inputs. The initial glyph-program analysis stops at the first unsupported
opcode; it is not a general hint interpreter or rasterizer.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path[:0] = [str(ROOT / 'tools'), str(ROOT / 'target/rea/capstone-runtime')]
from cell_schema_probe import SourceImage, SOURCE_SHA256
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
from fontTools.ttLib import TTFont

FONT = ROOT / 'target/source-ui/font-file-Spore_Graphics-00010000-027C5CEF-0248E873511B49CA.bin'
FONT_SHA = 'f6ad6f1e12e2d2a7c1e28b0ea3cb93a653482840b2b6340fb8682f3eb4c75f26'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_reply(data, request):
    """Independent fixed-fixture wire reader; reject all unparsed bytes."""
    at = 0

    def take(n):
        nonlocal at
        assert 0 <= n <= len(data) - at
        result = data[at:at + n]
        at += n
        return result

    def words(n):
        return struct.unpack(f'<{n}I', take(n * 4))

    def controls():
        cw, mxcsr, df = words(3)
        assert cw == 0x37F and mxcsr & ~0x3F == 0x1F80 and df == 0

    def hint():
        nc, ns, rounding, cvt_function, flags = words(5)
        assert (nc, ns) == (173, 47) and flags <= 0xFFFF
        return dict(round_function=f'{rounding:08X}', cvt_function=f'{cvt_function:08X}',
                    flags=flags, cvt=list(struct.unpack('<173i', take(692))),
                    storage=list(struct.unpack('<47i', take(188))))

    magic, policy, mode, count = struct.unpack_from('<4I', request)
    ids = list(struct.unpack_from(f'<{count}I', request, 16))
    assert magic == 0x31544E46 and mode in (0, 1) and 1 <= count <= 64
    assert len(request) == 16 + 4 * count and all(g < 578 for g in ids)
    assert policy & ~0x1FF == 0 and policy & 255 in (0, 165)
    header = words(12)
    assert header == (0x31524E46, 1, mode, count, policy & 255, policy >> 8,
                      117628, 578, 173, 47, 53, 85)
    contexts = count if mode == 0 else 1
    rows = []
    coverage = Counter()
    initial_states = []
    for context in range(contexts):
        first, length = (context, 1) if mode == 0 else (0, count)
        ctx = words(8)
        assert ctx[:7] == (0x31585443, context, first, length, 53, 15, 15) and ctx[7] > 0
        controls()
        previous = hint()
        initial_states.append(previous)
        for ordinal in range(first, first + length):
            h = words(15)
            assert h[:4] == (0x31594C47, context, ordinal, ids[ordinal])
            _, _, _, gid, width, height, pitch, size, ex, ey, advance, c8, present, live, live_bytes = h
            assert width <= pitch <= 4096 and height <= 4096 and size == pitch * height
            assert present in (0, 1) and (not size or present) and live and live_bytes
            controls()
            after = hint()
            pixels = take(size)
            active = b''.join(pixels[y * pitch:y * pitch + width] for y in range(height))
            coverage.update(active)
            digest = sha(b'FNTGLYPH1\0' + struct.pack('<7I', gid, width, height, ex, ey, advance, present) + active)
            rows.append(dict(glyph=gid, ordinal=ordinal, context=context, mode=mode,
                             width=width, height=height, offset_e4=ex, offset_e0=ey,
                             advance_d0=advance, before=previous, after=after,
                             active_sha256=sha(active), semantic_sha256=digest))
            previous = after
    footer = words(15)
    assert footer[0] == 0x31444E45 and footer[1] == contexts
    assert footer[2] == footer[3] and footer[-3:] == (0, 0, 0) and at == len(data)
    return dict(mode=mode, ids=ids, rows=rows, contexts=contexts,
                guard_checks=footer[5], initial_states=initial_states, coverage=coverage)


def prefix(program):
    """Resolve only the straight-line push/delta/axis/MIAP prefix."""
    stack, operations = [], []
    at = 0
    while at < len(program):
        start, opcode = at, program[at]
        at += 1
        if opcode in (0x40, 0x41) or 0xB0 <= opcode <= 0xBF:
            n = program[at] if opcode in (0x40, 0x41) else (opcode & 7) + 1
            if opcode in (0x40, 0x41):
                at += 1
            words = opcode == 0x41 or opcode >= 0xB8
            for _ in range(n):
                stack.append(struct.unpack_from('>h', program, at)[0] if words else program[at])
                at += 2 if words else 1
        elif opcode in (0, 1):
            operations.append(dict(offset=start, axis='vertical' if opcode == 0 else 'horizontal'))
        elif opcode in (0x5D, 0x71, 0x72):
            n = stack.pop()
            assert n >= 0 and len(stack) >= n * 2
            if n:
                del stack[-n * 2:]
            operations.append(dict(offset=start, delta_point_pairs=n))
        elif opcode in (0x3E, 0x3F):
            cvt, point = stack.pop(), stack.pop()
            operations.append(dict(offset=start, miap_round=opcode & 1, point=point, cvt_index=cvt))
        else:
            return dict(operations=operations, first_uninterpreted_offset=start,
                        first_uninterpreted_opcode=f'{opcode:02X}')
    return dict(operations=operations, first_uninterpreted_offset=None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cohort', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.resolve().relative_to((ROOT / 'target').resolve())
    assert not args.out.exists()
    assert sha(FONT.read_bytes()) == FONT_SHA
    cohort = args.cohort.resolve()
    cohort.relative_to((ROOT / 'target').resolve())
    manifest = json.loads((cohort / 'manifest.json').read_text())
    run = json.loads((cohort / 'run-v1.json').read_text())
    groups, baseline, source_runs = {}, {}, []
    totals, coverage = Counter(), Counter()
    for request in manifest['requests']:
        path = ROOT / request['request']
        assert sha(path.read_bytes()) == request['request_sha256']
        succeeded = [a for a in run['attempts'] if a['case'] == request['name'] and a['exit'] == 0]
        assert len(succeeded) == 1
        directory = Path(succeeded[0]['report']).parent
        raw = (directory / 'reply.bin').read_bytes()
        reply = read_reply(raw, path.read_bytes())
        totals.update(cases=1, observations=len(reply['rows']), contexts=reply['contexts'],
                      guard_checks=reply['guard_checks'])
        coverage.update(reply['coverage'])
        for g in reply['rows']:
            key = reply['mode'], tuple(reply['ids']), g['ordinal']
            if key in groups:
                assert groups[key] == g, 'metrics/pixels/captured hint state depend on heap policy'
            else:
                groups[key] = g
                baseline[reply['mode'], g['glyph']] = g
            # The file name belongs to the retained protocol, not source font selection.
            pixel = directory / 'active' / f"glyph-{g['ordinal']:02d}-{g['glyph']:03d}.alpha"
            assert sha(pixel.read_bytes()) == g['active_sha256']
        source_runs.append(dict(case=request['name'], reply_sha256=sha(raw)))
    assert totals == dict(cases=80, observations=4624, contexts=2352, guard_checks=7154800)
    assert len(groups) == len(baseline) == 1156
    lines = ''.join(f'{mode}\t{gid:03d}\t{baseline[mode,gid]["semantic_sha256"]}\n'
                    for mode in (0, 1) for gid in range(578)).encode()
    assert sha(lines) == 'bad091f7c103d0b1a7bc7e86ef11d24124122871098013bc5b12f04e1c4156be'

    source = SourceImage(Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    cs = Cs(CS_ARCH_X86, CS_MODE_32)
    # Decode current pinned source bytes rather than admitting unrelated dossier pseudocode.
    checks = {
        0x8CC2DB: ('cmp', 'dword ptr [ebx + 4], eax'),
        0x8CC2E0: ('cmp', 'dword ptr [ebx + 8], ecx'),
        0x8CC2E3: ('je', '0x8cc5ab'),
        0x8CC4A9: ('inc', 'eax'),
        0x8CC4AC: ('mov', 'dword ptr [ecx + eax*4 - 4], esi'),
        0x8CC7B3: ('lea', 'eax, [ebx + 0x18]'),
        0x8CC7B6: ('lea', 'esi, [eax + 0x24]'),
        0x8CC7B9: ('lea', 'edi, [eax + 0x58]'),
        0x8CC7CD: ('mov', 'ecx, 0xd'),
        0x8CC7D2: ('rep movsd', 'dword ptr es:[edi], dword ptr [esi]'),
        0x8CC7E5: ('call', '0x8abce0'),
        0x8ACBB1: ('mov', 'eax, dword ptr [esi + 0x20]'),
        0x8ACBB4: ('mov', 'eax, dword ptr [eax]'),
        0x8ACBB8: ('call', 'ecx'),
        0x8ADCA2: ('mov', 'ecx, dword ptr [eax + 8]'),
        0x8ADCA5: ('mov', 'dword ptr [ecx + edi*4], edx'),
    }
    for address, expected in checks.items():
        op = next(cs.disasm(source.read(address, 15), address))
        assert (op.mnemonic, op.op_str) == expected
    assert struct.unpack('<I', source.read(0x1432F30 + 0x3F * 4, 4))[0] == 0x8ACB90
    assert struct.unpack('<I', source.read(0x1432F30 + 0x44 * 4, 4))[0] == 0x8ADC60
    font = TTFont(FONT)
    order = font.getGlyphOrder()
    names = {}
    for codepoint, name in font.getBestCmap().items():
        names.setdefault(name, []).append(f'U+{codepoint:04X}')
    differences = []
    for gid in range(578):
        fresh, sequence = baseline[0, gid], baseline[1, gid]
        if fresh['semantic_sha256'] == sequence['semantic_sha256']:
            continue
        glyph = font['glyf'][order[gid]]
        assert not glyph.isComposite() and glyph.numberOfContours > 0
        program = glyph.program.getBytecode()
        first = prefix(program)
        consumers = [op for op in first['operations'] if op.get('cvt_index') == 14]
        assert len(consumers) == 1 and consumers[0]['miap_round'] == 1
        assert fresh['before']['cvt'][14] == -192 and sequence['before']['cvt'][14] == -256
        assert sequence['height'] == fresh['height'] + 1
        fields = ['width', 'offset_e4', 'offset_e0', 'advance_d0']
        assert all(fresh[k] == sequence[k] for k in fields)
        assert fresh['after']['storage'] == sequence['after']['storage']
        assert all(fresh['after'][k] == sequence['after'][k] for k in ['round_function', 'cvt_function', 'flags'])
        changes = {phase: [dict(index=i, fresh=a, sequence=b) for i, (a, b) in
                           enumerate(zip(fresh[phase]['cvt'], sequence[phase]['cvt'])) if a != b]
                   for phase in ['before', 'after']}
        differences.append(dict(glyph=gid, name=order[gid], codepoints=names.get(order[gid], []),
                                contours=glyph.numberOfContours, instruction_bytes=len(program),
                                instruction_sha256=sha(program), initial_prefix=first,
                                cvt_differences=changes, fresh=fresh, sequence=sequence))
    assert [d['glyph'] for d in differences] == [354, 357, 364, 366, 371, 373, 442]
    result = dict(schema='native-font-glyph-hint-state-audit-v1', source_sha256=SOURCE_SHA256,
                  font_sha256=FONT_SHA, totals=dict(totals), identical_heap_input_groups=len(groups),
                  captured_hint_states_heap_independent=True, semantic_tsv_sha256=sha(lines),
                  source_checks=[f'{a:08X}' for a in checks], differences=differences,
                  native_runs=source_runs, native_process_launched=False,
                  observed_alpha=dict(bytes=sum(coverage.values()), minimum=min(coverage),
                                      maximum=max(coverage), distinct=len(coverage),
                                      histogram={str(k): v for k, v in sorted(coverage.items())}),
                  conclusion='The seven simple glyphs consume different persistent CVT14 values in their '
                  'straight-line vertical MIAP prefix. Native same-ppem PREP early exit and per-glyph '
                  '13-word graphics-state restore preserve the mutable CVT array. The measured CVT14 '
                  'difference is exactly 64 in 26.6 units and the measured output height difference is one '
                  'pixel. CVT41 also differs. No single-variable intervention or internal point trace '
                  'has yet established that CVT14 is the sole cause. No retail context/reset policy or '
                  'Rust hint/raster equivalence is claimed.')
    with args.out.open('x', encoding='utf-8') as output:
        json.dump(result, output, indent=2)
        output.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ['native_runs', 'differences']}))


if __name__ == '__main__':
    main()
