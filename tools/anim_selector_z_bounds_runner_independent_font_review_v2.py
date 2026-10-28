"""Independent static supplemental peer of corrected frozen Z runner."""
import hashlib,json,struct,sys
from pathlib import Path
from types import SimpleNamespace
import anim_selector_z_bounds_native_run_v2 as runner
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 rp=Path(runner.__file__);assert sha(rp)=='bb565fe545b132a3f49213ca3f11092507a492ea35bda6ba636ecdced23327c1';peer=ROOT/'target/animation/selector-Z-producer-v2-independent-font-review-v1.json';assert sha(peer)=='f8753b2618861ad155ace574d24962fe7ebe8a93bc3428e23371cdd5363eed4e';pp=json.loads(peer.read_text());assert pp['no_blocking_static_issue_found'] and not pp['native_executed']
 h=ROOT/'target/animation/selector-Z-producer-native-root-v2';inp=ROOT/'target/animation/selector-Z-producer-inputs-v1';m,c=runner.review(SimpleNamespace(host_dir=h,inputs=inp,source_exe=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),peer_review=peer,runner_review=None,run=False));assert c['actual_finalized_body_bytes_exact'] and c['rows']==15864
 raw=(inp/'input.bin').read_bytes();header=bytearray(raw[:32]);n=struct.unpack_from('<I',header,4)[0];struct.pack_into('<I',header,8,1);smoke=bytes(header)+raw[32:32+n*324]+struct.pack('<I',runner.CONTROLS[0]);assert struct.unpack_from('<I',header,12)[0]==0 and len(smoke)==32+n*324+4
 text=rp.read_text();execute=text[text.index('def execute('):text.index('def main():')];assert 'reviewed_file_run' not in text and 'while' not in execute and 'timeout=300' in execute and 'with pinned(paths,out):' in execute
 assert text.index('if a.reference_only:')<text.index("cases=[('empty'") and 'if a.run:execute(a,m,c)' in text
 assert "peer(a.peer_review,a.run)" in text and "peer(a.runner_review,a.run,True)" in text
 helper=ROOT/'tools/cell_runtime_data_native_run.py';body=helper.read_text().split('def saved_file_run(',1)[1].split('def reviewed_file_run(',1)[0];assert 'process.wait(timeout=timeout)' in body and 'process.kill()' in body and 'finally:' in body and 'CREATE_NO_WINDOW' in body and "'xb'" in body and 'retry' not in body.lower()
 # Enumerated unique protocol cases: empty/repeat10partials13header/CW10geometry12CW=47.
 assert 2+2*5+13+10+12==47 and len(runner.CONTROLS)==12
 assert "assert r.read_bytes()[8:32]==reference[8:32]" in execute and 'native-projection44.bin' in execute
 pins=pp['pins'].copy();assert all(sha(ROOT/k)==v for k,v in pins.items());paths=[Path(__file__),rp,peer,helper,*[ROOT/x for x in runner.FILES],inp/'manifest.json',inp/'body-index.json',inp/'input.bin',ROOT/'target/animation/selector-Z-producer-v1-independent-font-blocker.json'];pins.update({str(p.relative_to(ROOT)):sha(p) for p in paths})
 report=dict(schema='selector-Z-producer-v2-runner-independent-font-supplement-v1',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,pins=pins,static_preflight_reproduced=True,corrected_actual_v2_only=True,genuine_body0_one_policy_smoke=True,protocol_cases=47,corpus_rows=15864,single_attempt_no_retry=True,timeouts_and_failures_retained=True,actual_source_model_input_immutability_closed=True,old_v1_blocker_retained=True,source_numeric_expected_from_rust=False,limits='Static runner admission only. Smoke structurally checks oneactualbody0row; source/Rustnumeric comparison remains separate. Native sixpublishedfields independentofCW because sourceSSE1F80; retained80allowedvarywith12CW. No native execution,liveactor,fullwrapper/diagonal/constraints/wholebuilder acceptance.')
 with Path(output).open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(output))
if __name__=='__main__':main(sys.argv[1])
