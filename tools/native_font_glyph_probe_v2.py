"""Prepare, never execute, the versioned native-font EIF-adapter fixture.

V1 stays frozen. Only its low RET leaf is omitted; one opcode pointer is
replaced at runtime by an owned, exactly-one-byte RET adapter. All other
source code/data and provider policies are required to match V1 exactly.
"""
import argparse
from copy import deepcopy
import difflib
import json
from pathlib import Path
import re
import struct
import subprocess

from native_font_glyph_probe import (ROOT, SourceImage, PE, SOURCE_SHA256,
                                    CRT_SHA256, audit_owned, digest, freeze_review_bundle)

V1_LOCK_SHA256 = '88f5166c26770d62496c063f99aa341e9a5e708bf7cecc96ec203cf9d47a5fd3'
V1_PREPARER_SHA256 = '21ceadf686a11d8f98539fb824a4541be0a9fdb3ef00e4abcbfa6bd713174896'
LOW_ENTRY, LOW_PAGE, LOW_RESERVATION = 0x67C860, 0x67C000, 0x670000
EIF_SLOT = 0x1432F30 + 0x59 * 4


def replace_once(text, before, after):
    assert text.count(before) == 1, before
    return text.replace(before, after)


def map_symbol(path, name):
    values = []
    for line in path.read_text().splitlines():
        match = re.match(r'^\s*[0-9A-Fa-f]{4}:[0-9A-Fa-f]{8}\s+(\S+)\s+([0-9A-Fa-f]{8})(?:\s|$)', line)
        if match and match[1] == name:
            values.append(int(match[2], 16))
    assert len(values) == 1, (name, values)
    return values[0]


