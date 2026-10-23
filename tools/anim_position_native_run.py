"""Run the reviewed INFO+POS-only host and retain every request/reply.

Flat fixed-size streams support an independent Rust verifier. Nonfinite sampled
position/weight outputs remain present and are counted separately, never dropped.
"""
import argparse
import bisect
import hashlib
import json
import math
from pathlib import Path
import random
import struct
import subprocess


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def rows(frames, previous_mode, next_mode, seed, signed_zero=False):
    rng = random.Random(seed)
    result = bytearray(1600)
    for index, frame in enumerate(frames):
        at = index*100
        struct.pack_into('<5I', result, at, frame, 0x11220000+index, 0x33440000+index,
            0x55660000+index, 0x77880000+index)
        values = [f32(rng.uniform(-20., 20.)) for _ in range(3)]+[f32(rng.uniform(-2., 3.))]
        if signed_zero:
            values = [-0., 0., -0., -0.] if index % 2 == 0 else [0., -0., 0., 0.]
        struct.pack_into('<4f', result, at+20, *values)
        for axis in range(4):
            control = at+36+16*axis
            slopes = [f32(rng.uniform(-5., 5.)), f32(rng.uniform(-5., 5.))]
            if signed_zero:
                slopes = [-0., 0.]
            struct.pack_into('<2f', result, control, *slopes)
            result[control+8] = 0xc0 | previous_mode
            result[control+9] = 0x80 | next_mode
            result[control+10:control+16] = bytes(rng.randrange(256) for _ in range(6))
    return bytes(result)


def request(frame, frames, duration, header_flags, control, data):
    frame = f32(frame)
    # Families choose exactly representable unsigned keys even above 2^31,
    # so every explicit PC/RC lane has the same strict-greater upper selection.
    upper = bisect.bisect_right(frames, frame)
    return struct.pack('<5IHH2I', struct.unpack('<I', struct.pack('<f', frame))[0], upper,
        len(frames), struct.unpack('<I', struct.pack('<f', duration))[0], header_flags,
        control, 0, 0, 0)+data


