"""Independent native linear FadeEffect fixtures and installed-property comparison.

Native evidence: 963D70 defaults, 963740 attach/tick, 9635F0 target changes,
96EFB0 alpha conversion. This models bounded linear visibility transitions;
it is not a retail runtime capture or a complete SPUI runner.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import struct

from dbpf_extract import payload
from gmdl_skin_probe import indexes
from spui_layout_probe import container, elements


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def bits(value):
    return f'{struct.unpack("<I", struct.pack("<f", value))[0]:08X}'


def color(progress):
    product = f32(f32(1.0 - progress) * 255.)
    return f'{(math.trunc(product) << 24) | 0x00ffffff:08X}'


def tick(current, target, delta, duration=1.):
    old, target, delta, duration = map(f32, (current, target, delta, duration))
    current = old
    if current != target:
        if delta > duration:
            current = target
        else:
            step = f32(delta / duration)
            if current < target:
                current = min(target, f32(current + step))
            else:
                current = max(target, f32(current - step))
    completed = abs(current - target) < f32(.001)
    if completed:
        current = target
    return current, completed, color(current) if current != old else '-'


def fixture_rows():
    ticks, sequences = [], []
    for current in [0., .25, .5, .75, 1.]:
        for target in [0., 1.]:
            for delta in [0., .0005, .001, .01, .02, .1, .25, .5, .999, 1., 1.25]:
                out, completed, shade = tick(current, target, delta)
                ticks.append('\t'.join([bits(current), str(int(target)), bits(delta),
                                        bits(out), str(int(completed)), shade]))
    for initial, target in [(0., 1.), (1., 0.)]:
        for name, deltas in [('quarters', [.25]*4), ('tenths', [.1]*10),
                             ('sixty_hz', [1./60.]*60), ('snap', [.5,.4995]),
                             ('threshold', [.5,.499])]:
            current = initial
            name += f'_{int(initial)}to{int(target)}'
            for index, delta in enumerate(deltas):
                current, completed, shade = tick(current, target, delta)
                sequences.append('\t'.join([name,bits(initial),str(int(target)),str(index),
                    bits(delta),bits(current),str(int(completed)),shade]))
    assert len(ticks) == 110 and len(sequences) == 156
    return ticks, sequences


def source_rows(install):
    rows, counts = [], Counter()
    package_count = resource_count = 0
    dt = f32(1./60.)
    for path in sorted(install.rglob('*.package')):
        package_count += 1
        for entry in indexes(path):
            if entry['type'] != 0x250fe9a2:
                continue
            resource_count += 1
            raw = payload(path, entry)
            header = container(raw)
            body = elements(raw, header['body_offset'])
            base = len(header['images']) + len(header['atlases']) + header['hit_mask_count']
            for element in body['records']:
                if header['proxies'][element['index']-base] != 0x6f2b6d2c:
                    continue
                props = {p['id']:p for p in element['properties']}
                values = [0.,1,0]
                for i, default in enumerate(values):
                    ident = f'{0x4f2b0000+i:08X}'
                    p = props.get(ident)
                    if p is None:
                        continue
                    assert p['count'] == 1 and p['type'] == ('000B' if i==0 else '0009')
                    values[i] = struct.unpack('<f' if i==0 else '<I',bytes.fromhex(p['value']))[0]
                duration, trigger, interpolation = values
                prefix = [path.name,f'{entry["instance"]:016X}',str(element['index']),
                          bits(duration),f'{trigger:08X}',f'{interpolation:08X}']
                if interpolation:
                    status = 'unsupported-interpolation'
                elif trigger > 1:
                    status = 'unsupported-trigger'
                elif not math.isfinite(duration) or duration < 0:
                    status = 'invalid-duration'
                else:
                    status = 'linear'
                counts[status] += 1
                result = ['-']*7
                if status == 'linear':
                    initial = 1. if trigger==0 else 0.
                    first_target = 1.-initial
                    current, done, shade = tick(initial,first_target,dt,duration)
                    current2, done2, shade2 = tick(current,initial,dt,duration)
                    result = [color(initial),bits(current),shade,str(int(done)),
                              bits(current2),shade2,str(int(done2))]
                rows.append('\t'.join(prefix+[status]+result))
    rows.sort()
    return rows, dict(packages=package_count,resources=resource_count,
                      fades=len(rows),status_counts=dict(counts),tick_seconds_bits=bits(dt))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('install',type=Path,nargs='?')
    parser.add_argument('--fixture-dir',type=Path)
    parser.add_argument('--rust',type=Path)
    parser.add_argument('--expected',type=Path)
    parser.add_argument('--out',type=Path)
    args = parser.parse_args()
    ticks, sequences = fixture_rows()
    report = dict(native_provenance=['963D70','963740','9635F0','96EFB0'],
                  tick_fixtures=len(ticks),sequences=10,sequence_frames=len(sequences),
                  final_ui_claim=False)
    if args.fixture_dir:
        args.fixture_dir.mkdir(parents=True,exist_ok=True)
        for name, header, data in [
            ('spui_linear_fade_ticks.tsv','before_bits target delta_bits after_bits completed callback_argb',ticks),
            ('spui_linear_fade_sequences.tsv','name initial_bits target frame delta_bits after_bits completed callback_argb',sequences)]:
            content='# Native-derived bounded linear FadeEffect fixtures; generated independently by tools/spui_effect_probe.py\n'
            content+='# '+header+'\n'+'\n'.join(data)+'\n'
            (args.fixture_dir/name).write_text(content,encoding='utf-8',newline='\n')
    if args.install:
        rows, census = source_rows(args.install)
        report.update(census)
        report['sorted_lf_sha256'] = hashlib.sha256(('\n'.join(rows)+'\n').encode()).hexdigest()
        if args.expected:
            args.expected.write_text('\n'.join(rows)+'\n',encoding='utf-8',newline='\n')
        if args.rust:
            actual = sorted(args.rust.read_text().splitlines())
            report.update(rust_records=len(actual),exact=actual==rows)
    if args.out:
        args.out.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
    if args.rust:
        assert report.get('exact'), 'source FadeEffect parameter/linear runtime mismatch'


if __name__=='__main__':
    main()
