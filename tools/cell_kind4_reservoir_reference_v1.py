"""Source-byte-pinned integer oracle for a bounded four-row kind-4 selection.

No original instructions are executed. Four supplied row words are symbolic
runtime fields, not authenticated package IDs or handles. Stable cache,
successful insertion/resolution, no reentrancy and PC53/PC64 masked controls
are explicit admissions. Unsupported misses are rejected before cache mutation
by this test protocol, not as a claim about native failure ordering.

  python -I -S tools/cell_kind4_reservoir_reference_v1.py prepare target/cell-kind4-reservoir-repro
  tools\build.bat run -p spore-core --example cell_kind4_reservoir_compare_v1 -- target/cell-kind4-reservoir-repro/input.bin target/cell-kind4-reservoir-repro/rust.bin
  python -I -S tools/cell_kind4_reservoir_reference_v1.py verify target/cell-kind4-reservoir-repro target/cell-kind4-reservoir-repro/rust.bin
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct
import sys

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from cell_schema_probe import SourceImage, SOURCE_SHA256

EXE=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe')
MULT=0x278DDE6D
DEN=1<<32
SOURCE_SPANS=[(0xE5C136,0xE5C17A,'e4f773ce8842b64673a76722f58fa045869060fd31c0b616b7c081bc13ee54b8'),
              (0x935D40,0x935D76,'35d81765a9410a57d5516506680395d783bffbc0d2ff6cc20e1b74aa1211afce'),
              (0xE65640,0xE656B4,'74a1e35557446c5f88b29b3c3a9fbf37116fea2ed3750b687ab87d9911f4d61c')]

def sha(b): return hashlib.sha256(b).hexdigest()
def file_sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()
def pins():
    return {str(p.relative_to(ROOT)):file_sha(p) for p in [Path(__file__),ROOT/'tools/cell_schema_probe.py',
        ROOT/'crates/spore-core/src/cell_kind4_cache_selection_v1.rs',
        ROOT/'crates/spore-core/src/cell_kind4_list_disk_v1.rs',
        ROOT/'crates/spore-core/src/cell_request_rng_v1.rs',ROOT/'crates/spore-core/src/native_x87.rs',
        ROOT/'crates/spore-core/examples/cell_kind4_reservoir_compare_v1.rs']}
def source_audit(path):
    if file_sha(path)!=SOURCE_SHA256: raise ValueError('original executable hash differs')
    image=SourceImage(path)
    for start,end,expected in SOURCE_SPANS:
        if sha(image.read(start,end-start))!=expected: raise ValueError(f'source span differs {start:08X}')
    if image.read(0x143F52C,4)!=bytes.fromhex('0000802f') or image.read(0x1471B2C,4)!=bytes.fromhex('0000003f'):
        raise ValueError('RNG constant differs')

def evaluate(case):
    seed=case['seed']; control=case['control']; cache=dict(case['cache'])
    outputs=[]
    for request in case['requests']:
        key=request['structure_handle']
        if key in cache:
            outputs.append({'kind':'hit','triple':cache[key], 'state':seed,'draws':[]})
            continue
        # Protocol restriction: reject before mutation; source insertion is
        # earlier than selection and native zero state invokes RDTSC.
        if seed in (0,0x80000000) or control&0x3f!=0x3f or control&0x300 not in (0x200,0x300):
            outputs.append({'kind':'excluded_before_cache','state':seed,'draws':[]})
            continue
        selected=-1; draws=[]
        for i in range(4):
            # Accumulator f32 store and retained x87 -> f64 store are both
            # exactly integer 1..4; no binary64 rounding can differ here.
            seed=(seed*MULT)&0xffffffff
            numerator=seed^0x80000000
            # numerator/2^32 is exact in PC53/64. For i<=4 the product has
            # <=34 significant bits, so x87 rounding and clamping are exact.
            # Source selects only if weight 1 > product; equality is rejected.
            product=numerator*(i+1)
            if product<DEN: selected=i
            draws.append({'state':seed,'numerator':numerator,'total':i+1,
                          'product_numerator':product,'selected_after':selected})
        if selected<0: raise AssertionError('first row must be selected')
        triple=[request['row_words'][selected],0x3D97A8E4,request['global_word']]
        cache[key]=triple
        outputs.append({'kind':'miss','triple':triple,'state':seed,'selected':selected,'draws':draws})
    return {'outputs':outputs}

def requests(key):
    return [{'structure_handle':key,'list_handle':0x10001,'global_word':0x10203040,
             'row_words':[0xA1C46DF2,0xE219E5D0,0xE219E5D3,0xE219E5D2]},
            {'structure_handle':key,'list_handle':0x20002,'global_word':0x55667788,
             'row_words':[1,2,3,4]}]

def prepare(out,source):
    source_audit(source)
    # Choose seed so row four's generated numerator is just below, exactly at,
    # or just above 1/4; the exact equality must not replace row three.
    inverse=pow(pow(MULT,4,1<<32),-1,1<<32)
    seeds=[(((0x40000000+d)^0x80000000)*inverse)&0xffffffff for d in (-1,0,1)]
    assert seeds[1]==0xc0000000
    rng=random.Random(0x4B344341)
    seeds += [rng.randrange(1,0x80000000) for _ in range(64)]
    cases=[]
    for seed in seeds:
        for pc in (0x200,0x300):
            for rc in range(4):
                cases.append({'seed':seed,'control':0x7f|pc|(rc<<10),'cache':[], 'requests':requests(0x99112233)})
    # A hit skips list selection and RNG even when the incoming RNG word is zero.
    cases.append({'seed':0,'control':0x027f,'cache':[(0x99112233,[9,8,7])], 'requests':requests(0x99112233)})
    for seed in (0,0x80000000):
        cases.append({'seed':seed,'control':0x027f,'cache':[], 'requests':requests(0x99112233)[:1]})
    equality=evaluate(cases[8])['outputs'][0]  # middle boundary seed, first control
    assert equality['draws'][3]['product_numerator']==DEN and equality['selected']==2
    request=('\n'.join(json.dumps(c,separators=(',',':'),sort_keys=True) for c in cases)+'\n').encode()
    expected=('\n'.join(json.dumps(evaluate(c),separators=(',',':'),sort_keys=True) for c in cases)+'\n').encode()
    # Adapter protocol compares only fields the production API exposes. The
    # independent JSONL traces retain per-row decisions as reference evidence;
    # they are not falsely presented as observed Rust intermediate values.
    adapter_input=bytearray(b'K4IN0001'+struct.pack('<I',len(cases)))
    adapter_expected=bytearray(b'K4OUT001'+struct.pack('<I',len(cases)))
    for case in cases:
        if len(case['cache'])>1 or not 1<=len(case['requests'])<=2: raise ValueError('adapter protocol bounds')
        cache=case['cache'][0] if case['cache'] else (0,[0,0,0])
        fields=[case['seed'],case['control'],len(case['cache']),cache[0],*cache[1],len(case['requests'])]
        for req in case['requests']:
            fields += [req['structure_handle'],req['list_handle'],req['global_word'],*req['row_words']]
        fields += [0]*(22-len(fields))
        if len(fields)!=22: raise ValueError('adapter framing')
        adapter_input.extend(struct.pack('<22I',*fields))
        outputs=evaluate(case)['outputs']
        adapter_expected.extend(struct.pack('<I',len(outputs)))
        for result in outputs:
            kind={'hit':0,'miss':1,'excluded_before_cache':2}[result['kind']]
            triple=result.get('triple',[0,0,0]); selected=result.get('selected',0xffffffff)
            adapter_expected.extend(struct.pack('<7I',kind,*triple,result['state'],selected,len(result['draws'])))
    out.resolve().relative_to((ROOT/'target').resolve());out.mkdir(parents=True,exist_ok=False)
    (out/'cases.jsonl').write_bytes(request);(out/'expected.jsonl').write_bytes(expected)
    (out/'input.bin').write_bytes(adapter_input);(out/'expected.bin').write_bytes(adapter_expected)
    report={'schema':'kind4-reservoir-integer-reference-v1','cases':len(cases),
            'source_sha256':SOURCE_SHA256,'source_spans':[{'start':f'{a:08X}','end_exclusive':f'{b:08X}','sha256':h} for a,b,h in SOURCE_SPANS],
            'input_sha256':sha(request),'expected_sha256':sha(expected),'pins':pins(),
            'adapter_input_sha256':sha(adapter_input),'adapter_expected_sha256':sha(adapter_expected),
            'equality_witness_case':8,'seed_zero_miss_excluded':True,'zero_numerator_miss_excluded':True,
            'scope':'four caller-supplied type-0 rows, exact weight 1; stable cache and successful acquisition; synthetic handle words',
            'source_execution':False,'runtime_cache_or_actor_observed':False}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

def verify(directory,reply):
    m=json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    if m['pins']!=pins(): raise ValueError('source pins changed')
    request=(directory/'input.bin').read_bytes(); expected=(directory/'expected.bin').read_bytes(); actual=reply.read_bytes()
    if sha(request)!=m['adapter_input_sha256'] or sha(expected)!=m['adapter_expected_sha256']: raise ValueError('fixture hash mismatch')
    if actual!=expected: raise ValueError(f'mismatch or truncation expected={sha(expected)} actual={sha(actual)}')
    print(json.dumps({'cases':m['cases'],'reply_sha256':sha(actual),'reply_provenance_authenticated':False,'source_execution':False}))

def main():
    p=argparse.ArgumentParser(description=__doc__); s=p.add_subparsers(dest='action',required=True)
    a=s.add_parser('prepare');a.add_argument('output',type=Path);a.add_argument('--source-exe',type=Path,default=EXE)
    a=s.add_parser('verify');a.add_argument('directory',type=Path);a.add_argument('reply',type=Path)
    a=p.parse_args()
    if a.action=='prepare':prepare(a.output,a.source_exe)
    else:verify(a.directory,a.reply)
if __name__=='__main__':main()
