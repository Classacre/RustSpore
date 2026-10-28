"""Prepare bounded arithmetic-only looping requests; no native execution."""
import argparse
from collections import Counter
import json
from pathlib import Path
import struct
from cell_runtime_curves_fixture import CONTEXTS,bits,sha_file
import cell_runtime_loop_clock_native_probe as host
ROOT=Path(__file__).resolve().parent.parent
def double(x):return struct.unpack('<Q',struct.pack('<d',x))[0]
def request(time=0x3FA00000,duration=double(1.),frames=0x43050000,cw=0x027F,loop=1,boundary=1,crt=1):
    raw=struct.pack('<IHHIQQIIII',0,cw,0,time,0,duration,frames,loop,boundary,crt)+bytes(20)
    assert len(raw)==64;return raw
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--host-dir',type=Path,required=True)
    p.add_argument('--source-census',type=Path,default=ROOT/'target/source-cell/runtime-curves-source-census-v1');args=p.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve());manifest=json.loads((args.host_dir/'manifest.json').read_text())
    assert manifest['host_source_sha256']==sha_file(args.host_dir/'host-x86.asm')and manifest['preparation_tool_sha256']==sha_file(Path(host.__file__))
    pairs=[(x,double(1.))for x in[0,0x80000000,1,0x00800000,bits(.125),bits(.5),0x3F7FFFFF,0x3F800000,0x3F800001,
        bits(1.25),bits(1.5),0x3FFFFFFF,bits(2.),bits(2.5),bits(7.75),bits(2.**22+.5),bits(2.**23-.5),bits(2.**23),bits(2.**24-1),bits(2.**52),0x7F7FFFFF]]
    pairs += [(bits(1.),0x3FF0000000000001),(bits(1.),0x3FEFFFFFFFFFFFFF),(bits(7.75),double(3.)),(1,double(2.**1023)),(0,1)]
    frames=[bits(1.),0x3F800001,bits(69.),bits(133.),bits(2.**24),0x7F7FFFFF]
    args.out.mkdir(parents=True,exist_ok=False);cases=0;families=Counter();contexts=Counter()
    with(args.out/'input.bin').open('xb')as inputs,(args.out/'cases.tsv').open('x')as table:
        table.write('case\tfamily\tcw\ttime_bits\tduration_bits\tframes_bits\tsource_sha256\n')
        def emit(time,duration,frame,cw,family,source='-'):
            nonlocal cases
            inputs.write(request(time,duration,frame,cw));table.write(f'{cases}\t{family}\t{cw:04X}\t{time:08X}\t{duration:016X}\t{frame:08X}\t{source}\n')
            cases+=1;families[family]+=1;contexts[f'{cw:04X}']+=1
        for cw in CONTEXTS:
            for frame in frames:
                for time,duration in pairs:emit(time,duration,frame,cw,'retained_quotient_commit_and_modf_branches')
        selected={}
        for line in(args.source_census/'components.jsonl').read_text().splitlines():
            item=json.loads(line);frame=int(item['source_duration_bits'],16)
            if 0x3F800000<=frame<0x7F800000:selected.setdefault(frame,item['source_sha256'])
            if len(selected)==8:break
        for cw in CONTEXTS:
            for frame,source in selected.items():
                for time in[bits(.25),bits(1.25),bits(2.5),bits(7.75)]:emit(time,double(1.),frame,cw,'source_frame_word_explicit_loop_arithmetic',source)
    rejects=[('negative_time',request(time=bits(-.25))),('time_nan',request(time=0x7FC12345)),('duration_zero',request(duration=0)),
        ('duration_negative',request(duration=double(-1.))),('duration_nan',request(duration=0x7FF8000000000000)),
        ('frames_zero',request(frames=0)),('frames_infinite',request(frames=0x7F800000)),
        ('missing_loop',request(loop=0)),('missing_boundary',request(boundary=0)),('missing_crt_dispatch',request(crt=0)),
        ('reserved_precision',request(cw=0x017F)),('nonfinite_quotient',request(time=0x7F7FFFFF,duration=1))]
    protocol=[];(args.out/'protocol').mkdir()
    for name,raw in rejects:
        path=args.out/'protocol'/f'{name}.bin';path.write_bytes(raw);protocol.append(dict(name=name,path=f'protocol/{name}.bin',expected_exit=7,expected_output_bytes=0))
    report=dict(schema='arithmetic-only-loop-clock-inputs-v1',cases=cases,families=dict(families),contexts=dict(contexts),protocols=protocol,
        host_manifest_sha256=sha_file(args.host_dir/'manifest.json'),tool_sha256=sha_file(Path(__file__)),
        source_census_sha256=sha_file(args.source_census/'components.jsonl'),
        hashes={str(p.relative_to(args.out)):sha_file(p)for p in args.out.rglob('*')if p.is_file()},native_executed=False,
        scope='Arithmetic snapshots only, not period counters/events or actual clip loop state. All cases explicitly select finite SSE2modf and maskedCW/MXCSR1F80. Near-integer quotients distinguish retained80 from committed helper32; source resource frame words do not establish a live duration setter.')
    (args.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(cases=cases,protocols=len(protocol),families=dict(families)),indent=2))
if __name__=='__main__':main()
