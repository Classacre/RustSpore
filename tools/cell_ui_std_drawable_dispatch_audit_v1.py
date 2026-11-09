"""Retain bounded source StdDrawable dispatch evidence; no native execution."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
from cell_schema_probe import SourceImage, SOURCE_SHA256


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('exe', type=Path)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    args.out.resolve().relative_to((Path(__file__).resolve().parent.parent / 'target').resolve())
    if args.out.exists():
        raise FileExistsError(args.out)
    source = SourceImage(args.exe)
    words = lambda address, count: list(struct.unpack('<' + 'I' * count, source.read(address, count * 4)))
    primary = words(0x1445da8, 8)
    modes = words(0x987d40, 4)
    renderer_slot = words(0x1441470 + 0x58, 1)[0]
    if primary != [0x980c30, 0x9806b0, 0x988500, 0x987950, 0x987d50, 0x9880f0, 0x988400, 0x9883d0]:
        raise ValueError('unexpected constructor vtable')
    if modes != [0x987c70, 0x987c8f, 0x987cc6, 0x987d06]:
        raise ValueError('unexpected draw helper mode table')
    if renderer_slot != 0x956860:
        raise ValueError('unexpected measured Graphics2D slot')
    if source.read(0x1445d04, 12) != b'StdDrawable\0':
        raise ValueError('unexpected class name')
    md = Cs(CS_ARCH_X86, CS_MODE_32)
    ranges = [(0x988430, 0xc0), (0x987a00, 0x77), (0x987c00, 0x13d),
              (0x987d50, 0x3a0), (0x9880f0, 0x2c6), (0x1300ad0, 0x128),
              (0x95d120, 0x534), (0x956860, 0x31), (0x957a50, 0x68)]
    decoded = []
    for address, length in ranges:
        data = source.read(address, length)
        decoded.append({'start': f'{address:08X}', 'bytes': length,
                        'sha256': hashlib.sha256(data).hexdigest(),
                        'instructions': [{'address': f'{i.address:08X}', 'bytes': i.bytes.hex(),
                                          'mnemonic': i.mnemonic, 'operands': i.op_str}
                                         for i in md.disasm(data, address)]})
    report = {'scope': __doc__, 'source_sha256': SOURCE_SHA256,
              'tool_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'constructor_primary_vtable': [f'{x:08X}' for x in primary],
              'mode_1_through_4_targets': [f'{x:08X}' for x in modes],
              'graphics2d_slot_58': f'{renderer_slot:08X}',
              'constants_f32_bits': {f'{a:08X}': f'{words(a, 1)[0]:08X}' for a in (0x14861d0, 0x1471b2c, 0x1485e24)},
              'decoded_ranges': decoded}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open('x', encoding='utf-8') as out:
        json.dump(report, out, indent=2)
        out.write('\n')
    print(f'Pinned executable decoded: {len(ranges)} bounded ranges; mode 2 -> {modes[1]:08X}; no native execution')


if __name__ == '__main__':
    main()
