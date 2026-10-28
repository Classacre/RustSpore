"""Admit only the separately reviewed v2 owned-stack-exit clock host.

Default mode is preflight only. The unchanged v1 wire inspector/executor is
hash-pinned and reused with v2 host identities; the old host is never executed.
The retained v1 access fault is evidence, not a retryable allocation failure.
"""
import argparse
import json
from pathlib import Path
import struct

import cell_runtime_clock_native_run as wire
import cell_runtime_clock_native_probe_v2 as probe
from cell_runtime_data_native_run import digest
from cell_schema_probe import SourceImage,SOURCE_SHA256

ROOT=Path(__file__).resolve().parent.parent
HOST='1b50b88351a7b5eca1edc236decf8b0ddc773d5ea30451feed93ef09ec7503e9'
ASM='a53f6b06a98048484f2bd72ceb4b4f187fd099ac968d8236ca15c559e963c083'
OWNED='9d9262863837335b82c80d74577ccea968862dcb35acb9d1a28999db01a3fbb5'
EXPECTED=dict(wire.EXPECTED)
EXPECTED.update({
 'tools/cell_runtime_clock_native_run.py':'811f8b9bf438ddb73e7df4b4020c7651e867c275a9d32dc4b52617892e8fa468',
 'tools/cell_runtime_clock_native_probe_v2.py':'3b75609ce2428bd34f8ef7b7691d416bd4d11657f9fd3da86feb3d1a8af782a0',
 'target/source-cell/native-runtime-clock-host-v2/host-x86.asm':ASM,
 'target/source-cell/native-runtime-clock-host-v2/manifest.json':'9f0d221cfd5bb1c2ed463e9af5473cdebb10a739f172bf945a6f20f59c2f0291',
 'target/source-cell/native-runtime-clock-host-root-reproduction-v2/manifest.json':'9f0d221cfd5bb1c2ed463e9af5473cdebb10a739f172bf945a6f20f59c2f0291',
 'target/source-cell/native-runtime-clock-host-root-reproduction-v2/linked-audit.json':'612728be67c33622eecf907acde2fd11441fad4c91401ca565e81c1f11e6e4e8',
 'target/source-cell/runtime-clock-v2-independent-font-review-v1.json':'f3f64f46bce9ab81e373646f3b07a4ac33282b5b53d934568517735a7807ba86',
 'target/source-cell/runtime-clock-v2-independent-linked-font-review-v1.json':'3b44478a47dcd272b9dcff77ff83fc2b05903a5a6079f1d691af1fd354de4a4b',
 'target/source-cell/runtime-clock-native-run-v1/failure.json':'034fd22247a5a203492d568d8e6c5c181bbbb573a01a5c718a1fdd510de711d7',
 'target/source-cell/runtime-clock-native-run-v1/native-v1/protocol/crossing_equal-attempt01-status.json':'4def4c7b0ba75060d9b31943663bd9c18e00acb035a1fedc009d26ad8ce5de06',
 'target/source-cell/runtime-clock-native-run-v1/native-v1/protocol/crossing_equal-attempts.json':'a09c1d2578654b203bb09bf3cef45f25bcb56da07f58abf5816b2ef868f0f0e6',
})

