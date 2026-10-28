"""Prepare actual paired native POS/ROT values with explicit reference contexts.

All curve values are accepted original-host replies. Reference/rest contexts
are named fixtures, not inferred source creatures or actual actors. Never runs
original code, a compiler, Cargo or GPU work.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

ROOT=Path(__file__).resolve().parent.parent
CURVES=ROOT/'target/animation/root-pose-curve-inputs-v2'
NATIVE=ROOT/'target/animation/root-pose-curve-native-v1'
RETENTION=ROOT/'target/animation/root-whole-builder-followup-v1/first-root-static-retention-v3.json'
WHOLE=ROOT/'target/animation/root-whole-builder-followup-v1/all818-whole-channel-prepared-v2.json'
HOST=ROOT/'target/animation/root-pose-conversion-host-v4-review'
ROT_SHA='8b53cdb96631791556116957ce02ee299cedde0092c1144c0084dc9b7eb8a98f'
CONTEXTS=[
 dict(name='explicit_unit_reference',scale=1.,dimensions=[1.,1.,1.],size374=1.,rootmask=0,size434=1.,
      rest_position=[0.,0.,0.],rest_rotation=[0.,0.,0.,1.],reference_position=[0.,0.,0.],
      reference_rotation=[0.,0.,0.,1.],blend=1.),
 dict(name='explicit_rotated_translated_reference',scale=1.25,dimensions=[2.,3.,4.],size374=2.,rootmask=8,size434=1.5,
      rest_position=[2.,-3.,5.],rest_rotation=[0.,.6,0.,.8],reference_position=[1.,2.,3.],
      reference_rotation=[.3,-.4,.5,.70710677],blend=.5),
 dict(name='explicit_zero_size_reference',scale=.5,dimensions=[.5,1.5,2.5],size374=0.,rootmask=0,size434=0.,
      rest_position=[-2.,1.,.5],rest_rotation=[.6,0.,0.,.8],reference_position=[-.5,.25,1.],
      reference_rotation=[0.,0.,.6,.8],blend=.25),
]


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def bits(value):return struct.unpack('<I',struct.pack('<f',value))[0]


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    a.out.resolve().relative_to((ROOT/'target').resolve())
    curve_manifest=json.loads((CURVES/'manifest.json').read_text())
    comparison=json.loads((NATIVE/'rust-comparison-v1.json').read_text())
    assert comparison['rows']==comparison['compared_rows']==3702
    assert comparison['exact_for_compared_rows'] and not comparison['problems']
    assert comparison['explicit_arithmetic_exclusions']==0 and not comparison['excluded_rows']
    assert sha(NATIVE/'corpus-attempt01-output.bin')==ROT_SHA==comparison['native_output_sha256']
    assert sha(CURVES/'inputs.bin')==comparison['input_sha256']==curve_manifest['files']['inputs.bin']
    for name,expected in curve_manifest['files'].items():assert sha(CURVES/name)==expected
    whole={tuple(r['key']):r for r in json.loads(WHOLE.read_text())}
    retained={tuple(r['key']):r for r in json.loads(RETENTION.read_text())['candidates']}
    index=json.loads((CURVES/'input-index.json').read_text());assert len(index)==3702
    a.out.mkdir(parents=True,exist_ok=False)
    metadata=[]
    with (CURVES/'accepted-pos-replies.bin').open('rb') as positions, \
         (NATIVE/'corpus-attempt01-output.bin').open('rb') as rotations, (a.out/'inputs.bin').open('xb') as out:
        for row,item in enumerate(index):
            pos,rot=positions.read(124),rotations.read(240)
            assert len(pos)==124 and len(rot)==240
            pbits=struct.unpack_from('<4I',pos,20);qbits=struct.unpack_from('<5I',rot,100)
            assert all(x&0x7f800000!=0x7f800000 for x in pbits+qbits)
            key=tuple(item['key']);channel=whole[key]['channels'][0]
            assert struct.unpack_from('<H',rot,232)[0]==0x027f
            assert struct.unpack_from('<I',rot,228)[0]&~0x3f==0x1f80
            position=next(d for d in channel['descriptors'] if d[0]&15==1 and d[1])
            assert position[0] in(0x31,0x71,0xb1)
            for context in CONTEXTS:
                for source_record in retained[key]['conditional_records']:
                    packed=int(source_record['record_words'][3],16)
                    mirror=(packed>>25)&1
                    mask=int(source_record['record_words'][2],16)&0xffffff
                    words=[0x027f,position[0],channel['raw_metadata_words_88_d0'][0],mirror,
                        bits(context['blend']),bits(context['scale']),*[bits(v) for v in context['dimensions']],
                        bits(context['size374']),context['rootmask'],bits(context['size434']),
                        *[bits(v) for v in context['rest_position']],mask,
                        *[bits(v) for v in context['rest_rotation']],*[bits(v) for v in context['reference_position']],
                        *[bits(v) for v in context['reference_rotation']],*pbits[:3],pbits[3],*qbits[:4],qbits[4]]
                    assert len(words)==36 and mask&6==6
                    request=struct.pack('<36I',*words);out.write(request)
                    metadata.append(dict(row=len(metadata),curve_row=row,key=list(key),source_sha256=item['source_sha256'],
                        channel=0,frame_bits=item['frame_bits'],source_pos_case=item['source_pos_case'],
                        pass_index=source_record['pass_index'],binding_words=source_record['record_words'],
                        context=context['name'],effective_mirror=mirror,whole_binding_mask=f'{mask:08X}',
                        position_flags=f'{position[0]:08X}',input_sha256=hashlib.sha256(request).hexdigest()))
        assert positions.read(1)==rotations.read(1)==b''
    (a.out/'input-index.json').write_text(json.dumps(metadata,indent=2)+'\n')
    (a.out/'reference-contexts.json').write_text(json.dumps(CONTEXTS,indent=2)+'\n')
    pins=[CURVES/'manifest.json',CURVES/'inputs.bin',CURVES/'accepted-pos-replies.bin',CURVES/'input-index.json',
          NATIVE/'corpus-attempt01-output.bin',NATIVE/'rust-comparison-v1.json',WHOLE,RETENTION,
          HOST/'manifest.json',HOST/'host-x86.asm',Path(__file__)]
    report=dict(schema='paired-source-root-conversion-accumulation-inputs-v1',native_executed=False,
        rows=len(metadata),source_curve_pairs=3702,source_clips=215,contexts=len(CONTEXTS),source_binding_passes=2,
        input_bytes=144,output_bytes=176,expected_native_output_bytes=len(metadata)*176,
        source_curve_proof='EveryPOSvalue/weight andROTquaternion/weight comesfromacceptedoriginalnativecurve repliesatthesamecanonicalsourcechannel/frame. Noauthoredendpointsubstitute.',
        rest_reference_boundary='Threeexplicitnamedfinitefixturecontexts. TheseareNOTsourcecreatures,bodyassociation,actorrouteorcompleteanimationstate. Theyexerciseconditionalcoordinateconversiononly.',
        binding_proof='Wholemaskandmirrorfromstaticretainedfirstrootrecord,includingAC20mirror-suppression. Bothsourcepassrecordsretained,notassumedmirror0/1.',
        positive_blend_policy=True,
        files={name:sha(a.out/name) for name in('inputs.bin','input-index.json','reference-contexts.json')},
        pins={str(path.relative_to(ROOT)):sha(path) for path in pins},
        next_gate='Rootpeerreview/sourceclosure,compile/linkaudit,protocolpositive/negativechecks,originalnativeexecution,independentRust/referencecomparison. Stagedevaluation/childconstraints/finalposearenotinvoked.')
    (a.out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in('schema','rows','source_curve_pairs','source_clips','contexts','native_executed')},indent=2))


if __name__=='__main__':main()
