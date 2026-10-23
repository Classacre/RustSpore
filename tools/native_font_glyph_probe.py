"""Prepare, never execute, a pinned ordinary native-font glyph fixture.

Original bytes and generated C/header/assembly/executable live under target/.
The ordinary path keeps all native FPGM/PREP/glyph hint handlers. A separate
root review of the fresh bundle is required before any host execution.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path[:0] = [str(ROOT / 'tools'), str(ROOT / 'target/rea/capstone-runtime')]
from cell_schema_probe import SourceImage, SOURCE_SHA256
from native_crt_acos_probe import PE, SOURCE_SHA256 as CRT_SHA256
from capstone import Cs, CS_ARCH_X86, CS_MODE_32, CS_GRP_CALL, CS_GRP_JUMP
from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG

FONT_SHA256 = 'f6ad6f1e12e2d2a7c1e28b0ea3cb93a653482840b2b6340fb8682f3eb4c75f26'
IMPORTS = {
    'GetStdHandle', 'ReadFile', 'WriteFile', 'GetProcessHeap', 'HeapAlloc', 'HeapFree',
    'VirtualAlloc', 'VirtualProtect', 'FlushInstructionCache', 'GetLastError',
    'ExitProcess', 'SetErrorMode', 'AddVectoredExceptionHandler'
}
SOURCE_ENTRIES = {
    0x8A6E60, 0x8D08D0, 0x8CBBC0, 0x8B5F50, 0x8D02F0, 0x8BF1A0,
    0x8BF6E0, 0x8C1310, 0x8BFDF0, 0x8C1600, 0x8CEFB0, 0x8CBCE0, 0x8D0C50
}
REPORTS = {
    'frontier': 'target/source-ui/font-fixture-frontier-v7.json',
    'ordinary': 'target/source-ui/font-ordinary-path-static-v1.json',
    'contracts': 'target/source-ui/font-fixture-contracts-static-v1.json',
    'switches': 'target/source-ui/font-switch-data-v8.json',
    'fpgm': 'target/source-ui/font-fpgm-bootstrap-static-v1.json',
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def freeze_review_bundle(output):
    lock = output / 'bundle-lock.json'
    assert not lock.exists(), 'a frozen review bundle must not be overwritten'
    files = [{'path': path.name, 'bytes': path.stat().st_size, 'sha256': digest(path.read_bytes())}
             for path in sorted(output.iterdir()) if path.is_file()]
    lock.write_text(json.dumps({
        'schema': 'native-font-glyph-review-lock-v1',
        'host_executed': False, 'glyph_bitmap_verified': False,
        'policy': 'Frozen preparation artifacts; append later runtime evidence in a separate directory. '
                  'Any executable/code/data change requires a fresh review bundle.',
        'files': files,
    }, indent=2) + '\n')
    return digest(lock.read_bytes())


def coalesce(ranges):
    result = []
    for start, end in sorted(set(ranges)):
        assert start < end
        if result and start <= result[-1][1]:
            result[-1] = (result[-1][0], max(result[-1][1], end))
        else:
            result.append((start, end))
    return result


def c_bytes(name, data):
    lines = [f'static const unsigned char {name}[{len(data)}] = {{']
    for n in range(0, len(data), 24):
        lines.append('    ' + ','.join(f'0x{x:02X}' for x in data[n:n + 24]) + ',')
    return '\n'.join(lines + ['};'])


def register_name(instruction, register):
    name = instruction.reg_name(register)
    aliases = {
        'al': 'eax', 'ah': 'eax', 'ax': 'eax',
        'bl': 'ebx', 'bh': 'ebx', 'bx': 'ebx',
        'cl': 'ecx', 'ch': 'ecx', 'cx': 'ecx',
        'dl': 'edx', 'dh': 'edx', 'dx': 'edx',
        'si': 'esi', 'di': 'edi', 'bp': 'ebp', 'sp': 'esp',
    }
    return aliases.get(name, name)


def owned_register_targets(decoded, imports, map_path):
    """Conservative intraprocedural CFG constants, with Win32 ABI clobbers.

    Every function starts with unknown registers. A join retains a value
    only when every predecessor proves the same constant/import; loops
    reach a fixed point. Partial writes invalidate their whole register.
    Original callable entries obey the measured cdecl callee-save policy.
    This audits owned call targets, not original heap/VM memory safety.
    """
    boundaries = {i.address: i for i in decoded}
    start, end = decoded[0].address, decoded[-1].address + decoded[-1].size
    symbols = {}
    for line in map_path.read_text().splitlines():
        match = re.match(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\S+)\s+([0-9A-Fa-f]{8})\s+f\s+', line)
        if match and start <= int(match[2], 16) < end:
            symbols.setdefault(int(match[2], 16), []).append(match[1])
    assert min(symbols) == start and all(a in boundaries for a in symbols)
    starts = sorted(symbols)
    owners, states = {}, {}
    tracked = {'eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'ebp'}
    for ordinal, entry in enumerate(starts):
        limit = starts[ordinal + 1] if ordinal + 1 < len(starts) else end
        function = {a: i for a, i in boundaries.items() if entry <= a < limit}
        owners.update({a: symbols[entry] for a in function})
        incoming, pending = {entry: {}}, [entry]
        while pending:
            address = pending.pop()
            instruction = function[address]
            current = incoming[address]
            outgoing = dict(current)
            for register in instruction.regs_access()[1]:
                outgoing.pop(register_name(instruction, register), None)
            operands = instruction.operands
            if instruction.mnemonic == 'mov' and operands[0].type == X86_OP_REG and operands[0].size == 4:
                destination = register_name(instruction, operands[0].reg)
                source, value = operands[1], None
                if source.type == X86_OP_IMM:
                    value = ('constant', source.imm & 0xFFFFFFFF, frozenset([address]))
                elif source.type == X86_OP_REG and source.size == 4:
                    value = current.get(register_name(instruction, source.reg))
                elif source.type == X86_OP_MEM and not source.mem.segment and not source.mem.base and not source.mem.index and source.mem.disp in imports:
                    value = ('import', source.mem.disp, frozenset([address]))
                if value is not None and destination in tracked:
                    outgoing[destination] = value
            elif instruction.mnemonic == 'xor' and len(operands) == 2 and operands[0].type == X86_OP_REG and operands[0].size == 4 and operands[1].type == X86_OP_REG and operands[0].reg == operands[1].reg:
                destination = register_name(instruction, operands[0].reg)
                if destination in tracked:
                    outgoing[destination] = ('constant', 0, frozenset([address]))
            if instruction.group(CS_GRP_CALL):
                for register in ('eax', 'ecx', 'edx'):
                    outgoing.pop(register, None)
            successors = []
            fallthrough = address + instruction.size
            if instruction.group(CS_GRP_JUMP):
                if operands[0].type == X86_OP_IMM:
                    target = operands[0].imm & 0xFFFFFFFF
                    if target in function:
                        successors.append(target)
                    else:
                        assert instruction.mnemonic == 'jmp', (hex(address), hex(target))
                if instruction.mnemonic != 'jmp' and fallthrough < limit:
                    successors.append(fallthrough)
            elif not instruction.mnemonic.startswith('ret') and instruction.mnemonic not in ('int3', 'ud2') and fallthrough < limit:
                successors.append(fallthrough)
            for target in successors:
                assert target in function, (hex(address), hex(target))
                if target not in incoming:
                    incoming[target] = dict(outgoing)
                    pending.append(target)
                    continue
                previous = incoming[target]
                merged = {}
                for register, value in previous.items():
                    alternative = outgoing.get(register)
                    if alternative is not None and alternative[:2] == value[:2]:
                        merged[register] = (*value[:2], value[2] | alternative[2])
                if merged != previous:
                    incoming[target] = merged
                    pending.append(target)
        states.update(incoming)
    proofs = {}
    for instruction in decoded:
        if not (instruction.group(CS_GRP_CALL) or instruction.group(CS_GRP_JUMP)) or instruction.operands[0].type != X86_OP_REG:
            continue
        value = states.get(instruction.address, {}).get(register_name(instruction, instruction.operands[0].reg))
        assert value is not None, ('unresolved owned register transfer', hex(instruction.address), instruction.op_str)
        kind, target, origins = value
        if kind == 'constant':
            assert target in SOURCE_ENTRIES or (target == 0x78592370 and instruction.mnemonic == 'jmp'), (hex(instruction.address), hex(target))
        else:
            assert kind == 'import' and target in imports
        proofs[instruction.address] = {
            'kind': 'CFG-proven source entry' if kind == 'constant' else 'CFG-proven cached allowlisted import',
            'target': f'{target:08X}', 'name': imports[target][1] if kind == 'import' else None,
            'owner': owners[instruction.address], 'constant_or_import_loads': [f'{a:08X}' for a in sorted(origins)],
            'policy': 'All incoming CFG paths agree; volatile EAX/ECX/EDX killed at every call; partial writes invalidate full register.',
        }
    return proofs, len(symbols)


def audit_owned(executable, map_path):
    image = PE(executable)
    assert image.base == 0x24000000
    imports = image.imports()
    imported = {name for dll, name in imports.values()}
    assert imported == IMPORTS, (imported - IMPORTS, IMPORTS - imported)
    assert all(dll.lower() == 'kernel32.dll' for dll, _ in imports.values())
    code_section = [x for x in image.sections if x[4] == '.text']
    assert len(code_section) == 1
    start, _, raw_size, virtual_size, _ = code_section[0]
    size = virtual_size
    cs = Cs(CS_ARCH_X86, CS_MODE_32)
    cs.detail = True
    decoded = list(cs.disasm(image.read(start, size), start))
    assert sum(i.size for i in decoded) == size
    boundaries = {i.address for i in decoded}
    register_proofs, function_count = owned_register_targets(decoded, imports, map_path)
    external_calls, indirect = [], []
    for n, instruction in enumerate(decoded):
        if not (instruction.group(CS_GRP_CALL) or instruction.group(CS_GRP_JUMP)):
            continue
        operand = instruction.operands[0]
        if operand.type == X86_OP_IMM:
            target = operand.imm & 0xFFFFFFFF
            if target in boundaries:
                continue
            assert target in SOURCE_ENTRIES, (hex(instruction.address), instruction.mnemonic, hex(target))
            external_calls.append({'at': f'{instruction.address:08X}', 'target': f'{target:08X}'})
        elif operand.type == X86_OP_MEM and not operand.mem.segment and not operand.mem.base and not operand.mem.index:
            assert operand.mem.disp in imports, (hex(instruction.address), instruction.op_str)
            indirect.append({'at': f'{instruction.address:08X}', 'kind': 'allowlisted kernel32 import',
                             'name': imports[operand.mem.disp][1]})
        else:
            assert operand.type == X86_OP_REG, (hex(instruction.address), instruction.op_str)
            indirect.append({'at': f'{instruction.address:08X}',
                             'operation': instruction.mnemonic + ' ' + instruction.op_str,
                             **register_proofs[instruction.address]})
    return {
        'image_base': f'{image.base:08X}', 'imports': sorted(imported),
        'text_range': [f'{start:08X}', f'{start + size:08X}'],
        'text_bytes': size, 'instruction_count': len(decoded),
        'map_function_count': function_count, 'map_sha256': digest(map_path.read_bytes()),
        'register_indirect_transfer_count': len(register_proofs), 'unresolved_owned_transfers': [],
        'native_direct_calls': external_calls, 'indirect_transfers': indirect,
        'full_disassembly': [f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in decoded],
    }


def prepare(args):
    output = args.out.resolve()
    output.relative_to((ROOT / 'target').resolve())
    assert not output.exists(), 'use a fresh bundle directory'
    source = SourceImage(args.source_exe)
    crt = PE(args.crt_dll)
    assert digest(crt.data) == CRT_SHA256 and crt.export('_setjmp3') == 0x78592370
    font = args.font.read_bytes()
    assert digest(font) == FONT_SHA256
    reports = {name: json.loads((ROOT / path).read_text()) for name, path in REPORTS.items()}
    ordinary, contracts = reports['ordinary'], reports['contracts']
    assert ordinary['source_sha256'] == contracts['source_sha256'] == SOURCE_SHA256
    assert contracts['crt_sha256'] == CRT_SHA256
    assert not ordinary['unknown_direct_targets_or_edges'] and not ordinary['unknown_indirect_jumps']
    assert not ordinary['other_memory_operands']['active']
    assert not ordinary['other_data_literal_candidates']['active']
    assert ordinary['hint_opcode_root_count'] == 94
    assert ordinary['active_function_count'] == 269 and ordinary['active_code_bytes'] == 75464
    assert contracts['ordinary_indirect_call_categories'] == {
        'source IStream adapter virtual seek/read': 2, 'source longjmp import': 12,
        'native hint opcode table': 1, 'native hint internal math/rounding/recursive loop': 51,
        'input-stream read callback field shape': 28,
    }
    assert len(contracts['source_setjmp3_callsites']) == 12
    assert reports['fpgm']['definition_count'] == 53
    assert reports['fpgm']['independent_disassembly']['definition_ids_agree_in_source_order']

    cs = Cs(CS_ARCH_X86, CS_MODE_32)
    cs.detail = True
    all_functions = {x['entry']: x for x in reports['frontier']['functions']}
    active_addresses, code_intervals = set(), []
    for f in ordinary['active_functions']:
        original = all_functions[f['entry']]
        raw = b''.join(source.read(int(a, 16), int(b, 16) - int(a, 16)) for a, b in original['ranges'])
        assert digest(raw) == original['sha256']
        instructions = {}
        for a, b in original['ranges']:
            a, b = int(a, 16), int(b, 16)
            part = list(cs.disasm(source.read(a, b - a), a))
            assert sum(i.size for i in part) == b - a
            instructions.update({i.address: i for i in part})
        for address in f['active_instruction_addresses']:
            at = int(address, 16)
            assert at not in active_addresses
            active_addresses.add(at)
            code_intervals.append((at, at + instructions[at].size))
    code_intervals = coalesce(code_intervals)
    assert sum(b - a for a, b in code_intervals) == ordinary['active_code_bytes']
    switch_intervals, used_switches = [], []
    for row in reports['switches']['switches']:
        if int(row['jump'], 16) not in active_addresses:
            continue
        assert row['local_bound']['guard_fallthrough_dominates_jump_in_resolved_cfg']
        assert all(int(target, 16) in active_addresses for target in row['local_bound']['targets'])
        used_switches.append(row['jump'])
        for part in row['local_bound']['data_ranges']:
            start, end = int(part['start'], 16), int(part['end'], 16)
            assert digest(source.read(start, end - start)) == part['sha256']
            switch_intervals.append((start, end))
    switch_intervals = coalesce(switch_intervals)
    assert all(not (a < d and c < b) for a, b in code_intervals for c, d in switch_intervals)
    setjmp_start, setjmp_end = 0x78592370, 0x785923EB
    setjmp = crt.read(setjmp_start, setjmp_end - setjmp_start)
    assert digest(setjmp) == contracts['crt_setjmp3']['sha256']
    assert contracts['crt_setjmp3']['selected_buffer_bytes_written'] == 40
    assert contracts['crt_setjmp3']['admitted_buffer_bytes'] == 64
    hint_start, hint_end = 0x1432E68, 0x1433330
    hint = source.read(hint_start, hint_end - hint_start)
    assert digest(hint) == '76fd876ea4656f67fd2de02fcfa1f0b7fa2dd302fa55e7cf76fc4df2acf6669b'

    parts = []
    for start, end in code_intervals:
        parts.append((start, source.read(start, end - start), 'source font code', 'RX'))
    for start, end in switch_intervals:
        parts.append((start, source.read(start, end - start), 'source compiler-switch data', 'RX'))
    parts += [(setjmp_start, setjmp, 'source CRT setjmp3 code', 'RX'),
              (hint_start, hint, 'source bounded hint constants and opcode pointers', 'R'),
              (0x13CC430, bytes(4), 'owned longjmp IAT slot', 'R'),
              (0x13CC440, bytes(4), 'owned checked setjmp3 IAT slot', 'R'),
              (0x13CC478, bytes(8), 'owned memset/memcpy IAT slots', 'R'),
              (0x1668A2C, bytes(12), 'registered owned allocator callback globals', 'RW')]
    occupied, pages, protections = {}, {}, {}
    for start, data, kind, protection in parts:
        for index, byte in enumerate(data):
            address, offset = (start + index) & ~0xFFF, (start + index) & 0xFFF
            if address not in pages:
                pages[address] = bytearray([0xCC if protection == 'RX' else 0xA5] * 4096)
                protections[address] = protection
            assert protections[address] == protection
            assert start + index not in occupied
            occupied[start + index] = kind
            pages[address][offset] = byte
    reservations = sorted({a & ~0xFFFF for a in pages})
    read_ranges = coalesce([(a, a + len(data)) for a, data, _, _ in parts])
    payload_parts = [{'start': f'{a:08X}', 'end_exclusive': f'{a + len(data):08X}',
                      'bytes': len(data), 'kind': kind, 'protection': protection,
                      'sha256': digest(data)} for a, data, kind, protection in parts]
    sfnt_tables = {}
    for n in range(struct.unpack_from('>H', font, 4)[0]):
        tag, _, start, size = struct.unpack_from('>4sIII', font, 12 + n * 16)
        assert start <= len(font) and size <= len(font) - start
        sfnt_tables[tag] = font[start:start + size]
    maxp = sfnt_tables[b'maxp']
    glyph_count = struct.unpack_from('>H', maxp, 4)[0]
    function_capacity = struct.unpack_from('>H', maxp, 20)[0]
    storage_capacity = struct.unpack_from('>H', maxp, 18)[0]
    cvt_count = len(sfnt_tables[b'cvt ']) // 2
    assert glyph_count == 578 and function_capacity == 85 and storage_capacity == 47
    assert len(sfnt_tables[b'cvt ']) % 2 == 0
    output.mkdir(parents=True, exist_ok=False)
    header = [
        '#ifndef NATIVE_FONT_PAYLOAD_H', '#define NATIVE_FONT_PAYLOAD_H',
        'typedef struct {unsigned long address,size; const unsigned char *bytes; unsigned long protection;} NativeMap;',
        'typedef struct {unsigned long address,size;} NativeReservation;',
        'typedef struct {unsigned long address,size;} NativeReadRange;',
        'typedef struct {unsigned long index,start,length;} NativeDefinition;',
        f'#define FONT_BYTE_COUNT {len(font)}UL', f'#define FONT_GLYPH_COUNT {glyph_count}UL',
        f'#define FONT_FUNCTION_CAPACITY {function_capacity}UL',
        f'#define FONT_STORAGE_CAPACITY {storage_capacity}UL', f'#define FONT_CVT_COUNT {cvt_count}UL',
        '#define FONT_DEFINITION_COUNT 53UL', '#define NATIVE_SETJMP_ENTRY 0x78592370',
        c_bytes('font_payload', font),
    ]
    for ordinal, (address, data) in enumerate(sorted(pages.items())):
        header.append(c_bytes(f'native_page_{ordinal}', data))
    protection_flags = {'RX': 0x20, 'R': 0x02, 'RW': 0x04}
    header += [f'#define PAYLOAD_MAP_COUNT {len(pages)}UL', 'static const NativeMap payload_maps[] = {']
    for ordinal, address in enumerate(sorted(pages)):
        header.append(f'{{0x{address:08X}UL,4096UL,native_page_{ordinal},0x{protection_flags[protections[address]]:X}UL}},')
    header += ['};', f'#define PAYLOAD_RESERVATION_COUNT {len(reservations)}UL',
               'static const NativeReservation payload_reservations[] = {']
    header += [f'{{0x{a:08X}UL,65536UL}},' for a in reservations]
    header += ['};', f'#define PAYLOAD_READ_RANGE_COUNT {len(read_ranges)}UL',
               'static const NativeReadRange payload_read_ranges[] = {']
    header += [f'{{0x{a:08X}UL,{b - a}UL}},' for a, b in read_ranges]
    header += ['};', 'static const NativeDefinition font_definitions[] = {']
    header += [f'{{{x["function_index"]}UL,{x["body_start"]}UL,{x["body_bytes"]}UL}},'
               for x in reports['fpgm']['definitions']]
    rounds = [0x8AB680, 0x8AB6B0, 0x8AB6E0, 0x8AB710, 0x8AB740, 0x8ABA30, 0x8AB780, 0x8AB7D0]
    header += ['};', f'#define ROUND_TARGET_COUNT {len(rounds)}UL',
               'static const unsigned long round_targets[] = {' + ','.join(f'0x{x:08X}UL' for x in rounds) + '};',
               '#endif']
    header_path = output / 'native-font-payload.h'
    header_path.write_text('\n'.join(header) + '\n')
    c_path = output / 'host-x86.c'
    template = ROOT / 'tools/native_font_glyph_host.c.in'
    c_path.write_bytes(template.read_bytes())
    asm_path = output / 'entry-x86.asm'
    asm_path.write_text('.386\n.model flat\noption casemap:none\nEXTERN _font_probe_main:PROC\n'
                        'EXTERN _ExitProcess@4:PROC\n.code\nPUBLIC FontProbeEntry\n'
                        'FontProbeEntry PROC\n call _font_probe_main\n push 0\n call _ExitProcess@4\n'
                        'FontProbeEntry ENDP\nEND\n')
    obj, entry_obj = output / 'host-x86.obj', output / 'entry-x86.obj'
    executable, map_path = output / 'host-x86.exe', output / 'host-x86.map'
    command_sets = [
        [str(args.sdk_bin_x86 / 'cl.exe'), '/nologo', '/c', '/TC', '/O1', '/Ob0', '/Oi-', '/Oy-',
         '/GS-', '/Gs9999999', '/Zl', '/W4', '/D_WIN32_WINNT=0x0601',
         f'/I{args.vc_include}', f'/I{args.sdk_include / "shared"}', f'/I{args.sdk_include / "um"}',
         f'/I{args.sdk_include / "ucrt"}', f'/Fo{obj}', str(c_path)],
        [str(args.sdk_bin_x86 / 'ml.exe'), '/nologo', '/c', '/coff', f'/Fo{entry_obj}', str(asm_path)],
        [str(args.sdk_bin_x86 / 'link.exe'), '/NOLOGO', '/MACHINE:X86', '/SUBSYSTEM:CONSOLE',
         '/ENTRY:FontProbeEntry', '/BASE:0x24000000', '/DYNAMICBASE:NO', '/SAFESEH:NO',
         '/NODEFAULTLIB', '/OPT:REF', '/OPT:ICF', f'/OUT:{executable}', f'/MAP:{map_path}',
         str(obj), str(entry_obj), str(args.kernel32_lib)],
    ]
    logs = []
    for command in command_sets:
        completed = subprocess.run(command, capture_output=True, text=True,
                                   creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0))
        logs.append({'command': command, 'returncode': completed.returncode,
                     'stdout': completed.stdout, 'stderr': completed.stderr})
        (output / 'build-log.json').write_text(json.dumps(logs, indent=2) + '\n')
        assert completed.returncode == 0, completed.stdout + '\n' + completed.stderr
    owned = audit_owned(executable, map_path)
    (output / 'owned-host-audit.json').write_text(json.dumps(owned, indent=2) + '\n')
    (output / 'owned-host-disassembly.asm').write_text('\n'.join(owned['full_disassembly']) + '\n')
    report_paths = {}
    for name, path in REPORTS.items():
        report_paths[name] = {'path': path, 'sha256': digest((ROOT / path).read_bytes())}
    manifest = {
        'schema': 'bounded-original-font-glyph-host-v1',
        'source_path': str(args.source_exe.resolve()), 'source_sha256': SOURCE_SHA256,
        'crt_path': str(args.crt_dll.resolve()), 'crt_sha256': CRT_SHA256,
        'font_path': str(args.font.resolve()), 'font_sha256': FONT_SHA256,
        'host_executed': False, 'glyph_bitmap_verified': False, 'original_module_loaded': False,
        'owned_template_sha256': digest(template.read_bytes()),
        'owned_preparer_sha256': digest(Path(__file__).read_bytes()),
        'generated_header_sha256': digest(header_path.read_bytes()),
        'owned_c_sha256': digest(c_path.read_bytes()), 'owned_entry_sha256': digest(asm_path.read_bytes()),
        'host_exe_sha256': digest(executable.read_bytes()), 'analysis_reports': report_paths,
        'native_parts': payload_parts, 'native_code_instruction_count': len(active_addresses),
        'native_font_code_bytes': sum(b - a for a, b in code_intervals),
        'native_crt_setjmp3': contracts['crt_setjmp3'],
        'ordinary_source_predicates': ordinary['policy'],
        'ordinary_predicate_dead_pointer_fact': ordinary['dead_overwritten_pointer_literal_admissions'],
        'hint_opcode_targets_retained': 94, 'active_compiler_switches': used_switches,
        'compiler_switch_data_bytes': sum(b - a for a, b in switch_intervals),
        'hint_constant_bytes': len(hint),
        'reservations': [{'address': f'{a:08X}', 'bytes': 65536, 'initial': 'NOACCESS reserve'} for a in reservations],
        'committed_pages': [{'address': f'{a:08X}', 'bytes': 4096, 'initial': 'RW commit',
                             'final': protections[a], 'sha256_before_owned_slot_initialization': digest(data)}
                            for a, data in sorted(pages.items())],
        'omitted_code_policy': 'Only selected source instructions and bounded switch data are copied; '
                               'other committed RX-page bytes are INT3. Uncommitted reservation pages '
                               'stay NOACCESS. Unused data-page bytes are A5, not original image bytes.',
        'provider_policy': contracts['receiver_contracts'],
        'owned_imports': sorted(IMPORTS), 'owned_host_audit': 'owned-host-audit.json',
        'owned_host_audit_sha256': digest((output / 'owned-host-audit.json').read_bytes()),
        'owned_disassembly_sha256': digest((output / 'owned-host-disassembly.asm').read_bytes()),
        'owned_map_sha256': digest(map_path.read_bytes()),
        'owned_register_transfer_audit': {
            'function_entries': owned['map_function_count'],
            'register_indirect_sites': owned['register_indirect_transfer_count'],
            'unresolved_sites': owned['unresolved_owned_transfers'],
            'method': 'Intraprocedural CFG fixed point; all predecessor values must agree. '
                      'Calls invalidate EAX/ECX/EDX and partial writes invalidate their full register.',
            'limit': 'This audits owned call targets, not all original computed heap/VM accesses.',
        },
        'compiler_seh_policy': 'Manual owned 16-byte current-stack SEH record; /SAFESEH:NO, '
                               '/GS- and no CRT handler. C4733 FS:0 warning retained for review; '
                               'VEH reports faults and terminates without recovery.',
        'heap_policy': {'alignment': 16, 'raw_allocation_limit': 1048576, 'live_byte_limit': 67108864,
                        'ledger_slots': 4096, 'poisons': [0, 165], 'layouts': [0, 1],
                        'canaries': '16 bytes before/after every provided allocation',
                        'final_live_allocations_required': 0},
        'source_stream_policy': 'Only owned two-word absolute seek and exact read slots; immutable '
                                'font bytes, checked native-heap/current-stack destination, no file forwarding.',
        'error_policy': 'True pinned setjmp3 via checked tail entry; owned stack SEH registration, '
                        'count0 and64-byte buffer admission. Native longjmp reports original status '
                        'and exits; no native recovery equivalence. VEH reports faults and exits.',
        'input_protocol': {'header_u32': ['magic:FNT1', 'poison_low8|layout_bit8', 'mode:0fresh/1sequence', 'count:1..64'],
                           'body': 'count little-endian u32 glyph indices, each0..577',
                           'required': 'Exact EOF after request; whole protocol validated before native mapping/execution'},
        'output_protocol': {'header_bytes': 48, 'context_header_bytes': 44,
                            'glyph_header_bytes': 72, 'hint_state_header_bytes': 20,
                            'hint_state_body': f'{cvt_count} CVT s32 then47 storage s32 values',
                            'glyph_body': 'hint state then pitch*height byte-alpha bytes',
                            'footer_bytes': 60, 'error_record_bytes': 48,
                            'padding_policy': 'Pitched bytes retained as evidence; only width bytes per row '
                                              'and defined metrics form semantic bitmap comparisons',
                            'hint_snapshot_policy': 'Full CVT/storage snapshots are diagnostic raw evidence. '
                                                    'Storage words are not semantic equality inputs without '
                                                    'individual initialization/write proof.'},
        'font_inputs': {'style_size': 11, 'resolution': [96, 96], 'fixed_matrix': [720896, 0, 0, 720896],
                        'expected_ppem': [15, 15], 'mode': 3, 'flags': '0x8B',
                        'fpgm_definitions_checked_before_transform': 53, 'function_capacity': function_capacity,
                        'cvt_count': cvt_count, 'storage_capacity': storage_capacity,
                        'true_fpgm_prep_and_glyph_hinting': 'Original code/programs retained; runtime outcome unverified'},
        'compile_commands': command_sets,
        'next_step': 'Root reviews all fresh code/data/branch/provider/FS/heap/import/protocol evidence '
                     'and the concrete generated host before authorizing any hidden piped execution.'
    }
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    freeze_review_bundle(output)
    print(json.dumps({key: manifest[key] for key in ['host_executed', 'glyph_bitmap_verified',
        'host_exe_sha256', 'native_font_code_bytes', 'native_code_instruction_count',
        'compiler_switch_data_bytes', 'hint_constant_bytes', 'font_inputs']}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-exe', type=Path, required=True)
    parser.add_argument('--crt-dll', type=Path, required=True)
    parser.add_argument('--font', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--sdk-bin-x86', type=Path, required=True)
    parser.add_argument('--vc-include', type=Path, required=True)
    parser.add_argument('--sdk-include', type=Path, required=True)
    parser.add_argument('--kernel32-lib', type=Path, required=True)
    prepare(parser.parse_args())


if __name__ == '__main__':
    main()
