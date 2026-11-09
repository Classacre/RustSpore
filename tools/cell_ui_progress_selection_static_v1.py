"""Retain bounded x86 listings for the Cell progress selector; never runs game code."""
import argparse
import json
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
from cell_schema_probe import SourceImage, SOURCE_SHA256

RANGES = [(0xE283A0, 0xE28471), (0xE296F0, 0xE2980C), (0xE29BD3, 0xE29C8B)]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('exe', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    image = SourceImage(args.exe)
    decoder = Cs(CS_ARCH_X86, CS_MODE_32)
    listings = []
    for start, end in RANGES:
        raw = image.read(start, end-start)
        insns = list(decoder.disasm(raw, start))
        if not insns or insns[-1].address + insns[-1].size != end:
            raise ValueError(f'incomplete instruction decode at {start:08X}')
        listings.append({'start': f'{start:08X}', 'end_exclusive': f'{end:08X}',
                         'bytes_hex': raw.hex(), 'instructions': [
                             {'address': f'{i.address:08X}', 'mnemonic': i.mnemonic,
                              'operands': i.op_str} for i in insns]})
    result = {'source_sha256': SOURCE_SHA256, 'scope': 'static bytes; no execution or live state', 'listings': listings}
    with args.output.open('x', encoding='utf-8', newline='\n') as f:
        json.dump(result, f, indent=2)
        f.write('\n')

if __name__ == '__main__':
    main()
