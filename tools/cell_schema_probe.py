"""Recover Cell interface facts from bounded native static initializers.

This invokes Microsoft's SDK disassembler, not the game. Native disassembly
stays in a new ignored target/ directory. Only the measured executable identity
is admitted; other versions need independent registrar/boundary verification.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess

SOURCE_SHA256 = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
CLASSES = [
    ('cell', 0x1375c70, 0x1376642, 0x15a6fc8, 26),
    ('cAIData', 0x1374560, 0x13758d6, 0x16b4108, 51),
    ('cEatData', 0x1375920, 0x1375aed, 0x15a7680, 5),
    ('cLocalizedString', 0x1375b30, 0x1375c2f, 0x16b2808, 3),
]


class SourceImage:
    def __init__(self, path):
        self.data = path.read_bytes()
        if hashlib.sha256(self.data).hexdigest() != SOURCE_SHA256:
            raise ValueError('unverified source executable identity')
        pe, = struct.unpack_from('<I', self.data, 60)
        count, = struct.unpack_from('<H', self.data, pe+6)
        optional_size, = struct.unpack_from('<H', self.data, pe+20)
        base, = struct.unpack_from('<I', self.data, pe+52)
        self.sections = []
        for i in range(count):
            header = pe+24+optional_size+i*40
            virtual_size, rva, size, at = struct.unpack_from('<4I', self.data, header+8)
            self.sections.append((base+rva, virtual_size, size, at))

    def read(self, address, count):
        for base, virtual_size, size, at in self.sections:
            if base <= address and address+count <= base+max(virtual_size, size):
                delta = address-base
                stored = self.data[at+delta:at+min(size, delta+count)] if delta < size else b''
                return stored + bytes(count-len(stored))
        raise ValueError(f'unmapped source address {address:08X}')

    def ascii(self, address):
        if address == 0:
            return None
        text = self.read(address, 160).split(b'\0')[0]
        if not text or not all(32 <= byte < 127 for byte in text):
            raise ValueError(f'non-ASCII source field name {address:08X}')
        return text.decode('ascii')


def initializer_stores(text):
    """Track only literal/register stores required for names, offsets and types.

    E4A750 is the retained name-hash wrapper. 94E9D0 is the retained identity
    helper. Other call results remain unknown; no default tuning is extracted.
    """
    registers, memory = {}, {}
    last_push = None

    def value(token):
        if token in registers:
            return registers[token]
        if re.fullmatch(r'[0-9A-F]+h', token):
            return int(token[:-1], 16)
        if token.isdigit():
            return int(token)
        return None

    for line in text.splitlines():
        match = re.match(r'\s*([A-F0-9]+): (\w+)\s+(.*)', line)
        if not match:
            continue
        _, operation, operands = match.groups()
        if operation == 'push':
            last_push = value(operands.strip())
        elif operation == 'xor':
            left, right = map(str.strip, operands.split(',', 1))
            if left == right:
                registers[left] = 0
        elif operation == 'call':
            registers['eax'] = last_push if operands.strip() == '0094E9D0' else None
        elif operation == 'mov':
            left, right = map(str.strip, operands.split(',', 1))
            result = value(right)
            if left in ['eax', 'ebx', 'ecx', 'edx', 'esi', 'edi']:
                registers[left] = result
            target = re.fullmatch(r'dword ptr ds:\[([0-9A-F]+)h\]', left)
            if target:
                memory[int(target[1], 16)] = result
    return memory


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe', type=Path)
    ap.add_argument('--dumpbin', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--globals', action='store_true', help='also recover the current 276-byte globals schema')
    args = ap.parse_args()
    image = SourceImage(args.exe)
    # Analysis code and proprietary bytes must remain ignored local evidence.
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent/'target').resolve())
    args.out.mkdir(parents=True, exist_ok=False)
    rows = []
    classes = CLASSES + ([('globals', 0x137bae0, 0x137d5f8, 0x15a7ab8, 69)] if args.globals else [])
    for name, start, end, table, maximum in classes:
        output = args.out/f'{name}-initializer-sdk.asm'
        subprocess.run([str(args.dumpbin), '/DISASM:NOBYTES', f'/RANGE:0x{start:X},0x{end:X}',
            f'/OUT:{output}', str(args.exe)], check=True, capture_output=True)
        stores = initializer_stores(output.read_text())
        for ordinal in range(maximum):
            at = table + ordinal*40

            def field(delta):
                address = at+delta
                return stores.get(address, struct.unpack('<I', image.read(address, 4))[0])

            offset = field(8)
            if offset == 0xffffffff:
                assert ordinal == maximum-1
                break
            pointer, native_type = field(0), field(16)
            if not all(isinstance(v, int) for v in [offset, pointer, native_type]):
                raise ValueError(f'initializer field not established: {name}[{ordinal}]')
            rows.append(dict(struct=name, row=ordinal, descriptor=f'{at:08X}',
                name=image.ascii(pointer), offset=offset, native_type=native_type))
        else:
            raise ValueError(f'missing {name} sentinel')
    assert len(rows) == 81 + (68 if args.globals else 0)
    result = args.out/'schema.json'
    result.write_text(json.dumps(rows, indent=2))
    print(json.dumps(dict(source_sha256=SOURCE_SHA256, classes=len(classes),
        descriptor_rows=len(rows), schema_sha256=hashlib.sha256(result.read_bytes()).hexdigest()), indent=2))


if __name__ == '__main__':
    main()
