"""Bounded raw-byte scout for Cell camera manager setter; never executes target.

A byte match is a candidate, not an instruction or control-flow proof. In particular,
this scan cannot find aliasing, indirect calls with different encodings, or remote
uses of returned objects. Inspect candidates independently before drawing conclusions.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

PIN = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'

def scan(path):
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != PIN:
        raise ValueError('unexpected executable SHA-256')
    pe = struct.unpack_from('<I', data, 0x3c)[0]
    if data[pe:pe+4] != b'PE\0\0' or struct.unpack_from('<H', data, pe+24)[0] != 0x10b:
        raise ValueError('expected PE32')
    count, opt = struct.unpack_from('<H', data, pe+6)[0], struct.unpack_from('<H', data, pe+20)[0]
    base = struct.unpack_from('<I', data, pe+52)[0]
    sections = pe + 24 + opt
    matches = []
    for i in range(count):
        s = sections + 40*i
        if data[s:s+8].split(b'\0')[0] != b'.text':
            continue
        _, va, size, raw = struct.unpack_from('<IIII', data, s+8)
        b = data[raw:raw+size]
        if len(b) != size:
            raise ValueError('truncated section')
        getters, setters = [], []
        for j in range(len(b)-5):
            if b[j] == 0xe8:
                dest = base + va + j + 5 + struct.unpack_from('<i', b, j+1)[0]
                if dest == 0x67d830: getters.append(j)
                if dest == 0x7d8180: setters.append(j)
        for j in getters:
            for k in range(j+5, min(j+301, len(b)-6)):
                if b[k] == 0xff and ((0x50 <= b[k+1] <= 0x57 and b[k+2] == 0x54) or
                                     (0x90 <= b[k+1] <= 0x97 and b[k+2:k+6] == b'\x54\0\0\0')):
                    matches.append([f'{base+va+j:08X}', f'{base+va+k:08X}', b[k:k+6].hex()])
        return dict(sha256=digest, section='.text', section_va=f'{base+va:08X}', raw_size=size,
                    getter_raw_e8_candidates=len(getters), direct_setter_raw_e8_candidates=[f'{base+va+x:08X}' for x in setters],
                    forward_window_bytes=300, raw_indirect_ff_plus54_candidates=matches)
    raise ValueError('no .text section')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('exe', type=Path)
    print(json.dumps(scan(parser.parse_args().exe), indent=2))