def prepare(args):
    original, output = args.v1_bundle.resolve(), args.out.resolve()
    original.relative_to((ROOT / 'target').resolve())
    output.relative_to((ROOT / 'target').resolve())
    assert not output.exists(), 'use a fresh review bundle directory'
    lock_path = original / 'bundle-lock.json'
    assert digest(lock_path.read_bytes()) == V1_LOCK_SHA256
    for row in json.loads(lock_path.read_text())['files']:
        data = (original / row['path']).read_bytes()
        assert len(data) == row['bytes'] and digest(data) == row['sha256'], row['path']
    base_preparer = ROOT / 'tools/native_font_glyph_probe.py'
    assert digest(base_preparer.read_bytes()) == V1_PREPARER_SHA256
    v1 = json.loads((original / 'manifest.json').read_text())
    source, crt = SourceImage(Path(v1['source_path'])), PE(Path(v1['crt_path']))
    assert v1['source_sha256'] == SOURCE_SHA256 and digest(crt.data) == CRT_SHA256
    assert source.read(LOW_ENTRY, 1) == b'\xC3'
    assert struct.unpack('<I', source.read(EIF_SLOT, 4))[0] == LOW_ENTRY
    for row in v1['native_parts']:
        address, size = int(row['start'], 16), row['bytes']
        if row['kind'] in ('source font code', 'source compiler-switch data',
                           'source bounded hint constants and opcode pointers'):
            assert digest(source.read(address, size)) == row['sha256']
        elif row['kind'] == 'source CRT setjmp3 code':
            assert digest(crt.read(address, size)) == row['sha256']
        else:
            assert row['kind'].startswith(('owned ', 'registered owned '))
            assert digest(bytes(size)) == row['sha256']
    omitted = [row for row in v1['native_parts'] if int(row['start'], 16) == LOW_ENTRY]
    assert len(omitted) == 1 and omitted[0]['bytes'] == 1
    assert [row['address'] for row in v1['committed_pages'] if int(row['address'], 16) >> 16 == LOW_RESERVATION >> 16] == ['0067C000']

    header = (original / 'native-font-payload.h').read_text()
    low_declaration = re.search(r'static const unsigned char native_page_0\[4096\] = \{.*?\n\};\n', header, re.S)
    assert low_declaration is not None
    low_bytes = bytes(int(value, 16) for value in re.findall(r'0x([0-9A-F]{2})', low_declaration[0]))
    assert len(low_bytes) == 4096 and low_bytes[LOW_ENTRY - LOW_PAGE] == 0xC3
    assert all(value == 0xCC for n, value in enumerate(low_bytes) if n != LOW_ENTRY - LOW_PAGE)
    header = replace_once(header, low_declaration[0], '')
    header = replace_once(header, '#define PAYLOAD_MAP_COUNT 43UL', '#define PAYLOAD_MAP_COUNT 42UL')
    header = replace_once(header, '{0x0067C000UL,4096UL,native_page_0,0x20UL},\n', '')
    header = replace_once(header, '#define PAYLOAD_RESERVATION_COUNT 11UL', '#define PAYLOAD_RESERVATION_COUNT 10UL')
    header = replace_once(header, '{0x00670000UL,65536UL},\n', '')
    count_match = re.search(r'#define PAYLOAD_READ_RANGE_COUNT (\d+)UL', header)
    assert count_match is not None
    header = replace_once(header, count_match[0], f'#define PAYLOAD_READ_RANGE_COUNT {int(count_match[1]) - 1}UL')
    header = replace_once(header, '{0x0067C860UL,1UL},\n', '')

    # These comparisons admit no changed source/font page bytes.
    arrays = lambda text: {name: bytes(int(x, 16) for x in re.findall(r'0x([0-9A-F]{2})', body))
                           for name, body in re.findall(r'static const unsigned char (\w+)\[\d+\] = \{(.*?)\n\};', text, re.S)}
    before, after = arrays((original / 'native-font-payload.h').read_text()), arrays(header)
    assert before.keys() - after.keys() == {'native_page_0'} and not after.keys() - before.keys()
    assert all(data == before[name] for name, data in after.items())
    for n, row in enumerate(v1['committed_pages']):
        if int(row['address'], 16) != LOW_PAGE:
            assert digest(after[f'native_page_{n}']) == row['sha256_before_owned_slot_initialization']

    output.mkdir(parents=True, exist_ok=False)
    header_path, c_path, asm_path = output / 'native-font-payload.h', output / 'host-x86.c', output / 'entry-x86.asm'
    header_path.write_text(header)
    template = ROOT / 'tools/native_font_glyph_host_v2.c.in'
    c_path.write_bytes(template.read_bytes())
    asm_path.write_text('.386\n.model flat\noption casemap:none\nEXTERN _font_probe_main:PROC\n'
                        'EXTERN _ExitProcess@4:PROC\n.code\n'
                        'PUBLIC _owned_eif_adapter\nPUBLIC _owned_eif_adapter_end\n'
                        '_owned_eif_adapter PROC\n ret\n_owned_eif_adapter_end LABEL BYTE\n'
                        '_owned_eif_adapter ENDP\nPUBLIC FontProbeEntry\n'
                        'FontProbeEntry PROC\n call _font_probe_main\n push 0\n call _ExitProcess@4\n'
                        'FontProbeEntry ENDP\nEND\n')
    (output / 'owned-template-delta.diff').write_text(''.join(difflib.unified_diff(
        (original / 'host-x86.c').read_text().splitlines(True), c_path.read_text().splitlines(True),
        fromfile='v1/host-x86.c', tofile='v2/host-x86.c')))
    commands = [[part.replace(str(original), str(output)) for part in command] for command in v1['compile_commands']]
    assert [Path(command[0]).name.lower() for command in commands] == ['cl.exe', 'ml.exe', 'link.exe']
    logs = []
    for command in commands:
        completed = subprocess.run(command, capture_output=True, text=True,
                                   creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0))
        logs.append({'command': command, 'returncode': completed.returncode,
                     'stdout': completed.stdout, 'stderr': completed.stderr})
        (output / 'build-log.json').write_text(json.dumps(logs, indent=2) + '\n')
        assert completed.returncode == 0, completed.stdout + '\n' + completed.stderr
    executable, map_path = output / 'host-x86.exe', output / 'host-x86.map'
    owned = audit_owned(executable, map_path)
    (output / 'owned-host-audit.json').write_text(json.dumps(owned, indent=2) + '\n')
    (output / 'owned-host-disassembly.asm').write_text('\n'.join(owned['full_disassembly']) + '\n')
    adapter, adapter_end = map_symbol(map_path, '_owned_eif_adapter'), map_symbol(map_path, '_owned_eif_adapter_end')
    assert adapter_end == adapter + 1 and PE(executable).read(adapter, 1) == b'\xC3'
    assert sum(line.startswith(f'{adapter:08X}: ret') for line in owned['full_disassembly']) == 1
    expected_store = f'mov dword ptr [0x{str(hex(EIF_SLOT))[2:]}], 0x{adapter:x}'
    assert sum(line.split(': ', 1)[1] == expected_store for line in owned['full_disassembly']) == 1
    hint = bytearray(source.read(0x1432E68, 1224))
    struct.pack_into('<I', hint, EIF_SLOT - 0x1432E68, adapter)
    manifest = deepcopy(v1)
    manifest.update({
        'schema': 'bounded-original-font-glyph-host-v2-owned-eif',
        'v1_bundle': str(original), 'v1_bundle_lock_sha256': V1_LOCK_SHA256,
        'base_preparer_sha256': V1_PREPARER_SHA256,
        'owned_preparer_sha256': digest(Path(__file__).read_bytes()),
        'owned_template_sha256': digest(template.read_bytes()), 'owned_c_sha256': digest(c_path.read_bytes()),
        'owned_entry_sha256': digest(asm_path.read_bytes()), 'generated_header_sha256': digest(header_path.read_bytes()),
        'host_exe_sha256': digest(executable.read_bytes()), 'owned_map_sha256': digest(map_path.read_bytes()),
        'owned_host_audit_sha256': digest((output / 'owned-host-audit.json').read_bytes()),
        'owned_disassembly_sha256': digest((output / 'owned-host-disassembly.asm').read_bytes()),
        'native_parts': [row for row in v1['native_parts'] if row not in omitted],
        'native_code_instruction_count': v1['native_code_instruction_count'] - 1,
        'native_font_code_bytes': v1['native_font_code_bytes'] - 1,
        'reservations': [row for row in v1['reservations'] if int(row['address'], 16) != LOW_RESERVATION],
        'committed_pages': [row for row in v1['committed_pages'] if int(row['address'], 16) != LOW_PAGE],
        'owned_register_transfer_audit': {
            **v1['owned_register_transfer_audit'], 'function_entries': owned['map_function_count'],
            'register_indirect_sites': owned['register_indirect_transfer_count'],
            'unresolved_sites': owned['unresolved_owned_transfers'],
        },
        'source_byte_delta_from_v1': {'omitted': omitted, 'all_retained_source_parts_unchanged': True,
                                     'all_retained_generated_page_bytes_unchanged': True,
                                     'font_payload_unchanged': True},
        'explicit_runtime_pointer_substitution': {
            'name': 'owned EIF return adapter', 'opcode': '0x59', 'slot': f'{EIF_SLOT:08X}',
            'original_target': f'{LOW_ENTRY:08X}', 'owned_target': f'{adapter:08X}',
            'adapter_end_exclusive': f'{adapter_end:08X}', 'adapter_bytes': 1, 'adapter_hex': 'C3',
            'effect': 'Normal RET only: pops return EIP, advances ESP by four, preserves all other GPRs and EFLAGS.',
            'patch_stage': 'After source page copy and checked original pointer, before final R protection.',
            'hint_window_sha256_before_pointer_substitution': v1['native_parts'][next(n for n, r in enumerate(v1['native_parts']) if r['kind'] == 'source bounded hint constants and opcode pointers')]['sha256'],
            'hint_window_sha256_after_pointer_substitution': digest(hint),
            'native_opcode_targets_copied': 93, 'owned_opcode_adapters': 1,
            'scope': 'Explicit harness substitution; source table identity is claimed only before this four-byte patch.',
        },
        'compile_commands': commands,
        'preparation_reason': 'V1 had two phase1 low-reservation failures before any font entry. '
                              'Suspended-child census found that region free; subsequent allocation owner is unknown.',
        'next_step': 'Root reviews this fresh v2 code/data/pointer substitution and frozen bundle before any execution.',
    })
    manifest['input_protocol']['required'] += '; V2 retains V1 regular-file stdin EOF restriction.'
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    lock_sha = freeze_review_bundle(output)
    print(json.dumps({'bundle': str(output), 'host_executed': False, 'glyph_bitmap_verified': False,
                      'host_exe_sha256': manifest['host_exe_sha256'], 'bundle_lock_sha256': lock_sha,
                      'manifest_sha256': digest((output / 'manifest.json').read_bytes()),
                      'adapter': manifest['explicit_runtime_pointer_substitution'],
                      'native_code_bytes': manifest['native_font_code_bytes'],
                      'committed_pages': len(manifest['committed_pages']), 'reservations': len(manifest['reservations'])}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--v1-bundle', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    prepare(parser.parse_args())


if __name__ == '__main__':
    main()
