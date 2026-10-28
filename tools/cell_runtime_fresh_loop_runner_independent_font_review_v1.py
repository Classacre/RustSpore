"""Independent frozen fresh-loop runner supplemental static review."""
import hashlib,json,subprocess,sys
from pathlib import Path
import cell_runtime_fresh_loop_native_run_v1 as runner
ROOT=Path(__file__).resolve().parent.parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main(output):
 rp=Path(runner.__file__);assert sha(rp)=='7e8feaa83c11f850f4f3f6181c5ab6f8d41d6f3861f23a53179f4d6492baedd5';peer=ROOT/'target/source-cell/runtime-fresh-loop-independent-font-review-v2.json';assert sha(peer)=='39dc20bc274a0308132500278987f5df3deea7acfd168daed00ff55d43a9cb6b';pp=json.loads(peer.read_text());assert pp['no_blocking_static_issue_found'] and not pp['native_executed'];assert len(runner.HARD_PINS)==125
 for name,v in runner.HARD_PINS.items():assert sha(runner.resolve(name))==v,name
 runner.review(peer)
 preflight=ROOT/'target/source-cell/runtime-fresh-loop-independent-runner-preflight-v1';r=subprocess.run([sys.executable,str(rp),'--out',str(preflight),'--peer-review',str(peer)],cwd=ROOT,capture_output=True,text=True,check=True);assert 'Preflight PASS; no native execution'in r.stdout;p=json.loads((preflight/'preflight.json').read_text());assert p['native_executed']is False and p['execution_requested']is False and p['records']==300 and p['snapshots']==2700 and p['distinct_XMM_poison_loaded']is False
 text=rp.read_text();main=text[text.index('def main():'):text.index('def create_pin_dir')];run=main[main.index('if not a.run:'):];assert 'return' in run[:100] and 'assert a.peer_review and a.runner_review' in main and 'with pinned('in main and 'reviewed_file_run'not in text
 assert "execute('reference',raw[:256],0)"in text and "execute('valid_sequence',raw[:512],0)"in text and "for name in report['protocols']:execute"in text and "execute('complete_then_partial'"in text
 assert 'retries=0' in text and 'except BaseException' in text and "single_attempt=True"in text
 helper=ROOT/'tools/cell_runtime_data_native_run.py';saved=helper.read_text().split('def saved_file_run(',1)[1].split('def reviewed_file_run(',1)[0];assert 'process.wait(timeout=timeout)'in saved and 'process.kill()'in saved and 'CREATE_NO_WINDOW'in saved and 'finally:'in saved and 'retry'not in saved.lower()
 pins=pp['pins'].copy();pins.update(runner.HARD_PINS);paths=[Path(__file__),rp,peer,helper,preflight/'preflight.json'];pins.update({str(p.relative_to(ROOT)):sha(p)for p in paths})
 report=dict(schema='full-fresh-loop-hybrid-runner-independent-font-supplement-v1',no_blocking_static_issue_found=True,native_executed=False,host_executed=False,pins=pins,default_preflight_reproduced=True,literal_hardpins=125,records=300,snapshots=2700,native_snapshot_bytes=96,projection_snapshot_bytes=88,projection_record_bytes=792,wire_rejections=23,framing_cases=5,reference_requests=1,single_attempt_no_retry=True,timeouts_and_failure_logs_retained=True,source_metadata_and_derived_input_pins_closed=True,partial_frame_ABI_peer_required=True,distinct_XMM_poison_loaded=False,repeated_RunOnce_comparison='Entire864Bnativeoutputdeterminism only',limits='Static runner admission only. No automaticretry evenhighCRTmappingexit10; actualrootnative/numericalconformancepending. Corpusinputoperationsderivedfromacceptednative5resourcedurations/conservativeFractiondomain,notpredictedRustframeoutputs. BothactualEXEidentities andthisexactrunner requiredbyreports; before/afterpins includeallsource/model/inputs andeveryrequest. Explicitfreshborrowedresource/owner0/Repeat/fourzeromasks/isolatedglobals/SSE2policy only; no liveactor/sampling/events/registry admission.')
 with Path(output).open('x')as f:json.dump(report,f,indent=2);f.write('\n')
 print(sha(output))
if __name__=='__main__':main(sys.argv[1])