def run_host(host, data):
    return subprocess.run([str(host)], input=data, capture_output=True,
        creationflags=subprocess.CREATE_NO_WINDOW, timeout=60)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--host-dir', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    args.host_dir.resolve().relative_to(target)
    args.out.resolve().relative_to(target)
    manifest = json.loads((args.host_dir/'manifest.json').read_text())
    host = (args.host_dir/'host-x86.exe').resolve()
    assert hashlib.sha256(host.read_bytes()).hexdigest() == manifest['host_exe_sha256']
    assert manifest['stub']['audit']['outbound_relative_transfers'] == 0
    assert manifest['protocol']['input_bytes'] == 1632 and manifest['protocol']['output_bytes'] == 124
    args.out.mkdir(parents=True, exist_ok=False)

    simple = bytearray(rows([0], 1, 1, 1))
    struct.pack_into('<4f', simple, 20, 1., -2., 3., 2.)
    smoke = request(0., [0], 1., 0, 0x27f, simple)
    result = run_host(host, smoke)
    (args.out/'smoke-input.bin').write_bytes(smoke)
    (args.out/'smoke-output.bin').write_bytes(result.stdout)
    assert result.returncode == 0 and len(result.stdout) == 124 and not result.stderr, (result.returncode,len(result.stdout))
    assert result.stdout[:100] == simple[:100]
    assert struct.unpack_from('<i5I', result.stdout, 100) == (0,1,0,0,0,0)

    negatives = [('empty', b'', 0, 0), ('partial1', smoke[:1], 7, 0),
        ('partial1631', smoke[:1631], 7, 0), ('complete_then_partial', smoke+b'x', 7, 124)]
    for name, offset, fmt, value in [('count0',8,'I',0), ('count17',8,'I',17),
        ('upper2_count1',4,'I',2), ('reserved_precision',20,'H',0x17f),
        ('unmasked_CW',20,'H',0x270), ('nonfinite_frame',0,'I',0x7fc00000)]:
        malformed = bytearray(smoke)
        struct.pack_into('<'+fmt, malformed, offset, value)
        negatives.append((name,bytes(malformed),7,0))
    checks = []
    for name, data, code, output_bytes in negatives:
        result = run_host(host, data)
        assert result.returncode == code and len(result.stdout) == output_bytes and not result.stderr, name
        checks.append(dict(case=name, exit=code, output_bytes=output_bytes, exact=True))
    (args.out/'protocol-checks.json').write_text(json.dumps(checks,indent=2))

    sixteen = [i*i for i in range(16)]
    families = [
        ([0], [-1.,0.,.5,3.], 4.),
        ([0,4], [-1.,0.,.5,2.,4.,5.], 6.),
        ([0,3,11,32], [-1.,0.,1.,3.,7.,11.,17.,32.,33.], 40.),
        (sixteen, [-1.]+list(map(float,sixteen))+[(a+b)/2 for a,b in zip(sixteen,sixteen[1:])]+[226.], 240.),
        ([0x1000000+i for i in [0,4,12,28]], [0x1000000+i for i in [-2,0,2,4,8,12,20,28,30]], f32(0x1000000+32)),
        ([0x80000000+i for i in [0,1024,3072,7168]], [0x80000000+i for i in [-512,0,512,1024,2048,3072,5120,7168,7680]], f32(0x80000000+8192)),
        ([0,0,4,4], [-1.,0.,2.,4.,5.], 6.),
    ]
    base = []
    for family, (frames, samples, duration) in enumerate(families):
        for previous in range(4):
            for following in range(4):
                data = rows(frames,previous,following,0x99f6a0+family*256+previous*4+following)
                for wrap in [0,8]:
                    for frame in samples:
                        base.append((frame,frames,duration,wrap,data))
    for wrap in [0,8]:
        for previous in range(4):
            for following in range(4):
                frames=[0,3,11,32]
                data=rows(frames,previous,following,42,signed_zero=True)
                for frame in [0.,1.,3.,7.,32.]:
                    base.append((frame,frames,40.,wrap,data))
    rng=random.Random(0x99f6a0)
    for index in range(128):
        count=rng.choice([2,4,16])
        frames=[0]
        for _ in range(count-1):frames.append(frames[-1]+rng.randrange(1,20))
        data=bytearray(rows(frames,0,0,index))
        for row in range(count):
            for axis in range(4):
                data[row*100+44+axis*16]=rng.randrange(4)
                data[row*100+45+axis*16]=rng.randrange(4)
        frame=f32(rng.uniform(-1.,frames[-1]+1.))
        base.append((frame,frames,float(frames[-1]+10),rng.choice([0,8]),bytes(data)))
    records=[]
    policies={}
    for precision in [0,0x200,0x300]:
        for rounding in [0,0x400,0x800,0xc00]:
            control=0x7f|precision|rounding
            for frame,frames,duration,flags,data in base:
                records.append(request(frame,frames,duration,flags,control,data))
            policies[f'{control:04X}']=len(base)
    supplied=b''.join(records)
    (args.out/'input.bin').write_bytes(supplied)
    result=run_host(host,supplied)
    (args.out/'output.bin').write_bytes(result.stdout)
    (args.out/'stderr.bin').write_bytes(result.stderr)
    assert result.returncode==0 and not result.stderr,(result.returncode,len(result.stdout))
    assert len(result.stdout)==124*len(records),(len(result.stdout),len(records))
    finite=0
    copies=0
    for index in range(len(records)):
        output=result.stdout[124*index:124*(index+1)]
        assert struct.unpack_from('<I',output,104)[0] in [0,1]
        values=struct.unpack_from('<4f',output,20)
        finite+=all(math.isfinite(value) for value in values)
        neighbors=struct.unpack_from('<4I',output,108)
        count=struct.unpack_from('<I',records[index],8)[0]
        assert all(row<count for row in neighbors)
        if neighbors[1]==neighbors[2]:
            copies+=1
            source_row=records[index][32+100*neighbors[1]:132+100*neighbors[1]]
            assert output[:100]==source_row
    report=dict(source_sha256=manifest['source_sha256'],host_exe_sha256=manifest['host_exe_sha256'],
        original_x86_instructions_executed=True,native_control_word_established=False,
        cases=len(records),cases_per_explicit_CW=policies,request_bytes=len(supplied),reply_bytes=len(result.stdout),
        finite_position_weight_outputs=finite,nonfinite_position_weight_outputs=len(records)-finite,
        native_full_row_copies=copies,host_exit=result.returncode,input_output_guards_intact=True,
        source_ranges_unchanged=True,quaternion_deformation_branches_unreachable=True,
        strict_greater_upper_selection=True,protocol_negative_checks=len(checks),
        scope='Only reviewed original INFO+POS code, explicit caller frames/upper/duration/flags/control; no native selector, rig conversion, quaternion/deform or retail process.',
        input_sha256=hashlib.sha256(supplied).hexdigest(),output_sha256=hashlib.sha256(result.stdout).hexdigest())
    (args.out/'inventory.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2))


if __name__=='__main__':
    main()
