"""Independent source SimpleLayout flag and native local-edge operation census."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
from dbpf_extract import payload
from gmdl_skin_probe import indexes
from spui_layout_probe import container, elements


def adjusted(flags):
    # Independent edge-selection form of 97E920. This fixed fixture has parent
    # origin (10,20), spans (800,600) and the source-style bottom caption rect.
    original = [6.,-45.,-3.,-13.]
    offsets = [800. if flags & 8 and not flags & 4 else 0.,
               600. if flags & 2 and not flags & 1 else 0.,
               800. if flags & 8 else 0., 600. if flags & 2 else 0.]
    result = [a+b for a,b in zip(original,offsets)]
    clamped = [result[0],result[1],max(result[0],result[2]),max(result[1],result[3])]
    return ','.join(f'{struct.unpack("<I",struct.pack("<f",v))[0]:08X}' for v in result+clamped)


def image_sizing(properties):
    props = {p['id']:p for p in properties}
    def word(ident, default):
        p = props.get(ident)
        if p is None: return default
        assert p['type']=='0009' and p['count']==1
        return struct.unpack('<I',bytes.fromhex(p['value']))[0]
    flags = word('4F3C4A27',0)
    horizontal = word('4F3C4A29',3)
    vertical = word('4F3C4A28',3)
    scale_bits = 0x3f800000
    if '4F3C4A2B' in props:
        p = props['4F3C4A2B']
        assert p['type']=='000B' and p['count']==1
        scale_bits = struct.unpack('<I',bytes.fromhex(p['value']))[0]
    scale = struct.unpack('<f',struct.pack('<I',scale_bits))[0]
    f32 = lambda value:struct.unpack('<f',struct.pack('<f',value))[0]
    out = [5.,90.,799.,213.]
    for bit, first, last, size, align in [(1,0,2,420,horizontal),(2,1,3,166,vertical)]:
        if not flags & bit: continue
        length = f32(size*scale)
        if align==2: out[first] = f32(out[last]-length)
        elif align==3:
            span = f32(out[last]-out[first])
            padding = f32(span-length)
            out[first] = f32(f32(padding*0.5)+out[first])
        out[last] = f32(out[first]+length)
    assert all(abs(v) < float('inf') for v in out)
    words = ','.join(f'{struct.unpack("<I",struct.pack("<f",v))[0]:08X}' for v in out)
    return f'{flags:08X}\t{scale_bits:08X}\t{horizontal:08X}\t{vertical:08X}\t{words}'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('install',type=Path)
    ap.add_argument('--rust',type=Path,required=True)
    ap.add_argument('--out',type=Path,required=True)
    ap.add_argument('--images',type=Path)
    args = ap.parse_args()
    rows = []
    image_rows = []
    counts = Counter()
    packages = resources = missing = 0
    for path in sorted(args.install.rglob('*.package')):
        packages += 1
        for entry in indexes(path):
            if entry['type'] != 0x250fe9a2:
                continue
            resources += 1
            raw = payload(path,entry)
            header = container(raw)
            body = elements(raw,header['body_offset'])
            base = len(header['images'])+len(header['atlases'])+header['hit_mask_count']
            for element in body['records']:
                prefix = f'{path.name}\t{entry["instance"]:016X}\t{element["index"]}'
                class_id = header['proxies'][element['index']-base]
                if args.images and class_id == 0xcf3c4eae:
                    image_rows.append(prefix+'\t'+image_sizing(element['properties']))
                if class_id != 0x2f418d72:
                    continue
                prop = next((p for p in element['properties'] if p['id']=='0F3D0000'),None)
                if prop is None:
                    rows.append(prefix+'\tmissing-flags')
                    missing += 1
                    continue
                assert prop['type']=='0007' and prop['count']==1
                source = bytes.fromhex(prop['value'])
                assert len(source)==1
                flags = source[0]
                counts[f'{flags:02X}'] += 1
                rows.append(prefix+f'\t{flags:02X}\t{adjusted(flags)}')
    expected = sorted(rows)
    actual = sorted(args.rust.read_text().splitlines())
    report = dict(packages=packages,resources=resources,simple_layouts=len(rows),
        explicit_flags=sum(counts.values()),missing_flags=missing,flag_counts=dict(counts),
        rust_records=len(actual),exact=expected==actual,global_ui_claim=False,
        sorted_lf_sha256=hashlib.sha256(('\n'.join(expected)+'\n').encode()).hexdigest())
    if args.images:
        expected_images = sorted(image_rows)
        actual_images = sorted(args.images.read_text().splitlines())
        report['image_sizing'] = dict(instances=len(image_rows),rust_records=len(actual_images),
            exact=expected_images==actual_images,fixture=True,final_ui_claim=False,
            sorted_lf_sha256=hashlib.sha256(('\n'.join(expected_images)+'\n').encode()).hexdigest())
    args.out.write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2))
    assert report['exact'], 'source local layout mismatch'
    if args.images: assert report['image_sizing']['exact'], 'source image sizing mismatch'


if __name__=='__main__':
    main()
