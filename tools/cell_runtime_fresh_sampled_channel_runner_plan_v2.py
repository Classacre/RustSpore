"""Freeze reviewed actual event-tail successor identities/dependencies; no native."""
import json,sys,hashlib
from pathlib import Path
import cell_runtime_fresh_sampled_channel_native_run_v2 as runner
ROOT=runner.ROOT;STAGE=ROOT/'target/source-cell'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 host=STAGE/'native-runtime-fresh-sampled-channel-root-v3';peer=host/'binding-independent-peer-linked-v3.json';r=json.loads(peer.read_text());assert r['static_pass'] and r['native_executed']is False
 assert sha(host/'carrier-x86.exe')=='062fd3485391ba64aeca8a6aec8d9e66e78fe8c518b3860dac190cacb8925164'
 assert sha(host/'host-x86.exe')=='9bd9528e97ad1bb30d860270deac6238b3dcc9d5e60e47e007fec74262cd4164'
 assert sha(host/'host-x86.map')=='a14b0965fcbd862463564c20aa1953791c8a5c792ea0bccd3ee6a5812893bfa3'
 oldplan=STAGE/'runtime-fresh-sampled-channel-runner-plan-v1/plan.json';old=json.loads(oldplan.read_text());files={Path(p)for p in old['pins']};assert all(sha(p)==value for p,value in old['pins'].items())
 files.update(Path(m.__file__).resolve()for m in list(sys.modules.values())if getattr(m,'__file__',None)and Path(m.__file__).resolve().parent==ROOT/'tools')
 dirs=['runtime-fresh-sampled-channel-runner-plan-v1','runtime-fresh-sampled-channel-native-run-v1','runtime-fresh-sampled-channel-event-tail-diagnosis-v1','native-runtime-fresh-sampled-channel-host-v6','native-runtime-fresh-sampled-channel-root-v3']
 files.update(p for folder in dirs for p in(STAGE/folder).rglob('*')if p.is_file());files.add(Path(__file__).resolve());files.add(Path(runner.__file__).resolve())
 assert all(p.name!='lib.rs' and 'font'not in p.name.lower()for p in files)
 plan=dict(schema='fresh-sampled-channel-event-tail-successor-frozen-runner-plan-v2',native_executed=False,source_proved_owned_event_tail_successor=True,host=str(host),runner_sha256=sha(runner.__file__),inputs=str(STAGE/'runtime-fresh-sampled-channel-inputs-v1'),linked_peer=str(peer),carrier_sha256=sha(host/'carrier-x86.exe'),owned_sha256=sha(host/'host-x86.exe'),map_sha256=sha(host/'host-x86.map'),failed_v1_attempt_preserved=str(STAGE/'runtime-fresh-sampled-channel-native-run-v1/failure.json'),source_diagnosis=str(STAGE/'runtime-fresh-sampled-channel-event-tail-diagnosis-v1/diagnosis.json'),scope='OnlytwoownedpreclockEventsShadowcopieschanged.Original10403B/model/inputs/verifier/sourceinspection/executionalgorithmunchanged. Newreviewedcohort, oldfailedattemptretained.',pins={str(p):sha(p)for p in sorted(files)})
 runner.PLAN.parent.mkdir(exist_ok=False)
 with runner.PLAN.open('x')as f:json.dump(plan,f,indent=2);f.write('\n')
 print(sha(runner.PLAN));print(len(files))
if __name__=='__main__':main()
