"""Prepare bounded first-duration clock requests; never execute original code."""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
import struct
import cell_runtime_clock_native_probe as host
from cell_runtime_curves_fixture import CONTEXTS,bits,sha_file

ROOT=Path(__file__).resolve().parent.parent
def double(value):return struct.unpack('<Q',struct.pack('<d',value))[0]
def request(mode=0,cw=0x027F,time=0x3F000000,previous=0,duration=double(1.),frames=0x43050000,loop=0):
    raw=struct.pack('<IHHIQQII',mode,cw,0,time,previous,duration,frames,loop)+bytes(28)
    assert len(raw)==64
    return raw

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host-dir',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--source-census',type=Path,default=ROOT/'target/source-cell/runtime-curves-source-census-v1');args=p.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve());m=json.loads((args.host_dir/'manifest.json').read_text())
    assert sha_file(args.host_dir/'host-x86.asm')==m['host_source_sha256']
    assert sha_file(Path(host.__file__))==m['preparation_tool_sha256']
    args.out.mkdir(parents=True,exist_ok=False);cases=0;families=Counter();contexts=Counter()
    frames=[0x3F800000,0x3F800001,0x40000000,0x43050000,0x4B800000,0x7F7FFFFF]
    absolute=[(0,double(1.)),(0x80000000,double(1.)),(1,double(1.)),(0x00800000,double(1.)),
        (0x3E000000,double(1.)),(0x3F000000,double(1.)),(0x3F7FFFFF,double(1.)),
        (0x3F800000,0x3FF0000000000001),(0,double(2.**-1022)),(0,1)]
    advances=[(double(0.),0),(double(0.),0x80000000),(double(.5),0xBE800000),
        (double(1.),0xBF800000),(0x3FF0000000000001,0),(double(2.**-150),0),
        (double(0.),1),(double(.5),0x3E800000),(double(1.),0xBF7FFFFF)]
    with (args.out/'input.bin').open('xb')as binary,(args.out/'cases.tsv').open('x',newline='')as table:
        writer=csv.writer(table,delimiter='\t',lineterminator='\n');writer.writerow(['case','family','cw','mode','time_or_delta_bits','previous_time_bits','duration_bits','frames_bits','source_sha256'])
        def emit(raw,family,source='-'):
            nonlocal cases
            mode,cw,_,time,previous,duration,frame,_=struct.unpack_from('<IHHIQQII',raw)
            binary.write(raw);writer.writerow([cases,family,f'{cw:04X}',mode,f'{time:08X}',f'{previous:016X}',f'{duration:016X}',f'{frame:08X}',source])
            cases+=1;families[family]+=1;contexts[f'{cw:04X}']+=1
        for cw in CONTEXTS:
            for frame in frames:
                for time,duration in absolute:emit(request(cw=cw,time=time,duration=duration,frames=frame),'absolute_finite_edges')
                for previous,delta in advances:emit(request(mode=1,cw=cw,time=delta,previous=previous,duration=double(2.),frames=frame),'delta_fadd64_then_argument32')
        # Literal source resource frame-count words, not asserted runtime durations.
        selected={}
        with (args.source_census/'components.jsonl').open()as source:
            for line in source:
                row=json.loads(line);value=int(row['source_duration_bits'],16)
                if 0x3F800000<=value<0x7F800000:selected.setdefault(value,row['source_sha256'])
                if len(selected)==8:break
        for cw in CONTEXTS:
            for frame,source in selected.items():
                emit(request(cw=cw,frames=frame),'source_resource_frame_word_explicit_time',source)
                emit(request(mode=1,cw=cw,time=0x3E800000,previous=double(.25),frames=frame),'source_resource_frame_word_explicit_delta',source)
    rejects=[('mode',request(mode=2)),('reserved_cw',request(cw=0x017F)),('loop',request(loop=1)),
        ('time_nan',request(time=0x7FC12345)),('duration_zero',request(duration=0)),
        ('duration_negative',request(duration=double(-1.))),('duration_infinity',request(duration=0x7FF0000000000000)),
        ('frames_zero',request(frames=0)),('frames_infinity',request(frames=0x7F800000)),
        ('negative_time',request(time=0xBF000000)),('crossing_equal',request(time=0x3F800000)),
        ('crossing_greater',request(time=0x40000000)),('delta_negative_result',request(mode=1,time=0xBF800000)),
        ('delta_nonfinite_prior',request(mode=1,previous=0x7FF8000000000000)),
        ('delta_nonfinite_commit',request(mode=1,previous=double(1e300)))]
    protocols=[]
    (args.out/'protocol').mkdir()
    for name,raw in rejects:
        path=args.out/'protocol'/f'{name}.bin';path.write_bytes(raw);protocols.append(dict(name=name,path=f'protocol/{name}.bin',expected_exit=7,expected_output_bytes=0))
    (args.out/'protocol'/ 'partial.bin').write_bytes(request()[:-1]);protocols.append(dict(name='partial',path='protocol/partial.bin',expected_exit=7,expected_output_bytes=0))
    report=dict(schema='finite-first-duration-clock-inputs-v1',cases=cases,families=dict(families),contexts=dict(contexts),protocols=protocols,
        input_bytes=cases*64,expected_output_bytes=cases*28,source_frame_word_variants=len(selected),
        host_manifest_sha256=sha_file(args.host_dir/'manifest.json'),tool_sha256=sha_file(Path(__file__)),
        source_census_report_sha256=sha_file(args.source_census/'report.json'),source_components_sha256=sha_file(args.source_census/'components.jsonl'),
        hashes={str(path.relative_to(args.out)):sha_file(path)for path in args.out.rglob('*')if path.is_file()},
        native_executed=False,scope='Twelve explicit x87 policies and MXCSR1F80. First duration/nonloop only. Source resource frame-count words are retained with explicitly supplied time/duration; no duration setter, looping, event/counter or live clip claim. Rejects are separate positive protocol cases.')
    (args.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['schema','cases','families','source_frame_word_variants']},indent=2))
if __name__=='__main__':main()
