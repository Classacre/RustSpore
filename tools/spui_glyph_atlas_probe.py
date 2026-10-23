"""Independent synthetic corpus for the source DXT3 glyph-cache primitives.

Static source model: SporeApp.exe 88A400..88A8BB byte-alpha row-word writes
and 88A8C0..88A9B8 white/transparent clear. It follows the SDK instruction
loop layout, not the Rust implementation. Invalid-input statuses are the
bounded Rust API contract, not claimed native error handling. No original
glyph bitmap, font rasterizer, glyph placement or game screen is synthesized.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import random
import struct

MAX_USIZE = (1 << 64) - 1


def required(side, pitch):
    if not 0 < side <= 0x7fffffff:
        return 'InvalidLayout', 0
    blocks = (side + 3) >> 2
    row_bytes = blocks * 16
    size = (blocks - 1) * pitch + row_bytes
    if pitch < row_bytes or size > MAX_USIZE:
        return 'InvalidLayout', 0
    return 'ok', size


def native_clear(dst, side, pitch):
    count = (side + 3) >> 2
    pointer = 0
    for _ in range(count):
        cursor = pointer
        for _ in range(count):
            struct.pack_into('<IIII', dst, cursor, 0, 0, 0xffffffff, 0)
            cursor += 16
        pointer += pitch


def native_alpha8(dst, pitch, x, y, width, height, src, source_pitch):
    # The native full-group path packs four high nibbles into a row word;
    # the tail runs backward over its remaining bytes and writes one word.
    block_row = (y >> 2) * pitch + x * 4
    source_row = 0
    for current_y in range(y, y + height):
        cursor = block_row + (current_y & 3) * 2
        groups, tail = divmod(width, 4)
        for group in range(groups):
            value = struct.unpack_from('<I', src, source_row + group * 4)[0]
            packed = ((value >> 4) & 0x000f) | ((value >> 8) & 0x00f0)
            packed |= ((value >> 12) & 0x0f00) | ((value >> 16) & 0xf000)
            struct.pack_into('<H', dst, cursor, packed)
            cursor += 16
        if tail:
            packed = 0
            for i in range(tail - 1, -1, -1):
                packed = (packed << 4) + (src[source_row + groups * 4 + i] >> 4)
            struct.pack_into('<H', dst, cursor, packed)
        source_row += source_pitch
        if (current_y + 1) & 3 == 0:
            block_row += pitch


def evaluate(op, side, pitch, initial, x, y, width, height, source_pitch, source):
    dst = bytearray(initial)
    status, size = required(side, pitch)
    if status != 'ok':
        return status, dst
    if len(dst) < size:
        return 'ShortDestination', dst
    if op in ('C', 'CW'):
        native_clear(dst, side, pitch)
    if op == 'C':
        return 'ok', dst
    if x & 3:
        return 'UnalignedX', dst
    if x + width > min(MAX_USIZE, side) or y + height > min(MAX_USIZE, side):
        return 'InvalidRectangle', dst
    if width == 0 or height == 0:
        return 'ok', dst
    source_size = (height - 1) * source_pitch + width
    if source_pitch < width or source_size > MAX_USIZE or len(source) < source_size:
        return 'ShortSource', dst
    native_alpha8(dst, pitch, x, y, width, height, source, source_pitch)
    return 'ok', dst


def pattern(length, seed):
    return bytes(((i * 73 + seed * 29) ^ (i >> 2) ^ (seed >> 1)) & 255 for i in range(length))


def corpus():
    cases = []

    def add(op, side, pitch, initial, x=0, y=0, width=0, height=0, source_pitch=0, source=b'', label=''):
        ident = f'{len(cases):05d}-{label or op}'
        cases.append((ident, op, side, pitch, bytes(initial), x, y, width, height, source_pitch, bytes(source)))

    for side in (1,2,3,4,5,7,8,9,15,16,17,31,32,33,63,64):
        for pad in (0,1,7,16,31):
            pitch = ((side + 3) >> 2) * 16 + pad
            size = required(side, pitch)[1]
            for suffix in (0,17):
                add('C', side, pitch, pattern(size + suffix, side + pad), label='clear')
    # Every possible source alpha occurs, including each nibble threshold.
    for start in range(0,256,4):
        add('W', 4, 16, pattern(23,start), width=4,height=1,source_pitch=4,
            source=bytes(range(start,start+4)),label='all-alpha')
    for pitch in (64,65,80):
        for source_pad in (0,3):
            for x in (0,4,8,12):
                for width in range(1,min(16-x,9)+1):
                    for y in (0,1,2,3,4,7,12,15):
                        for height in sorted({1,min(3,16-y),min(5,16-y)}):
                            src_pitch = width + source_pad
                            src_len = (height-1) * src_pitch + width
                            seed = len(cases)
                            add('W',16,pitch,pattern(required(16,pitch)[1]+13,seed),x,y,width,height,
                                src_pitch,pattern(src_len,seed+1),label='row-tail-pitch')
    rng = random.Random(0x88a400)
    for i in range(1000):
        side = rng.choice((5,7,8,9,12,13,16,20,31,32))
        pitch = ((side+3)>>2)*16 + rng.randrange(20)
        x = rng.randrange((side+3)//4)*4
        y = rng.randrange(side+1)
        width, height = rng.randrange(side-x+1), rng.randrange(side-y+1)
        src_pitch = width + rng.randrange(7)
        src_size = (height-1)*src_pitch+width if width and height else 0
        add('CW' if i%3==0 else 'W',side,pitch,pattern(required(side,pitch)[1]+rng.randrange(20),i),
            x,y,width,height,src_pitch,pattern(src_size+rng.randrange(8),i+7),label='mixed')
    page = pattern(64,17)
    invalid = [
        ('C',0,0,page,0,0,0,0,0,b''),
        ('C',0x80000000,32,page,0,0,0,0,0,b''),
        ('C',8,31,page,0,0,0,0,0,b''),
        ('C',8,MAX_USIZE,page,0,0,0,0,0,b''),
        ('C',8,32,page[:-1],0,0,0,0,0,b''),
        ('W',8,32,page[:-1],0,0,1,1,1,b'\xff'),
        ('W',8,32,page,1,0,1,1,1,b'\xff'),
        ('W',8,32,page,4,0,5,1,5,b'\xff'*5),
        ('W',8,32,page,0,7,1,2,1,b'\xff'*2),
        ('W',8,32,page,MAX_USIZE-3,0,8,1,8,b'\xff'*8),
        ('W',8,32,page,0,MAX_USIZE,1,1,1,b'\xff'),
        ('W',8,32,page,0,0,5,2,5,b'\xff'*9),
        ('W',8,32,page,0,0,5,2,4,b'\xff'*10),
        ('W',8,32,page,0,0,1,2,MAX_USIZE,b'\xff'),
        ('W',8,32,page,0,0,0,8,0,b''),
        ('W',8,32,page,0,0,8,0,MAX_USIZE,b''),
        ('W',8,32,page,8,8,0,0,0,b''),
    ]
    for values in invalid:
        add(*values,label='bounded-contract')
    return cases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inputs',type=Path,required=True)
    parser.add_argument('--expected',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--compare',type=Path)
    args = parser.parse_args()
    inputs, expected, statuses, operations = [], [], Counter(), Counter()
    for case in corpus():
        name,op,side,pitch,initial,x,y,width,height,source_pitch,source = case
        status,result = evaluate(op,side,pitch,initial,x,y,width,height,source_pitch,source)
        statuses[status] += 1
        operations[op] += 1
        inputs.append('\t'.join(map(str,(name,op,side,pitch,initial.hex() or '-',x,y,width,height,source_pitch,source.hex() or '-'))))
        expected.append(f'{name}\t{status}\t{result.hex()}')
    input_data, expected_data = (('\n'.join(lines)+'\n').encode('ascii') for lines in (inputs,expected))
    args.inputs.write_bytes(input_data)
    args.expected.write_bytes(expected_data)
    report = {'cases':len(expected),'operations':dict(operations),'statuses':dict(statuses),
        'expected_lf_sha256':hashlib.sha256(expected_data).hexdigest(),
        'source_regions':['88A400..88A8BB','88A8C0..88A9B8'],
        'scope':'synthetic byte-alpha/clear instruction model and explicit bounded API guards; no glyph raster output'}
    if args.compare:
        actual = args.compare.read_bytes().replace(b'\r\n',b'\n')
        report.update(rust_rows=len(actual.splitlines()),rust_lf_sha256=hashlib.sha256(actual).hexdigest(),
                      exact_match=actual==expected_data)
        if actual!=expected_data:
            report['first_difference'] = next(({'line':i+1,'python':e.decode(),'rust':a.decode()}
                for i,(e,a) in enumerate(zip(expected_data.splitlines(),actual.splitlines())) if e!=a),{'row_counts_differ':True})
    args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps(report,indent=2))
    if args.compare and not report['exact_match']:
        raise SystemExit(1)


if __name__=='__main__':
    main()
