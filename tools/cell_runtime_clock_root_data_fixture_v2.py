"""Immutable stage directories for a fresh native clock/root-data composition.

Default prepares clocks. Each later --input-dir is copied into a new --out;
the previous packet and all actual native replies remain read-only.
"""
import argparse
from fractions import Fraction
import json
import math
from pathlib import Path
import struct
import cell_runtime_clock_root_data_fixture as base
from cell_runtime_curves_fixture import sha_file
from cell_runtime_clock_native_run import inspect as inspect_clock

ROOT=Path(__file__).resolve().parent.parent
BASE_TOOL_SHA='0d2d7ff41d5e7b8d65e47b88769568acc06d29fa7cde9468bb66bb1ea890149e'
CORE_PATHS=[
 'crates/spore-core/src/cell_runtime_clock_root_data.rs',
 'crates/spore-core/examples/cell_runtime_clock_root_data_verify.rs.pending',
 'crates/spore-core/src/cell_runtime_clock.rs','crates/spore-core/examples/cell_runtime_clock_verify.rs',
 'crates/spore-core/src/cell_runtime_root_data.rs','crates/spore-core/examples/cell_runtime_root_data_verify.rs',
 'crates/spore-core/src/cell_runtime_curves.rs','crates/spore-core/src/cell_runtime_contributions.rs',
 'crates/spore-core/src/cell_runtime_data.rs','crates/spore-core/src/cell_runtime_modifiers.rs',
 'crates/spore-core/src/native_sse.rs','crates/spore-core/src/native_x87.rs','crates/spore-core/src/anim.rs',
 'target/source-cell/runtime-clock-promotion-v1/cell_runtime_clock.rs',
 'target/source-cell/runtime-clock-promotion-v1/cell_runtime_clock_verify.rs.pending',
 'target/source-cell/runtime-root-data-promotion-review-v1/cell_runtime_root_data.rs',
 'target/source-cell/runtime-root-data-promotion-review-v1/cell_runtime_root_data_verify.rs.pending',
 'target/source-cell/runtime-root-data-native-run-v1/rust-comparison-v1.json',
 'tools/cell_runtime_clock_native_run.py',
]
def save(path,value):
    with path.open('x')as output:json.dump(value,output,indent=2);output.write('\n')
def domain(root):
    cases=json.loads((root/'cases.json').read_text());raw=(root/'clock-input.bin').read_bytes()
    assert len(cases)==40 and len(raw)==40*64
    groups=set();modes={0:0,1:0}
    for case in cases:
        ordinal=case['case'];record=raw[64*ordinal:64*(ordinal+1)]
        mode,cw,reserved,time,previous,duration,frames,loop=struct.unpack_from('<IHHIQQII',record)
        assert mode in[0,1]and cw==0x027F and reserved==loop==0 and record[36:]==bytes(28)
        assert duration==0x3FF0000000000000
        f32=lambda x:struct.unpack('<f',struct.pack('<I',x))[0]
        f64=lambda x:struct.unpack('<d',struct.pack('<Q',x))[0]
        assert math.isfinite(f32(time))and math.isfinite(f64(previous))and math.isfinite(f32(frames))and f32(frames)>=1
        exact=Fraction.from_float(f32(time))+(Fraction.from_float(f64(previous))if mode else 0)
        assert exact==Fraction(case['quarter'],4)and exact in[0,Fraction(1,4),Fraction(1,2),Fraction(3,4)]
        assert 0<=exact<1 and mode==case['clock_mode']
        payload=(root/case['clip']).read_bytes();assert struct.unpack_from('<I',payload,288)[0]==frames
        assert frames==int(case['resource_frames_bits'],16)
        groups.add((case['clip'],case['prepared'],case['channel']));modes[mode]+=1
    assert len(groups)==5 and modes=={0:20,1:20}
    return dict(cases=40,source_groups=5,absolute_cases=20,delta_cases=20,
        literal_frame_words_match_sources=True,strict_first_duration_domain_verified=True,
        rational_time_values_exact=True,frame_output_predicted=False,context='CW027F/MXCSR1F80')
