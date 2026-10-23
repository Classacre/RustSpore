"""Independent native Image property and float32 atlas mapping comparison."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from dbpf_extract import payload
from gmdl_skin_probe import indexes
from spui_layout_probe import container, elements


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('install', type=Path)
    parser.add_argument('--rust', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    rows = []
    packages = resources = outside = unresolved = 0
    flags = {}
    for path in sorted(args.install.rglob('*.package')):
        packages += 1
        for entry in indexes(path):
            if entry['type'] != 0x250fe9a2:
                continue
            resources += 1
            raw = payload(path, entry)
            header = container(raw)
            body = elements(raw, header['body_offset'])
            resource_count = len(header['images']) + len(header['atlases'])
            base = resource_count + header['hit_mask_count']
            for element in body['records']:
                if header['proxies'][element['index']-base] != 0x01be6b15:
                    continue
                properties = {p['id']:p for p in element['properties']}
                def scalar(ident, kind, size):
                    p = properties[ident]
                    assert p['type'] == kind and p['count'] == 1
                    data = bytes.fromhex(p['value'])
                    assert len(data) == size
                    return data
                reference = struct.unpack('<H',scalar('01BE0001','0013',2))[0]
                uv = struct.unpack('<4f',scalar('01BE0002','0010',16))
                dimensions = struct.unpack('<2I',scalar('01BE0003','000F',8))
                flag = struct.unpack('<I',scalar('01BE0004','0009',4))[0]
                assert all(abs(v) < float('inf') for v in uv)
                spans = [f32(uv[2]-uv[0]),f32(uv[3]-uv[1])]
                mapped = [f32(f32(relative*spans[i%2])+uv[i%2])
                    for i,relative in enumerate([-0.25,0.125,1.25,0.875])]
                words = ','.join(f'{struct.unpack("<I",struct.pack("<f",v))[0]:08X}' for v in (*uv,*mapped))
                prefix = f'{path.name}\t{entry["instance"]:016X}\t{element["index"]}'
                rows.append(f'{prefix}\t{reference:04X}\t{flag:08X}\t{dimensions[0]:08X},{dimensions[1]:08X}\t{words}')
                outside += any(v < 0. or v > 1. for v in uv)
                unresolved += reference >= resource_count
                flags[f'{flag:08X}'] = flags.get(f'{flag:08X}',0)+1
    rows.sort()
    actual = sorted(args.rust.read_text().splitlines())
    report = dict(packages=packages,resources=resources,images=len(rows),
        uv_outside_unit_rectangle=outside,non_resource_references=unresolved,
        flags=flags,rust_records=len(actual),exact=rows==actual,
        sorted_lf_sha256=hashlib.sha256(('\n'.join(rows)+'\n').encode()).hexdigest(),
        mapped_uv_fixture=[-0.25,0.125,1.25,0.875],final_ui_claim=False)
    args.out.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
    assert report['exact'], 'native Image property or UV mapping mismatch'


if __name__ == '__main__':
    main()
