"""Independent static supplement for frozen combined all-policy Z runner."""
import hashlib,json,sys,struct
from pathlib import Path
from types import SimpleNamespace
import anim_nonroot_selector_z_native_run_v1 as runner
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 rp=Path(runner.__file__);assert sha(rp)=='6b2851e713af4d5053da87752433a609d601ec5b0d8d3e925eba60c3e25e8031';peer=ROOT/'target/animation/nonroot-selector-Z-independent-font-review-v1.json';assert sha(peer)=='0bf877e32a714750b3eb5b0a4cb633a1b9d64c9995381d1a537f5211d4f16987';pp=json.loads(peer.read_text());assert pp['no_blocking_static_issue_found'] and not pp['native_executed'];h=ROOT/'target/animation/nonroot-selector-Z-native-root-v1';inp=ROOT/'target/animation/nonroot-selector-Z-inputs-v1-complete';m,c=runner.review(SimpleNamespace(host_dir=h,inputs=inp,source_exe=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'),peer_review=peer,runner_review=None,run=False))
 smokes=c['positive_Z_smokes'];assert {int(k)for k in smokes}=={0x4c,0x1040}
 for flag,(body,query)in [(0x1040,(1,70)),(0x4c,(243,62))]:
  row=smokes.get(flag,smokes.get(str(flag)));assert row['body']==body and row['query']==query
  with(inp/'input.bin').open('rb')as f,(inp/'expected1032.bin').open('rb')as e:
   f.seek(row['input_offset']);raw=f.read(32);n=struct.unpack_from('<I',raw,4)[0];assert struct.unpack_from('<I',raw,16)[0]==body;f.seek(row['input_offset']+32+n*324+query*32);q=struct.unpack('<8I',f.read(32));assert q[0]==flag and q[7]==query;e.seek(row['expected_offset']);gold=e.read(1032);assert struct.unpack_from('<3I',gold)==(body,query,1)
 text=rp.read_text();execute=text[text.index('def execute('):text.index('def main():')];assert 'reviewed_file_run' not in text and 'while' not in execute and 'with pinned(paths,out):'in execute and 'timeout=300'in execute
 assert 'if a.run:execute(a,m,c)'in text and text.index('if a.reference_only:')<text.index('smoke={}') and 'peer(a.peer_review,a.run)'in text and 'peer(a.runner_review,a.run,True)'in text
 helper=ROOT/'tools/cell_runtime_data_native_run.py';body=helper.read_text().split('def saved_file_run(',1)[1].split('def reviewed_file_run(',1)[0];assert 'process.wait(timeout=timeout)'in body and 'process.kill()'in body and 'finally:'in body and 'CREATE_NO_WINDOW'in body and 'retry'not in body.lower()
 assert 2+10+11+12+7+1+2*(3+12)==73
 pins=pp['pins'].copy();assert all(sha(ROOT/k)==v for k,v in pins.items());paths=[Path(__file__),rp,peer,helper,*[ROOT/x for x in runner.FILES],inp/'manifest.json'];pins.update({str(p.relative_to(ROOT)):sha(p)for p in paths})
 report=dict(schema='combined-selector-Z-runner-independent-font-supplement-v1',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,pins=pins,actual_preflight_reproduced=True,genuine_positive_smokes_closed=True,reference_body0_queries=122,protocols=73,corpus_queries=161284,single_attempt_no_retry=True,timeouts_and_failure_logs_retained=True,source_model_derived_input_immutability_closed=True,expected_not_from_rust=True,partial_caller_scope_preserved=True,limits='Static runner admission only. Nativecomparison pending. Actualsourcebody/querysmokes1040body1/q70and4Cbody243/q62eachselect1; no fabricatedfallback. Originalproducerbeforequeries; twelveCWgold reliesonacceptednativepublisherfieldinvariance. No AEownership/constructor/pruning/liveactor/fullcaller/wholebuilder admission.')
 with Path(output).open('x')as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(output))
if __name__=='__main__':main(sys.argv[1])
