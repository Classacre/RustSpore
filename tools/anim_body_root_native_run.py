"""Execute only the separately reviewed, pinned native root-selection fixture."""
import argparse
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import time

import anim_body_root_native_probe as probe
from cell_schema_probe import SourceImage, SOURCE_SHA256

HOST = '85a10b416af2be9dfcddb7bdb088364b17be378f91f2b9dcf95d257db8c5e61f'
ASM = '55eb0c0009e47c17dbc0dda934ba0c82ab5dbbb25e211df75f2afc6586136ce4'
INPUT = 'abaf288a0725b7d883f8fb7a270ee381ff55f629f734d94ec3b2c7ac467ad9fd'
EXPECTED = '0388646b646ea94479966b037aa04ff2e3ef6efcf8ff4b944fcf203c2dfd2cef'
ROWS = 8624


def sha(data):
    return hashlib.sha256(data).hexdigest()


def saved_run(host, request, folder, label, timeout_seconds=60):
    """Persist request before launch and every available process result on error."""
    with (folder/f'{label}-input.bin').open('xb') as stream:
        stream.write(request)
    started = time.perf_counter()
    process, output, errors, failure, cleanup_error = None, b'', b'', None, None
    timed_out = False
    try:
        process = subprocess.Popen([str(host)], stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW)
        output, errors = process.communicate(input=request, timeout=timeout_seconds)
    except BaseException as exc:
        failure = exc
        timed_out = isinstance(exc, subprocess.TimeoutExpired)
        output, errors = getattr(exc,'stdout',None) or b'', getattr(exc,'stderr',None) or b''
        if process is not None:
            try:
                if process.poll() is None:
                    process.kill()
                output, errors = process.communicate(timeout=5)
            except BaseException as cleanup:
                cleanup_error = repr(cleanup)
                output = getattr(cleanup,'stdout',None) or output
                errors = getattr(cleanup,'stderr',None) or errors
    finally:
        status = dict(exit=None if process is None else process.returncode, timeout=timed_out,
            exception=None if failure is None else repr(failure), cleanup_exception=cleanup_error,
            input_bytes=len(request),output_bytes=len(output),stderr_bytes=len(errors),
            elapsed_seconds=time.perf_counter()-started,hidden_binary_pipes=True)
        with (folder/f'{label}-output.bin').open('xb') as stream: stream.write(output)
        with (folder/f'{label}-stderr.bin').open('xb') as stream: stream.write(errors)
        with (folder/f'{label}-status.json').open('x') as stream:
            json.dump(status,stream,indent=2);stream.write('\n')
    if failure is not None:
        raise failure
    return subprocess.CompletedProcess([str(host)],process.returncode,output,errors)


def reviewed_run(host, request, folder, label, timeout_seconds=60):
    """Retry only the host's empty allocation-only exit10, never native failure."""
    attempts=[]
    try:
        for attempt in range(1,9):
            prefix=f'{label}-attempt{attempt:02}'
            try:
                result=saved_run(host,request,folder,prefix,timeout_seconds)
            except BaseException as exc:
                attempts.append(dict(attempt=attempt,label=prefix,retried=False,exception=repr(exc)))
                raise
            allocation_only=(result.returncode==10 and not result.stdout and not result.stderr)
            attempts.append(dict(attempt=attempt,label=prefix,exit=result.returncode,
                output_bytes=len(result.stdout),stderr_bytes=len(result.stderr),
                allocation_only_exit10=allocation_only,retried=allocation_only and attempt<8))
            if not allocation_only:
                return result
        return result
    finally:
        with (folder/f'{label}-attempts.json').open('x') as stream:
            json.dump(dict(maximum_attempts=8,attempts=attempts,
                retry_policy='Only exit10 with empty stdout and stderr; pre-ReadRecord VirtualAlloc-only host exit.'),stream,indent=2)
            stream.write('\n')


@contextmanager
def pinned_artifacts(paths, folder):
    pins = {str(path.resolve()):sha(path.read_bytes()) for path in paths}
    with (folder/'execution-pins-v1.json').open('x') as stream:
        json.dump(pins,stream,indent=2);stream.write('\n')
    failure = None
    try:
        yield
    except BaseException as exc:
        failure = repr(exc)
        raise
    finally:
        changed, read_errors = [], []
        for name,digest in pins.items():
            try:
                if sha(Path(name).read_bytes()) != digest: changed.append(name)
            except BaseException as exc:
                changed.append(name);read_errors.append(dict(path=name,error=repr(exc)))
        report = dict(pinned_files=len(pins),unchanged=not changed,changed_files=changed,
            read_errors=read_errors,operation_exception=failure)
        with (folder/'immutability-v1.json').open('x') as stream:
            json.dump(report,stream,indent=2);stream.write('\n')
        if changed:
            raise AssertionError('Pinned artifacts changed: '+repr(changed))