def check_native_refs(root):
    refs={}
    for name,field,hashfield in [('curve-preparation.json','clock_output','clock_output_sha256'),
        ('contribution-preparation.json','fresh_curve_output','fresh_curve_output_sha256'),
        ('consumer-preparation.json','contribution_output','contribution_output_sha256')]:
        if(root/name).exists():
            report=json.loads((root/name).read_text());path=Path(report[field]);assert sha_file(path)==report[hashfield];refs[str(path)]=report[hashfield]
    return refs
def verify_stage(root,stage):
    report=json.loads((root/f'{stage}-stage-v2.json').read_text())
    for name,expected in report['files'].items():assert sha_file(root/name)==expected,name
    for name,expected in report['core_and_promotion_pins'].items():assert sha_file(ROOT/name)==expected,name
    assert report['wrapper_sha256']==sha_file(Path(__file__))
    domain(root);check_native_refs(root)
    return report
def clone_stage(source,destination):
    assert not destination.exists()
    assert not destination.resolve().is_relative_to(source.resolve())
    files=[p for p in source.rglob('*')if p.is_file()]
    assert not any(p.is_symlink()for p in source.rglob('*'))
    destination.mkdir(parents=True)
    for path in files:
        dest=destination/path.relative_to(source);dest.parent.mkdir(parents=True,exist_ok=True)
        with dest.open('xb')as output:output.write(path.read_bytes())
def finish(args,stage,parent):
    report=dict(schema='immutable-clock-root-data-stage-v2',stage=stage,
        domain=domain(args.out),core_and_promotion_pins={name:sha_file(ROOT/name)for name in CORE_PATHS},
        wrapper_sha256=sha_file(Path(__file__)),base_tool_sha256=BASE_TOOL_SHA,
        parent_stage=None if parent is None else str(args.input_dir.resolve()),
        parent_manifest_sha256=None if parent is None else sha_file(args.input_dir/f'{parent}-stage-v2.json'),
        actual_native_output_pins=check_native_refs(args.out),
        files={str(p.relative_to(args.out)):sha_file(p)for p in args.out.rglob('*')if p.is_file()},
        native_executed_by_tool=False,scope='New immutable stage only. Actual native outputs are read and pinned, never generated or edited by this tool. Explicit positive blend1, root binding, duration1 and supplied actor route; no live resource/registry admission.')
    save(args.out/f'{stage}-stage-v2.json',report)
    print(json.dumps(dict(stage=stage,new_directory=str(args.out),parent=report['parent_stage'],cases=40,native_executed=False)))
def main():
    assert sha_file(Path(base.__file__))==BASE_TOOL_SHA
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--input-dir',type=Path)
    p.add_argument('--base',type=Path,default=ROOT/'target/source-cell/runtime-root-data-join-v3')
    p.add_argument('--clock-acceptance',type=Path,default=ROOT/'target/source-cell/runtime-clock-native-run-v2/rust-comparison-v1.json')
    phases=p.add_mutually_exclusive_group();phases.add_argument('--clock-output',type=Path);phases.add_argument('--curve-output',type=Path);phases.add_argument('--contribution-output',type=Path)
    args=p.parse_args();args.out.resolve().relative_to((ROOT/'target').resolve())
    if args.clock_output:stage,parent='curve','clock'
    elif args.curve_output:stage,parent='contribution','curve'
    elif args.contribution_output:stage,parent='consumer','contribution'
    else:stage,parent='clock',None
    if parent is None:
        assert args.input_dir is None;base.prepare_clock(args)
    else:
        assert args.input_dir is not None;verify_stage(args.input_dir,parent)
        clone_stage(args.input_dir,args.out)
        if stage=='curve':
            inspect_clock(args.out/'clock-input.bin',args.clock_output)
            base.prepare_curves(args)
        elif stage=='contribution':base.prepare_contributions(args)
        else:base.prepare_consumer(args)
    finish(args,stage,parent)
if __name__=='__main__':main()