def preflight(args):
    for name,expected in EXPECTED.items():assert digest(ROOT/name)==expected,name
    original=ROOT/'target/source-cell/native-runtime-clock-host-v1'
    packet=ROOT/'target/source-cell/native-runtime-clock-host-v2'
    old=json.loads((original/'manifest.json').read_text())
    manifest=json.loads((args.host_dir/'manifest.json').read_text())
    linked=json.loads((args.host_dir/'linked-audit.json').read_text())
    prepared=json.loads((args.input_dir/'report.json').read_text())
    review=json.loads((args.input_dir/'static-preparation-review.json').read_text())
    assert manifest==json.loads((packet/'manifest.json').read_text())
    assert manifest['schema']=='bounded-original-cell-runtime-clock-host-v2'
    assert manifest['source_sha256']==SOURCE_SHA256 and manifest['host_source_sha256']==ASM
    assert manifest['original_spans_unchanged']and manifest['original_source_bytes_unchanged']==146
    assert manifest['base_generation_tool_sha256']==probe.BASE_SHA
    assert manifest['preparation_tool_sha256']==digest(ROOT/'tools/cell_runtime_clock_native_probe_v2.py')
    for name in ['code_allowlist','source_bytes','constant','protocol','exits','pages','compile_commands']:
        assert manifest[name]==old[name],name
    assert digest(args.host_dir/'host-x86.exe')==HOST and digest(args.host_dir/'host-x86.asm')==ASM
    assert linked['host_exe_sha256']==HOST and linked['owned_code_sha256']==OWNED
    assert linked['source_bytes_exact']and linked['sections_and_guards_checked']
    # Inputs and the original preparation review remain bound to v1's unchanged
    # source/domain packet; they are deliberately not relabeled as v2 manifests.
    for name,expected in review['pins'].items():assert digest(ROOT/name)==expected,name
    assert digest(original/'manifest.json')==prepared['host_manifest_sha256']==review['host_manifest_sha256']
    assert review['source_input_domain_bounds_verified']and not review['arithmetic_oracle']
    for name,expected in prepared['hashes'].items():assert digest(args.input_dir/name)==expected,name
    assert digest(args.source_census/'report.json')==prepared['source_census_report_sha256']
    assert digest(args.source_census/'components.jsonl')==prepared['source_components_sha256']
    static=json.loads((ROOT/'target/source-cell/runtime-clock-v2-independent-font-review-v1.json').read_text())
    peer=json.loads((ROOT/'target/source-cell/runtime-clock-v2-independent-linked-font-review-v1.json').read_text())
    assert static['review_result'].startswith('STATIC PASS')and peer['review_result'].startswith('STATIC PASS')
    assert static['host_assembly_sha256']==ASM and static['original_spans_constants_domain_protocol_unchanged']
    assert peer['source_sha256']==SOURCE_SHA256 and peer['source_bytes']==146
    assert peer['native_host_sha256']==HOST and peer['host_asm_sha256']==ASM
    assert peer['linked_audit_sha256']==digest(args.host_dir/'linked-audit.json')
    fault=json.loads((ROOT/'target/source-cell/runtime-clock-native-run-v1/native-v1/protocol/crossing_equal-attempt01-status.json').read_text())
    assert fault['exit']==3221225477 and fault['output_bytes']==0
    image=SourceImage(args.source_exe)
    assert probe.base.audit_source(image)==manifest['code_allowlist']
    assert probe.host_source(image)==(args.host_dir/'host-x86.asm').read_text()
    assert probe.base.linked_audit(args.host_dir/'host-x86.exe',args.host_dir/'host-x86.map',image)==linked
    pe=probe.base.PE(args.host_dir/'host-x86.exe');at=struct.unpack_from('<I',pe.data,0x3C)[0]
    entry=pe.base+struct.unpack_from('<I',pe.data,at+24+16)[0]
    assert entry==int(linked['symbols']['HostMain'],16)
    closure=wire.request_closure(args.input_dir/'input.bin',args.input_dir/'cases.tsv')
    assert closure['families']==prepared['families']and closure['contexts']==prepared['contexts']
    assert prepared['cases']==1560 and len(prepared['protocols'])==16
    for item in prepared['protocols']:
        assert item['expected_exit']==7 and item['expected_output_bytes']==0
        assert(args.input_dir/item['path']).stat().st_size in[63,64]
    return prepared,review,closure,entry

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir',type=Path,default=ROOT/'target/source-cell/native-runtime-clock-host-root-reproduction-v2')
    p.add_argument('--input-dir',type=Path,default=ROOT/'target/source-cell/runtime-clock-inputs-v1')
    p.add_argument('--source-census',type=Path,default=ROOT/'target/source-cell/runtime-curves-source-census-v1')
    p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'))
    p.add_argument('--out',type=Path,required=True);p.add_argument('--run',action='store_true');args=p.parse_args()
    args.out.resolve().relative_to((ROOT/'target').resolve());prepared,review,closure,entry=preflight(args)
    args.out.mkdir(parents=True,exist_ok=False)
    wire.save(args.out/'preflight.json',dict(schema='clock-native-run-preflight-v2',**closure,
        host_exe_sha256=HOST,host_source_sha256=ASM,owned_code_sha256=OWNED,entrypoint=f'{entry:08X}',
        entrypoint_matches_HostMain=True,source_spans_and_linked_audit_reproduced=True,
        retained_v1_fault_unmodified=True,independent_static_and_linked_reviews_pass=True,
        runner_sha256=digest(Path(__file__)),native_execution_requested=args.run))
    if not args.run:print(json.dumps(dict(preflight=True,cases=1560,host_version=2,native_executed=False)));return
    # Reuse the hash-frozen wire executor, never its v1 host/preflight defaults.
    # Add this wrapper to its execution pins as well as its frozen dependencies.
    wire.HOST,wire.ASM,wire.OWNED=HOST,ASM,OWNED
    wire.EXPECTED=dict(EXPECTED)
    wire.EXPECTED[str(Path(__file__).resolve().relative_to(ROOT).as_posix())]=digest(Path(__file__))
    try:
        wire.execute(args,prepared,review)
        report=args.out/'native-v1/report.json'
        wire.save(args.out/'v2-admission.json',dict(schema='clock-v2-owned-exit-run-binding-v1',
            host_exe_sha256=HOST,wire_report_sha256=digest(report),wire_contract_version=1,
            reviewed_host_version=2,old_host_executed=False,retained_v1_failure_unmodified=True))
    except BaseException as error:
        wire.save(args.out/'failure.json',dict(error=repr(error),true_failure_retained=True,host_version=2));raise
if __name__=='__main__':main()
