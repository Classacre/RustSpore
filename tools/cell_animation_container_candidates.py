"""Locate source-byte candidates for Cell animation-container lifecycle.

This is a read-only static locator, not a disassembler or proof that a byte
pattern is an instruction. Only the verified executable is accepted. The
returned instruction candidates require independent control-flow disassembly.
"""
import argparse
import hashlib
import json
from pathlib import Path

from cell_schema_probe import SourceImage, SOURCE_SHA256

PATTERNS = {
    'offset_161bc_mov_eax_ebp': bytes.fromhex('89 a8 bc 61 01 00'),
    'offset_161c8_mov_eax_ebp': bytes.fromhex('89 a8 c8 61 01 00'),
    'offset_161bc_add_ebp': bytes.fromhex('81 c5 bc 61 01 00'),
    'offset_161c8_add_ebp': bytes.fromhex('81 c5 c8 61 01 00'),
}


def scan(image):
    found = {}
    for name, pattern in PATTERNS.items():
        matches = []
        for base, _virtual_size, size, offset in image.sections:
            data = image.data[offset:offset + size]
            start = 0
            while True:
                pos = data.find(pattern, start)
                if pos < 0:
                    break
                start = pos + 1
                context = data[max(0, pos - 16):min(size, pos + len(pattern) + 16)]
                matches.append({'address': f'{base + pos:08X}',
                                'context_sha256': hashlib.sha256(context).hexdigest()})
        found[name] = matches
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('exe', type=Path, help='original SporeApp.exe')
    args = parser.parse_args()
    image = SourceImage(args.exe)
    print(json.dumps({'source_sha256': SOURCE_SHA256,
                      'method': 'raw bytes; control flow and runtime types unverified',
                      'candidates': scan(image)}, indent=2))


if __name__ == '__main__':
    main()