def projection(data, output):
    assert len(data) % probe.INPUT_BYTES == 0
    count = len(data)//probe.INPUT_BYTES
    assert len(output) == count*probe.OUTPUT_BYTES
    result, controls, sticky = bytearray(), {}, {}
    for row in range(count):
        request = data[row*probe.INPUT_BYTES:(row+1)*probe.INPUT_BYTES]
        reply = output[row*probe.OUTPUT_BYTES:(row+1)*probe.OUTPUT_BYTES]
        mxcsr, cw, sw = struct.unpack_from('<IHH', reply, probe.PROJECTION_BYTES)
        assert mxcsr & ~0x3F == 0x1F80 and cw == struct.unpack_from('<H', request)[0] and sw == 0
        controls[f'{cw:04X}'] = controls.get(f'{cw:04X}', 0)+1
        sticky[f'{mxcsr & 0x3F:02X}'] = sticky.get(f'{mxcsr & 0x3F:02X}', 0)+1
        result.extend(reply[:probe.PROJECTION_BYTES])
    return bytes(result), controls, sticky


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--host-dir', type=Path, required=True)
    p.add_argument('--input-dir', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    target = (Path(__file__).resolve().parent.parent/'target').resolve()
    for path in [a.host_dir, a.input_dir, a.out]:
        path.resolve().relative_to(target)
    manifest_raw = (a.host_dir/'manifest.json').read_bytes()
    manifest = json.loads(manifest_raw)
    input_manifest_raw = (a.input_dir/'manifest.json').read_bytes()
    prepared = json.loads(input_manifest_raw)
    assert manifest['schema'] == 'bounded-original-body-root-selection-host-v1'
    assert manifest['host_exe_sha256'] == HOST and manifest['host_source_sha256'] == ASM
    assert prepared['rows'] == ROWS and prepared['hashes']['inputs.bin'] == INPUT
    assert prepared['hashes']['python-output.bin'] == EXPECTED
    assert sha((a.host_dir/'host-x86.exe').read_bytes()) == HOST
    assert sha((a.host_dir/'host-x86.asm').read_bytes()) == ASM
    source = SourceImage(Path(manifest['source_path']))
    assert probe.audit_source(source) == manifest['code_allowlist']
    assert probe.host_source(source)[0] == (a.host_dir/'host-x86.asm').read_text()
    assert probe.linked_audit(a.host_dir/'host-x86.exe',a.host_dir/'host-x86.map') == manifest['linked_audit']
    for name, digest in prepared['hashes'].items():
        assert sha((a.input_dir/name).read_bytes()) == digest
    data = (a.input_dir/'inputs.bin').read_bytes()
    expected = (a.input_dir/'python-output.bin').read_bytes()
    assert len(data) == ROWS*probe.INPUT_BYTES and len(expected) == ROWS*probe.PROJECTION_BYTES
    index = json.loads((a.input_dir/'input-index.json').read_text())
    a.out.mkdir(parents=True,exist_ok=False)
    (a.out/'reviewed-host-manifest.json').write_bytes(manifest_raw)
    (a.out/'reviewed-input-manifest.json').write_bytes(input_manifest_raw)
    tools = Path(__file__).resolve().parent
    tool_names = ['anim_body_root_native_run.py','anim_body_root_native_probe.py',
        'anim_body_root_fixture.py','anim_body_root_compare.py','cell_schema_probe.py',
        'native_crt_acos_probe.py','cell_native_math_probe.py','anim_body_source_probe.py',
        'anim_body_pose_probe.py','native_x87_probe.py','anim_body_hierarchy_probe.py',
        'native_archive_index_probe.py','creature_header_probe.py','dbpf_extract.py']
    paths = [Path(manifest['source_path']),a.input_dir/'manifest.json',
        a.out/'reviewed-host-manifest.json',a.out/'reviewed-input-manifest.json',
        *[a.input_dir/name for name in prepared['hashes']],
        *[path for path in a.host_dir.iterdir() if path.is_file()],
        *[tools/name for name in tool_names]]
    workspace = tools.parent
    paths += [workspace/'crates/spore-core/src/anim_body_topology.rs',
        workspace/'re/anim-body-topology-native.md']
    paths += [workspace/'crates/spore-core/src'/name for name in [
        'anim_body_capabilities.rs','anim_body_source.rs','anim_order.rs',
        'native_sse.rs','native_x87.rs','creature_data.rs','part_capabilities.rs']]
    for name in ['anim_body_root_verify','anim_body_topology_source_verify']:
        for suffix in ['.rs','.rs.pending']:
            path = workspace/'crates/spore-bevy/examples'/(name+suffix)
            if path.is_file(): paths.append(path)
    with pinned_artifacts(paths,a.out):
        host = (a.host_dir/'host-x86.exe').resolve()
        started = time.perf_counter()
        for family in ['source_constructor_no_optional_driver','synthetic_candidate_vote_reset',
            'synthetic_presence_only_nearest','synthetic_overflow_distances',
            'synthetic_unset_minimum_height','synthetic_unrolled_lane_rounding_witness',
            'synthetic_tail_lane_rounding_witness','synthetic_allspine_chain255',
            'synthetic_allspine_star255','synthetic_midpoint_nonspine_parent','synthetic_random_255']:
            wanted = 2 if family in {'synthetic_allspine_chain255','synthetic_allspine_star255',
                'synthetic_midpoint_nonspine_parent'} else 1 if family in {
                'synthetic_presence_only_nearest','synthetic_overflow_distances',
                'synthetic_unrolled_lane_rounding_witness','synthetic_tail_lane_rounding_witness'} else 0
            row = next(i for i,v in enumerate(index) if v['family']==family and v['policy']==wanted)
            request = data[row*probe.INPUT_BYTES:(row+1)*probe.INPUT_BYTES]
            completed = reviewed_run(host,request,a.out,'smoke-'+family)
            assert completed.returncode==0 and not completed.stderr
            actual,_,_ = projection(request,completed.stdout)
            assert actual == expected[row*probe.PROJECTION_BYTES:(row+1)*probe.PROJECTION_BYTES]
        smoke = data[:probe.INPUT_BYTES]
        first = reviewed_run(host,smoke,a.out,'protocol-reference')
        assert first.returncode==0 and not first.stderr
        negatives = [('empty',b'',0,b'')]
        for n in [1,15,16,probe.INPUT_BYTES-1]:
            negatives += [(f'partial{n}',smoke[:n],7,b''),
                          (f'complete_then_partial{n}',smoke+smoke[:n],7,first.stdout)]
        for name,at,word in [('reservedPC',0,0x17F),('unmaskedCW',0,0x27E),
            ('reservedCW',0,0x227F),('reservedHeaderWord',0,0x1027F),('countZero',4,0),
            ('count256',4,256),('policy4',8,4),('reserved12',12,1),('rootHasParent',16,0),
            ('rootMissingNonzeroSpine',24,0),('rootIsFootWithoutAncestor',24,12),
            ('invalidRootSymmetry',44,256),('nonfiniteRootCenter',28,0x7F800000),
            ('invalidForwardParent',48,255),('secondRoot',48,0xFFFFFFFF),('selfCycle',48,1)]:
            changed = bytearray(smoke); struct.pack_into('<I',changed,at,word)
            negatives.append((name,bytes(changed),7,b''))
        checks = a.out/'protocol'; checks.mkdir()
        protocols = []
        for label,request,status,reply in negatives:
            completed = reviewed_run(host,request,checks,label)
            assert completed.returncode==status and completed.stdout==reply and not completed.stderr,label
            protocols.append(dict(case=label,exit=status,exact=True))
        (a.out/'protocol-checks.json').write_text(json.dumps(protocols,indent=2)+'\n')
        completed = reviewed_run(host,data,a.out,'corpus')
        assert completed.returncode==0 and not completed.stderr
        actual,controls,sticky = projection(data,completed.stdout)
        (a.out/'output.bin').write_bytes(completed.stdout)
        (a.out/'native-projection.bin').write_bytes(actual)
        problems = [i for i in range(ROWS) if actual[i*probe.PROJECTION_BYTES:(i+1)*probe.PROJECTION_BYTES] != expected[i*probe.PROJECTION_BYTES:(i+1)*probe.PROJECTION_BYTES]]
        report = dict(schema='reviewed-native-body-root-selection-run-v1',rows=ROWS,
            exact_python_match=not problems,problem_rows=problems,protocol_checks=len(protocols),
            projection_sha256=sha(actual),python_projection_sha256=EXPECTED,output_sha256=sha(completed.stdout),
            input_sha256=INPUT,host_exe_sha256=HOST,source_sha256=SOURCE_SHA256,
            controls=controls,sse_sticky_flags=sticky,elapsed_seconds=time.perf_counter()-started,
            hidden_binary_pipes=True,original_modules_loaded=False,retail_process_started=False,gui_used=False,
            scope='Native root-vote/policy region only, with supplied bounded runtime blocks and fixed vector providers; no optional foot construction, anchor/localquaternion/capabilityrelabel/selector/runtimepose milestone.')
        (a.out/'run.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))
        assert not problems



if __name__ == '__main__':
    main()
