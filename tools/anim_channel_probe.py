"""Measure ANIM interleaved channel rows using original package readers.

This is independent of the Rust parser. It reports every selected resource and
does not silently discard unsupported channels or package/read failures.
"""
import argparse
import json
import math
import struct
from collections import Counter
from pathlib import Path

from dbpf_extract import parse_index, payload

TYPE = 0xEE17C6AD


def u32(data, offset):
    return struct.unpack_from('<I', data, offset)[0]


def inspect(data, stats, samples, make_digest=False):
    assert data[:4] == b'ANIM' and u32(data, 4) == len(data)
    n = u32(data, 324)
    table = u32(data, 328)
    stats['header_channel_count'][str((n, u32(data, 304)))] += 1
    offsets = [u32(data, table + 4 * i) for i in range(n)]
    assert all(data[o:o+4] == b'CHAN' for o in offsets)
    feed = bytearray(struct.pack('<I',n))
    def word(value): feed.extend(struct.pack('<I',value))
    for o in offsets:
        stats['channels']['total'] += 1
        name = data[o+8:o+136].split(b'\0')[0].decode('ascii', 'replace')
        count, keys, comps, descriptors = struct.unpack_from('<4I', data, o + 212)
        if make_digest:
            encoded = name.encode('utf-8')
            word(len(encoded)); feed.extend(encoded); word(count)
            feed.extend(data[o+136:o+212]); word(comps)
        assert descriptors + comps * 32 <= len(data)
        stats['descriptor_count'][comps] += 1
        stats['flags'][u32(data, o+136)] += 1
        stride = None
        for c in range(comps):
            p = descriptors + 32*c
            flags, magic, component, step, kind, a, b, c2 = struct.unpack_from('<I4s6I', data, p)
            assert component < step
            assert stride is None or stride == step
            stride = step
            if make_digest: feed.extend(data[p:p+32])
            stats['descriptor'][repr((magic,component,step,kind,a,b,c2,flags))] += 1
            stats['channels']['unsupported_components'] += int(magic not in [b'INFO',b'POS\0',b'ROT\0'])
        assert keys + count * (stride or 0) <= len(data)
        if make_digest:
            components={data[descriptors+32*i+4:descriptors+32*i+8]:u32(data,descriptors+32*i+8) for i in range(comps)}
            word(count)
            for i in range(count):
                row = keys + i*stride
                info = row + components[b'INFO']
                feed.extend(data[info:info+20])
                for tag,width in [(b'POS\0',12),(b'ROT\0',16)]:
                    word(int(tag in components))
                    if tag in components:
                        start=row+components[tag]
                        feed.extend(data[start:start+width])
            word(count*(stride or 0)); feed.extend(data[keys:keys+count*(stride or 0)])
        stats['channels']['empty'] += int(count == 0)
        if count and stride:
            times = [u32(data, keys + i*stride) for i in range(count)]
            stats['channels']['monotonic'] += int(all(a <= b for a,b in zip(times,times[1:])))
            stats['channels']['time_within_last_frame'] += int(max(times) <= u32(data,296))
            duration = struct.unpack_from('<f',data,288)[0]
            stats['channels']['time_below_duration'] += int(max(times) < duration)
            for c in range(comps):
                p = descriptors + 32*c
                flags,magic,component,step,kind = struct.unpack_from('<I4s3I',data,p)
                if magic == b'POS\0':
                    vals=[struct.unpack_from('<3f',data,keys+i*step+component) for i in range(count)]
                    stats['values']['positions'] += len(vals)
                    stats['values']['finite_positions'] += sum(all(math.isfinite(x) for x in v) for v in vals)
                if magic == b'ROT\0':
                    vals=[struct.unpack_from('<4f',data,keys+i*step+component) for i in range(count)]
                    stats['values']['rotations'] += len(vals)
                    stats['values']['finite_rotations'] += sum(all(math.isfinite(x) for x in v) for v in vals)
                    stats['values']['unit_rotations_01'] += sum(abs(sum(x*x for x in v)-1)<.01 for v in vals)
            if len(samples)<8:
                samples.append({'name':name,'count':count,'stride':stride,'times':times[:12],
                                'flags':u32(data,o+136), 'selector_words':[u32(data,o+i) for i in range(136,212,4)]})
    if make_digest:
        result=0xcbf29ce484222325
        for byte in feed: result=((result^byte)*0x100000001b3)&0xFFFFFFFFFFFFFFFF
        return result


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('packages',nargs='*',type=Path)
    parser.add_argument('--out',type=Path)
    parser.add_argument('--digest-out',type=Path)
    args=parser.parse_args()
    packages=args.packages or [Path('D:/Games/SPORE Collection')/p for p in
        ['Data/PatchData.package','Data/Spore_Game.package','DataEP1/Spore_EP1_Data.package','bp1content/BoosterPack_01.package']]
    stats={key:Counter() for key in ['channels','values','descriptor_count','descriptor','flags','header_channel_count']}
    samples=[]
    failures=[]
    totals={}
    digests=[]
    for package in packages:
        entries,_=parse_index(package)
        total=0
        for entry in entries:
            if entry['type'] != TYPE: continue
            total+=1
            try:
                digest=inspect(payload(package,entry),stats,samples,args.digest_out is not None)
                if args.digest_out:
                    digests.append(f"{package.name} {entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X} {digest:016X}")
            except Exception as exc: failures.append({'package':str(package),'instance':f"{entry['instance']:016X}",'error':repr(exc)})
        totals[package.name]=total
    report={'packages':totals,'statistics':stats,'samples':samples,'failures':failures}
    text=json.dumps(report,indent=2)
    if args.out: args.out.write_text(text,encoding='utf-8')
    if args.digest_out: args.digest_out.write_text('\n'.join(sorted(digests))+'\n',encoding='utf-8')
    print(json.dumps({'packages':totals,'channels':stats['channels'],'values':stats['values'],
                      'descriptor_kinds':len(stats['descriptor']),'failures':failures[:12]},indent=2))
    if failures: raise SystemExit(1)


if __name__=='__main__': main()
