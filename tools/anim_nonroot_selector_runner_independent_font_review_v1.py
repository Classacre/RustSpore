"""Independent frozen selector runner supplement; static only."""
import hashlib,json,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(out):
 runner=ROOT/'tools/anim_nonroot_selector_native_run_v1.py';assert sha(runner)=='c915d1e070944caa405229e491b0c96fb8bae6db544d3cdfdfec06a266789e3e';text=runner.read_text();peer=ROOT/'target/animation/nonroot-selector-independent-font-review-v1.json';assert sha(peer)=='b49320f9ef98f82637aaf32918054bde9517af90d3c57f44443c499f0dbb8f8d';old=json.loads(peer.read_text());assert old['no_blocking_static_issue_found'] and not old['native_executed']
 pins=old['pins'].copy();assert all(sha(ROOT/n)==v for n,v in pins.items());inp=ROOT/'target/animation/nonroot-selector-inputs-v3';protocol=ROOT/'target/animation/nonroot-selector-protocol-v2'
 import anim_nonroot_selector_native_run_v1 as r
 # Run static-only review including actual linked audit, genuine upstream joins and all guards.
 from types import SimpleNamespace
 args=SimpleNamespace(inputs=inp,host_dir=ROOT/'target/animation/nonroot-selector-native-root-v2',source_exe=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),peer_review=peer,runner_review=None,run=False)
 m,closure=r.review(args);assert closure['queries']==155996 and closure['actual_genuine_finalizer_body_bytes_exact']
 with (inp/'input.bin').open('rb') as f,(inp/'expected1032.bin').open('rb') as g:
  h=f.read(32);n,q=struct.unpack_from('<2I',h,4);assert struct.unpack_from('<I',h,16)[0]==0 and q==118;body0=h+f.read(n*324+q*32);gold0=g.read(q*1032);assert all(struct.unpack_from('<I',gold0,i*1032+8)[0]==0 for i in range(q))
  h=f.read(32);n,q=struct.unpack_from('<2I',h,4);records=f.read(n*324);queries=f.read(q*32);gold1=g.read(q*1032);header=bytearray(h);struct.pack_into('<I',header,8,1)
  assert bytes(header)+records+queries[32:64]==(protocol/'reference-input.bin').read_bytes();assert gold1[1032:2064]==(protocol/'reference-expected1032.bin').read_bytes();assert struct.unpack_from('<I',gold1,1032+8)[0]==2
 assert 'reviewed_file_run' not in text and 'saved_file_run' in text and 'timeout=300' in text
 assert 'if a.reference_only:' in text and text.index('if a.reference_only:')<text.index("positive=(PROTOCOL/") and 'else:print' in text and 'if a.run:execute' in text
 assert 'runner_peer(a.runner_review,a.run)' in text and 'peer(a.peer_review,a.run)' in text and "with pinned(paths,out):" in text
 helper=ROOT/'tools/cell_runtime_data_native_run.py';ht=helper.read_text();body=ht[ht.index('def saved_file_run('):ht.index('def reviewed_file_run(')];assert 'process.wait(timeout=timeout)' in body and 'process.kill()' in body and 'CREATE_NO_WINDOW' in body and 'finally:' in body and "'xb'" in body
 # Each call site is a single saved attempt; loops enumerate distinct cases, never retry failures.
 assert 'retry' not in body.lower();assert 'while' not in text[text.index('def execute('):text.index('def main():')]
 paths=[runner,peer,helper,ROOT/'crates/spore-core/src/anim_nonroot_selector.rs',ROOT/'crates/spore-core/examples/anim_nonroot_selector_verify.rs.pending',*[x for x in protocol.iterdir() if x.is_file()],inp/'manifest.json']
 pins.update({str(p.relative_to(ROOT)):sha(p) for p in paths})
 report=dict(schema='actual-nonroot-selector-runner-independent-font-supplement-v1',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,pins=pins,static_preflight_reproduced=True,genuine_body0_full118_zero_match_reference=True,genuine_body1_query1_positive_two_indices=True,source_expected_not_rust=True,single_attempt_no_retry=True,timeouts_retained=True,peer_and_source_and_input_pin_closure=True,framing_and_protocol_expectations_closed=True,protocols=44,limits='Static runner admission only. Reference-only preserves actual all-zero body0 results; no fallback. Full mode adds separately genuine positive memcpy path. CW/TOP and MXCSR control checked; FP sticky flags deliberately not asserted. No native/Rust arithmetic acceptance or actor association.')
 with Path(out).open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(out))
if __name__=='__main__':main(sys.argv[1])
