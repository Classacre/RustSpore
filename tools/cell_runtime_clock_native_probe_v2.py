"""Prepare/audit a new clock host with owned failure-stack restoration only.

The v1 packet and true crossing_equal failure remain frozen. The access fault
has no observed fault IP; fake-stack ExitProcess is an inferred cause only.
No compilation or native execution is performed by this wrapper.
"""
import hashlib
import json
from pathlib import Path
import sys
import cell_runtime_clock_native_probe as base

BASE_SHA='393af5d597c3587652c34e123c4c274bf676e66b99bd4fe0833be767c877e74b'
BASE_ASM='16b5ea19f5c942dfd608639ee1abe0430d28c8a11b482fb9e87f5935a721353f'
OLD='HostFailure::\npush 7\ncall _ExitProcess@4\n'
NEW=('HostFailure::\ncmp NativeStack,0\nje FailureStackReady\n'
     'mov esp,NativeStack\nFailureStackReady:\npush 7\ncall _ExitProcess@4\n')
original_host_source=base.host_source

def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def host_source(image):
    source=original_host_source(image)
    assert source.count(OLD)==1
    modified=source.replace(OLD,NEW)
    assert modified.replace(NEW,OLD)==source
    return modified

def main():
    assert digest(base.__file__)==BASE_SHA
    argv=sys.argv[1:];assert argv.count('--out')==1
    out=Path(argv[argv.index('--out')+1])
    auditing='--audit-exe'in argv
    if auditing:
        manifest=json.loads((out/'manifest.json').read_text())
        assert manifest['schema']=='bounded-original-cell-runtime-clock-host-v2'
        assert manifest['preparation_tool_sha256']==digest(__file__)
        assert manifest['base_generation_tool_sha256']==BASE_SHA
    base.host_source=host_source
    base.main()
    if auditing:return
    manifest=json.loads((out/'manifest.json').read_text())
    manifest.update(schema='bounded-original-cell-runtime-clock-host-v2',
        preparation_tool_sha256=digest(__file__),base_generation_tool_sha256=BASE_SHA,
        base_host_assembly_sha256=BASE_ASM,
        owned_only_change='HostFailure restores ESP from NativeStack when nonzero before invoking Windows ExitProcess. NativeStack zero retains the original real stack during pre-run initialization failures.',
        original_spans_unchanged=True,original_source_bytes_unchanged=146,
        prior_failure=dict(path='target/source-cell/runtime-clock-native-run-v1',
            case='crossing_equal',classification='Retained true native access violation, empty output; no retry authorized.',
            observed='Exit C0000005 after earlier standalone/reference/domain protocols passed. No fault IP or stack trace was captured.',
            inferred_cause='The excluded original crossing fallthrough reaches owned HostFailure while ESP is still on Frame800; calling Windows ExitProcess from that fake stack may fault. This version corrects that owned exit contract without claiming the cause was observed.'),
        failure_exit_contract='All HostFailure paths with saved NativeStack restore the real thread stack before any Windows call. Pre-run paths with NativeStack zero already use the real stack. ExitProcess is nonreturning; no caller-return/GPR ABI claim.',
        host_executed=False,compiled=False,root_review_required=True)
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({key:manifest[key]for key in ['schema','host_source_sha256','original_spans_unchanged','host_executed','compiled']},indent=2))
if __name__=='__main__':main()
